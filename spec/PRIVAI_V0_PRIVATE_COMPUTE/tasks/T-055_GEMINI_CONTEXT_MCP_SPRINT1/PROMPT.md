# T-055-GEMINI — privai-context-mcp Sprint 1

## Role

Senior Rust MCP implementer for `privAI`.

You are implementing Sprint 1 of `privai-context-mcp`.

This is a code task, not an architecture brainstorming task.

---

## Repository

Work in:

```text
/home/nxms-server/privAI
```

Create the server project here:

```text
/home/nxms-server/privAI/privai-context-mcp
```

Do not create server code inside the V0 docs folder.

---

## Critical Source Correction

The V0 truth folder is:

```text
/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE
```

For runtime truth in v1, use only accepted V0 docs from that folder.

Do not use:

- uploaded file copies,
- legacy docs,
- old marketplace docs,
- root `TASK_LOG.md`,
- root `PROMPT_LOG.md`,
- handoff docs,
- raw task outputs,
- raw model discussion outputs.

Important:

The folder now contains a `tasks/` workspace for prompts and model outputs.

Do **not** ingest `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/` into MCP runtime indexes.

Task outputs are non-canonical until reviewed and promoted.

---

## Read Scope

Read:

```text
/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md
/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md
/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md
```

You may also read other accepted V0 docs in:

```text
/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE
```

But exclude:

```text
/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks
/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/TASK_031_OPUS_WORKING_CONTEXT.md
```

If a file is clearly a handoff, raw prompt, raw model output, or superseded workspace artifact, do not use it as runtime truth.

---

## Write Scope

You may create and edit:

```text
/home/nxms-server/privAI/privai-context-mcp/**
/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-055_GEMINI_CONTEXT_MCP_SPRINT1/OUTPUT_GEMINI.md
```

Do not modify:

- existing `privAI` app/runtime code,
- existing chain/ledger/wallet/node code,
- canonical V0 docs,
- legacy docs,
- handoff docs,
- other task folders.

---

## Required Implementation

Create a real local MCP server in Rust for `stdio` use.

Do not implement pseudo-MCP by manually parsing raw stdin JSON lines.

Use a real MCP protocol implementation or SDK. If no suitable Rust MCP library is available in this repo/environment, stop and report instead of faking protocol compliance.

Implement exactly these 8 read-only tools:

1. `privai_v0_get_reading_order`
2. `privai_v0_get_current_status`
3. `privai_v0_lookup_direction`
4. `privai_v0_lookup_control`
5. `privai_v0_get_guardrails`
6. `privai_v0_route_question`
7. `privai_v0_prepare_task_context`
8. `privai_v0_build_correction_pill`

Do not add any extra tools.

---

## Store / Index Requirements

Use a file-backed store only.

Sprint 1 must be file-backed, but the store boundary must be designed so a future Vertex/RAG backend can be plugged in without rewriting the tool layer.

Implement:

```text
KnowledgeStore trait/interface
FileStore implementation
```

Do not implement:

```text
VertexRetrieverStore
HybridStore
embeddings
cloud retrieval
```

The tool layer must depend on the store trait, not directly on filesystem reads, so Sprint 2/3 can add Vertex RAG behind the same contract.

Generate JSON indexes under:

```text
/home/nxms-server/privAI/privai-context-mcp/data/
```

Required generated files:

```text
data/v0_master.json
data/v0_direction.json
data/v0_control.json
data/v0_future_spec.json
```

The index generator must load source documents only from:

```text
/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE
```

The generator must exclude:

```text
tasks/**
TASK_031_OPUS_WORKING_CONTEXT.md
```

Runtime metadata must enforce:

```text
source_scope = "v0_only"
legacy_allowed = false
```

Do not create a generic RAG system.

Do not add Vertex, vector DB, cloud retrieval, or embeddings.

Add a short README section named:

```text
Future Vertex/RAG Integration
```

It must explain:

- Sprint 1 uses `FileStore` only.
- Future Vertex/RAG must remain V0-only.
- Future Vertex/RAG must not ingest `tasks/`, legacy docs, handoff docs, or raw model outputs.
- Future Vertex/RAG must implement the same `KnowledgeStore` contract.
- MCP tool contracts must not change when storage changes.

---

## Expected Project Shape

Use this shape unless the MCP SDK requires minor changes:

```text
privai-context-mcp/
├── Cargo.toml
├── README.md
├── data/
│   ├── v0_master.json
│   ├── v0_direction.json
│   ├── v0_control.json
│   └── v0_future_spec.json
├── examples/
│   ├── claude-code-mcp.json
│   ├── codex-mcp.toml
│   └── kilo-code-mcp.json
├── src/
│   ├── main.rs
│   ├── server.rs
│   ├── config.rs
│   ├── errors.rs
│   ├── models/
│   ├── store/
│   ├── routing/
│   ├── builders/
│   └── tools/
└── tests/
```

