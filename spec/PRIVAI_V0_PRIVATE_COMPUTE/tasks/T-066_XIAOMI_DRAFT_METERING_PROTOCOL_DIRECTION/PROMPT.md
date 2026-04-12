# T-066-XIAOMI — Draft Metering Protocol Direction

You are Xiaomi acting as a senior architecture reviewer for `privAI V0`.

Your task is to write a strong direction-level draft for `Metering Protocol Direction`.

This is a direction doc draft.
It must define the decision space without collapsing into wire format.

## Output Path

Write to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-066_XIAOMI_DRAFT_METERING_PROTOCOL_DIRECTION/OUTPUT_XIAOMI.md
```

## Required Reading

Read:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-061_XIAOMI_PROVE_AMOUNT14_LEDGERAMOUNT_BOUNDARY/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-065_XIAOMI_DRAFT_OPERATORLESS_ESCROW_DIRECTION/OUTPUT_XIAOMI.md
```

If needed, read code directly.
Do not use legacy docs.
Do not use Vertex task.

## Main Question

Write the strongest direction-level answer to:

```text
What must metering do in privAI V0,
what counts as receipt evidence,
and what trust model is minimally acceptable before protocol-level settlement?
```

## Required Output Format

Use Polish.

```text
# T-066-XIAOMI — Draft Metering Protocol Direction

## 1. Verdict
## 2. What Metering Exists To Prove
## 3. Receipt Evidence Direction
## 4. Receipt Availability Direction
## 5. Trust Model Options
## 6. Minimum Acceptable Bridge Model
## 7. What Must Stay Open
## 8. Red Lines
## 9. Final Self-Check
```

## Guardrails

- Do not define final receipt fields as frozen wire format.
- Do not claim receipt truth is solved.
- Do not assume self-reporting is enough for protocol-level trust unless you justify it and label it as open.
