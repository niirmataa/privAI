use privai_chain::{
    hash::domain_hash, CanonicalEncode, InputAuth, InputRef, LweCiphertext, Nullifier, OutputNote,
    RecipientBox, SpendPolicy, SpendPolicyTag, Transaction, TransferNoteTx, TxCore, PRIVAI_V0,
    TX_TYPE_TRANSFER_NOTE,
};
use privai_ledger::state::{NoteRecord, NoteStatus};
use privai_ledger::MemoryStore;
use privai_node::{NodeConfig, NodeError, PrivaiNode};
use privai_nxms::{
    ContextId, EscrowApprovalBody, EscrowFundedBody, EscrowFundingDescriptor, EscrowSpendProposal,
    EscrowSpendProposalBody, Hash32, PrivaiBody,
};

fn make_node() -> PrivaiNode<MemoryStore> {
    let mut config = NodeConfig::example();
    config.data_dir = String::new();
    PrivaiNode::open(config, MemoryStore::new()).expect("node")
}

fn ctx(fill: u8) -> ContextId {
    [fill; 16]
}

fn dummy_output_note(seed: u8) -> OutputNote {
    OutputNote::new(
        [seed; 32],
        LweCiphertext::default(),
        [seed.wrapping_add(1); 32],
        RecipientBox::new(
            vec![seed],
            [seed; 24],
            vec![seed.wrapping_add(1)],
            [seed; 16],
            [seed; 16],
        ),
    )
}

fn escrow_output_note(policy: &SpendPolicy, seed: u8) -> OutputNote {
    OutputNote::new(
        policy.commitment(),
        LweCiphertext::default(),
        [seed.wrapping_add(1); 32],
        RecipientBox::new(
            vec![seed],
            [seed; 24],
            vec![seed.wrapping_add(1)],
            [seed; 16],
            [seed; 16],
        ),
    )
}

fn merchant_output_note(merchant_pk_hash: &Hash32, seed: u8) -> OutputNote {
    let policy = SpendPolicy::Single {
        falcon_pk_hash: *merchant_pk_hash,
    };
    escrow_output_note(&policy, seed)
}

fn buyer_output_note(buyer_pk_hash: &Hash32, seed: u8) -> OutputNote {
    let policy = SpendPolicy::Single {
        falcon_pk_hash: *buyer_pk_hash,
    };
    escrow_output_note(&policy, seed)
}

fn pk_hash(pk: &[u8]) -> Hash32 {
    domain_hash("privai:falcon-pk:v0", &[pk])
}

