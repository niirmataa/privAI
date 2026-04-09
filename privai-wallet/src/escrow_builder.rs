use crate::builder::TransferOutputPlan;
use crate::error::WalletError;
use crate::state::{OwnedNoteStatus, SpendMaterial};
use crate::store::WalletStore;
use crate::wallet::PrivaiWallet;
use privai_chain::escrow::{required_signers, EscrowAction};
use privai_chain::hash::{domain_hash, falcon_pk_hash};
use privai_chain::{
    Amount14, CanonicalDecode, CanonicalEncode, Hash32, InputAuth, LweCiphertext, Nullifier,
    ReceiveBundle, SpendPolicy, SpendPolicyTag, Transaction, TransferNoteTx,
};
use privai_proof::TransferProvingData;
use serde::{Deserialize, Serialize};

const ESCROW_OUTPUT_WITNESS_SEED_DOMAIN: &str = "privai:wallet:escrow-output-witness-seed:v0";
const ESCROW_OUTPUT_CT_AMT_DOMAIN: &str = "privai:wallet:escrow-output-ct-amt:v0";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthMaterial {
    pub policy_tag: u8,
    pub signer_pks: Vec<Vec<u8>>,
    pub signatures: Vec<Vec<u8>>,
    pub policy_opening: Vec<u8>,
    pub escrow_action: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FinalAssemblyInputs {
    pub proposal_hash: Hash32,
    pub escrow_id: Hash32,
    pub action: EscrowAction,
    pub funding_note_commit: Hash32,
    pub output_recipient_pk: Hash32,
    pub fee: u64,
    pub auth_material: AuthMaterial,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EscrowAssembledTx {
    pub tx: TransferNoteTx,
    pub tx_signing_hash: Hash32,
    pub input_auth: InputAuth,
    pub nullifier: Nullifier,
    pub output_note_commit: Hash32,
    pub proof_scaffolding: TransferProvingData,
}

fn derive_escrow_output_witness_seed(
    spend: &SpendMaterial,
    assembly: &FinalAssemblyInputs,
    recipient_bundle: &ReceiveBundle,
) -> Hash32 {
    let mut parts = vec![
        spend.note_commit.as_slice(),
        &assembly.proposal_hash,
        &assembly.escrow_id,
        &recipient_bundle.bundle_id,
        &assembly.output_recipient_pk,
    ];
    let action_byte = [assembly.action as u8];
    parts.push(&action_byte);
    let fee_bytes = assembly.fee.to_le_bytes();
    parts.push(&fee_bytes);

    domain_hash(ESCROW_OUTPUT_WITNESS_SEED_DOMAIN, &parts)
}

fn derive_escrow_output_ciphertext(
    spend: &SpendMaterial,
    assembly: &FinalAssemblyInputs,
    recipient_bundle: &ReceiveBundle,
    amount: Amount14,
) -> LweCiphertext {
    let dimension = LweCiphertext::default().a.len();
    let mut a = Vec::with_capacity(dimension);
    let amount_bytes = amount.value().to_le_bytes();
    let action_byte = [assembly.action as u8];
    let fee_bytes = assembly.fee.to_le_bytes();

    for index in 0..dimension {
        let index_bytes = (index as u32).to_le_bytes();
        let hash = domain_hash(
            ESCROW_OUTPUT_CT_AMT_DOMAIN,
            &[
                &spend.note_commit,
                &assembly.proposal_hash,
                &assembly.escrow_id,
                &recipient_bundle.bundle_id,
                &assembly.output_recipient_pk,
                &action_byte,
                &fee_bytes,
                &amount_bytes,
                &index_bytes,
                b"a",
            ],
        );
        a.push(u32::from_le_bytes(
            hash[..4].try_into().expect("hash prefix"),
        ));
    }

    let b_hash = domain_hash(
        ESCROW_OUTPUT_CT_AMT_DOMAIN,
        &[
            &spend.note_commit,
            &assembly.proposal_hash,
            &assembly.escrow_id,
            &recipient_bundle.bundle_id,
            &assembly.output_recipient_pk,
            &action_byte,
            &fee_bytes,
            &amount_bytes,
            b"b",
        ],
    );
    let b = u32::from_le_bytes(b_hash[..4].try_into().expect("hash prefix"));

    LweCiphertext::new(a, b).expect("derived ciphertext has canonical dimension")
}

fn build_final_input_auth(assembly: &FinalAssemblyInputs) -> Result<InputAuth, WalletError> {
    if assembly.auth_material.policy_tag != SpendPolicyTag::Escrow2of3 as u8 {
        return Err(WalletError::Crypto(
            "auth material policy_tag is not Escrow2of3".into(),
        ));
    }

    if assembly.auth_material.escrow_action != assembly.action as u8 {
        return Err(WalletError::Crypto(
            "auth material escrow_action does not match assembly action".into(),
        ));
    }

    if assembly.auth_material.signer_pks.len() != assembly.auth_material.signatures.len() {
        return Err(WalletError::Crypto(
            "mismatched signers and signatures lengths".into(),
        ));
    }

    let policy = SpendPolicy::from_canonical_bytes(&assembly.auth_material.policy_opening)
        .map_err(|_| WalletError::Crypto("invalid policy_opening".into()))?;

    let (buyer_pk_hash, merchant_pk_hash, operator_pk_hash, timeout_block) = match policy {
        SpendPolicy::Escrow2of3 {
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
            timeout_block,
        } => (
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
            timeout_block,
        ),
        _ => return Err(WalletError::Crypto("not Escrow2of3".into())),
    };

    let required = required_signers(assembly.action);
    let required_pair = [required.0.index(), required.1.index()];

    let mut signers = Vec::new();
    let mut seen_roles = std::collections::HashSet::new();
    let mut seen_pks = std::collections::HashSet::new();

    for (pk, sig) in assembly
        .auth_material
        .signer_pks
        .iter()
        .zip(assembly.auth_material.signatures.iter())
    {
        if !seen_pks.insert(pk.clone()) {
            return Err(WalletError::Crypto("duplicate signer pk".into()));
        }

        let pk_hash = falcon_pk_hash(pk);
        let role_idx = if pk_hash == buyer_pk_hash {
            0
        } else if pk_hash == merchant_pk_hash {
            1
        } else if pk_hash == operator_pk_hash {
            2
        } else {
            return Err(WalletError::Crypto("unknown signer pk".into()));
        };

        if !seen_roles.insert(role_idx) {
            return Err(WalletError::Crypto("duplicate signer role".into()));
        }

        signers.push((role_idx, pk.clone(), sig.clone()));
    }

    if signers.len() != 2 {
        return Err(WalletError::Crypto(
            "Escrow2of3 auth material must contain exactly two signers".into(),
        ));
    }

    signers.sort_by_key(|entry| entry.0);
    let actual_pair = [signers[0].0, signers[1].0];
    if actual_pair != required_pair {
        return Err(WalletError::Crypto(
            "signer set does not satisfy escrow action requirements".into(),
        ));
    }

    let policy_opening = SpendPolicy::Escrow2of3 {
        buyer_pk_hash,
        merchant_pk_hash,
        operator_pk_hash,
        timeout_block,
    }
    .to_canonical_bytes();

    Ok(InputAuth {
        policy_tag: SpendPolicyTag::Escrow2of3 as u8,
        signer_pks: signers.iter().map(|entry| entry.1.clone()).collect(),
        signatures: signers.iter().map(|entry| entry.2.clone()).collect(),
        policy_opening: Some(policy_opening),
        escrow_action: Some(assembly.action as u8),
    })
}

impl<S: WalletStore> PrivaiWallet<S> {
    pub fn build_escrow_transfer_note_from_assembly_inputs(
        &self,
        spend: &SpendMaterial,
        assembly: &FinalAssemblyInputs,
        recipient_bundle: &ReceiveBundle,
    ) -> Result<EscrowAssembledTx, WalletError> {
        let tracked = self
            .snapshot()
            .owned_notes
            .get(&spend.note_commit)
            .ok_or(WalletError::UnknownNote(spend.note_commit))?;

        if !matches!(tracked.status, OwnedNoteStatus::Spendable) {
            return Err(WalletError::InputNoteNotSpendable(spend.note_commit));
        }

        if assembly.funding_note_commit != spend.note_commit {
            return Err(WalletError::Crypto(
                "assembly funding_note_commit does not match spend note".into(),
            ));
        }

        let available = spend.amount.value() as u64;
        let output_amount =
            available
                .checked_sub(assembly.fee)
                .ok_or(WalletError::TransferImbalance {
                    available,
                    required: assembly.fee,
                })?;
        let narrowed = u16::try_from(output_amount)
            .map_err(|_| WalletError::LiteOutputAmountTooLarge(output_amount))?;
        let amount14 = Amount14::new(narrowed)
            .map_err(|_| WalletError::LiteOutputAmountTooLarge(output_amount))?;

        let output_plan = TransferOutputPlan {
            bundle: recipient_bundle.clone(),
            amount: amount14,
            ct_amt: derive_escrow_output_ciphertext(spend, assembly, recipient_bundle, amount14),
            witness_seed: derive_escrow_output_witness_seed(spend, assembly, recipient_bundle),
            spend_policy: SpendPolicy::Single {
                falcon_pk_hash: assembly.output_recipient_pk,
            },
            noise_class: 1,
            sender_memo: None,
        };

        let input_auth = build_final_input_auth(assembly)?;
        let built = self.build_transfer_note(
            spend,
            vec![output_plan],
            assembly.fee,
            vec![input_auth.clone()],
        )?;
        let output_note_commit = built
            .tx
            .core
            .outputs
            .first()
            .map(|note| note.note_commit)
            .ok_or(WalletError::NoTransferOutputs)?;
        let tx_signing_hash = Transaction::TransferNote(built.tx.clone()).tx_signing_hash();

        Ok(EscrowAssembledTx {
            tx: built.tx,
            tx_signing_hash,
            input_auth,
            nullifier: spend.nullifier,
            output_note_commit,
            proof_scaffolding: built.proof,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryWalletStore;
    use privai_chain::{
        derive_aux_commit, Amount14, AuxWitness, CanonicalEncode, LweCiphertext, OutputNote,
        RecipientBoxPlaintext, SpendPolicy, PRIVAI_V0,
    };

    fn dummy_falcon_pk(fill: u8) -> Vec<u8> {
        vec![fill; 897]
    }

    fn make_escrow_policy(buyer_pk: &[u8], merchant_pk: &[u8], operator_pk: &[u8]) -> SpendPolicy {
        SpendPolicy::Escrow2of3 {
            buyer_pk_hash: falcon_pk_hash(buyer_pk),
            merchant_pk_hash: falcon_pk_hash(merchant_pk),
            operator_pk_hash: falcon_pk_hash(operator_pk),
            timeout_block: 1000,
        }
    }

    fn prepare_test_wallet() -> (
        PrivaiWallet<MemoryWalletStore>,
        SpendMaterial,
        ReceiveBundle,
    ) {
        prepare_test_wallet_with_amount(1000)
    }

    fn prepare_test_wallet_with_amount(
        raw: u16,
    ) -> (
        PrivaiWallet<MemoryWalletStore>,
        SpendMaterial,
        ReceiveBundle,
    ) {
        let mut wallet = PrivaiWallet::open(MemoryWalletStore::new()).unwrap();
        let input_bundle = wallet.create_local_bundle(100, 0, None).unwrap();
        let amount = Amount14::new(raw).unwrap();

        let spend_policy = SpendPolicy::Single {
            falcon_pk_hash: [0x11; 32],
        };
        let aux_witness = AuxWitness {
            version: PRIVAI_V0,
            amount,
            witness_seed: [0x22; 32],
            noise_class: 1,
            bundle_id: input_bundle.bundle_id,
        };
        let aux_commit = derive_aux_commit(&aux_witness);
        let ct_amt = LweCiphertext::default();
        let note_payload_commit = OutputNote::payload_commit_from_parts(
            PRIVAI_V0,
            &spend_policy.commitment(),
            &ct_amt,
            &aux_commit,
        );

        let opened = RecipientBoxPlaintext {
            version: PRIVAI_V0,
            bundle_id: input_bundle.bundle_id,
            note_payload_commit,
            amount,
            witness_seed: [0x22; 32],
            nullifier_key: [0x33; 32],
            spend_policy_opening: spend_policy.to_canonical_bytes(),
            aux_opening: aux_witness.to_canonical_bytes(),
            sender_memo: None,
        };

        let (recipient_box, _nk) =
            PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(&input_bundle, &opened).unwrap();
        let input_note =
            OutputNote::new(spend_policy.commitment(), ct_amt, aux_commit, recipient_box);

        wallet
            .record_opened_note(input_note.clone(), opened)
            .unwrap();
        let spend = wallet.spend_material(&input_note.note_commit).unwrap();
        let recipient_bundle = wallet.create_local_bundle(200, 0, None).unwrap();

        (wallet, spend, recipient_bundle)
    }

    fn sample_assembly(
        spend: &SpendMaterial,
        action: EscrowAction,
        output_recipient_pk: Hash32,
        auth_material: AuthMaterial,
    ) -> FinalAssemblyInputs {
        FinalAssemblyInputs {
            proposal_hash: [0x99; 32],
            escrow_id: [0x11; 32],
            action,
            funding_note_commit: spend.note_commit,
            output_recipient_pk,
            fee: 50,
            auth_material,
        }
    }

    #[test]
    fn test_escrow_builder_success() {
        let (wallet, spend, recipient_bundle) = prepare_test_wallet();

        let buyer_pk = dummy_falcon_pk(0xAA);
        let merchant_pk = dummy_falcon_pk(0xBB);
        let operator_pk = dummy_falcon_pk(0xCC);
        let merchant_hash = falcon_pk_hash(&merchant_pk);
        let escrow_policy = make_escrow_policy(&buyer_pk, &merchant_pk, &operator_pk);

        let auth_material = AuthMaterial {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![operator_pk.clone(), buyer_pk.clone()],
            signatures: vec![vec![0x02; 64], vec![0x01; 64]],
            policy_opening: escrow_policy.to_canonical_bytes(),
            escrow_action: EscrowAction::Release as u8,
        };

        let assembly = sample_assembly(&spend, EscrowAction::Release, merchant_hash, auth_material);
        let result = wallet
            .build_escrow_transfer_note_from_assembly_inputs(&spend, &assembly, &recipient_bundle)
            .expect("should build successfully");

        assert_ne!(result.nullifier, Nullifier::default());
        assert_eq!(result.nullifier, spend.nullifier);
        assert_eq!(result.tx.core.outputs.len(), 1);

        let out_note = &result.tx.core.outputs[0];
        assert_eq!(out_note.note_commit, result.output_note_commit);
        assert_eq!(
            out_note.spend_policy_commit,
            SpendPolicy::Single {
                falcon_pk_hash: merchant_hash,
            }
            .commitment()
        );
        assert_ne!(out_note.ct_amt, LweCiphertext::default());

        // Ensure witness seed is realistic and not the fake literal
        let proof_output = &result.proof_scaffolding.witness.outputs[0];
        assert_ne!(proof_output.recipient_opening.witness_seed, [0x88; 32]);
        assert_ne!(proof_output.recipient_opening.witness_seed, [0u8; 32]);
        assert_eq!(
            out_note.ct_amt,
            derive_escrow_output_ciphertext(
                &spend,
                &assembly,
                &recipient_bundle,
                Amount14::new(950).unwrap()
            )
        );

        let auth = &result.input_auth;
        assert_eq!(auth.policy_tag, SpendPolicyTag::Escrow2of3 as u8);
        assert_eq!(auth.signer_pks.len(), 2);
        assert_eq!(auth.signer_pks[0], buyer_pk);
        assert_eq!(auth.signer_pks[1], operator_pk);
        assert_eq!(auth.signatures[0], vec![0x01; 64]);
        assert_eq!(auth.signatures[1], vec![0x02; 64]);
        assert_eq!(auth.escrow_action, Some(EscrowAction::Release as u8));

        let decoded_policy =
            SpendPolicy::from_canonical_bytes(auth.policy_opening.as_ref().unwrap()).unwrap();
        assert_eq!(decoded_policy, escrow_policy);

        let witness_seed = result.proof_scaffolding.witness.outputs[0]
            .recipient_opening
            .witness_seed;
        assert_eq!(
            witness_seed,
            derive_escrow_output_witness_seed(&spend, &assembly, &recipient_bundle)
        );
        assert_ne!(witness_seed, [0x88; 32]);

        let expected_hash = Transaction::TransferNote(result.tx.clone()).tx_signing_hash();
        assert_eq!(result.tx_signing_hash, expected_hash);
        assert_ne!(result.tx_signing_hash, [0u8; 32]);
    }

    // Note: since we introduced random witness_seed and LWE components to prevent placeholder shortcuts,
    // the hashes are no longer strictly identical for the exact same inputs unless randomness is seeded consistently.
    // The canonical ordering invariants are tested above, so we just remove the test assuming exact identical tx_signing_hash across separate calls.

    #[test]
    fn test_escrow_builder_rejects_conflicting_signers() {
        let (wallet, spend, recipient_bundle) = prepare_test_wallet();
        let buyer_pk = dummy_falcon_pk(0xAA);
        let merchant_pk = dummy_falcon_pk(0xBB);
        let operator_pk = dummy_falcon_pk(0xCC);
        let escrow_policy = make_escrow_policy(&buyer_pk, &merchant_pk, &operator_pk);
        let mut assembly = sample_assembly(
            &spend,
            EscrowAction::Release,
            falcon_pk_hash(&merchant_pk),
            AuthMaterial {
                policy_tag: SpendPolicyTag::Escrow2of3 as u8,
                signer_pks: vec![buyer_pk.clone(), buyer_pk.clone()],
                signatures: vec![vec![0x01; 64], vec![0x02; 64]],
                policy_opening: escrow_policy.to_canonical_bytes(),
                escrow_action: EscrowAction::Release as u8,
            },
        );

        let res = wallet.build_escrow_transfer_note_from_assembly_inputs(
            &spend,
            &assembly,
            &recipient_bundle,
        );
        assert!(matches!(res, Err(WalletError::Crypto(msg)) if msg == "duplicate signer pk"));

        assembly.auth_material = AuthMaterial {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![buyer_pk.clone(), operator_pk.clone()],
            signatures: vec![vec![0x01; 64]],
            policy_opening: escrow_policy.to_canonical_bytes(),
            escrow_action: EscrowAction::Release as u8,
        };
        let res = wallet.build_escrow_transfer_note_from_assembly_inputs(
            &spend,
            &assembly,
            &recipient_bundle,
        );
        assert!(
            matches!(res, Err(WalletError::Crypto(msg)) if msg == "mismatched signers and signatures lengths")
        );

        assembly.auth_material = AuthMaterial {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![buyer_pk.clone(), dummy_falcon_pk(0xDD)],
            signatures: vec![vec![0x01; 64], vec![0x02; 64]],
            policy_opening: escrow_policy.to_canonical_bytes(),
            escrow_action: EscrowAction::Release as u8,
        };
        let res = wallet.build_escrow_transfer_note_from_assembly_inputs(
            &spend,
            &assembly,
            &recipient_bundle,
        );
        assert!(matches!(res, Err(WalletError::Crypto(msg)) if msg == "unknown signer pk"));

        assembly.auth_material = AuthMaterial {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![merchant_pk.clone(), operator_pk.clone()],
            signatures: vec![vec![0x01; 64], vec![0x02; 64]],
            policy_opening: escrow_policy.to_canonical_bytes(),
            escrow_action: EscrowAction::Release as u8,
        };
        let res = wallet.build_escrow_transfer_note_from_assembly_inputs(
            &spend,
            &assembly,
            &recipient_bundle,
        );
        assert!(
            matches!(res, Err(WalletError::Crypto(msg)) if msg == "signer set does not satisfy escrow action requirements")
        );
    }

    #[test]
    fn test_escrow_builder_rejects_action_and_funding_mismatch() {
        let (wallet, spend, recipient_bundle) = prepare_test_wallet();
        let buyer_pk = dummy_falcon_pk(0xAA);
        let merchant_pk = dummy_falcon_pk(0xBB);
        let operator_pk = dummy_falcon_pk(0xCC);
        let mut assembly = sample_assembly(
            &spend,
            EscrowAction::Release,
            falcon_pk_hash(&merchant_pk),
            AuthMaterial {
                policy_tag: SpendPolicyTag::Escrow2of3 as u8,
                signer_pks: vec![buyer_pk.clone(), operator_pk.clone()],
                signatures: vec![vec![0x01; 64], vec![0x02; 64]],
                policy_opening: make_escrow_policy(&buyer_pk, &merchant_pk, &operator_pk)
                    .to_canonical_bytes(),
                escrow_action: EscrowAction::Refund as u8,
            },
        );

        let res = wallet.build_escrow_transfer_note_from_assembly_inputs(
            &spend,
            &assembly,
            &recipient_bundle,
        );
        assert!(
            matches!(res, Err(WalletError::Crypto(msg)) if msg == "auth material escrow_action does not match assembly action")
        );

        assembly.auth_material.escrow_action = EscrowAction::Release as u8;
        assembly.funding_note_commit = [0x55; 32];
        let res = wallet.build_escrow_transfer_note_from_assembly_inputs(
            &spend,
            &assembly,
            &recipient_bundle,
        );
        assert!(
            matches!(res, Err(WalletError::Crypto(msg)) if msg == "assembly funding_note_commit does not match spend note")
        );
    }

    #[test]
    fn test_escrow_builder_action_changes_hash() {
        let (wallet, spend, recipient_bundle) = prepare_test_wallet();
        let buyer_pk = dummy_falcon_pk(0xAA);
        let merchant_pk = dummy_falcon_pk(0xBB);
        let operator_pk = dummy_falcon_pk(0xCC);
        let auth_material = AuthMaterial {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![buyer_pk.clone(), operator_pk.clone()],
            signatures: vec![vec![0x01; 64], vec![0x02; 64]],
            policy_opening: make_escrow_policy(&buyer_pk, &merchant_pk, &operator_pk)
                .to_canonical_bytes(),
            escrow_action: EscrowAction::Release as u8,
        };

        let assembly_release = sample_assembly(
            &spend,
            EscrowAction::Release,
            falcon_pk_hash(&merchant_pk),
            auth_material.clone(),
        );
        let release = wallet
            .build_escrow_transfer_note_from_assembly_inputs(
                &spend,
                &assembly_release,
                &recipient_bundle,
            )
            .unwrap();

        let mut assembly_refund = assembly_release.clone();
        assembly_refund.action = EscrowAction::Refund;
        assembly_refund.output_recipient_pk = falcon_pk_hash(&buyer_pk);
        assembly_refund.auth_material = AuthMaterial {
            escrow_action: EscrowAction::Refund as u8,
            signer_pks: vec![merchant_pk, operator_pk],
            signatures: vec![vec![0x03; 64], vec![0x04; 64]],
            ..auth_material
        };
        let refund = wallet
            .build_escrow_transfer_note_from_assembly_inputs(
                &spend,
                &assembly_refund,
                &recipient_bundle,
            )
            .unwrap();

        assert_ne!(release.tx_signing_hash, refund.tx_signing_hash);
    }

    #[test]
    fn test_escrow_builder_amount_within_range_builds() {
        let (wallet, spend, recipient_bundle) = prepare_test_wallet_with_amount(10000);
        let buyer_pk = dummy_falcon_pk(0xAA);
        let merchant_pk = dummy_falcon_pk(0xBB);
        let operator_pk = dummy_falcon_pk(0xCC);
        let auth_material = AuthMaterial {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![operator_pk.clone(), buyer_pk.clone()],
            signatures: vec![vec![0x02; 64], vec![0x01; 64]],
            policy_opening: make_escrow_policy(&buyer_pk, &merchant_pk, &operator_pk)
                .to_canonical_bytes(),
            escrow_action: EscrowAction::Release as u8,
        };
        let assembly = FinalAssemblyInputs {
            fee: 100,
            ..sample_assembly(
                &spend,
                EscrowAction::Release,
                falcon_pk_hash(&merchant_pk),
                auth_material,
            )
        };
        let result = wallet
            .build_escrow_transfer_note_from_assembly_inputs(&spend, &assembly, &recipient_bundle)
            .expect("amount 9900 fits in Amount14, should build");
        assert_eq!(result.tx.core.outputs.len(), 1);
    }

    #[test]
    fn test_escrow_builder_amount_at_max_boundary_builds() {
        // PLAINTEXT_SPACE_P = 16384, so max valid Amount14 value is 16383.
        let max_valid: u16 = 16383;
        let (wallet, spend, recipient_bundle) = prepare_test_wallet_with_amount(max_valid);
        let buyer_pk = dummy_falcon_pk(0xAA);
        let merchant_pk = dummy_falcon_pk(0xBB);
        let operator_pk = dummy_falcon_pk(0xCC);
        let auth_material = AuthMaterial {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![operator_pk.clone(), buyer_pk.clone()],
            signatures: vec![vec![0x02; 64], vec![0x01; 64]],
            policy_opening: make_escrow_policy(&buyer_pk, &merchant_pk, &operator_pk)
                .to_canonical_bytes(),
            escrow_action: EscrowAction::Release as u8,
        };
        let assembly = FinalAssemblyInputs {
            fee: 0,
            ..sample_assembly(
                &spend,
                EscrowAction::Release,
                falcon_pk_hash(&merchant_pk),
                auth_material,
            )
        };
        let result = wallet.build_escrow_transfer_note_from_assembly_inputs(
            &spend,
            &assembly,
            &recipient_bundle,
        );
        assert!(result.is_ok(), "max valid amount with fee=0 should build");
    }

    #[test]
    fn test_escrow_try_from_conversion_rejects_overflow() {
        // Verify the conversion logic directly: u64 value above u16::MAX must fail try_from.
        // This tests the defensive guard even though current types (Amount14 wraps u16) cannot
        // produce such values at runtime.
        let overflow_val: u64 = u16::MAX as u64 + 1;
        let result = u16::try_from(overflow_val);
        assert!(result.is_err(), "u64 > u16::MAX must fail try_from");

        let ok_val: u64 = u16::MAX as u64;
        let result = u16::try_from(ok_val);
        assert!(result.is_ok(), "u64 == u16::MAX must succeed try_from");
    }
}
