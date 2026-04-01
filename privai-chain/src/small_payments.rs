use serde::{Deserialize, Serialize};

use crate::canonical::{
    CanonicalEncode, write_fixed, write_u8, write_u32, write_u64,
};
use crate::primitives::{Hash32, Nullifier};
use crate::hash::domain_hash;

pub const POLICY_DOMAIN: &str = "nxms_privai_policy_v0";
pub const GRANT_DOMAIN: &str = "nxms_privai_grant_v0";
pub const RECEIPT_DOMAIN: &str = "nxms_privai_receipt_v0";
pub const RECEIPT_ROOT_DOMAIN: &str = "nxms_privai_receipt_root_v0";
pub const SETTLEMENT_ROOT_DOMAIN: &str = "nxms_privai_settlement_root_v0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum AllowedRail {
    SmallPaymentsRail = 0x01,
    RecipientPrivacyLite = 0x02,
    FullPrivacy = 0x03,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PricingMode {
    ReservationThenSettle = 0x01,
    ExactAmount = 0x02,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServicePaymentPolicy {
    pub policy_version: u8,
    pub merchant_commit: Hash32,
    pub service_commit: Option<Hash32>,
    pub allowed_rail: AllowedRail,
    pub pricing_mode: PricingMode,
    pub min_deposit_required: u64,
    pub max_spend_per_session: u64,
    pub max_spend_per_window: u64,
    pub grant_expiry_rule: u32,       // seconds
    pub settlement_window_rule: u32,  // seconds
    pub requires_full_privacy_if: u64, // amount threshold
}

impl CanonicalEncode for ServicePaymentPolicy {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.policy_version);
        write_fixed(out, &self.merchant_commit);
        if let Some(svc) = &self.service_commit {
            write_u8(out, 1);
            write_fixed(out, svc);
        } else {
            write_u8(out, 0);
        }
        write_u8(out, self.allowed_rail as u8);
        write_u8(out, self.pricing_mode as u8);
        write_u64(out, self.min_deposit_required);
        write_u64(out, self.max_spend_per_session);
        write_u64(out, self.max_spend_per_window);
        write_u32(out, self.grant_expiry_rule);
        write_u32(out, self.settlement_window_rule);
        write_u64(out, self.requires_full_privacy_if);
    }
}

