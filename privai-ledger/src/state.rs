use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use privai_chain::{Block, DEFAULT_CHAIN_ID, Hash32, Nullifier, OutputNote};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteStatus {
    Unspent,
    Spent {
        nullifier: Nullifier,
        spent_in_block: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteRecord {
    pub note: OutputNote,
    pub created_in_block: Option<u64>,
    pub status: NoteStatus,
}

use privai_chain::QuorumCertificate;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConsensusSafetyState {
    pub current_view: u32,
    pub last_voted_view: u32,
    pub current_round: u32,
    pub locked_qc: Option<QuorumCertificate>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerSnapshot {
    pub chain_id: u32,
    pub height: u64,
    pub tip_hash: Hash32,
    pub blocks: BTreeMap<u64, Block>,
    pub notes: BTreeMap<Hash32, NoteRecord>,
    pub spent_nullifiers: BTreeSet<Nullifier>,
    /// Osobny zbiór ticket nullifiers — unika kolizji z note nullifiers.
    pub spent_ticket_nullifiers: BTreeSet<Nullifier>,
    /// Persystowane QCs po height — potrzebne do state sync.
    pub qcs: BTreeMap<u64, QuorumCertificate>,
    pub consensus_safety: ConsensusSafetyState,
}

impl LedgerSnapshot {
    pub fn genesis(chain_id: u32) -> Self {
        Self {
            chain_id,
            height: 0,
            tip_hash: [0; 32],
            blocks: BTreeMap::new(),
            notes: BTreeMap::new(),
            spent_nullifiers: BTreeSet::new(),
            spent_ticket_nullifiers: BTreeSet::new(),
            qcs: BTreeMap::new(),
            consensus_safety: ConsensusSafetyState::default(),
        }
    }

    pub fn is_ticket_nullifier_spent(&self, nullifier: &Nullifier) -> bool {
        self.spent_ticket_nullifiers.contains(nullifier)
    }

    pub fn mark_ticket_nullifier_spent(&mut self, nullifier: Nullifier) {
        self.spent_ticket_nullifiers.insert(nullifier);
    }
}

impl Default for LedgerSnapshot {
    fn default() -> Self {
        Self::genesis(DEFAULT_CHAIN_ID)
    }
}