/// Full test setup: generates Falcon keys, funds escrow, creates proposal,
/// collects quorum approvals, and builds a valid escrow TransferNoteTx.
fn setup_escrow_with_tx() -> (
    PrivaiNode<MemoryStore>,
    Hash32,
    Transaction,
    Vec<u8>, // buyer_sk
    Vec<u8>, // operator_sk
    Vec<u8>, // merchant_sk
    Hash32,  // merchant_pk_hash
) {
    let (buyer_sk, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen buyer");
    let (merchant_sk, merchant_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen merchant");
    let (operator_sk, operator_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen operator");

    let buyer_sk = buyer_sk.to_vec();
    let merchant_sk = merchant_sk.to_vec();
    let operator_sk = operator_sk.to_vec();

    let buyer_h = pk_hash(&buyer_pk);
    let merchant_h = pk_hash(&merchant_pk);
    let operator_h = pk_hash(&operator_pk);

    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    let descriptor = EscrowFundingDescriptor {
        escrow_id,
        buyer_pk: buyer_h,
        merchant_pk: merchant_h,
        operator_pk: operator_h,
        amount: 1000,
        spend_policy_commit: SpendPolicy::Escrow2of3 {
            buyer_pk_hash: buyer_h,
            merchant_pk_hash: merchant_h,
            operator_pk_hash: operator_h,
            timeout_block: 100,
        }
        .commitment(),
        timeout_blocks: 100,
    };

    let mut node = make_node();

    // Fund
    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor,
        funding_tx_ref: [20; 32],
    }))
    .unwrap();

    // Proposal: action=0 (release)
    node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(1),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: [40; 32],
            action: 0,
        },
    }))
    .unwrap();

    // Approvals: buyer + operator (release requires buyer + operator)
    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: buyer_h,
        signature: vec![0xAA],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: operator_h,
        signature: vec![0xBB],
    }))
    .unwrap();

    // Build a valid escrow TransferNoteTx
    let policy_opening = SpendPolicy::Escrow2of3 {
        buyer_pk_hash: buyer_h,
        merchant_pk_hash: merchant_h,
        operator_pk_hash: operator_h,
        timeout_block: 100,
    }
    .to_canonical_bytes();

    let escrow_policy = SpendPolicy::Escrow2of3 {
        buyer_pk_hash: buyer_h,
        merchant_pk_hash: merchant_h,
        operator_pk_hash: operator_h,
        timeout_block: 100,
    };

    let input_note = escrow_output_note(&escrow_policy, 0xAA);
    let escrow_input = InputRef {
        note_commit: input_note.note_commit,
    };

    // Seed the ledger with the input note (unspent)
    node.ledger_mut().snapshot_mut().notes.insert(
        input_note.note_commit,
        NoteRecord {
            note: input_note,
            created_in_block: Some(0),
            status: NoteStatus::Unspent,
        },
    );

    // Output note must have Single(merchant) spend_policy_commit for release action
    let output = merchant_output_note(&merchant_h, 42);

    let mut core = TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![escrow_input],
        input_nullifiers: vec![Nullifier([0xBB; 32])],
        outputs: vec![output],
        fee: 10,
        statement_commit: [0xCC; 32],
        auth: vec![InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![buyer_pk.clone(), operator_pk.clone()],
            signatures: vec![],
            policy_opening: Some(policy_opening),
            escrow_action: Some(0x01), // Release
        }],
    };

    // Sign tx_signing_hash (canonical signing message per spec)
    let tx_for_signing = Transaction::TransferNote(TransferNoteTx { core: core.clone() });
    let signing_msg = tx_for_signing.tx_signing_hash();

    let buyer_sig = nxms_transport::crypto::falcon_sign_ct_prepared(&buyer_sk, &signing_msg)
        .expect("sign buyer");
    let operator_sig = nxms_transport::crypto::falcon_sign_ct_prepared(&operator_sk, &signing_msg)
        .expect("sign operator");

    core.auth[0].signatures = vec![buyer_sig, operator_sig];

    let tx = Transaction::TransferNote(TransferNoteTx { core });

    (
        node,
        proposal_hash,
        tx,
        buyer_sk,
        operator_sk,
        merchant_sk,
        merchant_h,
    )
}

// ── Test 1: submit before quorum returns controlled error ──────────────

#[test]
fn submit_before_quorum_returns_error() {
    let mut node = make_node();
    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor: EscrowFundingDescriptor {
            escrow_id,
            buyer_pk: [1; 32],
            merchant_pk: [2; 32],
            operator_pk: [3; 32],
            amount: 1000,
            spend_policy_commit: [4; 32],
            timeout_blocks: 100,
        },
        funding_tx_ref: [20; 32],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(1),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: [40; 32],
            action: 0,
        },
    }))
    .unwrap();

    // Only 1 approval — not quorum
    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [1; 32],
        signature: vec![0xAA],
    }))
    .unwrap();

    // Build a dummy TransferNoteTx (doesn't matter, should fail at quorum check)
    let tx = Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: PRIVAI_V0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef {
                note_commit: [1; 32],
            }],
            input_nullifiers: vec![Nullifier([2; 32])],
            outputs: vec![dummy_output_note(10)],
            fee: 1,
            statement_commit: [3; 32],
            auth: vec![InputAuth {
                policy_tag: SpendPolicyTag::Escrow2of3 as u8,
                signer_pks: vec![[1; 32].to_vec()],
                signatures: vec![vec![0xAA]],
                policy_opening: Some(vec![1]),
                escrow_action: Some(0x01),
            }],
        },
    });

    let result = node.submit_escrow_transfer_note(&proposal_hash, tx, 1000);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(msg.contains("quorum"), "should mention quorum: {msg}");
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── Test 2: unknown proposal returns controlled error ──────────────────

#[test]
fn unknown_proposal_returns_error() {
    let mut node = make_node();
    let fake_proposal = [0xFF; 32];

    let tx = Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: PRIVAI_V0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef {
                note_commit: [1; 32],
            }],
            input_nullifiers: vec![Nullifier([2; 32])],
            outputs: vec![dummy_output_note(10)],
            fee: 1,
            statement_commit: [3; 32],
            auth: vec![InputAuth {
                policy_tag: SpendPolicyTag::Escrow2of3 as u8,
                signer_pks: vec![[1; 32].to_vec()],
                signatures: vec![vec![0xAA]],
                policy_opening: Some(vec![1]),
                escrow_action: Some(0x01),
            }],
        },
    });

    let result = node.submit_escrow_transfer_note(&fake_proposal, tx, 1000);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(msg.contains("not found"), "should mention not found: {msg}");
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── Test 3: non-TransferNote tx is rejected ────────────────────────────

