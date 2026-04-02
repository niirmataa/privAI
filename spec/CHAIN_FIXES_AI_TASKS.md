# Zadania naprawcze chain/ledger/node — wytyczne dla AI agenta

**Kontekst**: Po audycie kodu privai-chain/ledger/node znaleziono 13 problemów. Ten dokument opisuje każde zadanie z dokładnymi lokalizacjami, oczekiwanym wynikiem i testami weryfikacyjnymi.

**Zasady ogólne**:
- Po każdym zadaniu: `cargo check` i `cargo test` w danym crate muszą przejść
- Nie zmieniaj publicznego API bez uzasadnienia
- Nie dodawaj nowych dependencji bez uzasadnienia
- Komentarze po polsku (konwencja projektu)
- Testy nazywaj opisowo: `test_<co>_<scenariusz>`

---

## ZADANIE 1 — Fix: testy ledgera nie kompilują się (BLOCKER)

### Problem
Test `mempool_and_block_flow_spends_input` w `privai-ledger/src/ledger.rs:477-548` nie kompiluje się po dwóch zmianach:
1. `BlockTemplate` zyskał pole `state_root: Hash32` (commit `acba255`) — test go nie podaje
2. `Ledger::apply_block()` wymaga teraz 3 argumentów `(&block, min_proof_coverage, &epoch_params)` — test podaje 2

### Pliki do edycji
- `privai-ledger/src/ledger.rs` — sekcja `#[cfg(test)] mod tests`

### Co zrobić
1. Przeczytaj test `mempool_and_block_flow_spends_input` (linia ~477)
2. Dodaj helper `fn test_epoch_params() -> EpochParams` z dużymi limitami:
   ```rust
   fn test_epoch_params() -> privai_chain::EpochParams {
       privai_chain::EpochParams {
           epoch_number: 0,
           start_height: 0,
           end_height: 1_000_000,
           min_validator_stake: 0,
           min_prover_bond: 0,
           min_fee: 0,
           max_block_bytes: 10_000_000,
           max_block_statements: 100_000,
           min_proof_coverage: 1,
       }
   }
   ```
3. W teście: po zbudowaniu `block` ale PRZED `apply_block`, oblicz prawidłowy `state_root`:
   ```rust
   // Symuluj zastosowanie TX na kopii snapshota aby obliczyć state_root
   let mut temp = ledger.snapshot().clone();
   for tx in &block_txs {
       crate::ledger::apply_transaction_local(tx, 1, &mut temp);
   }
   let state_root = crate::ledger::compute_state_root(&temp);
   ```
   Następnie użyj tego `state_root` w `BlockTemplate`.
4. Zmień wywołanie `apply_block` na: `ledger.apply_block(&block, 1, &test_epoch_params())`
5. Zrób analogicznie dla testu `ledger_rejects_marketplace_batch_double_spend` — tam nie ma `apply_block` ale jest `validate_transaction` które teraz wymaga `min_fee` (sprawdź czy to 0 w teście — powinno być OK bo test podaje `min_fee: 0`)

### Weryfikacja
```bash
cd /home/nxms-server/privAI/privai-ledger && cargo test
```
Wszystkie testy muszą przejść (PASS). Żaden nowy warning.

### Uwaga
Funkcja `apply_transaction_local` jest publiczna (linia 54-60 w ledger.rs) — używaj jej w teście do symulacji state_root. NIE duplikuj logiki.

---

## ZADANIE 2 — Fix: MarketplaceBatchTx failuje na generic Falcon auth check (BLOCKER)

### Problem
`validate_transaction()` w `privai-ledger/src/ledger.rs:159-186` wymaga teraz ważnych podpisów Falcon w `TxCore.auth` dla WSZYSTKICH transakcji. Ale `MarketplaceBatchTx` ma osobny mechanizm auth (`operator_sig`) i historycznie miał `auth: vec![]`. 

Nowa logika auth (linia 162-186) wykonuje się PRZED specyficznym sprawdzeniem `MarketplaceBatchTx` (linia 188), więc MarketplaceBatch z pustym `auth` failuje na `"auth[0]: missing signer_pks or signatures"`.

### Pliki do edycji
- `privai-ledger/src/ledger.rs` — funkcja `validate_transaction`

### Co zrobić
1. Przeczytaj `validate_transaction` (linia 144-249)
2. Przenieś generic Falcon auth check (linie 159-186) TAK żeby NIE dotyczył `MarketplaceBatchTx`
3. Logika powinna być:
   ```
   if MarketplaceBatchTx:
       sprawdź operator_sig (jak teraz, linie 188-213)
       sprawdź ticket nullifiers (jak teraz)
   else:
       sprawdź generic Falcon auth (linie 159-186)
       sprawdź inputs/nullifiers (jak teraz)
   ```
