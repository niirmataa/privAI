# TASK 031 OPUS — Working Context For New Chat

**Date:** 2026-04-11
**Repo:** `/home/nxms-server/privAI`
**Folder:** `spec/PRIVAI_V0_PRIVATE_COMPUTE`
**Status:** working handoff for next chat window

---

## 1. Current Project Stage

We are in the V0 private compute reset.

The product/business model has been reset from:

```text
AI marketplace
```

to:

```text
post-quantum FullPrivacy private AI compute network
```

Canonical V0 rule:

```text
Privacy is the product.
Compute is the supply.
PVA is the incentive.
Chain is the settlement.
Transport is the shield.
```

This is not a cosmetic terminology change. It supersedes old marketplace/product framing.

---

## 2. Current Canonical Docs

Read first:

1. `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md`
2. `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_LEGACY_DOCSET_REWRITE_PLAN.md`
3. `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIAGRAMS.md`

Important rule:

```text
V0 supersedes product/business marketplace framing.
V0 does NOT supersede deep-spec mechanical truth.
```

Do not invent implementation truth from V0 direction.

---

## 3. Recent Completed Tasks

### P-T023-OPUS

Created:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_LEGACY_DOCSET_REWRITE_PLAN.md
```

Result:

- V0 answers all 6 original architecture gaps at direction level.
- V0 does not yet close protocol specs.
- Legacy docs were classified into:
  - canonical,
  - still valid mechanical reference,
  - partially superseded,
  - needs rewrite,
  - historical/archive.

### T-030-OPUS

Created:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIAGRAMS.md
```

Result:

- 15 Mermaid diagrams.
- 14 canonical V0 direction diagrams.
- 1 future strengthening diagram: Operatorless Escrow Transition.
- Legacy concepts explicitly rejected:
  - public AI marketplace,
  - public provider profile,
  - skill pack registry on-chain,
  - quality-of-answer settlement,
  - operator as canonical escrow decision-maker,
  - MarketplaceBatchTx as product center,
  - public reputation leaderboard,
  - artifact delivery as center of settlement.

---

## 4. Current Active Task

The next active task is:

```text
T-031-OPUS — Compute Lease Settlement Direction
```

Target new file:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md
```

Goal:

Create a V0 direction doc replacing old contract/quality/artifact settlement framing with:

```text
compute lease policy
metering receipts
resource delivery
timeout rules
pro-rata settlement direction
operatorless target
```

This task must not define wire formats.

This task must not claim pro-rata settlement is implemented.

This task must not claim operatorless escrow is implemented.

This task must not touch legacy docs or code.

---

## 5. T-031 Scope

Write scope:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md
```

Read-only source of truth:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_LEGACY_DOCSET_REWRITE_PLAN.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIAGRAMS.md
spec/privAI_handoff_2026-04-09/PRIVAI_CONTRACT_VERIFICATION_AND_SETTLEMENT_DIRECTION.md
spec/privAI_handoff_2026-04-09/PRIVAI_STAGE_A_STAGE_B_CONTRACT_FREEZE.md
spec/privAI_handoff_2026-04-09/PRIVAI_HALO2_PROOF_BOUNDARY_FREEZE.md
```

Do not read the full repo.

Do not grep broadly through code.

Do not change code.

---

## 6. T-031 Required Sections

The new T-031 doc should contain:

1. **Core Rule**
   - settlement is based on resource availability and metered session delivery, not subjective AI output quality.

2. **What Replaces Contract/Quality Settlement**
   - old vs V0 mapping:
     - task contract -> compute lease policy,
     - artifact delivery -> private compute session,
     - proof of delivery -> metering/session receipts,
     - semantic quality review -> not part of settlement,
     - provider -> compute miner,
     - buyer -> user / compute lessee.

3. **Compute Lease Policy**
   - direction-level fields only:
     - resource class,
     - runtime privacy class,
     - network mode,
     - lease duration / quota,
     - price in aPVA,
     - timeout,
     - settlement formula,
     - failure rules,
     - receipt requirements,
     - optional collateral/bond rule.

4. **Settlement Outcomes**
   - full release,
   - full refund,
   - pro-rata split,
   - penalty/slash direction,
   - recovery after timeout,
   - no-quality-dispute rule.

5. **Receipt / Metering Evidence**
   - direction-level only:
     - session started,
     - resource allocated,
     - heartbeat/challenge observed,
     - duration delivered,
     - resource class matched,
     - miner signed,
     - user/session ack optional.

6. **Operator Boundary**
   - operatorless by design.
   - operator-assisted paths are bootstrap/compatibility only.
   - current legacy escrow may still require operator today.

7. **Interaction With Existing Escrow Mechanics**
   - Stage A/B freeze remains mechanical truth.
   - current 2-of-3 release/refund/recovery remains current code reality.
   - V0 settlement direction does not claim pro-rata/operatorless is implemented.
   - future implementation must bridge current escrow to compute lease receipts.

8. **Privacy Boundary**
   - chain does not see prompt, workload, output, model interaction, public marketplace semantics, public provider profile.
   - chain may see commitments, nullifiers, encrypted/committed amount, escrow policy commitment, timeout, proof/statement commitments, settlement authorization.

9. **Non-Goals / Do Not Infer**
   - no subjective AI quality settlement,
   - no human marketplace dispute,
   - no public artifact delivery baseline,
   - no operator as canonical decision-maker,
   - no claim current code implements pro-rata,
   - no claim metering receipt schema is frozen,
   - no proof completeness overclaim.

10. **Open Protocol Follow-Ups**
   - exact lease object model,
   - exact metering receipt schema,
   - exact settlement formula,
   - pro-rata note/output mechanics,
   - timeout claim path,
   - operatorless escrow transition,
   - collateral/slash rules.

---

## 7. Strict Guardrails

Do not:

1. change code,
2. change legacy docs,
3. define wire formats,
4. define exact Rust structs,
5. claim pro-rata settlement is implemented,
6. claim operatorless escrow is implemented,
7. claim Halo2 proof is complete,
8. use quality-of-answer as settlement primitive,
9. restore AI marketplace framing,
10. create CLI/ledger/wallet tasks.

Mechanical truth guardrail:

```text
V0 changes direction.
V0 does not change what is currently implemented.
```

---

## 8. Current Git Notes

Recent checkpoint commit:

```text
ea38d4b docs:
```

Note:

The commit subject accidentally became only `docs:`. Do not amend if it was already pushed.

Known untracked local files may exist:

```text
PROMPT_LOG.md
TASK_LOG.md
check_*.py
validate*.py
spec/PRIVAI_V0_PRIVATE_COMPUTE/*.pdf
```

Do not include them unless explicitly asked.

---

## 9. Next Chat Start Instruction

When continuing in a new chat, start with:

```text
Kontynuujemy privAI V0 private compute reset.
Read TASK_031_OPUS_WORKING_CONTEXT.md.
Active task is T-031-OPUS Compute Lease Settlement Direction.
Do not touch code or legacy docs.
```
