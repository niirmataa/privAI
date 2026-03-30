use std::collections::BTreeSet;

use privai_chain::{Block, Hash32, Transaction};
use privai_proof::ProofVerifier;

use crate::error::{LedgerError, ValidationError};
use crate::mempool::Mempool;
use crate::state::{LedgerSnapshot, NoteRecord, NoteStatus};
use crate::store::LedgerStore;

pub struct Ledger<S: LedgerStore, V: ProofVerifier> {
    store: S,
    snapshot: LedgerSnapshot,
    mempool: Mempool,
    proof_verifier: V,
}

impl<S: LedgerStore, V: ProofVerifier> Ledger<S, V> {
    pub fn open(mut store: S, chain_id: u32, proof_verifier: V) -> Result<Self, LedgerError> {
        let snapshot = store.load()?.unwrap_or_else(|| LedgerSnapshot::genesis(chain_id));
        store.save(&snapshot)?;
        Ok(Self {
            store,
            snapshot,
            mempool: Mempool::new(),
            proof_verifier,
        })
    }

    pub fn snapshot(&self) -> &LedgerSnapshot {
        &self.snapshot
    }

    pub fn mempool(&self) -> &Mempool {
        &self.mempool
    }

    pub fn submit_transaction(
        &mut self,
        tx: Transaction,
        received_at_ms: u64,
    ) -> Result<Hash32, LedgerError> {
        Ok(self.mempool.insert(tx, received_at_ms, &self.snapshot)?)
    }

    pub fn candidate_transactions(&self, max_count: usize) -> Vec<Transaction> {
        self.mempool.best_transactions(max_count)
    }

    pub fn apply_block(
        &mut self,
        block: &Block,
        min_proof_coverage: u32,
    ) -> Result<(), LedgerError> {
        validate_block(block, &self.snapshot, &self.proof_verifier, min_proof_coverage)?;

        for tx in &block.body.txs {
            apply_transaction(tx, block.header.height, &mut self.snapshot)?;
        }

        self.snapshot.height = block.header.height;
        self.snapshot.tip_hash = block.hash();
        self.snapshot.blocks.insert(block.header.height, block.clone());
        self.mempool.remove_committed_block(&block.body.txs);
        self.store.save(&self.snapshot)?;
        Ok(())
    }
}

pub fn validate_transaction(
    tx: &Transaction,
    snapshot: &LedgerSnapshot,
) -> Result<(), ValidationError> {
    tx.validate_shape()?;

    let mut seen_inputs = BTreeSet::new();
    for input in tx.inputs() {
        if !seen_inputs.insert(input.note_commit) {
            return Err(ValidationError::DuplicateInput);
        }

        let Some(record) = snapshot.notes.get(&input.note_commit) else {
            return Err(ValidationError::MissingInput(input.note_commit));
        };

        if !matches!(record.status, NoteStatus::Unspent) {
            return Err(ValidationError::InputAlreadySpent(input.note_commit));
        }
    }

    let mut seen_nullifiers = BTreeSet::new();
    for nullifier in tx.input_nullifiers() {
        if !seen_nullifiers.insert(*nullifier) {
            return Err(ValidationError::DuplicateNullifier);
        }

        if snapshot.spent_nullifiers.contains(nullifier) {
            return Err(ValidationError::NullifierAlreadySpent(*nullifier));
        }
    }

    for output in tx.outputs() {
        if snapshot.notes.contains_key(&output.note_commit) {
            return Err(ValidationError::DuplicateOutput(output.note_commit));
        }
    }

    Ok(())
}

pub fn validate_block<V: ProofVerifier>(
    block: &Block,
    snapshot: &LedgerSnapshot,
    proof_verifier: &V,
    min_proof_coverage: u32,
) -> Result<(), ValidationError> {
    let expected_height = snapshot.height + 1;
    if block.header.height != expected_height {
        return Err(ValidationError::InvalidBlockHeight {
            expected: expected_height,
            actual: block.header.height,
        });
    }

    if block.header.prev_block_hash != snapshot.tip_hash {
        return Err(ValidationError::InvalidParent);
    }

    if !block.roots_match() {
        return Err(ValidationError::InvalidRoots);
    }

    proof_verifier.verify_block(block, min_proof_coverage)?;

    let mut temp = snapshot.clone();
    for tx in &block.body.txs {
        validate_transaction(tx, &temp)?;
        apply_transaction(tx, block.header.height, &mut temp)?;
    }

    Ok(())
}

