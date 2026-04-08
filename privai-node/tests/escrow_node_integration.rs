use privai_ledger::MemoryStore;
use privai_node::{EscrowIngestOutcome, NodeConfig, NodeError, PrivaiNode};
use privai_nxms::{
    ContextId, EscrowApprovalBody, EscrowFundedBody, EscrowFundingDescriptor, EscrowSpendProposal,
    EscrowSpendProposalBody, Hash32, PrivaiBody,
};

fn make_node() -> PrivaiNode<MemoryStore> {
    PrivaiNode::open(NodeConfig::example(), MemoryStore::new()).expect("node")
}

fn dummy_descriptor(escrow_id: Hash32) -> EscrowFundingDescriptor {
    EscrowFundingDescriptor {
        escrow_id,
        buyer_pk: [1; 32],
        merchant_pk: [2; 32],
        operator_pk: [3; 32],
        amount: 1000,
        spend_policy_commit: [4; 32],
        timeout_blocks: 100,
    }
}

fn ctx(fill: u8) -> ContextId {
    [fill; 16]
}

// ── Test 1: node ingests EscrowFunded ──────────────────────────────────

#[test]
fn node_ingests_escrow_funded() {
    let mut node = make_node();
    let escrow_id = [10; 32];

    let body = PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor: dummy_descriptor(escrow_id),
        funding_tx_ref: [20; 32],
    });

    let outcome = node.handle_privai_body(body).expect("ingest");
    assert_eq!(outcome, EscrowIngestOutcome::FundedStored);

    // Node exposes staged escrow via query API
    let staged = node.get_staged_escrow(&escrow_id).expect("staged escrow");
    assert_eq!(staged.escrow_id, escrow_id);
    assert_eq!(staged.funding_tx_ref, [20; 32]);
}

// ── Test 2: node ingests EscrowSpendProposal ───────────────────────────

#[test]
fn node_ingests_escrow_spend_proposal() {
    let mut node = make_node();
    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    // Fund first
    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor: dummy_descriptor(escrow_id),
        funding_tx_ref: [20; 32],
    }))
    .expect("fund");

    let outcome = node
        .handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: ctx(1),
            proposal: EscrowSpendProposal {
                proposal_hash,
                escrow_id,
                snapshot_hash: [40; 32],
                action: 0,
                tx_signing_hash: [50; 32],
            },
        }))
        .expect("proposal");

    assert_eq!(outcome, EscrowIngestOutcome::ProposalStored);

    // Node exposes staged proposal
    let staged = node
        .get_staged_proposal(&proposal_hash)
        .expect("staged proposal");
    assert_eq!(staged.proposal.escrow_id, escrow_id);
    assert!(staged.approvals.is_empty());
}

// ── Test 3: node ingests 2 approvals and reports quorum ready ──────────

#[test]
fn node_reports_quorum_ready_after_approvals() {
    let mut node = make_node();
    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    // Fund
    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor: dummy_descriptor(escrow_id),
        funding_tx_ref: [20; 32],
    }))
    .unwrap();

    // Proposal
    node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
        session_context: ctx(1),
        proposal: EscrowSpendProposal {
            proposal_hash,
            escrow_id,
            snapshot_hash: [40; 32],
            action: 0,
            tx_signing_hash: [50; 32],
        },
    }))
    .unwrap();

    // Not ready yet
    assert!(!node.is_escrow_quorum_ready(&proposal_hash));
    assert!(node.get_escrow_ready_approvals(&proposal_hash).is_none());

    // Approval 1 (buyer)
    let outcome = node
        .handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: [1; 32],
            signature: vec![1, 2, 3],
        }))
        .unwrap();
    assert_eq!(outcome, EscrowIngestOutcome::ApprovalStored);

    // Still not ready (only 1 of 2)
    assert!(!node.is_escrow_quorum_ready(&proposal_hash));

    // Approval 2 (merchant)
    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [2; 32],
        signature: vec![4, 5, 6],
    }))
    .unwrap();

    // Quorum ready (2 approvals)
    assert!(node.is_escrow_quorum_ready(&proposal_hash));

    let ready = node
        .get_escrow_ready_approvals(&proposal_hash)
        .expect("ready");
    assert_eq!(ready.len(), 2);

    // Approvals sorted by signer_pk
    assert!(ready[0].signer_pk <= ready[1].signer_pk);
}

// ── Test 4: duplicate approval rejected through node API ───────────────

