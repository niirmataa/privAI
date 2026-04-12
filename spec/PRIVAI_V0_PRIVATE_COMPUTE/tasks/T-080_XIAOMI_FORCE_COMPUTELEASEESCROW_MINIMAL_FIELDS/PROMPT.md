# T-080-XIAOMI — Force ComputeLeaseEscrow Minimal Fields Decision

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to FORCE a decision in an area where the project has been too cautious.

This is a decision-forcing task.
You must choose one primary path and defend it.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-080_XIAOMI_FORCE_COMPUTELEASEESCROW_MINIMAL_FIELDS/OUTPUT_XIAOMI.md
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
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-076_XIAOMI_FORCE_LEASE_POLICY_BINDING_DECISION/OUTPUT_XIAOMI.md
privai-chain/src/escrow.rs
privai-ledger/src/escrow.rs
```

Do not use legacy docs.
Do not use Vertex task.

## Decision Arena

At minimum evaluate these options:

- Define a very small minimal field set now and defer everything else.
- Define a rich field set now so future docs do not need revisiting.
- Keep ComputeLeaseEscrow field shape fully open until complete protocol specs exist.
- Bind almost everything through one policy commitment and keep policy structure off the spend policy itself.

## Main Question

```text
What is the minimal but honest ComputeLeaseEscrow field strategy we should choose now, if we want progress without locking fake details too early?
```

## Required Output Format

Use Polish.

```text
# T-080-XIAOMI — Force ComputeLeaseEscrow Minimal Fields Decision

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
