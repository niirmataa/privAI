# privAI Project Entrypoint

This is the first document an agent or operator should read before opening detailed specs or code.

Goal of this document:
- explain what `privAI` is,
- explain the architectural splits that matter,
- explain what is already real,
- explain what is still open,
- give a safe reading order for the rest of the docs.

## 1. Short Project Description

`privAI` is a privacy-oriented execution system with:
- chain / ledger / node runtime,
- validator networking,
- wallet-side final assembly,
- zero-knowledge proof plumbing,
- FullPrivacy transaction flows,
- escrow flows on top of FullPrivacy,
- NXMS control-plane transport for orchestration messages.

The immediate product goal of the current phase is:

`first honest escrow v1 end-to-end`

Meaning:
- funded escrow exists,
- Stage A proposal/approval flow exists,
- Stage B final assembly exists,
- proof handoff exists,
- node submit gate exists,
- block/import-ready path exists,
- and at least one local end-to-end flow is real, not hand-wavy.

## 2. Core Mental Model

### 2.1. Two planes

#### Control-plane
This is where:
- proposals are created,
- approvals are collected,
- quorum is formed,
- NXMS payloads move,
- orchestration logic lives.

This is not final ledger execution.

#### Execution-plane
This is where:
- the final tx is assembled,
- the final canonical signing context exists,
- proof handoff exists,
- node submit/import logic matters,
- ledger validation becomes authoritative.

### 2.2. Two escrow stages

#### Stage A
- proposal
- approvals
- quorum
- authorization material
- control-plane bundle

Stage A does **not** own the final canonical `tx_signing_hash`.

#### Stage B
- final tx assembly
- final auth insertion
- final `tx_signing_hash`
- proof-ready handoff
- final submit/import shape

If you forget this split, you will reintroduce the main architectural bug that was being fixed across docs, node, wallet, and orchestrator.

## 3. Core Decisions You Must Not Undo

### 3.1. FullPrivacy Option B

Rules:
- all `TransferNoteTx` inputs require auth,
- `policy_opening` is mandatory,
- `policy_tag` is only a hint and must match `policy_opening`.

### 3.2. Escrow is `Escrow2of3`

Roles:
- Buyer
- Merchant
- Operator

Operator is a workflow machine, not a trust anchor.

### 3.3. `nexum-core` is control-plane

It must not pretend to own final tx assembly semantics.

### 3.4. Transport split is real

For v1:
- validator transport = direct session P2P,
- NXMS control-plane = mailbox/Tor store-and-forward,
- gossip = validator session path,
- proof sidecar = mailbox-class path.

Do not collapse validator P2P and NXMS mailbox into one model.

## 4. What Is Already Real

Already delivered and verified:
- Stage A / Stage B architectural cleanup,
- node escrow staging / payload ingress / persistence / bundle export,
- wallet final assembly bridge,
- typed Stage B proof handoff,
- node escrow submit gate,
- validator session regression pack,
- honest local escrow `release` e2e,
- transport/runtime freeze memo.

This means the system is already beyond the "pure architecture sketch" phase.

## 5. What Is Still Open

Main open runtime blocker:
- mailbox runtime loop in `privai-node`

Important additional open work:
- honest escrow `refund` e2e,
- honest escrow `recovery_release` e2e,
- recovery timeout enforcement path,
- runtime retry/ack semantics,
- product-readiness / truthful status docs,
- operational hardening and observability.

## 6. Recommended Reading Order

Read in this order:

1. `PRIVAI_PROJECT_ENTRYPOINT.md`
2. `PRIVAI_V1_READINESS_AND_GAPS.md`
3. `PRIVAI_NEXT_DIRECTION.md`
4. `PRIVAI_DOCS_INDEX.md`
5. `PRIVAI_CHAT_ARCHIVE_2026-04-09.md`
6. Detailed specs and then code

This order is important: first understand the system, then the status, then the direction, then the deep details.

## 7. Environment Warning

Do not treat every WSL failure as proof that the code is broken.

Known environment issues from the recent work:
- broken WSL user mapping for `nxms-privAI`,
- wrong system cargo (`1.83.0`) when running as root,
- project expects the user toolchain (`1.94.1`).

So test failures must be separated into:
- actual code signal,
- environment/toolchain noise.

## 8. One-Line Status

`privAI` already has a coherent escrow Stage A model, a coherent Stage B assembly/proof handoff, a node submit gate, validator transport hardening, and a real local release e2e; the biggest remaining runtime gap is mailbox pull/ingest/ack integration.
