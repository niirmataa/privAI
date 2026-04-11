# T-053-GEMINI — Decision Matrix Cross-Review

You are Gemini 3.1 Pro acting as an independent architecture reviewer for `privAI V0`.

Your task is to review Xiaomi's first decision matrix draft and identify:

- decisions that are safe to freeze,
- decisions that are only strong candidates,
- decisions that are blocked by operator/spec/code audit,
- overclaims or premature implementation commitments,
- missing blockers or missing decisions.

This is a review task, not an implementation task.

Important process update:

- Opus is not available and must not be treated as a blocking authority.
- If Xiaomi says "Opus decision", reinterpret that as "missing direction/spec/reviewer decision".
- Gemini is being tested as an independent reviewer, not as a replacement oracle.

---

## Output Path

Write your answer to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-053_GEMINI_DECISION_MATRIX_REVIEW/OUTPUT_GEMINI.md
```

---

## Required Reading

Read first:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/REVIEW_CODEX.md
```

Then use these source docs as needed:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_AMOUNT14_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MARKETPLACE_TYPES_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MIGRATION_ARCHITECTURE_PL.md
```

Do not use legacy docs.

If you need implementation truth, read current code directly. Do not infer code behavior from legacy docs.

---

## Review Questions

Answer these directly:

1. Is Xiaomi's decision matrix safe to use as input for a canonical final decision doc?
2. Which decisions are truly freeze-ready?
3. Which decisions are only strong candidates?
4. Which decisions are blocked by operator decisions?
5. Which decisions are blocked by missing direction/spec docs?
6. Which decisions are blocked by code audit?
7. Which recommendations are too implementation-forward?
8. Which numeric constants, enum tags, or field lists should remain proposed instead of frozen?
9. Does the matrix accidentally reintroduce legacy marketplace framing anywhere?
10. What should Codex do next after this review?

---

## Required Output Format

Use Polish.

Structure the output exactly like this:

```text
# T-053-GEMINI — Decision Matrix Cross-Review

## 1. Verdict

## 2. Status Corrections Table

Columns:
Decision | Xiaomi Status | Gemini Status | Why

## 3. Freeze-Ready Decisions

## 4. Strong Candidates, Not Frozen

## 5. Blocked Decisions

Separate:
- Blocked by Operator
- Blocked by direction/spec/reviewer decision
- Blocked by Code Audit

## 6. Premature Implementation Commitments

## 7. Missing Decisions Or Missing Blockers

## 8. Legacy / Marketplace Drift Check

## 9. Recommended Next Task Order

## 10. Final Self-Check
```

---

## Guardrails

- Do not edit code.
- Do not edit canonical V0 docs.
- Do not define final wire formats.
- Do not treat Xiaomi's output as canonical.
- Do not treat Opus as a required gate or blocking authority.
- Do not use legacy docs.
- Do not claim operatorless escrow is implemented.
- Do not claim pro-rata split is implemented.
- Do not freeze `u64` vs `u128` unless max PVA supply is known.
- Do not freeze numeric enum tags unless the relevant protocol/versioning spec exists.
- Keep the answer review-level, not implementation-level.
