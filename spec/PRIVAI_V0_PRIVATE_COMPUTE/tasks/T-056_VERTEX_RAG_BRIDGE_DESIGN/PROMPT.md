# T-056-GEMINI — Vertex RAG Bridge Design For privai-context-mcp

## Role

Senior retrieval / MCP architecture reviewer for `privAI`.

This is a design task, not an implementation task.

Do not edit code.

---

## Repository

Work in:

```text
/home/nxms-server/privAI
```

Write output to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-056_VERTEX_RAG_BRIDGE_DESIGN/OUTPUT_GEMINI.md
```

Do not edit other files.

---

## Goal

Design how future Vertex RAG should connect to `privai-context-mcp` without breaking Sprint 1 local FileStore behavior and without contaminating V0 truth.

The result should define the future storage/retrieval boundary, not implement it.

---

## Source Of Truth

Read:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/README.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-055_GEMINI_CONTEXT_MCP_SPRINT1/PROMPT.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SINGLE_SOURCE_OF_TRUTH_CONTEXT_PLAN.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md
```

Do not read legacy docs.

Do not ingest raw task outputs as truth.

---

## Required Output

Use Polish.

Structure output exactly like this:

```text
# T-056-GEMINI — Vertex RAG Bridge Design

## 1. Verdict

## 2. Storage Boundary

## 3. FileStore vs VertexRetrieverStore vs HybridStore

## 4. Source Scope Enforcement

## 5. What Vertex May Index

## 6. What Vertex Must Never Index

## 7. Tool Contract Stability

## 8. Sync / Ingestion Pipeline

## 9. Golden Tests For Vertex RAG

## 10. Rollout Plan

## 11. Open Risks

## 12. Final Self-Check
```

---

## Hard Rules

- Vertex RAG must be future backend only, not Sprint 1 requirement.
- MCP tools must keep the same names and output contracts.
- Vertex must not ingest legacy docs.
- Vertex must not ingest `tasks/` raw outputs.
- Vertex must not ingest handoff docs.
- Vertex must not become `search_everything`.
- Vertex must preserve `source_scope = v0_only`.
- Vertex must preserve `legacy_allowed = false`.
- If implementation truth is needed, answer `repo_unverified`, not inferred.
