use privai_chain::escrow::EscrowAction;
use privai_chain::hash::falcon_pk_hash;
use privai_chain::{
    CanonicalEncode, InputAuth, InputRef, LweCiphertext, Nullifier, OutputNote, RecipientBox,
    SpendPolicy, SpendPolicyTag, Transaction, TransferNoteTx, TxCore, TX_TYPE_TRANSFER_NOTE,
};
use privai_ledger::error::ValidationError;
use privai_ledger::state::{LedgerSnapshot, NoteRecord, NoteStatus};

fn make_escrow_policy(
    buyer_pk: &[u8],
    merchant_pk: &[u8],
    operator_pk: &[u8],
    timeout: u64,
) -> SpendPolicy {
    SpendPolicy::Escrow2of3 {
        buyer_pk_hash: falcon_pk_hash(buyer_pk),
        merchant_pk_hash: falcon_pk_hash(merchant_pk),
        operator_pk_hash: falcon_pk_hash(operator_pk),
        timeout_block: timeout,
    }
}

fn make_output_with_policy(policy_commit: privai_chain::Hash32) -> OutputNote {
    OutputNote::new(
        policy_commit,
        LweCiphertext::default(),
        [0x99; 32],
        RecipientBox::new(vec![1], [2; 24], vec![3], [4; 16], [5; 16]),
    )
}

fn buyer_output(buyer_pk: &[u8]) -> OutputNote {
    let commit = SpendPolicy::Single {
        falcon_pk_hash: falcon_pk_hash(buyer_pk),
    }
    .commitment();
    make_output_with_policy(commit)
}

fn setup_snapshot_with_escrow_note(policy: &SpendPolicy) -> (LedgerSnapshot, privai_chain::Hash32) {
    let mut snapshot = LedgerSnapshot::genesis(17);
    let note = OutputNote::new(
        policy.commitment(),
        LweCiphertext::default(),
        [0x10; 32],
        RecipientBox::new(vec![0x10], [0x10; 24], vec![0x11], [0x10; 16], [0x10; 16]),
    );
    let note_commit = note.note_commit;
    snapshot.notes.insert(
        note_commit,
        NoteRecord {
            note,
            created_in_block: Some(0),
            status: NoteStatus::Unspent,
        },
    );
    (snapshot, note_commit)
}

