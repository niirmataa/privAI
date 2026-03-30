use std::fs;
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeConfig {
    pub chain_id: u32,
    pub node_pk_hash: Hash32,
    pub data_dir: String,
    pub epoch_params: EpochParams,
    pub max_block_txs: usize,
    pub validators: Vec<ValidatorConfig>,
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
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents).expect("valid node config TOML"))
    }
}
