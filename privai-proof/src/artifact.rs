#![allow(unused)]
// TODO(privai): remove this once the Alpine musl Rust toolchain stops ICE'ing
// while rendering warnings in this module during cargo-driven builds.

use std::collections::BTreeSet;

use blake3::Hasher;
use privai_chain::{merkle_root, Block, ExecutionBundle, Hash32, ProofCertificate};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{build_execution_bundle_from_transfer_proofs, BatchBuildError, TransferProvingData};

const PROOF_BYTES_HASH_DOMAIN_V0: &[u8] = b"privai:proof:artifact-bytes:v0";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockProofEntry {
    pub tx_index: u32,
    pub statement_commit: Hash32,
    pub public_inputs_hash: Hash32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchProofArtifact {
    pub proof_system_id: u8,
    pub statement_root: Hash32,
    pub public_inputs_root: Hash32,
    pub covered_tx_indexes: Vec<u32>,
    pub proof_bytes: Vec<u8>,
    pub prover_ids: Vec<Hash32>,
    pub proof_meta_hash: Hash32,
}

impl BatchProofArtifact {
    pub fn proof_bytes_hash(&self) -> Hash32 {
        let mut hasher = Hasher::new();
        write_len_prefixed(&mut hasher, PROOF_BYTES_HASH_DOMAIN_V0);
        write_len_prefixed(&mut hasher, &self.proof_bytes);
        *hasher.finalize().as_bytes()
    }

    pub fn certificate(&self) -> ProofCertificate {
        ProofCertificate {
            proof_system_id: self.proof_system_id,
            statement_root: self.statement_root,
            public_inputs_root: self.public_inputs_root,
            proof_bytes_hash: self.proof_bytes_hash(),
            prover_ids: self.prover_ids.clone(),
            proof_meta_hash: self.proof_meta_hash,
        }
    }
}

fn write_len_prefixed(hasher: &mut Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u32).to_le_bytes());
    hasher.update(bytes);
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockProofArtifacts {
    pub block_hash: Hash32,
    pub execution_bundle: ExecutionBundle,
    pub entries: Vec<BlockProofEntry>,
    pub artifacts: Vec<BatchProofArtifact>,
}

impl BlockProofArtifacts {
    pub fn from_transfer_proofs(
        block_hash: Hash32,
        proving_data: &[TransferProvingData],
        artifacts: Vec<BatchProofArtifact>,
    ) -> Result<Self, ArtifactError> {
        let execution_bundle = build_execution_bundle_from_transfer_proofs(
            proving_data,
            if proving_data.is_empty() {
                privai_chain::ExecutionMode::Housekeeping
            } else {
                privai_chain::ExecutionMode::FullBatchProof
            },
        )?;
        let entries = proving_data
            .iter()
            .enumerate()
            .map(|(index, proving)| BlockProofEntry {
                tx_index: index as u32,
                statement_commit: proving.statement.commitment(),
                public_inputs_hash: proving.public_inputs_hash(),
            })
            .collect();

        let bundle = Self {
            block_hash,
            execution_bundle,
            entries,
            artifacts,
        };
        bundle.validate()?;
        Ok(bundle)
    }

    pub fn statement_root(&self) -> Hash32 {
        merkle_root(self.execution_bundle.statement_commits.iter().copied())
    }

    pub fn public_inputs_root(&self) -> Hash32 {
        merkle_root(self.entries.iter().map(|entry| entry.public_inputs_hash))
    }

    pub fn proof_certificates(&self) -> Vec<ProofCertificate> {
        self.artifacts
            .iter()
            .map(BatchProofArtifact::certificate)
            .collect()
    }