---

## Tool Behavior Requirements

### `privai_v0_get_reading_order`

Input:

```text
mode = new_agent | reentry | deep_spec | task_execution
```

Output:

- ordered docs,
- why these docs,
- what not to read yet,
- source_scope,
- legacy_allowed.

### `privai_v0_get_current_status`

Output:

- ready,
- partial,
- open,
- highest_priority_gap,
- unsafe_claims,
- source_scope,
- legacy_allowed.

### `privai_v0_lookup_direction`

Search only V0 direction/master/settlement docs.

Must not return legacy content.

### `privai_v0_lookup_control`

Search only control/planning docs such as docs tree, V0 task log, V0 prompt log, MCP direction.

Must not return raw task outputs.

### `privai_v0_get_guardrails`

Return V0 guardrails:

- not marketplace,
- no public provider profiles,
- discovery private/encrypted/resource-based,
- settlement receipt/metering-based,
- no subjective AI quality settlement,
- operatorless is target, not implemented,
- pro-rata expected, not implemented,
- no silent downgrade from FullPrivacy,
- Falcon is signing tool, not public identity,
- no legacy docs for V0 code work.

### `privai_v0_route_question`

Return one of:

```text
direction
control
future_spec
repo_unverified
unknown
mixed
```

Must explain why.

### `privai_v0_prepare_task_context`

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

### `privai_v0_build_correction_pill`

Output short correction packets for agents that drift into old framing.

Example trigger:

```text
"privAI is an AI marketplace"
```

Expected correction:

```text
privAI V0 is a private compute network, not an AI marketplace.
```

---

## Forbidden Tools / Shortcuts

Do not add:

- `lookup_legacy`
- `search_everything`
- `read_whole_repo`
- `edit_code`
- `edit_docs`
- execution tools
- write tools

Do not infer implementation facts from V0 direction docs.

If code reality is unknown, answer with `repo_unverified`.

---

## Tests

Add unit tests and golden tests.

Golden tests must verify at least:

1. `privAI` is not described as an AI marketplace.
2. Public provider profiles are rejected.
3. Operatorless escrow is described as target/future, not current implementation.
4. Pro-rata split is described as expected/future, not current implementation.
5. Legacy lookup tool does not exist.
6. `tasks/` raw outputs are not indexed.
7. `TASK_031_OPUS_WORKING_CONTEXT.md` is not indexed.
8. `source_scope = v0_only`.
9. `legacy_allowed = false`.
10. `privai_v0_prepare_task_context` returns bounded reading scope, not whole repo.

---

## Minimal Commands

Build:

```text
cd /home/nxms-server/privAI/privai-context-mcp && cargo build
```

Test:

```text
cd /home/nxms-server/privAI/privai-context-mcp && cargo test
```

Run:

```text
cd /home/nxms-server/privAI/privai-context-mcp && cargo run
```

---

## Definition Of Done

- [ ] Rust project exists at `/home/nxms-server/privAI/privai-context-mcp`
- [ ] Server reads V0 truth only from `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE`
- [ ] `tasks/` is excluded from runtime indexing
- [ ] Handoff docs are excluded from runtime indexing
- [ ] Local `stdio` MCP server exists
- [ ] All 8 tools are registered and callable
- [ ] FileStore reads generated V0 JSON indexes
- [ ] No legacy docs are referenced in runtime seed data
- [ ] Example config files for Claude/Codex/Kilo exist
- [ ] Unit tests pass
- [ ] Golden tests pass
- [ ] README explains how to run locally

---

## Exact Final Report Format

Write final report to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-055_GEMINI_CONTEXT_MCP_SPRINT1/OUTPUT_GEMINI.md
```

Use exactly this structure:

```text
# T-055-GEMINI — privai-context-mcp Sprint 1 Output

## WHAT WAS CREATED

## TRUTH SOURCE USED

## TOOL CONTRACT IMPLEMENTED

## GUARDRAILS ENFORCED

## INDEXING EXCLUSIONS

## TEST RESULTS

## OPEN FOLLOW-UPS

## UNCHECKED ASSUMPTIONS

## FINAL SELF-CHECK
```

Final self-check must answer:

- real MCP SDK/protocol used: YES/NO
- pseudo-MCP raw stdin parser avoided: YES/NO
- V0 docs folder used as source: YES/NO
- `tasks/` excluded: YES/NO
- legacy docs excluded: YES/NO
- extra tools added: YES/NO
- existing app/runtime code modified: YES/NO
