# T-061-XIAOMI — Prove Amount14 And LedgerAmount Boundary

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to PROVE the correct architectural separation between:
- `Amount14` as proof/plaintext lane,
- ledger-level economics,
- aPVA denomination,
- and the future `LedgerAmount` abstraction.

This is a proof task, not a brainstorming task.
Use code reality, math, and migration safety.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-061_XIAOMI_PROVE_AMOUNT14_LEDGERAMOUNT_BOUNDARY/OUTPUT_XIAOMI.md
```

## Required Reading

Read:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_AMOUNT14_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-057_GEMINI_XIAOMI_OUTPUTS_SCAN/OUTPUT_GEMINI.md
```

You must also read these current code files directly:

```text
privai-chain/src/params.rs
privai-chain/src/note.rs
privai-chain/src/small_payments.rs
privai-chain/src/tx.rs
privai-wallet/src/escrow_builder.rs
```

Do not use legacy docs.
Do not use Vertex task.

## Claims You Must Resolve

You must explicitly decide whether each claim is:

- `PROVEN`
- `NOT PROVEN`
- `STILL OPEN`

Claims:

1. `Amount14` cannot safely become the economic ledger amount type.
2. The code already shows a split between proof-lane amounts and broader ledger-style numeric amounts.
3. `LedgerAmount` is directionally necessary even if its backing type is not frozen yet.
4. aPVA denomination and backing storage type are different decisions and must stay separate.

## Evidence Rules

- Every major claim must cite at least one doc and one code location.
- If you rely on math, show the math briefly and concretely.
- If you cannot prove something from code or current V0 docs, mark it `STILL OPEN`.

## Main Question

Prove the strongest answer to:

```text
What is the correct and safest separation between Amount14 and ledger economics,
and what exactly can be frozen now versus later?
```

## Required Output Format

Use Polish.

```text
# T-061-XIAOMI — Prove Amount14 And LedgerAmount Boundary

## 1. Verdict
## 2. Claims Table

Columns:
Claim | Status (`PROVEN` / `NOT PROVEN` / `STILL OPEN`) | Why

## 3. Code-Level Evidence
## 4. Mathematical Reality
## 5. What Amount14 Must Never Become
## 6. What LedgerAmount Must Mean
## 7. What Can Be Accepted Now
## 8. What Remains Blocked
## 9. Red Lines
## 10. Final Self-Check
```

## Guardrails

- Do not freeze `u64` vs `u128` unless max supply is known.
- Do not change Amount14 parameters.
- Do not define final wire formats.
- Distinguish audited facts from proposed abstractions.
