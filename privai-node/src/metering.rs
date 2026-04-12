//! Metering agent for compute miner.
//!
//! Produces signed, hash-chained telemetry records per window.
//! Aggregates into ComputeLeaseReceipt at session end.
//!
//! This module runs on the miner's machine.
//! It does NOT touch the chain — it produces evidence for settlement.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use privai_chain::canonical::{write_bytes, write_fixed, write_u64, write_u8, CanonicalEncode};
use privai_chain::compute_lease::{ComputeLeaseReceipt, HeartbeatStatus, PrivacyClass};
use privai_chain::hash::{domain_hash, merkle_root};
use privai_chain::primitives::Hash32;

// ── Domain separators ──────────────────────────────────────────────────

const ENV_FINGERPRINT_DOMAIN: &str = "privai:env-fingerprint:v1";
const STARTUP_MANIFEST_DOMAIN: &str = "privai:startup-manifest:v1";
const WINDOW_TELEMETRY_DOMAIN: &str = "privai:window-telemetry:v1";

// ── Environment Fingerprint ────────────────────────────────────────────

/// Snapshot of the system environment at session start.
///
/// Proves that the environment was in a known state when the session began.
/// Any change during the session can be detected.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentFingerprint {
    /// Hash of the agent binary.
    pub binary_hash: Hash32,
    /// Hash of the agent configuration file.
    pub config_hash: Hash32,
    /// OS kernel version hash.
    pub os_kernel_hash: Hash32,
    /// GPU driver version hash (if GPU class).
    pub gpu_driver_hash: Option<Hash32>,
    /// CUDA/ROCm version hash (if GPU class).
    pub compute_runtime_hash: Option<Hash32>,
    /// List of running process hashes (top N by resource usage).
    pub process_hashes: Vec<Hash32>,
    /// Disk fingerprint (hash of partition table / mount points).
    pub disk_fingerprint: Hash32,
    /// Timestamp of fingerprint creation.
    pub timestamp_unix: u64,
}

impl EnvironmentFingerprint {
    pub fn commitment(&self) -> Hash32 {
        domain_hash(ENV_FINGERPRINT_DOMAIN, &[&self.to_canonical_bytes()])
    }
}

impl CanonicalEncode for EnvironmentFingerprint {
    fn encode(&self, out: &mut Vec<u8>) {
        write_fixed(out, &self.binary_hash);
        write_fixed(out, &self.config_hash);
        write_fixed(out, &self.os_kernel_hash);
        match &self.gpu_driver_hash {
            Some(h) => {
                write_u8(out, 1);
                write_fixed(out, h);
            }
            None => write_u8(out, 0),
        }
        match &self.compute_runtime_hash {
            Some(h) => {
                write_u8(out, 1);
                write_fixed(out, h);
            }
            None => write_u8(out, 0),
        }
        write_u64(out, self.process_hashes.len() as u64);
        for h in &self.process_hashes {
            write_fixed(out, h);
        }
        write_fixed(out, &self.disk_fingerprint);
        write_u64(out, self.timestamp_unix);
    }
}

// ── Startup Manifest ───────────────────────────────────────────────────

/// Signed evidence that the session started with a known environment.
///
/// Produced once at session start. Signed by miner.
/// User verifies the signature and stores the manifest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartupManifest {
    /// Session ID (from escrow).
    pub session_id: Hash32,
    /// Environment fingerprint commitment.
    pub env_fingerprint_commit: Hash32,
    /// Miner's role key hash.
    pub miner_role_key_hash: Hash32,
    /// Timestamp of session start.
    pub start_timestamp_unix: u64,
    /// Block height at session start.
    pub start_block_height: u64,
    /// Falcon signature by miner.
    pub miner_signature: Vec<u8>,
}

impl StartupManifest {
    pub fn commitment(&self) -> Hash32 {
        domain_hash(STARTUP_MANIFEST_DOMAIN, &[&self.to_canonical_bytes()])
    }
}

impl CanonicalEncode for StartupManifest {
    fn encode(&self, out: &mut Vec<u8>) {
        write_fixed(out, &self.session_id);
        write_fixed(out, &self.env_fingerprint_commit);
        write_fixed(out, &self.miner_role_key_hash);
        write_u64(out, self.start_timestamp_unix);
        write_u64(out, self.start_block_height);
        write_bytes(out, &self.miner_signature);
    }
}

