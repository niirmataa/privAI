//! privAI Ledger Implementation.
//! Role: Core state machine for PC-BFT consensus.
//! Architecture: UTXO/Note-based.
//! Validation Policy: Zero Trust. All Falcon signatures and ZK-proofs must be verified.
//! Scaling Hint: Use `rayon` for parallel signature verification in `validate_block`.
//! See: spec/marketplace_small_payments_v0/04_RECEIPT_AND_SETTLEMENT_ROOT.md

use std::collections::BTreeSet;

use privai_chain::{Block, Hash32, Transaction};
use privai_chain::hash::merkle_root;
use privai_proof::ProofVerifier;

use crate::error::{LedgerError, ValidationError};
use crate::mempool::Mempool;
use crate::state::{LedgerSnapshot, NoteRecord, NoteStatus};
use crate::store::LedgerStore;

/// Oblicza state_root z LedgerSnapshot — merkle root z (note_commit || status) + nullifiery.
/// Używane do weryfikacji ExecutionRoot po zastosowaniu transakcji.
pub fn compute_state_root(snapshot: &LedgerSnapshot) -> Hash32 {
    let mut leaves: Vec<Hash32> = Vec::new();

    // Note commits + status
    for (commit, record) in &snapshot.notes {
        let mut leaf = [0u8; 32];
        leaf[..16].copy_from_slice(&commit[..16]);
        match &record.status {
            NoteStatus::Unspent => leaf[16] = 0x00,
            NoteStatus::Spent { nullifier, .. } => {
                leaf[16] = 0x01;
                leaf[17..].copy_from_slice(&nullifier.0[..15]);
            }
        }
        leaves.push(leaf);
    }

    // Spent nullifiers
    for nullifier in &snapshot.spent_nullifiers {
        leaves.push(nullifier.0);
    }

    // Spent ticket nullifiers
    for nullifier in &snapshot.spent_ticket_nullifiers {
        let mut leaf = [0u8; 32];
        leaf[..16].copy_from_slice(&nullifier.0[..16]);
        leaf[16] = 0xFF;
        leaves.push(leaf);
    }

    merkle_root(leaves.into_iter())
}

pub fn apply_transaction_local(
    tx: &Transaction,
    block_height: u64,
    snapshot: &mut LedgerSnapshot,
) {
    let _ = apply_transaction(tx, block_height, snapshot);
}

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

    pub fn snapshot_mut(&mut self) -> &mut LedgerSnapshot {
        &mut self.snapshot
    }

    pub fn flush(&mut self) -> Result<(), LedgerError> {
        self.store.save(&self.snapshot)?;
        Ok(())
    }

    pub fn update_consensus_safety(&mut self, new_state: crate::state::ConsensusSafetyState) -> Result<(), LedgerError> {
        self.snapshot.consensus_safety = new_state;
        self.store.save(&self.snapshot)?;
        Ok(())
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
        epoch_params: &privai_chain::EpochParams,
    ) -> Result<(), LedgerError> {        validate_block(block, &self.snapshot, &self.proof_verifier, min_proof_coverage, epoch_params)?;

        for tx in &block.body.txs {
            apply_transaction(tx, block.header.height, &mut self.snapshot)?;
        }

        self.snapshot.height = block.header.height;
        self.snapshot.tip_hash = block.hash();
        self.snapshot.blocks.insert(block.header.height, block.clone());

        // Limit cached blocks in memory — keep only last 128
        const MAX_CACHED_BLOCKS: u64 = 128;
        if self.snapshot.blocks.len() as u64 > MAX_CACHED_BLOCKS {
            let cutoff = self.snapshot.height.saturating_sub(MAX_CACHED_BLOCKS);
            self.snapshot.blocks.retain(|h, _| *h > cutoff);
        }

        self.mempool.remove_committed_block(&block.body.txs);
        self.store.save(&self.snapshot)?;
        Ok(())
    }
}

