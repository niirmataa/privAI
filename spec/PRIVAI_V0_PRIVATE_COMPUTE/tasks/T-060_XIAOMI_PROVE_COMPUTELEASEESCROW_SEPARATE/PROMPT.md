# T-060-XIAOMI — Prove ComputeLeaseEscrow Must Be Separate

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to PROVE whether `ComputeLeaseEscrow` must be a separate `SpendPolicy` path, instead of:
- extending `Escrow2of3`,
- mutating current escrow validation,
- or introducing a brand new transaction type right now.

This is a proof task, not a brainstorming task.
Use evidence.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-060_XIAOMI_PROVE_COMPUTELEASEESCROW_SEPARATE/OUTPUT_XIAOMI.md
```

## Required Reading

Read:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-057_GEMINI_XIAOMI_OUTPUTS_SCAN/OUTPUT_GEMINI.md
```

You must also read these current code files directly:

```text
privai-chain/src/escrow.rs
privai-ledger/src/escrow.rs
privai-chain/src/tx.rs
privai-chain/src/note.rs
privai-wallet/src/escrow_builder.rs
nxms-escrow-orchestrator/src/operator_escrow.rs
```

Do not use legacy docs.
Do not use Vertex task.

## Claims You Must Resolve

You must explicitly decide whether each claim is:

- `PROVEN`
- `NOT PROVEN`
- `STILL OPEN`

Claims:

1. Extending `Escrow2of3` would create unacceptable compatibility risk.
2. A separate `ComputeLeaseEscrow` path is safer than mutating existing escrow validation.
3. A new transaction variant is unnecessary at this stage.
4. The current codebase already gives enough foundation to justify a separate spend-policy path directionally.

## Evidence Rules

- Every major claim must cite at least one doc and one code location.
- If you rely on current code behavior, label it `code-audited`.
- If you cannot prove something from code or current V0 docs, mark it `STILL OPEN`.
- Do not answer at the level of "seems cleaner"; prove why.

## Main Question

Prove the strongest answer to:

```text
Why is a separate ComputeLeaseEscrow path the safest architecture,
and what exactly can be accepted now without overcommitting implementation?
```

## Required Output Format

Use Polish.

```text
# T-060-XIAOMI — Prove ComputeLeaseEscrow Must Be Separate

## 1. Verdict
## 2. Claims Table

Columns:
Claim | Status (`PROVEN` / `NOT PROVEN` / `STILL OPEN`) | Why

## 3. Code-Level Evidence
## 4. Why Extending Escrow2of3 Fails
## 5. Why A New Transaction Type Is Too Early
## 6. What Can Be Accepted Now
## 7. What Must Stay Open
## 8. Red Lines
## 9. Final Self-Check
```

## Guardrails

- Do not write code.
- Do not define numeric enum tags.
- Do not claim ComputeLeaseEscrow is implemented.
- Distinguish code-audited facts from direction-only conclusions.