4. Konkretnie: przenieś blok `// Weryfikacja podpisów Falcon na TxCore.auth` (linie 159-186) do wewnątrz brancha `else` (po `} else {` na linii 214), PRZED sprawdzeniem inputów

### Weryfikacja
```bash
cd /home/nxms-server/privAI/privai-ledger && cargo test
```
- Test `ledger_rejects_marketplace_batch_double_spend` musi przejść (MarketplaceBatch z pustym auth)
- Test `mempool_and_block_flow_spends_input` musi przejść (TransferNote z pustym auth — w testach auth jest pusty, więc ten test też failnie jeśli wymusimy auth na TransferNote. Sprawdź i ewentualnie ustaw `min_fee: 0` i skipnij auth check dla testów bez prawdziwych kluczy)

### Uwaga krytyczna
W testach nie mamy prawdziwych kluczy Falcon. Dlatego testy tworzą TX z `auth: Vec::new()`. Masz dwie opcje:
- (A) **Preferowana**: Auth check skipuje TX z pustym auth (prototyp v0 — loguj warning). Dodaj na początku auth check: `if tx.core().auth.is_empty() { /* v0: skip, log warning */ }`. To zachowuje kompatybilność.
- (B) Generuj test Falcon keypair w testach (trudniejsze, wymaga `nxms_transport::crypto`).

Wybierz opcję (A) — jest pragmatyczna dla v0. Dodaj komentarz: `// v0: brak wymuszenia auth dla prototypu. Docelowo każdy TX musi mieć ważny auth.`

---

## ZADANIE 3 — Fix: signers/signatures ordering mismatch w QC (SECURITY)

### Problem
W `privai-node/src/node.rs` funkcja `receive_vote()` (linia ~464) zbiera głosy w:
- `entry.0` = `BTreeSet<Vec<u8>>` — POSORTOWANE po bajtach
- `entry.2` = `Vec<Vec<u8>>` — KOLEJNOŚĆ WSTAWIANIA

Przy budowaniu QC (linie 511-518 i 531-538):
```rust
signers: entry.0.iter().cloned().collect(),  // z BTreeSet — posortowane
signatures: entry.2.clone(),                  // z Vec — kolejność insert
```
`signers[i]` NIE odpowiada `signatures[i]` chyba że walidatorzy głosują w kolejności sortowania ich PK.

`verify_qc` w consensus_loop.rs (linia 611-670) iteruje `qc.signers.par_iter().zip(qc.signatures.par_iter())` — zakłada matching.

### Pliki do edycji
- `privai-node/src/node.rs` — typ prevotes/precommits i budowanie QC

### Co zrobić
1. Zmień typ vote tracking z `(BTreeSet<Vec<u8>>, u64, Vec<Vec<u8>>)` na uporządkowaną strukturę. Najprościej: `BTreeMap<Vec<u8>, Vec<u8>>` (pk → sig):
   ```rust
   // Stary typ:
   prevotes: HashMap<Hash32, (BTreeSet<Vec<u8>>, u64, Vec<Vec<u8>>)>,
   // Nowy typ:
   prevotes: HashMap<Hash32, (BTreeMap<Vec<u8>, Vec<u8>>, u64)>,
   ```
2. Przy wstawianiu głosu:
   ```rust
   let entry = self.prevotes.entry(vote.block_hash)
       .or_insert_with(|| (BTreeMap::new(), 0));
   if !entry.0.contains_key(&vote.validator_pk) {
       entry.0.insert(vote.validator_pk.clone(), vote.falcon_sig.clone());
       entry.1 += voter_stake;
   }
   ```
3. Przy budowaniu QC:
   ```rust
   signers: entry.0.keys().cloned().collect(),
   signatures: entry.0.values().cloned().collect(),
   ```
   Teraz oba są w tej samej kolejności (BTreeMap sortuje po key).

4. Zrób to samo dla `precommits`.

5. Zaktualizuj `open_with_components` (inicjalizacja HashMap::new()) i `finalize_block_with_qc` / `receive_view_change` (clear).

### Weryfikacja
```bash
cd /home/nxms-server/privAI/privai-node && cargo test
```
- Test `node_builds_qc_on_threshold_votes` musi przejść
- Sprawdź ręcznie: w teście wypisz `qc.signers[0]` i `qc.signatures[0]` — muszą odpowiadać temu samemu walidatorowi

