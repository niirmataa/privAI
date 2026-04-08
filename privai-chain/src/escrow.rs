//! Escrow types for policy-constrained 2-of-3 multisig (escrow-2of3-v1).
//!
//! Architecture:
//! - `policy_tag + policy_version` imply a frozen rule table.
//! - Ledger reads action constraints from the implied rule table,
//!   NOT from individual policy fields.
//! - `signer_role` is control-plane metadata only — ledger derives
//!   roles from `policy_opening`.
//!
//! See: spec/PRIVAI_ESCROW_OBJECT_MODEL.md
//!      spec/PRIVAI_ESCROW_TX_MATRIX.md

use serde::{Deserialize, Serialize};

/// Escrow actions for `escrow-2of3-v1`.
///
/// Frozen rule table (implied by policy_tag):
/// - Release:         Buyer + Operator → output to Merchant (normal mode)
/// - Refund:          Merchant + Operator → output to Buyer (normal mode)
/// - RecoveryRelease: Buyer + Merchant → after timeout_block (recovery mode)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EscrowAction {
    Release = 0x01,
    Refund = 0x02,
    RecoveryRelease = 0x03,
}

impl EscrowAction {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::Release),
            0x02 => Some(Self::Refund),
            0x03 => Some(Self::RecoveryRelease),
            _ => None,
        }
    }
}

/// Signer roles within an escrow-2of3 policy.
///
/// Roles are derived by ledger from the policy's pk_hashes.
/// NOT self-declared by the signer — signer_role in control-plane
/// messages is metadata only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SignerRole {
    /// Index 0 in canonical signer ordering.
    Buyer = 0x00,
    /// Index 1 in canonical signer ordering.
    Merchant = 0x01,
    /// Index 2 in canonical signer ordering.
    Operator = 0x02,
}

impl SignerRole {
    /// Canonical signer index (by position in policy).
    pub fn index(self) -> usize {
        self as usize
    }
}

/// Frozen rule table for `escrow-2of3-v1`: required signer pair per action.
pub fn required_signers(action: EscrowAction) -> (SignerRole, SignerRole) {
    match action {
        EscrowAction::Release => (SignerRole::Buyer, SignerRole::Operator),
        EscrowAction::Refund => (SignerRole::Merchant, SignerRole::Operator),
        EscrowAction::RecoveryRelease => (SignerRole::Buyer, SignerRole::Merchant),
    }
}

/// Whether the action requires recovery mode (timeout must have passed).
pub fn is_recovery_action(action: EscrowAction) -> bool {
    matches!(action, EscrowAction::RecoveryRelease)
}

/// Output target constraint for an escrow action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetRecipient {
    /// Output must go to exactly this role.
    One(SignerRole),
    /// Output may go to either role (recovery).
    Either(SignerRole, SignerRole),
}

/// Target recipient per action from frozen rule table.
pub fn target_recipient(action: EscrowAction) -> TargetRecipient {
    match action {
        EscrowAction::Release => TargetRecipient::One(SignerRole::Merchant),
        EscrowAction::Refund => TargetRecipient::One(SignerRole::Buyer),
        EscrowAction::RecoveryRelease => {
            TargetRecipient::Either(SignerRole::Buyer, SignerRole::Merchant)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_roundtrip() {
        for byte in [0x01, 0x02, 0x03] {
            let action = EscrowAction::from_u8(byte).unwrap();
            assert_eq!(action as u8, byte);
        }
        assert!(EscrowAction::from_u8(0x00).is_none());
        assert!(EscrowAction::from_u8(0x04).is_none());
    }

    #[test]
    fn release_requires_buyer_and_operator() {
        let (a, b) = required_signers(EscrowAction::Release);
        assert_eq!(a, SignerRole::Buyer);
        assert_eq!(b, SignerRole::Operator);
    }

    #[test]
    fn refund_requires_merchant_and_operator() {
        let (a, b) = required_signers(EscrowAction::Refund);
        assert_eq!(a, SignerRole::Merchant);
        assert_eq!(b, SignerRole::Operator);
    }

    #[test]
    fn recovery_requires_buyer_and_merchant() {
        let (a, b) = required_signers(EscrowAction::RecoveryRelease);
        assert_eq!(a, SignerRole::Buyer);
        assert_eq!(b, SignerRole::Merchant);
        assert!(is_recovery_action(EscrowAction::RecoveryRelease));
        assert!(!is_recovery_action(EscrowAction::Release));
        assert!(!is_recovery_action(EscrowAction::Refund));
    }

    #[test]
    fn signer_role_indices_are_canonical() {
        assert_eq!(SignerRole::Buyer.index(), 0);
        assert_eq!(SignerRole::Merchant.index(), 1);
        assert_eq!(SignerRole::Operator.index(), 2);
    }
}
