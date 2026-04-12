//! Compute lease types for privAI V0.
//!
//! These types define the V0 private compute lease model:
//! - resource classification
//! - lease policy
//! - compute offerings
//! - settlement modes
//! - metering receipts
//!
//! All types use existing hash and canonical encoding infrastructure.
//! No existing types are modified.

use serde::{Deserialize, Serialize};

use crate::canonical::{write_bytes, write_fixed, write_u64, write_u8, CanonicalEncode};
use crate::hash::domain_hash;
use crate::primitives::Hash32;

// ── Domain separators ──────────────────────────────────────────────────

const COMPUTE_LEASE_POLICY_DOMAIN: &str = "privai:compute-lease-policy:v1";
const COMPUTE_LEASE_RECEIPT_DOMAIN: &str = "privai:compute-lease-receipt:v1";

// ── Resource Classification ────────────────────────────────────────────

/// GPU class identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum GpuClass {
    A100 = 0x01,
    H100 = 0x02,
    V100 = 0x03,
    T4 = 0x04,
    /// Generic GPU with explicit performance level.
    Generic(u8) = 0xFF,
}

impl GpuClass {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::A100),
            0x02 => Some(Self::H100),
            0x03 => Some(Self::V100),
            0x04 => Some(Self::T4),
            0xFF => Some(Self::Generic(0xFF)),
            _ => None,
        }
    }

    pub fn tag(&self) -> u8 {
        match self {
            Self::A100 => 0x01,
            Self::H100 => 0x02,
            Self::V100 => 0x03,
            Self::T4 => 0x04,
            Self::Generic(_) => 0xFF,
        }
    }
}

/// CPU tier identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum CpuTier {
    X86_64 = 0x01,
    Arm64 = 0x02,
    Generic = 0xFF,
}

impl CpuTier {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::X86_64),
            0x02 => Some(Self::Arm64),
            0xFF => Some(Self::Generic),
            _ => None,
        }
    }

    pub fn tag(&self) -> u8 {
        match self {
            Self::X86_64 => 0x01,
            Self::Arm64 => 0x02,
            Self::Generic => 0xFF,
        }
    }
}

/// Resource class — what kind of compute is being offered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceClass {
    Gpu {
        class: GpuClass,
        vram_mb: u32,
    },
    Cpu {
        tier: CpuTier,
        cores: u16,
    },
    Memory {
        ram_mb: u32,
    },
    Composite {
        gpu: Option<(GpuClass, u32)>, // (class, vram_mb)
        cpu: Option<(CpuTier, u16)>,  // (tier, cores)
        ram_mb: u32,
        storage_mb: u64,
    },
}

impl ResourceClass {
    pub fn tag(&self) -> u8 {
        match self {
            Self::Gpu { .. } => 0x01,
            Self::Cpu { .. } => 0x02,
            Self::Memory { .. } => 0x03,
            Self::Composite { .. } => 0x04,
        }
    }
}

// ── Privacy / Network / Settlement Modes ───────────────────────────────

/// Privacy class — isolation level of the runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PrivacyClass {
    Vm = 0x01,
    Container = 0x02,
    Sandbox = 0x03,
    ConfidentialRuntime = 0x04,
}

impl PrivacyClass {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::Vm),
            0x02 => Some(Self::Container),
            0x03 => Some(Self::Sandbox),
            0x04 => Some(Self::ConfidentialRuntime),
            _ => None,
        }
    }
}

/// Network mode — what network access the runtime has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NetworkMode {
    Isolated = 0x01,
    NxmsOnly = 0x02,
    TorGated = 0x03,
    InternetExit = 0x04,
}

impl NetworkMode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::Isolated),
            0x02 => Some(Self::NxmsOnly),
            0x03 => Some(Self::TorGated),
            0x04 => Some(Self::InternetExit),
            _ => None,
        }
    }
}

/// Settlement mode — how escrow divides funds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SettlementMode {
    AllOrNothing = 0x01,
    ProRata = 0x02,
}

impl SettlementMode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::AllOrNothing),
            0x02 => Some(Self::ProRata),
            _ => None,
        }
    }
}

// ── Identity Roles ─────────────────────────────────────────────────────

/// Node role types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RoleType {
    Validator = 0x01,
    ComputeMiner = 0x02,
    Relay = 0x03,
    Mailbox = 0x04,
    ExitNode = 0x05,
}

