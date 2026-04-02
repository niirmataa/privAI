//! Mempool — kolejka niepotwierdzonych transakcji.
//!
//! Każda transakcja wchodząca do mempoola MUSI mieć ważny podpis Falcon.
//! Zero Trust na wejściu — bez podpisu = odrzucenie.
//!
//! Flow:
//! 1. Użytkownik wysyła Tx przez Tor do węzła
//! 2. Węzeł weryfikuje podpis Falcon
//! 3. Jeśli poprawny → dodaje do mempoola
//! 4. Gossip: propaguje do innych peerów
//! 5. Lider pobiera tx z mempoola do Proposal

use std::collections::{HashMap, HashSet, VecDeque};

use privai_chain::{Hash32, Transaction, CanonicalEncode};

/// Maksymalny rozmiar mempoola (liczba transakcji).
pub const MAX_MEMPOOL_SIZE: usize = 10_000;

/// Maksymalny wiek transakcji w mempoolu (ms).
/// Po tym czasie tx jest usuwany jako stale.
pub const MAX_TX_AGE_MS: u64 = 300_000; // 5 minut

/// Pojedyncza transakcja w mempoolu z metadanymi.
#[derive(Clone, Debug)]
pub struct MempoolEntry {
    pub tx: Transaction,
    pub received_at_ms: u64,
    pub tx_hash: Hash32,
    /// Hash klucza publicznego nadawcy (z podpisu Falcon).
    /// Używany do deduplikacji i anti-spam.
    pub sender_pk_hash: Hash32,
}

