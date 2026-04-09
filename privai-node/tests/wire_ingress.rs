use nxms_transport::wire::NxmsPayloadV2;
use privai_ledger::MemoryStore;
use privai_node::{EscrowIngestOutcome, NodeConfig, NodeError, PrivaiNode};
use privai_nxms::{
    ContextId, EscrowApprovalBody, EscrowFundedBody, EscrowFundingDescriptor, EscrowSpendProposal,
    EscrowSpendProposalBody, Hash32, PrivaiBody, ProtocolError, PRIVAI_APP_PROTO_V1,
};

fn make_node() -> PrivaiNode<MemoryStore> {
    let mut config = NodeConfig::example();
    config.data_dir = String::new(); // no persistence in basic tests
    PrivaiNode::open(config, MemoryStore::new()).expect("node")
}

fn h32(fill: u8) -> Hash32 {
    [fill; 32]
}

fn ctx(fill: u8) -> ContextId {
    [fill; 16]
}

fn dummy_descriptor(escrow_id: Hash32) -> EscrowFundingDescriptor {
    EscrowFundingDescriptor {
        escrow_id,
        buyer_pk: h32(1),
        merchant_pk: h32(2),
        operator_pk: h32(3),
        amount: 1000,
        spend_policy_commit: h32(4),
        timeout_blocks: 100,
    }
}

fn funded_body(fill: u8) -> PrivaiBody {
    PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(fill),
        descriptor: dummy_descriptor(h32(fill)),
        funding_tx_ref: h32(fill.wrapping_add(10)),
    })
}

fn proposal_body(session: u8, proposal_hash: Hash32, escrow_id: Hash32, action: u8) -> PrivaiBody {
    PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(session),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: h32(40),
            action,
        },
    })
}

fn approval_body(session: u8, proposal_hash: Hash32, signer: u8) -> PrivaiBody {
    PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(session),
        proposal_hash,
        signer_pk: h32(signer),
        signature: vec![signer, signer.wrapping_add(1)],
    })
}

fn wire(body: &PrivaiBody, seq: u64) -> NxmsPayloadV2 {
    body.to_payload(ctx(1), "sender", "node", seq)
        .expect("to_payload")
}

// ── 1. Node ingests EscrowFunded payload ───────────────────────────────

#[test]
fn wire_ingest_funded() {
    let mut node = make_node();
    let body = funded_body(10);
    let payload = wire(&body, 1);

    let outcome = node.handle_nxms_payload(&payload).expect("ingest");
    assert_eq!(outcome, EscrowIngestOutcome::FundedStored);

    let staged = node.get_staged_escrow(&h32(10)).expect("staged");
    assert_eq!(staged.escrow_id, h32(10));
}

// ── 2. Node ingests EscrowSpendProposal payload ───────────────────────

#[test]
fn wire_ingest_proposal() {
    let mut node = make_node();
    let escrow_id = h32(10);

    // Fund first via payload
    node.handle_nxms_payload(&wire(&funded_body(10), 1))
        .unwrap();

    let body = proposal_body(10, h32(30), escrow_id, 0);
    let payload = wire(&body, 2);

    let outcome = node.handle_nxms_payload(&payload).expect("ingest");
    assert_eq!(outcome, EscrowIngestOutcome::ProposalStored);

    let staged = node.get_staged_proposal(&h32(30)).expect("staged");
    assert_eq!(staged.proposal.escrow_id, escrow_id);
}

// ── 3. Node ingests EscrowApproval payload ─────────────────────────────

#[test]
fn wire_ingest_approval() {
    let mut node = make_node();
    let escrow_id = h32(10);
    let proposal_hash = h32(30);

    node.handle_nxms_payload(&wire(&funded_body(10), 1))
        .unwrap();
    node.handle_nxms_payload(&wire(&proposal_body(10, proposal_hash, escrow_id, 0), 2))
        .unwrap();

    let body = approval_body(10, proposal_hash, 1);
    let payload = wire(&body, 3);

    let outcome = node.handle_nxms_payload(&payload).expect("ingest");
    assert_eq!(outcome, EscrowIngestOutcome::ApprovalStored);
}

