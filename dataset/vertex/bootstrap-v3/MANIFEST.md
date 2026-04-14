# bootstrap-v3

## Purpose

Final supervised tuning bundle for `pq-crypto-pro` focused on proof-vs-ledger discipline, exact error mapping, and colder code-review behavior on current privAI semantics.

## Sources

- `p1-transfer-proof-core`
- `p2-structural-proof-verifier`
- `p3-escrow-ledger-auth`
- `p4-batch-execution-bundle`
- `p5-escrow-auth-edge-cases`
- `p6-fullprivacy-auth-validation`

## Counts

- Train: 50
- Val: 14

## Boundaries

- Proof-only examples stay proof-only.
- Ledger/auth-only examples stay ledger/auth-only.
- Exact error variants are preferred over descriptive prose.
- No transport/native PQ semantics are included yet.

## Not included yet

- `nxms-transport`
- Falcon/native bridge internals
- broader proof-bundle coverage beyond current packs
- validator session transport
- RAG-only retrieval behavior