#[test]
fn non_transfer_note_rejected() {
    let (mut node, proposal_hash, _tx, _, _, _, _merchant_h) = setup_escrow_with_tx();

    // Use a SettlementTx instead of TransferNote
    let settlement_tx = Transaction::Settlement(privai_chain::SettlementTx {
        core: TxCore {
            version: PRIVAI_V0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![],
            input_nullifiers: vec![],
            outputs: vec![],
            fee: 0,
            statement_commit: [0; 32],
            auth: vec![InputAuth {
                policy_tag: SpendPolicyTag::Escrow2of3 as u8,
                signer_pks: vec![[1; 32].to_vec()],
                signatures: vec![vec![0xAA]],
                policy_opening: Some(vec![1]),
                escrow_action: Some(0x01),
            }],
        },
        settlement_id: [0; 32],
        marketplace_context: ctx(0),
        phase: privai_chain::SettlementPhase::Open,
        payload_commit: [0; 32],
    });

    let result = node.submit_escrow_transfer_note(&proposal_hash, settlement_tx, 1000);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("TransferNote"),
                "should mention TransferNote: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── Test 4: mismatched escrow action is rejected ──────────────────────

#[test]
fn mismatched_action_rejected() {
    let (buyer_sk, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (_merchant_sk, merchant_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (operator_sk, operator_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");

    let _buyer_sk = buyer_sk.to_vec();
    let _operator_sk = operator_sk.to_vec();

    let buyer_h = pk_hash(&buyer_pk);
    let merchant_h = pk_hash(&merchant_pk);
    let operator_h = pk_hash(&operator_pk);

    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    let mut node = make_node();

    let descriptor = EscrowFundingDescriptor {
        escrow_id,
        buyer_pk: buyer_h,
        merchant_pk: merchant_h,
        operator_pk: operator_h,
        amount: 1000,
        spend_policy_commit: SpendPolicy::Escrow2of3 {
            buyer_pk_hash: buyer_h,
            merchant_pk_hash: merchant_h,
            operator_pk_hash: operator_h,
            timeout_block: 100,
        }
        .commitment(),
        timeout_blocks: 100,
    };

    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor,
        funding_tx_ref: [20; 32],
    }))
    .unwrap();

    // Proposal: action=0 (release)
    node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(1),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: [40; 32],
            action: 0,
        },
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: buyer_h,
        signature: vec![0xAA],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: operator_h,
        signature: vec![0xBB],
    }))
    .unwrap();

    // Build tx with matching policy_opening but wrong action (0x02 = Refund, proposal expects release)
    let policy_opening = SpendPolicy::Escrow2of3 {
        buyer_pk_hash: buyer_h,
        merchant_pk_hash: merchant_h,
        operator_pk_hash: operator_h,
        timeout_block: 100,
    }
    .to_canonical_bytes();

    let core = TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![InputRef {
            note_commit: [0xAA; 32],
        }],
        input_nullifiers: vec![Nullifier([0xBB; 32])],
        outputs: vec![dummy_output_note(42)],
        fee: 10,
        statement_commit: [0xCC; 32],
        auth: vec![InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![buyer_pk, operator_pk],
            signatures: vec![vec![0xAA], vec![0xBB]],
            policy_opening: Some(policy_opening),
            escrow_action: Some(0x02), // Refund — but proposal is release (0)
        }],
    };

    let wrong_tx = Transaction::TransferNote(TransferNoteTx { core });

    let result = node.submit_escrow_transfer_note(&proposal_hash, wrong_tx, 1000);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("action mismatch"),
                "should mention action mismatch: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── Test 5: mismatched signer set vs Stage A bundle is rejected ────────

