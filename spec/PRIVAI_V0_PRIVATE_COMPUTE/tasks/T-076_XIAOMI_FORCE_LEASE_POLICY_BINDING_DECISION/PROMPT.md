# T-076-XIAOMI — Force Lease Policy Binding Decision

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to FORCE a decision in an area where the project has been too cautious.

This is a decision-forcing task.
You must choose one primary path and defend it.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-076_XIAOMI_FORCE_LEASE_POLICY_BINDING_DECISION/OUTPUT_XIAOMI.md
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
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-060_XIAOMI_PROVE_COMPUTELEASEESCROW_SEPARATE/OUTPUT_XIAOMI.md
privai-chain/src/escrow.rs
privai-ledger/src/escrow.rs
```

Do not use legacy docs.
Do not use Vertex task.

## Decision Arena

At minimum evaluate these options:

- Lease policy binds only off-chain at session start.
- Lease policy binds at escrow lock time via on-chain commitment or hash.
- Lease policy binds only at settlement claim time.
- Lease policy should stay implicit inside offering metadata only.

## Main Question

```text
At what exact point should lease policy become binding for settlement purposes, and why is that point the safest and cleanest architecture choice?
```

## Required Output Format

Use Polish.

```text
# T-076-XIAOMI — Force Lease Policy Binding Decision

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
