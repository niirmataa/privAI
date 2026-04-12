# T-057-GEMINI — Xiaomi Outputs Consistency Scan

You are Gemini 3.1 Pro acting as an independent architecture reviewer for `privAI V0`.

Your task is to scan the Xiaomi task outputs collected so far and produce a review-level synthesis:

- what Xiaomi got consistently right,
- where Xiaomi outputs contradict each other,
- where Xiaomi output is too implementation-forward,
- where Xiaomi still depends on unavailable or obsolete assumptions,
- which ideas are safe input for a canonical decision document,
- which ideas need Codex review, operator decision, spec work, or code audit first.

This is a review task, not an implementation task.

Do not create code.
Do not edit docs.
Do not promote raw model output to canonical architecture.

---

## Output Path

Write your answer to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-057_GEMINI_XIAOMI_OUTPUTS_SCAN/OUTPUT_GEMINI.md
```

---

## Process Context

Important process rules:

- Xiaomi, Gemini, Claude/Opus, and Codex are reviewer/worker inputs, not oracles.
- Opus/Claude is not a blocking dependency.
- If any Xiaomi output says "Opus decides", reinterpret that as "missing direction/spec/reviewer decision" or "operator decision needed".
- Raw task outputs are non-canonical.
- Canonical V0 docs live in:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/
```

- Task outputs live in:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/
```

- The Vertex task is out of scope for this review.

---

## Required Reading

Read the task workspace policy first:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/README.md
```

Read these saved Xiaomi pre-task audit outputs:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOMAIN_MODEL_CLASSIFICATION_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_AMOUNT14_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MARKETPLACE_TYPES_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md
```

Read these Xiaomi task outputs:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-047_DOMAIN_BOUNDARIES_FREEZE/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-048_MINIMAL_TYPES_FREEZE/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-049_IMPLEMENTATION_BLOCKERS/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-050_DOCS_DEPENDENCY_GRAPH/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-051_MCP_RAG_GOLDEN_QUESTIONS/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-054_FINAL_REVIEWER_BRIEF/OUTPUT_XIAOMI.md
```

Read these review artifacts for calibration:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/REVIEW_CODEX.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-053_GEMINI_DECISION_MATRIX_REVIEW/OUTPUT_GEMINI.md
```

Optional, superseded-only reading:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-052_PRE_OPUS_BRIEF/OUTPUT_XIAOMI.md
```

Use T-052 only to detect old phrasing or obsolete assumptions.
Do not treat T-052 as current guidance because it is superseded by T-054.

Use these canonical V0 docs as grounding when needed:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SINGLE_SOURCE_OF_TRUTH_CONTEXT_PLAN.md
```

Do not read:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-056_VERTEX_RAG_BRIDGE_DESIGN/
```

Do not use legacy docs.
Do not use old marketplace docs.
Do not use uploaded copies if the real repo files exist.

If implementation truth is needed, read current code directly and explicitly label it as code-audited.
Do not infer implementation facts from direction docs.

---

## Review Questions

Answer these directly:

1. Across all Xiaomi outputs, what conclusions are stable and repeated?
2. Which Xiaomi conclusions are contradicted by other Xiaomi outputs?
3. Which Xiaomi conclusions are contradicted by Codex or Gemini review artifacts?
4. Which conclusions are safe inputs for a canonical V0 decision doc?
5. Which conclusions are only candidates, not frozen?
6. Which conclusions require operator decision?
7. Which conclusions require missing direction/spec docs?
8. Which conclusions require code audit?
9. Which conclusions are too implementation-forward for current V0 status?
10. Which outputs still contain "Opus as gatekeeper" language and how should that be normalized?
11. Which outputs refer to task numbers or artifacts that do not exist or are superseded?
12. Does any Xiaomi output accidentally reintroduce marketplace framing?
13. Does any Xiaomi output overclaim current code reality, especially around operatorless escrow, pro-rata, receipt truth, hidden root identity, discovery, or MCP/RAG?
14. What should Codex review next?
15. What should be the next 5 task prompts after this scan?

---

## Required Output Format

Use Polish.

Structure the output exactly like this:

```text
# T-057-GEMINI — Xiaomi Outputs Consistency Scan

## 1. Verdict

## 2. Stable Xiaomi Conclusions

Table columns:
Conclusion | Appears In | Gemini Status | Notes

## 3. Contradictions Or Tensions

Table columns:
Topic | Xiaomi Output A | Xiaomi Output B | Gemini Resolution

## 4. Corrections From Codex/Gemini Reviews

Table columns:
Topic | Xiaomi Claim | Review Correction | Status After Correction

## 5. Freeze-Ready Inputs For Canonical Docs

## 6. Strong Candidates, Not Frozen

## 7. Blocked Items

Separate:
- Blocked by Operator Decision
- Blocked by Direction/Spec/Reviewer Decision
- Blocked by Code Audit

## 8. Too Implementation-Forward

## 9. Obsolete Or Superseded Language

Include:
- Opus-as-gatekeeper phrasing
- references to non-existent tasks
- references to superseded outputs

## 10. Legacy Marketplace Drift Check

## 11. Recommended Codex Review Queue

## 12. Recommended Next 5 Task Prompts

For each prompt:
Task ID suggestion | Target model | Purpose | Required input files | Expected output path

## 13. Final Self-Check
```

---

## Guardrails

- Do not edit code.
- Do not edit canonical V0 docs.
- Do not define final wire formats.
- Do not treat Xiaomi output as canonical.
- Do not treat Gemini output as canonical.
- Do not treat Opus/Claude as a required gate or blocking authority.
- Do not use legacy docs.
- Do not read or analyze the Vertex task folder.
- Do not claim operatorless escrow is implemented.
- Do not claim pro-rata split is implemented.
- Do not claim receipt truth is solved.
- Do not claim hidden root identity is implemented.
- Do not claim private discovery is implemented.
- Do not freeze `u64` vs `u128` unless max PVA supply is known.
- Do not freeze numeric enum tags unless the relevant protocol/versioning spec exists.
- Distinguish clearly between:
  - canonical V0 docs,
  - saved audit docs,
  - raw Xiaomi outputs,
  - Codex reviews,
  - Gemini reviews,
  - current code reality.
- Keep the answer review-level, not implementation-level.