#[test]
fn mismatched_signers_rejected() {
    let (buyer_sk, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (_merchant_sk, merchant_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (_operator_sk, operator_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");

    let _buyer_sk = buyer_sk.to_vec();
    let _merchant_sk = _merchant_sk.to_vec();

    let buyer_h = pk_hash(&buyer_pk);
    let merchant_h = pk_hash(&merchant_pk);
    let operator_h = pk_hash(&operator_pk);

    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    let mut node = make_node();

    let descriptor = EscrowFundingDescriptor {
        escrow_id,
        buyer_pk: buyer_h,
        merchant_pk: merchant_h,
        operator_pk: operator_h,
        amount: 1000,
        spend_policy_commit: SpendPolicy::Escrow2of3 {
            buyer_pk_hash: buyer_h,
            merchant_pk_hash: merchant_h,
            operator_pk_hash: operator_h,
            timeout_block: 100,
        }
        .commitment(),
        timeout_blocks: 100,
    };

    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor,
        funding_tx_ref: [20; 32],
    }))
    .unwrap();

    // Proposal: action=0 (release)
    node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(1),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: [40; 32],
            action: 0,
        },
    }))
    .unwrap();

    // Approvals: buyer + operator (matches release signers in bundle)
    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: buyer_h,
        signature: vec![0xAA],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: operator_h,
        signature: vec![0xBB],
    }))
    .unwrap();

    // Build tx with matching policy_opening but wrong signer set (buyer + merchant instead of buyer + operator)
    let policy_opening = SpendPolicy::Escrow2of3 {
        buyer_pk_hash: buyer_h,
        merchant_pk_hash: merchant_h,
        operator_pk_hash: operator_h,
        timeout_block: 100,
    }
    .to_canonical_bytes();

    let core = TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![InputRef {
            note_commit: [0xAA; 32],
        }],
        input_nullifiers: vec![Nullifier([0xBB; 32])],
        outputs: vec![dummy_output_note(42)],
        fee: 10,
        statement_commit: [0xCC; 32],
        auth: vec![InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![buyer_pk.clone(), merchant_pk], // wrong: should be buyer + operator
            signatures: vec![vec![0xAA], vec![0xBB]],
            policy_opening: Some(policy_opening),
            escrow_action: Some(0x01),
        }],
    };

    let wrong_tx = Transaction::TransferNote(TransferNoteTx { core });

    let result = node.submit_escrow_transfer_note(&proposal_hash, wrong_tx, 1000);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("signer set mismatch"),
                "should mention signer mismatch: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── Test 6: valid wallet-built escrow tx is accepted ──────────────────

#[test]
fn valid_escrow_tx_accepted() {
    let (mut node, proposal_hash, tx, _buyer_sk, _operator_sk, _merchant_sk, _merchant_h) =
        setup_escrow_with_tx();

    let result = node.submit_escrow_transfer_note(&proposal_hash, tx, 1000);
    assert!(
        result.is_ok(),
        "valid escrow tx should be accepted: {result:?}"
    );
}

// ── Test 7: submit still works after node reopen ──────────────────────

