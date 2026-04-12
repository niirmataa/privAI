use privai_chain::{merkle_root, Block, Hash32, Transaction, TransferNoteTx};
use privai_proof::{
    artifact::{BatchProofArtifact, BlockProofArtifacts},
    ProofJob, TransferProvingData,
};

use crate::error::WalletError;
use crate::escrow_builder::EscrowAssembledTx;

/// Frozen Stage B handoff: final tx plus the exact proof context that belongs to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EscrowProofReadyHandoff {
    pub tx: TransferNoteTx,
    pub tx_signing_hash: Hash32,
    pub proving_data: TransferProvingData,
    pub proof_job: ProofJob,
    pub statement_commit: Hash32,
    pub public_inputs_hash: Hash32,
}

/// Single-transfer proof attachment result for the escrow submit path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EscrowAttachedProof {
    pub handoff: EscrowProofReadyHandoff,
    pub artifact: BatchProofArtifact,
}

impl EscrowProofReadyHandoff {
    pub fn build(
        assembled: &EscrowAssembledTx,
        job_fee: u64,
        deadline_height: u64,
        requester_hint: Hash32,
    ) -> Result<Self, WalletError> {
        let expected_signing_hash =
            Transaction::TransferNote(assembled.tx.clone()).tx_signing_hash();
        if assembled.tx_signing_hash != expected_signing_hash {
            return Err(WalletError::Crypto(
                "tx_signing_hash mismatch between assembled payload and tx body".into(),
            ));
        }

        let rebuilt = TransferProvingData::from_tx_and_witness(
            &assembled.tx,
            assembled.proof_scaffolding.witness.clone(),
        )
        .map_err(WalletError::ProofBuild)?;

        if rebuilt.statement != assembled.proof_scaffolding.statement {
            return Err(WalletError::Crypto(
                "statement mismatch between tx and proving data".into(),
            ));
        }

        if rebuilt.public_inputs != assembled.proof_scaffolding.public_inputs {
            return Err(WalletError::Crypto(
                "public inputs mismatch between tx and proving data".into(),
            ));
        }

        let statement_commit = rebuilt.statement.commitment();
        let public_inputs_hash = rebuilt.public_inputs_hash();
        let proof_job = rebuilt.to_proof_job(job_fee, deadline_height, requester_hint);

        Ok(Self {
            tx: assembled.tx.clone(),
            tx_signing_hash: expected_signing_hash,
            proving_data: rebuilt,
            proof_job,
            statement_commit,
            public_inputs_hash,
        })
    }

    pub fn attach_single_tx_proof_result(
        &self,
        proof_bytes: Vec<u8>,
        proof_system_id: u8,
        prover_ids: Vec<Hash32>,
        proof_meta_hash: Hash32,
    ) -> Result<EscrowAttachedProof, WalletError> {
        if proof_system_id == 0 {
            return Err(WalletError::Crypto(
                "proof_system_id must be non-zero".into(),
            ));
        }
        if prover_ids.is_empty() {
            return Err(WalletError::Crypto(
                "proof result must include at least one prover id".into(),
            ));
        }

        let artifact = BatchProofArtifact {
            proof_system_id,
            statement_root: merkle_root(std::iter::once(self.statement_commit)),
            public_inputs_root: merkle_root(std::iter::once(self.public_inputs_hash)),
            covered_tx_indexes: vec![0],
            proof_bytes,
            prover_ids,
            proof_meta_hash,
        };

        Ok(EscrowAttachedProof {
            handoff: self.clone(),
            artifact,
        })
    }
}

impl EscrowAttachedProof {
    pub fn to_block_proof_artifacts(
        &self,
        block_hash: Hash32,
    ) -> Result<BlockProofArtifacts, WalletError> {
        BlockProofArtifacts::from_transfer_proofs(
            block_hash,
            std::slice::from_ref(&self.handoff.proving_data),
            vec![self.artifact.clone()],
        )
        .map_err(|err| WalletError::Crypto(format!("block proof artifact build failed: {err}")))
    }

