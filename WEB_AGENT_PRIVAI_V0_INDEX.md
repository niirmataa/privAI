# WEB AGENT privAI V0 Index

**Status:** web-access mirror index
**Branch:** `privAI-PRIVATE_COMPUTE`
**Canonical source:** `spec/PRIVAI_V0_PRIVATE_COMPUTE/`
**Reason:** some web agents fail to open the `spec/PRIVAI_V0_PRIVATE_COMPUTE/` folder through GitHub, even when the rest of the repo works.

This root-level index exists only to give web agents simple raw links outside the problematic folder path.

Do not treat these `WEB_AGENT_*` files as canonical docs.

Canonical docs remain in:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/
```

---

## How To Use

Give web agents raw links to these root-level files.

Do not give them GitHub UI links such as:

```text
https://github.com/.../blob/...
https://github.com/.../tree/...
```

Use:

```text
https://raw.githubusercontent.com/...
```

---

## Core Context Files

### V0 Product Direction

```text
https://raw.githubusercontent.com/niirmataa/privAI/privAI-PRIVATE_COMPUTE/WEB_AGENT_PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md
```

### V0 Docs Tree

```text
https://raw.githubusercontent.com/niirmataa/privAI/privAI-PRIVATE_COMPUTE/WEB_AGENT_PRIVAI_V0_DOCS_TREE.md
```

### V0 Context MCP Direction

```text
https://raw.githubusercontent.com/niirmataa/privAI/privAI-PRIVATE_COMPUTE/WEB_AGENT_PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md
```

### V0 Single Source Of Truth Context Plan

```text
https://raw.githubusercontent.com/niirmataa/privAI/privAI-PRIVATE_COMPUTE/WEB_AGENT_PRIVAI_V0_SINGLE_SOURCE_OF_TRUTH_CONTEXT_PLAN.md
```

---

## Task Context Files

### T-046 Xiaomi Output

```text
https://raw.githubusercontent.com/niirmataa/privAI/privAI-PRIVATE_COMPUTE/WEB_AGENT_T046_OUTPUT_XIAOMI.md
```

### T-046 Codex Review

```text
https://raw.githubusercontent.com/niirmataa/privAI/privAI-PRIVATE_COMPUTE/WEB_AGENT_T046_REVIEW_CODEX.md
```

---

## Active Prompt Files

### T-053 Gemini Decision Matrix Cross-Review

```text
https://raw.githubusercontent.com/niirmataa/privAI/privAI-PRIVATE_COMPUTE/WEB_AGENT_T053_GEMINI_DECISION_MATRIX_REVIEW_PROMPT.md
```

### T-055 Gemini privai-context-mcp Sprint 1

```text
https://raw.githubusercontent.com/niirmataa/privAI/privAI-PRIVATE_COMPUTE/WEB_AGENT_T055_GEMINI_CONTEXT_MCP_SPRINT1_PROMPT.md
```

### T-056 Gemini Vertex RAG Bridge Design

```text
https://raw.githubusercontent.com/niirmataa/privAI/privAI-PRIVATE_COMPUTE/WEB_AGENT_T056_GEMINI_VERTEX_RAG_BRIDGE_PROMPT.md
```

---

## Guardrails

- Use branch exactly: `privAI-PRIVATE_COMPUTE`.
- Use raw files, not GitHub HTML pages.
- Do not use `main`.
- Do not use legacy docs.
- Do not infer implementation facts from V0 direction docs.
- If implementation truth is needed, read code directly or answer `repo_unverified`.
