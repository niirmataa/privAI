# privAI v1 Readiness and Gaps

This document is the truth map for current project state.

Use it after reading `PRIVAI_PROJECT_ENTRYPOINT.md`.

## 1. Ready / Verified

### Ledger / privacy model
- `FullPrivacy + Option B` is the working model.
- Hidden-value / proving-oriented transaction shape is part of the real design, not placeholder prose.

### Stage A / Stage B boundary
- Stage A and Stage B were explicitly separated.
- `nexum-core` no longer claims ownership of final canonical Stage B signing context.
- `EscrowApprovalBundle` is treated as Stage A authorization material, not final on-chain auth.

### Node escrow runtime
- funded/proposal/approval staging exists,
- quorum exists,
- persistence exists,
- `handle_nxms_payload(...)` exists,
- approval bundle export exists,
- escrow-aware submit gate exists.

### Wallet / proof bridge
- wallet final assembly bridge exists,
- amount narrowing fixes were applied safely,
- typed Stage B proof handoff exists,
- proof attachment step exists.

### Transport / validator path
- validator session transport regression pack exists,
- validator P2P and mailbox control-plane are explicitly separated in docs.

### E2E
- honest local escrow `release` e2e exists and passes.

## 2. Partially Ready

### Proof runtime
- proof architecture is real,
- proving data / proof jobs / artifacts are real,
- proof handoff is real,
- but full production-style prover runtime/orchestration should still be described carefully and not overclaimed.

### Escrow product surface
- escrow `release` path is locally honest,
- but `refund` and `recovery_release` still need equivalent confidence.

### Runtime transport
- transport split is decided,
- validator path is hardened,
- mailbox runtime path has partial implementation work,
- but mailbox ingest loop is not yet treated as fully closed.

## 3. Open Gaps

### Highest-priority gap
- mailbox runtime loop in `privai-node`

What that means:
- mailbox pull,
- payload decode,
- `handle_nxms_payload(...)`,
- ack policy,
- retry/error behavior.

### Missing escrow path coverage
- honest local escrow `refund` e2e
- honest local escrow `recovery_release` e2e
- recovery timeout enforcement path

### Hardening gaps
- richer observability / metrics,
- stricter runtime retry/duplicate/delayed-ack semantics,
- restart/reload tests for more than Stage A persistence,
- golden vectors for escrow artifacts and final tx shapes.

## 4. Safe Claims

These are safe to say:
- `privAI` is already a real multi-layer system, not just docs.
- It has chain / ledger / node / wallet / proof / transport layers.
- It has a coherent escrow model on FullPrivacy.
- It has a real local `release` escrow e2e.
- It has validator transport hardening.

## 5. Unsafe Claims

These should not be stated carelessly:
- "full production prover runtime is finished"
- "all escrow flows are equally done"
- "mailbox runtime is fully operational"
- "everything around privacy/proof is fully frozen"
- "all runtime/operational failure modes are already hardened"

## 6. Status Table

| Area | Status | Notes |
|---|---|---|
| FullPrivacy ledger model | Ready | Option B shape is the active truth |
| Escrow object model | Ready | Stage A / Stage B split is documented |
| Node Stage A runtime | Ready | staging, quorum, persistence, bundle export |
| Escrow submit gate | Ready | validates staged context against final tx |
| Wallet Stage B assembly | Ready | real final assembly path |
| Proof handoff | Ready | typed handoff + proof attach |
| Escrow `release` e2e | Ready | honest local path passes |
| Validator session path | Ready | regression pack exists |
| Mailbox runtime loop | Open | biggest runtime blocker |
| Escrow `refund` e2e | Open | next major escrow path |
| Escrow `recovery_release` e2e | Open | timeout-sensitive path |
| Runtime observability | Open | still needs systematic work |

## 7. Bottom Line

`privAI v1` is already in a serious implementation phase, but the final story is still incomplete until mailbox runtime, refund/recovery escrow coverage, and runtime hardening are finished.