impl RoleType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::Validator),
            0x02 => Some(Self::ComputeMiner),
            0x03 => Some(Self::Relay),
            0x04 => Some(Self::Mailbox),
            0x05 => Some(Self::ExitNode),
            _ => None,
        }
    }
}

// ── Heartbeat Status ───────────────────────────────────────────────────

/// Heartbeat status for liveness checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum HeartbeatStatus {
    Active = 0x01,
    Missed = 0x02,
    Terminated = 0x03,
}

impl HeartbeatStatus {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::Active),
            0x02 => Some(Self::Missed),
            0x03 => Some(Self::Terminated),
            _ => None,
        }
    }
}

// ── Compute Lease Policy ───────────────────────────────────────────────

/// Lease policy — the terms of a compute lease.
///
/// Hashed on-chain as `lease_policy_commit`.
/// Full policy is off-chain (exchanged during negotiation).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeLeasePolicy {
    pub version: u8,
    pub resource_class: ResourceClass,
    pub min_duration_units: u64,
    pub max_duration_units: u64,
    pub price_aPVA_per_unit: u64, // LedgerAmount = u64
    pub privacy_class: PrivacyClass,
    pub network_mode: NetworkMode,
    pub settlement_mode: SettlementMode,
    pub meter_version: u8,
    pub timeout_blocks: u64,
    /// Window duration in blocks. E.g., 60 blocks ≈ 30 min at 30s/block.
    pub window_duration_blocks: u64,
    /// Total windows in the session.
    pub total_windows: u32,
    /// Performance benchmark floor in milliseconds.
    /// If benchmark time > floor → performance FAIL.
    pub benchmark_floor_ms: u32,
    /// Benchmark interval in windows. E.g., 10 = benchmark every 10th window.
    pub benchmark_interval: u32,
    /// Weight for degraded windows (availability PASS, performance FAIL).
    /// Default: 500 = 50.0% (in permille to avoid floats).
    pub degraded_weight_permille: u16,
}

impl ComputeLeasePolicy {
    /// Compute the policy commitment hash.
    pub fn commitment(&self) -> Hash32 {
        domain_hash(COMPUTE_LEASE_POLICY_DOMAIN, &[&self.to_canonical_bytes()])
    }

    /// Effective windows for settlement calculation.
    pub fn effective_windows(&self, passed: u32, degraded: u32) -> u64 {
        let degraded_contribution = (degraded as u64 * self.degraded_weight_permille as u64) / 1000;
        passed as u64 + degraded_contribution
    }
}

impl CanonicalEncode for ComputeLeasePolicy {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.version);
        write_u8(out, self.resource_class.tag());
        // Resource class details
        match &self.resource_class {
            ResourceClass::Gpu { class, vram_mb } => {
                write_u8(out, class.tag());
                write_fixed(out, &vram_mb.to_le_bytes());
            }
            ResourceClass::Cpu { tier, cores } => {
                write_u8(out, tier.tag() as u8);
                write_fixed(out, &cores.to_le_bytes());
            }
            ResourceClass::Memory { ram_mb } => {
                write_fixed(out, &ram_mb.to_le_bytes());
            }
            ResourceClass::Composite {
                gpu,
                cpu,
                ram_mb,
                storage_mb,
            } => {
                match gpu {
                    Some((gc, vram)) => {
                        write_u8(out, 1);
                        write_u8(out, gc.tag());
                        write_fixed(out, &vram.to_le_bytes());
                    }
                    None => write_u8(out, 0),
                }
                match cpu {
                    Some((ct, cores)) => {
                        write_u8(out, 1);
                        write_u8(out, ct.tag() as u8);
                        write_fixed(out, &cores.to_le_bytes());
                    }
                    None => write_u8(out, 0),
                }
                write_fixed(out, &ram_mb.to_le_bytes());
                write_fixed(out, &storage_mb.to_le_bytes());
            }
        }
        write_u64(out, self.min_duration_units);
        write_u64(out, self.max_duration_units);
        write_u64(out, self.price_aPVA_per_unit);
        write_u8(out, self.privacy_class as u8);
        write_u8(out, self.network_mode as u8);
        write_u8(out, self.settlement_mode as u8);
        write_u8(out, self.meter_version);
        write_u64(out, self.timeout_blocks);
        write_u64(out, self.window_duration_blocks);
        write_fixed(out, &self.total_windows.to_le_bytes());
        write_fixed(out, &self.benchmark_floor_ms.to_le_bytes());
        write_fixed(out, &self.benchmark_interval.to_le_bytes());
        write_fixed(out, &self.degraded_weight_permille.to_le_bytes());
    }
}

