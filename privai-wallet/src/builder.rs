//! Transaction Builder for privAI.
//! Role: Constructing valid, balanced TransferNoteTx and SettlementTx.
//! Privacy Tiers: RecipientPrivacyLite (Explicit Amt), FullPrivacy (LWE Encrypted Amt).
//! Constraint: Total Input Value == Total Output Value + Fee.
//! Escalation Rule: Auto-escalate to FullPrivacy for Escrow and high-value transfers.

use crate::error::WalletError;
use crate::state::{OwnedNoteStatus, SpendMaterial};
use crate::store::WalletStore;
use crate::wallet::PrivaiWallet;
use privai_chain::{
    derive_aux_commit, Amount14, AuxWitness, CanonicalEncode, Hash32, InputAuth, InputRef,
    LiteOutputNote, LiteTxCore, LiteTransferTx, LweCiphertext, OutputNote, ReceiveBundle,
    RecipientBoxPlaintext, SpendPolicy, TransferNoteTx, TxCore,
    TX_TYPE_LITE_TRANSFER, TX_TYPE_TRANSFER_NOTE, PRIVAI_V0,
};
use privai_proof::{
    LiteTransferStatement, TransferInputWitness, TransferOutputWitness, TransferProvingData,
    TransferStatement, TransferWitness,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferOutputPlan {
    pub bundle: ReceiveBundle,
    pub amount: privai_chain::Amount14,
    pub ct_amt: LweCiphertext,
    pub witness_seed: Hash32,
    pub nullifier_key: Hash32,
    pub spend_policy: SpendPolicy,
    pub noise_class: u8,
    pub sender_memo: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuiltTransferNote {
    pub tx: TransferNoteTx,
    pub proof: TransferProvingData,
}

/// Output specification for a LiteTransfer (RPL) transaction.
/// Amounts are plain u64 — no LWE encryption, no zero-knowledge proof.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiteTransferOutputPlan {
    pub bundle: ReceiveBundle,
    pub amount: u64,
    pub witness_seed: Hash32,
    pub nullifier_key: Hash32,
    pub spend_policy: SpendPolicy,
    pub noise_class: u8,
    pub sender_memo: Option<Vec<u8>>,
}

/// A built LiteTransfer transaction. Contains no proof — lite transfers are not zero-knowledge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuiltLiteTransferNote {
    pub tx: LiteTransferTx,
}

impl<S: WalletStore> PrivaiWallet<S> {
    pub fn build_transfer_note(
        &self,
        spend: &SpendMaterial,
        outputs: Vec<TransferOutputPlan>,
        fee: u64,
        auth: Vec<InputAuth>,
    ) -> Result<BuiltTransferNote, WalletError> {
        if outputs.is_empty() {
            return Err(WalletError::NoTransferOutputs);
        }

        let tracked = self
            .snapshot()
            .owned_notes
            .get(&spend.note_commit)
            .ok_or(WalletError::UnknownNote(spend.note_commit))?;

        if !matches!(tracked.status, OwnedNoteStatus::Spendable) {
            return Err(WalletError::InputNoteNotSpendable(spend.note_commit));
        }
        if tracked.derived_nullifier != spend.nullifier
            || tracked.opened.witness_seed != spend.witness_seed
            || tracked.opened.nullifier_key != spend.nullifier_key
            || tracked.opened.spend_policy_opening != spend.spend_policy_opening
            || tracked.opened.aux_opening != spend.aux_opening
        {
            return Err(WalletError::SpendMaterialMismatch(spend.note_commit));
        }

        let required = outputs.iter().try_fold(fee, |acc, output| {
            acc.checked_add(output.amount.value() as u64)
        });
        let Some(required) = required else {
            return Err(WalletError::TransferArithmeticOverflow);
        };
        let available = spend.amount.value() as u64;
        if available != required {
            return Err(WalletError::TransferImbalance { available, required });
        }

        let mut built_outputs = Vec::with_capacity(outputs.len());
        let mut output_witnesses = Vec::with_capacity(outputs.len());

        for output in outputs {
            let spend_policy_commit = output.spend_policy.commitment();
            let aux_witness = AuxWitness {
                version: PRIVAI_V0,
                amount: output.amount,
                witness_seed: output.witness_seed,
                noise_class: output.noise_class,
                bundle_id: output.bundle.bundle_id,
            };
            let aux_commit = derive_aux_commit(&aux_witness);
            let note_payload_commit = OutputNote::payload_commit_from_parts(
                PRIVAI_V0,
                &spend_policy_commit,
                &output.ct_amt,
                &aux_commit,
            );
            let recipient_opening = RecipientBoxPlaintext {
                version: PRIVAI_V0,
                bundle_id: output.bundle.bundle_id,
                note_payload_commit,
                amount: output.amount,
                witness_seed: output.witness_seed,
                nullifier_key: output.nullifier_key,
                spend_policy_opening: output.spend_policy.to_canonical_bytes(),
                aux_opening: aux_witness.to_canonical_bytes(),
                sender_memo: output.sender_memo,
            };
            let recipient_box = Self::seal_recipient_box(&output.bundle, &recipient_opening)?;
            let note = OutputNote::new(
                spend_policy_commit,
                output.ct_amt,
                aux_commit,
                recipient_box,
            );

            output_witnesses.push(TransferOutputWitness {
                note_commit: note.note_commit,
                recipient_opening,
            });
            built_outputs.push(note);
        }

        let statement = TransferStatement {
            input_note_commits: vec![spend.note_commit],
            input_nullifiers: vec![spend.nullifier],
            output_note_commits: built_outputs.iter().map(|note| note.note_commit).collect(),
            fee,
        };

        let tx = TransferNoteTx {
            core: TxCore {
                version: PRIVAI_V0,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: vec![InputRef {
                    note_commit: spend.note_commit,
                }],
                input_nullifiers: vec![spend.nullifier],
                outputs: built_outputs,
                fee,
                statement_commit: statement.commitment(),
                auth,
            },
        };
        tx.validate_shape().map_err(WalletError::TxShape)?;

        let proof = TransferProvingData::from_tx_and_witness(
            &tx,
            TransferWitness {
                input: TransferInputWitness {
                    amount: spend.amount,
                    witness_seed: spend.witness_seed,
                    nullifier_key: spend.nullifier_key,
                    spend_policy_opening: spend.spend_policy_opening.clone(),
                    aux_opening: spend.aux_opening.clone(),
                },
                outputs: output_witnesses,
            },
        )
        .map_err(WalletError::ProofBuild)?;

        Ok(BuiltTransferNote { tx, proof })
    }

