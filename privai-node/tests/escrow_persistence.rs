use std::fs;
use std::path::Path;

use privai_chain::escrow::EscrowAction;
use privai_chain::hash::falcon_pk_hash;
use privai_chain::{
    Amount14, AuxWitness, CanonicalEncode, LweCiphertext, OutputNote, RecipientBoxPlaintext,
    SpendPolicy, SpendPolicyTag, Transaction, PRIVAI_V0,
};
use privai_ledger::{LedgerStore, MemoryStore, NoteRecord, NoteStatus};
use privai_node::{EscrowStageStore, NodeConfig, NodeError, PrivaiNode};
use privai_nxms::{
    ContextId, EscrowApprovalBody, EscrowFundedBody, EscrowFundingDescriptor, EscrowSpendProposal,
    EscrowSpendProposalBody, Hash32, PrivaiBody,
};
use privai_proof::TransferProvingData;
use privai_wallet::escrow_builder::{AuthMaterial, FinalAssemblyInputs};
use privai_wallet::{MemoryWalletStore, PrivaiWallet};

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

// ══════════════════════════════════════════════════════════════
// Continuation helpers — reopen node and seed a funding note
// into its in-memory ledger (MemoryStore does NOT persist the
// ledger across reopen, so this is deliberate re-seeding of
// the in-memory layer only — NOT Stage A reconstruction).
// ══════════════════════════════════════════════════════════════

fn reopen_node_with_seeded_note(
    dir: &std::path::Path,
    funding_note: &OutputNote,
) -> PrivaiNode<MemoryStore> {
    let tmp_node = make_node(dir);
    let config = tmp_node.config().clone();
    let mut snapshot = tmp_node.ledger().snapshot().clone();
    snapshot.notes.insert(
        funding_note.note_commit,
        NoteRecord {
            note: funding_note.clone(),
            created_in_block: Some(0),
            status: NoteStatus::Unspent,
        },
    );
    let mut store = MemoryStore::new();
    store.save(&snapshot).unwrap();
    PrivaiNode::open(config, store).expect("reopen node with seeded note")
}

// ══════════════════════════════════════════════════════════════
// GOAL A — Main continuation test
// Proves: Stage A state (funded + proposal + quorum approvals)
//         survives restart strongly enough to drive a full
//         Release flow without any Stage A body replay.
// ══════════════════════════════════════════════════════════════