### Test dodatkowy do napisania
```rust
#[test]
fn qc_signers_and_signatures_are_aligned() {
    // Utwórz 3 walidatorów z różnymi PK
    // Wyślij głosy w odwrotnej kolejności sortowania
    // Zbuduj QC
    // Sprawdź: signers[i] i signatures[i] pasują do tego samego walidatora
}
```

---

## ZADANIE 4 — Fix: falcon_verify wywołane z pk_hash zamiast prawdziwym PK (SECURITY)

### Problem
W wielu miejscach `falcon_verify(pk, msg, sig)` dostaje 32-bajtowy **hash** klucza zamiast prawdziwego 1793-bajtowego **klucza publicznego Falcon**. To sprawia że weryfikacja zawsze failuje na prawdziwych podpisach.

Lokalizacje:
1. `privai-node/src/node.rs:272` — `create_vote_for_proposal` ustawia `validator_pk: self.config.node_pk_hash.to_vec()` (32B hash)
2. `privai-node/src/consensus_loop.rs:185` — `falcon_verify(&block.header.proposer_pk_hash, ...)` — 32B hash
3. `privai-node/src/consensus_loop.rs:502` — `falcon_verify(&sender_pk_hash, ...)` — 32B hash

### Pliki do edycji
- `privai-node/src/node.rs` — `create_vote_for_proposal`
- `privai-node/src/consensus_loop.rs` — proposal verify, PeersList verify

### Co zrobić

**4a.** W `create_vote_for_proposal` (node.rs:267-274):
```rust
// BYŁO:
validator_pk: self.config.node_pk_hash.to_vec(),
// POWINNO BYĆ:
validator_pk: self.config.node_sig_pk.clone(),
```
`node_sig_pk` to prawdziwy Falcon PK. Hash zawsze można obliczyć z PK ale nie odwrotnie.

**4b.** W `consensus_loop.rs` proposal verify (linia ~185): Trzeba resolver `proposer_pk_hash` → prawdziwy PK. Opcje:
- (A) **Preferowana**: Lookup w `config.validators` — dodaj `sig_pk: Vec<u8>` do `ValidatorConfig` i szukaj po `pk_hash`. Wtedy:
  ```rust
  let proposer_pk = self.node.config().validators.iter()
      .find(|v| v.pk_hash == block.header.proposer_pk_hash)
      .map(|v| &v.sig_pk);
  ```
- (B) Lookup w PeerBook (już jest sig_pk_b64)

Jeśli wybierasz (A), dodaj pole `pub sig_pk: Vec<u8>` do `ValidatorConfig` w `privai-node/src/config.rs`. Zaktualizuj `NodeConfig::example()`.

**4c.** PeersList verify (consensus_loop.rs:502): Zamień `sender_pk_hash` na prawdziwy PK z PeerBook:
```rust
let sender_pk = find_peer_pk_by_hash(&self.peer_book, &sender_pk_hash);
```
Albo z `config.validators` jak w 4b.

### Weryfikacja
```bash
cd /home/nxms-server/privAI/privai-node && cargo test
```
- Istniejące testy muszą przejść
- `cargo check` bez nowych warningów

### Uwaga
To jest duża zmiana. Jeśli dodajesz `sig_pk` do `ValidatorConfig`:
- Musisz zaktualizować KAŻDY test który tworzy `ValidatorConfig`
- `config.rs:67-72` (`NodeConfig::example()`) musi mieć placeholder `sig_pk`
- Serialization (toml/serde) musi obsłużyć nowe pole

---

## ZADANIE 5 — Fix: chain_id check używa złego error variant

### Problem
`privai-ledger/src/ledger.rs:260-263` — reuse'uje `InvalidBlockHeight` dla chain_id mismatch.

### Pliki do edycji
- `privai-ledger/src/error.rs` — dodaj nowy variant
- `privai-ledger/src/ledger.rs` — użyj go

### Co zrobić
1. W `error.rs` dodaj:
   ```rust
   #[error("chain_id mismatch: expected {expected}, got {actual}")]
   InvalidChainId { expected: u32, actual: u32 },
   ```
2. W `ledger.rs:260-263` zamień na:
   ```rust
   if block.header.chain_id != snapshot.chain_id {
       return Err(ValidationError::InvalidChainId {
           expected: snapshot.chain_id,
           actual: block.header.chain_id,
       });
   }
   ```

### Weryfikacja
```bash
cd /home/nxms-server/privAI/privai-ledger && cargo test
```

---

## ZADANIE 6 — Fix: block size mierzony przez serde_json

### Problem
`privai-ledger/src/ledger.rs:284` — `serde_json::to_vec(block)` serializuje cały blok do JSON przy każdej walidacji. Kosztowne i niedokładne (JSON != canonical).