// ── Compute Offering ───────────────────────────────────────────────────

/// Compute offering — what a miner advertises in discovery.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeOffering {
    pub resource_class: ResourceClass,
    pub price_aPVA_per_unit: u64,
    pub min_duration_units: u64,
    pub max_duration_units: u64,
    pub network_mode: NetworkMode,
    pub privacy_class: PrivacyClass,
    pub scoped_offering_id: Hash32,
    pub availability_start: u64,
    pub availability_end: u64,
    pub meter_version: u8,
    pub miner_role_key_hash: Hash32,
    pub benchmark_floor_ms: u32,
}

// ── Compute Lease Receipt ──────────────────────────────────────────────

/// Aggregate receipt — the evidence from a compute session.
///
/// Produced by the miner. Verified by the user.
/// Contains aggregate window results, not per-window telemetry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeLeaseReceipt {
    pub session_id: Hash32,
    pub total_windows: u32,
    pub passed_windows: u32,
    pub degraded_windows: u32,
    /// Merkle root of per-window hashes.
    /// Off-chain: full hashes available on demand (for dispute).
    pub window_hashes_root: Hash32,
    pub lease_policy_commit: Hash32,
    pub miner_role_key_hash: Hash32,
    pub meter_version: u8,
    pub miner_signature: Vec<u8>,
}

impl ComputeLeaseReceipt {
    /// Compute the receipt commitment hash.
    pub fn commitment(&self) -> Hash32 {
        domain_hash(COMPUTE_LEASE_RECEIPT_DOMAIN, &[&self.to_canonical_bytes()])
    }
}

impl CanonicalEncode for ComputeLeaseReceipt {
    fn encode(&self, out: &mut Vec<u8>) {
        write_fixed(out, &self.session_id);
        write_fixed(out, &self.total_windows.to_le_bytes());
        write_fixed(out, &self.passed_windows.to_le_bytes());
        write_fixed(out, &self.degraded_windows.to_le_bytes());
        write_fixed(out, &self.window_hashes_root);
        write_fixed(out, &self.lease_policy_commit);
        write_fixed(out, &self.miner_role_key_hash);
        write_u8(out, self.meter_version);
        write_bytes(out, &self.miner_signature);
    }
}

// ── Settlement Calculation ─────────────────────────────────────────────

/// Calculate miner share from receipt and policy.
///
/// Integer arithmetic only. No floats.
/// Remainder goes to user.
pub fn calculate_settlement(
    total_amount: u64,
    receipt: &ComputeLeaseReceipt,
    policy: &ComputeLeasePolicy,
) -> SettlementResult {
    if receipt.total_windows == 0 {
        return SettlementResult {
            miner_share: 0,
            user_share: total_amount,
            effective_windows: 0,
            total_windows: 0,
        };
    }

    let effective = policy.effective_windows(receipt.passed_windows, receipt.degraded_windows);
    let miner_share = total_amount * effective / receipt.total_windows as u64;
    let user_share = total_amount - miner_share;

    SettlementResult {
        miner_share,
        user_share,
        effective_windows: effective,
        total_windows: receipt.total_windows as u64,
    }
}

/// Settlement result — what the chain executes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettlementResult {
    pub miner_share: u64,
    pub user_share: u64,
    pub effective_windows: u64,
    pub total_windows: u64,
}