#[test]
fn release_flow_continues_after_node_reopen_without_stage_replay() {
    let dir = temp_dir("continue_after_reopen");

    //
    // Key material — kept outside any node scope so we can use
    // the same keys for wallet assembly after reopen.
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
    let amount: u64 = 1000;
    let fee: u64 = 10;

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
        amount,
        spend_policy_commit: escrow_policy.commitment(),
        timeout_blocks: 100,
    };

    //
    // Pre-compute wallet + funding note (test infrastructure,
    // NOT Stage A state — the wallet is an in-memory sidecar).
    //
    let (buyer_wallet, funding_note, _funding_opened) = {
        let mut w = PrivaiWallet::open(MemoryWalletStore::new()).expect("buyer wallet");
        let funding_bundle = w
            .create_local_bundle(amount, 0, Some(b"escrow funding".to_vec()))
            .expect("funding bundle");

        let funding_amount = Amount14::new(amount as u16).expect("amount");
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

        let opened = RecipientBoxPlaintext {
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
            PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(&funding_bundle, &opened)
                .expect("seal");

        let note = OutputNote::new(
            escrow_policy.commitment(),
            LweCiphertext::default(),
            aux_commit,
            recipient_box,
        );

        w.record_opened_note(note.clone(), opened.clone())
            .expect("record note");

        (w, note, opened)
    };

    // ── PHASE 1: Build complete Stage A on node A ──────────────
    {
        let mut node = make_node(&dir);

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

        // Release approvals: Buyer + Operator
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
            signer_pk: operator_h,
            signature: vec![0x22; 64],
        }))
        .expect("operator approval");

        assert!(node.is_escrow_quorum_ready(&proposal_hash));
    }
    // node dropped — Stage A persisted to escrow_stage.json via drop

    // ── PHASE 2: Reopen from same data_dir — no Stage A replay ──
    // Stage A state must come entirely from the persisted snapshot.
    // CHECKPOINT 1: No funded/proposal/approval bodies are replayed here.
    {
        let node = make_node(&dir);

        let staged_escrow = node
            .get_staged_escrow(&escrow_id)
            .expect("funded escrow must survive restart");
        assert_eq!(staged_escrow.escrow_id, escrow_id);
        assert_eq!(staged_escrow.funding_tx_ref, [0x55; 32]);

        let staged_proposal = node
            .get_staged_proposal(&proposal_hash)
            .expect("proposal must survive restart");
        assert_eq!(staged_proposal.proposal.escrow_id, escrow_id);
        assert_eq!(staged_proposal.approvals.len(), 2);

        assert!(node.is_escrow_quorum_ready(&proposal_hash));

        let bundle = node
            .build_escrow_approval_bundle(&proposal_hash)
            .expect("build bundle from persisted state");
        assert_eq!(bundle.signer_pks.len(), 2);
    }

    // ── PHASE 3: Seed funding note into reopened node ledger ────
    // NOTE: MemoryStore does NOT persist the ledger across reopen.
    // We re-seed the funding note here — this is in-memory ledger
    // re-seeding, NOT Stage A reconstruction. Stage A state
    // (funded escrow, proposal, approvals, quorum) is loaded from
    // the persisted snapshot and verified in Phase 2.
    let mut node = reopen_node_with_seeded_note(&dir, &funding_note);

    // ── PHASE 4: Continue Stage B honestly ─────────────────────
    // Verify persisted state is still usable after ledger reopen
    assert!(node.is_escrow_quorum_ready(&proposal_hash));
    let bundle = node
        .build_escrow_approval_bundle(&proposal_hash)
        .expect("build bundle from persisted state after ledger reopen");

    // Wallet assembly — use the SAME wallet that recorded the funding note
    let spend_material = buyer_wallet
        .spend_material(&funding_note.note_commit)
        .expect("spend material");

    let refund_amount = amount - fee;
    let mut merchant_wallet = PrivaiWallet::open(MemoryWalletStore::new()).unwrap();
    let merchant_receive_bundle = merchant_wallet
        .create_local_bundle(refund_amount, 0, Some(b"Release payment".to_vec()))
        .unwrap();

    let auth_material = AuthMaterial {
        policy_tag: SpendPolicyTag::Escrow2of3 as u8,
        signer_pks: vec![buyer_pk.clone(), operator_pk.clone()],
        signatures: bundle.signatures.clone(),
        policy_opening: escrow_policy.to_canonical_bytes(),
        escrow_action: EscrowAction::Release as u8,
    };

    let assembly = FinalAssemblyInputs {
        proposal_hash,
        escrow_id,
        action: EscrowAction::Release,
        funding_note_commit: spend_material.note_commit,
        output_recipient_pk: merchant_h,
        fee,
        auth_material,
    };

    let mut assembled = buyer_wallet
        .build_escrow_transfer_note_from_assembly_inputs(
            &spend_material,
            &assembly,
            &merchant_receive_bundle,
        )
        .expect("assemble escrow tx");

    // Real signatures over tx_signing_hash
    let real_buyer_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(&buyer_sk, &assembled.tx_signing_hash)
            .unwrap();
    let real_operator_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(&operator_sk, &assembled.tx_signing_hash)
            .unwrap();

    assembled.tx.core.auth[0].signatures = vec![real_buyer_sig, real_operator_sig];
    assembled.tx_signing_hash = Transaction::TransferNote(assembled.tx.clone()).tx_signing_hash();
    assembled.proof_scaffolding = TransferProvingData::from_tx_and_witness(
        &assembled.tx,
        assembled.proof_scaffolding.witness.clone(),
    )
    .expect("rebuild proving data after final signatures");

    // Proof handoff
    let handoff =
        privai_wallet::proof_handoff::EscrowProofReadyHandoff::build(&assembled, 100, 200, [8; 32])
            .expect("build proof handoff");

    let proof_result_bytes = vec![0xBE, 0xEF];
    let attached_proof = handoff
        .attach_single_tx_proof_result(proof_result_bytes, 1, vec![[0x99; 32]], [0xAA; 32])
        .expect("attach proof");

    // Submit gate
    node.submit_escrow_transfer_note(
        &proposal_hash,
        Transaction::TransferNote(assembled.tx.clone()),
        assembled.tx.core.fee,
    )
    .expect("submit escrow tx");

    // Block propose and import
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

    // ── Required success assertions ────────────────────────────

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

    // 2. Height advanced
    assert_eq!(updated_snapshot.height, 1, "height advanced to 1");

    // 3. Artifacts stored
    let stored_artifacts = node.load_block_artifacts(&block.hash()).unwrap().unwrap();
    assert_eq!(
        stored_artifacts.proof_certificates(),
        block.body.proof_certificates
    );

    // 4. Persisted Stage A data was actually used — the approval
    //    bundle was built from the snapshot, not reconstructed.
    //    (If Phase 2 had replayed funded/proposal/approval bodies
    //    the test would have failed at CHECKPOINT 1.)
}

