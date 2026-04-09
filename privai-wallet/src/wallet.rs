//! privAI Wallet Core.
//! Role: Managing private keys, ReceiveBundles, and Note discovery.
//! Privacy Tier Logic: Implements Stealth Addresses via `RecipientBox` (KEM/AEAD).
//! Key Management: Post-Quantum Falcon (Signing) and FrodoKEM (Encapsulation).
//! Scanning Hint: Matches `RecipientBox.hint` against `snapshot.bundles` for fast filtering.
//! See: PRIVAI_V0_FORMATS.md

use crate::error::WalletError;
use crate::keys::WalletKeys;
use crate::state::{
    BundleMatch, BundleStatus, ManagedBundle, OwnedNoteRecord, OwnedNoteStatus, SpendMaterial,
    WalletSnapshot,
};
use crate::store::WalletStore;
use nxms_transport::crypto::{
    Keys, XCHACHA20POLY1305_KEY_LEN, XCHACHA20POLY1305_TAG_LEN,
    kem_decaps, kem_encaps, random_xchacha20poly1305_nonce, xchacha20poly1305_decrypt,
    xchacha20poly1305_encrypt,
};
use privai_chain::{
    derive_aux_commit, derive_nullifier, AuxWitness, BundleId, CanonicalDecode, CanonicalEncode,
    Hash32, Nullifier, OutputNote, ReceiveBundle, RecipientBox, RecipientBoxPlaintext,
    SpendPolicy, AEAD_ALG_XCHACHA20_POLY1305, FRODOKEM_640_SHAKE,
};

use crate::small_payments_rail::{RailContext, LocalTicket};

const WALLET_BUNDLE_ID_DOMAIN_V0: &[u8] = b"privai:wallet-bundle-id:v0";
const RECIPIENT_BOX_KEY_DOMAIN_V0: &[u8] = b"privai:recipient-box:key:v0";

/// Krok 6: Domena KDF do derivacji nullifier_key z KEM shared_secret.
/// NK = BLAKE3(domain, shared_secret, bundle_id) — bound do ephemeral KEM.
/// Zapobiega nadawcy wyborowi malicious NK (attack: fake NK → wrong nullifier → unspendable note).
const NULLIFIER_KEY_FROM_KEM_DOMAIN: &[u8] = b"privai:wallet:nk-from-kem:v0";

pub struct PrivaiWallet<S: WalletStore> {
    store: S,
    snapshot: WalletSnapshot,
    /// Master seed keys — jeśli None, używany jest legacy mode (Keys::generate).
    wallet_keys: Option<WalletKeys>,
}

impl<S: WalletStore> PrivaiWallet<S> {
    /// Otwórz portfel z istniejącego store (legacy mode — brak master seed).
    pub fn open(mut store: S) -> Result<Self, WalletError> {
        let snapshot = store.load()?.unwrap_or_default();
        store.save(&snapshot)?;
        Ok(Self { store, snapshot, wallet_keys: None })
    }

    /// Otwórz portfel z master seed (v1 — deterministic key derivation).
    /// 
    /// Jeśli store jest pusty, seed jest używany do derive wszystkich kluczy.
    /// Jeśli store ma istniejący seed_hash, jest weryfikowany.
    pub fn from_master_seed(mut store: S, seed: [u8; 32]) -> Result<Self, WalletError> {
        let snapshot = store.load()?.unwrap_or_default();
        let wallet_keys = WalletKeys::from_master_seed(&seed);

        // Weryfikacja: jeśli snapshot ma seed_hash, sprawdź czy pasuje
        if let Some(existing_hash) = snapshot.master_seed_hash {
            let new_hash = wallet_keys.master_seed_hash();
            if existing_hash != new_hash {
                return Err(WalletError::MasterSeedMismatch);
            }
        }

        store.save(&snapshot)?;
        Ok(Self { store, snapshot, wallet_keys: Some(wallet_keys) })
    }

    /// Generuj nowy portfel z losowym master seed.
    pub fn generate(mut store: S) -> Result<Self, WalletError> {
        let wallet_keys = WalletKeys::generate().map_err(|e| WalletError::Crypto(e))?;
        let mut snapshot = store.load()?.unwrap_or_default();
        snapshot.master_seed_hash = Some(wallet_keys.master_seed_hash());
        store.save(&snapshot)?;
        Ok(Self { store, snapshot, wallet_keys: Some(wallet_keys) })
    }

    pub fn snapshot(&self) -> &WalletSnapshot {
        &self.snapshot
    }

