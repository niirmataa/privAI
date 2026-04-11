# privAI V0 Context MCP Server Direction

**Status:** V0 direction / implementation planning boundary
**Date:** 2026-04-11
**Working name:** `privai-context-mcp`
**Scope:** V0-only MCP context server direction, tool contract, source policy, rollout order

This document defines the intended shape of a `privAI`-specific MCP context server.

It does not implement the server.
It does not define final Rust modules as code truth.
It does not authorize code changes.
It does not authorize legacy docs as input.
It does not authorize RAG ingestion of legacy docs.

---

## 1. Decision

Build a narrow `privAI`-specific MCP server:

```text
privai-context-mcp
```

This is not a generic MCP framework.

It is a controlled context layer for `privAI V0 private compute` work.

Its purpose is to make Gemini/Xiaomi, Claude/Opus, Codex, and future agents work from one shared source of truth.

---

## 2. Core Purpose

The server has only three primary jobs:

1. Shorten new-agent onboarding.
2. Build bounded task context packs without bloat.
3. Enforce truth layers and V0 guardrails.

It is not responsible for:

- executing tasks,
- editing docs,
- editing code,
- replacing senior review,
- replacing production RAG,
- reading the whole repo,
- recovering old marketplace framing.

---

## 3. Source Policy

### 3.1 V0-Only Source Rule

The first production-use version reads only:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/
```

Legacy docs are excluded.

Old handoff docs are excluded.

Old root task/prompt logs are excluded.

Old marketplace docs are excluded.

### 3.2 Code Truth Rule

When code work begins later, implementation facts may only come from:

- current source code,
- current tests,
- explicit code audit output,
- V0 code-landing docs created under `spec/PRIVAI_V0_PRIVATE_COMPUTE/`.

Implementation facts must not come from legacy docs.

### 3.3 No Legacy Recovery By Default

The MCP server must not expose a tool such as:

```text
lookup_legacy
```

If a missing idea exists only in old docs or chat history, it must be rewritten manually into a V0 doc before entering the MCP/RAG context layer.

---

## 4. Knowledge Layers

The server should use V0-native layers only.

| Layer | Meaning | Allowed Sources |
|-------|---------|-----------------|
| `v0_master` | Highest-level V0 product/business truth. | V0 direction reset. |
| `v0_direction` | Accepted V0 domain direction docs. | V0 direction docs. |
| `v0_control` | Logs, docs tree, prompt tracking, context policy. | V0 logs and planning docs. |
| `v0_future_spec` | Planned specs that are not yet implementation truth. | Future-spec placeholders and docs tree entries. |
| `repo_verified` | Facts from explicit code/test reads. | Current code/tests only, later phase. |
| `unknown` | Not answered by current V0 docs. | No source. |

Do not use a `handoff` or legacy layer for V0 MCP.

---

## 5. v1 Tool Contract

v1 should expose no more than eight tools.

All tools are read-only.

### 5.1 `privai_v0_get_reading_order`

Input:

- `mode: new_agent | reentry | deep_spec | task_execution | review`

Output:

- ordered docs,
- why these docs,
- what not to read,
- what not to infer.

### 5.2 `privai_v0_get_current_status`

Output:

- current phase,
- completed V0 docs,
- planned V0 docs,
- highest-priority gap,
- unsafe claims,
- next recommended task.

### 5.3 `privai_v0_lookup_direction`

Input:

- `query`
- optional `topic`

Output:

- matching V0 direction items,
- source path,
- source layer,
- overclaim warnings.

Only reads V0 direction docs.

### 5.4 `privai_v0_lookup_control`

Input:

- `query`

Output:

- matching V0 task log, prompt log, docs tree, or context-policy entries.

Only reads V0 control/planning docs.

### 5.5 `privai_v0_get_guardrails`

Input:

- optional `topic`

Output:

- applicable non-negotiable rules,
- forbidden inferences,
- stop conditions.

### 5.6 `privai_v0_route_question`

Input:

- `question`

Output:

- `layer: v0_master | v0_direction | v0_control | v0_future_spec | repo_verified | unknown`
- `needs_code_audit: bool`
- `needs_protocol_spec: bool`
- `must_not_use_legacy: true`
- explanation.

### 5.7 `privai_v0_prepare_task_context`

Input:

- `task_title`
- `task_goal`
- optional `target_model`
- optional `write_scope`

Output:

- task,
- why_now,
- what_is_already_there,
- what_is_missing,
- depends_on,
- can_run_in_parallel,
- do_not_touch,
- minimal_reading_scope,
- unchecked_assumptions,
- definition_of_done,
- exact_final_report_format.

This is the most important v1 tool.

### 5.8 `privai_v0_build_correction_pill`

Input:

- `finding`
- `affected_file`
- `severity`
- optional `correction_direction`

Output:

- bounded correction task,
- source-of-truth docs,
- forbidden changes,
- definition of done,
- final report format.

---

## 6. Tools Explicitly Forbidden In v1

Do not build:

- `search_everything`
- `read_whole_repo`
- `lookup_legacy`
- `generate_architecture_from_code`
- `execute_task`
- `edit_docs`
- `edit_code`
- `auto_rewrite_docs`
- `auto_create_protocol_spec`

v1 is a context server, not an execution server.

---

## 7. Data Record Shape

The initial data store may be JSON generated from V0 docs.

Every item should carry source and guardrail metadata.

Example:

```json
{
  "id": "v0-master-private-compute-reset",
  "layer": "v0_master",
  "title": "Private compute network reset",
  "topic": "product_framing",
  "summary": "privAI is not an AI model marketplace; it is a post-quantum FullPrivacy private AI compute network.",
  "content": "Condensed content or full document section.",
  "tags": ["v0", "direction", "private-compute", "product"],
  "status": "canonical",
  "recommended_for": ["new_agent", "reentry", "task_execution"],
  "do_not_overclaim": [
    "do not infer public marketplace baseline",
    "do not infer subjective quality settlement",
    "do not infer operatorless escrow is implemented"
  ],
  "source_path": "spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md",
  "source_scope": "v0_only",
  "legacy_allowed": false
}
```

Required fields:

| Field | Purpose |
|-------|---------|
| `id` | Stable item id. |
| `layer` | V0 knowledge layer. |
| `title` | Human-readable label. |
| `topic` | Routing topic. |
| `summary` | Short context summary. |
| `content` | Full or condensed content. |
| `tags` | Retrieval tags. |
| `status` | canonical / direction / planning / future_spec / open. |
| `recommended_for` | Reading-order modes. |
| `do_not_overclaim` | Guardrails attached to the item. |
| `source_path` | Exact V0 source path. |
| `source_scope` | Must be `v0_only` in v1. |
| `legacy_allowed` | Must be `false` in v1. |

---

## 8. Storage Direction

### v1 Store

Use a simple file-backed store:

```text
FileStore
```

Inputs:

- generated JSON indexes from V0 docs,
- V0 docs tree,
- V0 logs.

### Later Stores

Only after v1 is stable:

- `DocsIndexStore`
- `VertexRetrieverStore`
- `HybridStore`

All later stores must preserve the same tool output contracts.

---

## 9. Planned Rust Module Shape

This is an implementation direction, not code truth.

```text
privai-context-mcp/
├── Cargo.toml
├── README.md
├── data/
│   ├── v0_master.json
│   ├── v0_direction.json
│   ├── v0_control.json
│   └── v0_future_spec.json
├── src/
│   ├── main.rs
│   ├── server.rs
│   ├── config.rs
│   ├── errors.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── knowledge_item.rs
│   │   ├── layer.rs
│   │   ├── task_pack.rs
│   │   └── tool_io.rs
│   ├── store/
│   │   ├── mod.rs
│   │   ├── trait.rs
│   │   └── file_store.rs
│   ├── routing/
│   │   ├── mod.rs
│   │   ├── question_router.rs
│   │   └── guardrails.rs
│   ├── builders/
│   │   ├── mod.rs
│   │   ├── reading_order.rs
│   │   ├── status_snapshot.rs
│   │   ├── task_context.rs
│   │   └── correction_pill.rs
│   └── tools/
│       ├── mod.rs
│       ├── get_reading_order.rs
│       ├── get_current_status.rs
│       ├── lookup_direction.rs
│       ├── lookup_control.rs
│       ├── get_guardrails.rs
│       ├── route_question.rs
│       ├── prepare_task_context.rs
│       └── build_correction_pill.rs
└── examples/
    ├── claude-code-mcp.json
    ├── codex-mcp.toml
    └── kilo-code-mcp.json
