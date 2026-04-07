
use tempfile::tempdir;

use privai_chain::small_payments::SettlementBatchSummary;
use privai_chain::tx::{MarketplaceBatchTx, TX_TYPE_MARKETPLACE_BATCH};
use privai_chain::{
    merkle_root, Block, BlockTemplate, ExecutionBundle, ExecutionMode, Hash32, InputRef, Nullifier,
    OutputNote, ProofCertificate, RecipientBox, Transaction, TransferNoteTx, TxCore,
    TX_TYPE_TRANSFER_NOTE,
};
use privai_proof::StructuralProofVerifier;

use privai_ledger::{
    apply_transaction_local, compute_state_root,
    state::{ConsensusSafetyState, LedgerSnapshot, NoteRecord, NoteStatus},
    store::{FileSystemStore, LedgerStore, MemoryStore},
    Ledger,
};

fn sample_note(seed: u8) -> OutputNote {
    OutputNote::new(
        [seed; 32],
        privai_chain::LweCiphertext::default(),
        [seed.wrapping_add(1); 32],
        RecipientBox::new(
            vec![seed],
            [seed; 24],
            vec![seed + 1],
            [seed; 16],
            [seed; 16],
        ),
    )
}

fn test_epoch_params() -> privai_chain::EpochParams {
    privai_chain::EpochParams {
        epoch_number: 0,
        start_height: 0,
        end_height: 1_000_000,
        min_validator_stake: 0,
        min_prover_bond: 0,
        min_fee: 0,
        max_block_bytes: 10_000_000,
        max_block_statements: 100_000,
        min_proof_coverage: 1,
    }
}

/// Helper: build a minimal valid block that extends the current tip.
fn make_block(
    height: u64,
    prev_block_hash: Hash32,
    seed: u8,
    current_snapshot: &LedgerSnapshot,
) -> Block {
    let output = sample_note(seed);
    let tx = Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: 0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![],
            input_nullifiers: vec![],
            outputs: vec![output],
            fee: 0,
            statement_commit: [seed; 32],
            auth: Vec::new(),
        },
    });

    let execution_bundle = ExecutionBundle {
        statement_commits: vec![tx.statement_commit()],
        covered_tx_indexes: vec![0],
        public_inputs_root: [seed; 32],
        execution_mode: ExecutionMode::FullBatchProof,
    };

    let mut temp = current_snapshot.clone();
    temp.height = height.saturating_sub(1);
    temp.tip_hash = prev_block_hash;
    apply_transaction_local(&tx, height, &mut temp);
    let state_root = compute_state_root(&temp);

    let statement_root = merkle_root(execution_bundle.statement_commits.iter().copied());

    Block::from_template(BlockTemplate {
        chain_id: privai_chain::DEFAULT_CHAIN_ID,
        height,
        epoch: 0,
        round: 0,
        timestamp_ms: height * 1000,
        prev_block_hash,
        proposer_pk_hash: [1; 32],
        epoch_seed_hash: [2; 32],
        parent_qc_hash: [3; 32],
        state_root,
        txs: vec![tx],
        execution_bundle,
        proof_certificates: vec![ProofCertificate {
            proof_system_id: 1,
            statement_root,
            public_inputs_root: [seed; 32],
            proof_bytes_hash: [6; 32],
            prover_ids: vec![[8; 32]],
            proof_meta_hash: [9; 32],
        }],
        extra_receipts: Vec::new(),
    })
}

// -----------------------------------------------------------------------
// 1. persist_state_then_restart_same_root
// -----------------------------------------------------------------------

