use crate::error::WalletError;
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

pub struct PrivaiWallet<S: WalletStore> {
    store: S,
    snapshot: WalletSnapshot,
}

impl<S: WalletStore> PrivaiWallet<S> {
    pub fn open(mut store: S) -> Result<Self, WalletError> {
        let snapshot = store.load()?.unwrap_or_default();
        store.save(&snapshot)?;
        Ok(Self { store, snapshot })
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
        let keys = Keys::generate().map_err(|err| WalletError::Crypto(err.to_string()))?;
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

    pub fn scan_output(&self, output: &OutputNote) -> BundleMatch {
        if self.snapshot.bundles.contains_key(&output.recipient_box.hint) {
            BundleMatch::HintMatched(output.recipient_box.hint)
        } else {
            BundleMatch::NoMatch
        }
    }

    pub fn record_opened_note(
        &mut self,
        note: OutputNote,
        opened: RecipientBoxPlaintext,
    ) -> Result<Nullifier, WalletError> {
        self.verify_opened_note(&note, &opened)?;
        if self.snapshot.owned_notes.contains_key(&note.note_commit) {
            return Err(WalletError::DuplicateNote(note.note_commit));
        }
        let bundle = self
            .snapshot
            .bundles
            .get_mut(&opened.bundle_id)
            .ok_or(WalletError::UnknownBundle(opened.bundle_id))?;

        let derived_nullifier = derive_nullifier(&note.note_commit, &opened.nullifier_key);
        bundle.status = BundleStatus::Used;
        self.snapshot.owned_notes.insert(
            note.note_commit,
            OwnedNoteRecord {
                note,
                opened,
                derived_nullifier,
                status: OwnedNoteStatus::Spendable,
            },
        );
        self.flush()?;
        Ok(derived_nullifier)
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
        self.verify_opened_note(note, &opened)?;
        Ok(opened)
    }

    pub fn seal_recipient_box(
        bundle: &ReceiveBundle,
        opened: &RecipientBoxPlaintext,
    ) -> Result<RecipientBox, WalletError> {
        let (kem_ct, shared_secret) =
            kem_encaps(&bundle.one_time_frodo_pk).map_err(|err| WalletError::Crypto(err.to_string()))?;
        let nonce = random_xchacha20poly1305_nonce();
        let recipient_box = RecipientBox::new(
            kem_ct,
            nonce,
            Vec::new(),
            [0u8; XCHACHA20POLY1305_TAG_LEN],
            bundle.bundle_id,
        );
        let key = derive_recipient_box_key(&shared_secret, &recipient_box);
        let aad = recipient_box_aad(&recipient_box);
        let (ciphertext, tag) = xchacha20poly1305_encrypt(
            &key,
            &nonce,
            &opened.to_canonical_bytes(),
            &aad,
        )
        .map_err(|err| WalletError::Crypto(err.to_string()))?;

        Ok(RecipientBox::new(
            recipient_box.kem_ct,
            nonce,
            ciphertext,
            tag,
            bundle.bundle_id,
        ))
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
        let bundle = sample_bundle();
        wallet.import_bundle(bundle.clone()).expect("bundle");
        let (note, opened) = sample_note(bundle.bundle_id);

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
        let recipient_box = PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(&bundle, &opened)
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
    fn opened_note_rejects_mismatched_aux_opening() {
        let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
        let bundle = sample_bundle();
        wallet.import_bundle(bundle.clone()).expect("bundle");
        let (note, mut opened) = sample_note(bundle.bundle_id);
        opened.aux_opening = AuxWitness {
            version: PRIVAI_V0,
            amount: Amount14::new(78).expect("amount"),
            witness_seed: opened.witness_seed,
            noise_class: 1,
            bundle_id: opened.bundle_id,
        }
        .to_canonical_bytes();

        let err = wallet.record_opened_note(note, opened).expect_err("must reject");
        assert!(matches!(err, WalletError::AuxCommitMismatch));
    }

    #[test]
    fn opened_note_rejects_mismatched_note_payload_commit() {
        let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
        let bundle = sample_bundle();
        wallet.import_bundle(bundle.clone()).expect("bundle");
        let (note, mut opened) = sample_note(bundle.bundle_id);
        opened.note_payload_commit = [0xAA; 32];

        let err = wallet.record_opened_note(note, opened).expect_err("must reject");
        assert!(matches!(err, WalletError::NotePayloadCommitMismatch));
    }

    #[test]
    fn create_bundle_is_deterministic_for_same_inputs() {
        let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
        let bundle_a = wallet
            .create_bundle(100, 0, vec![1, 2, 3], vec![4, 5, 6], Some(vec![7, 8]))
            .expect("bundle a");
        let mut wallet_b = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
        let bundle_b = wallet_b
            .create_bundle(100, 0, vec![1, 2, 3], vec![4, 5, 6], Some(vec![7, 8]))
            .expect("bundle b");

        assert_eq!(bundle_a.bundle_id, bundle_b.bundle_id);
        assert_eq!(bundle_a.commitment(), bundle_b.commitment());
    }
}
