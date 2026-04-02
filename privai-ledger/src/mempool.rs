use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use privai_chain::{Hash32, Nullifier, Transaction};

use crate::error::MempoolError;
use crate::ledger::validate_transaction;
use crate::state::LedgerSnapshot;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingTx {
    pub tx_id: Hash32,
    pub tx: Transaction,
    pub received_at_ms: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Mempool {
    entries: BTreeMap<Hash32, PendingTx>,
    reserved_inputs: BTreeMap<Hash32, Hash32>,
    reserved_nullifiers: BTreeMap<Nullifier, Hash32>,
}

impl Mempool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        tx: Transaction,
        received_at_ms: u64,
        snapshot: &LedgerSnapshot,
    ) -> Result<Hash32, MempoolError> {
        validate_transaction(&tx, snapshot, 0)?;
        let tx_id = tx.tx_id();

        for input in tx.inputs() {
            if let Some(conflict) = self.reserved_inputs.get(&input.note_commit) {
                return Err(MempoolError::InputConflict {
                    input: input.note_commit,
                    conflict: *conflict,
                });
            }
        }

        for nullifier in tx.input_nullifiers() {
            if let Some(conflict) = self.reserved_nullifiers.get(nullifier) {
                return Err(MempoolError::NullifierConflict {
                    nullifier: *nullifier,
                    conflict: *conflict,
                });
            }
        }

        for input in tx.inputs() {
            self.reserved_inputs.insert(input.note_commit, tx_id);
        }

        for nullifier in tx.input_nullifiers() {
            self.reserved_nullifiers.insert(*nullifier, tx_id);
        }

        self.entries.insert(
            tx_id,
            PendingTx {
                tx_id,
                tx,
                received_at_ms,
            },
        );
        Ok(tx_id)
    }

    pub fn best_transactions(&self, max_count: usize) -> Vec<Transaction> {
        let mut entries: Vec<&PendingTx> = self.entries.values().collect();
        entries.sort_by(|left, right| {
            right
                .tx
                .fee()
                .cmp(&left.tx.fee())
                .then_with(|| left.received_at_ms.cmp(&right.received_at_ms))
        });

        entries
            .into_iter()
            .take(max_count)
            .map(|entry| entry.tx.clone())
            .collect()
    }

    pub fn remove_committed_block(&mut self, block_txs: &[Transaction]) {
        let committed: BTreeSet<Hash32> = block_txs.iter().map(Transaction::tx_id).collect();
        self.entries.retain(|tx_id, _| !committed.contains(tx_id));
        self.reserved_inputs
            .retain(|_, tx_id| !committed.contains(tx_id));
        self.reserved_nullifiers
            .retain(|_, tx_id| !committed.contains(tx_id));
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
