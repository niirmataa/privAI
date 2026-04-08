//! Escrow validation for `escrow-2of3-v1`.
//!
//! This module validates escrow spend transactions at the ledger level.
//! The proof path is NOT modified — escrow spends use the existing
//! `TransferNoteTx` proof path (mixed v1 model).
//!
//! What this module covers (ledger-only):
//! - policy reconstruction from `policy_opening`
//! - `spend_policy_commit` binding
//! - signer identification, ordering, duplicate rejection
//! - action/signer combination validation (frozen rule table)
//! - recovery timeout check
//! - output target validation
//! - Falcon signature verification against `tx_signing_hash`
//!
//! What proof covers (NOT this module):
//! - note commitments, nullifiers, balance
//!
//! See: spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md

use privai_chain::decode::CanonicalDecode;
use privai_chain::escrow::{
    EscrowAction, SignerRole, TargetRecipient, is_recovery_action, required_signers,
    target_recipient,
};
use privai_chain::hash::falcon_pk_hash;
use privai_chain::note::{OutputNote, SpendPolicy};
use privai_chain::primitives::Hash32;
use privai_chain::tx::InputAuth;

use crate::error::ValidationError;

/// Validate a single escrow auth entry against the reconstructed policy.
///
/// Called from `validate_transaction` when `auth[i].policy_tag == Escrow2of3`.
///
/// # Arguments
/// - `auth_index`: position in `tx.auth[]` (for error messages)
/// - `auth`: the escrow auth entry (must have policy_tag == 0x03)
/// - `input_note_policy_commit`: `spend_policy_commit` from the input note in ledger state
/// - `outputs`: all transaction outputs (for target validation)
/// - `tx_signing_hash`: the canonical signing message (NOT tx_id)
/// - `current_block_height`: block height being validated (for timeout check)
pub fn validate_escrow_auth(
    auth_index: usize,
    auth: &InputAuth,
    input_note_policy_commit: &Hash32,
    outputs: &[OutputNote],
    tx_signing_hash: &Hash32,
    current_block_height: u64,
) -> Result<(), ValidationError> {
    // 1. Require escrow-specific fields
    let policy_opening_bytes = auth
        .policy_opening
        .as_ref()
        .ok_or(ValidationError::EscrowMissingPolicyOpening)?;
    let action_byte = auth
        .escrow_action
        .ok_or(ValidationError::EscrowMissingAction)?;
    let action = EscrowAction::from_u8(action_byte)
        .ok_or(ValidationError::EscrowInvalidAction(action_byte))?;

    // 2. Reconstruct policy from policy_opening (canonical decode)
    let policy = SpendPolicy::from_canonical_bytes(policy_opening_bytes)
        .map_err(|e| ValidationError::EscrowPolicyDecode(e.to_string()))?;

    // 3. Verify it's an Escrow2of3 policy (reject unsupported policy type)
    let (buyer_pk_hash, merchant_pk_hash, operator_pk_hash, timeout_block) = match &policy {
        SpendPolicy::Escrow2of3 {
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
            timeout_block,
        } => (*buyer_pk_hash, *merchant_pk_hash, *operator_pk_hash, *timeout_block),
        _ => return Err(ValidationError::EscrowUnsupportedPolicy),
    };

    // 4. Verify spend_policy_commit binding:
    //    policy_opening must produce the same commitment as the on-chain note.
    let computed_commit = policy.commitment();
    if computed_commit != *input_note_policy_commit {
        return Err(ValidationError::EscrowPolicyMismatch);
    }

    // 5. Verify signer count (escrow-2of3 requires exactly 2 signers)
    if auth.signer_pks.len() != 2 {
        return Err(ValidationError::EscrowWrongSignerCount(
            auth.signer_pks.len(),
        ));
    }
    if auth.signatures.len() != 2 {
        return Err(ValidationError::EscrowWrongSignerCount(
            auth.signatures.len(),
        ));
    }

    // 6. Identify signer roles by hashing PKs and comparing to policy fields.
    //    Roles are DERIVED, not self-declared.
    let mut signer_roles: [(SignerRole, usize); 2] = [(SignerRole::Buyer, 0); 2];
    for (j, pk) in auth.signer_pks.iter().enumerate() {
        let hash = falcon_pk_hash(pk);
        let role = if hash == buyer_pk_hash {
            SignerRole::Buyer
        } else if hash == merchant_pk_hash {
            SignerRole::Merchant
        } else if hash == operator_pk_hash {
            SignerRole::Operator
        } else {
            return Err(ValidationError::EscrowUnknownSigner(hash));
        };
        signer_roles[j] = (role, j);
    }

    // 7. Duplicate signer rejection
    if signer_roles[0].0 == signer_roles[1].0 {
        return Err(ValidationError::EscrowDuplicateSigner);
    }

    // 8. Canonical ordering check: signers must be ordered by policy index (ascending)
    if signer_roles[0].0.index() > signer_roles[1].0.index() {
        return Err(ValidationError::EscrowSignerOrderViolation);
    }

    // 9. Verify signer combination matches declared action (frozen rule table)
    let (required_a, required_b) = required_signers(action);
    let has_required_a = signer_roles.iter().any(|(r, _)| *r == required_a);
    let has_required_b = signer_roles.iter().any(|(r, _)| *r == required_b);
    if !has_required_a || !has_required_b {
        return Err(ValidationError::EscrowWrongSignerCombination);
    }

    // 10. Recovery timeout check:
    //     Recovery mode is only available after timeout_block.
    //     Before timeout, only normal mode (operator-required) is available.
    if is_recovery_action(action) && current_block_height < timeout_block {
        return Err(ValidationError::EscrowRecoveryBeforeTimeout {
            current: current_block_height,
            required: timeout_block,
        });
    }

    // 11. Verify Falcon signatures against tx_signing_hash (NOT tx_id)
    for (j, (pk, sig)) in auth
        .signer_pks
        .iter()
        .zip(auth.signatures.iter())
        .enumerate()
    {
        if pk.is_empty() || sig.is_empty() {
            return Err(ValidationError::InvalidAuth(format!(
                "auth[{}][{}]: empty pk or sig",
                auth_index, j
            )));
        }
        nxms_transport::crypto::falcon_verify(pk, tx_signing_hash, sig).map_err(|_| {
            ValidationError::InvalidAuth(format!(
                "auth[{}][{}]: invalid Falcon signature",
                auth_index, j
            ))
        })?;
    }

    // 12. Output target validation (P1 fix):
    //     ALL outputs must go to allowed recipients for this action.
    //     v1 = whole amount to one party, no split to unknown addresses.
    //     This prevents siphoning: e.g. 1 unit to merchant + 999 to attacker.
    validate_output_target(
        &action,
        outputs,
        &buyer_pk_hash,
        &merchant_pk_hash,
        &operator_pk_hash,
    )?;

    Ok(())
}

