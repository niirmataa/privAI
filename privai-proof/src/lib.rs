pub mod batch;
pub mod artifact;
pub mod halo2;
pub mod store;
pub mod transfer;
pub mod verify;

use std::collections::BTreeSet;

use privai_chain::{
    merkle_root, Block, ExecutionMode, Hash32, ProofCertificate,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use transfer::{
    LiteTransferStatement, TransferBuildError, TransferInputWitness, TransferOutputWitness,
    TransferProvingData, TransferPublicInputs, TransferStatement, TransferWitness,
};
pub use batch::{
    build_execution_bundle_from_transactions, build_execution_bundle_from_transfer_proofs,
    public_inputs_hash_for_transaction, BatchBuildError,
};
pub use verify::{
    ArtifactBackendError, ArtifactVerificationError, BatchProofVerifierBackend,
    BlockArtifactVerifier, ProofEnvelopeVerifier, SidecarProofVerifier,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofJob {
    pub job_id: Hash32,
    pub statement_commit: Hash32,
    pub job_fee: u64,
    pub deadline_height: u64,
    pub requester_hint: Hash32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofArtifact {
    pub job_id: Hash32,
    pub statement_commit: Hash32,
    pub proof_system_id: u8,
    pub proof_bytes: Vec<u8>,
    pub public_inputs_hash: Hash32,
    pub prover_pk_hash: Hash32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofRewardClaim {
    pub job_id: Hash32,
    pub proof_bytes_hash: Hash32,
    pub prover_pk: Vec<u8>,
    pub falcon_sig: Vec<u8>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProofError {
    #[error("housekeeping block must not carry user transactions")]
    HousekeepingCarriesTransactions,
    #[error("housekeeping block must not carry proof coverage")]
    HousekeepingCarriesProofCoverage,
    #[error("block with user transactions is missing proof coverage")]
    MissingProofCoverage,
    #[error("block with user transactions is missing proof certificates")]
    MissingProofCertificates,
    #[error("statement commits and covered tx indexes must have equal length")]
    CoverageLengthMismatch,
    #[error("covered tx index {0} appears more than once")]
    DuplicateCoveredIndex(u32),
    #[error("covered tx index {0} is outside the block body")]
    InvalidCoveredIndex(u32),
    #[error("covered tx index {tx_index} points to statement {actual:?}, expected {expected:?}")]
    StatementCommitMismatch {
        tx_index: u32,
        expected: Hash32,
        actual: Hash32,
    },
    #[error("proof coverage {actual} does not cover every tx in block ({expected})")]
    IncompleteProofCoverage { expected: usize, actual: usize },
    #[error("proof certificate must declare non-zero proof system id")]
    InvalidProofSystemId,
    #[error("proof certificate must include at least one prover id")]
    EmptyProverSet,
    #[error("proof certificate statement root does not match execution bundle")]
    CertificateStatementRootMismatch,
    #[error("proof certificate public inputs root does not match execution bundle")]
    CertificatePublicInputsRootMismatch,
}

pub trait ProofVerifier {
    fn verify_block(&self, block: &Block) -> Result<(), ProofError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StructuralProofVerifier;

impl StructuralProofVerifier {
    fn verify_housekeeping(&self, block: &Block) -> Result<(), ProofError> {
        if !block.body.txs.is_empty() {
            return Err(ProofError::HousekeepingCarriesTransactions);
        }
        if !block.body.execution_bundle.statement_commits.is_empty()
            || !block.body.execution_bundle.covered_tx_indexes.is_empty()
            || !block.body.proof_certificates.is_empty()
        {
            return Err(ProofError::HousekeepingCarriesProofCoverage);
        }
        Ok(())
    }

    fn verify_user_block(&self, block: &Block) -> Result<(), ProofError> {
        let execution_bundle = &block.body.execution_bundle;

        if execution_bundle.statement_commits.is_empty() || execution_bundle.covered_tx_indexes.is_empty() {
            return Err(ProofError::MissingProofCoverage);
        }
        if block.body.proof_certificates.is_empty() {
            return Err(ProofError::MissingProofCertificates);
        }
        if execution_bundle.statement_commits.len() != execution_bundle.covered_tx_indexes.len() {
            return Err(ProofError::CoverageLengthMismatch);
        }

        let mut seen_indexes = BTreeSet::new();
        for (statement_commit, tx_index) in execution_bundle
            .statement_commits
            .iter()
            .zip(execution_bundle.covered_tx_indexes.iter())
        {
            if !seen_indexes.insert(*tx_index) {
                return Err(ProofError::DuplicateCoveredIndex(*tx_index));
            }

            let Some(tx) = block.body.txs.get(*tx_index as usize) else {
                return Err(ProofError::InvalidCoveredIndex(*tx_index));
            };
            let expected_statement = tx.statement_commit();
            if *statement_commit != expected_statement {
                return Err(ProofError::StatementCommitMismatch {
                    tx_index: *tx_index,
                    expected: expected_statement,
                    actual: *statement_commit,
                });
            }
        }

        if seen_indexes.len() != block.body.txs.len() {
            return Err(ProofError::IncompleteProofCoverage {
                expected: block.body.txs.len(),
                actual: seen_indexes.len(),
            });
        }

        let statement_root = merkle_root(execution_bundle.statement_commits.iter().copied());
        for certificate in &block.body.proof_certificates {
            self.verify_certificate(certificate, statement_root, execution_bundle.public_inputs_root)?;
        }

        Ok(())
    }

    fn verify_certificate(
        &self,
        certificate: &ProofCertificate,
        statement_root: Hash32,
        public_inputs_root: Hash32,
    ) -> Result<(), ProofError> {
        if certificate.proof_system_id == 0 {
            return Err(ProofError::InvalidProofSystemId);
        }
        if certificate.prover_ids.is_empty() {
            return Err(ProofError::EmptyProverSet);
        }
        if certificate.statement_root != statement_root {
            return Err(ProofError::CertificateStatementRootMismatch);
        }
        if certificate.public_inputs_root != public_inputs_root {
            return Err(ProofError::CertificatePublicInputsRootMismatch);
        }
        Ok(())
    }
}

impl ProofVerifier for StructuralProofVerifier {
    fn verify_block(&self, block: &Block) -> Result<(), ProofError> {
        match block.body.execution_bundle.execution_mode {
            ExecutionMode::Housekeeping => self.verify_housekeeping(block),
            ExecutionMode::FullBatchProof | ExecutionMode::MultiProofBundle => {
                if block.body.txs.is_empty() {
                    return Err(ProofError::MissingProofCoverage);
                }
                self.verify_user_block(block)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use privai_chain::{
        Block, BlockTemplate, ExecutionBundle, InputRef, OutputNote, RecipientBox, Transaction,
        TransferNoteTx, TxCore, TX_TYPE_TRANSFER_NOTE,
    };

    use super::*;

    fn sample_note(seed: u8) -> OutputNote {
        OutputNote::new(
            [seed; 32],
            privai_chain::LweCiphertext::default(),
            [seed.wrapping_add(1); 32],
            RecipientBox::new(vec![seed], [seed; 24], vec![seed + 1], [seed; 16], [seed; 16]),
        )
    }

    fn sample_transfer() -> Transaction {
        Transaction::TransferNote(TransferNoteTx {
            core: TxCore {
                version: 0,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: vec![InputRef { note_commit: [9; 32] }],
                input_nullifiers: vec![privai_chain::Nullifier([7; 32])],
                outputs: vec![sample_note(10)],
                fee: 3,
                statement_commit: [11; 32],
                auth: Vec::new(),
            },
        })
    }

    fn sample_block() -> Block {
        let tx = sample_transfer();
        let execution_bundle = ExecutionBundle {
            statement_commits: vec![tx.statement_commit()],
            covered_tx_indexes: vec![0],
            public_inputs_root: [5; 32],
            execution_mode: ExecutionMode::FullBatchProof,
        };
        let statement_root = merkle_root(execution_bundle.statement_commits.iter().copied());

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
            state_root: [0; 32],
            txs: vec![tx],
            execution_bundle: execution_bundle.clone(),
            proof_certificates: vec![ProofCertificate {
                proof_system_id: 1,
                statement_root,
                public_inputs_root: execution_bundle.public_inputs_root,
                proof_bytes_hash: [6; 32],
                prover_ids: vec![[8; 32]],
                proof_meta_hash: [9; 32],
            }],
            extra_receipts: Vec::new(),
        })
    }

    #[test]
    fn structural_verifier_accepts_fully_covered_block() {
        let block = sample_block();
        StructuralProofVerifier
            .verify_block(&block)
            .expect("proof-carrying block");
    }

    #[test]
    fn structural_verifier_rejects_missing_certificates() {
        let mut block = sample_block();
        block.body.proof_certificates.clear();

        assert_eq!(
            StructuralProofVerifier.verify_block(&block),
            Err(ProofError::MissingProofCertificates)
        );
    }

    #[test]
    fn housekeeping_requires_empty_body() {
        let mut block = sample_block();
        block.body.execution_bundle.execution_mode = ExecutionMode::Housekeeping;

        assert_eq!(
            StructuralProofVerifier.verify_block(&block),
            Err(ProofError::HousekeepingCarriesTransactions)
        );
    }
}