```

Do not create this structure until a specific implementation task is issued.

---

## 10. Golden Tests For v1

Before connecting real agents, the server must answer these correctly:

1. What is privAI?
2. Is privAI an AI marketplace?
3. Is public discovery the baseline?
4. Is operatorless escrow implemented today?
5. Is pro-rata split implemented today?
6. What is the current settlement direction?
7. What is the next V0 doc/task?
8. Can legacy docs be used for code-touching V0 work?
9. What should an agent read before writing an operatorless escrow doc?
10. What should an agent do if asked for an exact receipt schema?

Expected core answers:

- privAI is a post-quantum FullPrivacy private AI compute network.
- No, privAI is not an AI marketplace.
- No, discovery is private/encrypted/resource-based as baseline.
- No, operatorless escrow is direction, not current implementation.
- No, pro-rata split is future protocol work.
- Settlement is lease policy + metering receipts, not AI output quality.
- Next heavy doc is operatorless escrow direction unless logs say otherwise.
- No, legacy docs are zero-use for V0 code work.
- Read V0 master, settlement direction, diagrams, docs tree, logs, and context policy.
- Mark exact receipt schema as future protocol spec; do not invent it.

---

## 11. Rollout Plan

### Sprint 0: Docs Contract

Goal:

- freeze this direction doc,
- finish required V0 direction docs,
- keep logs current.

No code.

### Sprint 1: Local MCP Skeleton

Goal:

- Rust MCP server,
- local `stdio`,
- v1 eight-tool contract,
- `FileStore`,
- generated V0 JSON data.

No RAG.

No write tools.

No legacy docs.

### Sprint 2: Guardrails And Golden Tests

Goal:

- V0-aware routing,
- task context builder,
- correction pill builder,
- golden tests.

### Sprint 3: Agent Integration

Goal:

- Claude Code config,
- Codex config,
- Kilo Code config,
- fresh-agent tests.

### Sprint 4: Retriever Upgrade

Goal:

- optional Vertex RAG / hybrid retriever,
- same output contracts,
- still V0-only unless explicitly extended by a future V0-approved spec.

---

## 12. Final Guardrail

```text
The MCP server exists to prevent context drift.
If it starts mixing V0 with legacy marketplace assumptions, it has failed.
```
