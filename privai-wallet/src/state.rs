use std::collections::BTreeMap;

use crate::small_payments_rail::RailContext;
use nxms_transport::crypto::Keys;
use privai_chain::{
    Amount14, BundleId, Hash32, Nullifier, OutputNote, ReceiveBundle, RecipientBoxPlaintext,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BundleStatus {
    Fresh,
    Offered,
    Used,
    Revoked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedBundle {
    pub bundle: ReceiveBundle,
    pub bundle_commit: Hash32,
    pub status: BundleStatus,
    pub local_keys: Option<Keys>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnedNoteStatus {
    Spendable,
    Locked,
    Spent { nullifier: Nullifier },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedNoteRecord {
    pub note: OutputNote,
    pub opened: RecipientBoxPlaintext,
    pub derived_nullifier: Nullifier,
    pub status: OwnedNoteStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpendMaterial {
    pub note: OutputNote,
    pub amount: Amount14,
    pub note_commit: Hash32,
    pub nullifier: Nullifier,
    pub witness_seed: Hash32,
    pub nullifier_key: Hash32,
    pub spend_policy_opening: Vec<u8>,
    pub aux_opening: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BundleMatch {
    NoMatch,
    HintMatched(BundleId),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WalletSnapshot {
    /// Master seed hash — weryfikacja czy seed pasuje do tego portfela.
    /// None = legacy mode (v0, brak master seed).
    pub master_seed_hash: Option<Hash32>,
    /// Kolejny index bundla do derive (v1 — deterministic key derivation).
    pub next_bundle_index: u64,
    pub bundles: BTreeMap<BundleId, ManagedBundle>,
    pub owned_notes: BTreeMap<Hash32, OwnedNoteRecord>,
    pub rail_context: Option<RailContext>,
}