#[test]
fn escrow_submit_works_after_reopen() {
    use std::fs;

    let dir = std::env::temp_dir().join(format!(
        "privai-escrow-submit-reopen-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");

    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    let (buyer_sk, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (operator_sk, operator_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");

    let buyer_sk = buyer_sk.to_vec();
    let operator_sk = operator_sk.to_vec();

    let buyer_h = pk_hash(&buyer_pk);
    let merchant_h = [2; 32];
    let operator_h = pk_hash(&operator_pk);

    // Phase 1: build quorum state
    {
        let mut config = NodeConfig::example();
        config.data_dir = dir.to_str().unwrap().to_string();
        let mut node = PrivaiNode::<MemoryStore>::open(config, MemoryStore::new()).expect("node");

        let descriptor = EscrowFundingDescriptor {
            escrow_id,
            buyer_pk: buyer_h,
            merchant_pk: merchant_h,
            operator_pk: operator_h,
            amount: 1000,
            spend_policy_commit: SpendPolicy::Escrow2of3 {
                buyer_pk_hash: buyer_h,
                merchant_pk_hash: merchant_h,
                operator_pk_hash: operator_h,
                timeout_block: 100,
            }
            .commitment(),
            timeout_blocks: 100,
        };

        node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: ctx(1),
            descriptor,
            funding_tx_ref: [20; 32],
        }))
        .unwrap();

        node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: ctx(1),
            proposal: EscrowSpendProposal {
                proposal_hash,
                escrow_id,
                snapshot_hash: [40; 32],
                action: 0,
            },
        }))
        .unwrap();

        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: buyer_h,
            signature: vec![0xAA],
        }))
        .unwrap();

        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: operator_h,
            signature: vec![0xBB],
        }))
        .unwrap();
    }

    // Phase 2: reopen and submit
    {
        let mut config = NodeConfig::example();
        config.data_dir = dir.to_str().unwrap().to_string();
        let mut node = PrivaiNode::<MemoryStore>::open(config, MemoryStore::new()).expect("node");

        // Quorum must survive
        assert!(node.is_escrow_quorum_ready(&proposal_hash));

        // Seed the ledger with the escrow input note
        let escrow_policy = SpendPolicy::Escrow2of3 {
            buyer_pk_hash: buyer_h,
            merchant_pk_hash: merchant_h,
            operator_pk_hash: operator_h,
            timeout_block: 100,
        };
        let input_note = escrow_output_note(&escrow_policy, 0xAA);
        node.ledger_mut().snapshot_mut().notes.insert(
            input_note.note_commit,
            NoteRecord {
                note: input_note.clone(),
                created_in_block: Some(0),
                status: NoteStatus::Unspent,
            },
        );

        // Build tx
        let policy_opening = escrow_policy.to_canonical_bytes();

        let mut core = TxCore {
            version: PRIVAI_V0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef {
                note_commit: input_note.note_commit,
            }],
            input_nullifiers: vec![Nullifier([0xBB; 32])],
            outputs: vec![merchant_output_note(&merchant_h, 42)],
            fee: 10,
            statement_commit: [0xCC; 32],
            auth: vec![InputAuth {
                policy_tag: SpendPolicyTag::Escrow2of3 as u8,
                signer_pks: vec![buyer_pk.clone(), operator_pk.clone()],
                signatures: vec![],
                policy_opening: Some(policy_opening),
                escrow_action: Some(0x01),
            }],
        };

        let tx_for_signing = Transaction::TransferNote(TransferNoteTx { core: core.clone() });
        let signing_msg = tx_for_signing.tx_signing_hash();

        let buyer_sig = nxms_transport::crypto::falcon_sign_ct_prepared(&buyer_sk, &signing_msg)
            .expect("sign buyer");
        let operator_sig =
            nxms_transport::crypto::falcon_sign_ct_prepared(&operator_sk, &signing_msg)
                .expect("sign operator");

        core.auth[0].signatures = vec![buyer_sig, operator_sig];
        let tx = Transaction::TransferNote(TransferNoteTx { core });

        let result = node.submit_escrow_transfer_note(&proposal_hash, tx, 2000);
        assert!(
            result.is_ok(),
            "valid escrow tx after reopen should be accepted: {result:?}"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

// ── Test 8: tx with missing policy_opening is rejected ─────────────────

#[test]
fn missing_policy_opening_rejected() {
    let (mut node, proposal_hash, _tx, _buyer_sk, _operator_sk, _merchant_sk, _merchant_h) =
        setup_escrow_with_tx();

    let (_sk1, pk1) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (_sk2, pk2) = nxms_transport::crypto::falcon_keygen().expect("keygen");

    let core = TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![InputRef {
            note_commit: [0xAA; 32],
        }],
        input_nullifiers: vec![Nullifier([0xBB; 32])],
        outputs: vec![dummy_output_note(42)],
        fee: 10,
        statement_commit: [0xCC; 32],
        auth: vec![InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![pk1, pk2],
            signatures: vec![vec![0xAA], vec![0xBB]],
            policy_opening: None, // missing!
            escrow_action: Some(0x01),
        }],
    };

    let tx = Transaction::TransferNote(TransferNoteTx { core });

    let result = node.submit_escrow_transfer_note(&proposal_hash, tx, 1000);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("policy_opening"),
                "should mention policy_opening: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── Test 9: tx with missing escrow_action is rejected ──────────────────

