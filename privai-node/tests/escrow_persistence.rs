use std::fs;
use std::path::Path;

use privai_ledger::MemoryStore;
use privai_node::{EscrowStageStore, NodeConfig, NodeError, PrivaiNode};
use privai_nxms::{
    ContextId, EscrowApprovalBody, EscrowFundedBody, EscrowFundingDescriptor, EscrowSpendProposal,
    EscrowSpendProposalBody, Hash32, PrivaiBody,
};

fn temp_dir(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "privai-persist-test-{}-{}",
        test_name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn make_node(dir: &Path) -> PrivaiNode<MemoryStore> {
    let mut config = NodeConfig::example();
    config.data_dir = dir.to_str().unwrap().to_string();
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

// ── Test 1: funded escrow survives node reopen ─────────────────────────

#[test]
fn funded_escrow_survives_node_reopen() {
    let dir = temp_dir("funded_survives");

    // Phase 1: ingest funded
    {
        let mut node = make_node(&dir);
        node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: ctx(1),
            descriptor: dummy_descriptor([10; 32]),
            funding_tx_ref: [20; 32],
        }))
        .unwrap();
    }

    // Phase 2: reopen node
    {
        let node = make_node(&dir);
        let staged = node
            .get_staged_escrow(&[10; 32])
            .expect("escrow should survive restart");
        assert_eq!(staged.escrow_id, [10; 32]);
        assert_eq!(staged.funding_tx_ref, [20; 32]);
        assert_eq!(staged.session_context, ctx(1));
    }
}

// ── Test 2: funded + proposal survive node reopen ──────────────────────

#[test]
fn funded_and_proposal_survive_node_reopen() {
    let dir = temp_dir("proposal_survives");

    // Phase 1: fund + proposal
    {
        let mut node = make_node(&dir);
        node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: ctx(1),
            descriptor: dummy_descriptor([10; 32]),
            funding_tx_ref: [20; 32],
        }))
        .unwrap();

        node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: ctx(1),
            proposal: EscrowSpendProposal {
                proposal_hash: [30; 32],
                escrow_id: [10; 32],
                snapshot_hash: [40; 32],
                action: 0,
            },
        }))
        .unwrap();
    }

    // Phase 2: reopen node
    {
        let node = make_node(&dir);

        let staged_escrow = node
            .get_staged_escrow(&[10; 32])
            .expect("escrow should survive");
        assert_eq!(staged_escrow.escrow_id, [10; 32]);

        let staged_proposal = node
            .get_staged_proposal(&[30; 32])
            .expect("proposal should survive");
        assert_eq!(staged_proposal.proposal.escrow_id, [10; 32]);
        assert_eq!(staged_proposal.proposal.action, 0);
        assert!(staged_proposal.approvals.is_empty());
    }
}

// ── Test 3: funded + proposal + approvals survive node reopen ──────────

#[test]
fn funded_proposal_approvals_survive_node_reopen() {
    let dir = temp_dir("approvals_survive");

    // Phase 1: fund + proposal + 2 approvals
    {
        let mut node = make_node(&dir);
        node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: ctx(1),
            descriptor: dummy_descriptor([10; 32]),
            funding_tx_ref: [20; 32],
        }))
        .unwrap();

        node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: ctx(1),
            proposal: EscrowSpendProposal {
                proposal_hash: [30; 32],
                escrow_id: [10; 32],
                snapshot_hash: [40; 32],
                action: 0,
            },
        }))
        .unwrap();

        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash: [30; 32],
            signer_pk: [1; 32],
            signature: vec![0xAA],
        }))
        .unwrap();

        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash: [30; 32],
            signer_pk: [2; 32],
            signature: vec![0xBB],
        }))
        .unwrap();
    }

    // Phase 2: reopen node
    {
        let node = make_node(&dir);

        let staged_proposal = node
            .get_staged_proposal(&[30; 32])
            .expect("proposal should survive");
        assert_eq!(staged_proposal.approvals.len(), 2);

        let ready = node
            .get_escrow_ready_approvals(&[30; 32])
            .expect("should have ready approvals");
        assert_eq!(ready.len(), 2);
        // Sorted by signer_pk
        assert!(ready[0].signer_pk <= ready[1].signer_pk);
    }
}

// ── Test 4: quorum readiness survives node reopen ──────────────────────

