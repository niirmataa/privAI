use privai_chain::{
    merkle_root, Block, BlockTemplate, ExecutionBundle, ExecutionMode, InputRef, OutputNote,
    ProofCertificate, RecipientBox, Transaction, TransferNoteTx, TxCore, TX_TYPE_TRANSFER_NOTE,
};
use privai_proof::{ProofError, ProofVerifier, StructuralProofVerifier};

fn sample_note(seed: u8) -> OutputNote {
    OutputNote::new(
        [seed; 32],
        privai_chain::LweCiphertext::default(),
        [seed.wrapping_add(1); 32],
        RecipientBox::new(
            vec![seed, seed.wrapping_add(1)],
            [seed; 24],
            vec![seed.wrapping_add(2), seed.wrapping_add(3)],
            [seed; 16],
            [seed; 16],
        ),
    )
}

fn sample_transfer(seed: u8, statement_commit: [u8; 32]) -> Transaction {
    Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: 0,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef {
                note_commit: [seed.wrapping_add(30); 32],
            }],
            input_nullifiers: vec![privai_chain::Nullifier([seed.wrapping_add(40); 32])],
            outputs: vec![sample_note(seed)],
            fee: seed as u64,
            statement_commit,
            auth: Vec::new(),
        },
    })
}

fn build_block(
    txs: Vec<Transaction>,
    covered_tx_indexes: Vec<u32>,
    statement_commits: Vec<[u8; 32]>,
    public_inputs_root: [u8; 32],
    execution_mode: ExecutionMode,
    proof_certificates: Vec<ProofCertificate>,
) -> Block {
    Block::from_template(BlockTemplate {
        chain_id: 17,
        height: 1,
        epoch: 0,
        round: 0,
        timestamp_ms: 1_000,
        prev_block_hash: [0; 32],
        proposer_pk_hash: [1; 32],
        epoch_seed_hash: [2; 32],
        parent_qc_hash: [3; 32],
        txs,
        execution_bundle: ExecutionBundle {
            statement_commits,
            covered_tx_indexes,
            public_inputs_root,
            execution_mode,
        },
        proof_certificates,
        extra_receipts: Vec::new(),
    })
}

fn fully_covered_block(execution_mode: ExecutionMode) -> Block {
    let tx_a = sample_transfer(10, [11; 32]);
    let tx_b = sample_transfer(20, [22; 32]);
    let statement_commits = vec![tx_a.statement_commit(), tx_b.statement_commit()];
    let public_inputs_root = [5; 32];
    let statement_root = merkle_root(statement_commits.iter().copied());

    build_block(
        vec![tx_a, tx_b],
        vec![0, 1],
        statement_commits,
        public_inputs_root,
        execution_mode,
        vec![ProofCertificate {
            proof_system_id: 1,
            statement_root,
            public_inputs_root,
            proof_bytes_hash: [6; 32],
            prover_ids: vec![[8; 32], [9; 32]],
            proof_meta_hash: [7; 32],
        }],
    )
}

#[test]
fn accepts_fully_covered_full_batch_block() {
    let block = fully_covered_block(ExecutionMode::FullBatchProof);
    StructuralProofVerifier
        .verify_block(&block, 2)
        .expect("full batch proof block");
}

#[test]
fn accepts_fully_covered_multi_proof_block() {
    let block = fully_covered_block(ExecutionMode::MultiProofBundle);
    StructuralProofVerifier
        .verify_block(&block, 2)
        .expect("multi proof bundle block");
}

#[test]
fn rejects_duplicate_covered_index() {
    let mut block = fully_covered_block(ExecutionMode::FullBatchProof);
    block.body.execution_bundle.covered_tx_indexes = vec![0, 0];

    assert_eq!(
        StructuralProofVerifier.verify_block(&block, 2),
        Err(ProofError::DuplicateCoveredIndex(0))
    );
}