fn apply_transaction(
    tx: &Transaction,
    block_height: u64,
    snapshot: &mut LedgerSnapshot,
) -> Result<(), ValidationError> {
    for (input, nullifier) in tx.inputs().iter().zip(tx.input_nullifiers().iter()) {
        let Some(record) = snapshot.notes.get_mut(&input.note_commit) else {
            return Err(ValidationError::MissingInput(input.note_commit));
        };

        if !matches!(record.status, NoteStatus::Unspent) {
            return Err(ValidationError::InputAlreadySpent(input.note_commit));
        }

        record.status = NoteStatus::Spent {
            nullifier: *nullifier,
            spent_in_block: block_height,
        };
        snapshot.spent_nullifiers.insert(*nullifier);
    }

    for output in tx.outputs() {
        snapshot.notes.insert(
            output.note_commit,
            NoteRecord {
                note: output.clone(),
                created_in_block: Some(block_height),
                status: NoteStatus::Unspent,
            },
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use privai_chain::{
        merkle_root, Block, BlockTemplate, ExecutionBundle, ExecutionMode, InputRef, OutputNote,
        ProofCertificate, RecipientBox, Transaction, TransferNoteTx, TxCore,
        TX_TYPE_TRANSFER_NOTE,
    };
    use privai_proof::StructuralProofVerifier;

    use super::*;
    use crate::store::MemoryStore;

    fn sample_note(seed: u8) -> OutputNote {
        OutputNote::new(
            [seed; 32],
            privai_chain::LweCiphertext::default(),
            [seed.wrapping_add(1); 32],
            RecipientBox::new(vec![seed], [seed; 24], vec![seed + 1], [seed; 16], [seed; 16]),
        )
    }

    #[test]
    fn mempool_and_block_flow_spends_input() {
        let mut ledger =
            Ledger::open(MemoryStore::new(), 17, StructuralProofVerifier).expect("ledger");
        let funding_note = sample_note(9);
        ledger.snapshot.notes.insert(
            funding_note.note_commit,
            NoteRecord {
                note: funding_note.clone(),
                created_in_block: Some(0),
                status: NoteStatus::Unspent,
            },
        );

        let spend_tx = Transaction::TransferNote(TransferNoteTx {
            core: TxCore {
                version: 0,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: vec![InputRef {
                    note_commit: funding_note.note_commit,
                }],
                input_nullifiers: vec![privai_chain::Nullifier([7; 32])],
                outputs: vec![sample_note(10)],
                fee: 3,
                statement_commit: [11; 32],
                auth: Vec::new(),
            },
        });

        ledger
            .submit_transaction(spend_tx.clone(), 1_000)
            .expect("submit");
        let execution_bundle = ExecutionBundle {
            statement_commits: vec![spend_tx.statement_commit()],
            covered_tx_indexes: vec![0],
            public_inputs_root: [5; 32],
            execution_mode: ExecutionMode::FullBatchProof,
        };
        let statement_root = merkle_root(execution_bundle.statement_commits.iter().copied());

        let block = Block::from_template(BlockTemplate {
            chain_id: 17,
            height: 1,
            epoch: 0,
            round: 0,
            timestamp_ms: 1_000,
            prev_block_hash: [0; 32],
            proposer_pk_hash: [1; 32],
            epoch_seed_hash: [2; 32],
            parent_qc_hash: [3; 32],
            txs: vec![spend_tx.clone()],
            execution_bundle: execution_bundle.clone(),
            proof_certificates: vec![ProofCertificate {
                proof_system_id: 1,
                statement_root,
                public_inputs_root: execution_bundle.public_inputs_root,
                proof_bytes_hash: [6; 32],
                prover_ids: vec![[8; 32]],
                proof_meta_hash: [9; 32],
            }],
            extra_receipts: Vec::new(),
        });

        ledger.apply_block(&block, 1).expect("apply block");

        assert!(matches!(
            ledger.snapshot.notes.get(&funding_note.note_commit).unwrap().status,
            NoteStatus::Spent { .. }
        ));
        assert!(ledger.mempool.is_empty());
    }
}