    pub fn validate_against_block(&self, block: &Block) -> Result<(), ArtifactError> {
        self.validate()?;

        let expected_block_hash = block.hash();
        if self.block_hash != expected_block_hash {
            return Err(ArtifactError::BlockHashMismatch {
                expected: expected_block_hash,
                actual: self.block_hash,
            });
        }

        if self.execution_bundle != block.body.execution_bundle {
            return Err(ArtifactError::ExecutionBundleMismatch);
        }

        if self.proof_certificates() != block.body.proof_certificates {
            return Err(ArtifactError::ProofCertificateMismatch);
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<(), ArtifactError> {
        if self.entries.len() != self.execution_bundle.statement_commits.len()
            || self.entries.len() != self.execution_bundle.covered_tx_indexes.len()
        {
            return Err(ArtifactError::EntryLengthMismatch {
                expected: self.execution_bundle.statement_commits.len(),
                actual: self.entries.len(),
            });
        }

        let mut seen_entries = BTreeSet::new();
        for (index, entry) in self.entries.iter().enumerate() {
            let expected_tx_index = self.execution_bundle.covered_tx_indexes[index];
            let expected_statement_commit = self.execution_bundle.statement_commits[index];
            if entry.tx_index != expected_tx_index {
                return Err(ArtifactError::EntryIndexMismatch {
                    index,
                    expected: expected_tx_index,
                    actual: entry.tx_index,
                });
            }
            if entry.statement_commit != expected_statement_commit {
                return Err(ArtifactError::EntryStatementMismatch {
                    index,
                    expected: expected_statement_commit,
                    actual: entry.statement_commit,
                });
            }
            if !seen_entries.insert(entry.tx_index) {
                return Err(ArtifactError::DuplicateEntryIndex(entry.tx_index));
            }
        }

        if !self.entries.is_empty() && self.artifacts.is_empty() {
            return Err(ArtifactError::MissingArtifacts);
        }

        let statement_root = self.statement_root();
        let public_inputs_root = self.public_inputs_root();
        let valid_indexes = self
            .execution_bundle
            .covered_tx_indexes
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut artifact_indexes = BTreeSet::new();

        for (artifact_index, artifact) in self.artifacts.iter().enumerate() {
            if artifact.covered_tx_indexes.is_empty() {
                return Err(ArtifactError::EmptyArtifactCoverage { artifact_index });
            }
            if artifact.statement_root != statement_root {
                return Err(ArtifactError::ArtifactStatementRootMismatch);
            }
            if artifact.public_inputs_root != public_inputs_root {
                return Err(ArtifactError::ArtifactPublicInputsRootMismatch);
            }
            for tx_index in &artifact.covered_tx_indexes {
                if !valid_indexes.contains(tx_index) {
                    return Err(ArtifactError::ArtifactInvalidCoveredIndex(*tx_index));
                }
                if !artifact_indexes.insert(*tx_index) {
                    return Err(ArtifactError::DuplicateArtifactCoverage(*tx_index));
                }
            }
        }

        if artifact_indexes.len() != valid_indexes.len() {
            return Err(ArtifactError::IncompleteArtifactCoverage {
                expected: valid_indexes.len(),
                actual: artifact_indexes.len(),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ArtifactError {
    #[error(transparent)]
    Batch(#[from] BatchBuildError),
    #[error("artifact entry length mismatch: expected {expected}, got {actual}")]
    EntryLengthMismatch { expected: usize, actual: usize },
    #[error("artifact entry {index} tx_index mismatch: expected {expected}, got {actual}")]
    EntryIndexMismatch {
        index: usize,
        expected: u32,
        actual: u32,
    },
    #[error("artifact entry {index} statement_commit mismatch: expected {expected:?}, got {actual:?}")]
    EntryStatementMismatch {
        index: usize,
        expected: Hash32,
        actual: Hash32,
    },
    #[error("artifact entries contain duplicate tx_index {0}")]
    DuplicateEntryIndex(u32),
    #[error("proof artifact sidecar is missing batch artifacts")]
    MissingArtifacts,
    #[error("batch proof artifact #{artifact_index} must cover at least one tx index")]
    EmptyArtifactCoverage { artifact_index: usize },
    #[error("batch proof artifact statement_root does not match execution bundle")]
    ArtifactStatementRootMismatch,
    #[error("batch proof artifact public_inputs_root does not match execution bundle entries")]
    ArtifactPublicInputsRootMismatch,
    #[error("batch proof artifact covers invalid tx index {0}")]
    ArtifactInvalidCoveredIndex(u32),
    #[error("batch proof artifacts cover tx index {0} more than once")]
    DuplicateArtifactCoverage(u32),
    #[error("batch proof artifacts cover {actual} tx indexes but execution bundle requires {expected}")]
    IncompleteArtifactCoverage { expected: usize, actual: usize },
    #[error("block proof artifacts block_hash mismatch: expected {expected:?}, got {actual:?}")]
    BlockHashMismatch { expected: Hash32, actual: Hash32 },
    #[error("block proof artifacts execution bundle does not match block body")]
    ExecutionBundleMismatch,
    #[error("block proof artifacts proof certificates do not match block body")]
    ProofCertificateMismatch,
}

#[cfg(test)]
mod tests {
    use privai_chain::{
        Amount14, AuxWitness, CanonicalEncode, InputRef, LweCiphertext, OutputNote, RecipientBox,
        RecipientBoxPlaintext, SpendPolicy, TransferNoteTx, TxCore, TX_TYPE_TRANSFER_NOTE,
        PRIVAI_V0,
    };

    use super::*;
    use crate::{TransferInputWitness, TransferOutputWitness, TransferStatement, TransferWitness};

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

    fn sample_tx_and_proving(seed: u8) -> (TransferNoteTx, TransferProvingData) {
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
        let proving = crate::TransferProvingData::from_tx_and_witness(
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

        (tx, proving)
    }

    fn sample_proving(seed: u8) -> TransferProvingData {
        sample_tx_and_proving(seed).1
    }

    #[test]
    fn block_proof_artifacts_roundtrip_to_certificates() {
        let proving_a = sample_proving(10);
        let proving_b = sample_proving(20);
        let execution_bundle = build_execution_bundle_from_transfer_proofs(
            &[proving_a.clone(), proving_b.clone()],
            privai_chain::ExecutionMode::FullBatchProof,
        )
        .expect("bundle");
        let artifact = BatchProofArtifact {
            proof_system_id: 1,
            statement_root: merkle_root(execution_bundle.statement_commits.iter().copied()),
            public_inputs_root: execution_bundle.public_inputs_root,
            covered_tx_indexes: vec![0, 1],
            proof_bytes: vec![1, 2, 3],
            prover_ids: vec![[9; 32]],
            proof_meta_hash: [8; 32],
        };

        let bundle = BlockProofArtifacts::from_transfer_proofs(
            [7; 32],
            &[proving_a, proving_b],
            vec![artifact.clone()],
        )
        .expect("proof artifacts");

        assert_eq!(bundle.proof_certificates(), vec![artifact.certificate()]);
        assert_eq!(bundle.statement_root(), artifact.statement_root);
        assert_eq!(bundle.public_inputs_root(), artifact.public_inputs_root);
    }

    #[test]
    fn block_proof_artifacts_reject_artifact_root_mismatch() {
        let proving = sample_proving(10);
        let execution_bundle = build_execution_bundle_from_transfer_proofs(
            std::slice::from_ref(&proving),
            privai_chain::ExecutionMode::FullBatchProof,
        )
        .expect("bundle");
        let artifact = BatchProofArtifact {
            proof_system_id: 1,
            statement_root: [0xAA; 32],
            public_inputs_root: execution_bundle.public_inputs_root,
            covered_tx_indexes: vec![0],
            proof_bytes: vec![1, 2, 3],
            prover_ids: vec![[9; 32]],
            proof_meta_hash: [8; 32],
        };

        assert!(matches!(
            BlockProofArtifacts::from_transfer_proofs([7; 32], &[proving], vec![artifact]),
            Err(ArtifactError::ArtifactStatementRootMismatch)
        ));
    }

    #[test]
    fn block_proof_artifacts_match_block() {
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
        let bundle = BlockProofArtifacts::from_transfer_proofs(
            [7; 32],
            std::slice::from_ref(&proving),
            vec![artifact.clone()],
        )
        .expect("proof artifacts");

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
            txs: vec![privai_chain::Transaction::TransferNote(tx)],
            execution_bundle: execution_bundle.clone(),
            proof_certificates: vec![artifact.certificate()],
            extra_receipts: Vec::new(),
        });
        let matching_bundle = BlockProofArtifacts {
            block_hash: block.hash(),
            execution_bundle,
            ..bundle
        };

        matching_bundle
            .validate_against_block(&block)
            .expect("matching artifacts");
    }

    #[test]
    fn block_proof_artifacts_reject_missing_artifact_coverage() {
        let proving_a = sample_proving(10);
        let proving_b = sample_proving(20);
        let execution_bundle = build_execution_bundle_from_transfer_proofs(
            &[proving_a.clone(), proving_b.clone()],
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

        assert!(matches!(
            BlockProofArtifacts::from_transfer_proofs([7; 32], &[proving_a, proving_b], vec![artifact]),
            Err(ArtifactError::IncompleteArtifactCoverage {
                expected: 2,
                actual: 1,
            })
        ));
    }

    #[test]
    fn block_proof_artifacts_reject_duplicate_artifact_coverage() {
        let proving_a = sample_proving(10);
        let proving_b = sample_proving(20);
        let execution_bundle = build_execution_bundle_from_transfer_proofs(
            &[proving_a.clone(), proving_b.clone()],
            privai_chain::ExecutionMode::FullBatchProof,
        )
        .expect("bundle");
        let artifact_a = BatchProofArtifact {
            proof_system_id: 1,
            statement_root: merkle_root(execution_bundle.statement_commits.iter().copied()),
            public_inputs_root: execution_bundle.public_inputs_root,
            covered_tx_indexes: vec![0, 1],
            proof_bytes: vec![1, 2, 3],
            prover_ids: vec![[9; 32]],
            proof_meta_hash: [8; 32],
        };
        let artifact_b = BatchProofArtifact {
            proof_system_id: 1,
            statement_root: merkle_root(execution_bundle.statement_commits.iter().copied()),
            public_inputs_root: execution_bundle.public_inputs_root,
            covered_tx_indexes: vec![1],
            proof_bytes: vec![4, 5, 6],
            prover_ids: vec![[7; 32]],
            proof_meta_hash: [6; 32],
        };

        assert!(matches!(
            BlockProofArtifacts::from_transfer_proofs(
                [7; 32],
                &[proving_a, proving_b],
                vec![artifact_a, artifact_b]
            ),
            Err(ArtifactError::DuplicateArtifactCoverage(1))
        ));
    }
}
