use privai_chain::{
    merkle_root, Block, BlockTemplate, ExecutionBundle, ExecutionMode, Hash32, OutputNote,
    ProofCertificate, RecipientBox, Transaction, TransferNoteTx, TxCore, TX_TYPE_TRANSFER_NOTE,
};
use privai_proof::StructuralProofVerifier;

use privai_ledger::{
    apply_transaction_local, compute_state_root,
    state::{LedgerSnapshot, NoteRecord, NoteStatus},
    store::{LedgerStore, MemoryStore},
    Ledger, LedgerError, ValidationError,
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
// 1. Snapshot z height > 0, ale bez blocks[height] 
// -----------------------------------------------------------------------
#[test]
fn test_open_with_missing_block_in_cache() {
    let mut store = MemoryStore::new();
    let mut snapshot = LedgerSnapshot::genesis(privai_chain::DEFAULT_CHAIN_ID);
    snapshot.height = 5;
    snapshot.tip_hash = [0x55; 32];
    
    // Umyślnie brak wpisu w `snapshot.blocks` pod kluczem 5.
    store.save(&snapshot).expect("save");

    // Zgodnie z zachowaniem Ledger::open() obecny wariant omija błąd, jeśli nie ma bloku w tablicy
    // w locie by zwalidować root. Oczekujemy, że node otworzy się bez faila!
    let ledger = Ledger::open(store, privai_chain::DEFAULT_CHAIN_ID, StructuralProofVerifier);
    assert!(ledger.is_ok(), "Ledger should open successfully when block cache is evicted, skipping root verification");
    assert_eq!(ledger.unwrap().snapshot().height, 5);
}

// -----------------------------------------------------------------------
// 2. Snapshot z niespójnym state_root (ale obecnym w cache blokiem)
// -----------------------------------------------------------------------
#[test]
fn test_open_with_mismatched_state_root_fails_fast() {
    let mut store = MemoryStore::new();
    let mut snapshot = LedgerSnapshot::genesis(privai_chain::DEFAULT_CHAIN_ID);
    
    let block = make_block(1, [0; 32], 10, &snapshot);
    
    // Celowe popsucie snapshotu by wymusić zły computed root przy wczytaniu vs `block.header.state_root`
    snapshot.height = 1;
    snapshot.tip_hash = block.hash();
    snapshot.blocks.insert(1, block.clone());
    
    let wrong_note = sample_note(99);
    snapshot.notes.insert(
        wrong_note.note_commit,
        NoteRecord {
            note: wrong_note,
            created_in_block: Some(1),
            status: NoteStatus::Unspent,
        },
    );
    
    store.save(&snapshot).expect("save");

    let ledger_err = Ledger::open(store, privai_chain::DEFAULT_CHAIN_ID, StructuralProofVerifier);
    
    assert!(ledger_err.is_err(), "Expected Ledger::open to fail due to StateRootMismatch");
    match ledger_err {
        Err(LedgerError::Validation(ValidationError::StateRootMismatch { .. })) => {}
        err => panic!("Oczekiwano StateRootMismatch, ale otrzymano inna pule / pass"),
    }
}

// -----------------------------------------------------------------------
// 3. Snapshot z niespójnym tip_hash
// -----------------------------------------------------------------------
#[test]
fn test_open_with_mismatched_tip_hash_is_loadable() {
    let mut store = MemoryStore::new();
    let mut snapshot = LedgerSnapshot::genesis(privai_chain::DEFAULT_CHAIN_ID);
    
    let block = make_block(1, [0; 32], 10, &snapshot);
    
    // Zapisz prawidłowy stan węzła...
    let mut temp = snapshot.clone();
    apply_transaction_local(&block.body.txs[0], 1, &mut temp);
    
    snapshot.height = 1;
    snapshot.notes = temp.notes; 
    snapshot.blocks.insert(1, block.clone());
    
    // ...ale psujemy tip_hash!
    snapshot.tip_hash = [0xff; 32];
    
    // Snapshot w takim stanie nie wyzwala żadnego checka w `Ledger::open()`.
    store.save(&snapshot).expect("save");

    let ledger = Ledger::open(store, privai_chain::DEFAULT_CHAIN_ID, StructuralProofVerifier);
    
    assert!(ledger.is_ok(), "Obecny kod nie weryfikuje tip_hash przy starcie (nie odrzuca zepsutego)");
    let reloaded = ledger.unwrap();
    assert_eq!(reloaded.snapshot().tip_hash, [0xff; 32]);
    assert_eq!(reloaded.snapshot().height, 1);
}

// -----------------------------------------------------------------------
// 4. Pusty store / genesis fallback
// -----------------------------------------------------------------------
#[test]
fn test_open_empty_store_fallback_to_genesis() {
    let store = MemoryStore::new();
    let ledger = Ledger::open(store, privai_chain::DEFAULT_CHAIN_ID, StructuralProofVerifier);
    
    assert!(ledger.is_ok(), "Ledger should fallback to Genesis");
    let unwrapped = ledger.unwrap();
    
    assert_eq!(unwrapped.snapshot().height, 0, "Genesis height should be 0");
    assert_eq!(unwrapped.snapshot().tip_hash, [0; 32], "Genesis tip_hash should be zeroed");
    assert!(unwrapped.snapshot().notes.is_empty(), "Genesis notes should be empty");
}

// -----------------------------------------------------------------------
// 5. Snapshot częściowy, ale ciągle ładowalny
// -----------------------------------------------------------------------
#[test]
fn test_open_partial_snapshot_still_loadable() {
    let mut store = MemoryStore::new();
    let mut snapshot = LedgerSnapshot::genesis(privai_chain::DEFAULT_CHAIN_ID);
    
    let note = sample_note(50);
    
    // Ten snapshot wybitnie nie posiada zdefiniowanego bloku, ma height 99 ale historycznie jest po prostu zmutowanym, gołym drzewem.
    snapshot.height = 99;
    snapshot.tip_hash = [0x99; 32];
    snapshot.notes.insert(
        note.note_commit,
        NoteRecord {
            note,
            created_in_block: Some(99),
            status: NoteStatus::Unspent,
        },
    );
    
    store.save(&snapshot).expect("save");

    // Z uwagi na fakt, że Ledger::open() nie pyta czy `blocks.get(&height)` MUSI istnieć (robi to tylko na Some by sprawdzić root),
    // system bez problemu przełyka tak poszarpany stan zrzucony brutalnie w pamięć, akceptując notę z kapelusza bez headerów z udowodnionym rootem.
    let ledger = Ledger::open(store, privai_chain::DEFAULT_CHAIN_ID, StructuralProofVerifier);
    assert!(ledger.is_ok(), "Dziurawy / nielogiczny snapshot bez pełnej historii, o ile nie ma bloku na swej wysokości, omija check w locie i wstaje.");
    
    let unwrapped = ledger.unwrap();
    assert_eq!(unwrapped.snapshot().height, 99);
    assert_eq!(unwrapped.snapshot().notes.len(), 1);
}