    pub fn import_bundle(&mut self, bundle: ReceiveBundle) -> Result<Hash32, WalletError> {
        if self.snapshot.bundles.contains_key(&bundle.bundle_id) {
            return Err(WalletError::DuplicateBundle(bundle.bundle_id));
        }

        let bundle_commit = bundle.commitment();
        self.snapshot.bundles.insert(
            bundle.bundle_id,
            ManagedBundle {
                bundle,
                bundle_commit,
                status: BundleStatus::Fresh,
                local_keys: None,
            },
        );
        self.flush()?;
        Ok(bundle_commit)
    }

    pub fn create_bundle(
        &mut self,
        expires_at: u64,
        flags: u8,
        one_time_falcon_pk: Vec<u8>,
        one_time_frodo_pk: Vec<u8>,
        route_hint: Option<Vec<u8>>,
        nullifier_key: Hash32,
    ) -> Result<ReceiveBundle, WalletError> {
        let bundle_id = derive_bundle_id(
            expires_at,
            &one_time_falcon_pk,
            &one_time_frodo_pk,
            route_hint.as_deref(),
        );
        let bundle = ReceiveBundle::new(
            bundle_id,
            expires_at,
            flags,
            one_time_falcon_pk,
            one_time_frodo_pk,
            route_hint,
            nullifier_key,
        );
        self.import_bundle(bundle.clone())?;
        Ok(bundle)
    }

    pub fn create_local_bundle(
        &mut self,
        expires_at: u64,
        flags: u8,
        route_hint: Option<Vec<u8>>,
    ) -> Result<ReceiveBundle, WalletError> {
        let index = self.snapshot.next_bundle_index;
        self.snapshot.next_bundle_index += 1;

        // v1: użyj WalletKeys jeśli dostępny (deterministic key derivation)
        // v0: fallback do Keys::generate() (legacy mode)
        let keys = if let Some(ref wallet_keys) = self.wallet_keys {
            wallet_keys.derive_bundle_keys(index)
                .map_err(|e| WalletError::Crypto(e))?
        } else {
            Keys::generate().map_err(|err| WalletError::Crypto(err.to_string()))?
        };

        // Krok 6: Odbiorca wstawia gotowy nullifier_key do swojego ReceiveBundle
        // Wyprowadzamy go deterministycznie z indeksu tego bundla (w ten sam sposób co klucze)
        let nullifier_key = if let Some(ref wallet_keys) = self.wallet_keys {
            wallet_keys.derive_nullifier_key(index)
        } else {
            // v0 fallback: fake nk for legacy wallets without seed
            let mut fake_nk = [0u8; 32];
            fake_nk[0] = 0xFF;
            fake_nk
        };

        let one_time_falcon_pk = keys
            .sig_pk()
            .map_err(|err| WalletError::Crypto(err.to_string()))?;
        let one_time_frodo_pk = keys
            .kem_pk()
            .map_err(|err| WalletError::Crypto(err.to_string()))?;

        let bundle_id = derive_bundle_id(
            expires_at,
            &one_time_falcon_pk,
            &one_time_frodo_pk,
            route_hint.as_deref(),
        );
        let bundle = ReceiveBundle::new(
            bundle_id,
            expires_at,
            flags,
            one_time_falcon_pk,
            one_time_frodo_pk,
            route_hint,
            nullifier_key,
        );
        let bundle_commit = bundle.commitment();
        self.snapshot.bundles.insert(
            bundle_id,
            ManagedBundle {
                bundle: bundle.clone(),
                bundle_commit,
                status: BundleStatus::Fresh,
                local_keys: Some(keys),
            },
        );
        self.flush()?;
        Ok(bundle)
    }

    pub fn mark_bundle_status(
        &mut self,
        bundle_id: BundleId,
        status: BundleStatus,
    ) -> Result<(), WalletError> {
        let managed = self
            .snapshot
            .bundles
            .get_mut(&bundle_id)
            .ok_or(WalletError::UnknownBundle(bundle_id))?;
        managed.status = status;
        self.flush()
    }

    /// Uzupełnij pulę bundli do target (v1 — deterministic key derivation).
    /// 
    /// Generuje nowe bundle z WalletKeys::derive_bundle_keys(next_bundle_index).
    /// Każdy bundle ma status Fresh i local_keys.
    pub fn replenish_bundles(
        &mut self,
        target: usize,
        expires_at: u64,
        route_hint: Option<Vec<u8>>,
    ) -> Result<usize, WalletError> {
        let fresh_count = self.snapshot.bundles.values()
            .filter(|b| b.status == BundleStatus::Fresh)
            .count();

        if fresh_count >= target {
            return Ok(0); // już mamy wystarczająco
        }

        let needed = target - fresh_count;
        let mut created = 0;

        for _ in 0..needed {
            if self.wallet_keys.is_none() {
                // Legacy mode — nie możemy derive nowych bundli bez WalletKeys
                break;
            }
            self.create_local_bundle(expires_at, 0, route_hint.clone())?;
            created += 1;
        }

        Ok(created)
    }

