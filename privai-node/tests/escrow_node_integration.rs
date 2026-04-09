use privai_ledger::MemoryStore;
use privai_node::{EscrowIngestOutcome, NodeConfig, NodeError, PrivaiNode};
use privai_nxms::{
    BundleValidationError, ContextId, EscrowApprovalBody, EscrowApprovalBundle, EscrowFundedBody,
    EscrowFundingDescriptor, EscrowSpendProposal, EscrowSpendProposalBody, Hash32, PrivaiBody,
    TX_SIGNING_HASH_STAGE_A,
};

fn make_node() -> PrivaiNode<MemoryStore> {
    let mut config = NodeConfig::example();
    config.data_dir = String::new(); // no persistence in basic tests
    PrivaiNode::open(config, MemoryStore::new()).expect("node")
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
            },
        }));

    assert!(result.is_err());
}

// ── Test 7: full lifecycle through node API ────────────────────────────

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

// ── Test 8: staged approvals returned by node are stable in ordering ───

#[test]
fn node_staged_approvals_ordering_is_stable() {
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
        },
    }))
    .unwrap();

    // Ingest approvals in REVERSE signer_pk order (merchant first, buyer second)
    let merchant_approval = EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [0x20; 32], // higher than buyer
        signature: vec![2, 3, 4],
    };
    let buyer_approval = EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [0x10; 32], // lower than merchant
        signature: vec![1, 2, 3],
    };

    node.handle_privai_body(PrivaiBody::EscrowApproval(merchant_approval))
        .unwrap();
    node.handle_privai_body(PrivaiBody::EscrowApproval(buyer_approval))
        .unwrap();

    let ready = node.get_escrow_ready_approvals(&proposal_hash).unwrap();
    assert_eq!(ready.len(), 2);
    // Must be sorted by signer_pk regardless of ingestion order
    assert!(ready[0].signer_pk < ready[1].signer_pk);
    assert_eq!(ready[0].signer_pk, [0x10; 32]); // buyer first
    assert_eq!(ready[1].signer_pk, [0x20; 32]); // merchant second

    // Call again — must return identical result (stable)
    let ready2 = node.get_escrow_ready_approvals(&proposal_hash).unwrap();
    assert_eq!(ready, ready2);
}

// ── Test 9: node rejects duplicate signer approval ──────────────────────

#[test]
fn node_rejects_duplicate_signer_in_staging() {
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
        },
    }))
    .unwrap();

    // First approval from signer [1; 32]
    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [1; 32],
        signature: vec![0xAA],
    }))
    .unwrap();

    // Duplicate from same signer — must be rejected
    let result = node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [1; 32],
        signature: vec![0xBB], // different signature, same signer
    }));
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        NodeError::EscrowIngest(privai_node::EscrowStageError::DuplicateApproval)
    ));
}

// ── Test 10: bundle built from node approvals passes validation ─────────

#[test]
fn bundle_from_node_approvals_is_valid() {
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
        },
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [1; 32],
        signature: vec![0xAA],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [2; 32],
        signature: vec![0xBB],
    }))
    .unwrap();

    let ready = node.get_escrow_ready_approvals(&proposal_hash).unwrap();

    // Build bundle from node-exported approvals
    let bundle = EscrowApprovalBundle::from_approvals_sorted(proposal_hash, &ready).unwrap();
    assert!(bundle.validate().is_ok());

    // Stage A sentinel: tx_signing_hash must be zeroed
    assert!(bundle.is_stage_a());
    assert_eq!(bundle.tx_signing_hash, TX_SIGNING_HASH_STAGE_A);

    // Signers in stable order
    assert_eq!(bundle.signer_pks[0], [1; 32]);
    assert_eq!(bundle.signer_pks[1], [2; 32]);
}

// ── Test 11: bundle rejects signer/signature count mismatch ─────────────

#[test]
fn bundle_rejects_signer_signature_mismatch() {
    let bundle = EscrowApprovalBundle {
        proposal_hash: [30; 32],
        tx_signing_hash: TX_SIGNING_HASH_STAGE_A,
        signer_pks: vec![[1; 32], [2; 32], [3; 32]],
        signatures: vec![vec![0xAA], vec![0xBB]], // missing one signature
    };
    assert!(matches!(
        bundle.validate(),
        Err(BundleValidationError::SignerSignatureCountMismatch {
            signers: 3,
            signatures: 2
        })
    ));
}

