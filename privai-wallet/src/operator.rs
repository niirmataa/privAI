use std::collections::HashMap;
use privai_chain::Hash32;
use privai_chain::small_payments::{SpendGrant, Receipt, SettlementBatchSummary};
use privai_chain::tx::{MarketplaceBatchTx, TxCore, TX_TYPE_MARKETPLACE_BATCH};
use privai_chain::merkle_root;

#[derive(Debug, Default)]
pub struct MarketplaceOperator {
    pub operator_commit: Hash32,
    pub active_grants: HashMap<Hash32, SpendGrant>,
    // Mapping from merchant_commit -> lists of receipts waiting to be batched
    pub pending_receipts: HashMap<Hash32, Vec<Receipt>>,
}

impl MarketplaceOperator {
    pub fn new(operator_commit: Hash32) -> Self {
        Self {
            operator_commit,
            active_grants: HashMap::new(),
            pending_receipts: HashMap::new(),
        }
    }

    /// PHASE 8.4: Grant Issuance
    /// Wystawia nowy SpendGrant po udowodnieniu prawa do depozytu.
    pub fn issue_grant(
        &mut self,
        merchant_commit: Hash32,
        session_scope: Hash32,
        spend_cap: u64,
        grant_expiry: u64,
        settlement_window: u64,
        policy_commit: Hash32,
    ) -> SpendGrant {
        let grant = SpendGrant {
            merchant_commit,
            service_commit: None,
            session_scope,
            spend_cap,
            grant_expiry,
            settlement_window,
            policy_commit,
            operator_sig: vec![], // Signed off-chain by actual operator keys
        };
        self.active_grants.insert(grant.grant_commit(), grant.clone());
        grant
    }

    /// PHASE 8.4: Receipt Intake
    /// Przyjmuje Receipt od merchanta i odkłada do odpowiedniego batcha.
    pub fn intake_receipt(&mut self, receipt: Receipt) -> Result<(), String> {
        let grant = self
            .active_grants
            .get(&receipt.grant_commit)
            .ok_or_else(|| "Unknown or expired SpendGrant".to_string())?;

        if receipt.amount > grant.spend_cap {
            return Err("Receipt amount exceeds spend cap".to_string());
        }

        let merchant_receipts = self
            .pending_receipts
            .entry(receipt.merchant_commit)
            .or_insert_with(Vec::new);

        // Deduplication could happen here based on receipt_id/ticket_nullifier
        for existing in merchant_receipts.iter() {
            if existing.ticket_nullifier == receipt.ticket_nullifier {
                return Err("Duplicate ticket nullifier used in this batch".to_string());
            }
        }

        merchant_receipts.push(receipt);
        Ok(())
    }

