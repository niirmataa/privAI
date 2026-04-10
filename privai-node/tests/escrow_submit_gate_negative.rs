use privai_chain::escrow::EscrowAction;
use privai_chain::hash::falcon_pk_hash;
use privai_chain::{
    Amount14, AuxWitness, CanonicalDecode, CanonicalEncode, Hash32, LweCiphertext, OutputNote,
    RecipientBoxPlaintext, SpendPolicy, SpendPolicyTag, Transaction, PRIVAI_V0,
};
use privai_ledger::{LedgerStore, MemoryStore, NoteRecord, NoteStatus};
use privai_node::{NodeConfig, NodeError, PrivaiNode};
use privai_nxms::{
    ContextId, EscrowApprovalBody, EscrowFundedBody, EscrowFundingDescriptor, EscrowSpendProposal,
    EscrowSpendProposalBody, PrivaiBody,
};
use privai_wallet::escrow_builder::{AuthMaterial, FinalAssemblyInputs};
use privai_wallet::{MemoryWalletStore, PrivaiWallet};

fn ctx(fill: u8) -> ContextId {
    [fill; 16]
}

struct TestSetup {
    node: PrivaiNode<MemoryStore>,
    proposal_hash: Hash32,
    assembled_tx: Transaction,
    fee: u64,
}

fn setup_escrow_stage_a(action_type: u8, include_all_approvals: bool) -> TestSetup {
    let (_, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen buyer");
    let (_, merchant_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen merchant");
    let (_, operator_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen operator");

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

    let mut config = NodeConfig::example();
    config.data_dir = String::new(); // in memory
    let mut node = PrivaiNode::open(config.clone(), MemoryStore::new()).expect("node");

    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor: descriptor.clone(),
        funding_tx_ref: [0x55; 32],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(1),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: [0x66; 32],
            action: action_type, // 0 = Release, 1 = Refund
        },
    }))
    .unwrap();

    // Add first approval (Buyer for Release, Merchant for Refund)
    if action_type == 0 {
        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: buyer_h,
            signature: vec![0x11; 64],
        }))
        .unwrap();
    } else {
        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: merchant_h,
            signature: vec![0x11; 64],
        }))
        .unwrap();
    }

    if include_all_approvals {
        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: operator_h,
            signature: vec![0x22; 64],
        }))
        .unwrap();
    }

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

    // Inject note to node
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

    // Re-inject state
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
            action: action_type,
        },
    }))
    .unwrap();

    if action_type == 0 {
        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: buyer_h,
            signature: vec![0x11; 64],
        }))
        .unwrap();
    } else {
        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: merchant_h,
            signature: vec![0x11; 64],
        }))
        .unwrap();
    }

    if include_all_approvals {
        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: operator_h,
            signature: vec![0x22; 64],
        }))
        .unwrap();
    }

    let receive_bundle = buyer_wallet
        .create_local_bundle(refund_amount as u64, 0, Some(b"Test payment".to_vec()))
        .unwrap();

    let auth_material = AuthMaterial {
        policy_tag: SpendPolicyTag::Escrow2of3 as u8,
        signer_pks: if action_type == 0 {
            vec![buyer_pk.clone(), operator_pk.clone()]
        } else {
            vec![merchant_pk.clone(), operator_pk.clone()]
        },
        signatures: vec![vec![0x11; 64], vec![0x22; 64]],
        policy_opening: escrow_policy.to_canonical_bytes(),
        escrow_action: if action_type == 0 {
            EscrowAction::Release as u8
        } else {
            EscrowAction::Refund as u8
        },
    };

    let assembly = FinalAssemblyInputs {
        proposal_hash,
        escrow_id,
        action: if action_type == 0 {
            EscrowAction::Release
        } else {
            EscrowAction::Refund
        },
        funding_note_commit: spend_material.note_commit,
        output_recipient_pk: if action_type == 0 {
            merchant_h
        } else {
            buyer_h
        },
        fee: fee as u64,
        auth_material,
    };

    let assembled = buyer_wallet
        .build_escrow_transfer_note_from_assembly_inputs(
            &spend_material,
            &assembly,
            &receive_bundle,
        )
        .expect("assemble escrow tx");

    TestSetup {
        node,
        proposal_hash,
        assembled_tx: Transaction::TransferNote(assembled.tx),
        fee: fee as u64,
    }
}