/// Validate that ALL outputs are directed to allowed recipients.
///
/// v1 rule: every output's `spend_policy_commit` must match a Single policy
/// for one of the allowed recipient roles. No output may go to an unknown party.
fn validate_output_target(
    action: &EscrowAction,
    outputs: &[OutputNote],
    buyer_pk_hash: &Hash32,
    merchant_pk_hash: &Hash32,
    _operator_pk_hash: &Hash32,
) -> Result<(), ValidationError> {
    if outputs.is_empty() {
        return Err(ValidationError::EscrowOutputTargetMismatch);
    }

    let target = target_recipient(*action);

    let allowed_commits: Vec<Hash32> = match target {
        TargetRecipient::One(role) => {
            vec![single_commit(pk_hash_for_role(role, buyer_pk_hash, merchant_pk_hash))]
        }
        TargetRecipient::Either(a, b) => {
            vec![
                single_commit(pk_hash_for_role(a, buyer_pk_hash, merchant_pk_hash)),
                single_commit(pk_hash_for_role(b, buyer_pk_hash, merchant_pk_hash)),
            ]
        }
    };

    // Every output must belong to the allowed set
    for output in outputs {
        if !allowed_commits.contains(&output.spend_policy_commit) {
            return Err(ValidationError::EscrowOutputTargetMismatch);
        }
    }

    Ok(())
}

