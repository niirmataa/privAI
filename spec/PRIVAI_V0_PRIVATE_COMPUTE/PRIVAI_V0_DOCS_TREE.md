# privAI V0 Docs Tree

**Status:** canonical V0 docs planning map
**Date:** 2026-04-11
**Scope:** documentation tree, sequencing, freeze boundaries

This document defines the documentation tree needed to carry `privAI` from V0 private compute direction to production planning.

It also defines the source boundary for the future RAG/MCP context layer.

It does not define protocol wire formats.
It does not change code.
It does not supersede code-confirmed mechanical docs.

---

## 1. Core Rule

```text
privAI is not an AI model marketplace.
privAI is a post-quantum FullPrivacy private AI compute network.
```

```text
Privacy is the product.
Compute is the supply.
PVA is the incentive.
Chain is the settlement.
Transport is the shield.
```

All V0 docs must preserve this direction.

---

## 2. Status Legend

Use these statuses consistently:

| Status | Meaning |
|--------|---------|
| `existing` | File exists and is part of the V0 docset. |
| `next` | Next high-priority file to create. |
| `planned` | Needed, but not next. |
| `blocked` | Needs prior direction/review before writing. |
| `future protocol spec` | Must not be written as implementation truth yet. |
| `legacy reference` | Old doc remains useful only where not contradicted by V0. |
| `superseded` | Old product/business framing replaced by V0. |
| `quarantined` | Not active V0 source material; may only be opened in explicit audit tasks. |

---

## 3. Directory Policy

Canonical V0 docs live here:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/
```

Old root logs remain pre-V0 history:

```text
PROMPT_LOG.md
TASK_LOG.md
```

V0 work is tracked only in:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_TASK_LOG.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_PROMPT_LOG.md
```

Legacy handoff docs and old marketplace docs are historical archive only for V0 work.

They are not source material for normal V0 tasks, prompts, RAG, MCP retrieval, or code-touching work.

If implementation truth is needed, read source code and tests directly.

Do not recover implementation tasks from legacy docs.

Do not patch legacy docs to make them current.

