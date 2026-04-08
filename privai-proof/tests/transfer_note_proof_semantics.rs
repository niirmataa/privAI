use privai_chain::{
    Amount14, InputRef, LweCiphertext, OutputNote, RecipientBox, RecipientBoxPlaintext,
    Transaction, TransferNoteTx, TxCore, PRIVAI_V0, TX_TYPE_TRANSFER_NOTE,
};
use privai_proof::{
    TransferBuildError, TransferInputWitness, TransferOutputWitness, TransferProvingData,
    TransferPublicInputs, TransferStatement, TransferWitness,
};

/// Zwraca bazowy (valid) przykład TxCore, który następnie jest mutowany w poszczególnych testach.
fn build_valid_transfer_core(seed: u8) -> TxCore {
    let output = sample_note(seed);
    let statement = TransferStatement {
        input_note_commits: vec![[seed.wrapping_add(30); 32]],
        input_nullifiers: vec![privai_chain::Nullifier([seed.wrapping_add(40); 32])],
        output_note_commits: vec![output.note_commit],
        fee: seed as u64,
    };
    
    TxCore {
        version: PRIVAI_V0,
        tx_type: TX_TYPE_TRANSFER_NOTE,
        inputs: vec![InputRef {
            note_commit: [seed.wrapping_add(30); 32],
        }],
        input_nullifiers: vec![privai_chain::Nullifier([seed.wrapping_add(40); 32])],
        outputs: vec![output],
        fee: seed as u64,
        statement_commit: statement.commitment(),
        auth: Vec::new(),
    }
}

/// Zwraca zaufany OutputNote, identyczny z wariantem z privai-proof/tests
fn sample_note(seed: u8) -> OutputNote {
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

/// Helper, zwracający kompletny pakiet valid: transakcja i odpowiadający jej poprawny Witness
fn sample_tx_and_witness() -> (TransferNoteTx, TransferWitness) {
    let core = build_valid_transfer_core(10);
    let output = core.outputs[0].clone();
    let tx = TransferNoteTx { core };

    let witness = TransferWitness {
        input: TransferInputWitness {
            amount: Amount14::new(42).expect("amount"),
            witness_seed: [1; 32],
            nullifier_key: [2; 32],
            spend_policy_opening: vec![3, 4],
            aux_opening: vec![5, 6],
        },
        outputs: vec![TransferOutputWitness {
            note_commit: output.note_commit,
            recipient_opening: RecipientBoxPlaintext {
                version: PRIVAI_V0,
                bundle_id: output.recipient_box.hint,
                note_payload_commit: output.payload_commit(),
                amount: Amount14::new(21).expect("amount"),
                witness_seed: [8; 32],
                nullifier_key: [9; 32],
                spend_policy_opening: vec![10],
                aux_opening: vec![11],
                sender_memo: None,
            },
        }],
    };
    (tx, witness)
}

// ------------------------------------------------------------------------------------------------
// 1. `transfer_statement_from_tx_matches_tx_core_fields`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_statement_from_tx_matches_tx_core_fields() {
    let (tx, _) = sample_tx_and_witness();
    let statement = TransferStatement::from_tx(&tx);

    assert_eq!(statement.input_note_commits.len(), tx.core.inputs.len());
    assert_eq!(
        statement.input_note_commits[0],
        tx.core.inputs[0].note_commit
    );
    assert_eq!(statement.input_nullifiers, tx.core.input_nullifiers);
    assert_eq!(statement.output_note_commits.len(), tx.core.outputs.len());
    assert_eq!(
        statement.output_note_commits[0],
        tx.core.outputs[0].note_commit
    );
    assert_eq!(statement.fee, tx.core.fee);
}

// ------------------------------------------------------------------------------------------------
// 2. `transfer_statement_commit_is_stable_for_fixed_tx`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_statement_commit_is_stable_for_fixed_tx() {
    let (tx, _) = sample_tx_and_witness();
    let statement1 = TransferStatement::from_tx(&tx);
    let commit1 = statement1.commitment();

    let statement2 = TransferStatement::from_tx(&tx);
    let commit2 = statement2.commitment();

    assert_eq!(commit1, commit2, "Commitment is non-deterministic for the same tx fields");
    
    // Potwierdzenie stabilnej struktury bez ukrytych mutacji w to_canonical_bytes
    assert_eq!(statement1.to_canonical_bytes(), statement2.to_canonical_bytes());
}

// ------------------------------------------------------------------------------------------------
// 3. `transfer_public_inputs_from_tx_match_tx_core_fields`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_public_inputs_from_tx_match_tx_core_fields() {
    let (tx, _) = sample_tx_and_witness();
    let public_inputs = TransferPublicInputs::from_tx(&tx);

    // TX_ID depends on the complete transaction wrapping via privai_chain
    assert_eq!(public_inputs.tx_id, Transaction::TransferNote(tx.clone()).tx_id());
    
    assert_eq!(public_inputs.statement_commit, tx.core.statement_commit);
    assert_eq!(
        public_inputs.input_note_commits[0],
        tx.core.inputs[0].note_commit
    );
    assert_eq!(public_inputs.input_nullifiers, tx.core.input_nullifiers);
    assert_eq!(
        public_inputs.output_note_commits[0],
        tx.core.outputs[0].note_commit
    );
    assert_eq!(public_inputs.fee, tx.core.fee);
}