impl SettlementResult {
    /// Verify that shares add up to total amount.
    pub fn is_balanced(&self, total_amount: u64) -> bool {
        self.miner_share + self.user_share == total_amount
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_full_completion() {
        let policy = test_policy();
        let receipt = test_receipt(1440, 1440, 0);
        let result = calculate_settlement(48_000_000_000_000, &receipt, &policy);
        assert_eq!(result.miner_share, 48_000_000_000_000);
        assert_eq!(result.user_share, 0);
        assert!(result.is_balanced(48_000_000_000_000));
    }

    #[test]
    fn settlement_full_refund() {
        let policy = test_policy();
        let receipt = test_receipt(1440, 0, 0);
        let result = calculate_settlement(48_000_000_000_000, &receipt, &policy);
        assert_eq!(result.miner_share, 0);
        assert_eq!(result.user_share, 48_000_000_000_000);
    }

    #[test]
    fn settlement_pro_rata() {
        let policy = test_policy();
        // 1368 passed, 0 degraded, out of 1440
        let receipt = test_receipt(1440, 1368, 0);
        let result = calculate_settlement(48_000_000_000_000, &receipt, &policy);
        // 1368 / 1440 = 95%
        assert_eq!(result.miner_share, 45_600_000_000_000);
        assert_eq!(result.user_share, 2_400_000_000_000);
        assert!(result.is_balanced(48_000_000_000_000));
    }

    #[test]
    fn settlement_with_degraded() {
        let policy = test_policy(); // degraded_weight = 500 (50%)
                                    // 1360 passed, 40 degraded, 40 failed out of 1440
        let receipt = test_receipt(1440, 1360, 40);
        let result = calculate_settlement(48_000_000_000_000, &receipt, &policy);
        // effective = 1360 + (40 * 0.5) = 1380
        // miner = 48T * 1380 / 1440 = 46T
        assert_eq!(result.effective_windows, 1380);
        assert_eq!(result.miner_share, 46_000_000_000_000);
        assert_eq!(result.user_share, 2_000_000_000_000);
        assert!(result.is_balanced(48_000_000_000_000));
    }

    #[test]
    fn settlement_remainder_goes_to_user() {
        let policy = test_policy();
        // 5 passed out of 7 total
        let receipt = test_receipt(7, 5, 0);
        let result = calculate_settlement(100, &receipt, &policy);
        // 100 * 5 / 7 = 71 (integer)
        // user = 100 - 71 = 29
        assert_eq!(result.miner_share, 71);
        assert_eq!(result.user_share, 29);
        assert!(result.is_balanced(100));
    }

    #[test]
    fn policy_commitment_is_deterministic() {
        let policy = test_policy();
        let c1 = policy.commitment();
        let c2 = policy.commitment();
        assert_eq!(c1, c2);
    }

    #[test]
    fn receipt_commitment_is_deterministic() {
        let receipt = test_receipt(1440, 1368, 0);
        let c1 = receipt.commitment();
        let c2 = receipt.commitment();
        assert_eq!(c1, c2);
    }

    #[test]
    fn enum_roundtrips() {
        assert_eq!(
            PrivacyClass::from_u8(PrivacyClass::Vm as u8),
            Some(PrivacyClass::Vm)
        );
        assert_eq!(
            PrivacyClass::from_u8(PrivacyClass::Container as u8),
            Some(PrivacyClass::Container)
        );
        assert_eq!(
            NetworkMode::from_u8(NetworkMode::Isolated as u8),
            Some(NetworkMode::Isolated)
        );
        assert_eq!(
            NetworkMode::from_u8(NetworkMode::InternetExit as u8),
            Some(NetworkMode::InternetExit)
        );
        assert_eq!(
            SettlementMode::from_u8(SettlementMode::ProRata as u8),
            Some(SettlementMode::ProRata)
        );
        assert_eq!(
            RoleType::from_u8(RoleType::ComputeMiner as u8),
            Some(RoleType::ComputeMiner)
        );
        assert_eq!(
            HeartbeatStatus::from_u8(HeartbeatStatus::Active as u8),
            Some(HeartbeatStatus::Active)
        );
        assert!(PrivacyClass::from_u8(0xFF).is_none());
        assert!(NetworkMode::from_u8(0x00).is_none());
    }

    // ── Helpers ─────────────────────────────────────────────────────

    fn test_policy() -> ComputeLeasePolicy {
        ComputeLeasePolicy {
            version: 1,
            resource_class: ResourceClass::Gpu {
                class: GpuClass::A100,
                vram_mb: 80_000,
            },
            min_duration_units: 1440,
            max_duration_units: 1440,
            price_aPVA_per_unit: 33_333_333_333, // ~2 PVA/h if window=30min
            privacy_class: PrivacyClass::Vm,
            network_mode: NetworkMode::TorGated,
            settlement_mode: SettlementMode::ProRata,
            meter_version: 1,
            timeout_blocks: 100,
            window_duration_blocks: 60,
            total_windows: 1440,
            benchmark_floor_ms: 150,
            benchmark_interval: 10,
            degraded_weight_permille: 500, // 50%
        }
    }

    fn test_receipt(total: u32, passed: u32, degraded: u32) -> ComputeLeaseReceipt {
        ComputeLeaseReceipt {
            session_id: [0xAA; 32],
            total_windows: total,
            passed_windows: passed,
            degraded_windows: degraded,
            window_hashes_root: [0xBB; 32],
            lease_policy_commit: [0xCC; 32],
            miner_role_key_hash: [0xDD; 32],
            meter_version: 1,
            miner_signature: vec![0xEE; 64],
        }
    }
}