#[test]
fn quorum_readiness_survives_node_reopen() {
    let dir = temp_dir("quorum_survives");

    // Phase 1: build quorum
    {
        let mut node = make_node(&dir);
        node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: ctx(1),
            descriptor: dummy_descriptor([10; 32]),
            funding_tx_ref: [20; 32],
        }))
        .unwrap();

        node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: ctx(1),
            proposal: EscrowSpendProposal {
                proposal_hash: [30; 32],
                escrow_id: [10; 32],
                snapshot_hash: [40; 32],
                action: 0,
            },
        }))
        .unwrap();

        // Only 1 approval — not quorum yet
        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash: [30; 32],
            signer_pk: [1; 32],
            signature: vec![0xAA],
        }))
        .unwrap();

        assert!(!node.is_escrow_quorum_ready(&[30; 32]));
    }

    // Phase 2: reopen — still not quorum
    {
        let node = make_node(&dir);
        assert!(!node.is_escrow_quorum_ready(&[30; 32]));
    }

    // Phase 3: add second approval, achieve quorum
    {
        let mut node = make_node(&dir);
        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash: [30; 32],
            signer_pk: [2; 32],
            signature: vec![0xBB],
        }))
        .unwrap();

        assert!(node.is_escrow_quorum_ready(&[30; 32]));
    }

    // Phase 4: reopen — quorum must still hold
    {
        let node = make_node(&dir);
        assert!(node.is_escrow_quorum_ready(&[30; 32]));
        let ready = node.get_escrow_ready_approvals(&[30; 32]).unwrap();
        assert_eq!(ready.len(), 2);
    }
}

// ── Test 5: missing snapshot file starts with empty store ──────────────

#[test]
fn missing_snapshot_file_starts_empty() {
    let dir = temp_dir("missing_snapshot");
    // Don't write anything — just open a node

    let node = make_node(&dir);
    assert!(node.get_staged_escrow(&[10; 32]).is_none());
    assert!(node.get_staged_proposal(&[30; 32]).is_none());
    assert!(!node.is_escrow_quorum_ready(&[30; 32]));
}

// ── Test 6: corrupted snapshot file returns controlled error ────────────

#[test]
fn corrupted_snapshot_returns_controlled_error() {
    let dir = temp_dir("corrupt_snapshot");
    let snapshot_path = dir.join(EscrowStageStore::SNAPSHOT_FILENAME);
    fs::create_dir_all(&dir).unwrap();
    fs::write(&snapshot_path, b"NOT VALID JSON {{{").unwrap();

    let mut config = NodeConfig::example();
    config.data_dir = dir.to_str().unwrap().to_string();

    let result = PrivaiNode::<MemoryStore>::open(config, MemoryStore::new());
    assert!(result.is_err());
    let err = result.err().expect("should be error");
    match err {
        NodeError::EscrowPersistence(msg) => {
            assert!(
                msg.contains("corrupt"),
                "error should mention corruption: {msg}"
            );
        }
        other => panic!("expected EscrowPersistence error, got: {other:?}"),
    }
}

// ── Test 7: idempotent behavior still works after reload ───────────────

#[test]
fn idempotent_funded_works_after_reload() {
    let dir = temp_dir("idempotent_reload");

    let funded = EscrowFundedBody {
        session_context: ctx(1),
        descriptor: dummy_descriptor([10; 32]),
        funding_tx_ref: [20; 32],
    };

    // Phase 1: ingest funded
    {
        let mut node = make_node(&dir);
        node.handle_privai_body(PrivaiBody::EscrowFunded(funded.clone()))
            .unwrap();
    }

    // Phase 2: reopen and re-ingest the same funded body (idempotent)
    {
        let mut node = make_node(&dir);
        let result = node.handle_privai_body(PrivaiBody::EscrowFunded(funded));
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            privai_node::EscrowIngestOutcome::FundedStored
        );
    }
}

// ── Test 8: conflicting funded after reload is rejected ────────────────

#[test]
fn conflicting_funded_after_reload_rejected() {
    let dir = temp_dir("conflict_reload");

    // Phase 1: ingest funded with tx_ref [20; 32]
    {
        let mut node = make_node(&dir);
        node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: ctx(1),
            descriptor: dummy_descriptor([10; 32]),
            funding_tx_ref: [20; 32],
        }))
        .unwrap();
    }

    // Phase 2: reopen and try to ingest same escrow with DIFFERENT tx_ref
    {
        let mut node = make_node(&dir);
        let result = node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: ctx(1),
            descriptor: dummy_descriptor([10; 32]),
            funding_tx_ref: [99; 32], // different ref
        }));
        assert!(result.is_err());
    }
}

// ── Test 9: snapshot file is atomic (temp+rename) ──────────────────────