// ── Window Telemetry Record ────────────────────────────────────────────

/// Single window measurement — the core of metering.
///
/// Hash-chained: each record references the hash of the previous record.
/// This creates an unbreakable chain — any tampering breaks the chain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowTelemetryRecord {
    /// Sequential window index (0-based).
    pub window_index: u32,
    /// Block height at which this window started.
    pub window_start_height: u64,
    /// Challenge hash for this window.
    pub challenge_hash: Hash32,
    /// Response time in milliseconds.
    pub response_time_ms: u32,
    /// Availability result.
    pub availability: bool,
    /// Performance result (None = no benchmark in this window).
    pub performance: Option<bool>,
    /// Heartbeat status.
    pub heartbeat: HeartbeatStatus,
    /// GPU utilization percentage (0-100), if applicable.
    pub gpu_utilization: Option<u8>,
    /// CPU utilization percentage (0-100).
    pub cpu_utilization: Option<u8>,
    /// RAM usage in MB.
    pub ram_used_mb: Option<u32>,
    /// Hash of the previous window record.
    /// First window (index 0) uses startup_manifest_commit.
    pub previous_record_hash: Hash32,
    /// Miner signature on this record.
    pub miner_signature: Vec<u8>,
}

impl WindowTelemetryRecord {
    /// Compute the hash of this record (for chaining).
    pub fn record_hash(&self) -> Hash32 {
        domain_hash(WINDOW_TELEMETRY_DOMAIN, &[&self.to_canonical_bytes()])
    }

    /// Verify that this record chains to the previous one.
    pub fn chains_to(&self, previous_hash: &Hash32) -> bool {
        &self.previous_record_hash == previous_hash
    }
}

impl CanonicalEncode for WindowTelemetryRecord {
    fn encode(&self, out: &mut Vec<u8>) {
        write_fixed(out, &self.window_index.to_le_bytes());
        write_u64(out, self.window_start_height);
        write_fixed(out, &self.challenge_hash);
        write_fixed(out, &self.response_time_ms.to_le_bytes());
        write_u8(out, if self.availability { 1 } else { 0 });
        match self.performance {
            Some(true) => write_u8(out, 1),
            Some(false) => write_u8(out, 2),
            None => write_u8(out, 0),
        }
        write_u8(out, self.heartbeat as u8);
        match self.gpu_utilization {
            Some(v) => {
                write_u8(out, 1);
                write_u8(out, v);
            }
            None => write_u8(out, 0),
        }
        match self.cpu_utilization {
            Some(v) => {
                write_u8(out, 1);
                write_u8(out, v);
            }
            None => write_u8(out, 0),
        }
        match self.ram_used_mb {
            Some(v) => {
                write_u8(out, 1);
                write_fixed(out, &v.to_le_bytes());
            }
            None => write_u8(out, 0),
        }
        write_fixed(out, &self.previous_record_hash);
        write_bytes(out, &self.miner_signature);
    }
}

// ── Agent State ────────────────────────────────────────────────────────

/// The metering agent that runs on the miner's machine.
///
/// Collects telemetry, builds hash-chain, produces receipt at session end.
pub struct MeteringAgent {
    session_id: Hash32,
    miner_role_key_hash: Hash32,
    startup_manifest: StartupManifest,
    env_fingerprint: EnvironmentFingerprint,
    window_records: Vec<WindowTelemetryRecord>,
    /// Hash of the last record (for chaining).
    last_record_hash: Hash32,
}

/// Error type for metering operations.
#[derive(Debug, Clone)]
pub enum MeteringError {
    /// Window index is not sequential.
    WindowIndexMismatch { expected: u32, got: u32 },
    /// Record does not chain to previous.
    ChainBreak { window_index: u32 },
    /// Session already finalized.
    SessionAlreadyFinalized,
    /// No windows recorded.
    NoWindows,
    /// Signature verification failed.
    InvalidSignature,
}

impl std::fmt::Display for MeteringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WindowIndexMismatch { expected, got } => {
                write!(f, "window index mismatch: expected {expected}, got {got}")
            }
            Self::ChainBreak { window_index } => {
                write!(f, "hash chain break at window {window_index}")
            }
            Self::SessionAlreadyFinalized => write!(f, "session already finalized"),
            Self::NoWindows => write!(f, "no windows recorded"),
            Self::InvalidSignature => write!(f, "invalid signature"),
        }
    }
}