#[test]
fn missing_escrow_action_rejected() {
    let (_sk1, pk1) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (_sk2, pk2) = nxms_transport::crypto::falcon_keygen().expect("keygen");

    let buyer_h = pk_hash(&pk1);
    let merchant_h = [2; 32];
    let operator_h = pk_hash(&pk2);

    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    let mut node = make_node();

    let descriptor = EscrowFundingDescriptor {
        escrow_id,
        buyer_pk: buyer_h,
        merchant_pk: merchant_h,
        operator_pk: operator_h,
        amount: 1000,
        spend_policy_commit: SpendPolicy::Escrow2of3 {
            buyer_pk_hash: buyer_h,
            merchant_pk_hash: merchant_h,
            operator_pk_hash: operator_h,
            timeout_block: 100,
        }
        .commitment(),
        timeout_blocks: 100,
    };

    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor,
        funding_tx_ref: [20; 32],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(1),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: [40; 32],
            action: 0,
        },
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: buyer_h,
        signature: vec![0xAA],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: operator_h,
        signature: vec![0xBB],
    }))
    .unwrap();

    let core = TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![InputRef {
            note_commit: [0xAA; 32],
        }],
        input_nullifiers: vec![Nullifier([0xBB; 32])],
        outputs: vec![dummy_output_note(42)],
        fee: 10,
        statement_commit: [0xCC; 32],
        auth: vec![InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![pk1, pk2],
            signatures: vec![vec![0xAA], vec![0xBB]],
            policy_opening: Some(
                SpendPolicy::Escrow2of3 {
                    buyer_pk_hash: buyer_h,
                    merchant_pk_hash: merchant_h,
                    operator_pk_hash: operator_h,
                    timeout_block: 100,
                }
                .to_canonical_bytes(),
            ),
            escrow_action: None, // missing!
        }],
    };

    let tx = Transaction::TransferNote(TransferNoteTx { core });

    let result = node.submit_escrow_transfer_note(&proposal_hash, tx, 1000);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("escrow_action"),
                "should mention escrow_action: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── Test 10: tx with bad signatures is rejected ───────────────────────

#[test]
fn bad_signatures_rejected() {
    let (_buyer_sk, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (_operator_sk, operator_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");

    let buyer_h = pk_hash(&buyer_pk);
    let merchant_h = [2; 32];
    let operator_h = pk_hash(&operator_pk);

    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    let mut node = make_node();

    let descriptor = EscrowFundingDescriptor {
        escrow_id,
        buyer_pk: buyer_h,
        merchant_pk: merchant_h,
        operator_pk: operator_h,
        amount: 1000,
        spend_policy_commit: SpendPolicy::Escrow2of3 {
            buyer_pk_hash: buyer_h,
            merchant_pk_hash: merchant_h,
            operator_pk_hash: operator_h,
            timeout_block: 100,
        }
        .commitment(),
        timeout_blocks: 100,
    };

    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor,
        funding_tx_ref: [20; 32],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(1),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: [40; 32],
            action: 0,
        },
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: buyer_h,
        signature: vec![0xAA],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: operator_h,
        signature: vec![0xBB],
    }))
    .unwrap();

    // Build tx with deliberately bad signatures
    let policy_opening = SpendPolicy::Escrow2of3 {
        buyer_pk_hash: buyer_h,
        merchant_pk_hash: merchant_h,
        operator_pk_hash: operator_h,
        timeout_block: 100,
    }
    .to_canonical_bytes();

    let core = TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![InputRef {
            note_commit: [0xAA; 32],
        }],
        input_nullifiers: vec![Nullifier([0xBB; 32])],
        outputs: vec![dummy_output_note(42)],
        fee: 10,
        statement_commit: [0xCC; 32],
        auth: vec![InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![buyer_pk, operator_pk],
            signatures: vec![vec![0xDE, 0xAD], vec![0xBE, 0xEF]], // bad sigs
            policy_opening: Some(policy_opening),
            escrow_action: Some(0x01),
        }],
    };

    let tx = Transaction::TransferNote(TransferNoteTx { core });

    let result = node.submit_escrow_transfer_note(&proposal_hash, tx, 1000);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(msg.contains("signature"), "should mention signature: {msg}");
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── Test 11: refund action works correctly ─────────────────────────────

