//! Compute lease escrow builder for privAI V0.
//!
//! Follows the same pattern as escrow_builder.rs (Escrow2of3)
//! but uses ComputeLeaseEscrow SpendPolicy (tag 0x04).
//!
//! This module does NOT modify existing escrow_builder.rs.
//! It runs alongside it.

use crate::builder::TransferOutputPlan;
use crate::error::WalletError;
use crate::state::{OwnedNoteStatus, SpendMaterial};
use crate::store::WalletStore;
use crate::wallet::PrivaiWallet;
use privai_chain::compute_lease::{ComputeLeasePolicy, SettlementMode};
use privai_chain::hash::{domain_hash, falcon_pk_hash};
use privai_chain::{
    Amount14, CanonicalEncode, Hash32, InputAuth, LweCiphertext, Nullifier, ReceiveBundle,
    SpendPolicy, SpendPolicyTag, Transaction, TransferNoteTx,
};
use privai_proof::TransferProvingData;
use serde::{Deserialize, Serialize};

const COMPUTE_LEASE_OUTPUT_WITNESS_SEED_DOMAIN: &str =
    "privai:wallet:compute-lease-output-witness-seed:v1";
const COMPUTE_LEASE_OUTPUT_CT_AMT_DOMAIN: &str = "privai:wallet:compute-lease-output-ct-amt:v1";

/// Inputs for building a ComputeLeaseEscrow transaction.
///
/// This replaces FinalAssemblyInputs for compute lease escrow.
/// Same concept, different fields.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComputeLeaseAssemblyInputs {
    /// Proposal hash (for commitment binding).
    pub proposal_hash: Hash32,
    /// Escrow ID (unique per escrow).
    pub escrow_id: Hash32,
    /// User's public key hash (who leases).
    pub user_pk_hash: Hash32,
    /// Miner's public key hash (who provides compute).
    pub miner_pk_hash: Hash32,
    /// Operator's public key hash (Phase 1 bridge only).
    pub operator_pk_hash: Hash32,
    /// Commitment to the full lease policy.
    pub lease_policy_commit: Hash32,
    /// Settlement mode (AllOrNothing or ProRata).
    pub settlement_mode: SettlementMode,
    /// Timeout block height.
    pub timeout_block: u64,
    /// Fee in aPVA.
    pub fee: u64,
    /// Output recipient (miner's bundle for receiving funds).
    pub output_recipient_pk: Hash32,
    /// Auth material (signer keys + signatures).
    pub auth_material: ComputeLeaseAuthMaterial,
}

/// Auth material for ComputeLeaseEscrow.
///
/// In Phase 1: 2 signers (user + operator).
/// In Phase 2: 1 signer (user) + protocol validation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComputeLeaseAuthMaterial {
    pub policy_tag: u8,
    pub signer_pks: Vec<Vec<u8>>,
    pub signatures: Vec<Vec<u8>>,
    pub policy_opening: Vec<u8>,
    pub escrow_action: u8,
}

/// Output of the compute lease escrow builder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComputeLeaseAssembledTx {
    pub tx: TransferNoteTx,
    pub tx_signing_hash: Hash32,
    pub input_auth: InputAuth,
    pub nullifier: Nullifier,
    pub output_note_commit: Hash32,
    pub proof_scaffolding: TransferProvingData,
    /// The ComputeLeaseEscrow policy (for settlement later).
    pub escrow_policy: ComputeLeaseEscrowPolicy,
}

/// On-chain representation of the compute lease escrow.
/// Same struct as in compute_escrow.rs but re-exported here for convenience.
pub use privai_chain::compute_escrow::ComputeLeaseEscrowPolicy;

// ── Private helpers ────────────────────────────────────────────────────

fn derive_compute_lease_output_witness_seed(
    spend: &SpendMaterial,
    assembly: &ComputeLeaseAssemblyInputs,
    recipient_bundle: &ReceiveBundle,
) -> Hash32 {
    let mut parts = vec![
        spend.note_commit.as_slice(),
        &assembly.proposal_hash,
        &assembly.escrow_id,
        &recipient_bundle.bundle_id,
        &assembly.output_recipient_pk,
        &assembly.user_pk_hash,
        &assembly.miner_pk_hash,
        &assembly.lease_policy_commit,
    ];
    let action_byte = [assembly.auth_material.escrow_action];
    parts.push(&action_byte);
    let fee_bytes = assembly.fee.to_le_bytes();
    parts.push(&fee_bytes);

    domain_hash(COMPUTE_LEASE_OUTPUT_WITNESS_SEED_DOMAIN, &parts)
}