// ── Test 12: bundle rejects duplicate signers at validate time ──────────

#[test]
fn bundle_rejects_duplicate_signers_at_validate() {
    let bundle = EscrowApprovalBundle {
        proposal_hash: [30; 32],
        tx_signing_hash: TX_SIGNING_HASH_STAGE_A,
        signer_pks: vec![[1; 32], [1; 32]], // duplicate
        signatures: vec![vec![0xAA], vec![0xBB]],
    };
    assert!(matches!(
        bundle.validate(),
        Err(BundleValidationError::DuplicateSigner(_))
    ));
}

// ── Test 13: approval semantics doc — bundle is authorization material ──

#[test]
fn bundle_stage_a_is_not_final_auth() {
    // Stage A bundle: tx_signing_hash is unset
    let approvals = vec![
        EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash: [30; 32],
            signer_pk: [1; 32],
            signature: vec![0xAA],
        },
        EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash: [30; 32],
            signer_pk: [2; 32],
            signature: vec![0xBB],
        },
    ];

    let bundle = EscrowApprovalBundle::from_approvals_sorted([30; 32], &approvals).unwrap();

    // The bundle is authorization material — not final ledger-ready auth
    assert!(bundle.is_stage_a());
    assert_eq!(bundle.tx_signing_hash, TX_SIGNING_HASH_STAGE_A);
    // Final tx_signing_hash is computed in Stage B after canonical tx body assembly
    assert!(bundle.validate().is_ok());
}

// ── Test 14: deterministic ordering is stable across multiple builds ────

#[test]
fn bundle_ordering_stable_across_multiple_builds() {
    let approvals_ascending = vec![
        EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash: [30; 32],
            signer_pk: [1; 32],
            signature: vec![1],
        },
        EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash: [30; 32],
            signer_pk: [2; 32],
            signature: vec![2],
        },
    ];

    let approvals_descending = vec![
        approvals_ascending[1].clone(),
        approvals_ascending[0].clone(),
    ];

    let bundle_a =
        EscrowApprovalBundle::from_approvals_sorted([30; 32], &approvals_ascending).unwrap();
    let bundle_b =
        EscrowApprovalBundle::from_approvals_sorted([30; 32], &approvals_descending).unwrap();

    // Both must produce identical bundles
    assert_eq!(bundle_a, bundle_b);
}

// ── Test 15: build bundle before quorum returns controlled error ────────

#[test]
fn bundle_build_before_quorum_returns_error() {
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

    // Must return controlled error, not panic
    let result = node.build_escrow_approval_bundle(&proposal_hash);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowBundle(msg) => {
            assert!(msg.contains("quorum"), "error should mention quorum: {msg}");
        }
        other => panic!("expected EscrowBundle error, got: {other:?}"),
    }
}

// ── Test 16: build bundle after quorum returns valid bundle ─────────────

#[test]
fn bundle_build_after_quorum_returns_valid_bundle() {
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
        },
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [1; 32],
        signature: vec![0xAA],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [2; 32],
        signature: vec![0xBB],
    }))
    .unwrap();

    let bundle = node
        .build_escrow_approval_bundle(&proposal_hash)
        .expect("bundle should build after quorum");
    assert!(bundle.validate().is_ok());
    assert_eq!(bundle.proposal_hash, proposal_hash);
    assert_eq!(bundle.signer_pks.len(), 2);
    assert_eq!(bundle.signatures.len(), 2);
}

// ── Test 17: bundle uses TX_SIGNING_HASH_STAGE_A ───────────────────────