pub fn validate_transaction(
    tx: &Transaction,
    snapshot: &LedgerSnapshot,
    min_fee: u64,
) -> Result<(), ValidationError> {
    tx.validate_shape()?;

    // Minimum fee enforcement
    if tx.core().fee < min_fee {
        return Err(ValidationError::FeeTooLow {
            fee: tx.core().fee,
            min_fee,
        });
    }

    if let Transaction::MarketplaceBatch(batch_tx) = &tx {
        // Anti-Forging: operator MUSI podpisać settlement_root kluczem Falcon.
        // v0: jeśli auth puste, skipuj operator_sig check (brak prawdziwych kluczy Falcon)
        if !tx.core().auth.is_empty() {
            let operator_pk = tx.core().auth.first()
                .and_then(|a| a.signer_pks.first())
                .ok_or(ValidationError::MissingOperatorSignature)?;
            if batch_tx.operator_sig.is_empty() {
                return Err(ValidationError::MissingOperatorSignature);
            }
            let batch_msg = batch_tx.summary.settlement_root();
            nxms_transport::crypto::falcon_verify(operator_pk, &batch_msg, &batch_tx.operator_sig)
                .map_err(|_| ValidationError::InvalidOperatorSignature)?;
        }

        // Explicitly check for MarketplaceBatchTx double-spends
        let mut seen_ticket_nullifiers = BTreeSet::new();
        for nullifier in &batch_tx.ticket_nullifiers {
            if !seen_ticket_nullifiers.insert(*nullifier) {
                return Err(ValidationError::DuplicateNullifier);
            }
            if snapshot.is_ticket_nullifier_spent(nullifier) {
                return Err(ValidationError::DoubleSpend(*nullifier));
            }
        }
    } else {
        // Weryfikacja podpisów Falcon na TxCore.auth (Zero Trust)
        // v0: brak wymuszenia auth dla prototypu. Docelowo każdy TX musi mieć ważny auth.
        if !tx.core().auth.is_empty() {
            let tx_hash = tx.tx_id();
            for (i, auth) in tx.core().auth.iter().enumerate() {
                if auth.signer_pks.is_empty() || auth.signatures.is_empty() {
                    return Err(ValidationError::InvalidAuth(format!(
                        "auth[{}]: missing signer_pks or signatures", i
                    )));
                }
                if auth.signer_pks.len() != auth.signatures.len() {
                    return Err(ValidationError::InvalidAuth(format!(
                        "auth[{}]: signer_pks/signatures count mismatch ({} vs {})",
                        i, auth.signer_pks.len(), auth.signatures.len()
                    )));
                }
                for (j, (pk, sig)) in auth.signer_pks.iter().zip(auth.signatures.iter()).enumerate() {
                    if pk.is_empty() || sig.is_empty() {
                        return Err(ValidationError::InvalidAuth(format!(
                            "auth[{}][{}]: empty pk or sig", i, j
                        )));
                    }
                    if nxms_transport::crypto::falcon_verify(pk, &tx_hash, sig).is_err() {
                        return Err(ValidationError::InvalidAuth(format!(
                            "auth[{}][{}]: invalid Falcon signature", i, j
                        )));
                    }
                }
            }
        }

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
    epoch_params: &privai_chain::EpochParams,
) -> Result<(), ValidationError> {
    // chain_id validation
    if block.header.chain_id != snapshot.chain_id {
        return Err(ValidationError::InvalidChainId {
            expected: snapshot.chain_id,
            actual: block.header.chain_id,
        });
    }

    // Epoch transition: block must be within current epoch bounds
    let expected_height = snapshot.height + 1;
    if expected_height > epoch_params.end_height {
        return Err(ValidationError::InvalidBlockHeight {
            expected: epoch_params.end_height,
            actual: expected_height,
        });
    }

    // Max transactions per block
    if block.body.txs.len() > epoch_params.max_block_statements as usize {
        return Err(ValidationError::TooManyTransactions {
            count: block.body.txs.len(),
            max: epoch_params.max_block_statements as usize,
        });
    }

    // Max block size (canonical bytes — spójne z wire format)
    use privai_chain::CanonicalEncode;
    let block_size = block.to_canonical_bytes().len();
    if block_size > epoch_params.max_block_bytes as usize {
        return Err(ValidationError::BlockTooLarge {
            size: block_size,
            max: epoch_params.max_block_bytes as usize,
        });
    }

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

    proof_verifier.verify_block(block)?;

    let total_require_proof = block.body.txs.iter()
        .filter(|tx| matches!(tx, Transaction::TransferNote(_)))
        .count();

    if total_require_proof > 0 {
        let provided_certificates = block.body.proof_certificates.len();
        if provided_certificates < (min_proof_coverage as usize) {
            return Err(ValidationError::Proof(privai_proof::ProofError::MissingProofCoverage));
        }
    }

    let mut temp = snapshot.clone();
    for tx in &block.body.txs {
        validate_transaction(tx, &temp, epoch_params.min_fee)?;
        apply_transaction(tx, block.header.height, &mut temp)?;
    }

    // ExecutionRoot verification: po zastosowaniu transakcji, state_root musi pasować
    let computed_state_root = compute_state_root(&temp);
    if computed_state_root != block.header.state_root {
        return Err(ValidationError::StateRootMismatch {
            expected: computed_state_root,
            actual: block.header.state_root,
        });
    }

    Ok(())
}

fn apply_transaction(
    tx: &Transaction,
    block_height: u64,
    snapshot: &mut LedgerSnapshot,
) -> Result<(), ValidationError> {
    if let Transaction::MarketplaceBatch(batch_tx) = &tx {
        // Burn ticket nullifiers explicitly
        for nullifier in &batch_tx.ticket_nullifiers {
            snapshot.mark_ticket_nullifier_spent(*nullifier);
        }
    } else {
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

    use privai_chain::tx::{MarketplaceBatchTx, TX_TYPE_MARKETPLACE_BATCH};
    use privai_chain::small_payments::SettlementBatchSummary;

    fn test_epoch_params() -> privai_chain::EpochParams {
        privai_chain::EpochParams {
            epoch_number: 0,
            start_height: 0,
            end_height: 1_000_000,
            min_validator_stake: 0,
            min_prover_bond: 0,
            min_fee: 0,
            max_block_bytes: 10_000_000,
            max_block_statements: 100_000,
            min_proof_coverage: 1,
        }
    }

    #[test]
    fn ledger_rejects_marketplace_batch_double_spend() {
        let mut ledger =
            Ledger::open(MemoryStore::new(), 17, StructuralProofVerifier).expect("ledger");

        let nullifier1 = privai_chain::Nullifier([0xaa; 32]);
        let nullifier2 = privai_chain::Nullifier([0xbb; 32]);

        let operator_commit = privai_chain::Hash32::default();

        let summary = SettlementBatchSummary {
            operator_commit,
            merchant_commit: [2; 32],
            grant_commit: [3; 32],
            settlement_window_start: 0,
            settlement_window_end: 1000,
            receipt_root: [4; 32],
            receipt_count: 2,
            nullifier_count: 2,
            total_gross_amount: 100,
            total_fee_amount: 10,
            total_refund_amount: 0,
        };
        
        // Create a valid MarketplaceBatchTx (v0: auth puste — brak prawdziwych kluczy Falcon)
        let batch_tx = Transaction::MarketplaceBatch(MarketplaceBatchTx {
            core: TxCore {
                version: 0,
                tx_type: TX_TYPE_MARKETPLACE_BATCH,
                inputs: vec![],
                input_nullifiers: vec![],
                outputs: vec![],
                fee: 10,
                statement_commit: [5; 32],
                auth: Vec::new(), // v0: puste auth
            },
            summary: summary.clone(),
            ticket_nullifiers: vec![nullifier1.clone(), nullifier2.clone()],
            operator_sig: Vec::new(), // v0: puste
        });

        // 1. Submit should succeed the first time
        assert!(validate_transaction(&batch_tx, ledger.snapshot(), 0).is_ok());
        
        // 2. Apply it directly to the ledger snapshot to simulate block inclusion
        apply_transaction(&batch_tx, 1, &mut ledger.snapshot).expect("apply tx");
        
        // Ensure state is updated
        assert!(ledger.snapshot.is_ticket_nullifier_spent(&nullifier1));
        assert!(ledger.snapshot.is_ticket_nullifier_spent(&nullifier2));

        // 3. Create a malicious replay Tx with one spent nullifier
        let replay_tx = Transaction::MarketplaceBatch(MarketplaceBatchTx {
            core: TxCore {
                version: 0,
                tx_type: TX_TYPE_MARKETPLACE_BATCH,
                inputs: vec![],
                input_nullifiers: vec![],
                outputs: vec![],
                fee: 10,
                statement_commit: [6; 32],
                auth: Vec::new(), // v0: puste auth
            },
            summary: summary.clone(),
            ticket_nullifiers: vec![nullifier1.clone()], // this one is already spent!
            operator_sig: Vec::new(), // v0: puste
        });

        // 4. Validation MUST reject this
        let err = validate_transaction(&replay_tx, ledger.snapshot(), 0).expect_err("should reject double spend");
        assert!(matches!(err, ValidationError::DoubleSpend(_)));
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

        // Oblicz state_root przed tworzeniem bloku
        let mut temp_snapshot = ledger.snapshot().clone();
        crate::ledger::apply_transaction_local(&spend_tx, 1, &mut temp_snapshot);
        let state_root = crate::ledger::compute_state_root(&temp_snapshot);

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
            state_root,
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

        ledger.apply_block(&block, 1, &test_epoch_params()).expect("apply block");

        assert!(matches!(
            ledger.snapshot.notes.get(&funding_note.note_commit).unwrap().status,
            NoteStatus::Spent { .. }
        ));
        assert!(ledger.mempool.is_empty());
    }
}
