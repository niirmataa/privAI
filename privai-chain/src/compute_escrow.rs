//! Escrow SpendPolicy for Compute Lease (tag 0x04)
//!
//! Architecture (Phase 1 / V0 Simple 4):
//! - Escrow locks PVA based on ComputeLeasePolicy.
//! - Action: Settle (Release + Refund) based on ComputeLeaseReceipt.
//! - Operator signature is required ONLY in Phase 1 (as bootstrap).
//! - Settlement is mathematically deterministic using `calculate_settlement`.

use serde::{Deserialize, Serialize};

use crate::canonical::{write_fixed, write_u64, write_u8, CanonicalEncode};
use crate::compute_lease::{
    calculate_settlement, ComputeLeasePolicy, ComputeLeaseReceipt, SettlementResult,
};
use crate::primitives::Hash32;

pub const COMPUTE_LEASE_ESCROW_TAG: u8 = 0x04;

/// On-chain state representation of the locked compute lease escrow.
/// Replaces generic Escrow2of3 for compute jobs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeLeaseEscrowPolicy {
    pub tag: u8, // Always 0x04
    pub user_pk_hash: Hash32,
    pub miner_pk_hash: Hash32,
    /// In Phase 1, the operator blindly co-signs the deterministic output.
    pub operator_pk_hash: Hash32,
    /// Commitment to the full lease terms (off-chain).
    pub lease_policy_commit: Hash32,
    pub locked_amount: u64,
    pub timeout_block_height: u64,
}

impl ComputeLeaseEscrowPolicy {
    pub fn new(
        user_pk_hash: Hash32,
        miner_pk_hash: Hash32,
        operator_pk_hash: Hash32,
        lease_policy_commit: Hash32,
        locked_amount: u64,
        timeout_block_height: u64,
    ) -> Self {
        Self {
            tag: COMPUTE_LEASE_ESCROW_TAG,
            user_pk_hash,
            miner_pk_hash,
            operator_pk_hash,
            lease_policy_commit,
            locked_amount,
            timeout_block_height,
        }
    }

    /// Evaluates a settlement claim using the receipt.
    /// In Phase 1, this requires operator signature on the transaction itself.
    pub fn evaluate_settlement(
        &self,
        receipt: &ComputeLeaseReceipt,
        full_policy: &ComputeLeasePolicy,
    ) -> Result<SettlementResult, &'static str> {
        // 1. Verify receipt matches the locked policy
        if receipt.lease_policy_commit != self.lease_policy_commit {
            return Err("Receipt does not match locked lease policy");
        }

        // 2. Verify policy commitment matches the provided full policy
        if full_policy.commitment() != self.lease_policy_commit {
            return Err("Provided policy does not match commitment");
        }

        // 3. Verify miner identity
        if receipt.miner_role_key_hash != self.miner_pk_hash {
            return Err("Receipt signed by wrong miner");
        }

        // 4. Calculate deterministic split
        Ok(calculate_settlement(
            self.locked_amount,
            receipt,
            full_policy,
        ))
    }
}

impl CanonicalEncode for ComputeLeaseEscrowPolicy {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.tag);
        write_fixed(out, &self.user_pk_hash);
        write_fixed(out, &self.miner_pk_hash);
        write_fixed(out, &self.operator_pk_hash);
        write_fixed(out, &self.lease_policy_commit);
        write_u64(out, self.locked_amount);
        write_u64(out, self.timeout_block_height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute_lease::{
        GpuClass, NetworkMode, PrivacyClass, ResourceClass, SettlementMode,
    };

    #[test]
    fn test_compute_lease_escrow_settlement() {
        let policy = ComputeLeasePolicy {
            version: 1,
            resource_class: ResourceClass::Gpu {
                class: GpuClass::A100,
                vram_mb: 80_000,
            },
            min_duration_units: 10,
            max_duration_units: 10,
            price_aPVA_per_unit: 100,
            privacy_class: PrivacyClass::Vm,
            network_mode: NetworkMode::TorGated,
            settlement_mode: SettlementMode::ProRata,
            meter_version: 1,
            timeout_blocks: 100,
            window_duration_blocks: 60,
            total_windows: 10,
            benchmark_floor_ms: 150,
            benchmark_interval: 1,
            degraded_weight_permille: 500,
        };

        let policy_commit = policy.commitment();
        let miner_pk = [0xAA; 32];

        let escrow = ComputeLeaseEscrowPolicy::new(
            [0xBB; 32],
            miner_pk,
            [0xCC; 32],
            policy_commit,
            1000,
            12345,
        );

        let receipt = ComputeLeaseReceipt {
            session_id: [0x11; 32],
            total_windows: 10,
            passed_windows: 9,
            degraded_windows: 0,
            window_hashes_root: [0x22; 32],
            lease_policy_commit: policy_commit,
            miner_role_key_hash: miner_pk,
            meter_version: 1,
            miner_signature: vec![],
        };

        let result = escrow.evaluate_settlement(&receipt, &policy).unwrap();
        assert_eq!(result.miner_share, 900); // 90% of 1000
        assert_eq!(result.user_share, 100);
    }
}
