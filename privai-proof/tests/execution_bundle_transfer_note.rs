use privai_chain::{
    merkle_root, Amount14, AuxWitness, CanonicalEncode, ExecutionMode, Hash32, InputRef,
    LweCiphertext, OutputNote, RecipientBox, RecipientBoxPlaintext, SpendPolicy, Transaction,
    TransferNoteTx, TxCore, PRIVAI_V0, TX_TYPE_TRANSFER_NOTE,
};
use privai_proof::{
    build_execution_bundle_from_transactions, build_execution_bundle_from_transfer_proofs,
    public_inputs_hash_for_transaction, BatchBuildError, TransferInputWitness,
    TransferOutputWitness, TransferProvingData, TransferStatement, TransferWitness,
};

// ------------------------------------------------------------------------------------------------
// Local Test Helpers
// ------------------------------------------------------------------------------------------------

fn sample_output(seed: u8) -> OutputNote {
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

fn sample_transfer(seed: u8, statement_commit: Hash32) -> Transaction {
    Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: PRIVAI_V0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef {
                note_commit: [seed.wrapping_add(30); 32],
            }],
            input_nullifiers: vec![privai_chain::Nullifier([seed.wrapping_add(40); 32])],
            outputs: vec![sample_output(seed)],
            fee: seed as u64,
            statement_commit,
            auth: Vec::new(),
        },
    })
}

fn sample_proving(seed: u8) -> TransferProvingData {
    let output = sample_output(seed);
    let statement = TransferStatement {
        input_note_commits: vec![[seed.wrapping_add(30); 32]],
        input_nullifiers: vec![privai_chain::Nullifier([seed.wrapping_add(40); 32])],
        output_note_commits: vec![output.note_commit],
        fee: seed as u64,
    };
    let tx = TransferNoteTx {
        core: TxCore {
            version: PRIVAI_V0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef {
                note_commit: [seed.wrapping_add(30); 32],
            }],
            input_nullifiers: vec![privai_chain::Nullifier([seed.wrapping_add(40); 32])],
            outputs: vec![output.clone()],
            fee: seed as u64,
            statement_commit: statement.commitment(),
            auth: Vec::new(),
        },
    };
    TransferProvingData::from_tx_and_witness(
        &tx,
        TransferWitness {
            input: TransferInputWitness {
                amount: Amount14::new(10).expect("amount"),
                witness_seed: [1; 32],
                nullifier_key: [2; 32],
                spend_policy_opening: vec![3],
                aux_opening: vec![4],
            },
            outputs: vec![TransferOutputWitness {
                note_commit: output.note_commit,
                recipient_opening: RecipientBoxPlaintext {
                    version: PRIVAI_V0,
                    bundle_id: output.recipient_box.hint,
                    note_payload_commit: output.payload_commit(),
                    amount: Amount14::new(10).expect("amount"),
                    witness_seed: [5; 32],
                    nullifier_key: [6; 32],
                    spend_policy_opening: SpendPolicy::Single {
                        falcon_pk_hash: [7; 32],
                    }
                    .to_canonical_bytes(),
                    aux_opening: AuxWitness {
                        version: PRIVAI_V0,
                        amount: Amount14::new(10).expect("amount"),
                        witness_seed: [5; 32],
                        noise_class: 1,
                        bundle_id: output.recipient_box.hint,
                    }
                    .to_canonical_bytes(),
                    sender_memo: None,
                },
            }],
        },
    )
    .expect("proving")
}

fn sample_housekeeping_tx() -> Transaction {
    // MarketplaceBatchTx nie wymaga proof_system'u (w obecnym build'zie). Zatem generuje housekeeping.
    use privai_chain::small_payments::SettlementBatchSummary;
    use privai_chain::tx::{MarketplaceBatchTx, TX_TYPE_MARKETPLACE_BATCH};
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
    Transaction::MarketplaceBatch(MarketplaceBatchTx {
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
        ticket_nullifiers: vec![privai_chain::Nullifier([0xcc; 32])],
        operator_sig: Vec::new(),
    })
}

// ------------------------------------------------------------------------------------------------
// 1. `empty_transaction_batch_produces_housekeeping_bundle`
// ------------------------------------------------------------------------------------------------
#[test]
fn empty_transaction_batch_produces_housekeeping_bundle() {
    let bundle = build_execution_bundle_from_transactions(&[], ExecutionMode::FullBatchProof)
        .expect("bundle");

    assert_eq!(
        bundle.execution_mode,
        ExecutionMode::Housekeeping,
        "Empty batch should enforce housekeeping mode"
    );
    assert!(bundle.statement_commits.is_empty());
    assert!(bundle.covered_tx_indexes.is_empty());
    assert_eq!(
        bundle.public_inputs_root,
        merkle_root(std::iter::empty::<Hash32>())
    );
}

// ------------------------------------------------------------------------------------------------
// 2. `transfer_note_batch_collects_statement_commits`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_note_batch_collects_statement_commits() {
    let tx_a = sample_transfer(10, [11; 32]);
    let tx_b = sample_transfer(20, [22; 32]);
    let txs = vec![tx_a.clone(), tx_b.clone()];

    let bundle =
        build_execution_bundle_from_transactions(&txs, ExecutionMode::FullBatchProof).unwrap();

    assert_eq!(
        bundle.statement_commits,
        vec![tx_a.statement_commit(), tx_b.statement_commit()]
    );
}