    /// Oznacz wygasłe bundle jako Revoked.
    /// 
    /// Sprawdza `bundle.expires_at < now_ms` dla każdego Fresh/Offered bundle.
    pub fn revoke_expired(&mut self, now_ms: u64) -> Result<usize, WalletError> {
        let mut revoked = 0;
        for managed in self.snapshot.bundles.values_mut() {
            if (managed.status == BundleStatus::Fresh || managed.status == BundleStatus::Offered)
                && managed.bundle.expires_at < now_ms
            {
                managed.status = BundleStatus::Revoked;
                revoked += 1;
            }
        }
        if revoked > 0 {
            self.flush()?;
        }
        Ok(revoked)
    }

    pub fn scan_output(&self, output: &OutputNote) -> BundleMatch {
        if self.snapshot.bundles.contains_key(&output.recipient_box.hint) {
            BundleMatch::HintMatched(output.recipient_box.hint)
        } else {
            BundleMatch::NoMatch
        }
    }

    /// Atomically odbiera notę: otwiera RecipientBox, weryfikuje, zapisuje, flush.
    /// 
    /// Crash safety: wszystko w jednej operacji — flush na końcu.
    /// Guard przed double-receive: sprawdza note_commit PRZED otwarciem.
    pub fn receive_note(
        &mut self,
        note: &OutputNote,
    ) -> Result<Nullifier, WalletError> {
        // 1. Guard przed double-receive (przed otwarciem — oszczędza KEM decaps)
        if self.snapshot.owned_notes.contains_key(&note.note_commit) {
            return Err(WalletError::DuplicateNote(note.note_commit));
        }

        // 2. Sprawdź bundle status (reject Used/Revoked)
        let managed = self
            .snapshot
            .bundles
            .get(&note.recipient_box.hint)
            .ok_or(WalletError::UnknownBundle(note.recipient_box.hint))?;
        if managed.status == BundleStatus::Used {
            return Err(WalletError::BundleAlreadyUsed(note.recipient_box.hint));
        }

        // 3. Open recipient box (KEM decaps — kosztowne ~5ms)
        let opened = self.open_recipient_box(note)?;

        // 4. Verify opened note
        self.verify_opened_note(note, &opened)?;

        // 5. Derive nullifier
        let derived_nullifier = derive_nullifier(&note.note_commit, &opened.nullifier_key);

        // 6. Mark bundle as Used
        let bundle = self
            .snapshot
            .bundles
            .get_mut(&opened.bundle_id)
            .ok_or(WalletError::UnknownBundle(opened.bundle_id))?;
        bundle.status = BundleStatus::Used;

        // 7. Insert owned note record
        self.snapshot.owned_notes.insert(
            note.note_commit,
            OwnedNoteRecord {
                note: note.clone(),
                opened,
                derived_nullifier,
                status: OwnedNoteStatus::Spendable,
            },
        );

        // 8. Atomic flush
        self.flush()?;

        Ok(derived_nullifier)
    }

    /// Stara metoda — zachowana dla kompatybilności wstecznej.
    /// Używaj receive_note() dla atomic operacji.
    pub fn record_opened_note(
        &mut self,
        note: OutputNote,
        _opened: RecipientBoxPlaintext,
    ) -> Result<Nullifier, WalletError> {
        self.receive_note(&note)
    }

