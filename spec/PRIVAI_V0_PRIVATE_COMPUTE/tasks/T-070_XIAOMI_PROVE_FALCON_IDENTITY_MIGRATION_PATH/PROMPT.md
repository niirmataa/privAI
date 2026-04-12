# T-070-XIAOMI — Prove Falcon Identity Migration Path

You are Xiaomi acting as a senior architecture and code reviewer for `privAI V0`.

Your task is to PROVE the safest migration path from current code reality:

- Falcon public key hash as practical node identity,
- validator-centric key usage,
- existing escrow signer reality,

toward the target model:

- hidden root credential,
- role keys,
- session / epoch derivation,
- Falcon as signing tool / role-key layer rather than final identity.

This is a proof task.
Not a vague identity essay.
You must design the transition path and prove why it is safe or unsafe.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-070_XIAOMI_PROVE_FALCON_IDENTITY_MIGRATION_PATH/OUTPUT_XIAOMI.md
```

## Required Reading

Read:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-057_GEMINI_XIAOMI_OUTPUTS_SCAN/OUTPUT_GEMINI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-067_XIAOMI_DRAFT_IDENTITY_MODEL_DIRECTION/PROMPT.md
```

You must also read these current code files directly:

```text
privai-node/src/identity_provider.rs
privai-node/src/config.rs
privai-chain/src/escrow.rs
privai-ledger/src/escrow.rs
privai-chain/src/tx.rs
```

Do not use legacy docs.
Do not use Vertex task.

## Claims You Must Resolve

You must explicitly decide whether each claim is:

- `PROVEN`
- `NOT PROVEN`
- `STILL OPEN`

Claims:

1. Current Falcon-key reality can be semantically reinterpreted as role-key reality without breaking current consensus.
2. Hidden root can be introduced additively rather than by replacing current key semantics in one step.
3. Session/epoch key architecture is not safe to freeze yet.
4. The safest near-term decision is semantic reinterpretation first, implementation later.

## Evidence Rules

- Every major claim must cite at least one doc and one code location.
- Separate semantic reinterpretation from actual code migration.
- If a step would break consensus or escrow signer reality, say so explicitly.

## Main Question

Prove the strongest answer to:

```text
How do we safely move from Falcon-key-as-identity reality to the hidden-root target model,
what can be reinterpreted now,
what must remain frozen,
and what must be added later?
```

## Required Output Format

Use Polish.

```text
# T-070-XIAOMI — Prove Falcon Identity Migration Path

## 1. Verdict
## 2. Claims Table

Columns:
Claim | Status (`PROVEN` / `NOT PROVEN` / `STILL OPEN`) | Why

## 3. Current Code Reality
## 4. What Can Be Reinterpreted Now
## 5. What Must Stay Frozen
## 6. Safe Migration Path
## 7. What Must Be Added Later
## 8. What Must Not Be Done
## 9. Minimal Acceptable Canonical Decision Set
## 10. Final Self-Check
```

## Guardrails

- Do not claim hidden root is implemented.
- Do not redefine consensus identity in code reality.
- Do not define final credential wire format.
- Distinguish semantic reinterpretation from code implementation.
- If something still needs code audit, say so explicitly.
