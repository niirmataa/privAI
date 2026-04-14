# bootstrap-v4

## Purpose

Expanded supervised tuning bundle for `pq-crypto-pro` focused on proof-vs-ledger discipline, exact error mapping, accept/reject balance, multi-error priority, and crypto semantics specific to privAI.

## Sources

- `p1-transfer-proof-core`
- `p2-structural-proof-verifier`
- `p3-escrow-ledger-auth`
- `p4-batch-execution-bundle`
- `p5-escrow-auth-edge-cases`
- `p6-fullprivacy-auth-validation`
- `p7-block-validation-core`
- `p8-transaction-validation-core`
- `p9-pq-crypto-foundations-for-privai`
- `p10-crypto-bindings-and-auth-semantics`

## Counts

- Train: 150
- Val: 38

## Boundaries

- Proof-only examples stay proof-only.
- Ledger/auth-only examples stay ledger/auth-only.
- Crypto examples focus on privAI semantics, not generic textbook exposition.
- Exact error variants are preferred over descriptive prose.
- Accept cases and multi-error priority cases are intentionally included.

## Not included yet

- writer/revision loop packs
- review-feedback protocol packs
- transport/native PQ internals beyond current semantics
- broader blockchain curriculum beyond current validation paths
- RAG-only retrieval behavior
