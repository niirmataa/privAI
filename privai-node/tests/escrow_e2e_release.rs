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
fn e2e_honest_escrow_release_flow() {
    //
    // Setup participants
    //
    let (buyer_sk, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen buyer");
    let (_merchant_sk, merchant_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen merchant");
    let (operator_sk, operator_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen operator");

    let buyer_h = falcon_pk_hash(&buyer_pk);
    let merchant_h = falcon_pk_hash(&merchant_pk);
    let operator_h = falcon_pk_hash(&operator_pk);

    let escrow_id = [0xEE; 32];
    let proposal_hash = [0x11; 32];
    let amount = 1000;
    let fee = 10;
    let refund_amount = amount - fee;

    let escrow_policy = SpendPolicy::Escrow2of3 {
        buyer_pk_hash: buyer_h,
        merchant_pk_hash: merchant_h,
        operator_pk_hash: operator_h,
        timeout_block: 100,
    };

    let descriptor = EscrowFundingDescriptor {
        escrow_id,
        buyer_pk: buyer_h,
        merchant_pk: merchant_h,
        operator_pk: operator_h,
        amount: amount as u64,
        spend_policy_commit: escrow_policy.commitment(),
        timeout_blocks: 100,
    };

    // Initialize node
    let mut config = NodeConfig::example();
    config.data_dir = String::new(); // in memory
    let mut node = PrivaiNode::open(config.clone(), MemoryStore::new()).expect("node");

    //
    // Stage A: Setup & approvals in node
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
            action: 0,
        },
    }))
    .expect("handle proposal");

    // Buyer + Operator sign the control-plane approval (authorization material)
    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: buyer_h,
        signature: vec![0x11; 64], // fake authorization sigs
    }))
    .expect("buyer approval");

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: operator_h,
        signature: vec![0x22; 64],
    }))
    .expect("operator approval");

    // Node ensures quorum is complete
    assert!(node.is_escrow_quorum_ready(&proposal_hash));
    let bundle = node
        .build_escrow_approval_bundle(&proposal_hash)
        .expect("build bundle");
    assert_eq!(bundle.signer_pks.len(), 2);

    //
    // Setup Wallet and Funding Note (from Buyer)
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

    // Record the funding note into wallet
    buyer_wallet
        .record_opened_note(funding_note.clone(), funding_opened)
        .expect("record note");
    let spend_material = buyer_wallet
        .spend_material(&funding_note.note_commit)
        .expect("spend material");

    // Also seed the node ledger with this funding note so it can be spent later
    let mut ledger_snapshot = node.ledger().snapshot().clone();
    ledger_snapshot.notes.insert(
        funding_note.note_commit,
        NoteRecord {
            note: funding_note.clone(),
            created_in_block: Some(0),
            status: NoteStatus::Unspent,
        },
    );
    // Directly mutate snapshot through memory store isn't straightforward without recreating node,
    // so we just reconstruct the node to inject the unspent note.
    let mut store = MemoryStore::new();
    store.save(&ledger_snapshot).unwrap();
    let mut node = PrivaiNode::open(config.clone(), store).unwrap();
    // Also inject state back for the submit gate
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
            action: 0,
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
        signer_pk: operator_h,
        signature: vec![0x22; 64],
    }))
    .unwrap();

    //
    // Stage B: Final Assembly in Wallet
    //

    // Fake merchant receive bundle
    let mut merchant_wallet = PrivaiWallet::open(MemoryWalletStore::new()).unwrap();
    let merchant_receive_bundle = merchant_wallet
        .create_local_bundle(refund_amount as u64, 0, Some(b"Release payment".to_vec()))
        .unwrap();

    // Re-construct auth_material correctly for EscrowBuilder from node bundle
    // (Note: bundle contains signatures which are placeholder control plane auths;
    // later we sign the final tx_signing_hash).
    let auth_material = AuthMaterial {
        policy_tag: SpendPolicyTag::Escrow2of3 as u8,
        signer_pks: vec![buyer_pk.clone(), operator_pk.clone()],
        signatures: bundle.signatures.clone(), // placeholder, will replace after assembly
        policy_opening: escrow_policy.to_canonical_bytes(),
        escrow_action: EscrowAction::Release as u8,
    };

    let assembly = FinalAssemblyInputs {
        proposal_hash,
        escrow_id,
        action: EscrowAction::Release,
        funding_note_commit: spend_material.note_commit,
        output_recipient_pk: merchant_h,
        fee: fee as u64,
        auth_material,
    };

    let mut assembled = buyer_wallet
        .build_escrow_transfer_note_from_assembly_inputs(
            &spend_material,
            &assembly,
            &merchant_receive_bundle,
        )
        .expect("assemble escrow tx");

    // Real Signatures over tx_signing_hash
    let real_buyer_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(&buyer_sk, &assembled.tx_signing_hash)
            .unwrap();
    let real_operator_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(&operator_sk, &assembled.tx_signing_hash)
            .unwrap();

    // We know Buyer is index 0 and Operator is index 1 in the sorted auth_material based on their roles
    assembled.tx.core.auth[0].signatures = vec![real_buyer_sig, real_operator_sig];
    // Re-derive hash (even though signature change doesn't change it) to be pedantic
    assembled.tx_signing_hash = Transaction::TransferNote(assembled.tx.clone()).tx_signing_hash();
    assembled.proof_scaffolding = TransferProvingData::from_tx_and_witness(
        &assembled.tx,
        assembled.proof_scaffolding.witness.clone(),
    )
    .expect("rebuild proving data after final signatures");

    //
    // Proof Handoff (Attach Proof)
    //
    let handoff =
        privai_wallet::proof_handoff::EscrowProofReadyHandoff::build(&assembled, 100, 200, [8; 32])
            .expect("build proof handoff");

    let proof_result_bytes = vec![0xBE, 0xEF]; // Fake proof bytes from "prover"
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
    // Block Build and Finalization checking success
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

    // Honest Success Conditions:
    // 1. Funding note is spent
    let updated_snapshot = node.ledger().snapshot();
    let spent_record = updated_snapshot
        .notes
        .get(&funding_note.note_commit)
        .expect("funding note record");
    assert!(
        matches!(spent_record.status, NoteStatus::Spent { .. }),
        "escrow note should be spent"
    );
    // 2. Block successfully advanced
    assert_eq!(updated_snapshot.height, 1, "height advanced to 1");
    // 3. Artifact is safely stored
    let stored_artifacts = node.load_block_artifacts(&block.hash()).unwrap().unwrap();
    assert_eq!(
        stored_artifacts.proof_certificates(),
        block.body.proof_certificates
    );
}