impl std::error::Error for MeteringError {}

impl MeteringAgent {
    /// Create a new metering agent for a session.
    pub fn new(
        session_id: Hash32,
        miner_role_key_hash: Hash32,
        env_fingerprint: EnvironmentFingerprint,
        start_block_height: u64,
    ) -> Self {
        let env_commit = env_fingerprint.commitment();

        // Startup manifest (unsigned — miner signs externally)
        let startup_manifest = StartupManifest {
            session_id,
            env_fingerprint_commit: env_commit,
            miner_role_key_hash,
            start_timestamp_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            start_block_height,
            miner_signature: vec![], // to be filled by miner
        };

        let manifest_commit = startup_manifest.commitment();

        Self {
            session_id,
            miner_role_key_hash,
            startup_manifest,
            env_fingerprint,
            window_records: Vec::new(),
            last_record_hash: manifest_commit,
        }
    }

    /// Record a window telemetry measurement.
    ///
    /// This is called once per window by the metering loop.
    /// The miner signs the record externally and passes the signature.
    pub fn record_window(
        &mut self,
        window_index: u32,
        window_start_height: u64,
        challenge_hash: Hash32,
        response_time_ms: u32,
        availability: bool,
        performance: Option<bool>,
        heartbeat: HeartbeatStatus,
        gpu_utilization: Option<u8>,
        cpu_utilization: Option<u8>,
        ram_used_mb: Option<u32>,
        miner_signature: Vec<u8>,
    ) -> Result<Hash32, MeteringError> {
        // Verify sequential index
        let expected_index = self.window_records.len() as u32;
        if window_index != expected_index {
            return Err(MeteringError::WindowIndexMismatch {
                expected: expected_index,
                got: window_index,
            });
        }

        let record = WindowTelemetryRecord {
            window_index,
            window_start_height,
            challenge_hash,
            response_time_ms,
            availability,
            performance,
            heartbeat,
            gpu_utilization,
            cpu_utilization,
            ram_used_mb,
            previous_record_hash: self.last_record_hash,
            miner_signature,
        };

        let record_hash = record.record_hash();
        self.window_records.push(record);
        self.last_record_hash = record_hash;

        Ok(record_hash)
    }

    /// Verify that the hash chain is intact.
    pub fn verify_chain(&self) -> Result<(), MeteringError> {
        if self.window_records.is_empty() {
            return Ok(());
        }

        let mut expected_hash = self.startup_manifest.commitment();

        for record in &self.window_records {
            if !record.chains_to(&expected_hash) {
                return Err(MeteringError::ChainBreak {
                    window_index: record.window_index,
                });
            }
            expected_hash = record.record_hash();
        }

        Ok(())
    }

    /// Count passed and degraded windows.
    pub fn count_results(&self) -> (u32, u32) {
        let mut passed = 0u32;
        let mut degraded = 0u32;

        for record in &self.window_records {
            if record.availability {
                match record.performance {
                    Some(true) | None => passed += 1, // PASS or N/A = PASS
                    Some(false) => degraded += 1,     // availability PASS, performance FAIL
                }
            }
            // availability FAIL = not counted (neither passed nor degraded)
        }

        (passed, degraded)
    }

    /// Finalize the session and produce a ComputeLeaseReceipt.
    ///
    /// Verifies chain integrity, computes Merkle root of window hashes,
    /// and produces the aggregate receipt.
    pub fn finalize(
        &self,
        lease_policy_commit: Hash32,
        meter_version: u8,
    ) -> Result<ComputeLeaseReceipt, MeteringError> {
        // Verify chain integrity
        self.verify_chain()?;

        if self.window_records.is_empty() {
            return Err(MeteringError::NoWindows);
        }

        // Count results
        let (passed, degraded) = self.count_results();
        let total = self.window_records.len() as u32;

        // Compute Merkle root of window hashes
        let window_hashes: Vec<Hash32> = self
            .window_records
            .iter()
            .map(|r| r.record_hash())
            .collect();
        let window_hashes_root = merkle_root(window_hashes);

        // Build receipt (unsigned — miner signs externally)
        let receipt = ComputeLeaseReceipt {
            session_id: self.session_id,
            total_windows: total,
            passed_windows: passed,
            degraded_windows: degraded,
            window_hashes_root,
            lease_policy_commit,
            miner_role_key_hash: self.miner_role_key_hash,
            meter_version,
            miner_signature: vec![], // to be filled by miner
        };

        Ok(receipt)
    }