#[test]
fn submit_without_quorum_fails() {
    let mut setup = setup_escrow_stage_a(0, false); // only 1 approval

    let result =
        setup
            .node
            .submit_escrow_transfer_note(&setup.proposal_hash, setup.assembled_tx, setup.fee);

    assert!(result.is_err());
    if let Err(NodeError::EscrowSubmit(msg)) = result {
        assert!(
            msg.contains("quorum not met for proposal"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected NodeError::EscrowSubmit, got {:?}", result);
    }
}

#[test]
fn submit_with_wrong_proposal_hash_fails() {
    let mut setup = setup_escrow_stage_a(0, true); // full quorum

    let wrong_hash = [0x99; 32];
    let result = setup
        .node
        .submit_escrow_transfer_note(&wrong_hash, setup.assembled_tx, setup.fee);

    assert!(result.is_err());
    if let Err(NodeError::EscrowSubmit(msg)) = result {
        assert!(
            msg.contains("proposal not found"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected NodeError::EscrowSubmit, got {:?}", result);
    }
}

#[test]
fn submit_non_transfer_note_fails() {
    let mut setup = setup_escrow_stage_a(0, true); // full quorum

    let tx_core = match setup.assembled_tx {
        Transaction::TransferNote(t) => t.core,
        _ => panic!("Expected TransferNote"),
    };

    let settlement_tx = privai_chain::SettlementTx {
        core: tx_core,
        settlement_id: [0x55; 32],
        marketplace_context: [0x55; 16],
        phase: privai_chain::SettlementPhase::Release,
        payload_commit: [0x55; 32],
    };
    let wrong_tx = Transaction::Settlement(settlement_tx);

    let result = setup
        .node
        .submit_escrow_transfer_note(&setup.proposal_hash, wrong_tx, setup.fee);

    assert!(result.is_err());
    if let Err(NodeError::EscrowSubmit(msg)) = result {
        assert!(
            msg.contains("expected Transaction::TransferNote"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected NodeError::EscrowSubmit, got {:?}", result);
    }
}

#[test]
fn submit_with_action_mismatch_fails() {
    let mut setup = setup_escrow_stage_a(0, true); // Proposal action = 0 (Release)

    // Modify the assembled tx to have a Refund action (0x02) in its auth material.
    let mut tx = match setup.assembled_tx {
        Transaction::TransferNote(t) => t,
        _ => panic!("Expected TransferNote"),
    };

    tx.core.auth[0].escrow_action = Some(EscrowAction::Refund as u8);
    let mutated_tx = Transaction::TransferNote(tx);

    let result =
        setup
            .node
            .submit_escrow_transfer_note(&setup.proposal_hash, mutated_tx, setup.fee);

    assert!(result.is_err());
    if let Err(NodeError::EscrowSubmit(msg)) = result {
        assert!(
            msg.contains("escrow action mismatch"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected NodeError::EscrowSubmit, got {:?}", result);
    }
}

#[test]
fn submit_with_missing_policy_opening_fails() {
    // Gate check 6: policy_opening must be present.
    let mut setup = setup_escrow_stage_a(0, true); // Release, full quorum

    let mut tx = match setup.assembled_tx {
        Transaction::TransferNote(t) => t,
        _ => panic!("Expected TransferNote"),
    };

    // Remove policy_opening from the single Escrow2of3 auth entry.
    tx.core.auth[0].policy_opening = None;

    let result = setup.node.submit_escrow_transfer_note(
        &setup.proposal_hash,
        Transaction::TransferNote(tx),
        setup.fee,
    );

    assert!(result.is_err());
    if let Err(NodeError::EscrowSubmit(msg)) = result {
        assert!(
            msg.contains("missing policy_opening"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected NodeError::EscrowSubmit, got {:?}", result);
    }
}

#[test]
fn submit_with_multiple_escrow_auth_entries_fails() {
    // Gate check 5: exactly 1 Escrow2of3 auth entry required.
    let mut setup = setup_escrow_stage_a(0, true); // Release, full quorum

    let mut tx = match setup.assembled_tx {
        Transaction::TransferNote(t) => t,
        _ => panic!("Expected TransferNote"),
    };

    // Clone the Escrow2of3 auth entry to create a second one.
    let mut second_auth = tx.core.auth[0].clone();
    // Give it distinct signer PKs so it passes per-entry checks.
    second_auth.signer_pks = vec![vec![0xAA; 897], vec![0xBB; 897]];
    second_auth.signatures = vec![vec![0x11; 64], vec![0x22; 64]];
    tx.core.auth.push(second_auth);

    let result = setup.node.submit_escrow_transfer_note(
        &setup.proposal_hash,
        Transaction::TransferNote(tx),
        setup.fee,
    );

    assert!(result.is_err());
    if let Err(NodeError::EscrowSubmit(msg)) = result {
        assert!(
            msg.contains("expected exactly 1 Escrow2of3 auth entry"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected NodeError::EscrowSubmit, got {:?}", result);
    }
}

#[test]
fn submit_with_signer_set_mismatch_fails() {
    // Gate check 8: tx signer PK set must match Stage A bundle signer set.
    let mut setup = setup_escrow_stage_a(0, true); // Release, full quorum

    let mut tx = match setup.assembled_tx {
        Transaction::TransferNote(t) => t,
        _ => panic!("Expected TransferNote"),
    };

    // Replace one signer PK with a full Falcon key not in the escrow.
    let (_, rogue_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen rogue");
    tx.core.auth[0].signer_pks[0] = rogue_pk;

    let result = setup.node.submit_escrow_transfer_note(
        &setup.proposal_hash,
        Transaction::TransferNote(tx),
        setup.fee,
    );

    assert!(result.is_err());
    if let Err(NodeError::EscrowSubmit(msg)) = result {
        assert!(
            msg.contains("signer set mismatch"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected NodeError::EscrowSubmit, got {:?}", result);
    }
}

#[test]
fn submit_with_timeout_block_mismatch_fails() {
    // Gate check 6b: decoded timeout_block from policy_opening must match
    // funded descriptor timeout_blocks.
    let mut setup = setup_escrow_stage_a(0, true); // Release, full quorum

    let mut tx = match setup.assembled_tx {
        Transaction::TransferNote(t) => t,
        _ => panic!("Expected TransferNote"),
    };

    // Decode policy_opening, mutate timeout, re-encode.
    let policy_bytes = tx.core.auth[0]
        .policy_opening
        .as_ref()
        .expect("policy_opening present")
        .clone();
    let mut decoded =
        SpendPolicy::from_canonical_bytes(&policy_bytes).expect("decode policy_opening");
    match &mut decoded {
        SpendPolicy::Escrow2of3 { timeout_block, .. } => {
            *timeout_block = 999; // differs from descriptor's 100
        }
        _ => panic!("expected Escrow2of3 policy"),
    }
    tx.core.auth[0].policy_opening = Some(decoded.to_canonical_bytes());

    let result = setup.node.submit_escrow_transfer_note(
        &setup.proposal_hash,
        Transaction::TransferNote(tx),
        setup.fee,
    );

    assert!(result.is_err());
    if let Err(NodeError::EscrowSubmit(msg)) = result {
        assert!(
            msg.contains("timeout_block mismatch"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected NodeError::EscrowSubmit, got {:?}", result);
    }
}