#[test]
fn test_persist_state_then_restart_same_root_memory() {
    let mut ledger = Ledger::open(
        MemoryStore::new(),
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    let block1 = make_block(1, [0; 32], 10, ledger.snapshot());
    ledger
        .apply_block(&block1, 1, &test_epoch_params())
        .expect("apply block 1");

    let root_before = compute_state_root(ledger.snapshot());
    let height_before = ledger.snapshot().height;

    // Odtwarzamy stan korzystając z tego samego mechanizmu, który MemoryStore oferuje naturalnie
    let mut store2 = MemoryStore::new();
    store2
        .save(ledger.snapshot())
        .expect("save to memory store");
    let ledger_reloaded = Ledger::open(
        store2,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("reload");

    let root_after = compute_state_root(ledger_reloaded.snapshot());

    assert_eq!(
        root_before, root_after,
        "state_root musi przetrwać restart (MemoryStore)"
    );
    assert_eq!(ledger_reloaded.snapshot().height, height_before);
}

// Regression test: verifies that the BTreeMap<Hash32, NoteRecord> serde fix works.
// (Previously this was a blocker test that expected failure.)
#[test]
fn test_filesystem_store_serializes_notes_correctly() {
    let dir = tempdir().expect("tempdir");
    let store = FileSystemStore::new(dir.path());

    let mut ledger = Ledger::open(
        store,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    let block1 = make_block(1, [0; 32], 10, ledger.snapshot());

    // Apply block triggers a flush() — should succeed now with hex_key_map serde fix
    let result = ledger.apply_block(&block1, 1, &test_epoch_params());

    assert!(
        result.is_ok(),
        "apply_block should succeed after serde fix, got: {:?}",
        result.err()
    );
}

#[test]
fn test_persist_state_then_restart_same_root_filesystem() {
    let dir = tempdir().expect("tempdir");
    let mut store = FileSystemStore::new(dir.path());
    store.ensure_initialized().expect("init");

    let mut ledger = Ledger::open(
        store,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    let block1 = make_block(1, [0; 32], 10, ledger.snapshot());
    ledger
        .apply_block(&block1, 1, &test_epoch_params())
        .expect("apply block 1");

    let root_before = compute_state_root(ledger.snapshot());
    let height_before = ledger.snapshot().height;

    let store2 = FileSystemStore::new(dir.path());
    let ledger_reloaded = Ledger::open(
        store2,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("reload");

    let root_after = compute_state_root(ledger_reloaded.snapshot());

    assert_eq!(root_before, root_after, "state_root musi przetrwać restart");
    assert_eq!(ledger_reloaded.snapshot().height, height_before);
}

// -----------------------------------------------------------------------
// 2. recover_tip_and_height_after_restart
// -----------------------------------------------------------------------

#[test]
fn test_recover_tip_and_height_after_restart_memory() {
    let mut ledger = Ledger::open(
        MemoryStore::new(),
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    let mut prev = [0u8; 32];
    for i in 1..=3 {
        let block = make_block(i, prev, i as u8, ledger.snapshot());
        prev = block.hash();
        ledger
            .apply_block(&block, 1, &test_epoch_params())
            .expect("apply block");
    }

    assert_eq!(ledger.snapshot().height, 3);
    let tip = ledger.snapshot().tip_hash;

    let mut store2 = MemoryStore::new();
    store2
        .save(ledger.snapshot())
        .expect("save to memory store");
    let ledger2 = Ledger::open(
        store2,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("reload");

    assert_eq!(ledger2.snapshot().height, 3, "height musi zostać odzyskana");
    assert_eq!(
        ledger2.snapshot().tip_hash,
        tip,
        "tip_hash musi zostać odzyskany"
    );
}

#[test]
fn test_recover_tip_and_height_after_restart_filesystem() {
    let dir = tempdir().expect("tempdir");
    let mut store = FileSystemStore::new(dir.path());
    store.ensure_initialized().expect("init");

    let mut ledger = Ledger::open(
        store,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    let mut prev = [0u8; 32];
    for i in 1..=3 {
        let block = make_block(i, prev, i as u8, ledger.snapshot());
        prev = block.hash();
        ledger
            .apply_block(&block, 1, &test_epoch_params())
            .expect("apply block");
    }

    let tip = ledger.snapshot().tip_hash;

    let store2 = FileSystemStore::new(dir.path());
    let ledger2 = Ledger::open(
        store2,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("reload");

    assert_eq!(ledger2.snapshot().height, 3, "height musi zostać odzyskana");
    assert_eq!(
        ledger2.snapshot().tip_hash,
        tip,
        "tip_hash musi zostać odzyskany"
    );
}

// -----------------------------------------------------------------------
// 3. recover_spent_nullifiers_after_restart
// -----------------------------------------------------------------------

#[test]
fn test_recover_spent_nullifiers_after_restart_memory() {
    let mut ledger = Ledger::open(
        MemoryStore::new(),
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    let funding_note = sample_note(50);
    ledger.snapshot_mut().notes.insert(
        funding_note.note_commit,
        NoteRecord {
            note: funding_note.clone(),
            created_in_block: Some(0),
            status: NoteStatus::Unspent,
        },
    );

    let nullifier = Nullifier([0xee; 32]);
    let spend_tx = Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: 0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef {
                note_commit: funding_note.note_commit,
            }],
            input_nullifiers: vec![nullifier],
            outputs: vec![sample_note(51)],
            fee: 0,
            statement_commit: [51; 32],
            auth: Vec::new(),
        },
    });

    apply_transaction_local(&spend_tx, 1, ledger.snapshot_mut());
    ledger.flush().expect("flush");

    assert_eq!(ledger.snapshot().spent_nullifiers.len(), 1);

    let mut store2 = MemoryStore::new();
    store2
        .save(ledger.snapshot())
        .expect("save to memory store");
    let ledger2 = Ledger::open(
        store2,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("reload");

    assert!(
        ledger2.snapshot().spent_nullifiers.contains(&nullifier),
        "nullifier noty musi zostać zachowany"
    );
}

#[test]
fn test_recover_spent_nullifiers_after_restart_filesystem() {
    let dir = tempdir().expect("tempdir");
    let mut store = FileSystemStore::new(dir.path());
    store.ensure_initialized().expect("init");

    let mut ledger = Ledger::open(
        store,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    let funding_note = sample_note(50);
    ledger.snapshot_mut().notes.insert(
        funding_note.note_commit,
        NoteRecord {
            note: funding_note.clone(),
            created_in_block: Some(0),
            status: NoteStatus::Unspent,
        },
    );

    let nullifier = Nullifier([0xee; 32]);
    let spend_tx = Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: 0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef {
                note_commit: funding_note.note_commit,
            }],
            input_nullifiers: vec![nullifier],
            outputs: vec![sample_note(51)],
            fee: 0,
            statement_commit: [51; 32],
            auth: Vec::new(),
        },
    });

    apply_transaction_local(&spend_tx, 1, ledger.snapshot_mut());
    ledger.flush().expect("flush");

    let store2 = FileSystemStore::new(dir.path());
    let ledger2 = Ledger::open(
        store2,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("reload");

    assert!(
        ledger2.snapshot().spent_nullifiers.contains(&nullifier),
        "nullifier noty musi zostać zachowany"
    );
}

// -----------------------------------------------------------------------
// 4. recover_ticket_nullifiers_after_restart
// -----------------------------------------------------------------------

#[test]
fn test_recover_ticket_nullifiers_after_restart() {
    // Ticket nullifiers are serialized properly because spent_ticket_nullifiers is a BTreeSet,
    // which serializes to a JSON array. However, we won't add any Notes here.
    let dir = tempdir().expect("tempdir");
    let mut store = FileSystemStore::new(dir.path());
    store.ensure_initialized().expect("init");

    let mut ledger = Ledger::open(
        store,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    let nullifier1 = Nullifier([0xcc; 32]);

    let summary = SettlementBatchSummary {
        operator_commit: Hash32::default(),
        merchant_commit: [2; 32],
        grant_commit: [3; 32],
        settlement_window_start: 0,
        settlement_window_end: 1000,
        receipt_root: [4; 32],
        receipt_count: 1,
        nullifier_count: 1,
        total_gross_amount: 100,
        total_fee_amount: 10,
        total_refund_amount: 0,
    };

    let batch_tx = Transaction::MarketplaceBatch(MarketplaceBatchTx {
        core: TxCore {
            version: 0,
            tx_type: TX_TYPE_MARKETPLACE_BATCH,
            inputs: vec![],
            input_nullifiers: vec![],
            outputs: vec![],
            fee: 10,
            statement_commit: [5; 32],
            auth: Vec::new(),
        },
        summary,
        ticket_nullifiers: vec![nullifier1.clone()],
        operator_sig: Vec::new(),
    });

    apply_transaction_local(&batch_tx, 1, ledger.snapshot_mut());
    ledger.flush().expect("flush");

    assert!(ledger.snapshot().is_ticket_nullifier_spent(&nullifier1));

    let store2 = FileSystemStore::new(dir.path());
    let ledger2 = Ledger::open(
        store2,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("reload");

    assert!(
        ledger2.snapshot().is_ticket_nullifier_spent(&nullifier1),
        "ticket nullifier musi zostać zachowany"
    );
}

// -----------------------------------------------------------------------
// 5. restart_with_missing_ephemeral_state_still_safe
// -----------------------------------------------------------------------

#[test]
fn test_restart_with_missing_ephemeral_state_still_safe() {
    let dir = tempdir().expect("tempdir");
    let mut store = FileSystemStore::new(dir.path());
    store.ensure_initialized().expect("init");

    let mut ledger = Ledger::open(
        store,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    let tx = Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: 0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![],
            input_nullifiers: vec![],
            outputs: vec![sample_note(99)],
            fee: 5,
            statement_commit: [99; 32],
            auth: Vec::new(),
        },
    });
    ledger.submit_transaction(tx, 1000).expect("submit");
    assert!(!ledger.mempool().is_empty(), "mempool powinien mieć wpis");

    ledger.flush().expect("flush");

    let store2 = FileSystemStore::new(dir.path());
    let ledger2 = Ledger::open(
        store2,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("reload");

    assert!(
        ledger2.mempool().is_empty(),
        "mempool musi być pusty po restarcie (ephemeral)"
    );
}

// -----------------------------------------------------------------------
// 6. state_root_matches_recomputed_state_after_reload
// -----------------------------------------------------------------------

#[test]
fn test_state_root_matches_recomputed_state_after_reload_memory() {
    let mut ledger = Ledger::open(
        MemoryStore::new(),
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    for i in 0..5 {
        let note = sample_note(100 + i);
        ledger.snapshot_mut().notes.insert(
            note.note_commit,
            NoteRecord {
                note,
                created_in_block: Some(0),
                status: NoteStatus::Unspent,
            },
        );
    }

    let root_before = compute_state_root(ledger.snapshot());
    ledger.flush().expect("flush");

    let mut store2 = MemoryStore::new();
    store2
        .save(ledger.snapshot())
        .expect("save to memory store");
    let ledger2 = Ledger::open(
        store2,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("reload");
    let root_after = compute_state_root(ledger2.snapshot());

    assert_eq!(
        root_before, root_after,
        "state_root musi pasować do odtworzonego stanu"
    );
}

#[test]
fn test_state_root_matches_recomputed_state_after_reload_filesystem() {
    let dir = tempdir().expect("tempdir");
    let mut store = FileSystemStore::new(dir.path());
    store.ensure_initialized().expect("init");

    let mut ledger = Ledger::open(
        store,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    for i in 0..5 {
        let note = sample_note(100 + i);
        ledger.snapshot_mut().notes.insert(
            note.note_commit,
            NoteRecord {
                note,
                created_in_block: Some(0),
                status: NoteStatus::Unspent,
            },
        );
    }

    let root_before = compute_state_root(ledger.snapshot());
    ledger.flush().expect("flush");

    let store2 = FileSystemStore::new(dir.path());
    let ledger2 = Ledger::open(
        store2,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("reload");
    let root_after = compute_state_root(ledger2.snapshot());

    assert_eq!(
        root_before, root_after,
        "state_root musi pasować do odtworzonego stanu"
    );
}

// -----------------------------------------------------------------------
// 7. Corrupted / Truncated / Missing Snapshot Recovery Rules Test
// -----------------------------------------------------------------------

#[test]
fn test_corrupted_snapshot_fails_to_load() {
    let dir = tempdir().expect("tempdir");
    let mut store = FileSystemStore::new(dir.path());
    store.ensure_initialized().expect("init");

    // Apply some data
    let mut ledger = Ledger::open(
        store,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");
    let block1 = make_block(1, [0; 32], 10, ledger.snapshot());
    ledger.apply_block(&block1, 1, &test_epoch_params()).expect("apply block 1");
    ledger.flush().expect("flush");

    // Corrupt the file manually with invalid JSON keys/chars
    let path = dir.path().join("ledger-state.json");
    let corrupted_content = b"{ \"chain_id\": 17, \"corrupted_unclosed_bracket_no_data... ";
    std::fs::write(&path, corrupted_content).expect("corrupting file");

    // Reopen ledger and check fail behavior
    let store2 = FileSystemStore::new(dir.path());
    let ledger_err = Ledger::open(store2, privai_chain::DEFAULT_CHAIN_ID, StructuralProofVerifier);

    assert!(
        ledger_err.is_err(),
        "Zepsuty plik zrzutu nie powinien zostać załadowany!"
    );
    let err_str = match ledger_err {
        Err(e) => e.to_string(),
        Ok(_) => unreachable!(),
    };
    assert!(
        err_str.contains("EOF while parsing a string") || err_str.contains("expected value at line") || err_str.contains("key must be a string"),
        "Oczekiwano błędu serializacji powiązanego z Serde, ale otrzymano: {}", err_str
    );
}

#[test]
fn test_truncated_partial_snapshot_fails_to_load() {
    let dir = tempdir().expect("tempdir");
    let mut store = FileSystemStore::new(dir.path());
    store.ensure_initialized().expect("init");

    let mut ledger = Ledger::open(
        store,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");
    let block1 = make_block(1, [0; 32], 10, ledger.snapshot());
    ledger.apply_block(&block1, 1, &test_epoch_params()).expect("apply block 1");
    ledger.flush().expect("flush");

    let path = dir.path().join("ledger-state.json");
    let content = std::fs::read(&path).expect("read file");
    
    // Ucinamy zrzut JSON-a na połowie jego oryginalnej długości, by zrzucić EOF error
    let truncated_content = &content[0..(content.len() / 2)];
    std::fs::write(&path, truncated_content).expect("truncate file");

    let store2 = FileSystemStore::new(dir.path());
    let ledger_err = Ledger::open(store2, privai_chain::DEFAULT_CHAIN_ID, StructuralProofVerifier);

    assert!(
        ledger_err.is_err(),
        "Ucięty plik zrzutu nie powinien zostać załadowany!"
    );
    let err_str = match ledger_err {
        Err(e) => e.to_string(),
        Ok(_) => unreachable!(),
    };
    assert!(
        err_str.contains("EOF"),
        "Oczekiwano błędu niespodziewanego EOF, ale otrzymano: {}", err_str
    );
}

#[test]
fn test_missing_snapshot_rebuilds_genesis() {
    let dir = tempdir().expect("tempdir");
    let mut store = FileSystemStore::new(dir.path());
    store.ensure_initialized().expect("init");

    // Delete file
    let path = dir.path().join("ledger-state.json");
    std::fs::remove_file(&path).expect("Usuwanie pliku");

    // Reopen ledger
    let store2 = FileSystemStore::new(dir.path());
    let ledger_reloaded = Ledger::open(store2, privai_chain::DEFAULT_CHAIN_ID, StructuralProofVerifier).expect("ledger reload on missing file");

    assert_eq!(
        ledger_reloaded.snapshot().height,
        0,
        "Skasowany zrzut powinien po udanym bootowaniu wstać z czystego Genesis"
    );
}

// -----------------------------------------------------------------------
// 8. RocksDB tests (Analogiczny pakiet testów dla RocksDBStore)
// -----------------------------------------------------------------------

#[test]
fn test_persist_state_then_restart_same_root_rocksdb() {
    let dir = tempdir().expect("tempdir");
    let store = privai_ledger::RocksDBStore::new(dir.path()).expect("new RocksDB");
    store.ensure_initialized().expect("init");

    let mut ledger = Ledger::open(store, privai_chain::DEFAULT_CHAIN_ID, StructuralProofVerifier).expect("ledger");

    let block1 = make_block(1, [0; 32], 10, ledger.snapshot());
    ledger.apply_block(&block1, 1, &test_epoch_params()).expect("apply block 1");

    let root_before = compute_state_root(ledger.snapshot());
    let height_before = ledger.snapshot().height;

    // Flush and Re-open
    ledger.flush().expect("flush");
    drop(ledger); // Drop to release RocksDB locks

    let store2 = privai_ledger::RocksDBStore::new(dir.path()).expect("re-open RocksDB");
    let ledger_reloaded = Ledger::open(store2, privai_chain::DEFAULT_CHAIN_ID, StructuralProofVerifier).expect("reload");

    let root_after = compute_state_root(ledger_reloaded.snapshot());

    assert_eq!(root_before, root_after, "state_root musi przetrwać restart (RocksDB)");
    assert_eq!(ledger_reloaded.snapshot().height, height_before);
}

#[test]
fn test_recover_tip_and_height_after_restart_rocksdb() {
    let dir = tempdir().expect("tempdir");
    let store = privai_ledger::RocksDBStore::new(dir.path()).expect("new RocksDB");
    store.ensure_initialized().expect("init");

    let mut ledger = Ledger::open(store, privai_chain::DEFAULT_CHAIN_ID, StructuralProofVerifier).expect("ledger");

    let mut prev = [0u8; 32];
    for i in 1..=3 {
        let block = make_block(i, prev, i as u8, ledger.snapshot());
        prev = block.hash();
        ledger.apply_block(&block, 1, &test_epoch_params()).expect("apply block");
    }

    let tip = ledger.snapshot().tip_hash;
    ledger.flush().expect("flush");
    drop(ledger); // Drop to release RocksDB locks

    let store2 = privai_ledger::RocksDBStore::new(dir.path()).expect("re-open RocksDB");
    let ledger2 = Ledger::open(store2, privai_chain::DEFAULT_CHAIN_ID, StructuralProofVerifier).expect("reload");

    assert_eq!(ledger2.snapshot().height, 3, "height musi zostać odzyskana (RocksDB)");
    assert_eq!(ledger2.snapshot().tip_hash, tip, "tip_hash musi zostać odzyskany (RocksDB)");
}

// -----------------------------------------------------------------------
// 9. persist_block_then_restart_then_continue_import (Opcjonalny)
// -----------------------------------------------------------------------

#[test]
fn test_persist_block_then_restart_then_continue_import_memory() {
    let mut ledger = Ledger::open(
        MemoryStore::new(),
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    // Zastosuj blok 1
    let block1 = make_block(1, [0; 32], 10, ledger.snapshot());
    ledger
        .apply_block(&block1, 1, &test_epoch_params())
        .expect("block 1");
    let h1 = block1.hash();

    // Restart
    let mut store2 = MemoryStore::new();
    store2
        .save(ledger.snapshot())
        .expect("save to memory store");
    let mut ledger2 = Ledger::open(
        store2,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("reload");

    // Zastosuj blok 2 po restarcie
    let block2 = make_block(2, h1, 20, ledger2.snapshot());
    ledger2
        .apply_block(&block2, 1, &test_epoch_params())
        .expect("block 2");

    assert_eq!(
        ledger2.snapshot().height,
        2,
        "chain rośnie prawidłowo po restarcie"
    );
}

#[test]
fn test_persist_block_then_restart_then_continue_import_filesystem() {
    let dir = tempdir().expect("tempdir");
    let mut store = FileSystemStore::new(dir.path());
    store.ensure_initialized().expect("init");

    let mut ledger = Ledger::open(
        store,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("ledger");

    let block1 = make_block(1, [0; 32], 10, ledger.snapshot());
    ledger
        .apply_block(&block1, 1, &test_epoch_params())
        .expect("block 1");
    let h1 = block1.hash();

    let store2 = FileSystemStore::new(dir.path());
    let mut ledger2 = Ledger::open(
        store2,
        privai_chain::DEFAULT_CHAIN_ID,
        StructuralProofVerifier,
    )
    .expect("reload");

    let block2 = make_block(2, h1, 20, ledger2.snapshot());
    ledger2
        .apply_block(&block2, 1, &test_epoch_params())
        .expect("block 2");

    assert_eq!(
        ledger2.snapshot().height,
        2,
        "chain rośnie prawidłowo po restarcie"
    );
}
