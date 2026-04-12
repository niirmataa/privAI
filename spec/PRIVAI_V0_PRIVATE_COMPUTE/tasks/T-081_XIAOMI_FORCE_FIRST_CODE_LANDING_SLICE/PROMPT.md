# T-081-XIAOMI — Force First Code Landing Slice Decision

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to FORCE a decision in an area where the project has been too cautious.

This is a decision-forcing task.
You must choose one primary path and defend it.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-081_XIAOMI_FORCE_FIRST_CODE_LANDING_SLICE/OUTPUT_XIAOMI.md
```

## Decision Discipline

- Choose one primary path.
- You may keep at most one blocker fully open.
- If two options both seem viable, still choose the better one.
- Do not answer with only `it depends`.
- If every listed option is bad, propose a replacement and defend why it beats all listed options.
- You must say what we should do, not only what is risky.

## Required Reading

Read:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-060_XIAOMI_PROVE_COMPUTELEASEESCROW_SEPARATE/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-061_XIAOMI_PROVE_AMOUNT14_LEDGERAMOUNT_BOUNDARY/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-063_XIAOMI_PROVE_OPERATOR_DECISION_BOUNDARY/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-070_XIAOMI_PROVE_FALCON_IDENTITY_MIGRATION_PATH/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-071_XIAOMI_PROVE_APVA_INTRODUCTION_PATH/OUTPUT_XIAOMI.md
```

If needed, read code directly.
Do not use legacy docs.
Do not use Vertex task.

## Decision Arena

At minimum evaluate these options:

- First code slice should be minimal types only.
- First code slice should be ComputeLeaseEscrow path only.
- First code slice should be identity reinterpretation and comments only.
- First code slice should be orchestrator bridge only.
- First code slice should be aPVA or LedgerAmount groundwork only.

## Main Question

```text
If privAI had to land one narrow code slice first, what should it be, and why is that the best leverage point instead of the other options?
```

## Required Output Format

Use Polish.

```text
# T-081-XIAOMI — Force First Code Landing Slice Decision

## 1. Verdict
## 2. Chosen Path
## 3. Rejected Alternatives
## 4. Why This Path Wins Under Current Evidence
## 5. Immediate Consequences
## 6. Risks We Accept
## 7. What Must Still Be Verified
## 8. Minimal Canonical Decision Text
## 9. Final Self-Check
```

## Guardrails

- Do not write code.
- Do not define final wire formats.
- Do not claim unimplemented systems already exist.
- Distinguish code-audited facts from direction-only conclusions.
- If a choice depends on operator authority, say exactly why.
- Prefer a hard choice over an inflated hedge.
