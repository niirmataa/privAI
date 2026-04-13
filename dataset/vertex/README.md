# Vertex datasets (JSONL)

This folder contains Vertex-style JSONL training/validation packs.

## Packs

### `p1-transfer-proof-core`

- Scope: **proof-only** checks for Transfer proof builder inputs.
- Boundary: proof semantics only (no ledger auth semantics).

### `p3-escrow-ledger-auth`

- Scope: **ledger/auth-only** validation for escrow authorization.
- Boundary: ledger validation only (no proof semantics).

## Expected JSONL structure

Each line is one JSON object with:

- `contents`: array of message objects
  - `role`: `"user"` or `"model"`
  - `parts`: array of parts
    - `text`: string

Example shape:

```json
{
  "contents": [
    { "role": "user", "parts": [{ "text": "..." }] },
    { "role": "model", "parts": [{ "text": "..." }] }
  ]
}
```
