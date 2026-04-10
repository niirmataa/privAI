use privai_chain::escrow::EscrowAction;
use privai_chain::hash::falcon_pk_hash;
use privai_chain::{
    Amount14, AuxWitness, CanonicalEncode, LweCiphertext, OutputNote, RecipientBoxPlaintext,
    SpendPolicy, SpendPolicyTag, Transaction, PRIVAI_V0,
};
use privai_ledger::{LedgerStore, MemoryStore, NoteRecord, NoteStatus};
use privai_node::{NodeConfig, PrivaiNode};
use privai_nxms::{
    ContextId, EscrowApprovalBody, EscrowFundedBody, EscrowFundingDescriptor, EscrowSpendProposal,
    EscrowSpendProposalBody, PrivaiBody,
};
use privai_proof::TransferProvingData;
use privai_wallet::escrow_builder::{AuthMaterial, FinalAssemblyInputs};
use privai_wallet::{MemoryWalletStore, PrivaiWallet};

fn ctx(fill: u8) -> ContextId {
    [fill; 16]
}

#[test]
fn e2e_honest_escrow_recovery_release_at_boundary() {
    //
    // STEP 1 — Keygen
    //
    let (buyer_sk, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen buyer");
    let (merchant_sk, merchant_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen merchant");
    let (_operator_sk, operator_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen operator");

    let buyer_h = falcon_pk_hash(&buyer_pk);
    let merchant_h = falcon_pk_hash(&merchant_pk);
    let operator_h = falcon_pk_hash(&operator_pk);

    let escrow_id = [0xEE; 32];
    let proposal_hash = [0x11; 32];
    let amount = 1000;
    let fee = 10;
    let recovery_amount = amount - fee;

    // timeout_block = 1: mempool validates at snapshot.height + 1 = 1,
    // so 1 < 1 is false → recovery is allowed.
    let escrow_policy = SpendPolicy::Escrow2of3 {
        buyer_pk_hash: buyer_h,
        merchant_pk_hash: merchant_h,
        operator_pk_hash: operator_h,
        timeout_block: 1,
    };

    let descriptor = EscrowFundingDescriptor {
        escrow_id,
        buyer_pk: buyer_h,
        merchant_pk: merchant_h,
        operator_pk: operator_h,
        amount: amount as u64,
        spend_policy_commit: escrow_policy.commitment(),
        timeout_blocks: 1,
    };

    // Initialize node
    let mut config = NodeConfig::example();
    config.data_dir = String::new(); // in memory
    let mut node = PrivaiNode::open(config.clone(), MemoryStore::new()).expect("node");

    //
    // STEP 3 — Stage A: Setup & approvals in node
    // RecoveryRelease: proposal action = 2
    // Signers: Buyer + Merchant (NOT Operator)
    //
    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor: descriptor.clone(),
        funding_tx_ref: [0x55; 32],
    }))
    .expect("handle funded");

    node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(1),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: [0x66; 32],
            action: 2,
        },
    }))
    .expect("handle proposal");

    // CHECKPOINT 2: Buyer + Merchant sign — Operator does NOT sign
    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: buyer_h,
        signature: vec![0x11; 64],
    }))
    .expect("buyer approval");

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: merchant_h,
        signature: vec![0x22; 64],
    }))
    .expect("merchant approval");

    // Node ensures quorum is complete
    assert!(node.is_escrow_quorum_ready(&proposal_hash));
    let bundle = node
        .build_escrow_approval_bundle(&proposal_hash)
        .expect("build bundle");
    assert_eq!(bundle.signer_pks.len(), 2);
    // Verify Operator is NOT in the signing set
    let bundle_hashes: Vec<_> = bundle.signer_pks.iter().collect();
    assert!(
        !bundle_hashes.contains(&&operator_h),
        "Operator must not be in RecoveryRelease signing set"
    );
    assert!(bundle_hashes.contains(&&buyer_h));
    assert!(bundle_hashes.contains(&&merchant_h));

    //
    // STEP 4 — Funding note + replay
    //
    let mut buyer_wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("buyer wallet");
    let funding_bundle = buyer_wallet
        .create_local_bundle(amount as u64, 0, Some(b"escrow funding".to_vec()))
        .expect("funding bundle");

    let funding_amount = Amount14::new(amount).expect("amount");
    let aux_witness = AuxWitness {
        version: PRIVAI_V0,
        amount: funding_amount,
        witness_seed: [0x22; 32],
        noise_class: 1,
        bundle_id: funding_bundle.bundle_id,
    };
    let aux_commit = privai_chain::derive_aux_commit(&aux_witness);
    let note_payload_commit = OutputNote::payload_commit_from_parts(
        PRIVAI_V0,
        &escrow_policy.commitment(),
        &LweCiphertext::default(),
        &aux_commit,
    );

    let funding_opened = RecipientBoxPlaintext {
        version: PRIVAI_V0,
        bundle_id: funding_bundle.bundle_id,
        note_payload_commit,
        amount: funding_amount,
        witness_seed: [0x22; 32],
        nullifier_key: [0x33; 32],
        spend_policy_opening: escrow_policy.to_canonical_bytes(),
        aux_opening: aux_witness.to_canonical_bytes(),
        sender_memo: None,
    };

    let (recipient_box, _) =
        PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(&funding_bundle, &funding_opened)
            .expect("seal");

    let funding_note = OutputNote::new(
        escrow_policy.commitment(),
        LweCiphertext::default(),
        aux_commit,
        recipient_box,
    );

    buyer_wallet
        .record_opened_note(funding_note.clone(), funding_opened)
        .expect("record note");
    let spend_material = buyer_wallet
        .spend_material(&funding_note.note_commit)
        .expect("spend material");

    // Seed the node ledger with this funding note
    let mut ledger_snapshot = node.ledger().snapshot().clone();
    ledger_snapshot.notes.insert(
        funding_note.note_commit,
        NoteRecord {
            note: funding_note.clone(),
            created_in_block: Some(0),
            status: NoteStatus::Unspent,
        },
    );
    let mut store = MemoryStore::new();
    store.save(&ledger_snapshot).unwrap();
    let mut node = PrivaiNode::open(config.clone(), store).unwrap();

    // Replay Stage A events
    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor,
        funding_tx_ref: [0x55; 32],
    }))
    .unwrap();
    node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(1),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: [0x66; 32],
            action: 2,
        },
    }))
    .unwrap();
    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: buyer_h,
        signature: vec![0x11; 64],
    }))
    .unwrap();
    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: merchant_h,
        signature: vec![0x22; 64],
    }))
    .unwrap();

    // CHECKPOINT 3: Quorum must still be ready after replay
    assert!(node.is_escrow_quorum_ready(&proposal_hash));

    //
    // STEP 5 — Stage B / wallet assembly
    // action = RecoveryRelease
    // auth material signer PKs = Buyer + Merchant
    // output recipient = Buyer (valid for RecoveryRelease Either target)
    // no Operator signing
    //
    let buyer_receive_bundle = buyer_wallet
        .create_local_bundle(
            recovery_amount as u64,
            0,
            Some(b"Recovery payment".to_vec()),
        )
        .unwrap();

    // CHECKPOINT 4: Output target is Buyer — valid for RecoveryRelease Either(Buyer, Merchant)
    let auth_material = AuthMaterial {
        policy_tag: SpendPolicyTag::Escrow2of3 as u8,
        signer_pks: vec![buyer_pk.clone(), merchant_pk.clone()],
        signatures: bundle.signatures.clone(),
        policy_opening: escrow_policy.to_canonical_bytes(),
        escrow_action: EscrowAction::RecoveryRelease as u8,
    };

    let assembly = FinalAssemblyInputs {
        proposal_hash,
        escrow_id,
        action: EscrowAction::RecoveryRelease,
        funding_note_commit: spend_material.note_commit,
        output_recipient_pk: buyer_h,
        fee: fee as u64,
        auth_material,
    };

    let mut assembled = buyer_wallet
        .build_escrow_transfer_note_from_assembly_inputs(
            &spend_material,
            &assembly,
            &buyer_receive_bundle,
        )
        .expect("assemble escrow tx");

    //
    // STEP 6 — Real signatures: sign tx_signing_hash with Buyer + Merchant keys
    //
    let real_buyer_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(&buyer_sk, &assembled.tx_signing_hash)
            .unwrap();
    let real_merchant_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(&merchant_sk, &assembled.tx_signing_hash)
            .unwrap();

    // CHECKPOINT 5: Operator does NOT sign — only Buyer + Merchant
    assembled.tx.core.auth[0].signatures = vec![real_buyer_sig, real_merchant_sig];
    assembled.tx_signing_hash = Transaction::TransferNote(assembled.tx.clone()).tx_signing_hash();
    assembled.proof_scaffolding = TransferProvingData::from_tx_and_witness(
        &assembled.tx,
        assembled.proof_scaffolding.witness.clone(),
    )
    .expect("rebuild proving data after final signatures");

    //
    // Proof Handoff
    //
    let handoff =
        privai_wallet::proof_handoff::EscrowProofReadyHandoff::build(&assembled, 100, 200, [8; 32])
            .expect("build proof handoff");

    let proof_result_bytes = vec![0xBE, 0xEF];
    let attached_proof = handoff
        .attach_single_tx_proof_result(proof_result_bytes.clone(), 1, vec![[0x99; 32]], [0xAA; 32])
        .expect("attach proof");

    //
    // Submit Gate (Node)
    //
    node.submit_escrow_transfer_note(
        &proposal_hash,
        Transaction::TransferNote(assembled.tx.clone()),
        assembled.tx.core.fee,
    )
    .expect("submit escrow tx");

    //
    // STEP 7 — Block / import path: boundary case height == timeout_block
    // Block height will be 1, timeout_block = 1 in policy.
    // RecoveryRelease exercises the boundary: height (1) >= timeout_block (1),
    // which is the edge condition where recovery becomes available.
    //
    let block = node
        .propose_block(
            0,
            0,
            1_000,
            [2; 32],
            [3; 32],
            vec![attached_proof.artifact.certificate()],
            Vec::new(),
        )
        .expect("propose block");

    let artifacts = privai_proof::artifact::BlockProofArtifacts::from_transfer_proofs(
        block.hash(),
        std::slice::from_ref(&attached_proof.handoff.proving_data),
        vec![attached_proof.artifact.clone()],
    )
    .expect("build sidecar artifacts");

    node.import_block_with_artifacts(&block, Some(&artifacts))
        .expect("import block");

    //
    // STEP 8 — Assertions
    //
    // 1. Funding note is spent
    let updated_snapshot = node.ledger().snapshot();
    let spent_record = updated_snapshot
        .notes
        .get(&funding_note.note_commit)
        .expect("funding note record");
    assert!(
        matches!(spent_record.status, NoteStatus::Spent { .. }),
        "escrow note should be spent after RecoveryRelease"
    );
    // 2. Block height advanced
    assert_eq!(updated_snapshot.height, 1, "height advanced to 1");
    // 3. Artifacts stored
    let stored_artifacts = node.load_block_artifacts(&block.hash()).unwrap().unwrap();
    assert_eq!(
        stored_artifacts.proof_certificates(),
        block.body.proof_certificates
    );
    // 4. Output spend policy is aligned with buyer (not merchant) — confirms
    //    the receive bundle and recipient_pk are consistent for the same party.
    let output_note = &assembled.tx.core.outputs[0];
    assert_eq!(
        output_note.spend_policy_commit,
        SpendPolicy::Single {
            falcon_pk_hash: buyer_h,
        }
        .commitment(),
        "output spend policy must be Buyer's, matching the receive bundle owner"
    );
    // 5. Witness seed is non-default — proves the builder derived real data
    //    from the buyer receive bundle, not a placeholder.
    let witness_seed = assembled.proof_scaffolding.witness.outputs[0]
        .recipient_opening
        .witness_seed;
    assert_ne!(witness_seed, [0u8; 32], "witness seed must be non-zero");
    assert_ne!(
        witness_seed, [0x88; 32],
        "witness seed must not be a test placeholder"
    );
}