    pub fn to_block_proof_artifacts_for_block(
        &self,
        block: &Block,
    ) -> Result<BlockProofArtifacts, WalletError> {
        let artifacts = self.to_block_proof_artifacts(block.hash())?;
        artifacts.validate_against_block(block).map_err(|err| {
            WalletError::Crypto(format!("block proof artifact validation failed: {err}"))
        })?;
        Ok(artifacts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use privai_chain::ExecutionMode;
    use privai_chain::{
        Amount14, AuxWitness, BlockTemplate, CanonicalEncode, InputAuth, InputRef, LweCiphertext,
        Nullifier, OutputNote, RecipientBox, RecipientBoxPlaintext, SpendPolicy, TxCore, PRIVAI_V0,
        TX_TYPE_TRANSFER_NOTE,
    };
    use privai_proof::{
        build_execution_bundle_from_transfer_proofs, TransferInputWitness, TransferOutputWitness,
        TransferStatement, TransferWitness,
    };

    fn sample_assembled() -> EscrowAssembledTx {
        let seed = 10u8;
        let output = OutputNote::new(
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
        );
        let statement = TransferStatement {
            input_note_commits: vec![[seed.wrapping_add(30); 32]],
            input_nullifiers: vec![Nullifier([seed.wrapping_add(40); 32])],
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
                input_nullifiers: vec![Nullifier([seed.wrapping_add(40); 32])],
                outputs: vec![output.clone()],
                fee: seed as u64,
                statement_commit: statement.commitment(),
                auth: vec![InputAuth {
                    policy_tag: 1,
                    signer_pks: vec![vec![0xAA; 16]],
                    signatures: vec![vec![0xBB; 16]],
                    policy_opening: None,
                    escrow_action: None,
                }],
            },
        };
        let witness = TransferWitness {
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
        };
        let proving_data =
            TransferProvingData::from_tx_and_witness(&tx, witness).expect("proving data");
        let tx_signing_hash = Transaction::TransferNote(tx.clone()).tx_signing_hash();

        EscrowAssembledTx {
            tx,
            tx_signing_hash,
            input_auth: InputAuth {
                policy_tag: 1,
                signer_pks: vec![vec![0xAA; 16]],
                signatures: vec![vec![0xBB; 16]],
                policy_opening: None,
                escrow_action: None,
            },
            nullifier: Nullifier([seed.wrapping_add(40); 32]),
            output_note_commit: output.note_commit,
            proof_scaffolding: proving_data,
        }
    }

    fn sample_block(attached: &EscrowAttachedProof) -> Block {
        let execution_bundle = build_execution_bundle_from_transfer_proofs(
            std::slice::from_ref(&attached.handoff.proving_data),
            ExecutionMode::FullBatchProof,
        )
        .expect("execution bundle");

        Block::from_template(BlockTemplate {
            chain_id: 1,
            height: 1,
            epoch: 0,
            round: 0,
            timestamp_ms: 1_000,
            prev_block_hash: [0u8; 32],
            proposer_pk_hash: [0xA1; 32],
            epoch_seed_hash: [0xB2; 32],
            parent_qc_hash: [0xC3; 32],
            state_root: [0xD4; 32],
            txs: vec![Transaction::TransferNote(attached.handoff.tx.clone())],
            execution_bundle,
            proof_certificates: vec![attached.artifact.certificate()],
            extra_receipts: Vec::new(),
        })
    }

    #[test]
    fn handoff_builds_from_consistent_assembled_tx() {
        let assembled = sample_assembled();
        let handoff =
            EscrowProofReadyHandoff::build(&assembled, 100, 200, [8; 32]).expect("handoff");

        assert_eq!(handoff.tx, assembled.tx);
        assert_eq!(handoff.tx_signing_hash, assembled.tx_signing_hash);
        assert_eq!(handoff.statement_commit, assembled.tx.core.statement_commit);
        assert_eq!(
            handoff.public_inputs_hash,
            handoff.proving_data.public_inputs_hash()
        );
        assert_eq!(handoff.proof_job.job_fee, 100);
        assert_eq!(handoff.proof_job.deadline_height, 200);
    }

    #[test]
    fn handoff_rejects_tx_signing_hash_mismatch() {
        let mut assembled = sample_assembled();
        assembled.tx_signing_hash = [0xAA; 32];

        let result = EscrowProofReadyHandoff::build(&assembled, 100, 200, [8; 32]);
        assert!(
            matches!(result, Err(WalletError::Crypto(msg)) if msg.contains("tx_signing_hash mismatch"))
        );
    }

    #[test]
    fn handoff_rejects_public_inputs_mismatch() {
        let mut assembled = sample_assembled();
        assembled.proof_scaffolding.public_inputs.fee = 999;

        let result = EscrowProofReadyHandoff::build(&assembled, 100, 200, [8; 32]);
        assert!(
            matches!(result, Err(WalletError::Crypto(msg)) if msg.contains("public inputs mismatch"))
        );
    }

    #[test]
    fn handoff_rejects_witness_derived_mismatch() {
        let mut assembled = sample_assembled();
        assembled.proof_scaffolding.witness.outputs[0].note_commit = [0xEE; 32];

        let result = EscrowProofReadyHandoff::build(&assembled, 100, 200, [8; 32]);
        assert!(matches!(result, Err(WalletError::ProofBuild(_))));
    }

    #[test]
    fn attach_single_tx_proof_result_builds_artifact() {
        let assembled = sample_assembled();
        let handoff =
            EscrowProofReadyHandoff::build(&assembled, 100, 200, [8; 32]).expect("handoff");
        let attached = handoff
            .attach_single_tx_proof_result(vec![1, 2, 3], 2, vec![[9; 32]], [10; 32])
            .expect("attach");

        assert_eq!(attached.artifact.proof_system_id, 2);
        assert_eq!(attached.artifact.proof_bytes, vec![1, 2, 3]);
        assert_eq!(attached.artifact.prover_ids, vec![[9; 32]]);
        assert_eq!(attached.artifact.proof_meta_hash, [10; 32]);
        assert_eq!(attached.artifact.covered_tx_indexes, vec![0]);
        assert_eq!(
            attached.artifact.statement_root,
            merkle_root(std::iter::once(handoff.statement_commit))
        );
        assert_eq!(
            attached.artifact.public_inputs_root,
            merkle_root(std::iter::once(handoff.public_inputs_hash))
        );
    }

    #[test]
    fn attach_requires_non_empty_prover_set_and_non_zero_system_id() {
        let assembled = sample_assembled();
        let handoff =
            EscrowProofReadyHandoff::build(&assembled, 100, 200, [8; 32]).expect("handoff");

        let bad_system = handoff.attach_single_tx_proof_result(vec![1], 0, vec![[9; 32]], [10; 32]);
        assert!(
            matches!(bad_system, Err(WalletError::Crypto(msg)) if msg.contains("proof_system_id"))
        );

        let bad_provers = handoff.attach_single_tx_proof_result(vec![1], 2, vec![], [10; 32]);
        assert!(
            matches!(bad_provers, Err(WalletError::Crypto(msg)) if msg.contains("at least one prover id"))
        );
    }

    #[test]
    fn attached_proof_can_build_import_ready_block_artifacts() {
        let assembled = sample_assembled();
        let handoff =
            EscrowProofReadyHandoff::build(&assembled, 100, 200, [8; 32]).expect("handoff");
        let attached = handoff
            .attach_single_tx_proof_result(vec![1, 2, 3], 2, vec![[9; 32]], [10; 32])
            .expect("attach");
        let block = sample_block(&attached);

        let artifacts = attached
            .to_block_proof_artifacts_for_block(&block)
            .expect("block artifacts");

        assert_eq!(artifacts.block_hash, block.hash());
        assert_eq!(artifacts.entries.len(), 1);
        assert_eq!(artifacts.entries[0].tx_index, 0);
        assert_eq!(artifacts.execution_bundle, block.body.execution_bundle);
        assert_eq!(
            artifacts.proof_certificates(),
            block.body.proof_certificates
        );
    }
}