/// Compute spend_policy_commit for a Single policy with the given pk_hash.
fn single_commit(pk_hash: &Hash32) -> Hash32 {
    SpendPolicy::Single {
        falcon_pk_hash: *pk_hash,
    }
    .commitment()
}

fn pk_hash_for_role<'a>(
    role: SignerRole,
    buyer: &'a Hash32,
    merchant: &'a Hash32,
) -> &'a Hash32 {
    match role {
        SignerRole::Buyer => buyer,
        SignerRole::Merchant => merchant,
        SignerRole::Operator => {
            // Operator is never a valid output target in v1.
            buyer // unreachable for v1 rule table
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use privai_chain::canonical::CanonicalEncode;
    use privai_chain::hash::falcon_pk_hash;
    use privai_chain::note::{OutputNote, RecipientBox, SpendPolicy, SpendPolicyTag};
    use privai_chain::primitives::LweCiphertext;

    /// Helper: create a SpendPolicy::Escrow2of3 from raw pk bytes.
    fn make_escrow_policy(
        buyer_pk: &[u8],
        merchant_pk: &[u8],
        operator_pk: &[u8],
        timeout: u64,
    ) -> SpendPolicy {
        SpendPolicy::Escrow2of3 {
            buyer_pk_hash: falcon_pk_hash(buyer_pk),
            merchant_pk_hash: falcon_pk_hash(merchant_pk),
            operator_pk_hash: falcon_pk_hash(operator_pk),
            timeout_block: timeout,
        }
    }

    /// Helper: create an output note with a given spend_policy_commit.
    fn make_output_with_policy(policy_commit: Hash32) -> OutputNote {
        OutputNote::new(
            policy_commit,
            LweCiphertext::default(),
            [0x99; 32],
            RecipientBox::new(vec![1], [2; 24], vec![3], [4; 16], [5; 16]),
        )
    }

    /// Helper: build an InputAuth for escrow.
    fn make_escrow_auth(
        signer_pks: Vec<Vec<u8>>,
        signatures: Vec<Vec<u8>>,
        policy: &SpendPolicy,
        action: EscrowAction,
    ) -> InputAuth {
        InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks,
            signatures,
            policy_opening: Some(policy.to_canonical_bytes()),
            escrow_action: Some(action as u8),
        }
    }

    const BUYER_PK: &[u8] = &[0xB0; 64];
    const MERCHANT_PK: &[u8] = &[0xC0; 64];
    const OPERATOR_PK: &[u8] = &[0xD0; 64];

    fn test_policy() -> SpendPolicy {
        make_escrow_policy(BUYER_PK, MERCHANT_PK, OPERATOR_PK, 1000)
    }

    fn merchant_output() -> OutputNote {
        let commit = SpendPolicy::Single {
            falcon_pk_hash: falcon_pk_hash(MERCHANT_PK),
        }
        .commitment();
        make_output_with_policy(commit)
    }

    fn buyer_output() -> OutputNote {
        let commit = SpendPolicy::Single {
            falcon_pk_hash: falcon_pk_hash(BUYER_PK),
        }
        .commitment();
        make_output_with_policy(commit)
    }

    // ── Policy reconstruction tests ────────────────────────────────────

    #[test]
    fn reject_missing_policy_opening() {
        let auth = InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![BUYER_PK.to_vec(), OPERATOR_PK.to_vec()],
            signatures: vec![vec![0xAA; 32], vec![0xBB; 32]],
            policy_opening: None,
            escrow_action: Some(EscrowAction::Release as u8),
        };
        let result = validate_escrow_auth(0, &auth, &[0; 32], &[], &[0; 32], 0);
        assert!(matches!(result, Err(ValidationError::EscrowMissingPolicyOpening)));
    }

    #[test]
    fn reject_missing_action() {
        let policy = test_policy();
        let auth = InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![BUYER_PK.to_vec(), OPERATOR_PK.to_vec()],
            signatures: vec![vec![0xAA; 32], vec![0xBB; 32]],
            policy_opening: Some(policy.to_canonical_bytes()),
            escrow_action: None,
        };
        let result = validate_escrow_auth(0, &auth, &[0; 32], &[], &[0; 32], 0);
        assert!(matches!(result, Err(ValidationError::EscrowMissingAction)));
    }

    #[test]
    fn reject_invalid_action_byte() {
        let policy = test_policy();
        let auth = make_escrow_auth(
            vec![BUYER_PK.to_vec(), OPERATOR_PK.to_vec()],
            vec![vec![0xAA; 32], vec![0xBB; 32]],
            &policy,
            EscrowAction::Release,
        );
        let mut auth = auth;
        auth.escrow_action = Some(0xFF);
        let result = validate_escrow_auth(0, &auth, &policy.commitment(), &[], &[0; 32], 0);
        assert!(matches!(result, Err(ValidationError::EscrowInvalidAction(0xFF))));
    }

    #[test]
    fn reject_policy_mismatch() {
        let policy = test_policy();
        let wrong_commit = [0xDE; 32]; // doesn't match policy
        let auth = make_escrow_auth(
            vec![BUYER_PK.to_vec(), OPERATOR_PK.to_vec()],
            vec![vec![0xAA; 32], vec![0xBB; 32]],
            &policy,
            EscrowAction::Release,
        );
        let result = validate_escrow_auth(
            0,
            &auth,
            &wrong_commit,
            &[merchant_output()],
            &[0; 32],
            0,
        );
        assert!(matches!(result, Err(ValidationError::EscrowPolicyMismatch)));
    }

    #[test]
    fn reject_non_escrow_policy_in_opening() {
        let single_policy = SpendPolicy::Single {
            falcon_pk_hash: [1; 32],
        };
        let auth = InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![BUYER_PK.to_vec(), OPERATOR_PK.to_vec()],
            signatures: vec![vec![0xAA; 32], vec![0xBB; 32]],
            policy_opening: Some(single_policy.to_canonical_bytes()),
            escrow_action: Some(EscrowAction::Release as u8),
        };
        let result = validate_escrow_auth(
            0,
            &auth,
            &single_policy.commitment(),
            &[],
            &[0; 32],
            0,
        );
        assert!(matches!(result, Err(ValidationError::EscrowUnsupportedPolicy)));
    }

    // ── Signer validation tests ────────────────────────────────────────

    #[test]
    fn reject_unknown_signer() {
        let policy = test_policy();
        let unknown_pk = vec![0xFF; 64];
        let auth = make_escrow_auth(
            vec![BUYER_PK.to_vec(), unknown_pk],
            vec![vec![0xAA; 32], vec![0xBB; 32]],
            &policy,
            EscrowAction::Release,
        );
        let result = validate_escrow_auth(
            0,
            &auth,
            &policy.commitment(),
            &[merchant_output()],
            &[0; 32],
            0,
        );
        assert!(matches!(result, Err(ValidationError::EscrowUnknownSigner(_))));
    }

    #[test]
    fn reject_duplicate_signer() {
        let policy = test_policy();
        let auth = make_escrow_auth(
            vec![BUYER_PK.to_vec(), BUYER_PK.to_vec()],
            vec![vec![0xAA; 32], vec![0xBB; 32]],
            &policy,
            EscrowAction::Release,
        );
        let result = validate_escrow_auth(
            0,
            &auth,
            &policy.commitment(),
            &[merchant_output()],
            &[0; 32],
            0,
        );
        assert!(matches!(result, Err(ValidationError::EscrowDuplicateSigner)));
    }

    #[test]
    fn reject_wrong_signer_order() {
        let policy = test_policy();
        // Operator (index 2) before Buyer (index 0) — wrong order
        let auth = make_escrow_auth(
            vec![OPERATOR_PK.to_vec(), BUYER_PK.to_vec()],
            vec![vec![0xAA; 32], vec![0xBB; 32]],
            &policy,
            EscrowAction::Release,
        );
        let result = validate_escrow_auth(
            0,
            &auth,
            &policy.commitment(),
            &[merchant_output()],
            &[0; 32],
            0,
        );
        assert!(matches!(
            result,
            Err(ValidationError::EscrowSignerOrderViolation)
        ));
    }

    #[test]
    fn reject_wrong_signer_combination_for_release() {
        let policy = test_policy();
        // Merchant + Operator is for refund, not release
        let auth = make_escrow_auth(
            vec![MERCHANT_PK.to_vec(), OPERATOR_PK.to_vec()],
            vec![vec![0xAA; 32], vec![0xBB; 32]],
            &policy,
            EscrowAction::Release,
        );
        let result = validate_escrow_auth(
            0,
            &auth,
            &policy.commitment(),
            &[merchant_output()],
            &[0; 32],
            0,
        );
        assert!(matches!(
            result,
            Err(ValidationError::EscrowWrongSignerCombination)
        ));
    }

    #[test]
    fn reject_wrong_signer_count() {
        let policy = test_policy();
        // Only 1 signer — escrow requires exactly 2
        let auth = InputAuth {
            policy_tag: SpendPolicyTag::Escrow2of3 as u8,
            signer_pks: vec![BUYER_PK.to_vec()],
            signatures: vec![vec![0xAA; 32]],
            policy_opening: Some(policy.to_canonical_bytes()),
            escrow_action: Some(EscrowAction::Release as u8),
        };
        let result = validate_escrow_auth(
            0,
            &auth,
            &policy.commitment(),
            &[merchant_output()],
            &[0; 32],
            0,
        );
        assert!(matches!(
            result,
            Err(ValidationError::EscrowWrongSignerCount(1))
        ));
    }

    // ── Recovery timeout tests ────────────────────────────────────────

    #[test]
    fn reject_recovery_before_timeout() {
        let policy = test_policy(); // timeout_block = 1000
        let auth = make_escrow_auth(
            vec![BUYER_PK.to_vec(), MERCHANT_PK.to_vec()],
            vec![vec![0xAA; 32], vec![0xBB; 32]],
            &policy,
            EscrowAction::RecoveryRelease,
        );
        // current block = 999, timeout = 1000
        let result = validate_escrow_auth(
            0,
            &auth,
            &policy.commitment(),
            &[buyer_output()],
            &[0; 32],
            999,
        );
        assert!(matches!(
            result,
            Err(ValidationError::EscrowRecoveryBeforeTimeout {
                current: 999,
                required: 1000,
            })
        ));
    }

    // Note: valid recovery after timeout would pass the timeout check but
    // fail at signature verification (we use dummy signatures). The timeout
    // gate itself is tested by the reject test above.

    // ── Output target tests ───────────────────────────────────────────

    #[test]
    fn reject_wrong_output_target_for_release() {
        let policy = test_policy();
        let auth = make_escrow_auth(
            vec![BUYER_PK.to_vec(), OPERATOR_PK.to_vec()],
            vec![vec![0xAA; 32], vec![0xBB; 32]],
            &policy,
            EscrowAction::Release,
        );
        // Output goes to buyer, but release requires merchant
        let _result = validate_escrow_auth(
            0,
            &auth,
            &policy.commitment(),
            &[buyer_output()],
            &[0; 32],
            0,
        );
        // This passes policy/signer checks but fails at sig verification
        // (dummy sigs) — so we test output target separately.
    }

    #[test]
    fn output_target_validation_release_to_merchant() {
        let policy = test_policy();
        let SpendPolicy::Escrow2of3 {
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
            ..
        } = &policy
        else {
            unreachable!();
        };

        // Release requires output to merchant
        let result = validate_output_target(
            &EscrowAction::Release,
            &[merchant_output()],
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
        );
        assert!(result.is_ok());

        // Buyer output should fail for release
        let result = validate_output_target(
            &EscrowAction::Release,
            &[buyer_output()],
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
        );
        assert!(matches!(
            result,
            Err(ValidationError::EscrowOutputTargetMismatch)
        ));
    }

    #[test]
    fn output_target_validation_refund_to_buyer() {
        let policy = test_policy();
        let SpendPolicy::Escrow2of3 {
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
            ..
        } = &policy
        else {
            unreachable!();
        };

        let result = validate_output_target(
            &EscrowAction::Refund,
            &[buyer_output()],
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
        );
        assert!(result.is_ok());

        // Merchant output should fail for refund
        let result = validate_output_target(
            &EscrowAction::Refund,
            &[merchant_output()],
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
        );
        assert!(matches!(
            result,
            Err(ValidationError::EscrowOutputTargetMismatch)
        ));
    }

    #[test]
    fn output_target_validation_recovery_accepts_either() {
        let policy = test_policy();
        let SpendPolicy::Escrow2of3 {
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
            ..
        } = &policy
        else {
            unreachable!();
        };

        // Recovery accepts buyer
        let result = validate_output_target(
            &EscrowAction::RecoveryRelease,
            &[buyer_output()],
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
        );
        assert!(result.is_ok());

        // Recovery accepts merchant
        let result = validate_output_target(
            &EscrowAction::RecoveryRelease,
            &[merchant_output()],
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
        );
        assert!(result.is_ok());
    }

    // ── P1: mixed output siphoning test ──────────────────────────────

    #[test]
    fn reject_mixed_outputs_siphoning() {
        // P1 regression: attacker creates tx with one valid output (to merchant)
        // and one siphon output (to attacker's address). ALL outputs must match.
        let policy = test_policy();
        let SpendPolicy::Escrow2of3 {
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
            ..
        } = &policy
        else {
            unreachable!();
        };

        let attacker_commit = SpendPolicy::Single {
            falcon_pk_hash: [0xFF; 32], // unknown attacker key
        }
        .commitment();
        let attacker_output = make_output_with_policy(attacker_commit);

        // Release: merchant_output + attacker_output → must fail
        let result = validate_output_target(
            &EscrowAction::Release,
            &[merchant_output(), attacker_output.clone()],
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
        );
        assert!(
            matches!(result, Err(ValidationError::EscrowOutputTargetMismatch)),
            "mixed outputs (valid + attacker) must be rejected"
        );

        // Refund: buyer_output + attacker_output → must fail
        let result = validate_output_target(
            &EscrowAction::Refund,
            &[buyer_output(), attacker_output],
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
        );
        assert!(
            matches!(result, Err(ValidationError::EscrowOutputTargetMismatch)),
            "mixed outputs (valid + attacker) must be rejected for refund too"
        );
    }

    #[test]
    fn reject_empty_outputs() {
        let policy = test_policy();
        let SpendPolicy::Escrow2of3 {
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
            ..
        } = &policy
        else {
            unreachable!();
        };

        let result = validate_output_target(
            &EscrowAction::Release,
            &[],
            buyer_pk_hash,
            merchant_pk_hash,
            operator_pk_hash,
        );
        assert!(matches!(
            result,
            Err(ValidationError::EscrowOutputTargetMismatch)
        ));
    }
}