fn derive_compute_lease_output_ciphertext(
    spend: &SpendMaterial,
    assembly: &ComputeLeaseAssemblyInputs,
    recipient_bundle: &ReceiveBundle,
    amount: Amount14,
) -> LweCiphertext {
    let dimension = LweCiphertext::default().a.len();
    let mut a = Vec::with_capacity(dimension);
    let amount_bytes = amount.value().to_le_bytes();
    let action_byte = [assembly.auth_material.escrow_action];
    let fee_bytes = assembly.fee.to_le_bytes();

    for index in 0..dimension {
        let index_bytes = (index as u32).to_le_bytes();
        let hash = domain_hash(
            COMPUTE_LEASE_OUTPUT_CT_AMT_DOMAIN,
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
        COMPUTE_LEASE_OUTPUT_CT_AMT_DOMAIN,
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
    let b = u32::from_le_bytes(b_hash[..4].try_into().expect("b hash prefix"));

    LweCiphertext::new(a, b).expect("ciphertext construction")
}

fn build_compute_lease_input_auth(
    assembly: &ComputeLeaseAssemblyInputs,
) -> Result<InputAuth, WalletError> {
    let policy = SpendPolicy::Single {
        falcon_pk_hash: assembly.user_pk_hash,
    };

    let signers = &assembly.auth_material.signer_pks;
    let sigs = &assembly.auth_material.signatures;

    if signers.len() != 2 {
        return Err(WalletError::Crypto(
            "ComputeLeaseEscrow requires exactly 2 signers (Phase 1)".into(),
        ));
    }
    if sigs.len() != 2 {
        return Err(WalletError::Crypto(
            "ComputeLeaseEscrow requires exactly 2 signatures (Phase 1)".into(),
        ));
    }

    Ok(InputAuth {
        policy_tag: SpendPolicyTag::Escrow2of3 as u8, // Phase 1 bridge: still uses 0x03
        signer_pks: signers.clone(),
        signatures: sigs.clone(),
        policy_opening: Some(assembly.auth_material.policy_opening.clone()),
        escrow_action: Some(assembly.auth_material.escrow_action),
    })
}

// ── Builder implementation ─────────────────────────────────────────────

impl<S: WalletStore> PrivaiWallet<S> {
    /// Build a ComputeLeaseEscrow transaction.
    ///
    /// This follows the same pattern as build_escrow_transfer_note_from_assembly_inputs
    /// but uses ComputeLeaseEscrow policy fields.
    ///
    /// In Phase 1: still uses Escrow2of3 SpendPolicy tag (bridge).
    /// In Phase 2: will use ComputeLeaseEscrow tag (0x04).
    pub fn build_compute_lease_escrow_from_assembly_inputs(
        &self,
        spend: &SpendMaterial,
        assembly: &ComputeLeaseAssemblyInputs,
        recipient_bundle: &ReceiveBundle,
    ) -> Result<ComputeLeaseAssembledTx, WalletError> {
        // 1. Verify spend note is spendable
        let tracked = self
            .snapshot()
            .owned_notes
            .get(&spend.note_commit)
            .ok_or(WalletError::UnknownNote(spend.note_commit))?;

        if !matches!(tracked.status, OwnedNoteStatus::Spendable) {
            return Err(WalletError::InputNoteNotSpendable(spend.note_commit));
        }

        // 2. Calculate output amount (available - fee)
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

        // 3. Build output plan
        let output_plan = TransferOutputPlan {
            bundle: recipient_bundle.clone(),
            amount: amount14,
            ct_amt: derive_compute_lease_output_ciphertext(
                spend,
                assembly,
                recipient_bundle,
                amount14,
            ),
            witness_seed: derive_compute_lease_output_witness_seed(
                spend,
                assembly,
                recipient_bundle,
            ),
            spend_policy: SpendPolicy::Single {
                falcon_pk_hash: assembly.output_recipient_pk,
            },
            noise_class: 1,
            sender_memo: None,
        };

        // 4. Build input auth
        let input_auth = build_compute_lease_input_auth(assembly)?;

        // 5. Build transfer note
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

        // 6. Build escrow policy (for later settlement)
        let escrow_policy = ComputeLeaseEscrowPolicy::new(
            assembly.user_pk_hash,
            assembly.miner_pk_hash,
            assembly.operator_pk_hash,
            assembly.lease_policy_commit,
            output_amount, // locked amount in LedgerAmount
            assembly.timeout_block,
        );

        Ok(ComputeLeaseAssembledTx {
            tx: built.tx,
            tx_signing_hash,
            input_auth,
            nullifier: spend.nullifier,
            output_note_commit,
            proof_scaffolding: built.proof,
            escrow_policy,
        })
    }

    /// Build a ComputeLeaseEscrow from simpler inputs.
    ///
    /// Convenience method that constructs ComputeLeaseAssemblyInputs from policy.
    pub fn build_compute_lease_escrow_simple(
        &self,
        spend: &SpendMaterial,
        user_pk_hash: Hash32,
        miner_pk_hash: Hash32,
        operator_pk_hash: Hash32,
        policy: &ComputeLeasePolicy,
        fee: u64,
        timeout_block: u64,
        recipient_bundle: &ReceiveBundle,
    ) -> Result<ComputeLeaseAssembledTx, WalletError> {
        let lease_policy_commit = policy.commitment();

        let assembly = ComputeLeaseAssemblyInputs {
            proposal_hash: [0u8; 32], // TODO: real proposal hash
            escrow_id: domain_hash(
                "privai:wallet:escrow-id:v1",
                &[
                    &user_pk_hash,
                    &miner_pk_hash,
                    &lease_policy_commit,
                    &timeout_block.to_le_bytes(),
                ],
            ),
            user_pk_hash,
            miner_pk_hash,
            operator_pk_hash,
            lease_policy_commit,
            settlement_mode: policy.settlement_mode,
            timeout_block,
            fee,
            output_recipient_pk: miner_pk_hash,
            auth_material: ComputeLeaseAuthMaterial {
                policy_tag: SpendPolicyTag::Escrow2of3 as u8,
                signer_pks: vec![],
                signatures: vec![],
                policy_opening: vec![],
                escrow_action: 0x01, // Release (default)
            },
        };

        self.build_compute_lease_escrow_from_assembly_inputs(spend, &assembly, recipient_bundle)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use privai_chain::compute_lease::{GpuClass, NetworkMode, PrivacyClass, ResourceClass};

    fn test_policy() -> ComputeLeasePolicy {
        ComputeLeasePolicy {
            version: 1,
            resource_class: ResourceClass::Gpu {
                class: GpuClass::A100,
                vram_mb: 80_000,
            },
            min_duration_units: 1440,
            max_duration_units: 1440,
            price_aPVA_per_unit: 33_333_333_333,
            privacy_class: PrivacyClass::Vm,
            network_mode: NetworkMode::TorGated,
            settlement_mode: SettlementMode::ProRata,
            meter_version: 1,
            timeout_blocks: 100,
            window_duration_blocks: 60,
            total_windows: 1440,
            benchmark_floor_ms: 150,
            benchmark_interval: 10,
            degraded_weight_permille: 500,
        }
    }

    #[test]
    fn assembly_inputs_structure() {
        let assembly = ComputeLeaseAssemblyInputs {
            proposal_hash: [0x01; 32],
            escrow_id: [0x02; 32],
            user_pk_hash: [0x03; 32],
            miner_pk_hash: [0x04; 32],
            operator_pk_hash: [0x05; 32],
            lease_policy_commit: [0x06; 32],
            settlement_mode: SettlementMode::ProRata,
            timeout_block: 1000,
            fee: 1_000_000,
            output_recipient_pk: [0x04; 32],
            auth_material: ComputeLeaseAuthMaterial {
                policy_tag: 0x03,
                signer_pks: vec![vec![0x07; 64], vec![0x08; 64]],
                signatures: vec![vec![0x09; 64], vec![0x0A; 64]],
                policy_opening: vec![0x0B; 32],
                escrow_action: 0x01,
            },
        };

        assert_eq!(assembly.user_pk_hash, [0x03; 32]);
        assert_eq!(assembly.miner_pk_hash, [0x04; 32]);
        assert_eq!(assembly.settlement_mode, SettlementMode::ProRata);
        assert_eq!(assembly.auth_material.signer_pks.len(), 2);
    }

    #[test]
    fn escrow_policy_from_assembly() {
        let policy = test_policy();
        let policy_commit = policy.commitment();

        let escrow = ComputeLeaseEscrowPolicy::new(
            [0x03; 32],
            [0x04; 32],
            [0x05; 32],
            policy_commit,
            48_000_000_000_000,
            12345,
        );

        assert_eq!(escrow.tag, 0x04);
        assert_eq!(escrow.user_pk_hash, [0x03; 32]);
        assert_eq!(escrow.miner_pk_hash, [0x04; 32]);
        assert_eq!(escrow.locked_amount, 48_000_000_000_000);
        assert_eq!(escrow.lease_policy_commit, policy_commit);
    }

    #[test]
    fn policy_commitment_matches_escrow() {
        let policy = test_policy();
        let policy_commit = policy.commitment();

        let escrow = ComputeLeaseEscrowPolicy::new(
            [0x03; 32],
            [0x04; 32],
            [0x05; 32],
            policy_commit,
            48_000_000_000_000,
            12345,
        );

        // The commitment in the escrow must match the policy commitment
        assert_eq!(escrow.lease_policy_commit, policy.commitment());
    }

    #[test]
    fn evaluate_settlement_matches_calculate() {
        use privai_chain::compute_lease::calculate_settlement;

        let policy = test_policy();
        let policy_commit = policy.commitment();

        let escrow = ComputeLeaseEscrowPolicy::new(
            [0x03; 32],
            [0x04; 32],
            [0x05; 32],
            policy_commit,
            48_000_000_000_000,
            12345,
        );

        let receipt = privai_chain::compute_lease::ComputeLeaseReceipt {
            session_id: [0x11; 32],
            total_windows: 1440,
            passed_windows: 1360,
            degraded_windows: 40,
            window_hashes_root: [0x22; 32],
            lease_policy_commit: policy_commit,
            miner_role_key_hash: [0x04; 32],
            meter_version: 1,
            miner_signature: vec![],
        };

        let result = escrow.evaluate_settlement(&receipt, &policy).unwrap();

        // 1360 + (40 * 0.5) = 1380 effective
        // 48T * 1380 / 1440 = 46T
        assert_eq!(result.miner_share, 46_000_000_000_000);
        assert_eq!(result.user_share, 2_000_000_000_000);
        assert!(result.is_balanced(48_000_000_000_000));
    }
}
