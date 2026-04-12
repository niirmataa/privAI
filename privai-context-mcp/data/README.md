# Data Manifests

These files are local ingestion manifests and golden-test fixtures.

They are not the production runtime source of truth. Production retrieval should use the approved Vertex RAG corpus populated only from V0 records.

Required manifest files:

- `v0_master.json`
- `v0_direction.json`
- `v0_control.json`
- `v0_future_spec.json`

Every record must include:

```text
source_scope = v0_only
legacy_allowed = false
source_path under /home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE
```