#[test]
fn snapshot_write_is_atomic() {
    let dir = temp_dir("atomic_write");
    let snapshot_path = dir.join(EscrowStageStore::SNAPSHOT_FILENAME);

    {
        let mut node = make_node(&dir);
        node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: ctx(1),
            descriptor: dummy_descriptor([10; 32]),
            funding_tx_ref: [20; 32],
        }))
        .unwrap();
    }

    // After successful write, the temp file should NOT exist
    let tmp_path = snapshot_path.with_extension("tmp");
    assert!(
        !tmp_path.exists(),
        "temp file should be cleaned up after rename"
    );
    assert!(snapshot_path.exists(), "snapshot file should exist");
}

// ── Test 10: empty data_dir disables persistence gracefully ────────────

#[test]
fn empty_data_dir_disables_persistence() {
    let mut config = NodeConfig::example();
    config.data_dir = String::new();
    let mut node = PrivaiNode::<MemoryStore>::open(config, MemoryStore::new()).expect("node");

    // Ingest should work fine without persistence
    node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
        session_context: ctx(1),
        descriptor: dummy_descriptor([10; 32]),
        funding_tx_ref: [20; 32],
    }))
    .unwrap();

    assert!(node.get_staged_escrow(&[10; 32]).is_some());
    assert!(node.escrow_snapshot_path().is_none());
}

// ── Test 11: multiple escrows persist independently ────────────────────

#[test]
fn multiple_escrows_persist_independently() {
    let dir = temp_dir("multi_escrow");

    {
        let mut node = make_node(&dir);

        // Escrow A
        node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: ctx(1),
            descriptor: dummy_descriptor([10; 32]),
            funding_tx_ref: [20; 32],
        }))
        .unwrap();

        // Escrow B
        node.handle_privai_body(PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: ctx(2),
            descriptor: dummy_descriptor([20; 32]),
            funding_tx_ref: [30; 32],
        }))
        .unwrap();

        // Proposal for escrow A only
        node.handle_privai_body(PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: ctx(1),
            proposal: EscrowSpendProposal {
                proposal_hash: [30; 32],
                escrow_id: [10; 32],
                snapshot_hash: [40; 32],
                action: 0,
            },
        }))
        .unwrap();
    }

    // Reopen and verify both escrows and the proposal persist
    {
        let node = make_node(&dir);
        assert!(node.get_staged_escrow(&[10; 32]).is_some());
        assert!(node.get_staged_escrow(&[20; 32]).is_some());
        assert!(node.get_staged_proposal(&[30; 32]).is_some());
    }
}

// ── Test 12: direct store load/save roundtrip ──────────────────────────

#[test]
fn store_load_save_roundtrip() {
    let dir = temp_dir("roundtrip");
    let path = dir.join(EscrowStageStore::SNAPSHOT_FILENAME);

    // Build a store with funded + proposal + 2 approvals
    let mut store = EscrowStageStore::new();
    store
        .ingest_funded(EscrowFundedBody {
            session_context: ctx(1),
            descriptor: dummy_descriptor([10; 32]),
            funding_tx_ref: [20; 32],
        })
        .unwrap();
    store
        .ingest_proposal(EscrowSpendProposalBody {
            session_context: ctx(1),
            proposal: EscrowSpendProposal {
                proposal_hash: [30; 32],
                escrow_id: [10; 32],
                snapshot_hash: [40; 32],
                action: 0,
            },
        })
        .unwrap();
    store
        .ingest_approval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash: [30; 32],
            signer_pk: [1; 32],
            signature: vec![0xAA],
        })
        .unwrap();
    store
        .ingest_approval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash: [30; 32],
            signer_pk: [2; 32],
            signature: vec![0xBB],
        })
        .unwrap();

    store.save_to_path(&path).unwrap();

    // Load from disk
    let loaded = EscrowStageStore::load_from_path(&path).unwrap();
    assert!(loaded.funded_escrows.contains_key(&[10; 32]));
    assert!(loaded.proposals.contains_key(&[30; 32]));
    assert_eq!(loaded.proposals.get(&[30; 32]).unwrap().approvals.len(), 2);
    assert!(loaded.is_quorum_ready(&[30; 32]));
}

// ── Test 13: load from nonexistent path returns empty store ─────────────

#[test]
fn load_nonexistent_path_returns_empty_store() {
    let path = Path::new("/tmp/privai-nonexistent-12345/escrow_stage.json");
    let store = EscrowStageStore::load_from_path(path).unwrap();
    assert!(store.funded_escrows.is_empty());
    assert!(store.proposals.is_empty());
}
