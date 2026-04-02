use std::time::{SystemTime, UNIX_EPOCH};

use privai_chain::{
    derive_aux_commit, Amount14, AuxWitness, CanonicalDecode, CanonicalEncode, LweCiphertext,
    OutputNote, RecipientBoxPlaintext, ReceiveBundle, SpendPolicy, PRIVAI_V0,
};
use privai_wallet::{
    BundleStatus, FileSystemWalletStore, MemoryWalletStore, PrivaiWallet, WalletError,
};

fn sample_opened_plaintext(bundle: &ReceiveBundle) -> (RecipientBoxPlaintext, SpendPolicy, LweCiphertext) {
    let amount = Amount14::new(77).expect("amount");
    let spend_policy = SpendPolicy::Single {
        falcon_pk_hash: [0x31; 32],
    };
    let ct_amt = LweCiphertext::default();
    let aux_opening = AuxWitness {
        version: PRIVAI_V0,
        amount,
        witness_seed: [0x21; 32],
        noise_class: 1,
        bundle_id: bundle.bundle_id,
    };
    let note_payload_commit = OutputNote::payload_commit_from_parts(
        PRIVAI_V0,
        &spend_policy.commitment(),
        &ct_amt,
        &derive_aux_commit(&aux_opening),
    );
    let opened = RecipientBoxPlaintext {
        version: PRIVAI_V0,
        bundle_id: bundle.bundle_id,
        note_payload_commit,
        amount,
        witness_seed: [0x21; 32],
        nullifier_key: [0x22; 32],
        spend_policy_opening: spend_policy.to_canonical_bytes(),
        aux_opening: aux_opening.to_canonical_bytes(),
        sender_memo: Some(b"market-session-1".to_vec()),
    };

    (opened, spend_policy, ct_amt)
}

fn build_note_for_bundle(bundle: &ReceiveBundle) -> OutputNote {
    let (opened, spend_policy, ct_amt) = sample_opened_plaintext(bundle);
    let aux_witness =
        AuxWitness::from_canonical_bytes(&opened.aux_opening).expect("aux decode");
    let aux_commit = derive_aux_commit(&aux_witness);
    let (recipient_box, _derived_nk) = PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(bundle, &opened)
        .expect("seal recipient box");

    OutputNote::new(spend_policy.commitment(), ct_amt, aux_commit, recipient_box)
}

fn temp_wallet_root(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "privai-wallet-integration-{}-{}-{}",
        label,
        std::process::id(),
        unique
    ))
}

#[test]
fn tampered_recipient_box_ciphertext_is_rejected() {
    let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
    let bundle = wallet
        .create_local_bundle(100, 0, Some(vec![7, 8]))
        .expect("local bundle");
    let mut note = build_note_for_bundle(&bundle);
    note.recipient_box.ciphertext[0] ^= 0x01;

    let err = wallet
        .open_recipient_box(&note)
        .expect_err("tampered ciphertext must fail");
    assert!(matches!(err, WalletError::Crypto(_)));
}

#[test]
fn persisted_wallet_can_reopen_local_bundle_and_track_note() {
    let root = temp_wallet_root("persist");
    let note = {
        let mut wallet = PrivaiWallet::open(FileSystemWalletStore::new(&root)).expect("wallet");
        let bundle = wallet
            .create_local_bundle(100, 0, Some(vec![7, 8]))
            .expect("local bundle");
        build_note_for_bundle(&bundle)
    };

    let mut wallet = PrivaiWallet::open(FileSystemWalletStore::new(&root)).expect("wallet");
    let opened = wallet.open_recipient_box(&note).expect("open recipient box");
    let nullifier = wallet
        .record_opened_note(note.clone(), opened)
        .expect("record opened note");

    let wallet = PrivaiWallet::open(FileSystemWalletStore::new(&root)).expect("wallet");
    assert_eq!(wallet.spendable_notes().len(), 1);
    assert_eq!(
        wallet.snapshot().bundles.get(&note.recipient_box.hint).unwrap().status,
        BundleStatus::Used
    );
    assert_eq!(
        wallet
            .spend_material(&note.note_commit)
            .expect("spend material")
            .nullifier,
        nullifier
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn imported_public_bundle_without_local_keys_cannot_open_note() {
    let mut sender_wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
    let bundle = sender_wallet
        .create_local_bundle(100, 0, Some(vec![7, 8]))
        .expect("local bundle");
    let note = build_note_for_bundle(&bundle);

    let mut receiver_wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
    receiver_wallet
        .import_bundle(bundle.clone())
        .expect("import public bundle");

    let err = receiver_wallet
        .open_recipient_box(&note)
        .expect_err("wallet without secret keys must fail");
    assert!(matches!(err, WalletError::MissingLocalKeys(id) if id == bundle.bundle_id));
}
