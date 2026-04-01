use std::path::Path;

use serde::{Deserialize, Serialize};

use privai_chain::{DEFAULT_CHAIN_ID, EpochParams, Hash32};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorConfig {
    pub pk_hash: Hash32,
    pub stake_weight: u64,
    pub availability: u32,
    pub proof_score: u32,
}

impl ValidatorConfig {
    pub fn score(&self) -> u128 {
        self.stake_weight as u128
            * self.availability.max(1) as u128
            * self.proof_score.max(1) as u128
    }
}

/// Domyślny timeout rundy konsensusu dla Tor (30s).
/// Tor ma ~200-500ms latency na hop, 3+ hops → round-trip ~1-3s.
/// Dodajemy bufor na processing + sieć.
pub const DEFAULT_CONSENSUS_TIMEOUT_MS: u64 = 30_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeConfig {
    pub chain_id: u32,
    pub node_pk_hash: Hash32,
    pub data_dir: String,
    pub epoch_params: EpochParams,
    pub max_block_txs: usize,
    pub validators: Vec<ValidatorConfig>,
    /// Timeout rundy konsensusu w ms. Domyślnie 30s dla Tor.
    /// Można obniżyć dla testów lokalnych.
    pub consensus_timeout_ms: u64,
}

impl NodeConfig {
    pub fn example() -> Self {
        Self {
            chain_id: DEFAULT_CHAIN_ID,
            node_pk_hash: [7; 32],
            data_dir: "./var/privai-node".to_string(),
            epoch_params: EpochParams {
                epoch_number: 0,
                start_height: 1,
                end_height: 10_000,
                min_validator_stake: 1_000,
                min_prover_bond: 100,
                max_block_bytes: 1_000_000,
                max_block_statements: 2_048,
                min_proof_coverage: 1,
            },
            max_block_txs: 256,
            validators: vec![ValidatorConfig {
                pk_hash: [7; 32],
                stake_weight: 100,
                availability: 1,
                proof_score: 1,
            }],
            consensus_timeout_ms: DEFAULT_CONSENSUS_TIMEOUT_MS,
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let contents = std::fs::read_to_string(path)?;
        toml::from_str(&contents).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