#[test]
fn bundle_uses_tx_signing_hash_stage_a() {
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
        },
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [1; 32],
        signature: vec![0xAA],
    }))
    .unwrap();

    node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
        session_context: ctx(1),
        proposal_hash,
        signer_pk: [2; 32],
        signature: vec![0xBB],
    }))
    .unwrap();

    let bundle = node
        .build_escrow_approval_bundle(&proposal_hash)
        .expect("bundle should build");

    // Stage A contract: tx_signing_hash must be the sentinel
    assert_eq!(bundle.tx_signing_hash, TX_SIGNING_HASH_STAGE_A);
    assert!(bundle.is_stage_a());
}

// ── Test 18: bundle ordering is stable regardless approval ingest order ─

#[test]
fn bundle_ordering_stable_regardless_ingest_order() {
    // Build two nodes with same data but different ingest order
    let mut node_a = make_node();
    let mut node_b = make_node();
    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    for node in [&mut node_a, &mut node_b] {
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
            },
        }))
        .unwrap();
    }

    // Node A: ingest buyer first, merchant second
    node_a
        .handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: [1; 32],
            signature: vec![0xAA],
        }))
        .unwrap();
    node_a
        .handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: [2; 32],
            signature: vec![0xBB],
        }))
        .unwrap();

    // Node B: ingest merchant first, buyer second (reversed order)
    node_b
        .handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: [2; 32],
            signature: vec![0xBB],
        }))
        .unwrap();
    node_b
        .handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: [1; 32],
            signature: vec![0xAA],
        }))
        .unwrap();

    let bundle_a = node_a
        .build_escrow_approval_bundle(&proposal_hash)
        .expect("bundle_a");
    let bundle_b = node_b
        .build_escrow_approval_bundle(&proposal_hash)
        .expect("bundle_b");

    // Bundles must be identical regardless of ingest order
    assert_eq!(bundle_a, bundle_b);

    // And must be sorted by signer_pk
    assert!(bundle_a.signer_pks[0] < bundle_a.signer_pks[1]);
}

// ── Test 19: bundle export still works after node reopen ────────────────

use std::fs;
use std::path::Path;

fn temp_dir(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "privai-bundle-test-{}-{}",
        test_name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn make_node_with_dir(dir: &Path) -> PrivaiNode<MemoryStore> {
    let mut config = NodeConfig::example();
    config.data_dir = dir.to_str().unwrap().to_string();
    PrivaiNode::open(config, MemoryStore::new()).expect("node")
}

#[test]
fn bundle_export_works_after_node_reopen() {
    let dir = temp_dir("bundle_after_reopen");
    let escrow_id = [10; 32];
    let proposal_hash = [30; 32];

    // Phase 1: build full quorum state
    {
        let mut node = make_node_with_dir(&dir);

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
            },
        }))
        .unwrap();

        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: [1; 32],
            signature: vec![0xAA],
        }))
        .unwrap();

        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: [2; 32],
            signature: vec![0xBB],
        }))
        .unwrap();

        // Build bundle before close
        let bundle_before = node
            .build_escrow_approval_bundle(&proposal_hash)
            .expect("bundle before reopen");
        assert!(bundle_before.validate().is_ok());
    }

    // Phase 2: reopen node and build bundle again
    {
        let node = make_node_with_dir(&dir);

        // Quorum must survive
        assert!(node.is_escrow_quorum_ready(&proposal_hash));

        let bundle_after = node
            .build_escrow_approval_bundle(&proposal_hash)
            .expect("bundle after reopen");
        assert!(bundle_after.validate().is_ok());
        assert_eq!(bundle_after.tx_signing_hash, TX_SIGNING_HASH_STAGE_A);
        assert!(bundle_after.is_stage_a());
        assert_eq!(bundle_after.signer_pks.len(), 2);
    }
}

// ── Test 20: build bundle for nonexistent proposal returns error ────────

#[test]
fn bundle_build_nonexistent_proposal_returns_error() {
    let node = make_node();
    let fake_proposal = [0xFF; 32];

    let result = node.build_escrow_approval_bundle(&fake_proposal);
    assert!(result.is_err());
    match result.unwrap_err() {
        NodeError::EscrowBundle(msg) => {
            assert!(
                msg.contains("not found"),
                "error should mention proposal not found: {msg}"
            );
        }
        other => panic!("expected EscrowBundle error, got: {other:?}"),
    }
}
