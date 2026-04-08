//! privAI Ledger Implementation.
//! Role: Core state machine for PC-BFT consensus.
//! Architecture: UTXO/Note-based.
//! Validation Policy: Zero Trust. All Falcon signatures and ZK-proofs must be verified.
//! Scaling Hint: Use `rayon` for parallel signature verification in `validate_block`.
//! See: spec/marketplace_small_payments_v0/04_RECEIPT_AND_SETTLEMENT_ROOT.md

use std::collections::BTreeSet;

use privai_chain::hash::merkle_root;
use privai_chain::{Block, Hash32, Transaction};
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

pub fn apply_transaction_local(tx: &Transaction, block_height: u64, snapshot: &mut LedgerSnapshot) {
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
        let snapshot = store
            .load()?
            .unwrap_or_else(|| LedgerSnapshot::genesis(chain_id));
            
        // Weryfikacja spójności (boot-time state root verification)
        if snapshot.height > 0 {
            if let Some(last_block) = snapshot.blocks.get(&snapshot.height) {
                let computed = compute_state_root(&snapshot);
                if computed != last_block.header.state_root {
                    return Err(LedgerError::Validation(ValidationError::StateRootMismatch {
                        expected: last_block.header.state_root,
                        actual: computed,
                    }));
                }
            }
        }
            
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

    pub fn update_consensus_safety(
        &mut self,
        new_state: crate::state::ConsensusSafetyState,
    ) -> Result<(), LedgerError> {
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
    ) -> Result<(), LedgerError> {
        validate_block(
            block,
            &self.snapshot,
            &self.proof_verifier,
            min_proof_coverage,
            epoch_params,
        )?;

        for tx in &block.body.txs {
            apply_transaction(tx, block.header.height, &mut self.snapshot)?;
        }

        self.snapshot.height = block.header.height;
        self.snapshot.tip_hash = block.hash();
        self.snapshot
            .blocks
            .insert(block.header.height, block.clone());

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
    block_height: u64,
) -> Result<(), ValidationError> {
    tx.validate_shape()?;

    // Minimum fee enforcement (using tx.fee() to handle all variants)
    if tx.fee() < min_fee {
        return Err(ValidationError::FeeTooLow {
            fee: tx.fee(),
            min_fee,
        });
    }

    if let Transaction::MarketplaceBatch(batch_tx) = &tx {
        // Anti-Forging: operator MUSI podpisać settlement_root kluczem Falcon.
        // v0: jeśli auth puste, skipuj operator_sig check (brak prawdziwych kluczy Falcon)
        if !tx.core().auth.is_empty() {
            let operator_pk = tx
                .core()
                .auth
                .first()
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
        // ── Input existence check (moved before auth to support escrow policy lookup) ──
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

        let is_full_privacy = matches!(tx, Transaction::TransferNote(_));

        if is_full_privacy {
            // FullPrivacy Option B: mandatory auth for all inputs
            if tx.auth().len() != tx.inputs().len() {
                return Err(ValidationError::AuthCountMismatch);
            }
        } else {
            // Legacy / Prototype checks for non-TransferNote transactions
            let has_escrow_input = tx.auth().iter().any(|a| {
                a.policy_tag == privai_chain::SpendPolicyTag::Escrow2of3 as u8
            });
            if has_escrow_input && tx.auth().len() < tx.inputs().len() {
                return Err(ValidationError::MissingAuth);
            }
        }

        // ── Auth verification ──────────────────────────────────────────────
        // Signers sign `tx_signing_hash` (NOT `tx_id`) to break circular dependency.
        let tx_hash = tx.tx_signing_hash();

        for (i, input) in tx.inputs().iter().enumerate() {
            let record = snapshot.notes.get(&input.note_commit)
                .expect("input existence already validated above");
            let input_policy_commit = record.note.spend_policy_commit;

            let auth_entry = tx.auth().get(i);

            if let Some(auth) = auth_entry {
                if is_full_privacy {
                    // Option B FullPrivacy Validation
                    // 1. Mandatory policy_opening
                    let policy_opening_bytes = auth.policy_opening.as_ref()
                        .ok_or(ValidationError::MissingPolicyOpening)?;
                    
                    // 2. Decode policy
                    use privai_chain::decode::CanonicalDecode;
                    let policy = privai_chain::note::SpendPolicy::from_canonical_bytes(policy_opening_bytes)
                        .map_err(|e| ValidationError::PolicyDecode(e.to_string()))?;

                    // 3. Verify spend_policy_commit
                    let computed_commit = policy.commitment();
                    if computed_commit != input_policy_commit {
                        return Err(ValidationError::PolicyMismatch);
                    }

                    // 4. Check policy_tag matches derived tag
                    let derived_tag = policy.tag();
                    if auth.policy_tag != derived_tag as u8 {
                        return Err(ValidationError::PolicyTagMismatch);
                    }

                    // 5. Dispatch validation based on derived policy
                    match policy {
                        privai_chain::note::SpendPolicy::Single { falcon_pk_hash } => {
                            if auth.escrow_action.is_some() {
                                return Err(ValidationError::InvalidAuth("Single policy should not have escrow_action".into()));
                            }
                            if auth.signer_pks.len() != 1 || auth.signatures.len() != 1 {
                                return Err(ValidationError::InvalidSingleSignerCount(auth.signer_pks.len()));
                            }
                            let pk = &auth.signer_pks[0];
                            let sig = &auth.signatures[0];
                            
                            if pk.is_empty() || sig.is_empty() {
                                return Err(ValidationError::InvalidAuth(format!(
                                    "auth[{}]: empty pk or sig", i
                                )));
                            }
                            
                            // Check signer pk_hash
                            use privai_chain::hash::falcon_pk_hash as hash_pk;
                            if hash_pk(pk) != falcon_pk_hash {
                                return Err(ValidationError::InvalidAuth(format!(
                                    "auth[{}]: signer pk hash mismatch", i
                                )));
                            }
                            
                            if nxms_transport::crypto::falcon_verify(pk, &tx_hash, sig).is_err() {
                                return Err(ValidationError::InvalidAuth(format!(
                                    "auth[{}]: invalid Falcon signature", i
                                )));
                            }
                        }
                        privai_chain::note::SpendPolicy::Escrow2of3 { .. } => {
                            crate::escrow::validate_escrow_auth(
                                i,
                                auth,
                                &input_policy_commit,
                                tx.outputs(),
                                &tx_hash,
                                block_height,
                            )?;
                        }
                        privai_chain::note::SpendPolicy::MarketplaceSettlement { .. } => {
                            // MarketplaceSettlement isn't part of the escrow/FullPrivacy model
                            return Err(ValidationError::InvalidAuth("MarketplaceSettlement unsupported in FullPrivacy".into()));
                        }
                    }
                } else {
                    // Legacy auth verification for non-FullPrivacy
                    if auth.policy_tag == privai_chain::SpendPolicyTag::Escrow2of3 as u8 {
                        crate::escrow::validate_escrow_auth(
                            i,
                            auth,
                            &input_policy_commit,
                            tx.outputs(),
                            &tx_hash,
                            block_height,
                        )?;
                    } else {
                        if auth.signer_pks.is_empty() || auth.signatures.is_empty() {
                            return Err(ValidationError::InvalidAuth(format!(
                                "auth[{}]: missing signer_pks or signatures",
                                i
                            )));
                        }
                        if auth.signer_pks.len() != auth.signatures.len() {
                            return Err(ValidationError::InvalidAuth(format!(
                                "auth[{}]: signer_pks/signatures count mismatch",
                                i
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
            } else if is_full_privacy {
                // P0: Input has no auth entry but it's FullPrivacy
                return Err(ValidationError::AuthCountMismatch);
            }
            // else: non-escrow tx with no auth → v0 prototype mode (allowed)
        }

        // ── Nullifier checks ──────────────────────────────────────────────
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

    let total_require_proof = block
        .body
        .txs
        .iter()
        .filter(|tx| matches!(tx, Transaction::TransferNote(_)))
        .count();

    if total_require_proof > 0 {
        let provided_certificates = block.body.proof_certificates.len();
        if provided_certificates < (min_proof_coverage as usize) {
            return Err(ValidationError::Proof(
                privai_proof::ProofError::MissingProofCoverage,
            ));
        }
    }

    let mut temp = snapshot.clone();
    for tx in &block.body.txs {
        validate_transaction(tx, &temp, epoch_params.min_fee, block.header.height)?;
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
#[allow(unused_imports, dead_code)]
mod tests {
    use privai_chain::{
        merkle_root, Block, BlockTemplate, ExecutionBundle, ExecutionMode, InputRef, OutputNote,
        ProofCertificate, RecipientBox, Transaction, TransferNoteTx, TxCore, DEFAULT_CHAIN_ID,
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
            RecipientBox::new(
                vec![seed],
                [seed; 24],
                vec![seed + 1],
                [seed; 16],
                [seed; 16],
            ),
        )
    }

    use privai_chain::small_payments::SettlementBatchSummary;
    use privai_chain::tx::{MarketplaceBatchTx, TX_TYPE_MARKETPLACE_BATCH};

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
        assert!(validate_transaction(&batch_tx, ledger.snapshot(), 0, 1).is_ok());

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
            operator_sig: Vec::new(),                    // v0: puste
        });

        // 4. Validation MUST reject this
        let err = validate_transaction(&replay_tx, ledger.snapshot(), 0, 2)
            .expect_err("should reject double spend");
        assert!(matches!(err, ValidationError::DoubleSpend(_)));
    }

    // ── P0: escrow input without auth must be rejected ───────────────

    #[test]
    fn p0_escrow_bearing_tx_must_not_pass_with_partial_auth() {
        // P0 property: A tx that contains ANY escrow auth tag CANNOT pass
        // if some inputs lack auth entries. The tx must be rejected.
        //
        // With dummy Falcon keys, auth[0]'s escrow validation fails at
        // policy decode or sig verification before we reach input[1]'s
        // MissingAuth check. Either way the tx is REJECTED — which is
        // the P0 invariant: it cannot silently pass.
        use privai_chain::{InputAuth, Nullifier, SpendPolicyTag};

        let mut snapshot = LedgerSnapshot::genesis(17);

        let note1 = sample_note(0x10);
        let note2 = sample_note(0x20);
        snapshot.notes.insert(
            note1.note_commit,
            NoteRecord {
                note: note1.clone(),
                created_in_block: Some(0),
                status: NoteStatus::Unspent,
            },
        );
        snapshot.notes.insert(
            note2.note_commit,
            NoteRecord {
                note: note2.clone(),
                created_in_block: Some(0),
                status: NoteStatus::Unspent,
            },
        );

        // 2 inputs, 1 auth (escrow) — auth[1] is missing
        let tx = Transaction::TransferNote(TransferNoteTx {
            core: TxCore {
                version: 1,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: vec![
                    InputRef { note_commit: note1.note_commit },
                    InputRef { note_commit: note2.note_commit },
                ],
                input_nullifiers: vec![
                    Nullifier([0xE1; 32]),
                    Nullifier([0xE2; 32]),
                ],
                outputs: vec![sample_note(0x30)],
                fee: 0,
                statement_commit: [0xF0; 32],
                auth: vec![
                    InputAuth {
                        policy_tag: SpendPolicyTag::Escrow2of3 as u8,
                        signer_pks: vec![vec![0xAA; 64], vec![0xBB; 64]],
                        signatures: vec![vec![0xCC; 32], vec![0xDD; 32]],
                        policy_opening: Some(vec![0x01]),
                        escrow_action: Some(0x01),
                    },
                ],
            },
        });

        let result = validate_transaction(&tx, &snapshot, 0, 1);
        assert!(
            result.is_err(),
            "P0: escrow-bearing tx with partial auth MUST be rejected"
        );
        if let Err(ValidationError::AuthCountMismatch) = result {
            // expected in Option B FullPrivacy
        } else {
            panic!("Expected AuthCountMismatch, got {:?}", result);
        }
    }

    #[test]
    fn p0_has_escrow_input_detection_works() {
        // Unit test for the has_escrow_input detection logic:
        // auth tags drive escrow detection (not the opaque spend_policy_commit).
        use privai_chain::{InputAuth, SpendPolicyTag};

        let single_auth = InputAuth {
            policy_tag: SpendPolicyTag::Single as u8,
            signer_pks: vec![],
            signatures: vec![],
            policy_opening: None,
            escrow_action: None,
        };
        let escrow_auth = InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![],
            signatures: vec![],
            policy_opening: None,
            escrow_action: None,
        };

        // No auth → no escrow
        let empty_auth: Vec<InputAuth> = vec![];
        assert!(!empty_auth.iter().any(|a| a.policy_tag == SpendPolicyTag::Escrow2of3 as u8));

        // Only Single → no escrow
        let single_only = vec![single_auth.clone()];
        assert!(!single_only.iter().any(|a| a.policy_tag == SpendPolicyTag::Escrow2of3 as u8));

        // Has Escrow2of3 → escrow detected
        let with_escrow = vec![single_auth.clone(), escrow_auth.clone()];
        assert!(with_escrow.iter().any(|a| a.policy_tag == SpendPolicyTag::Escrow2of3 as u8));

        // Only Escrow2of3 → escrow detected
        let escrow_only = vec![escrow_auth];
        assert!(escrow_only.iter().any(|a| a.policy_tag == SpendPolicyTag::Escrow2of3 as u8));
    }

    #[test]
    fn p0_non_escrow_tx_without_auth_allowed_in_prototype() {
        // FullPrivacy Option B: non-escrow TransferNote tx with empty auth must be REJECTED.
        let mut snapshot = LedgerSnapshot::genesis(17);

        let note1 = sample_note(0x10);
        snapshot.notes.insert(
            note1.note_commit,
            NoteRecord {
                note: note1.clone(),
                created_in_block: Some(0),
                status: NoteStatus::Unspent,
            },
        );

        let tx = Transaction::TransferNote(TransferNoteTx {
            core: TxCore {
                version: 1,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: vec![InputRef { note_commit: note1.note_commit }],
                input_nullifiers: vec![privai_chain::Nullifier([0xE1; 32])],
                outputs: vec![sample_note(0x30)],
                fee: 0,
                statement_commit: [0xF0; 32],
                auth: vec![], // empty auth -> must reject for TransferNoteTx
            },
        });

        let result = validate_transaction(&tx, &snapshot, 0, 1);
        assert!(
            result.is_err(),
            "TransferNoteTx without auth must be rejected in FullPrivacy Option B"
        );
        if let Err(ValidationError::AuthCountMismatch) = result {
            // expected
        } else {
            panic!("Expected AuthCountMismatch, got {:?}", result);
        }
    }

    // ── FullPrivacy Option B Tests ───────────────────────────────────

    fn setup_single_auth_test() -> (LedgerSnapshot, Transaction, privai_chain::InputAuth, Vec<u8>) {
        use privai_chain::{InputAuth, Nullifier, SpendPolicyTag, note::SpendPolicy};
        use privai_chain::CanonicalEncode;

        let mut snapshot = LedgerSnapshot::genesis(17);
        let (sk, pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");
        let pk_hash = privai_chain::hash::falcon_pk_hash(&pk);
        
        let policy = SpendPolicy::Single { falcon_pk_hash: pk_hash };
        let policy_commit = policy.commitment();
        let policy_bytes = policy.to_canonical_bytes();

        let note = OutputNote::new(
            policy_commit,
            privai_chain::LweCiphertext::default(),
            [0x10; 32],
            RecipientBox::new(vec![0x10], [0x10; 24], vec![0x11], [0x10; 16], [0x10; 16]),
        );

        snapshot.notes.insert(
            note.note_commit,
            NoteRecord {
                note: note.clone(),
                created_in_block: Some(0),
                status: NoteStatus::Unspent,
            },
        );

        let tx = Transaction::TransferNote(TransferNoteTx {
            core: TxCore {
                version: 1,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: vec![InputRef { note_commit: note.note_commit }],
                input_nullifiers: vec![Nullifier([0xE1; 32])],
                outputs: vec![sample_note(0x30)],
                fee: 0,
                statement_commit: [0xF0; 32],
                auth: vec![], // placeholder
            },
        });

        let auth = InputAuth {
            policy_tag: SpendPolicyTag::Single as u8,
            signer_pks: vec![pk],
            signatures: vec![], // compute later
            policy_opening: Some(policy_bytes),
            escrow_action: None,
        };

        (snapshot, tx, auth, sk.to_vec())
    }

    fn finalize_tx_with_auth(mut tx: Transaction, mut auth: privai_chain::InputAuth, sk: Option<&[u8]>) -> Transaction {
        // set auth first to compute the correct signing hash
        if let Transaction::TransferNote(ref mut t) = tx {
            t.core.auth = vec![auth.clone()];
        } else {
            panic!("expected TransferNoteTx");
        }

        if let Some(secret_key) = sk {
            let tx_hash = tx.tx_signing_hash();
            let sig = nxms_transport::crypto::falcon_sign_ct_prepared(secret_key, &tx_hash).expect("sign");
            auth.signatures = vec![sig];
            if let Transaction::TransferNote(ref mut t) = tx {
                t.core.auth = vec![auth];
            }
        } else {
            if let Transaction::TransferNote(ref mut t) = tx {
                t.core.auth = vec![auth];
            }
        }
        tx
    }

    #[test]
    fn fullprivacy_missing_policy_opening() {
        let (snapshot, tx, mut auth, _sk) = setup_single_auth_test();
        auth.policy_opening = None;
        let tx = finalize_tx_with_auth(tx, auth, None);
        let result = validate_transaction(&tx, &snapshot, 0, 1);
        assert!(matches!(result, Err(ValidationError::MissingPolicyOpening)));
    }

    #[test]
    fn fullprivacy_policy_mismatch() {
        use privai_chain::CanonicalEncode;
        let (snapshot, tx, mut auth, _sk) = setup_single_auth_test();
        // Modify policy opening to valid policy but wrong commit
        let other_policy = privai_chain::note::SpendPolicy::Single {
            falcon_pk_hash: [0x99; 32],
        };
        auth.policy_opening = Some(other_policy.to_canonical_bytes());
        let tx = finalize_tx_with_auth(tx, auth, None);
        let result = validate_transaction(&tx, &snapshot, 0, 1);
        assert!(matches!(result, Err(ValidationError::PolicyMismatch)));
    }

    #[test]
    fn fullprivacy_policy_tag_mismatch() {
        let (snapshot, tx, mut auth, _sk) = setup_single_auth_test();
        // Change tag to Escrow2of3 while opening remains Single
        auth.policy_tag = privai_chain::SpendPolicyTag::Escrow2of3 as u8;
        let tx = finalize_tx_with_auth(tx, auth, None);
        let result = validate_transaction(&tx, &snapshot, 0, 1);
        assert!(matches!(result, Err(ValidationError::PolicyTagMismatch)));
    }

    #[test]
    fn fullprivacy_valid_single_spend() {
        let (snapshot, tx, auth, sk) = setup_single_auth_test();
        let tx = finalize_tx_with_auth(tx, auth, Some(&sk));
        let result = validate_transaction(&tx, &snapshot, 0, 1);
        assert!(result.is_ok(), "Valid single FullPrivacy spend should succeed, got: {:?}", result);
    }

    #[test]
    fn fullprivacy_invalid_single_signer_pk_hash() {
        let (snapshot, tx, mut auth, sk) = setup_single_auth_test();
        let (_, wrong_pk) = nxms_transport::crypto::falcon_keygen().expect("keygen");
        auth.signer_pks = vec![wrong_pk];
        let tx = finalize_tx_with_auth(tx, auth, Some(&sk));
        let result = validate_transaction(&tx, &snapshot, 0, 1);
        assert!(
            matches!(result, Err(ValidationError::InvalidAuth(ref msg)) if msg.contains("signer pk hash mismatch"))
        );
    }

    #[test]
    fn fullprivacy_invalid_single_signature() {
        let (snapshot, tx, auth, _sk) = setup_single_auth_test();
        // Give a random wrong signature
        let (wrong_sk, _) = nxms_transport::crypto::falcon_keygen().expect("keygen");
        let tx = finalize_tx_with_auth(tx, auth, Some(&wrong_sk));
        let result = validate_transaction(&tx, &snapshot, 0, 1);
        assert!(
            matches!(result, Err(ValidationError::InvalidAuth(ref msg)) if msg.contains("invalid Falcon signature"))
        );
    }

    #[test]
    fn fullprivacy_single_must_not_carry_escrow_action() {
        let (snapshot, tx, mut auth, sk) = setup_single_auth_test();
        auth.escrow_action = Some(1);
        let tx = finalize_tx_with_auth(tx, auth, Some(&sk));
        let result = validate_transaction(&tx, &snapshot, 0, 1);
        assert!(
            matches!(result, Err(ValidationError::InvalidAuth(ref msg)) if msg.contains("Single policy should not have escrow_action"))
        );
    }

    #[test]
    fn fullprivacy_marketplacesettlement_unsupported() {
        use privai_chain::{InputAuth, Nullifier, SpendPolicyTag, note::SpendPolicy};
        use privai_chain::CanonicalEncode;

        let mut snapshot = LedgerSnapshot::genesis(17);
        let policy = SpendPolicy::MarketplaceSettlement {
            buyer_pk_hash: [0; 32],
            seller_pk_hash: [1; 32],
            moderator_pk_hash: [2; 32],
            timeout_block: 100,
        };
        let policy_commit = policy.commitment();
        let policy_bytes = policy.to_canonical_bytes();

        let note = OutputNote::new(
            policy_commit,
            privai_chain::LweCiphertext::default(),
            [0x10; 32],
            RecipientBox::new(vec![0x10], [0x10; 24], vec![0x11], [0x10; 16], [0x10; 16]),
        );

        snapshot.notes.insert(
            note.note_commit,
            NoteRecord {
                note: note.clone(),
                created_in_block: Some(0),
                status: NoteStatus::Unspent,
            },
        );

        let auth = InputAuth {
            policy_tag: SpendPolicyTag::MarketplaceSettlement as u8,
            signer_pks: vec![],
            signatures: vec![],
            policy_opening: Some(policy_bytes),
            escrow_action: None,
        };

        let tx = Transaction::TransferNote(TransferNoteTx {
            core: TxCore {
                version: 1,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: vec![InputRef { note_commit: note.note_commit }],
                input_nullifiers: vec![Nullifier([0xE1; 32])],
                outputs: vec![sample_note(0x30)],
                fee: 0,
                statement_commit: [0xF0; 32],
                auth: vec![auth],
            },
        });

        let result = validate_transaction(&tx, &snapshot, 0, 1);
        assert!(
            matches!(result, Err(ValidationError::InvalidAuth(ref msg)) if msg.contains("MarketplaceSettlement unsupported in FullPrivacy"))
        );
    }

}
