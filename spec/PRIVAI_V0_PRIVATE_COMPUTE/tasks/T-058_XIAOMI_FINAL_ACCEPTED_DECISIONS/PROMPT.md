# T-058-XIAOMI — Final Accepted Decisions Draft

You are Xiaomi acting as a senior architecture reviewer for `privAI V0`.

Your task is to produce the strongest possible final candidate decision document for `privAI V0`.

This is not a brainstorming task.
This is not an implementation task.
This is not a code-writing task.

You must decide, as cleanly as possible:

- what should be accepted now,
- what should be accepted only directionally,
- what should be deferred,
- what should be rejected,
- what still requires operator decision,
- what still requires direction/spec docs,
- what still requires code audit.

Your output will be used as candidate input for a canonical V0 decision document.

---

## Output Path

Write your answer to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-058_XIAOMI_FINAL_ACCEPTED_DECISIONS/OUTPUT_XIAOMI.md
```

---

## Process Rules

- No model is an oracle.
- Opus/Claude is not a blocking authority.
- If older Xiaomi text says `blocked by Opus`, normalize it to:
  - `blocked by operator decision`,
  - `blocked by missing direction/spec`, or
  - `blocked by code audit`.
- Raw task outputs are non-canonical.
- Canonical V0 docs live in:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/
```

- Task outputs live in:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/
```

- Do not analyze the Vertex task.

---

## Required Reading

Read first:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/README.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-057_GEMINI_XIAOMI_OUTPUTS_SCAN/OUTPUT_GEMINI.md
```

Read these saved Xiaomi audit docs:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOMAIN_MODEL_CLASSIFICATION_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_AMOUNT14_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MARKETPLACE_TYPES_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md
```

Read these task outputs:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-047_DOMAIN_BOUNDARIES_FREEZE/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-048_MINIMAL_TYPES_FREEZE/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-049_IMPLEMENTATION_BLOCKERS/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-050_DOCS_DEPENDENCY_GRAPH/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-051_MCP_RAG_GOLDEN_QUESTIONS/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-054_FINAL_REVIEWER_BRIEF/OUTPUT_XIAOMI.md
```

Ground yourself in these canonical docs:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md
```

Do not use:

- legacy docs,
- old marketplace docs,
- T-056 Vertex task,
- old root logs as source of truth.

If implementation truth is needed, read current code directly and label it explicitly as code-audited.

---

## Main Question

Write the best final candidate answer to:

```text
What do we actually accept now as the V0 final domain-and-migration decision set,
and what explicitly remains open?
```

---

## Required Output Format

Use Polish.

Structure the output exactly like this:

```text
# T-058-XIAOMI — Final Accepted Decisions Draft

## 1. Verdict

## 2. Accepted Now

Flat bullets only.
Each bullet must be a real decision, not a vague theme.

## 3. Accepted Directionally, But Not Frozen In Code

## 4. Deferred / Not Accepted Yet

## 5. Rejected Or Explicitly Not Allowed

## 6. Required Operator Decisions

## 7. Required Direction / Spec Documents

## 8. Required Code Audits

## 9. Canonical Doc Outline Proposal

Include proposed section titles for:
`PRIVAI_V0_FINAL_DOMAIN_AND_MIGRATION_DECISIONS_PL.md`

## 10. Red Lines

## 11. Final Self-Check
```

---

## Guardrails

- Do not write code.
- Do not edit canonical docs.
- Do not define final wire formats.
- Do not freeze `u64` vs `u128` unless max supply is known.
- Do not freeze numeric enum tags.
- Do not claim operatorless escrow is implemented.
- Do not claim pro-rata is implemented.
- Do not claim hidden-root identity is implemented.
- Do not claim private discovery is implemented.
- Do not treat Xiaomi's older outputs as automatically correct.
- Keep this as a final candidate decision draft, not a speculative roadmap.
