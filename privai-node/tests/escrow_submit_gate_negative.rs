use privai_chain::{
    hash::domain_hash, CanonicalEncode, InputAuth, InputRef, LweCiphertext, Nullifier, OutputNote,
    RecipientBox, SettlementPhase, SettlementTx, SpendPolicy, SpendPolicyTag, Transaction,
    TransferNoteTx, TxCore, PRIVAI_V0, TX_TYPE_TRANSFER_NOTE,
};
use privai_ledger::{MemoryStore, NoteRecord, NoteStatus};
use privai_node::{NodeConfig, NodeError, PrivaiNode};
use privai_nxms::{
    ContextId, EscrowApprovalBody, EscrowFundedBody, EscrowFundingDescriptor, EscrowSpendProposal,
    EscrowSpendProposalBody, Hash32, PrivaiBody,
};

fn ctx(fill: u8) -> ContextId {
    [fill; 16]
}

fn make_node() -> PrivaiNode<MemoryStore> {
    let mut config = NodeConfig::example();
    config.data_dir = String::new();
    PrivaiNode::open(config, MemoryStore::new()).expect("node")
}

fn pk_hash(pk: &[u8]) -> Hash32 {
    domain_hash("privai:falcon-pk:v0", &[pk])
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

/// Full setup for release (action=0) escrow with quorum approvals from buyer + operator.
/// Returns (node, proposal_hash, buyer_sk, operator_sk, buyer_pk, operator_pk,
///          buyer_h, merchant_h, operator_h, escrow_policy).
fn setup_release_escrow() -> (
    PrivaiNode<MemoryStore>,
    Hash32,
    Vec<u8>,
    Vec<u8>,
    Vec<u8>,
    Vec<u8>,
    Hash32,
    Hash32,
    Hash32,
    SpendPolicy,
) {
    let (buyer_sk, buyer_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen buyer");
    let (_merchant_sk, merchant_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen merchant");
    let (operator_sk, operator_pk) =
        nxms_transport::crypto::falcon_keygen().expect("keygen operator");

    let buyer_sk = buyer_sk.to_vec();
    let operator_sk = operator_sk.to_vec();

    let buyer_h = pk_hash(&buyer_pk);
    let merchant_h = pk_hash(&merchant_pk);
    let operator_h = pk_hash(&operator_pk);

    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

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
        amount: 1000,
        spend_policy_commit: escrow_policy.commitment(),
        timeout_blocks: 100,
    };

    let mut node = make_node();

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

    // Quorum approvals: buyer + operator
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

    (
        node,
        proposal_hash,
        buyer_sk,
        operator_sk,
        buyer_pk,
        operator_pk,
        buyer_h,
        merchant_h,
        operator_h,
        escrow_policy,
    )
}

/// Build a valid TransferNoteTx for the release escrow (action=0x01).
/// Signs with buyer_sk and operator_sk.
fn build_release_transfer_tx(
    escrow_policy: &SpendPolicy,
    buyer_pk: &[u8],
    operator_pk: &[u8],
    buyer_sk: &[u8],
    operator_sk: &[u8],
    merchant_h: &Hash32,
) -> Transaction {
    let policy_opening = escrow_policy.to_canonical_bytes();

    let input_note = escrow_output_note(escrow_policy, 0xAA);

    let mut core = TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![InputRef {
            note_commit: input_note.note_commit,
        }],
        input_nullifiers: vec![Nullifier([0xBB; 32])],
        outputs: vec![merchant_output_note(merchant_h, 42)],
        fee: 10,
        statement_commit: [0xCC; 32],
        auth: vec![InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![buyer_pk.to_vec(), operator_pk.to_vec()],
            signatures: vec![],
            policy_opening: Some(policy_opening),
            escrow_action: Some(0x01), // Release
        }],
    };

    let tx_for_signing = Transaction::TransferNote(TransferNoteTx { core: core.clone() });
    let signing_msg = tx_for_signing.tx_signing_hash();

    let buyer_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(buyer_sk, &signing_msg).unwrap();
    let operator_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(operator_sk, &signing_msg).unwrap();

    core.auth[0].signatures = vec![buyer_sig, operator_sig];
    Transaction::TransferNote(TransferNoteTx { core })
}

