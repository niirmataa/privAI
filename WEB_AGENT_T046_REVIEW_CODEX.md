# T-046 Codex Review — Decision Matrix Draft

**Status:** reviewed
**Reviewed by:** Codex
**Date:** 2026-04-11
**Reviewed output:** `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/OUTPUT_XIAOMI.md`

---

## Verdict

T-046 is a strong first decision matrix. It correctly separates frozen direction, strong candidates, operator decisions, missing spec/reviewer decisions, and code-audit findings. It is good enough to use as input for the next review step.

It should not be promoted directly to canonical docs yet. Several rows are directionally right but too strong in wording, especially where a code type, numeric tag, or `u64` backing choice is presented as "add now" before the relevant freeze/spec decision.

---

## What Is Strong

1. **Bridge vs target is handled correctly.** `Escrow2of3` stays as Phase 0/1 bridge, while operatorless settlement remains target/future.
2. **RecoveryRelease is correctly treated as the operatorless anchor.** This is code-confirmed and useful as a migration anchor.
3. **Amount14 is no longer treated as a fatal blocker.** The audit-backed distinction between proof/plaintext lane and economic amount lane is the right mental model.
4. **Marketplace contamination is called out without proposing deletion.** Deprecation/legacy isolation is safer than removal.
5. **Ownership is useful.** Operator, model reviewers, and code-audit responsibilities are separated instead of blended.
6. **V0-only MCP/RAG remains frozen.** This protects the whole multi-agent workflow from legacy drift.

---

## Corrections Before Canonical Promotion

0. **Opus must not be treated as a blocking authority.** Where Xiaomi says "Opus decisions", the corrected status is "missing direction/spec/reviewer decision." Opus can review later if available, but the workflow must not wait for it.
1. **`LedgerAmount = u64` should not be frozen as final until max supply is decided.** Safer wording: `LedgerAmount` is required; backing type is blocked by max supply (`u64` strong candidate, `u128` fallback).
2. **Numeric tags should stay proposed, not frozen.** `SpendPolicyTag::ComputeLeaseEscrow = 0x04`, `EscrowAction::ProRataSplit = 0x04`, and `TimeoutAutoRefund = 0x05` need protocol/version registry confirmation before being final.
3. **"Decyzje do dodania teraz (14 typow)" is too implementation-forward.** Better: "first candidate type surface after direction/spec gate." We can design them now, but code addition should wait for the relevant decision/spec.
4. **`HiddenRootCredential` appears in both "add now" and "additive later" logic.** Keep the concept in the final model, but do not add root credential code until Identity Direction / Credential Schema is ready.
5. **`ComputeLeaseReceipt` fields must remain direction-level.** The matrix should say that the receipt concept is a strong candidate, not that the schema is frozen.
6. **VM default privacy class needs runtime privacy direction.** The instinct is correct, but "default VM" can overclaim unless miner visibility and cost tradeoffs are defined.
7. **Marketplace deprecation needs operator approval.** It is a safe cleanup candidate, not an automatic code task.
8. **`ComputeLeaseEscrow` is the strongest technical path, but its fields are not final.** The decision can freeze "new SpendPolicy variant, separate validation"; not the exact struct fields yet.

---

## Recommended Status Adjustments

| Topic | Xiaomi Status | Codex Adjustment |
|---|---:|---|
| V0 private compute model | FROZEN_CANDIDATE | Keep |
| Escrow2of3 bridge | FROZEN_CANDIDATE | Keep |
| RecoveryRelease anchor | FROZEN_CANDIDATE | Keep |
| Amount14 proof lane only | STRONG_CANDIDATE | Promote toward freeze after final review |
| LedgerAmount u64 | STRONG_CANDIDATE | Split into `LedgerAmount required` and `u64 vs u128 blocked` |
| ComputeLeaseEscrow tag `0x04` | STRONG_CANDIDATE | Keep concept; make numeric tag proposed |
| Marketplace deprecation | BLOCKED_BY_OPERATOR | Keep |
| Falcon PK as ValidatorRoleKey | STRONG_CANDIDATE | Keep as semantic/docs/comment change |
| HiddenRootCredential code | CANDIDATE / add now | Delay code; keep model |
| ProRataSplit action | CANDIDATE | Keep candidate; no numeric freeze |
| VM default | CANDIDATE | Keep candidate pending Runtime Privacy Direction |

---

## Next Step

Run an independent Gemini review before drafting any final canonical decision document. Gemini should specifically check:

1. whether Xiaomi over-promoted any `STRONG_CANDIDATE` into practical freeze,
2. whether any "add now" type should stay design-only,
3. whether the matrix misses blockers from the audits,
4. whether any decision violates the V0 no-legacy/no-marketplace guardrails.

After Gemini review, the best sequence is:

1. T-047 Domain Boundaries Freeze
2. T-048 Minimal Types Freeze
3. T-049 Implementation Blockers
4. Draft `PRIVAI_V0_FINAL_DOMAIN_AND_MIGRATION_DECISIONS_PL.md`

---

## Promotion Recommendation

Do not promote T-046 directly.

Use it as input to a final decision doc only after cross-review and after status wording is tightened from "add now" to "candidate type surface / gated implementation."
