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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerSnapshot {
    pub chain_id: u32,
    pub height: u64,
    pub tip_hash: Hash32,
    pub blocks: BTreeMap<u64, Block>,
    pub notes: BTreeMap<Hash32, NoteRecord>,
    pub spent_nullifiers: BTreeSet<Nullifier>,
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
        }
    }

    pub fn is_ticket_nullifier_spent(&self, nullifier: &Nullifier) -> bool {
        self.spent_nullifiers.contains(nullifier)
    }

    pub fn mark_ticket_nullifier_spent(&mut self, nullifier: Nullifier) {
        self.spent_nullifiers.insert(nullifier);
    }
}

impl Default for LedgerSnapshot {
    fn default() -> Self {
        Self::genesis(DEFAULT_CHAIN_ID)
    }
}