impl ServicePaymentPolicy {
    pub fn policy_commit(&self) -> Hash32 {
        domain_hash(POLICY_DOMAIN, &[&self.to_canonical_bytes()])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendGrant {
    pub merchant_commit: Hash32,
    pub service_commit: Option<Hash32>,
    pub session_scope: Hash32,
    pub spend_cap: u64,
    pub grant_expiry: u64, // absolute timestamp
    pub settlement_window: u64, // absolute timestamp
    pub policy_commit: Hash32,
    pub operator_sig: Vec<u8>,
}

impl CanonicalEncode for SpendGrant {
    fn encode(&self, out: &mut Vec<u8>) {
        write_fixed(out, &self.merchant_commit);
        if let Some(svc) = &self.service_commit {
            write_u8(out, 1);
            write_fixed(out, svc);
        } else {
            write_u8(out, 0);
        }
        write_fixed(out, &self.session_scope);
        write_u64(out, self.spend_cap);
        write_u64(out, self.grant_expiry);
        write_u64(out, self.settlement_window);
        write_fixed(out, &self.policy_commit);
        // Signature is NOT part of the signed payload usually, or if it is it's outside grant_commit
    }
}

impl SpendGrant {
    pub fn grant_commit(&self) -> Hash32 {
        domain_hash(GRANT_DOMAIN, &[&self.to_canonical_bytes()])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    pub receipt_id: Hash32,
    pub merchant_commit: Hash32,
    pub service_commit: Option<Hash32>,
    pub session_commit: Hash32,
    pub grant_commit: Hash32,
    pub purchase_commit: Hash32,
    pub ticket_nullifier: Nullifier,
    pub amount: u64,
    pub policy_commit: Hash32,
    pub result_commit: Hash32,
    pub issued_at: u64,
    pub merchant_sig: Vec<u8>,
}

impl CanonicalEncode for Receipt {
    fn encode(&self, out: &mut Vec<u8>) {
        write_fixed(out, &self.receipt_id);
        write_fixed(out, &self.merchant_commit);
        if let Some(svc) = &self.service_commit {
            write_u8(out, 1);
            write_fixed(out, svc);
        } else {
            write_u8(out, 0);
        }
        write_fixed(out, &self.session_commit);
        write_fixed(out, &self.grant_commit);
        write_fixed(out, &self.purchase_commit);
        write_fixed(out, &self.ticket_nullifier.0);
        write_u64(out, self.amount);
        write_fixed(out, &self.policy_commit);
        write_fixed(out, &self.result_commit);
        write_u64(out, self.issued_at);
        // Exclude signature from canonical hash typically used for receipt_root leaf
    }
}

impl Receipt {
    pub fn receipt_commit(&self) -> Hash32 {
        domain_hash(RECEIPT_DOMAIN, &[&self.to_canonical_bytes()])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementBatchSummary {
    pub operator_commit: Hash32,
    pub merchant_commit: Hash32,
    pub grant_commit: Hash32,
    pub settlement_window_start: u64,
    pub settlement_window_end: u64,
    pub receipt_root: Hash32,
    pub receipt_count: u32,
    pub nullifier_count: u32,
    pub total_gross_amount: u64,
    pub total_fee_amount: u64,
    pub total_refund_amount: u64,
}

impl CanonicalEncode for SettlementBatchSummary {
    fn encode(&self, out: &mut Vec<u8>) {
        write_fixed(out, &self.operator_commit);
        write_fixed(out, &self.merchant_commit);
        write_fixed(out, &self.grant_commit);
        write_u64(out, self.settlement_window_start);
        write_u64(out, self.settlement_window_end);
        write_fixed(out, &self.receipt_root);
        write_u32(out, self.receipt_count);
        write_u32(out, self.nullifier_count);
        write_u64(out, self.total_gross_amount);
        write_u64(out, self.total_fee_amount);
        write_u64(out, self.total_refund_amount);
    }
}

impl SettlementBatchSummary {
    pub fn settlement_root(&self) -> Hash32 {
        domain_hash(SETTLEMENT_ROOT_DOMAIN, &[&self.to_canonical_bytes()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_payment_policy_canonical_encoding_and_commit() {
        let policy1 = ServicePaymentPolicy {
            policy_version: 1,
            merchant_commit: [1u8; 32],
            service_commit: Some([2u8; 32]),
            allowed_rail: AllowedRail::SmallPaymentsRail,
            pricing_mode: PricingMode::ReservationThenSettle,
            min_deposit_required: 1000,
            max_spend_per_session: 5000,
            max_spend_per_window: 10000,
            grant_expiry_rule: 3600,
            settlement_window_rule: 86400,
            requires_full_privacy_if: 50000,
        };

        let policy2 = ServicePaymentPolicy {
            policy_version: 1,
            merchant_commit: [1u8; 32],
            service_commit: Some([2u8; 32]),
            allowed_rail: AllowedRail::SmallPaymentsRail,
            pricing_mode: PricingMode::ReservationThenSettle,
            min_deposit_required: 1000,
            max_spend_per_session: 5000,
            max_spend_per_window: 10000,
            grant_expiry_rule: 3600,
            settlement_window_rule: 86400,
            requires_full_privacy_if: 50000,
        };

        let policy_different = ServicePaymentPolicy {
            policy_version: 1,
            merchant_commit: [1u8; 32],
            service_commit: None, // different
            allowed_rail: AllowedRail::SmallPaymentsRail,
            pricing_mode: PricingMode::ReservationThenSettle,
            min_deposit_required: 1000,
            max_spend_per_session: 5000,
            max_spend_per_window: 10000,
            grant_expiry_rule: 3600,
            settlement_window_rule: 86400,
            requires_full_privacy_if: 50000,
        };

        // Identical structs should produce identical bytes & commits
        assert_eq!(policy1.to_canonical_bytes(), policy2.to_canonical_bytes());
        assert_eq!(policy1.policy_commit(), policy2.policy_commit());

        // Different structs should produce different commits
        assert_ne!(policy1.policy_commit(), policy_different.policy_commit());
    }

    #[test]
    fn test_spend_grant_commit_derivation() {
        let grant = SpendGrant {
            merchant_commit: [1u8; 32],
            service_commit: None,
            session_scope: [3u8; 32],
            spend_cap: 500,
            grant_expiry: 1670000000,
            settlement_window: 1670086400,
            policy_commit: [4u8; 32],
            operator_sig: vec![0xaa, 0xbb, 0xcc],
        };

        let commit = grant.grant_commit();
        // Just verify it doesn't panic and produces a 32-byte hash
        assert_ne!(commit, [0u8; 32]);
    }
}