All new direction, protocol, planning, and code-landing documentation must be written under:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/
```

---

## 4. Tier 0: Control / Navigation Docs

These docs keep agents oriented. They are not protocol specs.

| File | Status | Purpose | Notes |
|------|--------|---------|-------|
| `PRIVAI_V0_TASK_LOG.md` | `existing` | Canonical V0 task log. | Update after completed V0 tasks. |
| `PRIVAI_V0_PROMPT_LOG.md` | `existing` | Canonical V0 prompt log. | Update when prompts/results are issued. |
| `TASK_031_OPUS_WORKING_CONTEXT.md` | `existing` | New-chat handoff around T-031. | Working context, not canonical direction. |
| `PRIVAI_V0_DOCS_TREE.md` | `existing` | This docs tree. | Guides next docs and phase order. |
| `PRIVAI_V0_SINGLE_SOURCE_OF_TRUTH_CONTEXT_PLAN.md` | `existing` | RAG/MCP source policy and multi-agent context plan. | V0-only; legacy excluded. |
| `PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md` | `existing` | Direction for `privai-context-mcp` v1. | Read-only V0 context server; no code yet. |
| `README.md` | `planned` | Folder entrypoint. | Recreate/update if needed; current folder may not have tracked README. |

---

## 5. Tier 1: Canonical Direction Docs

These are the highest-level V0 product/business direction docs.

| File | Status | Purpose | Notes |
|------|--------|---------|-------|
| `PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md` | `existing` | Master V0 direction reset. | Highest product/business authority. |
| `PRIVAI_V0_LEGACY_DOCSET_REWRITE_PLAN.md` | `existing` | Senior review + migration plan. | Classifies legacy docs and rewrite order. |
| `PRIVAI_V0_DIAGRAMS.md` | `existing` | Visual companion. | 15 diagrams; one future-strengthening transition diagram. |
| `PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md` | `existing` | Settlement direction. | Receipt/metering settlement, not quality settlement. |

Rule:

```text
Tier 1 docs define direction.
They do not claim implementation completeness.
```

---

## 6. Tier 2: Required V0 Direction Docs

These docs should be written before any serious protocol or code work.

| File | Status | Purpose | Must Not Do |
|------|--------|---------|-------------|
| `PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md` | `next` | Operatorless target, Phase 0/1/2 bridge, current 2-of-3 reality. | Must not claim operatorless is implemented. |
| `PRIVAI_V0_IDENTITY_MODEL_DIRECTION.md` | `planned` | Hidden root credential, scoped role/session/epoch IDs, Falcon boundaries. | Must not define credential wire format. |
| `PRIVAI_V0_NODE_ROLES_AND_INCENTIVES_DIRECTION.md` | `planned` | Validator / compute miner / relay / mailbox / exit roles and incentives. | Must not define final reward formulas. |
| `PRIVAI_V0_METERING_PROTOCOL_DIRECTION.md` | `planned` | Direction-level metering, receipts, challenges, availability evidence. | Must not define exact receipt schema. |
| `PRIVAI_V0_PRIVATE_DISCOVERY_DIRECTION.md` | `planned` | Private/encrypted/resource-based discovery, bootstrap phases. | Must not pick final DHT/gossip/registry architecture. |
| `PRIVAI_V0_RUNTIME_PRIVACY_CLASSES_DIRECTION.md` | `planned` | Runtime slice classes: VM/container/sandbox/confidential runtime. | Must not overclaim host privacy guarantees. |
| `PRIVAI_V0_TRANSPORT_MAILBOX_PRIVACY_DIRECTION.md` | `planned` | NXMS/mailbox/relay metadata hardening direction. | Must not claim current NXMS is metadata-free. |
| `PRIVAI_V0_EXIT_NODE_DIRECTION.md` | `planned` | Internet exit as explicit opt-in role, risk and pricing boundary. | Must not make exit default. |
| `PRIVAI_V0_APVA_DENOMINATION_DIRECTION.md` | `planned` | PVA/aPVA precision and integer accounting decision. | Must not choose type/precision without supply constraints. |
| `PRIVAI_V0_PROTOCOL_VERSIONING_DIRECTION.md` | `planned` | Version domains and no silent downgrade rule. | Must not define final registry format yet. |

---

## 7. Tier 3: Future Protocol Specs

These must come after Tier 2 direction docs are accepted.

Do not write these as final specs until the direction docs and senior review are done.

| File | Status | Depends On | Purpose |
|------|--------|------------|---------|
| `PRIVAI_V0_APVA_PRECISION_FREEZE.md` | `future protocol spec` | aPVA direction, supply assumptions | Freeze `1 PVA = N aPVA`, integer type, rounding rules. |
| `PRIVAI_V0_COMPUTE_LEASE_OBJECT_SPEC.md` | `future protocol spec` | settlement direction | Exact lease object, commitment, lifecycle. |
| `PRIVAI_V0_METERING_RECEIPT_SCHEMA.md` | `future protocol spec` | metering direction | Exact receipt fields, signing message, validation rules. |
| `PRIVAI_V0_METERING_TRUST_AND_CHALLENGE_SPEC.md` | `future protocol spec` | metering direction, Xiaomi notes | Challenge/response and anti-self-report fraud model. |
| `PRIVAI_V0_RECEIPT_AVAILABILITY_SPEC.md` | `future protocol spec` | settlement direction, metering direction | Who stores receipts, retention, redundancy, settlement evidence availability. |
| `PRIVAI_V0_OPERATORLESS_ESCROW_PROTOCOL_BRIDGE.md` | `future protocol spec` | operatorless direction | Phase 0 -> Phase 1 -> Phase 2 transition mechanics. |
| `PRIVAI_V0_PRORATA_NOTE_SPLIT_SPEC.md` | `future protocol spec` | settlement direction, aPVA freeze | 1 input -> 2 outputs mechanics, rounding, target validation. |
| `PRIVAI_V0_IDENTITY_CREDENTIAL_SCHEMA.md` | `future protocol spec` | identity direction | Hidden root credential, scoped IDs, rotation, revocation. |
| `PRIVAI_V0_PRIVATE_DISCOVERY_PROTOCOL_SPEC.md` | `future protocol spec` | discovery direction, identity direction | Discovery query/offering protocol. |
| `PRIVAI_V0_RELIABILITY_SCORING_SPEC.md` | `future protocol spec` | roles/incentives direction, metering direction | Deterministic machine score formula. |
| `PRIVAI_V0_VERSION_REGISTRY_SPEC.md` | `future protocol spec` | versioning direction | Registry fields, activation rules, compatibility matrix. |

---

## 8. Tier 4: Legacy Quarantine / Migration Docs

These old docs are not V0 source material.

They must not be indexed into V0 RAG.

They must not be given to agents as default context.

They must not be used for code-touching V0 work.

V0 work creates new V0 docs instead of rewriting old docs.

| Legacy Doc | Status | Action |
|------------|--------|--------|
| `PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md` | `quarantined` | Old product/business framing; not V0 context. |
| `PRIVAI_PRODUCTION_SYSTEM_DIAGRAMS.md` | `quarantined` | Superseded by V0 diagrams; not V0 context. |
| `PRIVAI_CONTRACT_VERIFICATION_AND_SETTLEMENT_DIRECTION.md` | `quarantined` | Superseded by V0 compute lease settlement direction. |
| `PRIVAI_OPERATOR_AND_DISPUTE_QUORUM_DIRECTION.md` | `quarantined` | Old operator/dispute framing; replace with V0 operatorless direction. |
| `PRIVAI_SYSTEM_PRODUCT_FOUNDATION.md` | `quarantined` | Old entry/product framing; not active V0 context. |
| `PRIVAI_PROJECT_ENTRYPOINT.md` | `quarantined` | Old entry framing; use V0 folder entrypoint once created. |
| `PRIVAI_V1_READINESS_AND_GAPS.md` | `quarantined` | May contain implementation facts, but not default V0 context. |
| `PRIVAI_NEXT_DIRECTION.md` | `quarantined` | Old next-direction framing; use V0 docs tree instead. |
| `PRIVAI_DOCS_INDEX.md` | `quarantined` | Old/global index; use V0 docs tree for V0 work. |
| `PRIVAI_START_HERE.md` | `quarantined` | Old/global start path; use V0 folder entrypoint once created. |

Rule:

```text
Do not mass rewrite legacy docs.
Do not use legacy docs as context.
Do not use legacy docs for code-touching V0 work.
Create new replacement docs only under `spec/PRIVAI_V0_PRIVATE_COMPUTE/`.
```

---

## 9. Tier 5: Implementation Planning Docs

These come only after Tier 2 and key Tier 3 specs are reviewed.

| File | Status | Purpose |
|------|--------|---------|
| `PRIVAI_V0_CODE_LANDING_ZONES.md` | `blocked` | Map V0 specs to chain/ledger/wallet/node/transport code areas. |
| `PRIVAI_V0_TEST_MATRIX.md` | `blocked` | Define tests per phase before code tasks. |
| `PRIVAI_V0_PHASE_1_AUTOMATED_OPERATOR_IMPLEMENTATION_PLAN.md` | `blocked` | Implementation plan after operatorless direction and metering direction. |
| `PRIVAI_V0_COMPUTE_LEASE_DEVNET_PLAN.md` | `blocked` | Devnet plan for private compute lease sessions. |
| `PRIVAI_V0_PRODUCTION_READINESS_CHECKLIST.md` | `blocked` | Final readiness criteria. |
| `PRIVAI_V0_CONTEXT_MCP_IMPLEMENTATION_PLAN.md` | `blocked` | Implementation plan for `privai-context-mcp` after context direction is accepted. |

Rule:

```text
No code task before the relevant direction doc and protocol spec exist.
```

---

## 10. Production Phase Plan

This is the high-level phase order. Detailed phase docs can be created later.

### Phase 0: V0 Direction Freeze

Goal:

- stop product drift,
- freeze private compute framing,
- create direction docs.

DoD:

- V0 master exists,
- diagrams exist,
- settlement direction exists,
- docs tree exists,
- V0 task/prompt logs are active.

Current status: `in progress`.

### Phase 1: Compatibility Bridge

Goal:

- keep current 2-of-3 escrow mechanics,
- use automated operator as receipt-checking bridge,
- test lease policy and metering outside consensus.

DoD:

- operatorless direction exists,
- metering direction exists,
- receipt availability direction exists,
- automated operator behavior is directionally specified.

### Phase 2: Receipt-Aware Escrow

Goal:

- define lease policy,
- define receipt validation,
- define pro-rata mechanics,
- bridge to ledger/escrow protocol.

DoD:

- compute lease object spec exists,
- metering receipt schema exists,
- pro-rata note split spec exists,
- timeout claim path is specified.

### Phase 3: Operatorless Settlement

Goal:

- move receipt validation from automated operator toward protocol validation,
- remove operator from canonical normal settlement path.

DoD:

- operatorless protocol bridge accepted,
- migration path from current escrow notes defined,
- test matrix accepted.

### Phase 4: Private Discovery / Identity / Transport Hardening

Goal:

- hidden root + scoped IDs,
- private resource discovery,
- NXMS/mailbox metadata hardening,
- exit node opt-in model.

DoD:

- identity credential schema exists,
- discovery protocol spec exists,
- transport/mailbox privacy direction accepted,
- exit node direction accepted.

### Phase 5: Production Readiness

Goal:

- no silent downgrade,
- V0 docs and code align,
- all roles and incentives defined,
- devnet/testnet readiness.

DoD:

- code landing zones done,
- test matrix done,
- production checklist done,
- legacy marketplace framing no longer drives agents.

---

## 11. Current Blocking Facts

- Opus is unavailable until 2026-04-15.
- Heavy final specs should wait for Opus review.
- Xiaomi can be used for read-only discussion and risk analysis.
- Codex can maintain logs, working memos, and task prompts.
- No code changes should be made for V0 until the docs phase plan is stable.

---

## 12. Next Safe Work Until Opus Returns

Safe work:

1. Complete the V0 direction baseline before production RAG/MCP:
   - operatorless escrow,
   - identity,
   - node roles and incentives,
   - metering,
   - private discovery,
   - runtime privacy classes,
   - transport/mailbox privacy,
   - exit node,
   - aPVA denomination,
   - protocol versioning.

2. Collect Xiaomi/Codex discussion notes on:
   - receipt availability,
   - metering trust model,
   - aPVA precision,
   - private discovery tradeoffs,
   - runtime privacy classes,
   - exit node risk/pricing.

3. Keep V0 logs current.

4. Prepare strict task prompts for Opus:
   - Operatorless Escrow Direction,
   - Identity Model Direction,
   - Node Roles And Incentives Direction,
   - Metering Protocol Direction.

5. Prepare the future RAG/MCP source policy from `PRIVAI_V0_SINGLE_SOURCE_OF_TRUTH_CONTEXT_PLAN.md`, but do not ingest legacy docs.

6. Prepare the future `privai-context-mcp` implementation task from `PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md`, but do not implement it until the V0 direction baseline is stable.

7. Do not freeze protocol wire formats yet.

---

## 13. Non-Negotiable Guardrails

1. No public AI marketplace as baseline.
2. No public discovery as baseline.
3. No public provider profile as baseline.
4. No subjective AI quality settlement.
5. No operator as canonical escrow decision-maker.
6. No silent downgrade from FullPrivacy.
7. No code changes before relevant direction/spec docs exist.
8. No V0 product reset overriding code-confirmed deep-spec truth.
9. No final wire formats without explicit protocol spec task.
10. No mass rewrite of legacy docs.
11. No legacy docs in V0 RAG/MCP retrieval.
12. No agent gets a different source of truth.
13. No code-touching V0 task reads legacy docs.

---

## 14. Handoff Guide For The Next Model

This section is a plain-language guide for any model continuing the V0 work.

The important point:

```text
Do not treat this folder as a pile of unrelated notes.
This folder is the new V0 direction track.
```

### 14.1 What Happened

The project deliberately moved away from the old AI marketplace framing.

The old framing created too many dangerous surfaces:

- public provider profiles,
- public discovery,
- quality disputes,
- skill/artifact marketplace semantics,
- operator/moderator gravity,
- public reputation/social graph risks.

The new V0 framing is:

```text
private AI-capable compute network
```

The user privately leases isolated runtime capacity. The compute miner provides machine resources. The chain settles value. Transport protects communication.

### 14.2 How To Read This Folder

Read in this order:

1. `PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md`
   - Understand the new product/business model.
   - This is the top-level V0 authority.

2. `PRIVAI_V0_DIAGRAMS.md`
   - Build a visual mental model.
   - Use this to avoid slipping back into marketplace language.

3. `PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md`
   - Understand what escrow settles now.
   - Settlement is resource delivery, not AI quality.

4. `PRIVAI_V0_LEGACY_DOCSET_REWRITE_PLAN.md`
   - Understand which old docs are still useful and which are superseded.

5. `PRIVAI_V0_DOCS_TREE.md`
   - Use this file to choose the next doc/task.

6. `PRIVAI_V0_TASK_LOG.md` and `PRIVAI_V0_PROMPT_LOG.md`
   - Check what has already been done.
   - Update these after V0 work.

### 14.3 What Each Tier Means

**Tier 0: Control / Navigation**

These files are for orientation and tracking. They prevent context loss across chats and models. They are not product specs and not protocol specs.

Use them to answer:

- What task are we on?
- What has already been completed?
- What should be read first?
- What should not be updated anymore?

**Tier 1: Canonical Direction**

These files define what the system is becoming.

They answer:

- What is privAI now?
- What is no longer true?
- What is the settlement model?
- What diagrams should agents carry in their heads?

They do not prove implementation completeness.

**Tier 2: Required Direction Docs**

These are the next docs that must exist before code work.

They answer one domain at a time:

- How does operatorless escrow evolve?
- What is identity?
- Who earns PVA and why?
- What is metering?
- How does private discovery work directionally?
- What are runtime privacy classes?
- What does transport/mailbox privacy need to become?

These are not wire formats.

**Tier 3: Future Protocol Specs**

These are precise protocol documents. Do not write them too early.

They answer:

- What exact fields exist?
- What exact signatures are made?
- What exact commitments are used?
- What exact rounding rules apply?
- What exact validation logic is required?

Only write these after the related Tier 2 direction doc is accepted.

**Tier 4: Legacy Migration**

These are old docs that still contain useful code/mechanical truth, but often carry old marketplace product framing.

Do not mass rewrite them.

The safe rule is:

```text
replace old framing only after the V0 replacement doc exists
```

**Tier 5: Implementation Planning**

These docs map accepted V0 direction/specs into code and tests.

Do not start here.

Implementation planning comes after:

- direction docs,
- protocol specs,
- review,
- agreed phase order.

### 14.4 How To Continue The Conversation

If continuing in a new chat, start with:

```text
We are continuing privAI V0 private compute.
Read PRIVAI_V0_DOCS_TREE.md first.
Then read the V0 task/prompt logs.
Do not use old AI marketplace framing.
Do not touch code.
```

Then check the next task in `PRIVAI_V0_TASK_LOG.md`.

### 14.5 Current Next Work

As of this document:

```text
Next heavy doc: PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md
```

But Opus is unavailable until 2026-04-15, so the safest near-term work is:

- collect read-only discussion notes,
- refine task prompts,
- update V0 logs,
- avoid protocol freezes,
- avoid code changes.

### 14.6 Common Mistakes To Avoid

Do not say:

```text
privAI is an AI marketplace
provider sells a skill/artifact
settlement judges AI output quality
operator is canonical decision-maker
public discovery is baseline
public reputation is baseline
MarketplaceBatchTx defines the product
```

Say instead:

```text
privAI is a private compute network
compute miner provides resource capacity
settlement verifies resource delivery
operator is bootstrap/compatibility, not destination
discovery is private/resource-based
scoring is deterministic machine reliability
FullPrivacy escrow is the settlement direction
```

### 14.7 What To Do When Unsure

If there is a conflict:

```text
V0 wins over old product/business marketplace framing.
Deep-spec mechanical docs win over V0 on current implementation facts.
```

If the question is about current code reality, do not answer from V0 direction alone.

If the question is about product/business direction, start from V0.

If the question requires a wire format, schema, or exact protocol rule, mark it as:

```text
future protocol spec
```

Do not guess.

### 14.8 One-Sentence Project Memory

Use this sentence to keep the whole system aligned:

```text
privAI is a post-quantum FullPrivacy private compute network where users privately lease AI-capable runtime capacity, compute miners earn PVA for delivered resources, validators secure settlement, and transport hides communication.
```