#[test]
fn rejects_invalid_covered_index() {
    let mut block = fully_covered_block(ExecutionMode::FullBatchProof);
    block.body.execution_bundle.covered_tx_indexes = vec![0, 2];

    assert_eq!(
        StructuralProofVerifier.verify_block(&block, 2),
        Err(ProofError::InvalidCoveredIndex(2))
    );
}

#[test]
fn rejects_statement_commit_mismatch() {
    let mut block = fully_covered_block(ExecutionMode::FullBatchProof);
    block.body.execution_bundle.statement_commits[1] = [0xAA; 32];

    assert_eq!(
        StructuralProofVerifier.verify_block(&block, 2),
        Err(ProofError::StatementCommitMismatch {
            tx_index: 1,
            expected: [22; 32],
            actual: [0xAA; 32],
        })
    );
}

#[test]
fn rejects_incomplete_coverage_even_if_minimum_is_met() {
    let mut block = fully_covered_block(ExecutionMode::FullBatchProof);
    block.body.execution_bundle.covered_tx_indexes = vec![0];
    block.body.execution_bundle.statement_commits = vec![block.body.txs[0].statement_commit()];
    let public_inputs_root = block.body.execution_bundle.public_inputs_root;
    let statement_root = merkle_root(block.body.execution_bundle.statement_commits.iter().copied());
    block.body.proof_certificates = vec![ProofCertificate {
        proof_system_id: 1,
        statement_root,
        public_inputs_root,
        proof_bytes_hash: [6; 32],
        prover_ids: vec![[8; 32]],
        proof_meta_hash: [7; 32],
    }];

    assert_eq!(
        StructuralProofVerifier.verify_block(&block, 1),
        Err(ProofError::IncompleteProofCoverage {
            expected: 2,
            actual: 1,
        })
    );
}

#[test]
fn rejects_insufficient_coverage_before_full_coverage() {
    let mut block = fully_covered_block(ExecutionMode::FullBatchProof);
    block.body.execution_bundle.covered_tx_indexes = vec![0];
    block.body.execution_bundle.statement_commits = vec![block.body.txs[0].statement_commit()];
    let public_inputs_root = block.body.execution_bundle.public_inputs_root;
    let statement_root = merkle_root(block.body.execution_bundle.statement_commits.iter().copied());
    block.body.proof_certificates = vec![ProofCertificate {
        proof_system_id: 1,
        statement_root,
        public_inputs_root,
        proof_bytes_hash: [6; 32],
        prover_ids: vec![[8; 32]],
        proof_meta_hash: [7; 32],
    }];

    assert_eq!(
        StructuralProofVerifier.verify_block(&block, 2),
        Err(ProofError::InsufficientProofCoverage {
            required: 2,
            actual: 1,
        })
    );
}

#[test]
fn rejects_certificate_statement_root_mismatch() {
    let mut block = fully_covered_block(ExecutionMode::FullBatchProof);
    block.body.proof_certificates[0].statement_root = [0x55; 32];

    assert_eq!(
        StructuralProofVerifier.verify_block(&block, 2),
        Err(ProofError::CertificateStatementRootMismatch)
    );
}

#[test]
fn rejects_certificate_public_inputs_root_mismatch() {
    let mut block = fully_covered_block(ExecutionMode::FullBatchProof);
    block.body.proof_certificates[0].public_inputs_root = [0x44; 32];

    assert_eq!(
        StructuralProofVerifier.verify_block(&block, 2),
        Err(ProofError::CertificatePublicInputsRootMismatch)
    );
}

#[test]
fn rejects_zero_proof_system_id() {
    let mut block = fully_covered_block(ExecutionMode::FullBatchProof);
    block.body.proof_certificates[0].proof_system_id = 0;

    assert_eq!(
        StructuralProofVerifier.verify_block(&block, 2),
        Err(ProofError::InvalidProofSystemId)
    );
}

#[test]
fn rejects_empty_prover_set() {
    let mut block = fully_covered_block(ExecutionMode::FullBatchProof);
    block.body.proof_certificates[0].prover_ids.clear();

    assert_eq!(
        StructuralProofVerifier.verify_block(&block, 2),
        Err(ProofError::EmptyProverSet)
    );
}