    /// Build a LiteTransfer (RPL) transaction from a spendable note.
    ///
    /// Outputs use explicit u64 amounts — no LWE encryption, no zero-knowledge proof.
    /// Balance constraint: sum(output amounts) + fee == input note amount.
    pub fn build_lite_transfer_note(
        &self,
        spend: &SpendMaterial,
        outputs: Vec<LiteTransferOutputPlan>,
        fee: u64,
        auth: Vec<InputAuth>,
    ) -> Result<BuiltLiteTransferNote, WalletError> {
        if outputs.is_empty() {
            return Err(WalletError::NoTransferOutputs);
        }

        let tracked = self
            .snapshot()
            .owned_notes
            .get(&spend.note_commit)
            .ok_or(WalletError::UnknownNote(spend.note_commit))?;

        if !matches!(tracked.status, OwnedNoteStatus::Spendable) {
            return Err(WalletError::InputNoteNotSpendable(spend.note_commit));
        }
        if tracked.derived_nullifier != spend.nullifier
            || tracked.opened.witness_seed != spend.witness_seed
            || tracked.opened.nullifier_key != spend.nullifier_key
            || tracked.opened.spend_policy_opening != spend.spend_policy_opening
            || tracked.opened.aux_opening != spend.aux_opening
        {
            return Err(WalletError::SpendMaterialMismatch(spend.note_commit));
        }

        let required = outputs.iter().try_fold(fee, |acc, output| {
            acc.checked_add(output.amount)
        });
        let Some(required) = required else {
            return Err(WalletError::TransferArithmeticOverflow);
        };
        let available = spend.amount.value() as u64;
        if available != required {
            return Err(WalletError::TransferImbalance { available, required });
        }

        let mut built_outputs = Vec::with_capacity(outputs.len());

        for output in outputs {
            let spend_policy_commit = output.spend_policy.commitment();
            let amount14 = Amount14::new(output.amount as u16)
                .map_err(|_| WalletError::LiteOutputAmountTooLarge(output.amount))?;
            let aux_witness = AuxWitness {
                version: PRIVAI_V0,
                amount: amount14,
                witness_seed: output.witness_seed,
                noise_class: output.noise_class,
                bundle_id: output.bundle.bundle_id,
            };
            let aux_commit = derive_aux_commit(&aux_witness);
            let note_payload_commit = LiteOutputNote::payload_commit_from_parts(
                PRIVAI_V0,
                &spend_policy_commit,
                output.amount,
                &aux_commit,
            );
            let recipient_opening = RecipientBoxPlaintext {
                version: PRIVAI_V0,
                bundle_id: output.bundle.bundle_id,
                note_payload_commit,
                amount: amount14,
                witness_seed: output.witness_seed,
                nullifier_key: output.nullifier_key,
                spend_policy_opening: output.spend_policy.to_canonical_bytes(),
                aux_opening: aux_witness.to_canonical_bytes(),
                sender_memo: output.sender_memo,
            };
            let recipient_box = Self::seal_recipient_box(&output.bundle, &recipient_opening)?;
            let note = LiteOutputNote::new(
                output.amount,
                spend_policy_commit,
                aux_commit,
                recipient_box,
            );
            built_outputs.push(note);
        }

        let tx = LiteTransferTx {
            core: LiteTxCore {
                version: PRIVAI_V0,
                tx_type: TX_TYPE_LITE_TRANSFER,
                inputs: vec![InputRef {
                    note_commit: spend.note_commit,
                }],
                input_nullifiers: vec![spend.nullifier],
                outputs: built_outputs,
                fee,
                statement_commit: [0u8; 32],
                auth,
            },
        };
        tx.validate_shape().map_err(WalletError::TxShape)?;

        Ok(BuiltLiteTransferNote { tx })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryWalletStore;
    use crate::wallet::PrivaiWallet;
    use privai_chain::{Amount14, CanonicalEncode};

    fn generated_bundle(expires_at: u64, route_hint: Option<Vec<u8>>) -> ReceiveBundle {
        let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
        wallet
            .create_local_bundle(expires_at, 0, route_hint)
            .expect("generated bundle")
    }

    fn sample_output_plan(bundle: ReceiveBundle, amount: u16) -> TransferOutputPlan {
        TransferOutputPlan {
            bundle,
            amount: Amount14::new(amount).expect("amount"),
            ct_amt: LweCiphertext::default(),
            witness_seed: [0x41; 32],
            nullifier_key: [0x42; 32],
            spend_policy: SpendPolicy::Single {
                falcon_pk_hash: [0x51; 32],
            },
            noise_class: 1,
            sender_memo: Some(b"memo".to_vec()),
        }
    }

    fn prepare_spendable_wallet() -> (PrivaiWallet<MemoryWalletStore>, SpendMaterial, ReceiveBundle) {
        let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("wallet");
        let input_bundle = wallet
            .create_local_bundle(100, 0, Some(vec![1, 2]))
            .expect("input bundle");

        let amount = Amount14::new(77).expect("amount");
        let spend_policy = SpendPolicy::Single {
            falcon_pk_hash: [0x31; 32],
        };
        let aux_witness = AuxWitness {
            version: PRIVAI_V0,
            amount,
            witness_seed: [0x21; 32],
            noise_class: 1,
            bundle_id: input_bundle.bundle_id,
        };
        let aux_commit = derive_aux_commit(&aux_witness);
        let note_payload_commit = OutputNote::payload_commit_from_parts(
            PRIVAI_V0,
            &spend_policy.commitment(),
            &LweCiphertext::default(),
            &aux_commit,
        );
        let opened = RecipientBoxPlaintext {
            version: PRIVAI_V0,
            bundle_id: input_bundle.bundle_id,
            note_payload_commit,
            amount,
            witness_seed: [0x21; 32],
            nullifier_key: [0x22; 32],
            spend_policy_opening: spend_policy.to_canonical_bytes(),
            aux_opening: aux_witness.to_canonical_bytes(),
            sender_memo: Some(vec![9]),
        };
        let recipient_box = PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(&input_bundle, &opened)
            .expect("seal input box");
        let input_note = OutputNote::new(
            spend_policy.commitment(),
            LweCiphertext::default(),
            aux_commit,
            recipient_box,
        );
        wallet
            .record_opened_note(input_note.clone(), opened)
            .expect("record note");
        let spend = wallet
            .spend_material(&input_note.note_commit)
            .expect("spend material");

        let recipient_bundle = generated_bundle(200, Some(vec![5, 6]));

        (wallet, spend, recipient_bundle)
    }

    #[test]
    fn build_transfer_note_creates_tx_and_proof_data() {
        let (wallet, spend, recipient_bundle) = prepare_spendable_wallet();
        let built = wallet
            .build_transfer_note(
                &spend,
                vec![
                    sample_output_plan(recipient_bundle, 50),
                    sample_output_plan(generated_bundle(201, None), 24),
                ],
                3,
                Vec::new(),
            )
            .expect("build transfer");

        assert_eq!(built.tx.core.inputs.len(), 1);
        assert_eq!(built.tx.core.outputs.len(), 2);
        assert_eq!(built.tx.core.statement_commit, built.proof.statement.commitment());
        assert_eq!(
            built.proof.public_inputs.output_note_commits,
            built.tx.core.outputs.iter().map(|note| note.note_commit).collect::<Vec<_>>()
        );
    }

    #[test]
    fn build_transfer_note_rejects_imbalanced_amounts() {
        let (wallet, spend, recipient_bundle) = prepare_spendable_wallet();
        let err = wallet
            .build_transfer_note(
                &spend,
                vec![sample_output_plan(recipient_bundle, 70)],
                3,
                Vec::new(),
            )
            .expect_err("must reject imbalance");

        assert!(matches!(
            err,
            WalletError::TransferImbalance {
                available: 77,
                required: 73,
            }
        ));
    }

    fn sample_lite_output_plan(bundle: ReceiveBundle, amount: u64) -> LiteTransferOutputPlan {
        LiteTransferOutputPlan {
            bundle,
            amount,
            witness_seed: [0x61; 32],
            nullifier_key: [0x62; 32],
            spend_policy: SpendPolicy::Single {
                falcon_pk_hash: [0x71; 32],
            },
            noise_class: 1,
            sender_memo: Some(b"lite-memo".to_vec()),
        }
    }

    #[test]
    fn build_lite_transfer_note_creates_tx() {
        let (wallet, spend, recipient_bundle) = prepare_spendable_wallet();
        let built = wallet
            .build_lite_transfer_note(
                &spend,
                vec![
                    sample_lite_output_plan(recipient_bundle, 50),
                    sample_lite_output_plan(generated_bundle(201, None), 24),
                ],
                3,
                Vec::new(),
            )
            .expect("build lite transfer");

        assert_eq!(built.tx.core.inputs.len(), 1);
        assert_eq!(built.tx.core.outputs.len(), 2);
        assert_eq!(built.tx.core.tx_type, privai_chain::TX_TYPE_LITE_TRANSFER);
        assert_eq!(built.tx.core.fee, 3);
        assert_eq!(built.tx.core.outputs[0].amount, 50);
        assert_eq!(built.tx.core.outputs[1].amount, 24);
    }

    #[test]
    fn build_lite_transfer_note_rejects_imbalanced_amounts() {
        let (wallet, spend, recipient_bundle) = prepare_spendable_wallet();
        let err = wallet
            .build_lite_transfer_note(
                &spend,
                vec![sample_lite_output_plan(recipient_bundle, 70)],
                3,
                Vec::new(),
            )
            .expect_err("must reject imbalance");

        assert!(matches!(
            err,
            WalletError::TransferImbalance {
                available: 77,
                required: 73,
            }
        ));
    }

    #[test]
    fn build_lite_transfer_note_rejects_non_spendable_input() {
        let (mut wallet, spend, recipient_bundle) = prepare_spendable_wallet();
        wallet.mark_note_spent(spend.note_commit).expect("mark spent");

        let err = wallet
            .build_lite_transfer_note(
                &spend,
                vec![sample_lite_output_plan(recipient_bundle, 74)],
                3,
                Vec::new(),
            )
            .expect_err("must reject spent input");

        assert!(matches!(err, WalletError::InputNoteNotSpendable(nc) if nc == spend.note_commit));
    }

    #[test]
    fn build_lite_transfer_note_output_commits_are_deterministic() {
        let (wallet, spend, recipient_bundle) = prepare_spendable_wallet();
        let built1 = wallet
            .build_lite_transfer_note(
                &spend,
                vec![sample_lite_output_plan(recipient_bundle.clone(), 77)],
                0,
                Vec::new(),
            )
            .expect("first build");
        let built2 = wallet
            .build_lite_transfer_note(
                &spend,
                vec![sample_lite_output_plan(recipient_bundle, 77)],
                0,
                Vec::new(),
            )
            .expect("second build");

        assert_eq!(
            built1.tx.core.outputs[0].note_commit,
            built2.tx.core.outputs[0].note_commit
        );
    }

    #[test]
    fn build_transfer_note_rejects_non_spendable_input() {
        let (mut wallet, spend, recipient_bundle) = prepare_spendable_wallet();
        wallet.mark_note_spent(spend.note_commit).expect("mark spent");

        let err = wallet
            .build_transfer_note(
                &spend,
                vec![sample_output_plan(recipient_bundle, 74)],
                3,
                Vec::new(),
            )
            .expect_err("must reject spent input");

        assert!(matches!(err, WalletError::InputNoteNotSpendable(note_commit) if note_commit == spend.note_commit));
    }
}