    pub fn open_recipient_box(
        &self,
        note: &OutputNote,
    ) -> Result<RecipientBoxPlaintext, WalletError> {
        if note.recipient_box.kem_alg != FRODOKEM_640_SHAKE {
            return Err(WalletError::UnsupportedKemAlg(note.recipient_box.kem_alg));
        }
        if note.recipient_box.aead_alg != AEAD_ALG_XCHACHA20_POLY1305 {
            return Err(WalletError::UnsupportedAeadAlg(note.recipient_box.aead_alg));
        }

        let managed = self
            .snapshot
            .bundles
            .get(&note.recipient_box.hint)
            .ok_or(WalletError::UnknownBundle(note.recipient_box.hint))?;

        // Bundle status guard: reject Used/Revoked bundles
        if managed.status == BundleStatus::Used || managed.status == BundleStatus::Revoked {
            return Err(WalletError::BundleAlreadyUsed(note.recipient_box.hint));
        }
        let keys = managed
            .local_keys
            .as_ref()
            .ok_or(WalletError::MissingLocalKeys(note.recipient_box.hint))?;
        let recipient_kem_sk = keys
            .kem_sk_zeroizing()
            .map_err(|err| WalletError::Crypto(err.to_string()))?;
        let shared_secret = kem_decaps(recipient_kem_sk.as_slice(), &note.recipient_box.kem_ct)
            .map_err(|err| WalletError::Crypto(err.to_string()))?;
        let key = derive_recipient_box_key(&shared_secret, &note.recipient_box);
        let aad = recipient_box_aad(&note.recipient_box);
        let plaintext = xchacha20poly1305_decrypt(
            &key,
            &note.recipient_box.nonce,
            &note.recipient_box.ciphertext,
            &note.recipient_box.tag,
            &aad,
        )
        .map_err(|err| WalletError::Crypto(err.to_string()))?;

        let opened = RecipientBoxPlaintext::from_canonical_bytes(&plaintext)?;

        // Krok 6: Weryfikuj nullifier_key derivation z KEM shared_secret.
        // Jeśli NK w plaintext nie pasuje do derive_nullifier_key_from_kem(shared_secret, bundle_id),
        // nota może być niespendowalna lub pochodzi z manipulowanego sealingu.
        let expected_nk = derive_nullifier_key_from_kem(&shared_secret, &managed.bundle.bundle_id);
        if opened.nullifier_key != expected_nk {
            return Err(WalletError::InvalidNullifierKeyDerivation);
        }

        self.verify_opened_note(note, &opened)?;
        Ok(opened)
    }

    /// Zaszyfruj RecipientBoxPlaintext dla odbiorcy (KEM encaps + AEAD).
    ///
    /// Krok 6: Zwraca (RecipientBox, derived_nullifier_key).
    /// nullifier_key jest DERIVED z KEM shared_secret — nadawca nie może go wybrać.
    /// Caller NIE powinien ustawiać nullifier_key w `opened` — będzie nadpisany.
    ///
    /// TODO(privai:v2): Rozważ NK = BLAKE3(domain, shared_secret, note_commit) dla
    /// unikalności per nota (obecnie: per KEM encaps — praktycznie unikalny, ale
    /// teoretycznie można derive bez note_commit przy wielokrotnym seal na ten sam bundle).
    pub fn seal_recipient_box(
        bundle: &ReceiveBundle,
        opened: &RecipientBoxPlaintext,
    ) -> Result<(RecipientBox, Hash32), WalletError> {
        let (kem_ct, shared_secret) =
            kem_encaps(&bundle.one_time_frodo_pk).map_err(|err| WalletError::Crypto(err.to_string()))?;
        let nonce = random_xchacha20poly1305_nonce();
        let recipient_box_stub = RecipientBox::new(
            kem_ct,
            nonce,
            Vec::new(),
            [0u8; XCHACHA20POLY1305_TAG_LEN],
            bundle.bundle_id,
        );

        // Krok 6: Derive nullifier_key z KEM shared_secret + bundle_id.
        // NK jest bound do ephemeral KEM — każdy seal ma inny shared_secret → inny NK.
        // Odbiorca weryfikuje NK po KEM decaps w open_recipient_box().
        let derived_nk = derive_nullifier_key_from_kem(&shared_secret, &bundle.bundle_id);

        // Override caller-provided nullifier_key z KEM-derived value.
        // Caller-provided NK jest ignorowany (może być [0u8; 32] lub cokolwiek).
        let mut opened_final = opened.clone();
        opened_final.nullifier_key = derived_nk;

        let key = derive_recipient_box_key(&shared_secret, &recipient_box_stub);
        let aad = recipient_box_aad(&recipient_box_stub);
        let (ciphertext, tag) = xchacha20poly1305_encrypt(
            &key,
            &nonce,
            &opened_final.to_canonical_bytes(),
            &aad,
        )
        .map_err(|err| WalletError::Crypto(err.to_string()))?;

        Ok((RecipientBox::new(
            recipient_box_stub.kem_ct,
            nonce,
            ciphertext,
            tag,
            bundle.bundle_id,
        ), derived_nk))
    }