// ------------------------------------------------------------------------------------------------
// 4. `transfer_public_inputs_hash_is_stable_for_fixed_tx`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_public_inputs_hash_is_stable_for_fixed_tx() {
    let (tx, _) = sample_tx_and_witness();
    let pi1 = TransferPublicInputs::from_tx(&tx);
    let pi2 = TransferPublicInputs::from_tx(&tx);

    assert_eq!(pi1.hash(), pi2.hash(), "Public Inputs hash must be deterministic");
}

// ------------------------------------------------------------------------------------------------
// 5. `transfer_proving_data_accepts_matching_tx_and_witness`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_proving_data_accepts_matching_tx_and_witness() {
    let (tx, witness) = sample_tx_and_witness();
    
    let proving_result = TransferProvingData::from_tx_and_witness(&tx, witness);
    assert!(
        proving_result.is_ok(),
        "Matching tx and witness failed validation: {:?}",
        proving_result.err()
    );
    
    let proving_data = proving_result.unwrap();
    // Validate mapping sanity check
    assert_eq!(proving_data.statement.fee, tx.core.fee);
    assert_eq!(proving_data.public_inputs.statement_commit, tx.core.statement_commit);
}

// ------------------------------------------------------------------------------------------------
// 6. `transfer_proving_data_rejects_statement_commit_mismatch`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_proving_data_rejects_statement_commit_mismatch() {
    let (mut tx, witness) = sample_tx_and_witness();
    
    // Psucie wyliczonego wcześnie statement_commit (celowy mismatch na core layer)
    tx.core.statement_commit = [0xAA; 32];
    
    let proving_result = TransferProvingData::from_tx_and_witness(&tx, witness);
    assert!(
        matches!(proving_result, Err(TransferBuildError::StatementCommitMismatch { .. })),
        "Expected StatementCommitMismatch but got {:?}", proving_result
    );
}

// ------------------------------------------------------------------------------------------------
// 7. `transfer_proving_data_rejects_output_witness_count_mismatch`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_proving_data_rejects_output_witness_count_mismatch() {
    let (tx, mut witness) = sample_tx_and_witness();
    
    // Obcinamy listę witnessów aby wymusić niezgodność z `tx.core.outputs.len()`
    witness.outputs.clear();
    
    let proving_result = TransferProvingData::from_tx_and_witness(&tx, witness);
    assert!(
        matches!(proving_result, Err(TransferBuildError::OutputWitnessCountMismatch { .. })),
        "Expected OutputWitnessCountMismatch but got {:?}", proving_result
    );
}

// ------------------------------------------------------------------------------------------------
// 8. `transfer_proving_data_rejects_output_note_commit_mismatch`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_proving_data_rejects_output_note_commit_mismatch() {
    let (tx, mut witness) = sample_tx_and_witness();
    
    // Psucie note_commit z punktu widzenia witnessa na danym output element
    witness.outputs[0].note_commit = [0xBB; 32];
    
    let proving_result = TransferProvingData::from_tx_and_witness(&tx, witness);
    assert!(
        matches!(proving_result, Err(TransferBuildError::OutputNoteCommitMismatch { index: 0, .. })),
        "Expected OutputNoteCommitMismatch but got {:?}", proving_result
    );
}

// ------------------------------------------------------------------------------------------------
// 9. `transfer_proving_data_rejects_output_payload_commit_mismatch`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_proving_data_rejects_output_payload_commit_mismatch() {
    let (tx, mut witness) = sample_tx_and_witness();
    
    // Psucie payload_commit na poziomie `recipient_opening` witnessa
    witness.outputs[0].recipient_opening.note_payload_commit = [0xCC; 32];
    
    let proving_result = TransferProvingData::from_tx_and_witness(&tx, witness);
    assert!(
        matches!(proving_result, Err(TransferBuildError::OutputPayloadCommitMismatch { index: 0 })),
        "Expected OutputPayloadCommitMismatch but got {:?}", proving_result
    );
}

// ------------------------------------------------------------------------------------------------
// 10. `transfer_proving_data_rejects_output_bundle_hint_mismatch`
// ------------------------------------------------------------------------------------------------
#[test]
fn transfer_proving_data_rejects_output_bundle_hint_mismatch() {
    let (tx, mut witness) = sample_tx_and_witness();
    
    // Psucie bundle_id (musi zgadzać się z note.recipient_box.hint transakcji - 16 bytes!)
    witness.outputs[0].recipient_opening.bundle_id = [0xDD; 16];
    
    let proving_result = TransferProvingData::from_tx_and_witness(&tx, witness);
    assert!(
        matches!(proving_result, Err(TransferBuildError::OutputBundleHintMismatch { index: 0 })),
        "Expected OutputBundleHintMismatch but got {:?}", proving_result
    );
}
