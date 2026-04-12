# T-079-XIAOMI — Force Node Role Implementation Boundary

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to FORCE a decision in an area where the project has been too cautious.

This is a decision-forcing task.
You must choose one primary path and defend it.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-079_XIAOMI_FORCE_NODE_ROLE_IMPLEMENTATION_BOUNDARY/OUTPUT_XIAOMI.md
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
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIAGRAMS.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-067_XIAOMI_DRAFT_IDENTITY_MODEL_DIRECTION/OUTPUT_XIAOMI.md
privai-node/src/node.rs
privai-node/src/config.rs
```

Do not use legacy docs.
Do not use Vertex task.

## Decision Arena

At minimum evaluate these options:

- Implement five separate roles in code immediately.
- Freeze five roles directionally, but keep code mostly validator-centric for now.
- Merge some roles permanently because the codebase is simpler that way.
- Delay role separation entirely until after settlement work.

## Main Question

```text
What node-role boundary should privAI commit to now, so the architecture stays honest without forcing premature code explosion?
```

## Required Output Format

Use Polish.

```text
# T-079-XIAOMI — Force Node Role Implementation Boundary

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
