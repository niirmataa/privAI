use privai_chain::{
    merkle_root, ExecutionBundle, ExecutionMode, Hash32, ModelTx, SettlementTx, StakeTx,
    Transaction,
};
use thiserror::Error;

use crate::{TransferProvingData, TransferPublicInputs};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BatchBuildError {
    #[error("automatic public input derivation is not implemented for tx type {tx_type:#x}")]
    UnsupportedTransactionType { tx_type: u8 },
    #[error("transfer proof at index {index} has statement_commit {actual:?}, expected {expected:?}")]
    TransferProofStatementMismatch {
        index: usize,
        expected: Hash32,
        actual: Hash32,
    },
}

pub fn public_inputs_hash_for_transaction(tx: &Transaction) -> Result<Hash32, BatchBuildError> {
    match tx {
        Transaction::TransferNote(tx) => Ok(TransferPublicInputs::from_tx(tx).hash()),
        Transaction::Settlement(SettlementTx { core, .. }) => Err(BatchBuildError::UnsupportedTransactionType {
            tx_type: core.tx_type,
        }),
        Transaction::Model(ModelTx { core, .. }) => Err(BatchBuildError::UnsupportedTransactionType {
            tx_type: core.tx_type,
        }),
        Transaction::Stake(StakeTx { core, .. }) => Err(BatchBuildError::UnsupportedTransactionType {
            tx_type: core.tx_type,
        }),
    }
}

pub fn build_execution_bundle_from_transactions(
    txs: &[Transaction],
    mode: ExecutionMode,
) -> Result<ExecutionBundle, BatchBuildError> {
    if txs.is_empty() {
        return Ok(ExecutionBundle {
            statement_commits: Vec::new(),
            covered_tx_indexes: Vec::new(),
            public_inputs_root: merkle_root(std::iter::empty::<Hash32>()),
            execution_mode: ExecutionMode::Housekeeping,
        });
    }

    let statement_commits = txs.iter().map(Transaction::statement_commit).collect::<Vec<_>>();
    let covered_tx_indexes = (0..txs.len()).map(|index| index as u32).collect::<Vec<_>>();
    let public_inputs_root =
        merkle_root(txs.iter().map(public_inputs_hash_for_transaction).collect::<Result<Vec<_>, _>>()?);

    Ok(ExecutionBundle {
        statement_commits,
        covered_tx_indexes,
        public_inputs_root,
        execution_mode: mode,
    })
}

pub fn build_execution_bundle_from_transfer_proofs(
    proving_data: &[TransferProvingData],
    mode: ExecutionMode,
) -> Result<ExecutionBundle, BatchBuildError> {
    if proving_data.is_empty() {
        return Ok(ExecutionBundle {
            statement_commits: Vec::new(),
            covered_tx_indexes: Vec::new(),
            public_inputs_root: merkle_root(std::iter::empty::<Hash32>()),
            execution_mode: ExecutionMode::Housekeeping,
        });
    }

    let mut statement_commits = Vec::with_capacity(proving_data.len());
    let mut public_inputs_hashes = Vec::with_capacity(proving_data.len());
    for (index, proving) in proving_data.iter().enumerate() {
        let expected = proving.statement.commitment();
        let actual = proving.public_inputs.statement_commit;
        if actual != expected {
            return Err(BatchBuildError::TransferProofStatementMismatch {
                index,
                expected,
                actual,
            });
        }
        statement_commits.push(expected);
        public_inputs_hashes.push(proving.public_inputs_hash());
    }

    Ok(ExecutionBundle {
        covered_tx_indexes: (0..proving_data.len()).map(|index| index as u32).collect(),
        statement_commits,
        public_inputs_root: merkle_root(public_inputs_hashes),
        execution_mode: mode,
    })
}

#[cfg(test)]
mod tests {
    use privai_chain::{
        merkle_root, Amount14, AuxWitness, CanonicalEncode, InputRef, LweCiphertext, OutputNote, RecipientBox,
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
            RecipientBox::new(vec![seed], [seed; 24], vec![seed.wrapping_add(1)], [seed; 16], [seed; 16]),
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
        crate::TransferProvingData::from_tx_and_witness(
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

    #[test]
    fn execution_bundle_from_transactions_derives_public_inputs_root() {
        let tx_a = sample_transfer(10, [11; 32]);
        let tx_b = sample_transfer(20, [22; 32]);
        let bundle = build_execution_bundle_from_transactions(
            &[tx_a.clone(), tx_b.clone()],
            ExecutionMode::FullBatchProof,
        )
        .expect("bundle");

        assert_eq!(bundle.statement_commits, vec![tx_a.statement_commit(), tx_b.statement_commit()]);
        assert_eq!(bundle.covered_tx_indexes, vec![0, 1]);
        assert_eq!(
            bundle.public_inputs_root,
            merkle_root([
                public_inputs_hash_for_transaction(&tx_a).expect("tx a"),
                public_inputs_hash_for_transaction(&tx_b).expect("tx b"),
            ])
        );
    }

    #[test]
    fn execution_bundle_from_transfer_proofs_matches_transaction_batch() {
        let proving_a = sample_proving(10);
        let proving_b = sample_proving(20);

        let tx_a = Transaction::TransferNote(TransferNoteTx {
            core: TxCore {
                version: PRIVAI_V0,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: vec![InputRef { note_commit: proving_a.statement.input_note_commits[0] }],
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
                inputs: vec![InputRef { note_commit: proving_b.statement.input_note_commits[0] }],
                input_nullifiers: proving_b.statement.input_nullifiers.clone(),
                outputs: vec![sample_output(20)],
                fee: proving_b.statement.fee,
                statement_commit: proving_b.statement.commitment(),
                auth: Vec::new(),
            },
        });

        let from_txs = build_execution_bundle_from_transactions(
            &[tx_a, tx_b],
            ExecutionMode::FullBatchProof,
        )
        .expect("tx bundle");
        let from_proofs = build_execution_bundle_from_transfer_proofs(
            &[proving_a, proving_b],
            ExecutionMode::FullBatchProof,
        )
        .expect("proof bundle");

        assert_eq!(from_proofs.statement_commits, from_txs.statement_commits);
        assert_eq!(from_proofs.covered_tx_indexes, from_txs.covered_tx_indexes);
        assert_eq!(from_proofs.public_inputs_root, from_txs.public_inputs_root);
    }

    #[test]
    fn empty_batch_becomes_housekeeping_bundle() {
        let bundle =
            build_execution_bundle_from_transactions(&[], ExecutionMode::FullBatchProof).expect("bundle");
        assert_eq!(bundle.execution_mode, ExecutionMode::Housekeeping);
        assert!(bundle.statement_commits.is_empty());
        assert!(bundle.covered_tx_indexes.is_empty());
    }
}
