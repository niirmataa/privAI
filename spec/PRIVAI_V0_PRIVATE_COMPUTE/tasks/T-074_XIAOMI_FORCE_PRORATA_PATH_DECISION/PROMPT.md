# T-074-XIAOMI — Force Pro-Rata Path Decision

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to FORCE a decision in an area where the project has been too cautious.

This is a decision-forcing task.
You must choose one primary path and defend it.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-074_XIAOMI_FORCE_PRORATA_PATH_DECISION/OUTPUT_XIAOMI.md
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
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md
privai-chain/src/escrow.rs
privai-ledger/src/escrow.rs
privai-chain/src/note.rs
```

Do not use legacy docs.
Do not use Vertex task.

## Decision Arena

At minimum evaluate these options:

- No pro-rata before a full new note split mechanism exists.
- Phase 1 should use an ugly Release plus Refund bridge, then Phase 2 gets proper split mechanics.
- We should force proper pro-rata note split design immediately and refuse any bridge.
- Timed compute should avoid pro-rata entirely and stay all-or-nothing.

## Main Question

```text
What pro-rata path should privAI commit to, given current escrow code reality and the need to move forward without fantasy implementations?
```

## Required Output Format

Use Polish.

```text
# T-074-XIAOMI — Force Pro-Rata Path Decision

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