// ── 4. Wrong app_proto is rejected ─────────────────────────────────────

#[test]
fn wire_rejects_wrong_app_proto() {
    let mut node = make_node();
    let body = funded_body(10);
    let mut payload = wire(&body, 1);
    payload.app_proto = "ESCROW/1".to_string();

    let result = node.handle_nxms_payload(&payload);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::Protocol(ProtocolError::UnexpectedAppProto(proto)) => {
            assert_eq!(proto, "ESCROW/1");
        }
        other => panic!("expected Protocol::UnexpectedAppProto, got: {other}"),
    }
}

// ── 5. Mismatched msg_type is rejected ─────────────────────────────────

#[test]
fn wire_rejects_msg_type_mismatch() {
    let mut node = make_node();
    let body = funded_body(10);
    let mut payload = wire(&body, 1);
    payload.msg_type = "escrow_spend_proposal".to_string(); // wrong for body kind

    let result = node.handle_nxms_payload(&payload);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::Protocol(ProtocolError::MsgTypeMismatch { payload, expected }) => {
            assert_eq!(payload, "escrow_spend_proposal");
            assert_eq!(expected, "escrow_funded");
        }
        other => panic!("expected Protocol::MsgTypeMismatch, got: {other}"),
    }
}

// ── 6. Non-escrow PrivaiBody payload is ignored, no corruption ─────────

#[test]
fn wire_non_escrow_body_ignored_no_corruption() {
    let mut node = make_node();

    // Fund an escrow first
    node.handle_nxms_payload(&wire(&funded_body(10), 1))
        .unwrap();

    // Send a non-escrow body via payload
    let bundle = PrivaiBody::BundleOffer(privai_nxms::BundleOfferBody {
        bundle_id: ctx(99),
        bundle_commit: h32(99),
        relay_hint: None,
        expires_at_block: 42,
    });
    let payload = wire(&bundle, 2);

    let outcome = node
        .handle_nxms_payload(&payload)
        .expect("should not error");
    assert_eq!(outcome, EscrowIngestOutcome::Ignored);

    // Escrow state intact
    assert!(node.get_staged_escrow(&h32(10)).is_some());
}

// ── 7. Malformed JSON payload gives controlled error, no panic ──────────

#[test]
fn wire_malformed_json_gives_controlled_error() {
    let mut node = make_node();
    let payload = NxmsPayloadV2 {
        app_proto: PRIVAI_APP_PROTO_V1.to_string(),
        msg_type: "escrow_funded".to_string(),
        context_id_hex: hex::encode(ctx(1)),
        from: "sender".to_string(),
        to: "node".to_string(),
        seq: 1,
        data: "{broken json}".to_string(),
    };

    let result = node.handle_nxms_payload(&payload);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::Protocol(ProtocolError::Json(_)) => { /* expected */ }
        other => panic!("expected Protocol::Json, got: {other}"),
    }
}

// ── 8. Full lifecycle through wire ingress: fund -> proposal -> 2 approvals -> quorum ──