    /// Get the startup manifest.
    pub fn startup_manifest(&self) -> &StartupManifest {
        &self.startup_manifest
    }

    /// Get the environment fingerprint.
    pub fn env_fingerprint(&self) -> &EnvironmentFingerprint {
        &self.env_fingerprint
    }

    /// Get all window records.
    pub fn window_records(&self) -> &[WindowTelemetryRecord] {
        &self.window_records
    }

    /// Get the number of recorded windows.
    pub fn window_count(&self) -> usize {
        self.window_records.len()
    }
}

// ── Builder helpers ────────────────────────────────────────────────────

impl EnvironmentFingerprint {
    /// Create a placeholder fingerprint (for testing).
    pub fn placeholder() -> Self {
        Self {
            binary_hash: [0x01; 32],
            config_hash: [0x02; 32],
            os_kernel_hash: [0x03; 32],
            gpu_driver_hash: Some([0x04; 32]),
            compute_runtime_hash: Some([0x05; 32]),
            process_hashes: vec![[0x06; 32], [0x07; 32]],
            disk_fingerprint: [0x08; 32],
            timestamp_unix: 0,
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION_ID: Hash32 = [0xAA; 32];
    const MINER_KEY_HASH: Hash32 = [0xBB; 32];
    const POLICY_COMMIT: Hash32 = [0xCC; 32];

    fn test_agent() -> MeteringAgent {
        MeteringAgent::new(
            SESSION_ID,
            MINER_KEY_HASH,
            EnvironmentFingerprint::placeholder(),
            1000,
        )
    }

    fn record_window(
        agent: &mut MeteringAgent,
        index: u32,
        availability: bool,
        performance: Option<bool>,
    ) {
        agent
            .record_window(
                index,
                1000 + (index as u64 * 60),
                [index as u8; 32],
                50, // 50ms response time
                availability,
                performance,
                HeartbeatStatus::Active,
                Some(75),
                Some(30),
                Some(16_000),
                vec![0xDD; 64],
            )
            .expect("record window");
    }

    #[test]
    fn agent_records_windows_sequentially() {
        let mut agent = test_agent();
        record_window(&mut agent, 0, true, Some(true));
        record_window(&mut agent, 1, true, Some(true));
        record_window(&mut agent, 2, true, Some(false)); // degraded
        assert_eq!(agent.window_count(), 3);
    }

    #[test]
    fn agent_rejects_non_sequential_index() {
        let mut agent = test_agent();
        record_window(&mut agent, 0, true, Some(true));
        // Skip index 1, try to record index 2
        let result = agent.record_window(
            2,
            1120,
            [2; 32],
            50,
            true,
            Some(true),
            HeartbeatStatus::Active,
            None,
            None,
            None,
            vec![],
        );
        assert!(result.is_err());
    }

    #[test]
    fn hash_chain_is_valid() {
        let mut agent = test_agent();
        record_window(&mut agent, 0, true, Some(true));
        record_window(&mut agent, 1, true, Some(true));
        record_window(&mut agent, 2, true, Some(true));
        assert!(agent.verify_chain().is_ok());
    }

    #[test]
    fn count_results_full_completion() {
        let mut agent = test_agent();
        for i in 0..100 {
            record_window(&mut agent, i, true, Some(true));
        }
        let (passed, degraded) = agent.count_results();
        assert_eq!(passed, 100);
        assert_eq!(degraded, 0);
    }

    #[test]
    fn count_results_with_degraded() {
        let mut agent = test_agent();
        // 80 passed, 10 degraded, 10 failed
        for i in 0..80 {
            record_window(&mut agent, i, true, Some(true));
        }
        for i in 80..90 {
            record_window(&mut agent, i, true, Some(false)); // degraded
        }
        for i in 90..100 {
            record_window(&mut agent, i, false, None); // failed
        }
        let (passed, degraded) = agent.count_results();
        assert_eq!(passed, 80);
        assert_eq!(degraded, 10);
    }

    #[test]
    fn count_results_n_a_performance_is_passed() {
        let mut agent = test_agent();
        // performance = None means N/A (no benchmark in this window)
        // N/A should count as passed
        agent
            .record_window(
                0,
                1000,
                [0; 32],
                50,
                true,
                None, // N/A
                HeartbeatStatus::Active,
                None,
                None,
                None,
                vec![],
            )
            .unwrap();
        let (passed, degraded) = agent.count_results();
        assert_eq!(passed, 1); // N/A = PASS
        assert_eq!(degraded, 0);
    }

    #[test]
    fn finalize_produces_receipt() {
        let mut agent = test_agent();
        for i in 0..1440 {
            record_window(&mut agent, i, true, Some(true));
        }
        let receipt = agent.finalize(POLICY_COMMIT, 1).expect("finalize");
        assert_eq!(receipt.session_id, SESSION_ID);
        assert_eq!(receipt.total_windows, 1440);
        assert_eq!(receipt.passed_windows, 1440);
        assert_eq!(receipt.degraded_windows, 0);
        assert_eq!(receipt.lease_policy_commit, POLICY_COMMIT);
        assert_eq!(receipt.miner_role_key_hash, MINER_KEY_HASH);
        assert_eq!(receipt.meter_version, 1);
        // window_hashes_root should not be zero
        assert_ne!(receipt.window_hashes_root, [0; 32]);
    }

    #[test]
    fn finalize_fails_on_empty_session() {
        let agent = test_agent();
        let result = agent.finalize(POLICY_COMMIT, 1);
        assert!(result.is_err());
    }

    #[test]
    fn finalize_fails_on_broken_chain() {
        let mut agent = test_agent();
        record_window(&mut agent, 0, true, Some(true));
        // Manually tamper with the chain
        agent.window_records[0].previous_record_hash = [0xFF; 32]; // break chain
        let result = agent.finalize(POLICY_COMMIT, 1);
        assert!(result.is_err());
    }

    #[test]
    fn receipt_matches_settlement_calculation() {
        use privai_chain::compute_lease::{
            calculate_settlement, ComputeLeasePolicy, GpuClass, ResourceClass, SettlementMode,
        };

        let mut agent = test_agent();
        // 1368 passed, 0 degraded, 72 failed out of 1440
        for i in 0..1368 {
            record_window(&mut agent, i, true, Some(true));
        }
        for i in 1368..1440 {
            record_window(&mut agent, i, false, None);
        }

        let receipt = agent.finalize(POLICY_COMMIT, 1).unwrap();

        let policy = ComputeLeasePolicy {
            version: 1,
            resource_class: ResourceClass::Gpu {
                class: GpuClass::A100,
                vram_mb: 80_000,
            },
            min_duration_units: 1440,
            max_duration_units: 1440,
            price_aPVA_per_unit: 33_333_333_333,
            privacy_class: PrivacyClass::Vm,
            network_mode: privai_chain::compute_lease::NetworkMode::TorGated,
            settlement_mode: SettlementMode::ProRata,
            meter_version: 1,
            timeout_blocks: 100,
            window_duration_blocks: 60,
            total_windows: 1440,
            benchmark_floor_ms: 150,
            benchmark_interval: 10,
            degraded_weight_permille: 500,
        };

        let result = calculate_settlement(48_000_000_000_000, &receipt, &policy);
        assert_eq!(result.miner_share, 45_600_000_000_000); // 95%
        assert_eq!(result.user_share, 2_400_000_000_000);
        assert!(result.is_balanced(48_000_000_000_000));
    }

    #[test]
    fn window_hashes_root_is_deterministic() {
        let mut agent1 = test_agent();
        let mut agent2 = test_agent();
        for i in 0..10 {
            record_window(&mut agent1, i, true, Some(true));
            record_window(&mut agent2, i, true, Some(true));
        }
        let r1 = agent1.finalize(POLICY_COMMIT, 1).unwrap();
        let r2 = agent2.finalize(POLICY_COMMIT, 1).unwrap();
        assert_eq!(r1.window_hashes_root, r2.window_hashes_root);
    }
}