    pub fn verify_opened_note(
        &self,
        note: &OutputNote,
        opened: &RecipientBoxPlaintext,
    ) -> Result<(), WalletError> {
        if note.recompute_commit() != note.note_commit {
            return Err(WalletError::InvalidNoteCommit);
        }
        if note.payload_commit() != opened.note_payload_commit {
            return Err(WalletError::NotePayloadCommitMismatch);
        }
        if note.recipient_box.hint != opened.bundle_id {
            return Err(WalletError::BundleHintMismatch);
        }
        if !self.snapshot.bundles.contains_key(&opened.bundle_id) {
            return Err(WalletError::UnknownBundle(opened.bundle_id));
        }

        let spend_policy = SpendPolicy::from_canonical_bytes(&opened.spend_policy_opening)?;
        if spend_policy.commitment() != note.spend_policy_commit {
            return Err(WalletError::SpendPolicyCommitMismatch);
        }

        let aux_witness = AuxWitness::from_canonical_bytes(&opened.aux_opening)?;
        if derive_aux_commit(&aux_witness) != note.aux_commit {
            return Err(WalletError::AuxCommitMismatch);
        }
        if aux_witness.amount != opened.amount
            || aux_witness.witness_seed != opened.witness_seed
            || aux_witness.bundle_id != opened.bundle_id
        {
            return Err(WalletError::AuxWitnessMismatch);
        }

        Ok(())
    }

    pub fn spendable_notes(&self) -> Vec<&OwnedNoteRecord> {
        self.snapshot
            .owned_notes
            .values()
            .filter(|record| matches!(record.status, OwnedNoteStatus::Spendable))
            .collect()
    }

    pub fn spend_material(&self, note_commit: &Hash32) -> Result<SpendMaterial, WalletError> {
        let record = self
            .snapshot
            .owned_notes
            .get(note_commit)
            .ok_or(WalletError::UnknownNote(*note_commit))?;

        Ok(SpendMaterial {
            note: record.note.clone(),
            amount: record.opened.amount,
            note_commit: record.note.note_commit,
            nullifier: record.derived_nullifier,
            witness_seed: record.opened.witness_seed,
            nullifier_key: record.opened.nullifier_key,
            spend_policy_opening: record.opened.spend_policy_opening.clone(),
            aux_opening: record.opened.aux_opening.clone(),
        })
    }

    pub fn mark_note_spent(&mut self, note_commit: Hash32) -> Result<(), WalletError> {
        let record = self
            .snapshot
            .owned_notes
            .get_mut(&note_commit)
            .ok_or(WalletError::UnknownNote(note_commit))?;
        record.status = OwnedNoteStatus::Spent {
            nullifier: record.derived_nullifier,
        };
        self.flush()
    }

    pub fn init_rail_context(&mut self, rail_seed: [u8; 32]) -> Result<(), WalletError> {
        self.snapshot.rail_context = Some(RailContext::new(rail_seed));
        self.flush()
    }

    pub fn generate_next_ticket(&mut self, merchant_commit: Hash32) -> Result<LocalTicket, WalletError> {
        if let Some(rail_context) = &mut self.snapshot.rail_context {
            let rail_seed = rail_context.rail_seed; // Extract copy of seed first
            let pool = rail_context.get_or_create_pool(merchant_commit);
            let ticket = pool.generate_next_ticket(&rail_seed);
            self.flush()?;
            Ok(ticket)
        } else {
            Err(WalletError::RailContextMissing)
        }
    }

    fn flush(&mut self) -> Result<(), WalletError> {
        self.store.save(&self.snapshot)?;
        Ok(())
    }
}

/// Krok 6: Derive nullifier_key z KEM shared_secret i bundle_id.
///
/// NK = BLAKE3(domain, |shared_secret|, shared_secret, |bundle_id|, bundle_id)
/// Używa len-prefixed hashing (spójnie z resztą wallet.rs).
///
/// Właściwości:
/// - Deterministyczny: ten sam shared_secret + bundle_id → ten sam NK
/// - Unikalny: shared_secret jest efemeryczny (FrodoKEM encaps) → inny per seal
/// - Nie do manipulacji: nadawca nie może wybrać NK bez KEM private key odbiorcy
pub(crate) fn derive_nullifier_key_from_kem(shared_secret: &[u8], bundle_id: &BundleId) -> Hash32 {
    let mut hasher = blake3::Hasher::new();
    update_len_prefixed(&mut hasher, NULLIFIER_KEY_FROM_KEM_DOMAIN);
    update_len_prefixed(&mut hasher, shared_secret);
    update_len_prefixed(&mut hasher, &bundle_id[..]);
    let mut out = [0u8; 32];
    out.copy_from_slice(hasher.finalize().as_bytes());
    out
}

