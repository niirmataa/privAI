# privai-context-mcp

Local read-only MCP context server scaffold for `privAI` V0 private compute work.

This project is intentionally separate from the V0 docs folder:

```text
/home/nxms-server/privAI/privai-context-mcp
```

The canonical truth source remains:

```text
/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE
```

## Purpose

`privai-context-mcp` is the guarded context layer for agents working on `privAI`.

It should:

- expose exactly eight V0 read-only tools,
- retrieve only approved V0 context,
- use Vertex RAG as the primary retrieval backend,
- use local JSON/JSONL only for ingestion manifests, metadata audits, and golden tests,
- reject legacy/handoff/old marketplace material,
- never execute tasks or edit files.

## Architecture

```text
V0 markdown docs
  -> ingestion/chunking/metadata
  -> data/*.json manifests
  -> approved Vertex RAG corpus
  -> VertexRagStore
  -> MCP tool guardrails
  -> agent
```

The server must fail closed when retrieval results are missing required metadata.

Every returned knowledge item must satisfy:

```text
source_scope = v0_only
legacy_allowed = false
source_path starts with /home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE
```

## Tools

The v1 contract allows exactly these tools:

- `privai_v0_get_reading_order`
- `privai_v0_get_current_status`
- `privai_v0_lookup_direction`
- `privai_v0_lookup_control`
- `privai_v0_get_guardrails`
- `privai_v0_route_question`
- `privai_v0_prepare_task_context`
- `privai_v0_build_correction_pill`

No write tools, execution tools, legacy lookup tools, or repo-wide search tools are allowed.

## Local Commands

```bash
cd /home/nxms-server/privAI/privai-context-mcp
cargo build
cargo test
cargo run -- --help
```

## Vertex Configuration

Runtime configuration is read from CLI flags or environment variables:

```text
PRIVAI_V0_ROOT=/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE
VERTEX_PROJECT_ID=<project>
VERTEX_LOCATION=<location>
VERTEX_RAG_CORPUS=<corpus_resource_name>
```

Sprint 1 should wire a real MCP Rust SDK for `stdio` transport and a real Vertex RAG retrieval client.

This scaffold deliberately does not hand-roll raw stdin JSON parsing.

## Source Policy

Index only approved `.md` files from the V0 folder.

Do not index:

- legacy docs outside the V0 folder,
- `TASK_*_WORKING_CONTEXT.md`,
- uploaded copies,
- old root logs,
- text files masquerading as PDFs,
- code files,
- existing app/runtime crates.

## Status

This is a project scaffold and contract boundary. It is not a V0 protocol implementation and does not claim any app/runtime behavior.