// ── TEST 1: submit_without_quorum_fails ─────────────────────────────────

#[test]
fn submit_without_quorum_fails() {
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

    // Only 1 approval — quorum not met
    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [1; 32],
        signature: vec![0xAA],
    }))
    .unwrap();

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
    assert!(result.is_err(), "should fail without quorum");
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("quorum not met"),
                "expected 'quorum not met' in error, got: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── TEST 2: submit_with_wrong_proposal_hash_fails ───────────────────────

#[test]
fn submit_with_wrong_proposal_hash_fails() {
    let (
        mut node,
        _proposal_hash,
        buyer_sk,
        operator_sk,
        buyer_pk,
        operator_pk,
        _buyer_h,
        merchant_h,
        _operator_h,
        escrow_policy,
    ) = setup_release_escrow();

    // Use a random proposal hash that was never staged
    let fake_proposal = [0xFF; 32];

    let tx = build_release_transfer_tx(
        &escrow_policy,
        &buyer_pk,
        &operator_pk,
        &buyer_sk,
        &operator_sk,
        &merchant_h,
    );

    let result = node.submit_escrow_transfer_note(&fake_proposal, tx, 1000);
    assert!(result.is_err(), "should fail with wrong proposal hash");
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("proposal not found"),
                "expected 'proposal not found' in error, got: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── TEST 3: submit_non_transfer_note_fails ──────────────────────────────

#[test]
fn submit_non_transfer_note_fails() {
    let (
        mut node,
        proposal_hash,
        _buyer_sk,
        _operator_sk,
        _buyer_pk,
        _operator_pk,
        _buyer_h,
        _merchant_h,
        _operator_h,
        _escrow_policy,
    ) = setup_release_escrow();

    // Use a SettlementTx instead of TransferNote
    let settlement_tx = Transaction::Settlement(SettlementTx {
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
        phase: SettlementPhase::Open,
        payload_commit: [0; 32],
    });

    let result = node.submit_escrow_transfer_note(&proposal_hash, settlement_tx, 1000);
    assert!(result.is_err(), "should fail with non-TransferNote");
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("expected Transaction::TransferNote"),
                "expected 'expected Transaction::TransferNote' in error, got: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}

// ── TEST 4: submit_with_action_mismatch_fails ───────────────────────────

#[test]
fn submit_with_action_mismatch_fails() {
    let (
        mut node,
        proposal_hash,
        buyer_sk,
        operator_sk,
        buyer_pk,
        operator_pk,
        _buyer_h,
        merchant_h,
        _operator_h,
        escrow_policy,
    ) = setup_release_escrow();

    // Seed the ledger with an escrow input note so the tx structure is valid
    let input_note = escrow_output_note(&escrow_policy, 0xAA);
    node.ledger_mut().snapshot_mut().notes.insert(
        input_note.note_commit,
        NoteRecord {
            note: input_note.clone(),
            created_in_block: Some(0),
            status: NoteStatus::Unspent,
        },
    );

    // Build TransferNote with escrow_action = 0x02 (Refund), but proposal is release (action=0)
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
            escrow_action: Some(0x02), // Refund — but proposal is release (action=0)
        }],
    };

    let tx_for_signing = Transaction::TransferNote(TransferNoteTx { core: core.clone() });
    let signing_msg = tx_for_signing.tx_signing_hash();
    let buyer_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(&buyer_sk, &signing_msg).unwrap();
    let operator_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(&operator_sk, &signing_msg).unwrap();
    core.auth[0].signatures = vec![buyer_sig, operator_sig];

    let wrong_tx = Transaction::TransferNote(TransferNoteTx { core });

    let result = node.submit_escrow_transfer_note(&proposal_hash, wrong_tx, 1000);
    assert!(result.is_err(), "should fail with action mismatch");
    match result.unwrap_err() {
        NodeError::EscrowSubmit(msg) => {
            assert!(
                msg.contains("escrow action mismatch"),
                "expected 'escrow action mismatch' in error, got: {msg}"
            );
        }
        other => panic!("expected EscrowSubmit, got: {other:?}"),
    }
}
