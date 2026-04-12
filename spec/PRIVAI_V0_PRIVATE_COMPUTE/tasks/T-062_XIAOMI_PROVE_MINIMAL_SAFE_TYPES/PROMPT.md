# T-062-XIAOMI — Prove Minimal Safe Types

You are Xiaomi acting as a senior architecture reviewer for `privAI V0`.

Your task is to PROVE which minimal domain types can be added safely without creating future refactor debt.

This is not "list all nice types".
This is a proof task about survivable, additive, low-regret types only.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-062_XIAOMI_PROVE_MINIMAL_SAFE_TYPES/OUTPUT_XIAOMI.md
```

## Required Reading

Read:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-048_MINIMAL_TYPES_FREEZE/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-057_GEMINI_XIAOMI_OUTPUTS_SCAN/OUTPUT_GEMINI.md
```

Use code audit only if needed.
Do not use legacy docs.
Do not use Vertex task.

## Main Question

Prove the strongest answer to:

```text
Which minimal types are safe enough to survive canonization,
and which types are still too early even if they look directionally correct?
```

## Required Output Format

Use Polish.

```text
# T-062-XIAOMI — Prove Minimal Safe Types

## 1. Verdict
## 2. Safe Now
## 3. Safe Directionally Only
## 4. Too Early
## 5. Why Each Early Type Is Still Early
## 6. Minimal Safe Set Proposal
## 7. Red Lines
## 8. Final Self-Check
```

## Guardrails

- Do not freeze numeric tags.
- Do not freeze field lists unless backed by spec.
- Prefer fewer types over inflated certainty.