#[test]
fn node_rejects_duplicate_approval() {
    let mut node = make_node();
    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor: dummy_descriptor(escrow_id),
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
            tx_signing_hash: [50; 32],
        },
    }))
    .unwrap();

    let approval = PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [1; 32],
        signature: vec![1, 2, 3],
    });

    node.handle_privai_body(approval.clone()).unwrap();

    // Duplicate
    let result = node.handle_privai_body(approval);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, NodeError::EscrowIngest(_)));
}

// ── Test 5: non-escrow PrivaiBody returns Ignored, no corruption ───────

#[test]
fn non_escrow_body_ignored_no_corruption() {
    let mut node = make_node();
    let escrow_id = [10; 32];

    // Fund an escrow first
    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor: dummy_descriptor(escrow_id),
        funding_tx_ref: [20; 32],
    }))
    .unwrap();

    // Send a non-escrow body
    let outcome = node
        .handle_privai_body(PrivaiBody::BundleOffer(privai_nxms::BundleOfferBody {
            bundle_id: ctx(99),
            bundle_commit: [99; 32],
            relay_hint: None,
            expires_at_block: 0,
        }))
        .expect("non-escrow should not error");

    assert_eq!(outcome, EscrowIngestOutcome::Ignored);

    // Escrow state is not corrupted
    let staged = node
        .get_staged_escrow(&escrow_id)
        .expect("escrow should still exist");
    assert_eq!(staged.escrow_id, escrow_id);
}

// ── Test 6: proposal before funding returns error ──────────────────────

#[test]
fn proposal_before_funding_returns_error() {
    let mut node = make_node();

    let result =
        node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: ctx(1),
            proposal: EscrowSpendProposal {
                proposal_hash: [30; 32],
                escrow_id: [10; 32], // not funded
                snapshot_hash: [40; 32],
                action: 0,
                tx_signing_hash: [50; 32],
            },
        }));

    assert!(result.is_err());
}

// ── Test 7: proposal_hash == tx_signing_hash rejected ──────────────────

#[test]
fn proposal_hash_equals_tx_signing_hash_rejected() {
    let mut node = make_node();
    let escrow_id = [10; 32];

    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor: dummy_descriptor(escrow_id),
        funding_tx_ref: [20; 32],
    }))
    .unwrap();

    let bad_hash = [30; 32];
    let result =
        node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: ctx(1),
            proposal: EscrowSpendProposal {
                proposal_hash: bad_hash,
                escrow_id,
                snapshot_hash: [40; 32],
                action: 0,
                tx_signing_hash: bad_hash, // SAME
            },
        }));

    assert!(result.is_err());
}

// ── Test 8: full lifecycle through node API ────────────────────────────

#[test]
fn full_escrow_lifecycle_through_node_api() {
    let mut node = make_node();
    let escrow_id = [42; 32];
    let proposal_hash = [77; 32];

    // 1. Fund
    assert_eq!(
        node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: ctx(5),
            descriptor: dummy_descriptor(escrow_id),
            funding_tx_ref: [100; 32],
        }))
        .unwrap(),
        EscrowIngestOutcome::FundedStored
    );

    // 2. Proposal
    assert_eq!(
        node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: ctx(5),
            proposal: EscrowSpendProposal {
                proposal_hash,
                escrow_id,
                snapshot_hash: [200; 32],
                action: 1, // refund
                tx_signing_hash: [250; 32],
            },
        },))
            .unwrap(),
        EscrowIngestOutcome::ProposalStored
    );

    // 3. Approval from buyer
    assert_eq!(
        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(5),
            proposal_hash,
            signer_pk: [1; 32],
            signature: vec![0xAA],
        }))
        .unwrap(),
        EscrowIngestOutcome::ApprovalStored
    );

    assert!(!node.is_escrow_quorum_ready(&proposal_hash));

    // 4. Approval from operator
    assert_eq!(
        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(5),
            proposal_hash,
            signer_pk: [3; 32],
            signature: vec![0xBB],
        }))
        .unwrap(),
        EscrowIngestOutcome::ApprovalStored
    );

    // 5. Quorum ready
    assert!(node.is_escrow_quorum_ready(&proposal_hash));

    let ready = node.get_escrow_ready_approvals(&proposal_hash).unwrap();
    assert_eq!(ready.len(), 2);

    // 6. Query staged escrow
    let esc = node.get_staged_escrow(&escrow_id).unwrap();
    assert_eq!(esc.session_context, ctx(5));
    assert_eq!(esc.funding_tx_ref, [100; 32]);

    // 7. Query staged proposal
    let prop = node.get_staged_proposal(&proposal_hash).unwrap();
    assert_eq!(prop.proposal.action, 1); // refund
    assert_eq!(prop.approvals.len(), 2);
}
