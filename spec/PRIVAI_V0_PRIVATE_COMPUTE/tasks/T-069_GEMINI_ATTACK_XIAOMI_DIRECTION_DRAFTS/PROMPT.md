# T-069-GEMINI — Attack Xiaomi Direction Drafts

You are Gemini 3.1 Pro acting as an adversarial direction reviewer for `privAI V0`.

Your task is to ATTACK Xiaomi's direction drafts and identify:
- overreach,
- implementation leakage,
- unjustified assumptions,
- fake certainty,
- and the minimal subset that can actually survive promotion into canonical docs.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-069_GEMINI_ATTACK_XIAOMI_DIRECTION_DRAFTS/OUTPUT_GEMINI.md
```

## Required Reading

Read:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-065_XIAOMI_DRAFT_OPERATORLESS_ESCROW_DIRECTION/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-066_XIAOMI_DRAFT_METERING_PROTOCOL_DIRECTION/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-067_XIAOMI_DRAFT_IDENTITY_MODEL_DIRECTION/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md
```

Use code audit only when needed.
Do not use legacy docs.
Do not use Vertex task.

## Main Question

Attack the strongest answer to:

```text
What in Xiaomi's direction drafts is safe enough for canonical V0 docs,
and what still leaks implementation, speculation, or unsupported certainty?
```

## Required Output Format

Use Polish.

```text
# T-069-GEMINI — Attack Xiaomi Direction Drafts

## 1. Verdict
## 2. Safe Direction Material
## 3. Overreach
## 4. Implementation Leakage
## 5. Unsupported Assumptions
## 6. Minimal Surviving Canonical Set
## 7. What Must Stay Open
## 8. Final Self-Check
```

## Guardrails

- Do not define final wire formats.
- Do not claim implementation exists where it does not.
- Prefer a smaller surviving set over verbose approval.
