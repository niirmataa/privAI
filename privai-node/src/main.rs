use std::path::PathBuf;

use clap::{Parser, Subcommand};

use privai_chain::{derive_epoch_seed, VoteType};
use privai_ledger::{FileSystemStore, RocksDBStore};
use privai_node::{NodeConfig, PrivaiNode};

#[derive(Debug, Parser)]
#[command(name = "privai-node")]
#[command(about = "privAI node scaffold")]
struct Cli {
    #[arg(long)]
    config: Option<PathBuf>,
    /// Use RocksDB instead of filesystem storage
    #[arg(long)]
    use_rocksdb: bool,
    /// Path to nexum-cli vault export for PQC Identity
    #[arg(long)]
    vault: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    PrintConfig,
    ProposeOnce {
        #[arg(long, default_value_t = 0)]
        epoch: u64,
        #[arg(long, default_value_t = 0)]
        round: u32,
        #[arg(long, default_value_t = 0)]
        timestamp_ms: u64,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::PrintConfig => {
            let example = NodeConfig::example();
            println!("{}", toml::to_string_pretty(&example)?);
        }
        Command::ProposeOnce {
            epoch,
            round,
            timestamp_ms,
        } => {
            let config_path = cli
                .config
                .unwrap_or_else(|| PathBuf::from("privai-node.example.toml"));
            let config = NodeConfig::load(&config_path).unwrap_or_else(|_| NodeConfig::example());

            if cli.use_rocksdb {
                let rocksdb_path = PathBuf::from(&config.data_dir).join("rocksdb");
                let store = RocksDBStore::new(&rocksdb_path)?;
                store.ensure_initialized()?;
                let mut node = PrivaiNode::open(config.clone(), store)?;
                if let Some(vault_path) = &cli.vault {
                    node.load_identity(vault_path)?;
                }
                let epoch_seed_hash = derive_epoch_seed(&[0; 32], epoch);
                let block = node.propose_block(
                    epoch,
                    round,
                    timestamp_ms,
                    epoch_seed_hash,
                    [0; 32],
                    Vec::new(),
                    Vec::new(),
                )?;
                println!(
                    "proposed block height={} txs={} vote_type_hint={:?}",
                    block.header.height,
                    block.body.txs.len(),
                    VoteType::Precommit
                );
            } else {
                let store = FileSystemStore::new(&config.data_dir);
                let mut node = PrivaiNode::open(config.clone(), store)?;
                if let Some(vault_path) = &cli.vault {
                    node.load_identity(vault_path)?;
                }
                let epoch_seed_hash = derive_epoch_seed(&[0; 32], epoch);
                let block = node.propose_block(
                    epoch,
                    round,
                    timestamp_ms,
                    epoch_seed_hash,
                    [0; 32],
                    Vec::new(),
                    Vec::new(),
                )?;
                println!(
                    "proposed block height={} txs={} vote_type_hint={:?}",
                    block.header.height,
                    block.body.txs.len(),
                    VoteType::Precommit
                );
            };
            return Ok(());
        }
    }

    Ok(())
}
