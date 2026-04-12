# T-063-XIAOMI — Prove Operator Decision Boundary

You are Xiaomi acting as a senior architecture reviewer for `privAI V0`.

Your task is to PROVE which decisions really require operator/project-owner authority,
and which items models have been lazily deferring without necessity.

This is a proof task.
No vague lists.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-063_XIAOMI_PROVE_OPERATOR_DECISION_BOUNDARY/OUTPUT_XIAOMI.md
```

## Required Reading

Read:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-049_IMPLEMENTATION_BLOCKERS/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-054_FINAL_REVIEWER_BRIEF/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-057_GEMINI_XIAOMI_OUTPUTS_SCAN/OUTPUT_GEMINI.md
```

You must also audit these current code / architecture touchpoints if needed:

```text
privai-chain/src/params.rs
privai-chain/src/tx.rs
privai-chain/src/escrow.rs
privai-ledger/src/escrow.rs
privai-node/src/config.rs
nxms-escrow-orchestrator/src/operator_escrow.rs
```

Do not use legacy docs.
Do not use Vertex task.

## Claims You Must Resolve

For each decision, classify it as one of:

- `OPERATOR DECISION`
- `DIRECTION/SPEC DECISION`
- `CODE-AUDIT DECISION`
- `WE CAN DECIDE NOW`

At minimum classify:

1. `u64 vs u128`
2. marketplace deprecation timing
3. `ComputeLeaseEscrow` as separate path
4. numeric enum tags
5. hidden-root target model
6. `Amount14` proof-lane-only interpretation
7. Phase 1 kill criteria
8. phase naming cleanup

## Evidence Rules

- Do not just repeat old blocker lists.
- If you call something an operator decision, explain why a model or reviewer cannot settle it from available evidence.
- If you call something decidable now, state the proof basis.

## Main Question

Prove the strongest answer to:

```text
Which decisions truly belong to the operator,
and which ones can already be accepted or rejected without waiting?
```

## Required Output Format

Use Polish.

```text
# T-063-XIAOMI — Prove Operator Decision Boundary

## 1. Verdict
## 2. Decision Classification Table

Columns:
Decision | Classification | Why | What Evidence Supports It

## 3. True Operator Decisions
## 4. Fake Or Avoidable Deferrals
## 5. Decisions We Can Take Now
## 6. Decisions We Still Cannot Take
## 7. Minimal Operator Decision Packet
## 8. Final Self-Check
```

## Guardrails

- Do not defer things to Opus/Claude.
- Be strict: if something is not truly operator-owned, say so.