### Pliki do edycji
- `privai-ledger/src/ledger.rs` — `validate_block`

### Co zrobić
Zamień:
```rust
let block_size = serde_json::to_vec(block).map(|b| b.len()).unwrap_or(0);
```
Na:
```rust
use privai_chain::CanonicalEncode;
let block_size = block.to_canonical_bytes().len();
```
`Block` implementuje `CanonicalEncode` w privai-chain. Sprawdź to: `grep "impl CanonicalEncode for Block" privai-chain/src/consensus.rs`.

Jeśli `Block` NIE implementuje `CanonicalEncode`, użyj `BlockHeader`:
```rust
let header_size = block.header.to_canonical_bytes().len();
// + przybliżony rozmiar body
let approx_size = header_size + block.body.txs.len() * 1024; // rough estimate
```

### Weryfikacja
```bash
cd /home/nxms-server/privAI/privai-ledger && cargo test
```

---

## ZADANIE 7 — Fix: state_sync continue zamiast break

### Problem
`privai-node/src/state_sync.rs:200-208` — `continue` pomija blok z błędnym `prev_hash` ale następne bloki też będą miały zły `prev_hash`. Powinien być `break`.

### Pliki do edycji
- `privai-node/src/state_sync.rs`

### Co zrobić
Zamień `continue` na `break` w linii ~208:
```rust
if block.header.prev_block_hash != expected_prev_hash {
    eprintln!(
        "[sync] REJECTED block at height={}: prev_hash mismatch — stopping sync",
        block.header.height,
    );
    break;  // ← było continue
}
```

### Weryfikacja
```bash
cd /home/nxms-server/privAI/privai-node && cargo test
```

---

## ZADANIE 8 — Fix: RocksDB zbyt duże bufory

### Problem
`privai-ledger/src/store.rs:105-108` — 256MB write buffer × 16 = 4GB RAM minimum.

### Pliki do edycji
- `privai-ledger/src/store.rs`

### Co zrobić
Zmień:
```rust
cf_opts.set_max_write_buffer_number(4);           // było 16
cf_opts.set_write_buffer_size(16 * 1024 * 1024);  // było 256MB → 16MB
cf_opts.set_target_file_size_base(16 * 1024 * 1024); // było 64MB → 16MB
cf_opts.set_max_bytes_for_level_base(64 * 1024 * 1024); // było 512MB → 64MB
```

### Weryfikacja
```bash
cd /home/nxms-server/privAI/privai-ledger && cargo check
```

---

## ZADANIE 9 — Fix: dead code warnings w net.rs

### Problem
`privai-node/src/net.rs` — dwa warningi:
- `WriterMsg::Shutdown` (linia ~419) — nigdy nie używany
- `decrypt_frame` (linia ~539) — nigdy nie używana

### Pliki do edycji
- `privai-node/src/net.rs`

### Co zrobić
- Dodaj `#[allow(dead_code)]` nad oboma albo:
  - Jeśli `Shutdown` jest planowane do użycia — zostaw z `#[allow(dead_code)]`
  - Jeśli `decrypt_frame` jest używane gdzie indziej — sprawdź. Jeśli nie — dodaj `#[allow(dead_code)]` z komentarzem `// Używane przy odczycie frame w connection read loop`

### Weryfikacja
```bash
cd /home/nxms-server/privAI/privai-node && cargo check 2>&1 | grep warning
```
Zero warningów.

---

## KOLEJNOŚĆ WYKONANIA

```
FAZA 1 (testy muszą przejść):
  Zadanie 2 (MarketplaceBatch auth skip) → Zadanie 1 (fix testy) → cargo test

FAZA 2 (security fixes):
  Zadanie 3 (signers ordering) → Zadanie 4 (falcon_verify z PK) → cargo test

FAZA 3 (cleanup):
  Zadanie 5 (chain_id error) → Zadanie 6 (block size) → Zadanie 7 (break) →
  Zadanie 8 (RocksDB bufory) → Zadanie 9 (warnings) → cargo test all
```

## FINALNA WERYFIKACJA

Po wszystkich zadaniach:
```bash
cd /home/nxms-server/privAI/privai-chain && cargo test
cd /home/nxms-server/privAI/privai-ledger && cargo test
cd /home/nxms-server/privAI/privai-node && cargo test
cd /home/nxms-server/privAI/privai-proof && cargo test
cd /home/nxms-server/privAI/privai-wallet && cargo test
```
Wszystkie PASS, zero warnings (oprócz ewentualnych `#[allow(dead_code)]`).
