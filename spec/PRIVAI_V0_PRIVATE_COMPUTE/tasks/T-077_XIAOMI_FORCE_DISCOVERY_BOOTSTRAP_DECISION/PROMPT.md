# T-077-XIAOMI — Force Discovery Bootstrap Decision

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to FORCE a decision in an area where the project has been too cautious.

This is a decision-forcing task.
You must choose one primary path and defend it.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-077_XIAOMI_FORCE_DISCOVERY_BOOTSTRAP_DECISION/OUTPUT_XIAOMI.md
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
nxms-mailbox/src/lib.rs
```

Do not use legacy docs.
Do not use Vertex task.

## Decision Arena

At minimum evaluate these options:

- Start with encrypted bootstrap coordinator or registry.
- Start with mailbox-based discovery queries.
- Start with hybrid mailbox plus encrypted bootstrap coordinator.
- Skip bootstrap and go straight to DHT or gossip.

## Main Question

```text
What discovery bootstrap path should privAI choose now, if the goal is to move forward instead of waiting for a perfect decentralized design?
```

## Required Output Format

Use Polish.

```text
# T-077-XIAOMI — Force Discovery Bootstrap Decision

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