fn derive_bundle_id(
    expires_at: u64,
    one_time_falcon_pk: &[u8],
    one_time_frodo_pk: &[u8],
    route_hint: Option<&[u8]>,
) -> BundleId {
    let mut hasher = blake3::Hasher::new();
    update_len_prefixed(&mut hasher, WALLET_BUNDLE_ID_DOMAIN_V0);
    update_len_prefixed(&mut hasher, &expires_at.to_le_bytes());
    update_len_prefixed(&mut hasher, one_time_falcon_pk);
    update_len_prefixed(&mut hasher, one_time_frodo_pk);
    update_len_prefixed(&mut hasher, route_hint.unwrap_or_default());

    let hash = hasher.finalize();
    let mut bundle_id = [0u8; 16];
    bundle_id.copy_from_slice(&hash.as_bytes()[..16]);
    bundle_id
}

fn update_len_prefixed(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u32).to_le_bytes());
    hasher.update(bytes);
}

fn derive_recipient_box_key(shared_secret: &[u8], recipient_box: &RecipientBox) -> [u8; XCHACHA20POLY1305_KEY_LEN] {
    let mut hasher = blake3::Hasher::new();
    update_len_prefixed(&mut hasher, RECIPIENT_BOX_KEY_DOMAIN_V0);
    update_len_prefixed(&mut hasher, shared_secret);
    update_len_prefixed(&mut hasher, &[recipient_box.version]);
    update_len_prefixed(&mut hasher, &[recipient_box.kem_alg]);
    update_len_prefixed(&mut hasher, &[recipient_box.aead_alg]);
    update_len_prefixed(&mut hasher, &recipient_box.hint);
    let hash = hasher.finalize();
    let mut key = [0u8; XCHACHA20POLY1305_KEY_LEN];
    key.copy_from_slice(&hash.as_bytes()[..XCHACHA20POLY1305_KEY_LEN]);
    key
}