/// Mempool — kolejka niepotwierdzonych transakcji.
pub struct Mempool {
    /// Kolejka FIFO transakcji
    entries: VecDeque<MempoolEntry>,
    /// Indeks po tx_hash (do szybkiego lookupu i deduplikacji)
    by_hash: HashSet<Hash32>,
    /// Indeks po sender_pk_hash (do rate-limit per user)
    by_sender: HashMap<Hash32, u32>,
    /// Maksymalna liczba tx na sendera (anti-spam)
    max_per_sender: u32,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            by_hash: HashSet::new(),
            by_sender: HashMap::new(),
            max_per_sender: 100,
        }
    }

    /// Dodaje transakcję do mempoola.
    /// Zwraca true jeśli dodano, false jeśli odrzucono.
    pub fn insert(&mut self, entry: MempoolEntry) -> bool {
        // Sprawdź czy już jest (deduplikacja)
        if self.by_hash.contains(&entry.tx_hash) {
            eprintln!("[mempool] rejected: duplicate tx {:?}", &entry.tx_hash[..8]);
            return false;
        }

        // Sprawdź limit na sendera (anti-spam)
        let sender_count = *self.by_sender.get(&entry.sender_pk_hash).unwrap_or(&0);
        if sender_count >= self.max_per_sender {
            eprintln!(
                "[mempool] rejected: sender {:?} exceeded limit ({}/{})",
                &entry.sender_pk_hash[..8],
                sender_count,
                self.max_per_sender
            );
            return false;
        }

        // Sprawdź rozmiar mempoola — evict oldest if full
        if self.entries.len() >= MAX_MEMPOOL_SIZE {
            if let Some(oldest) = self.entries.pop_front() {
                self.by_hash.remove(&oldest.tx_hash);
                if let Some(count) = self.by_sender.get_mut(&oldest.sender_pk_hash) {
                    *count = count.saturating_sub(1);
                }
            }
        }

        // Dodaj do mempoola
        self.by_hash.insert(entry.tx_hash);
        *self.by_sender.entry(entry.sender_pk_hash).or_insert(0) += 1;
        self.entries.push_back(entry);

        true
    }

    /// Pobiera do `max` transakcji z mempoola do włączenia w blok.
    /// Zwraca transakcje i usuwa je z mempoola.
    pub fn take_for_block(&mut self, max: usize) -> Vec<Transaction> {
        let count = max.min(self.entries.len());
        let mut txs = Vec::with_capacity(count);

        for _ in 0..count {
            if let Some(entry) = self.entries.pop_front() {
                self.by_hash.remove(&entry.tx_hash);
                if let Some(count) = self.by_sender.get_mut(&entry.sender_pk_hash) {
                    *count = count.saturating_sub(1);
                }
                txs.push(entry.tx);
            }
        }

        txs
    }

    /// Usuwa przestarzałe transakcje. O(n) — retain zamiast O(n²) remove-by-index.
    pub fn evict_stale(&mut self, current_time_ms: u64) {
        let stale_hashes: Vec<Hash32> = self.entries.iter()
            .filter(|e| current_time_ms.saturating_sub(e.received_at_ms) > MAX_TX_AGE_MS)
            .map(|e| e.tx_hash)
            .collect();

        for hash in &stale_hashes {
            self.by_hash.remove(hash);
        }

        self.entries.retain(|e| {
            if current_time_ms.saturating_sub(e.received_at_ms) > MAX_TX_AGE_MS {
                if let Some(c) = self.by_sender.get_mut(&e.sender_pk_hash) {
                    *c = c.saturating_sub(1);
                }
                false
            } else {
                true
            }
        });
    }

    /// Liczba transakcji w mempoolu.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Czy mempool jest pusty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Sprawdza czy transakcja jest w mempoolu.
    pub fn contains(&self, tx_hash: &Hash32) -> bool {
        self.by_hash.contains(tx_hash)
    }

    /// Weryfikuje podpisy Falcon w transakcji (anti-spam, CPU-intensive).
    ///
    /// Każdy InputAuth zawiera listę signer_pks i signatures.
    /// Każdy podpis jest weryfikowany kluczem publicznym nadawcy.
    ///
    /// UWAGA: Ta operacja jest kosztowna CPU — powinna być wywoływana
    /// POZA głównym wątkiem konsensusu (np. w dedykowanym tokio::spawn).
    ///
    /// Zwraca true jeśli wszystkie podpisy są ważne, false jeśli którykolwiek jest niepoprawny.
    pub fn verify_tx_signatures(tx: &Transaction) -> bool {
        let core = tx.core();
        let tx_hash = tx.tx_id();

        for (i, auth) in core.auth.iter().enumerate() {
            if auth.signer_pks.is_empty() || auth.signatures.is_empty() {
                eprintln!(
                    "[mempool] tx {:?} auth[{}]: missing signer_pks or signatures",
                    &tx_hash[..8], i
                );
                return false;
            }

            if auth.signer_pks.len() != auth.signatures.len() {
                eprintln!(
                    "[mempool] tx {:?} auth[{}]: signer_pks/signatures count mismatch ({} vs {})",
                    &tx_hash[..8], i, auth.signer_pks.len(), auth.signatures.len()
                );
                return false;
            }

            // Weryfikuj każdy podpis Falcon
            for (j, (pk, sig)) in auth.signer_pks.iter().zip(auth.signatures.iter()).enumerate() {
                if pk.is_empty() || sig.is_empty() {
                    eprintln!(
                        "[mempool] tx {:?} auth[{}][{}]: empty pk or sig",
                        &tx_hash[..8], i, j
                    );
                    return false;
                }

                // Używamy tx_hash jako wiadomości do weryfikacji
                // (zgodnie z konwencją: podpis = falcon_sign(sk, tx_hash))
                if let Err(e) = nxms_transport::crypto::falcon_verify(pk, &tx_hash, sig) {
                    eprintln!(
                        "[mempool] tx {:?} auth[{}][{}]: falcon_verify failed: {}",
                        &tx_hash[..8], i, j, e
                    );
                    return false;
                }
            }
        }

        true
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use privai_chain::{Transaction, TransferNoteTx, TxCore, TX_TYPE_TRANSFER_NOTE};

    fn dummy_tx(seed: u8) -> Transaction {
        Transaction::TransferNote(TransferNoteTx {
            core: TxCore {
                version: 0,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: Vec::new(),
                input_nullifiers: Vec::new(),
                outputs: Vec::new(),
                fee: seed as u64,
                statement_commit: [seed; 32],
                auth: Vec::new(),
            },
        })
    }

    #[test]
    fn mempool_insert_and_take() {
        let mut mempool = Mempool::new();

        let tx = dummy_tx(1);
        let entry = MempoolEntry {
            tx_hash: tx.tx_id(),
            tx: tx.clone(),
            received_at_ms: 1000,
            sender_pk_hash: [10; 32],
        };

        assert!(mempool.insert(entry));
        assert_eq!(mempool.len(), 1);
        assert!(mempool.contains(&tx.tx_id()));

        let taken = mempool.take_for_block(10);
        assert_eq!(taken.len(), 1);
        assert!(mempool.is_empty());
    }

    #[test]
    fn mempool_rejects_duplicates() {
        let mut mempool = Mempool::new();

        let tx = dummy_tx(2);
        let entry1 = MempoolEntry {
            tx_hash: tx.tx_id(),
            tx: tx.clone(),
            received_at_ms: 1000,
            sender_pk_hash: [20; 32],
        };
        let entry2 = MempoolEntry {
            tx_hash: tx.tx_id(),
            tx,
            received_at_ms: 2000,
            sender_pk_hash: [20; 32],
        };

        assert!(mempool.insert(entry1));
        assert!(!mempool.insert(entry2)); // duplicate rejected
        assert_eq!(mempool.len(), 1);
    }

    #[test]
    fn mempool_evicts_stale() {
        let mut mempool = Mempool::new();

        let entry = MempoolEntry {
            tx_hash: [99; 32],
            tx: dummy_tx(99),
            received_at_ms: 1000,
            sender_pk_hash: [30; 32],
        };

        mempool.insert(entry);
        assert_eq!(mempool.len(), 1);

        // Evict po 6 minut (ponad MAX_TX_AGE_MS = 5 min)
        mempool.evict_stale(1000 + 360_000);
        assert!(mempool.is_empty());
    }
}