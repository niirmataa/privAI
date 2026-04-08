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
    #[error("chain_id mismatch: expected {expected}, got {actual}")]
    InvalidChainId { expected: u32, actual: u32 },
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
    #[error("missing operator signature on MarketplaceBatchTx")]
    MissingOperatorSignature,
    #[error("invalid operator signature on MarketplaceBatchTx")]
    InvalidOperatorSignature,
    #[error("missing auth for transaction (Zero Trust requires Falcon signature)")]
    MissingAuth,
    #[error("auth entries count does not match inputs count")]
    AuthCountMismatch,
    #[error("missing policy_opening in auth")]
    MissingPolicyOpening,
    #[error("policy_opening does not decode to a valid policy: {0}")]
    PolicyDecode(String),
    #[error("policy_opening commitment does not match note's spend_policy_commit")]
    PolicyMismatch,
    #[error("policy_tag does not match derived policy type")]
    PolicyTagMismatch,
    #[error("auth requires exactly 1 signer for Single, got {0}")]
    InvalidSingleSignerCount(usize),
    // ── Escrow-specific errors ────────────────────────────────────────
    #[error("escrow: policy_opening missing for escrow-2of3 auth")]
    EscrowMissingPolicyOpening,
    #[error("escrow: escrow_action missing for escrow-2of3 auth")]
    EscrowMissingAction,
    #[error("escrow: invalid action byte {0:#x}")]
    EscrowInvalidAction(u8),
    #[error("escrow: policy_opening does not decode to a valid policy: {0}")]
    EscrowPolicyDecode(String),
    #[error("escrow: policy_opening decodes to non-Escrow2of3 policy type")]
    EscrowUnsupportedPolicy,
    #[error("escrow: policy_opening commitment does not match note's spend_policy_commit")]
    EscrowPolicyMismatch,
    #[error("escrow: signer pk_hash {0:?} not found in policy")]
    EscrowUnknownSigner(Hash32),
    #[error("escrow: duplicate signer in auth")]
    EscrowDuplicateSigner,
    #[error("escrow: signers not in canonical order (by policy index)")]
    EscrowSignerOrderViolation,
    #[error("escrow: signer combination does not match declared action")]
    EscrowWrongSignerCombination,
    #[error("escrow: recovery before timeout (current block {current}, required {required})")]
    EscrowRecoveryBeforeTimeout { current: u64, required: u64 },
    #[error("escrow: no output matches expected recipient for action")]
    EscrowOutputTargetMismatch,
    #[error("escrow: auth requires exactly 2 signers, got {0}")]
    EscrowWrongSignerCount(usize),
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
