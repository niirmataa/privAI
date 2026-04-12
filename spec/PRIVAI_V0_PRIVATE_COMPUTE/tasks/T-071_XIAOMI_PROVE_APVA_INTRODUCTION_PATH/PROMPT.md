# T-071-XIAOMI — Prove aPVA Introduction Path

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to PROVE the safest path for introducing aPVA accounting into the system,
while respecting current code reality and unresolved questions.

You must address:

- current amount reality in code,
- Amount14 proof lane constraints,
- ledger economics,
- future LedgerAmount abstraction,
- aPVA denomination,
- what can be accepted now,
- and what absolutely must remain open.

This is a proof task.
Not a denomination brainstorm.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-071_XIAOMI_PROVE_APVA_INTRODUCTION_PATH/OUTPUT_XIAOMI.md
```

## Required Reading

Read:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_AMOUNT14_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-061_XIAOMI_PROVE_AMOUNT14_LEDGERAMOUNT_BOUNDARY/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-063_XIAOMI_PROVE_OPERATOR_DECISION_BOUNDARY/OUTPUT_XIAOMI.md
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

1. aPVA can be introduced directionally before freezing backing storage type.
2. aPVA denomination must stay separate from `Amount14`.
3. The system needs a ledger-level amount abstraction even before final denomination freeze.
4. `u64 vs u128` is operator-owned only if max supply remains genuinely undecided.

## Evidence Rules

- Every major claim must cite at least one doc and one code location.
- Distinguish denomination, accounting abstraction, and storage type.
- If something is still blocked by max supply, say exactly why.

## Main Question

Prove the strongest answer to:

```text
How should aPVA be introduced safely into privAI,
what does it mean architecturally,
what can be frozen now,
and what remains blocked by max supply or further spec work?
```

## Required Output Format

Use Polish.

```text
# T-071-XIAOMI — Prove aPVA Introduction Path

## 1. Verdict
## 2. Claims Table

Columns:
Claim | Status (`PROVEN` / `NOT PROVEN` / `STILL OPEN`) | Why

## 3. Current Code Amount Reality
## 4. What aPVA Must Mean
## 5. Safe Separation Of Layers
## 6. What Can Be Accepted Now
## 7. What Must Remain Open
## 8. What Must Not Be Done
## 9. Minimal Acceptable Canonical Decision Set
## 10. Final Self-Check
```

## Guardrails

- Do not freeze `u64` vs `u128` unless max supply is known.
- Do not redefine Amount14 as ledger economics.
- Do not define final wire format.
- Distinguish denomination from backing storage type.
- If something depends on operator decision, mark it explicitly.
