# privAI V0 Single Source Of Truth Context Plan

**Status:** V0 context-layer planning doc
**Date:** 2026-04-11
**Scope:** documentation completion gate, RAG/MCP source policy, multi-agent coordination

This document defines how V0 documentation becomes the single source of truth for Gemini/Xiaomi, Claude/Opus, Codex, and future MCP/RAG-backed agents.

It does not implement RAG.
It does not implement MCP.
It does not change code.
It does not import legacy marketplace docs as context.

---

## 1. Core Rule

```text
The V0 folder is the active source of truth.
Legacy marketplace docs are not source material for V0 agent work.
Code-touching V0 work must not read or rely on legacy docs at all.
```

The active folder is:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/
```

The active product model is:

```text
privAI is a post-quantum FullPrivacy private AI compute network.
It is not an AI marketplace.
```

---

## 2. Why This Exists

The project is now too large to rely on chat memory.

Every model must see the same context:

- Gemini/Xiaomi for broad reasoning and alternate perspective,
- Claude/Opus for senior review and freeze decisions,
- Codex for local repo work, logs, prompts, and later implementation,
- future RAG/MCP tools for retrieval and controlled repo access.

The system must prevent each model from reconstructing the project differently.

---

## 3. Source Policy

### 3.1 Active Canonical Sources

Only these are active by default:

- V0 docs inside `spec/PRIVAI_V0_PRIVATE_COMPUTE/`,
- V0 task log,
- V0 prompt log,
- future V0 docs created inside the same folder.

### 3.2 Explicitly Excluded

These are not active V0 context:

- old root `TASK_LOG.md`,
- old root `PROMPT_LOG.md`,
- legacy handoff docs,
- legacy marketplace docs,
- old production direction docs,
- old contract/quality settlement docs,
- old public marketplace/discovery/reputation docs.

They must not be indexed into V0 RAG.
They must not be exposed by V0 MCP.
They must not be used for code-touching tasks.

### 3.3 Legacy Zero-Use Rule For V0 Code Work

Legacy docs are historical archive only.

For V0 code work:

- do not read legacy docs,
- do not cite legacy docs,
- do not map code tasks from legacy docs,
- do not use old product/business language,
- do not "carry over" old marketplace assumptions.

If implementation truth is needed, read:

- current source code,
- current tests,
- current V0 docs,
- current V0 protocol specs once written.

If a missing concept is discovered, create or update a V0 doc inside:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/
```

Do not patch legacy docs to make them current.

---

## 4. Documentation Completion Gate

Do not build production RAG/MCP until the V0 documentation baseline is complete.

### 4.1 Minimum V0 Direction Baseline

The direction baseline is complete only when these exist:

- `PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md`
- `PRIVAI_V0_DIAGRAMS.md`
- `PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md`
- `PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md`
- `PRIVAI_V0_IDENTITY_MODEL_DIRECTION.md`
- `PRIVAI_V0_NODE_ROLES_AND_INCENTIVES_DIRECTION.md`
- `PRIVAI_V0_METERING_PROTOCOL_DIRECTION.md`
- `PRIVAI_V0_PRIVATE_DISCOVERY_DIRECTION.md`
- `PRIVAI_V0_RUNTIME_PRIVACY_CLASSES_DIRECTION.md`
- `PRIVAI_V0_TRANSPORT_MAILBOX_PRIVACY_DIRECTION.md`
- `PRIVAI_V0_EXIT_NODE_DIRECTION.md`
- `PRIVAI_V0_APVA_DENOMINATION_DIRECTION.md`
- `PRIVAI_V0_PROTOCOL_VERSIONING_DIRECTION.md`

Before this list is complete, RAG/MCP may be tested locally, but it must be labeled:

```text
experimental V0 docs retrieval, not production context layer
```

### 4.2 Protocol Spec Baseline

Protocol specs come after direction docs.

They must not be guessed by RAG or agents.

Examples:

- exact metering receipt schema,
- exact lease object,
- exact aPVA precision freeze,
- exact pro-rata note split,
- exact identity credential schema,
- exact version registry.

### 4.3 Implementation Baseline

Implementation planning comes after direction and relevant protocol specs.