// ------------------------------------------------------------------------------------------------
// 3. `transfer_note_batch_collects_covered_tx_indexes`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_note_batch_collects_covered_tx_indexes() {
    let tx_proof_a = sample_transfer(10, [11; 32]);
    let tx_no_proof = sample_housekeeping_tx(); // tx bez proof requirement 
    let tx_proof_b = sample_transfer(20, [22; 32]);
    
    // Lista ma 3 elementy (0 - proof, 1 - brak proof, 2 - proof)
    let txs = vec![tx_proof_a, tx_no_proof, tx_proof_b];

    let bundle =
        build_execution_bundle_from_transactions(&txs, ExecutionMode::FullBatchProof).unwrap();

    assert_eq!(
        bundle.covered_tx_indexes,
        vec![0, 2],
        "Should only record index of proof-requiring txs"
    );
}

// ------------------------------------------------------------------------------------------------
// 4. `transfer_note_batch_derives_public_inputs_root_from_transfer_public_inputs_hashes`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_note_batch_derives_public_inputs_root_from_transfer_public_inputs_hashes() {
    let tx_a = sample_transfer(10, [11; 32]);
    let tx_b = sample_transfer(20, [22; 32]);
    let txs = vec![tx_a.clone(), tx_b.clone()];

    let bundle =
        build_execution_bundle_from_transactions(&txs, ExecutionMode::FullBatchProof).unwrap();

    let expected_root = merkle_root([
        public_inputs_hash_for_transaction(&tx_a).unwrap(),
        public_inputs_hash_for_transaction(&tx_b).unwrap(),
    ]);

    assert_eq!(bundle.public_inputs_root, expected_root);
}

// ------------------------------------------------------------------------------------------------
// 5. `transfer_proof_batch_matches_transaction_batch_for_transfer_note_path`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_proof_batch_matches_transaction_batch_for_transfer_note_path() {
    let proving_a = sample_proving(10);
    let proving_b = sample_proving(20);

    let tx_a = Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: PRIVAI_V0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef {
                note_commit: proving_a.statement.input_note_commits[0],
            }],
            input_nullifiers: proving_a.statement.input_nullifiers.clone(),
            outputs: vec![sample_output(10)],
            fee: proving_a.statement.fee,
            statement_commit: proving_a.statement.commitment(),
            auth: Vec::new(),
        },
    });
    let tx_b = Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: PRIVAI_V0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef {
                note_commit: proving_b.statement.input_note_commits[0],
            }],
            input_nullifiers: proving_b.statement.input_nullifiers.clone(),
            outputs: vec![sample_output(20)],
            fee: proving_b.statement.fee,
            statement_commit: proving_b.statement.commitment(),
            auth: Vec::new(),
        },
    });

    let from_txs =
        build_execution_bundle_from_transactions(&[tx_a, tx_b], ExecutionMode::FullBatchProof)
            .expect("tx bundle");
    let from_proofs = build_execution_bundle_from_transfer_proofs(
        &[proving_a, proving_b],
        ExecutionMode::FullBatchProof,
    )
    .expect("proof bundle");

    // Muszą być w pełni zgodne w ramach covered_indexes, commits oraz wliczonych roots
    assert_eq!(from_proofs.statement_commits, from_txs.statement_commits);
    assert_eq!(from_proofs.covered_tx_indexes, from_txs.covered_tx_indexes);
    assert_eq!(from_proofs.public_inputs_root, from_txs.public_inputs_root);
}

// ------------------------------------------------------------------------------------------------
// 6. `transfer_proof_batch_rejects_statement_commit_mismatch`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_proof_batch_rejects_statement_commit_mismatch() {
    let mut proving_data = sample_proving(10);
    
    // Psucie statement commit po stronie public inputs - nie zgadza się ze statement.commitment()
    proving_data.public_inputs.statement_commit = [0xAA; 32];

    let from_proofs_result = build_execution_bundle_from_transfer_proofs(
        &[proving_data],
        ExecutionMode::FullBatchProof,
    );

    assert!(
        matches!(from_proofs_result, Err(BatchBuildError::TransferProofStatementMismatch { index: 0, .. })),
        "Expected TransferProofStatementMismatch but got {:?}", from_proofs_result
    );
}

// ------------------------------------------------------------------------------------------------
// 7. `housekeeping_mode_is_forced_when_no_proof_requiring_txs_remain`
// ------------------------------------------------------------------------------------------------
#[test]
fn housekeeping_mode_is_forced_when_no_proof_requiring_txs_remain() {
    let tx_no_proof1 = sample_housekeeping_tx();
    let tx_no_proof2 = sample_housekeeping_tx();
    
    let txs = vec![tx_no_proof1, tx_no_proof2];

    // Chociaż rządamy FullBatchProof, mechanika bundle'a musi zdegradować nas do Housekeeping (brak proof-requiring txs).
    let bundle =
        build_execution_bundle_from_transactions(&txs, ExecutionMode::FullBatchProof).unwrap();

    assert_eq!(
        bundle.execution_mode,
        ExecutionMode::Housekeeping,
        "Bundle fallback to Housekeeping failed when array lacked any TransferNoteTx"
    );
    assert!(bundle.covered_tx_indexes.is_empty());
}
