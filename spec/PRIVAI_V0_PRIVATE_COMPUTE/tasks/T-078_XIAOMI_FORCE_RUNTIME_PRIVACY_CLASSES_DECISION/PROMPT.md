# T-078-XIAOMI — Force Runtime Privacy Classes Decision

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to FORCE a decision in an area where the project has been too cautious.

This is a decision-forcing task.
You must choose one primary path and defend it.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-078_XIAOMI_FORCE_RUNTIME_PRIVACY_CLASSES_DECISION/OUTPUT_XIAOMI.md
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
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-057_GEMINI_XIAOMI_OUTPUTS_SCAN/OUTPUT_GEMINI.md
```

Do not use legacy docs.
Do not use Vertex task.

## Decision Arena

At minimum evaluate these options:

- Freeze 4 classes now: VM, Container, Sandbox, ConfidentialRuntime.
- Freeze only 2 classes now: VM and LowerPrivacyRuntime, refine later.
- Freeze 3 classes now: VM, Container, Sandbox, defer confidential runtime.
- Do not freeze any runtime privacy classes yet.

## Main Question

```text
What runtime privacy class strategy should privAI choose right now so users get meaningful guarantees without fake precision?
```

## Required Output Format

Use Polish.

```text
# T-078-XIAOMI — Force Runtime Privacy Classes Decision

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