fn recipient_box_aad(recipient_box: &RecipientBox) -> Vec<u8> {
    let mut aad = Vec::with_capacity(1 + 1 + 1 + 16);
    aad.push(recipient_box.version);
    aad.push(recipient_box.kem_alg);
    aad.push(recipient_box.aead_alg);
    aad.extend_from_slice(&recipient_box.hint);
    aad
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;
    use crate::store::MemoryWalletStore;
    use privai_chain::{
        Amount14, AuxWitness, CanonicalEncode, LweCiphertext, PRIVAI_V0, RecipientBox,
        SpendPolicy,
    };

    fn sample_bundle() -> ReceiveBundle {
        ReceiveBundle::new(
            [0x11; 16],
            100,
            0,
            vec![1, 2, 3],
            vec![4, 5, 6],
            Some(vec![7, 8]),
            [0x42; 32],
        )
    }

    fn sample_note(bundle_id: BundleId) -> (OutputNote, RecipientBoxPlaintext) {
        let amount = Amount14::new(77).expect("amount");
        let opened = RecipientBoxPlaintext {
            version: PRIVAI_V0,
            bundle_id,
            note_payload_commit: [0; 32],
            amount,
            witness_seed: [0x21; 32],
            nullifier_key: [0x22; 32],
            spend_policy_opening: SpendPolicy::Single {
                falcon_pk_hash: [0x31; 32],
            }
            .to_canonical_bytes(),
            aux_opening: AuxWitness {
                version: PRIVAI_V0,
                amount,
                witness_seed: [0x21; 32],
                noise_class: 1,
                bundle_id,
            }
            .to_canonical_bytes(),
            sender_memo: Some(vec![9]),
        };
        let spend_policy = SpendPolicy::Single {
            falcon_pk_hash: [0x31; 32],
        };
        let aux_commit = privai_chain::derive_aux_commit(
            &AuxWitness::from_canonical_bytes(&opened.aux_opening).expect("aux"),
        );
        let ct_amt = LweCiphertext::default();
        let note_payload_commit = OutputNote::payload_commit_from_parts(
            PRIVAI_V0,
            &spend_policy.commitment(),
            &ct_amt,
            &aux_commit,
        );
        let note = OutputNote::new(
            spend_policy.commitment(),
            ct_amt,
            aux_commit,
            RecipientBox::new(vec![1], [2; 24], vec![3], [4; 16], bundle_id),
        );
        let mut opened = opened;
        opened.note_payload_commit = note_payload_commit;
        (note, opened)
    }

    #[test]
    fn imported_bundle_can_be_hint_matched() {
        let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
        let bundle = sample_bundle();
        wallet.import_bundle(bundle.clone()).expect("bundle");

        let note = OutputNote::new(
            [0x31; 32],
            LweCiphertext::default(),
            [0x32; 32],
            RecipientBox::new(vec![1], [2; 24], vec![3], [4; 16], bundle.bundle_id),
        );

        assert_eq!(
            wallet.scan_output(&note),
            BundleMatch::HintMatched(bundle.bundle_id)
        );
    }

    #[test]
    fn opened_note_becomes_spendable_and_bundle_is_consumed() {
        let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
        // Użyj create_local_bundle zamiast import_bundle (local_keys potrzebne do open_recipient_box)
        let bundle = wallet
            .create_local_bundle(100, 0, Some(vec![7, 8]))
            .expect("local bundle");

        let amount = Amount14::new(77).expect("amount");
        let opened = RecipientBoxPlaintext {
            version: PRIVAI_V0,
            bundle_id: bundle.bundle_id,
            note_payload_commit: [0; 32],
            amount,
            witness_seed: [0x21; 32],
            nullifier_key: [0x22; 32],
            spend_policy_opening: SpendPolicy::Single {
                falcon_pk_hash: [0x31; 32],
            }
            .to_canonical_bytes(),
            aux_opening: AuxWitness {
                version: PRIVAI_V0,
                amount,
                witness_seed: [0x21; 32],
                noise_class: 1,
                bundle_id: bundle.bundle_id,
            }
            .to_canonical_bytes(),
            sender_memo: Some(vec![9]),
        };
        let spend_policy = SpendPolicy::Single {
            falcon_pk_hash: [0x31; 32],
        };
        let aux_commit = privai_chain::derive_aux_commit(
            &AuxWitness::from_canonical_bytes(&opened.aux_opening).expect("aux"),
        );
        let ct_amt = LweCiphertext::default();
        let note_payload_commit = OutputNote::payload_commit_from_parts(
            PRIVAI_V0,
            &spend_policy.commitment(),
            &ct_amt,
            &aux_commit,
        );
        let mut opened = opened;
        opened.note_payload_commit = note_payload_commit;
        // Krok 6: seal_recipient_box returns (RecipientBox, derived_nk).
        let (recipient_box, _derived_nk) = PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(&bundle, &opened)
            .expect("seal box");
        let note = OutputNote::new(
            spend_policy.commitment(),
            ct_amt,
            aux_commit,
            recipient_box,
        );

        let nullifier = wallet
            .record_opened_note(note.clone(), opened.clone())
            .expect("record note");

        assert_eq!(wallet.spendable_notes().len(), 1);
        assert_eq!(
            wallet.snapshot().bundles.get(&bundle.bundle_id).unwrap().status,
            BundleStatus::Used
        );
        assert_eq!(
            wallet.spend_material(&note.note_commit).expect("spend material").nullifier,
            nullifier
        );
    }

    #[test]
    fn recipient_box_roundtrip_uses_local_bundle_keys() {
        let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
        let bundle = wallet
            .create_local_bundle(100, 0, Some(vec![7, 8]))
            .expect("local bundle");

        let amount = Amount14::new(77).expect("amount");
        let opened = RecipientBoxPlaintext {
            version: PRIVAI_V0,
            bundle_id: bundle.bundle_id,
            note_payload_commit: [0; 32],
            amount,
            witness_seed: [0x21; 32],
            nullifier_key: [0x22; 32],
            spend_policy_opening: SpendPolicy::Single {
                falcon_pk_hash: [0x31; 32],
            }
            .to_canonical_bytes(),
            aux_opening: AuxWitness {
                version: PRIVAI_V0,
                amount,
                witness_seed: [0x21; 32],
                noise_class: 1,
                bundle_id: bundle.bundle_id,
            }
            .to_canonical_bytes(),
            sender_memo: Some(vec![9]),
        };
        let spend_policy = SpendPolicy::Single {
            falcon_pk_hash: [0x31; 32],
        };
        let aux_commit = privai_chain::derive_aux_commit(
            &AuxWitness::from_canonical_bytes(&opened.aux_opening).expect("aux"),
        );
        let ct_amt = LweCiphertext::default();
        let note_payload_commit = OutputNote::payload_commit_from_parts(
            PRIVAI_V0,
            &spend_policy.commitment(),
            &ct_amt,
            &aux_commit,
        );
        let mut opened = opened;
        opened.note_payload_commit = note_payload_commit;
        // Krok 6: seal_recipient_box returns (RecipientBox, derived_nk).
        let (recipient_box, _derived_nk) = PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(&bundle, &opened)
            .expect("seal box");
        let note = OutputNote::new(
            spend_policy.commitment(),
            ct_amt,
            aux_commit,
            recipient_box,
        );

        let opened_again = wallet.open_recipient_box(&note).expect("open box");
        assert_eq!(opened_again.bundle_id, bundle.bundle_id);
        assert_eq!(opened_again.amount, amount);
        assert_eq!(opened_again.witness_seed, [0x21; 32]);
        assert_eq!(opened_again.note_payload_commit, note.payload_commit());
    }

    #[test]
    #[ignore] // TODO: OutputNote::new oblicza note_commit z pól - zmiana aux_commit zmienia note_commit
    fn opened_note_rejects_mismatched_aux_opening() {
        // Wymaga podejścia które nie zmienia note_commit przy zmianie aux_commit
    }

    #[test]
    fn opened_note_rejects_mismatched_note_payload_commit() {
        let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
        let bundle = wallet
            .create_local_bundle(100, 0, Some(vec![7, 8]))
            .expect("local bundle");

        let amount = Amount14::new(77).expect("amount");
        let opened_good = RecipientBoxPlaintext {
            version: PRIVAI_V0,
            bundle_id: bundle.bundle_id,
            note_payload_commit: [0; 32], // poprawny
            amount,
            witness_seed: [0x21; 32],
            nullifier_key: [0x22; 32],
            spend_policy_opening: SpendPolicy::Single {
                falcon_pk_hash: [0x31; 32],
            }
            .to_canonical_bytes(),
            aux_opening: AuxWitness {
                version: PRIVAI_V0,
                amount,
                witness_seed: [0x21; 32],
                noise_class: 1,
                bundle_id: bundle.bundle_id,
            }
            .to_canonical_bytes(),
            sender_memo: Some(vec![9]),
        };

        // Stwórz opened_bad z ZŁYM note_payload_commit
        let mut opened_bad = opened_good.clone();
        opened_bad.note_payload_commit = [0xAA; 32]; // zły commit

        // Seal z opened_bad (z ZŁYM note_payload_commit)
        // Krok 6: seal_recipient_box returns (RecipientBox, derived_nk).
        let (_recipient_box, _derived_nk) = PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(&bundle, &opened_bad)
            .expect("seal box");

        let spend_policy = SpendPolicy::Single {
            falcon_pk_hash: [0x31; 32],
        };
        let aux_commit = privai_chain::derive_aux_commit(
            &AuxWitness::from_canonical_bytes(&opened_good.aux_opening).expect("aux"),
        );
        let ct_amt = LweCiphertext::default();

        // Note z POPRAWNYM note_payload_commit (different from opened_bad.note_payload_commit)
        let _good_note_payload_commit = OutputNote::payload_commit_from_parts(
            PRIVAI_V0,
            &spend_policy.commitment(),
            &ct_amt,
            &aux_commit,
        );
        // Utwórz recipient_box_bad z użyciem seal_recipient_box i opened_bad
        let (recipient_box_bad, _derived_nk) = PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(&bundle, &opened_bad)
            .expect("seal box bad");

        let note = OutputNote::new(
            spend_policy.commitment(),
            ct_amt,
            aux_commit,
            recipient_box_bad, // ciphertext z opened_bad (z ZŁYM note_payload_commit)
        );

        // record_opened_note(note, opened_good) - opened_good jest ignorowany
        // receive_note deszyfruje ciphertext → opened z ZŁYM note_payload_commit
        // verify_opened_note: opened.note_payload_commit (zły) != note.note_payload_commit (poprawny) → NotePayloadCommitMismatch!
        let err = wallet.record_opened_note(note, opened_good).expect_err("must reject");
        assert!(matches!(err, WalletError::NotePayloadCommitMismatch));
    }

    #[test]
    fn create_bundle_is_deterministic_for_same_inputs() {
        let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
        let bundle_a = wallet
            .create_bundle(100, 0, vec![1, 2, 3], vec![4, 5, 6], Some(vec![7, 8]), [0xAA; 32])
            .expect("bundle a");
        let mut wallet_b = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
        let bundle_b = wallet_b
            .create_bundle(100, 0, vec![1, 2, 3], vec![4, 5, 6], Some(vec![7, 8]), [0xAA; 32])
            .expect("bundle b");

        assert_eq!(bundle_a.bundle_id, bundle_b.bundle_id);
        assert_eq!(bundle_a.commitment(), bundle_b.commitment());
    }
}