#[test]
fn refund_action_accepted() {
    let (merchant_sk, merchant_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (operator_sk, operator_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");

    let merchant_sk = merchant_sk.to_vec();
    let operator_sk = operator_sk.to_vec();

    let buyer_h = [1; 32];
    let merchant_h = pk_hash(&merchant_pk);
    let operator_h = pk_hash(&operator_pk);

    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    let mut node = make_node();

    let descriptor = EscrowFundingDescriptor {
        escrow_id,
        buyer_pk: buyer_h,
        merchant_pk: merchant_h,
        operator_pk: operator_h,
        amount: 1000,
        spend_policy_commit: SpendPolicy::Escrow2of3 {
            buyer_pk_hash: buyer_h,
            merchant_pk_hash: merchant_h,
            operator_pk_hash: operator_h,
            timeout_block: 100,
        }
        .commitment(),
        timeout_blocks: 100,
    };

    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor,
        funding_tx_ref: [20; 32],
    }))
    .unwrap();

    // Proposal: action=1 (refund)
    node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(1),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: [40; 32],
            action: 1,
        },
    }))
    .unwrap();

    // Refund requires merchant + operator
    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: merchant_h,
        signature: vec![0xAA],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: operator_h,
        signature: vec![0xBB],
    }))
    .unwrap();

    // Build tx with refund action (0x02)
    let escrow_policy = SpendPolicy::Escrow2of3 {
        buyer_pk_hash: buyer_h,
        merchant_pk_hash: merchant_h,
        operator_pk_hash: operator_h,
        timeout_block: 100,
    };
    let policy_opening = escrow_policy.to_canonical_bytes();

    // Seed the ledger with the escrow input note
    let input_note = escrow_output_note(&escrow_policy, 0xAA);
    node.ledger_mut().snapshot_mut().notes.insert(
        input_note.note_commit,
        NoteRecord {
            note: input_note.clone(),
            created_in_block: Some(0),
            status: NoteStatus::Unspent,
        },
    );

    // Refund outputs go to buyer (Single policy)
    let output = buyer_output_note(&buyer_h, 42);

    let mut core = TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![InputRef {
            note_commit: input_note.note_commit,
        }],
        input_nullifiers: vec![Nullifier([0xBB; 32])],
        outputs: vec![output],
        fee: 10,
        statement_commit: [0xCC; 32],
        auth: vec![InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![merchant_pk.clone(), operator_pk.clone()],
            signatures: vec![],
            policy_opening: Some(policy_opening),
            escrow_action: Some(0x02), // Refund
        }],
    };

    let tx_for_signing = Transaction::TransferNote(TransferNoteTx { core: core.clone() });
    let signing_msg = tx_for_signing.tx_signing_hash();

    let merchant_sig = nxms_transport::crypto::falcon_sign_ct_prepared(&merchant_sk, &signing_msg)
        .expect("sign merchant");
    let operator_sig = nxms_transport::crypto::falcon_sign_ct_prepared(&operator_sk, &signing_msg)
        .expect("sign operator");

    core.auth[0].signatures = vec![merchant_sig, operator_sig];
    let tx = Transaction::TransferNote(TransferNoteTx { core });

    let result = node.submit_escrow_transfer_note(&proposal_hash, tx, 1000);
    assert!(
        result.is_ok(),
        "valid refund tx should be accepted: {result:?}"
    );
}

// ── Test 12: tx with no escrow auth entries is rejected ────────────────

#[test]
fn no_escrow_auth_entries_rejected() {
    let (mut node, proposal_hash, _tx, _buyer_sk, _operator_sk, _merchant_sk, _merchant_h) =
        setup_escrow_with_tx();

    let core = TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![InputRef {
            note_commit: [0xAA; 32],
        }],
        input_nullifiers: vec![Nullifier([0xBB; 32])],
        outputs: vec![dummy_output_note(42)],
        fee: 10,
        statement_commit: [0xCC; 32],
        auth: vec![InputAuth {
            policy_tag: SpendPolicyTag::Single as u8, // not escrow!
            signer_pks: vec![[1; 32].to_vec()],
            signatures: vec![vec![0xAA]],
            policy_opening: None,
            escrow_action: None,
        }],
    };

    let tx = Transaction::TransferNote(TransferNoteTx { core });

    let result = node.submit_escrow_transfer_note(&proposal_hash, tx, 1000);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("exactly 1"),
                "should mention exactly 1 escrow auth: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── Test 13: malformed policy_opening (undecodable) is rejected ─────────

#[test]
fn malformed_policy_opening_rejected() {
    let (mut node, proposal_hash, _tx, _buyer_sk, _operator_sk, _merchant_sk, _merchant_h) =
        setup_escrow_with_tx();

    let (_sk1, pk1) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (_sk2, pk2) = nxms_transport::crypto::falcon_keygen().expect("keygen");

    let core = TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![InputRef {
            note_commit: [0xAA; 32],
        }],
        input_nullifiers: vec![Nullifier([0xBB; 32])],
        outputs: vec![dummy_output_note(42)],
        fee: 10,
        statement_commit: [0xCC; 32],
        auth: vec![InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![pk1, pk2],
            signatures: vec![vec![0xAA], vec![0xBB]],
            policy_opening: Some(vec![0xFF, 0xFE, 0xFD]), // garbage bytes
            escrow_action: Some(0x01),
        }],
    };

    let tx = Transaction::TransferNote(TransferNoteTx { core });

    let result = node.submit_escrow_transfer_note(&proposal_hash, tx, 1000);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("decode") || msg.contains("policy_opening"),
                "should mention decode failure: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── Test 14: policy_opening with wrong policy type (Single) is rejected ─

