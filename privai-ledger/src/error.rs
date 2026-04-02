use std::io;

use thiserror::Error;

use privai_chain::{Hash32, Nullifier, TxShapeError};
use privai_proof::ProofError;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error(transparent)]
    Shape(#[from] TxShapeError),
    #[error("duplicate input note in transaction")]
    DuplicateInput,
    #[error("duplicate nullifier in transaction")]
    DuplicateNullifier,
    #[error("input note {0:?} not found")]
    MissingInput(Hash32),
    #[error("input note {0:?} is already spent")]
    InputAlreadySpent(Hash32),
    #[error("nullifier {0:?} already spent")]
    NullifierAlreadySpent(Nullifier),
    #[error("ticket nullifier {0:?} has already been spent")]
    DoubleSpend(Nullifier),
    #[error("output note {0:?} already exists")]
    DuplicateOutput(Hash32),
    #[error("block height {actual} does not extend tip {expected}")]
    InvalidBlockHeight { expected: u64, actual: u64 },
    #[error("block prev hash does not match local tip")]
    InvalidParent,
    #[error("block roots do not match body")]
    InvalidRoots,
    #[error("covered tx index {0} is outside the block body")]
    InvalidCoveredIndex(u32),
    #[error("invalid auth: {0}")]
    InvalidAuth(String),
    #[error("block too large: {size} bytes exceeds max {max} bytes")]
    BlockTooLarge { size: usize, max: usize },
    #[error("too many transactions: {count} exceeds max {max}")]
    TooManyTransactions { count: usize, max: usize },
    #[error("transaction fee too low: {fee} < {min_fee}")]
    FeeTooLow { fee: u64, min_fee: u64 },
    #[error("state root mismatch: expected {expected:?}, got {actual:?}")]
    StateRootMismatch { expected: Hash32, actual: Hash32 },
    #[error(transparent)]
    Proof(#[from] ProofError),
}

#[derive(Debug, Error)]
pub enum MempoolError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("input note {input:?} is already reserved by tx {conflict:?}")]
    InputConflict { input: Hash32, conflict: Hash32 },
    #[error("nullifier {nullifier:?} is already reserved by tx {conflict:?}")]
    NullifierConflict {
        nullifier: Nullifier,
        conflict: Hash32,
    },
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("rocksdb error: {0}")]
    RocksDB(String),
}

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    Mempool(#[from] MempoolError),
    #[error(transparent)]
    Store(#[from] StoreError),
}
