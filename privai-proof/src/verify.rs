use std::collections::BTreeMap;

use privai_chain::{Block, ExecutionBundle};
use thiserror::Error;

use crate::{
    artifact::{ArtifactError, BatchProofArtifact, BlockProofArtifacts, BlockProofEntry},
    ProofError, ProofVerifier, StructuralProofVerifier,
};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ArtifactBackendError {
    #[error("proof artifact must carry non-empty proof bytes")]
    EmptyProofBytes,
    #[error("proof artifact must cover at least one tx index")]
    EmptyCoverage,
    #[error("{0}")]
    Rejected(String),
}

pub trait BatchProofVerifierBackend {
    fn verify_batch_artifact(
        &self,
        artifact: &BatchProofArtifact,
        covered_entries: &[BlockProofEntry],
        execution_bundle: &ExecutionBundle,
    ) -> Result<(), ArtifactBackendError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ProofEnvelopeVerifier;

impl BatchProofVerifierBackend for ProofEnvelopeVerifier {
    fn verify_batch_artifact(
        &self,
        artifact: &BatchProofArtifact,
        covered_entries: &[BlockProofEntry],
        _execution_bundle: &ExecutionBundle,
    ) -> Result<(), ArtifactBackendError> {
        if artifact.proof_bytes.is_empty() {
            return Err(ArtifactBackendError::EmptyProofBytes);
        }
        if covered_entries.is_empty() {
            return Err(ArtifactBackendError::EmptyCoverage);
        }
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ArtifactVerificationError {
    #[error(transparent)]
    Structural(#[from] ProofError),
    #[error(transparent)]
    Artifact(#[from] ArtifactError),
    #[error("artifact verifier could not resolve tx index {tx_index} in sidecar entries")]
    MissingEntry { tx_index: u32 },
    #[error("proof backend rejected artifact for proof system {proof_system_id}: {error}")]
    Backend {
        proof_system_id: u8,
        error: ArtifactBackendError,
    },
}

pub trait BlockArtifactVerifier {
    fn verify_block_artifacts(
        &self,
        block: &Block,
        artifacts: &BlockProofArtifacts,
        min_proof_coverage: u32,
    ) -> Result<(), ArtifactVerificationError>;
}

#[derive(Clone, Debug, Default)]
pub struct SidecarProofVerifier<B = ProofEnvelopeVerifier> {
    structural: StructuralProofVerifier,
    backend: B,
}

impl<B> SidecarProofVerifier<B> {
    pub fn new(backend: B) -> Self {
        Self {
            structural: StructuralProofVerifier,
            backend,
        }
    }
}

impl<B: Default> SidecarProofVerifier<B> {
    pub fn with_default_backend() -> Self {
        Self::new(B::default())
    }
}

impl<B: BatchProofVerifierBackend> BlockArtifactVerifier for SidecarProofVerifier<B> {
    fn verify_block_artifacts(
        &self,
        block: &Block,
        artifacts: &BlockProofArtifacts,
        min_proof_coverage: u32,
    ) -> Result<(), ArtifactVerificationError> {
        self.structural.verify_block(block, min_proof_coverage)?;
        artifacts.validate_against_block(block)?;

        let entries_by_index = artifacts
            .entries
            .iter()
            .cloned()
            .map(|entry| (entry.tx_index, entry))
            .collect::<BTreeMap<_, _>>();

        for artifact in &artifacts.artifacts {
            let mut covered_entries = Vec::with_capacity(artifact.covered_tx_indexes.len());
            for tx_index in &artifact.covered_tx_indexes {
                let Some(entry) = entries_by_index.get(tx_index) else {
                    return Err(ArtifactVerificationError::MissingEntry { tx_index: *tx_index });
                };
                covered_entries.push(entry.clone());
            }

            self.backend
                .verify_batch_artifact(artifact, &covered_entries, &artifacts.execution_bundle)
                .map_err(|error| ArtifactVerificationError::Backend {
                    proof_system_id: artifact.proof_system_id,
                    error,
                })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use privai_chain::{
        merkle_root, Amount14, AuxWitness, CanonicalEncode, InputRef, LweCiphertext, OutputNote,
        RecipientBox, RecipientBoxPlaintext, SpendPolicy, Transaction, TransferNoteTx, TxCore,
        TX_TYPE_TRANSFER_NOTE, PRIVAI_V0,
    };

    use super::*;
    use crate::{
        artifact::{BatchProofArtifact, BlockProofArtifacts},
        build_execution_bundle_from_transfer_proofs, TransferInputWitness, TransferOutputWitness,
        TransferProvingData, TransferStatement, TransferWitness,
    };

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

    fn sample_tx_and_proving(seed: u8) -> (Transaction, TransferProvingData) {
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
        let proving = TransferProvingData::from_tx_and_witness(
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
        .expect("proving");

        (Transaction::TransferNote(tx), proving)
    }

    fn sample_block_and_artifacts() -> (Block, BlockProofArtifacts) {
        let (tx, proving) = sample_tx_and_proving(10);
        let execution_bundle = build_execution_bundle_from_transfer_proofs(
            std::slice::from_ref(&proving),
            privai_chain::ExecutionMode::FullBatchProof,
        )
        .expect("bundle");
        let artifact = BatchProofArtifact {
            proof_system_id: 1,
            statement_root: merkle_root(execution_bundle.statement_commits.iter().copied()),
            public_inputs_root: execution_bundle.public_inputs_root,
            covered_tx_indexes: vec![0],
            proof_bytes: vec![1, 2, 3],
            prover_ids: vec![[9; 32]],
            proof_meta_hash: [8; 32],
        };
        let block = Block::from_template(privai_chain::BlockTemplate {
            chain_id: 17,
            height: 1,
            epoch: 0,
            round: 0,
            timestamp_ms: 1_000,
            prev_block_hash: [0; 32],
            proposer_pk_hash: [1; 32],
            epoch_seed_hash: [2; 32],
            parent_qc_hash: [3; 32],
            txs: vec![tx],
            execution_bundle: execution_bundle.clone(),
            proof_certificates: vec![artifact.certificate()],
            extra_receipts: Vec::new(),
        });
        let artifacts = BlockProofArtifacts::from_transfer_proofs(
            block.hash(),
            &[proving],
            vec![artifact],
        )
        .expect("artifacts");

        (block, artifacts)
    }

    #[test]
    fn sidecar_verifier_accepts_valid_block_artifacts() {
        let (block, artifacts) = sample_block_and_artifacts();
        SidecarProofVerifier::<ProofEnvelopeVerifier>::default()
            .verify_block_artifacts(&block, &artifacts, 1)
            .expect("verify");
    }

    #[test]
    fn sidecar_verifier_rejects_empty_proof_bytes() {
        let (block, mut artifacts) = sample_block_and_artifacts();
        artifacts.artifacts[0].proof_bytes.clear();
        artifacts.artifacts[0].covered_tx_indexes = vec![0];
        block
            .body
            .proof_certificates
            .get(0)
            .expect("certificate");
        assert!(matches!(
            SidecarProofVerifier::<ProofEnvelopeVerifier>::default()
                .verify_block_artifacts(&block, &artifacts, 1),
            Err(ArtifactVerificationError::Artifact(ArtifactError::ProofCertificateMismatch))
                | Err(ArtifactVerificationError::Backend {
                    error: ArtifactBackendError::EmptyProofBytes,
                    ..
                })
        ));
    }

    #[derive(Clone, Copy, Debug, Default)]
    struct RejectingBackend;

    impl BatchProofVerifierBackend for RejectingBackend {
        fn verify_batch_artifact(
            &self,
            _artifact: &BatchProofArtifact,
            _covered_entries: &[BlockProofEntry],
            _execution_bundle: &ExecutionBundle,
        ) -> Result<(), ArtifactBackendError> {
            Err(ArtifactBackendError::Rejected("forced rejection".into()))
        }
    }

    #[test]
    fn sidecar_verifier_surfaces_backend_rejection() {
        let (block, artifacts) = sample_block_and_artifacts();
        let verifier = SidecarProofVerifier::new(RejectingBackend);
        assert!(matches!(
            verifier.verify_block_artifacts(&block, &artifacts, 1),
            Err(ArtifactVerificationError::Backend {
                error: ArtifactBackendError::Rejected(message),
                ..
            }) if message == "forced rejection"
        ));
    }
}
