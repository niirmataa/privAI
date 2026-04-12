# T-072-XIAOMI — Force Receipt Trust Decision

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to FORCE a decision in an area where the project has been too cautious.

This is a decision-forcing task.
You must choose one primary path and defend it.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-072_XIAOMI_FORCE_RECEIPT_TRUST_DECISION/OUTPUT_XIAOMI.md
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
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-065_XIAOMI_DRAFT_OPERATORLESS_ESCROW_DIRECTION/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-066_XIAOMI_DRAFT_METERING_PROTOCOL_DIRECTION/OUTPUT_XIAOMI.md
nxms-escrow-orchestrator/src/operator_escrow.rs
privai-chain/src/small_payments.rs
```

Do not use legacy docs.
Do not use Vertex task.

## Decision Arena

At minimum evaluate these options:

- Self-reported receipts are enough even for protocol-level settlement.
- Self-reported receipts are enough only for Phase 1 bridge; Phase 2 must require challenge/response.
- Challenge/response must already be required in Phase 1.
- Trusted attestation or TEE must be baseline before any serious settlement path.

## Main Question

```text
Which receipt trust model should privAI actually choose now as the working direction, and what exactly should be deferred to later phases?
```

## Required Output Format

Use Polish.

```text
# T-072-XIAOMI — Force Receipt Trust Decision

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