#[test]
fn wrong_policy_type_opening_rejected() {
    let (mut node, proposal_hash, _tx, _buyer_sk, _operator_sk, _merchant_sk, _merchant_h) =
        setup_escrow_with_tx();

    let (_sk1, pk1) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (_sk2, pk2) = nxms_transport::crypto::falcon_keygen().expect("keygen");

    // Encode a Single policy instead of Escrow2of3
    let single_policy = SpendPolicy::Single {
        falcon_pk_hash: [0x42; 32],
    };
    let policy_opening = single_policy.to_canonical_bytes();

    let core = TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![InputRef {
            note_commit: [0xAA; 32],
        }],
        input_nullifiers: vec![Nullifier([0xBB; 32])],
        outputs: vec![dummy_output_note(42)],
        fee: 10,
        statement_commit: [0xCC; 32],
        auth: vec![InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![pk1, pk2],
            signatures: vec![vec![0xAA], vec![0xBB]],
            policy_opening: Some(policy_opening),
            escrow_action: Some(0x01),
        }],
    };

    let tx = Transaction::TransferNote(TransferNoteTx { core });

    let result = node.submit_escrow_transfer_note(&proposal_hash, tx, 1000);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("not Escrow2of3"),
                "should mention wrong policy type: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── Test 15: policy_opening Escrow2of3 with wrong buyer_pk_hash is rejected ─

#[test]
fn policy_mismatch_wrong_buyer_pk_hash_rejected() {
    let (buyer_sk, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (_merchant_sk, merchant_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");
    let (operator_sk, operator_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");

    let buyer_sk = buyer_sk.to_vec();
    let operator_sk = operator_sk.to_vec();

    let buyer_h = pk_hash(&buyer_pk);
    let merchant_h = pk_hash(&merchant_pk);
    let operator_h = pk_hash(&operator_pk);

    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    let mut node = make_node();

    let descriptor = EscrowFundingDescriptor {
        escrow_id,
        buyer_pk: buyer_h,
        merchant_pk: merchant_h,
        operator_pk: operator_h,
        amount: 1000,
        spend_policy_commit: SpendPolicy::Escrow2of3 {
            buyer_pk_hash: buyer_h,
            merchant_pk_hash: merchant_h,
            operator_pk_hash: operator_h,
            timeout_block: 100,
        }
        .commitment(),
        timeout_blocks: 100,
    };

    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor,
        funding_tx_ref: [20; 32],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(1),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: [40; 32],
            action: 0,
        },
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: buyer_h,
        signature: vec![0xAA],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: operator_h,
        signature: vec![0xBB],
    }))
    .unwrap();

    // Build tx with Escrow2of3 policy_opening that has a WRONG buyer_pk_hash
    let wrong_buyer_h = [0xFF; 32]; // does not match descriptor
    let policy_opening = SpendPolicy::Escrow2of3 {
        buyer_pk_hash: wrong_buyer_h,
        merchant_pk_hash: merchant_h,
        operator_pk_hash: operator_h,
        timeout_block: 100,
    }
    .to_canonical_bytes();

    let mut core = TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![InputRef {
            note_commit: [0xAA; 32],
        }],
        input_nullifiers: vec![Nullifier([0xBB; 32])],
        outputs: vec![dummy_output_note(42)],
        fee: 10,
        statement_commit: [0xCC; 32],
        auth: vec![InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![buyer_pk.clone(), operator_pk.clone()],
            signatures: vec![],
            policy_opening: Some(policy_opening),
            escrow_action: Some(0x01),
        }],
    };

    let tx_for_signing = Transaction::TransferNote(TransferNoteTx { core: core.clone() });
    let signing_msg = tx_for_signing.tx_signing_hash();
    let buyer_sig = nxms_transport::crypto::falcon_sign_ct_prepared(&buyer_sk, &signing_msg)
        .expect("sign buyer");
    let operator_sig = nxms_transport::crypto::falcon_sign_ct_prepared(&operator_sk, &signing_msg)
        .expect("sign operator");
    core.auth[0].signatures = vec![buyer_sig, operator_sig];

    let tx = Transaction::TransferNote(TransferNoteTx { core });

    let result = node.submit_escrow_transfer_note(&proposal_hash, tx, 1000);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("buyer_pk_hash mismatch"),
                "should mention buyer_pk_hash mismatch: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}