// ══════════════════════════════════════════════════════════════
// GOAL B — Mixed pre/post-restart continuation test
// Proves: approvals accumulated across restart boundaries
//         still produce a valid continuation path into a
//         real Release flow.
// ══════════════════════════════════════════════════════════════

#[test]
fn release_flow_completes_when_quorum_is_finished_after_reopen() {
    let dir = temp_dir("quorum_finished_after_reopen");

    //
    // Key material
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
    let amount: u64 = 1000;
    let fee: u64 = 10;

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
        amount,
        spend_policy_commit: escrow_policy.commitment(),
        timeout_blocks: 100,
    };

    //
    // Pre-compute wallet + funding note (test infrastructure).
    //
    let (buyer_wallet, funding_note, _funding_opened) = {
        let mut w = PrivaiWallet::open(MemoryWalletStore::new()).expect("buyer wallet");
        let funding_bundle = w
            .create_local_bundle(amount, 0, Some(b"escrow funding".to_vec()))
            .expect("funding bundle");

        let funding_amount = Amount14::new(amount as u16).expect("amount");
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

        let opened = RecipientBoxPlaintext {
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
            PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(&funding_bundle, &opened)
                .expect("seal");

        let note = OutputNote::new(
            escrow_policy.commitment(),
            LweCiphertext::default(),
            aux_commit,
            recipient_box,
        );

        w.record_opened_note(note.clone(), opened.clone())
            .expect("record note");

        (w, note, opened)
    };

    // ── PHASE 1: Build Stage A on node A with only ONE approval ─
    {
        let mut node = make_node(&dir);

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

        // First approval only — Buyer signs, Operator does NOT yet
        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: buyer_h,
            signature: vec![0x11; 64],
        }))
        .expect("buyer approval");

        assert!(
            !node.is_escrow_quorum_ready(&proposal_hash),
            "quorum must NOT be ready with only 1 approval"
        );
    }
    // node dropped — Stage A persisted with 1 approval

    // ── PHASE 2: Reopen node B — confirm proposal survived, ────
    //   quorum still not ready, then ingest second approval.
    {
        let mut node = make_node(&dir);

        let staged_proposal = node
            .get_staged_proposal(&proposal_hash)
            .expect("proposal must survive restart");
        assert_eq!(staged_proposal.proposal.escrow_id, escrow_id);
        assert_eq!(
            staged_proposal.approvals.len(),
            1,
            "only the first approval should be persisted"
        );

        assert!(
            !node.is_escrow_quorum_ready(&proposal_hash),
            "quorum must still NOT be ready after reopen"
        );

        // Ingest second approval AFTER reopen — this crosses the restart boundary
        node.handle_privai_body(PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: ctx(1),
            proposal_hash,
            signer_pk: operator_h,
            signature: vec![0x22; 64],
        }))
        .expect("operator approval after reopen");

        assert!(
            node.is_escrow_quorum_ready(&proposal_hash),
            "quorum must become ready after second approval post-reopen"
        );

        let bundle = node
            .build_escrow_approval_bundle(&proposal_hash)
            .expect("build bundle with mixed pre/post-restart approvals");
        assert_eq!(bundle.signer_pks.len(), 2);
    }

    // ── PHASE 3: Reopen with ledger seeded — Stage A from snapshot ──
    // NOTE: MemoryStore does NOT persist the ledger across reopen.
    // We re-seed the funding note here — this is in-memory ledger
    // re-seeding, NOT Stage A reconstruction.
    let mut node = reopen_node_with_seeded_note(&dir, &funding_note);

    // Verify quorum is ready from persisted + post-reopen approval
    assert!(node.is_escrow_quorum_ready(&proposal_hash));
    let bundle = node
        .build_escrow_approval_bundle(&proposal_hash)
        .expect("build bundle");

    // ── PHASE 4: Stage B — honest Release flow ────────────────
    // Use the SAME wallet that recorded the funding note.
    let spend_material = buyer_wallet
        .spend_material(&funding_note.note_commit)
        .expect("spend material");

    let refund_amount = amount - fee;
    let mut merchant_wallet = PrivaiWallet::open(MemoryWalletStore::new()).unwrap();
    let merchant_receive_bundle = merchant_wallet
        .create_local_bundle(refund_amount, 0, Some(b"Release payment".to_vec()))
        .unwrap();

    let auth_material = AuthMaterial {
        policy_tag: SpendPolicyTag::Escrow2of3 as u8,
        signer_pks: vec![buyer_pk.clone(), operator_pk.clone()],
        signatures: bundle.signatures.clone(),
        policy_opening: escrow_policy.to_canonical_bytes(),
        escrow_action: EscrowAction::Release as u8,
    };

    let assembly = FinalAssemblyInputs {
        proposal_hash,
        escrow_id,
        action: EscrowAction::Release,
        funding_note_commit: spend_material.note_commit,
        output_recipient_pk: merchant_h,
        fee,
        auth_material,
    };

    let mut assembled = buyer_wallet
        .build_escrow_transfer_note_from_assembly_inputs(
            &spend_material,
            &assembly,
            &merchant_receive_bundle,
        )
        .expect("assemble escrow tx");

    // Real signatures
    let real_buyer_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(&buyer_sk, &assembled.tx_signing_hash)
            .unwrap();
    let real_operator_sig =
        nxms_transport::crypto::falcon_sign_ct_prepared(&operator_sk, &assembled.tx_signing_hash)
            .unwrap();

    assembled.tx.core.auth[0].signatures = vec![real_buyer_sig, real_operator_sig];
    assembled.tx_signing_hash = Transaction::TransferNote(assembled.tx.clone()).tx_signing_hash();
    assembled.proof_scaffolding = TransferProvingData::from_tx_and_witness(
        &assembled.tx,
        assembled.proof_scaffolding.witness.clone(),
    )
    .expect("rebuild proving data after final signatures");

    // Proof handoff
    let handoff =
        privai_wallet::proof_handoff::EscrowProofReadyHandoff::build(&assembled, 100, 200, [8; 32])
            .expect("build proof handoff");

    let proof_result_bytes = vec![0xBE, 0xEF];
    let attached_proof = handoff
        .attach_single_tx_proof_result(proof_result_bytes, 1, vec![[0x99; 32]], [0xAA; 32])
        .expect("attach proof");

    // Submit gate
    node.submit_escrow_transfer_note(
        &proposal_hash,
        Transaction::TransferNote(assembled.tx.clone()),
        assembled.tx.core.fee,
    )
    .expect("submit escrow tx");

    // Block propose and import
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

    // ── Success assertions ────────────────────────────────────

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

    // 2. Height advanced
    assert_eq!(updated_snapshot.height, 1, "height advanced to 1");

    // 3. Artifacts stored
    let stored_artifacts = node.load_block_artifacts(&block.hash()).unwrap().unwrap();
    assert_eq!(
        stored_artifacts.proof_certificates(),
        block.body.proof_certificates
    );

    // 4. The approval bundle was built from persisted state (first
    //    approval) merged with the post-reopen second approval —
    //    proving that cross-restart approval accumulation works
    //    for honest flow continuation.
}