// ──────────────────────────────────────────────────────────────────────────────
// TEST 1: RecoveryRelease before timeout is rejected at ledger validation
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn recovery_release_before_timeout_rejected() {
    let (buyer_sk, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen buyer");
    let (merchant_sk, merchant_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen merchant");
    let (_operator_sk, operator_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen operator");

    // timeout_block = 100 — recovery is only valid at height >= 100
    let policy = make_escrow_policy(&buyer_pk, &merchant_pk, &operator_pk, 100);
    let (snapshot, note_commit) = setup_snapshot_with_escrow_note(&policy);

    let output = buyer_output(&buyer_pk);
    let tx = Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: 1,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef { note_commit }],
            input_nullifiers: vec![Nullifier([0xE1; 32])],
            outputs: vec![output],
            fee: 0,
            statement_commit: [0xF0; 32],
            auth: vec![InputAuth {
                policy_tag: SpendPolicyTag::Escrow2of3 as u8,
                signer_pks: vec![buyer_pk.clone(), merchant_pk.clone()],
                signatures: vec![vec![], vec![]],
                policy_opening: Some(policy.to_canonical_bytes()),
                escrow_action: Some(EscrowAction::RecoveryRelease as u8),
            }],
        },
    });

    // Sign with real keys so sig verification passes (tests the timeout gate specifically)
    let tx_hash = tx.tx_signing_hash();
    let buyer_sig = nxms_transport::crypto::falcon_sign_ct_prepared(&buyer_sk, &tx_hash).unwrap();
    let merchant_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(&merchant_sk, &tx_hash).unwrap();

    let tx = match tx {
        Transaction::TransferNote(mut t) => {
            t.core.auth[0].signatures = vec![buyer_sig, merchant_sig];
            Transaction::TransferNote(t)
        }
        _ => unreachable!(),
    };

    // Attempt validation at block height 1, which is BEFORE timeout_block (100)
    let result = privai_ledger::ledger::validate_transaction(&tx, &snapshot, 0, 1);
    assert!(
        result.is_err(),
        "RecoveryRelease before timeout must be rejected"
    );
    match result.unwrap_err() {
        ValidationError::EscrowRecoveryBeforeTimeout { current, required } => {
            assert_eq!(current, 1, "current block height should be 1");
            assert_eq!(required, 100, "required timeout should be 100");
        }
        other => panic!("Expected EscrowRecoveryBeforeTimeout, got: {:?}", other),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// TEST 2: RecoveryRelease with wrong signer pair is rejected
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn recovery_release_with_wrong_signer_pair_rejected() {
    use privai_chain::derive_aux_commit;
    use privai_chain::Amount14;
    use privai_wallet::escrow_builder::{AuthMaterial, FinalAssemblyInputs};
    use privai_wallet::{MemoryWalletStore, PrivaiWallet};

    let (_buyer_sk, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen buyer");
    let (_merchant_sk, merchant_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen merchant");
    let (_operator_sk, operator_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen operator");

    let buyer_h = falcon_pk_hash(&buyer_pk);

    let policy = make_escrow_policy(&buyer_pk, &merchant_pk, &operator_pk, 100);

    // Setup wallet with a spendable note
    let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).unwrap();
    let input_bundle = wallet
        .create_local_bundle(1000, 0, Some(b"escrow funding".to_vec()))
        .unwrap();

    let amount = Amount14::new(1000).unwrap();
    let aux_witness = privai_chain::AuxWitness {
        version: privai_chain::PRIVAI_V0,
        amount,
        witness_seed: [0x22; 32],
        noise_class: 1,
        bundle_id: input_bundle.bundle_id,
    };
    let aux_commit = derive_aux_commit(&aux_witness);
    let note_payload_commit = OutputNote::payload_commit_from_parts(
        privai_chain::PRIVAI_V0,
        &policy.commitment(),
        &LweCiphertext::default(),
        &aux_commit,
    );

    let opened = privai_chain::RecipientBoxPlaintext {
        version: privai_chain::PRIVAI_V0,
        bundle_id: input_bundle.bundle_id,
        note_payload_commit,
        amount,
        witness_seed: [0x22; 32],
        nullifier_key: [0x33; 32],
        spend_policy_opening: policy.to_canonical_bytes(),
        aux_opening: aux_witness.to_canonical_bytes(),
        sender_memo: None,
    };

    let (recipient_box, _) =
        PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(&input_bundle, &opened).unwrap();
    let note = OutputNote::new(
        policy.commitment(),
        LweCiphertext::default(),
        aux_commit,
        recipient_box,
    );

    wallet.record_opened_note(note.clone(), opened).unwrap();
    let spend = wallet.spend_material(&note.note_commit).unwrap();

    let mut receive_wallet = PrivaiWallet::open(MemoryWalletStore::new()).unwrap();
    let receive_bundle = receive_wallet
        .create_local_bundle(500, 0, Some(b"recovery".to_vec()))
        .unwrap();

    // WRONG signer pair: Buyer + Operator instead of Buyer + Merchant
    // RecoveryRelease requires Buyer + Merchant per frozen rule table
    let auth_material = AuthMaterial {
        policy_tag: SpendPolicyTag::Escrow2of3 as u8,
        signer_pks: vec![buyer_pk.clone(), operator_pk.clone()],
        signatures: vec![vec![0x01; 64], vec![0x02; 64]],
        policy_opening: policy.to_canonical_bytes(),
        escrow_action: EscrowAction::RecoveryRelease as u8,
    };

    let assembly = FinalAssemblyInputs {
        proposal_hash: [0x11; 32],
        escrow_id: [0xEE; 32],
        action: EscrowAction::RecoveryRelease,
        funding_note_commit: spend.note_commit,
        output_recipient_pk: buyer_h,
        fee: 10,
        auth_material,
    };

    // The wallet builder validates signer pairs against the frozen rule table
    // and rejects mismatched combinations before any tx is built.
    let result =
        wallet.build_escrow_transfer_note_from_assembly_inputs(&spend, &assembly, &receive_bundle);

    assert!(
        result.is_err(),
        "RecoveryRelease with Buyer+Operator signers must be rejected"
    );
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("signer set does not satisfy escrow action requirements"),
        "Expected signer mismatch error, got: {}",
        err_msg
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// TEST 3: RecoveryRelease with invalid output target is rejected
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn recovery_release_invalid_output_target_rejected() {
    let (buyer_sk, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen buyer");
    let (merchant_sk, merchant_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen merchant");
    let (_operator_sk, operator_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen operator");

    // timeout_block = 1 — recovery valid at height >= 1
    let policy = make_escrow_policy(&buyer_pk, &merchant_pk, &operator_pk, 1);
    let (snapshot, note_commit) = setup_snapshot_with_escrow_note(&policy);

    // Output targeted to neither Buyer nor Merchant — uses an attacker key
    let attacker_commit = SpendPolicy::Single {
        falcon_pk_hash: [0xFF; 32],
    }
    .commitment();
    let attacker_output = make_output_with_policy(attacker_commit);

    let tx = Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: 1,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef { note_commit }],
            input_nullifiers: vec![Nullifier([0xE2; 32])],
            outputs: vec![attacker_output],
            fee: 0,
            statement_commit: [0xF1; 32],
            auth: vec![InputAuth {
                policy_tag: SpendPolicyTag::Escrow2of3 as u8,
                signer_pks: vec![buyer_pk.clone(), merchant_pk.clone()],
                signatures: vec![vec![], vec![]],
                policy_opening: Some(policy.to_canonical_bytes()),
                escrow_action: Some(EscrowAction::RecoveryRelease as u8),
            }],
        },
    });

    // Sign with real keys so we reach the output target check
    let tx_hash = tx.tx_signing_hash();
    let buyer_sig = nxms_transport::crypto::falcon_sign_ct_prepared(&buyer_sk, &tx_hash).unwrap();
    let merchant_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(&merchant_sk, &tx_hash).unwrap();

    let tx = match tx {
        Transaction::TransferNote(mut t) => {
            t.core.auth[0].signatures = vec![buyer_sig, merchant_sig];
            Transaction::TransferNote(t)
        }
        _ => unreachable!(),
    };

    // Validate at height 1 (>= timeout_block 1, so timeout gate passes)
    let result = privai_ledger::ledger::validate_transaction(&tx, &snapshot, 0, 1);
    assert!(
        result.is_err(),
        "RecoveryRelease with attacker output must be rejected"
    );
    assert!(
        matches!(
            result.unwrap_err(),
            ValidationError::EscrowOutputTargetMismatch
        ),
        "Expected EscrowOutputTargetMismatch for invalid output target"
    );
}