    /// PHASE 8.4: Batch Settlement Publisher
    /// Tworzy ostateczny `MarketplaceBatchTx` (On-Chain Settlement) na podstawie uzbieranych receiptów.
    pub fn publish_settlement_batch(
        &mut self,
        merchant_commit: Hash32,
        grant_commit: Hash32,
        window_start: u64,
        window_end: u64,
    ) -> Result<MarketplaceBatchTx, String> {
        let receipts = self
            .pending_receipts
            .remove(&merchant_commit)
            .ok_or_else(|| "No pending receipts for this merchant".to_string())?;

        let grant = self
            .active_grants
            .get(&grant_commit)
            .ok_or_else(|| "Unknown or expired SpendGrant".to_string())?;

        let mut total_gross: u64 = 0;
        let mut ticket_nullifiers = Vec::new();
        let mut receipt_commits = Vec::new();

        for receipt in &receipts {
            total_gross = total_gross.saturating_add(receipt.amount);
            ticket_nullifiers.push(receipt.ticket_nullifier);
            receipt_commits.push(receipt.receipt_commit());
        }

        let receipt_root = merkle_root(receipt_commits);
        
        let total_refund = grant.spend_cap.saturating_sub(total_gross);

        let summary = SettlementBatchSummary {
            operator_commit: self.operator_commit,
            merchant_commit,
            grant_commit,
            settlement_window_start: window_start,
            settlement_window_end: window_end,
            receipt_root,
            receipt_count: receipts.len() as u32,
            nullifier_count: ticket_nullifiers.len() as u32,
            total_gross_amount: total_gross,
            total_fee_amount: 0,
            total_refund_amount: total_refund,
        };

        // Here we build the lightweight transaction core without large utxo logic
        let core = TxCore {
            version: 0,
            tx_type: TX_TYPE_MARKETPLACE_BATCH,
            inputs: vec![],
            input_nullifiers: vec![],
            outputs: vec![],
            fee: 0,
            statement_commit: summary.settlement_root(), // Use settlement root as statement commit
            auth: vec![],
        };

        let tx = MarketplaceBatchTx {
            core,
            summary,
            ticket_nullifiers,
            operator_sig: vec![], // Operator signs the whole struct externally
        };

        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use privai_chain::Nullifier;

    #[test]
    fn test_integration_flow_and_adversarial_rejections() {
        let operator_commit = [0x99; 32];
        let mut operator = MarketplaceOperator::new(operator_commit);

        let merchant_commit = [0x11; 32];
        let session_scope = [0x22; 32];
        let policy_commit = [0x33; 32];
        
        // 1. Operator wystawia SpendGrant
        let grant = operator.issue_grant(
            merchant_commit,
            session_scope,
            1000, // spend cap = 1000
            0,
            0,
            policy_commit,
        );

        let ticket_1_nullifier = Nullifier([0xaa; 32]);
        let ticket_2_nullifier = Nullifier([0xbb; 32]);

        // 2. Receipt Intake (dobry kwit)
        let receipt1 = Receipt {
            receipt_id: [1u8; 32],
            merchant_commit,
            service_commit: None,
            session_commit: session_scope,
            grant_commit: grant.grant_commit(),
            purchase_commit: [4u8; 32],
            ticket_nullifier: ticket_1_nullifier.clone(),
            amount: 200,
            policy_commit,
            result_commit: [5u8; 32],
            issued_at: 0,
            merchant_sig: vec![],
        };
        assert!(operator.intake_receipt(receipt1.clone()).is_ok());

        // 3. ADVERSARIAL: Replay attempt (ten sam nullifier)
        let receipt1_replay = Receipt {
            receipt_id: [2u8; 32],
            ticket_nullifier: ticket_1_nullifier.clone(),
            ..receipt1.clone()
        };
        assert!(operator.intake_receipt(receipt1_replay).is_err()); // duplicate ticket nullifier w jednym batchu

        // 4. ADVERSARIAL: Overcharge attempt (kwota wieksza niz cap grantu)
        let receipt_overcharge = Receipt {
            receipt_id: [3u8; 32],
            ticket_nullifier: ticket_2_nullifier.clone(),
            amount: 1200, // > 1000 (cap)
            ..receipt1.clone()
        };
        assert!(operator.intake_receipt(receipt_overcharge).is_err()); // exceeds spend cap

        // 5. Drugi dobry kwit
        let receipt2 = Receipt {
            receipt_id: [4u8; 32],
            ticket_nullifier: ticket_2_nullifier.clone(),
            amount: 300,
            ..receipt1.clone()
        };
        assert!(operator.intake_receipt(receipt2).is_ok());

        // 6. Publikacja Settlement Batch
        let batch_tx = operator.publish_settlement_batch(merchant_commit, grant.grant_commit(), 0, 1000).unwrap();

        // 7. Weryfikacja SettlementTx 
        assert_eq!(batch_tx.ticket_nullifiers.len(), 2);
        assert!(batch_tx.ticket_nullifiers.contains(&ticket_1_nullifier));
        assert!(batch_tx.ticket_nullifiers.contains(&ticket_2_nullifier));
        assert_eq!(batch_tx.summary.total_gross_amount, 500); // 200 + 300
        assert_eq!(batch_tx.summary.total_refund_amount, 500); // 1000 cap - 500
        assert_eq!(batch_tx.summary.receipt_count, 2);
        assert_eq!(batch_tx.core.tx_type, TX_TYPE_MARKETPLACE_BATCH);
    }
}