#[test]
fn wire_full_escrow_lifecycle() {
    let mut node = make_node();
    let escrow_id = h32(42);
    let proposal_hash = h32(77);

    // 1. Fund
    let funded = PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(5),
        descriptor: EscrowFundingDescriptor {
            escrow_id,
            buyer_pk: h32(1),
            merchant_pk: h32(2),
            operator_pk: h32(3),
            amount: 5000,
            spend_policy_commit: h32(4),
            timeout_blocks: 200,
        },
        funding_tx_ref: h32(100),
    });
    assert_eq!(
        node.handle_nxms_payload(&wire(&funded, 1)).unwrap(),
        EscrowIngestOutcome::FundedStored
    );

    // 2. Proposal
    let proposal = PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(5),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: h32(200),
            action: 0, // release
        },
    });
    assert_eq!(
        node.handle_nxms_payload(&wire(&proposal, 2)).unwrap(),
        EscrowIngestOutcome::ProposalStored
    );

    // Not ready yet
    assert!(!node.is_escrow_quorum_ready(&proposal_hash));

    // 3. Approval 1 (buyer)
    let approval1 = approval_body(5, proposal_hash, 1);
    assert_eq!(
        node.handle_nxms_payload(&wire(&approval1, 3)).unwrap(),
        EscrowIngestOutcome::ApprovalStored
    );
    assert!(!node.is_escrow_quorum_ready(&proposal_hash));

    // 4. Approval 2 (operator)
    let approval2 = approval_body(5, proposal_hash, 3);
    assert_eq!(
        node.handle_nxms_payload(&wire(&approval2, 4)).unwrap(),
        EscrowIngestOutcome::ApprovalStored
    );

    // 5. Quorum ready
    assert!(node.is_escrow_quorum_ready(&proposal_hash));
    let ready = node.get_escrow_ready_approvals(&proposal_hash).unwrap();
    assert_eq!(ready.len(), 2);

    // 6. Query state
    let esc = node.get_staged_escrow(&escrow_id).unwrap();
    assert_eq!(esc.session_context, ctx(5));
    assert_eq!(esc.funding_tx_ref, h32(100));

    let prop = node.get_staged_proposal(&proposal_hash).unwrap();
    assert_eq!(prop.proposal.escrow_id, escrow_id);
    assert_eq!(prop.proposal.action, 0);
    assert_eq!(prop.approvals.len(), 2);
}

// ── 9. Empty data field rejected ───────────────────────────────────────

#[test]
fn wire_empty_data_rejected() {
    let mut node = make_node();
    let payload = NxmsPayloadV2 {
        app_proto: PRIVAI_APP_PROTO_V1.to_string(),
        msg_type: "escrow_funded".to_string(),
        context_id_hex: hex::encode(ctx(1)),
        from: "sender".to_string(),
        to: "node".to_string(),
        seq: 1,
        data: "".to_string(),
    };

    let result = node.handle_nxms_payload(&payload);
    assert!(result.is_err());
    // This will be a json parse error (empty string is not valid JSON)
    match result.unwrap_err() {
        NodeError::Protocol(ProtocolError::Json(_)) => { /* expected */ }
        other => panic!("expected Protocol::Json on empty data, got: {other}"),
    }
}

// ── 10. Wire ingress preserves existing handle_privai_body behavior ────

#[test]
fn wire_ingest_matches_direct_body_ingest() {
    let mut node_wire = make_node();
    let mut node_direct = make_node();

    // Same body, same data
    let body = funded_body(10);
    let payload = wire(&body, 1);

    let wire_outcome = node_wire.handle_nxms_payload(&payload).unwrap();
    let direct_outcome = node_direct.handle_privai_body(body).unwrap();

    assert_eq!(wire_outcome, direct_outcome);
    assert_eq!(wire_outcome, EscrowIngestOutcome::FundedStored);

    // Both nodes have identical state
    assert!(node_wire.get_staged_escrow(&h32(10)).is_some());
    assert!(node_direct.get_staged_escrow(&h32(10)).is_some());
}

// ── 11. Duplicate funded via wire ingress handled correctly ─────────────

#[test]
fn wire_duplicate_funded_is_idempotent() {
    let mut node = make_node();
    let body = funded_body(10);
    let payload = wire(&body, 1);

    node.handle_nxms_payload(&payload).unwrap();
    // Same body, same payload — idempotent
    let outcome = node.handle_nxms_payload(&payload).unwrap();
    assert_eq!(outcome, EscrowIngestOutcome::FundedStored);
}

// ── 12. Escrow wire error not panic on arbitrary garbage ────────────────

#[test]
fn wire_no_panic_on_garbage_payload() {
    let mut node = make_node();
    let payload = NxmsPayloadV2 {
        app_proto: PRIVAI_APP_PROTO_V1.to_string(),
        msg_type: "escrow_funded".to_string(),
        context_id_hex: hex::encode(ctx(1)),
        from: "sender".to_string(),
        to: "node".to_string(),
        seq: 1,
        data: "\"just a string, not an object\"".to_string(),
    };

    // Must not panic — should return a protocol error
    let result = node.handle_nxms_payload(&payload);
    assert!(result.is_err());
}