No code task should be issued because a RAG answer "sounds complete".

---

## 5. RAG Ingestion Plan

### Phase R0: No RAG, Docs First

Goal:

- finish V0 direction docs,
- keep logs current,
- keep legacy out of the active context.

### Phase R1: V0-Only Retrieval

Ingest only:

- V0 docs,
- V0 task log,
- V0 prompt log,
- V0 docs tree,
- this context plan.

Required metadata per chunk:

| Field | Meaning |
|-------|---------|
| `doc_id` | Stable filename. |
| `tier` | Tier from docs tree. |
| `status` | existing / next / planned / future protocol spec. |
| `authority` | master / direction / planning / log / prompt / future. |
| `allowed_use` | direction / tracking / planning / read-only context. |
| `forbidden_use` | what not to infer from the chunk. |
| `version_track` | `V0_PRIVATE_COMPUTE`. |

### Phase R2: Retrieval Evaluation

Before using RAG with agents, test it with fixed questions:

1. What is privAI?
2. Is privAI an AI marketplace?
3. Is operatorless escrow implemented today?
4. Is pro-rata settlement implemented today?
5. What is the next V0 doc?
6. Are legacy docs active source of truth?
7. What does settlement judge?
8. What must not be inferred from V0 docs?

The expected answers must match the V0 docs.

If RAG retrieves legacy marketplace framing as a normal answer, the RAG setup fails.

### Phase R3: MCP Read-Only Layer

Initial MCP tools should be read-only:

- `v0_list_docs`
- `v0_read_doc`
- `v0_search_docs`
- `v0_get_docs_tree`
- `v0_get_task_log`
- `v0_get_prompt_log`
- `v0_get_next_task`
- `v0_get_guardrails`
- `v0_get_forbidden_inferences`

No write tools in the first MCP phase.

No code edit tools in the first MCP phase.

### Phase R4: Agent Team Routing

All agents receive the same context policy.

They differ by role, not by source of truth.

| Agent | Primary Use | Allowed To Do |
|-------|-------------|---------------|
| Gemini/Xiaomi | read-only reasoning, risk analysis, alternate framing | answer bounded prompts, ask hard questions, identify gaps |
| Claude/Opus | senior review, freeze memos, final direction validation | approve/refine docs, identify contradictions |
| Codex | repo work, logs, prompts, later implementation | create/edit docs, maintain logs, run checks, implement only after docs/specs |

### Phase R5: Controlled Expansion

Only after V0 direction docs are stable:

- add selected code-fact summaries derived from code/tests only,
- add protocol specs,
- add test plans,
- add implementation plans.

Do not add legacy docs to retrieval.

---

## 6. Agent Answer Contract

Every V0 agent response should state:

1. Which V0 files were read.
2. Whether the answer is:
   - current implementation fact,
   - V0 direction,
   - future protocol follow-up,
   - open question.
3. Whether any files were edited.
4. What is explicitly not claimed.
5. Whether there are blocking questions.

If the agent cannot answer from V0 docs, it must say:

```text
Not answered by current V0 docs.
Needs a V0 direction/spec doc or explicit code audit.
```

---

## 7. Non-Negotiable Retrieval Guardrails

1. Do not answer from old AI marketplace framing.
2. Do not retrieve legacy docs.
3. Do not claim operatorless escrow is implemented.
4. Do not claim pro-rata settlement is implemented.
5. Do not claim metering receipt schema is frozen.
6. Do not claim hidden-root identity schema is frozen.
7. Do not claim Halo2 full privacy proof is complete.
8. Do not define wire formats from direction docs.
9. Do not merge validator and compute miner as one protocol role.
10. Do not make internet exit default.

---

## 8. Stop Conditions

Stop and ask for operator decision if:

- a model tries to reintroduce marketplace as the product,
- a model tries to use legacy docs as source material,
- a model tries to write code before direction/spec docs exist,
- a model claims an unimplemented feature is implemented,
- a RAG answer mixes old and V0 framing,
- a task requires exact protocol format not yet frozen.

---

## 9. One-Line Team Rule

```text
Different models may reason differently, but they must all read the same V0 source of truth and must not invent missing protocol truth or recover old marketplace assumptions from legacy docs.
```
