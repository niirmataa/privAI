# bootstrap-v1

## Purpose

First supervised tuning bundle for privAI proof-vs-ledger discipline.

## Sources

- `p1-transfer-proof-core`
- `p3-escrow-ledger-auth`

## Counts

- Train: 14
- Val: 4

## Boundaries

- Proof-only examples stay proof-only.
- Ledger/auth-only examples stay ledger/auth-only.
- No transport/native PQ semantics included yet.

## Not included yet

- `nxms-transport`
- Falcon/native bridge
- broader proof-bundle coverage
- validator session transport
