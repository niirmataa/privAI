use std::io;

use privai_chain::{BundleId, DecodeError, Hash32};
use privai_proof::TransferBuildError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletStoreError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum WalletError {
    #[error(transparent)]
    Store(#[from] WalletStoreError),
    #[error("bundle {0:?} is already known to wallet")]
    DuplicateBundle(BundleId),
    #[error("bundle {0:?} is not known to wallet")]
    UnknownBundle(BundleId),
    #[error("bundle {0:?} has already been used")]
    BundleAlreadyUsed(BundleId),
    #[error("note {0:?} is already tracked by wallet")]
    DuplicateNote(Hash32),
    #[error("note {0:?} is not known to wallet")]
    UnknownNote(Hash32),
    #[error("note {0:?} is not currently spendable")]
    InputNoteNotSpendable(Hash32),
    #[error("spend material for note {0:?} does not match wallet state")]
    SpendMaterialMismatch(Hash32),
    #[error("transfer must include at least one output")]
    NoTransferOutputs,
    #[error("transfer amount accounting overflowed while summing outputs and fee")]
    TransferArithmeticOverflow,
    #[error("transfer does not balance exactly: available {available}, required {required}")]
    TransferImbalance { available: u64, required: u64 },
    #[error("lite transfer output amount {0} exceeds Amount14 range (max 16383)")]
    LiteOutputAmountTooLarge(u64),
    #[error("opened recipient box note payload commitment does not match output note")]
    NotePayloadCommitMismatch,
    #[error("output note commitment no longer matches its contents")]
    InvalidNoteCommit,
    #[error("opened recipient box bundle_id does not match note hint")]
    BundleHintMismatch,
    #[error("wallet has no local secret material for bundle {0:?}")]
    MissingLocalKeys(BundleId),
    #[error("unsupported recipient box KEM algorithm id {0:#x}")]
    UnsupportedKemAlg(u8),
    #[error("unsupported recipient box AEAD algorithm id {0:#x}")]
    UnsupportedAeadAlg(u8),
    #[error("opened spend policy does not match note spend_policy_commit")]
    SpendPolicyCommitMismatch,
    #[error("opened aux witness does not match note aux_commit")]
    AuxCommitMismatch,
    #[error("opened aux witness fields do not match recipient box plaintext")]
    AuxWitnessMismatch,
    #[error("recipient box crypto error: {0}")]
    Crypto(String),
    #[error("master seed hash mismatch — wrong seed for this wallet")]
    MasterSeedMismatch,
    /// Krok 6: Weryfikacja NK derivation z KEM shared_secret.
    /// Zwracany gdy odbiorca wykryje że NK w RecipientBoxPlaintext
    /// nie pasuje do derive_nullifier_key_from_kem(shared_secret, bundle_id).
    /// Wskazuje na próbę manipulacji NK przez nadawcę (attack vector: false NK → wrong nullifier).
    #[error("nullifier key in RecipientBoxPlaintext does not match KEM-derived expected value")]
    InvalidNullifierKeyDerivation,
    #[error("rail context is missing or not initialized")]
    RailContextMissing,
    #[error(transparent)]
    TxShape(#[from] privai_chain::TxShapeError),
    #[error(transparent)]
    ProofBuild(#[from] TransferBuildError),
    #[error(transparent)]
    Decode(#[from] DecodeError),
}
