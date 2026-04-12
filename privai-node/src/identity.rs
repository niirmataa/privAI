//! Phase 0-5 Identity Model
//! 
//! Simple 4 / Core Draft explicitly rejects `HiddenRootCredential` for now.
//! Instead, the protocol relies on two strictly separated Falcon PKs.
//! This module provides strong typing to prevent mixing them.

use serde::{Deserialize, Serialize};
use privai_chain::primitives::Hash32;

/// Key used by the Node to participate in consensus and sign blocks.
/// Driven directly by the frozen `node_pk_hash` on-chain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorRoleKey {
    pub falcon_pk_hash: Hash32,
}

/// Key used by the Compute Miner to sign `ComputeOffering`, `StartupManifest`,
/// `WindowTelemetryRecord`, and `ComputeLeaseReceipt`.
/// Completely independent of the `ValidatorRoleKey`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeMinerRoleKey {
    pub falcon_pk_hash: Hash32,
}

impl ComputeMinerRoleKey {
    pub fn new(hash: Hash32) -> Self {
        Self { falcon_pk_hash: hash }
    }
}
