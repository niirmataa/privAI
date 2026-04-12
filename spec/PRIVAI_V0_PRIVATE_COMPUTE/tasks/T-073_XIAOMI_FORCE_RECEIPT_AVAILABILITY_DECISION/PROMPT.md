# T-073-XIAOMI — Force Receipt Availability Decision

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to FORCE a decision in an area where the project has been too cautious.

This is a decision-forcing task.
You must choose one primary path and defend it.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-073_XIAOMI_FORCE_RECEIPT_AVAILABILITY_DECISION/OUTPUT_XIAOMI.md
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
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-066_XIAOMI_DRAFT_METERING_PROTOCOL_DIRECTION/OUTPUT_XIAOMI.md
nxms-mailbox/src/lib.rs
nxms-escrow-orchestrator/src/operator_escrow.rs
```

Do not use legacy docs.
Do not use Vertex task.

## Decision Arena

At minimum evaluate these options:

- Only miner stores receipts.
- Only user stores receipts.
- Both parties store receipts, with commitment or hash binding at settlement time.
- Mailbox or third-party service stores canonical receipts.
- Full receipts should go on-chain.

## Main Question

```text
What receipt availability architecture should privAI choose as the default working direction, and why is it stronger than the alternatives?
```

## Required Output Format

Use Polish.

```text
# T-073-XIAOMI — Force Receipt Availability Decision

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
