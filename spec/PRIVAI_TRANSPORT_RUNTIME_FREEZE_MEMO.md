# privAI Transport / Runtime Freeze Memo

Status: architectural analysis memo — current-state map, drift list, v1 freeze recommendation, and implementation backlog for the transport/runtime layer.
Canonicality: non-overriding analysis document. This memo does not create new canonical semantics; it maps what exists, identifies drift, recommends a v1 freeze point, and proposes an ordered backlog. Freeze decisions require explicit acceptance via `PRIVAI_DECISION_REGISTER.md`.
Owner: privAI networking, transport and runtime architecture.
Depends on:
- `spec/PRIVAI_EXECUTION_SPINE.md`
- `spec/PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`
- `spec/PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`
- `spec/PRIVAI_ESCROW_OBJECT_MODEL.md`
- `spec/PRIVAI_ESCROW_FULLPRIVACY_BOUNDARY_DECISION_MEMO.md`
- `nexum-core/README.md`
- `nexum-core/crates/nxms-mailbox/README.md`

## 1. Purpose

This memo answers one question: **what is the actual transport/runtime architecture today, and what should the v1 freeze look like?**

Two communication models appear across the repo:

- **Model A — Validator P2P:** direct session transport, PQC handshake, encrypted frames, `ConnectionPool`, gossip fanout over Tor.
- **Model B — NXMS Control-plane:** `NxmsEnvelope` / `NxmsPayloadV2`, `SealedPacket`, mailbox store-and-forward over Tor onion services, `nxms-mailbox-client`.

This memo maps code and docs to each model, identifies where they drift or overlap, and proposes one coherent v1 freeze.

---

## 2. Current-State Map

### 2.1. Validator Session Path (Model A — Direct P2P)

| Component | Location | Status |
|-----------|----------|--------|
| Handshake (Falcon sig + FrodoKEM) | `privai-node/src/session_impl.rs` | **Working code.** Challenge-response, transcript binding, shared secret derivation. |
| Encrypted frame transport | `privai-node/src/session_impl.rs` | **Working code.** XChaCha20-Poly1305 with seq-AAD. |
| Connection pool (outgoing) | `privai-node/src/session_impl.rs` | **Working code.** Actor-style writer tasks, bounded MPSC queues, stale/idle rebuild, maintenance tick. |
| Listener (incoming) | `privai-node/src/session_impl.rs` | **Working code.** Rate limiting, ban list, pressure guard, handshake cooldown. |
| Facade API | `privai-node/src/session_transport.rs` (`ValidatorSessionTransport`) | **Working code.** `send_message`, `broadcast_message`, `spawn_listener`, `spawn_maintenance`. Placeholder key guard. |
| Compatibility shim | `privai-node/src/net.rs` | Thin re-export of session_impl types. |
| Gossip | `privai-node/src/gossip.rs` | **Working code.** Tx propagation via `ValidatorSessionTransport`. Fanout=3, max hops=5, Falcon sig verification at entry. |
| Consensus overlay | `privai-node/src/consensus_loop.rs` | Uses `ValidatorSessionTransport` for Proposal/Vote/QC/ViewChange/Sync. |
| State sync | `privai-node/src/state_sync.rs` | Uses `ValidatorSessionTransport`. |
| Tor connectivity | `nxms-transport/src/tor_net.rs` | Shared. SOCKS5h connect, framed TCP read/write. |
| PQC crypto primitives | `nxms-transport/src/crypto.rs` | Shared. Falcon sign/verify, FrodoKEM encap/decap, XChaCha20-Poly1305. |
| Peer registry | `nxms-transport/src/peers.rs` | Shared. `Peer`, `PeerBook` (allowlist model). |

**Summary:** Validator P2P is real, working code with a well-defined boundary described in `PRIVAI_TRANSPORT_AND_P2P_SPLIT.md` and `PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`. It is the most mature transport path in the repo.

### 2.2. Control-Plane / NXMS Path (Model B — Mailbox / Store-and-Forward)

| Component | Location | Status |
|-----------|----------|--------|
| Wire format: `NxmsEnvelope`, `NxmsEnvelopeV2`, `NxmsPayloadV2` | `nxms-transport/src/wire.rs` | **Defined types.** Protocol versions `NXMS/1`, `NXMS/2`. |
| Packet crypto: `SealedPacket` | `nxms-transport/src/crypto.rs` | **Working code.** Encrypt/decrypt for envelope payloads. |
| Mailbox server (store-and-forward) | `nexum-core/crates/nxms-mailbox/` | **Working code.** HTTP API, Tor onion service, pull/push/ack, lease semantics, per-inbox scoping. |
| Mailbox client | `nexum-core/crates/nxms-mailbox-client/src/lib.rs` | **Working code.** `push`, `pull`, `ack`, `health`. Tor proxy support. |
| Escrow orchestrator | `nexum-core/crates/nxms-escrow-orchestrator/` | **Exists** in nexum-core. Workflow engine for escrow lifecycle. |
| privAI application bodies | `privai-nxms/src/lib.rs` (`PrivaiBody`) | **Working code.** 15 body variants (BundleOffer, MarketOffer, EscrowFunded, EscrowApproval, ProofServiceRequest, etc.). Serialization to `NxmsPayloadV2`. |
| Node ingress for NXMS payloads | `privai-node/src/node.rs` (`handle_nxms_payload`) | **Working code.** Decodes `NxmsPayloadV2` -> `PrivaiBody`, routes escrow bodies to `EscrowStageStore`. |

**Summary:** The NXMS control-plane has working wire formats, mailbox infrastructure, and a client library. `privai-node` has a payload ingress path for escrow events. However, the **glue** between mailbox delivery and `privai-node` ingestion is not yet wired end-to-end in runtime code — `handle_nxms_payload` exists but there is no running loop that pulls from mailbox and feeds it.

### 2.3. Gossip Path

| Aspect | Detail |
|--------|--------|
| Transport | Uses `ValidatorSessionTransport` (Model A). |
| Scope | Tx propagation between validators only. |
| Policy | Fanout=3, max hops=5, Falcon sig check, mempool dedup, per-sender rate limit. |
| Code | `privai-node/src/gossip.rs` |
| Status | **Working code**, well-scoped. |

Gossip is **validator-only** and rides on Model A. No drift.

### 2.4. Proof Artifact Sidecar Path

| Aspect | Detail |
|--------|--------|
| Bodies defined | `ProofServiceRequestBody`, `ProofServiceResponseBody` in `privai-nxms/src/lib.rs`. |
| Transport | Intended to use `NxmsPayloadV2` (Model B). |
| Runtime glue | **Not wired.** Node has `handle_nxms_payload` but it returns `Ignored` for proof bodies. |
| Status | **Schema only**, no runtime integration. |

### 2.5. Marketplace / Inference Path

| Aspect | Detail |
|--------|--------|
| Bodies defined | `MarketOfferBody`, `MarketAcceptBody`, `InferenceRequestBody`, `InferenceResponseBody` in `privai-nxms/src/lib.rs`. |
| Transport | Intended to use `NxmsPayloadV2` (Model B). |
| Runtime glue | **Not wired** beyond type definitions. |
| Status | **Schema only.** |

---

## 3. Drift / Ambiguity List

### DRIFT-01: No runtime mailbox-to-node loop

**Location:** `privai-node/src/node.rs:497` (`handle_nxms_payload`) + `nxms-mailbox-client`.

**Problem:** `handle_nxms_payload` can decode and ingest escrow payloads, and `nxms-mailbox-client` can pull from a mailbox server. But there is no running async loop or daemon path in `privai-node` that connects the two. The escrow control-plane ingress path is structurally complete but not runtime-wired.

**Risk:** A future developer/agent might build an ad-hoc polling loop that duplicates the mailbox client, ignores lease/ack semantics, or mishandles replay protection.

### DRIFT-02: `nxms-transport` crate mixes two distinct concerns

**Location:** `nexum-core/crates/nxms-transport/src/` — `wire.rs` (escrow envelope types), `crypto.rs` (shared PQC + `SealedPacket`), `tor_net.rs` (shared Tor helpers), `peers.rs` (shared peer types).

**Problem:** `nxms-transport` is consumed by both validator P2P (via `tor_net`, `peers`, `crypto` primitives) and escrow control-plane (via `wire`, `SealedPacket`). The crate name suggests "the transport", but it is really "shared crypto/net primitives + escrow wire types". This is already identified in `PRIVAI_TRANSPORT_AND_P2P_SPLIT.md` §3.1 and §8.2, but no code action has been taken.

**Risk:** New contributors may assume `nxms-transport` = the transport layer, and try to route validator messages through `NxmsEnvelope` or `SealedPacket`.

### DRIFT-03: nexum-core README describes a different system model

**Location:** `nexum-core/README.md`

**Problem:** The nexum-core README describes a Monero-based auto-multisig escrow system with `nxms-monero-core`, `nxms-signer`, and `nxms-escrow-orchestrator` as core components. privAI's escrow model is note-based FullPrivacy 2-of-3 with Falcon/FrodoKEM. The nexum-core README mentions `nxms-transport` as "the only canonical wire format" and `nxms-mailbox` as "the only relay/store-and-forward" — statements that are correct for the NXMS control-plane but could be misread as applying to validator P2P.

**Risk:** An agent or dev reading nexum-core's README in isolation will think `nxms-transport` wire format is the system-wide canonical wire format, including for validators. The TRANSPORT_AND_P2P_SPLIT doc explicitly freezes that this is NOT the case.

### DRIFT-04: Proof sidecar bodies defined but unrouted

**Location:** `privai-nxms/src/lib.rs:302-317` (`ProofServiceRequestBody`, `ProofServiceResponseBody`).

**Problem:** Proof delegation bodies exist in `PrivaiBody` but `handle_nxms_payload` -> `handle_privai_body` returns `Ignored` for them. No decision exists on whether proof artifact exchange goes over:
- Model A (validator session, direct P2P), or
- Model B (NXMS mailbox, store-and-forward), or
- a separate sidecar channel.

**Risk:** Future implementation may route proof artifacts through the wrong transport, e.g., large proof blobs over the bounded validator session queue (64-message capacity, 1 MiB frame limit).

### DRIFT-05: Escrow event delivery model is ambiguous

**Location:** `PRIVAI_ESCROW_FULLPRIVACY_BOUNDARY_DECISION_MEMO.md` §13 describes operator receiving `EscrowFundingDescriptor` via "nxms-transport / mailbox". `PRIVAI_ESCROW_OBJECT_MODEL.md` §4.4 says "Descriptor jest przekazywany do buyer wallet" but does not specify transport.

**Problem:** The spec says "event goes through nxms-transport / mailbox" but the node code has no mailbox pull loop. The actual delivery mechanism is undefined at runtime level.

**Risk:** Escrow events might be hand-delivered via test harness or ad-hoc calls, creating a gap between spec and production runtime.

### DRIFT-06: Validator session layer not yet extracted

**Location:** `privai-node/src/session_impl.rs`, `privai-node/src/net.rs`, `privai-node/src/session_transport.rs`.

**Problem:** `PRIVAI_TRANSPORT_AND_P2P_SPLIT.md` §4.3 designates a future `privai-p2p` crate as the home for the validator session layer. The code is still in `privai-node`. The spec calls this "in progress" (§8.2, non-conformity #1).

**Risk:** Low near-term risk (code works), but the longer extraction is deferred, the more consensus-layer dependencies leak into session internals.

### DRIFT-07: `privai-nxms` vs `nxms-transport` naming confusion

**Location:** `privai-nxms/src/lib.rs` (privAI application protocol) vs `nexum-core/crates/nxms-transport/` (wire + crypto).

**Problem:** `privai-nxms` defines `PrivaiBody` application-level messages that wrap into `NxmsPayloadV2` from `nxms-transport`. The naming suggests `privai-nxms` IS nxms, when it's actually the privAI application layer ON TOP of the nxms wire layer. This is a naming/conceptual confusion, not a code bug.

**Risk:** Developers may add transport-level code to `privai-nxms` or application-level code to `nxms-transport`.

---

## 4. Recommended v1 Freeze

### 4.1. Validator P2P = Session Transport (Model A)

**Freeze:** Validators communicate via direct P2P sessions as implemented today.

- Handshake: Falcon + FrodoKEM challenge-response (as in `session_impl.rs`).
- Frames: XChaCha20-Poly1305 encrypted, seq-AAD, JSON-serialized `ConsensusMsg`.
- Pool: `ConnectionPool` with bounded queues, stale rebuild, lazy reconnect.
- Admission: `PeerBook` allowlist, `BanList`, rate limiter, pressure guard.
- Gossip: rides on the same session transport (fanout=3, hops≤5).
- State sync: rides on the same session transport.

**Non-goal for v1:** Extracting to `privai-p2p` crate. The current in-node location is acceptable for v1 as long as the `ValidatorSessionTransport` facade API remains the only entry point for consensus/gossip/sync.

### 4.2. NXMS Control-Plane = Mailbox-first over Tor (Model B)

**Freeze:** All escrow/marketplace/inference control-plane messaging goes through `NxmsPayloadV2` + `SealedPacket` + `nxms-mailbox` store-and-forward over Tor onion services.

- Wire: `NxmsPayloadV2` with `app_proto = "PRIVAI/1"`.
- Crypto: `SealedPacket` envelope encryption (end-to-end over mailbox relay).
- Relay: `nxms-mailbox` HTTP API (push/pull/ack), Tor hidden service.
- Client: `nxms-mailbox-client` with pull-lease-ack semantics.
- Application bodies: `PrivaiBody` variants in `privai-nxms`.
- Anti-replay: `(escrow_id, from, seq)` dedup per NxmsEnvelope.
- Node ingress: `handle_nxms_payload` decodes and routes.

**v1 requirement:** Build the missing mailbox pull loop in `privai-node` runtime (see Backlog T-03).

### 4.3. Proof Artifact Sidecar = NXMS Mailbox (Model B)

**Freeze:** Proof service request/response bodies already exist in `PrivaiBody`. They should use the same NXMS mailbox path as escrow events.

**Rationale:** Proof artifacts can be large (witness, proof bytes) and asynchronous. Store-and-forward fits better than the bounded validator session queue (64 slots, 1 MiB frame limit). Validators do not need proof artifacts in real-time — they verify proof certificates attached to blocks, not raw proof generation traffic.

### 4.4. Gossip = Validator Session Transport (Model A)

**Freeze:** Tx gossip remains on Model A. No change needed.

### 4.5. Summary Table

| Traffic class | Transport model | Wire format | Relay |
|---------------|-----------------|-------------|-------|
| Consensus (Proposal/Vote/QC/ViewChange) | Model A — direct P2P session | `ConsensusMsg` JSON + encrypted frame | None (direct) |
| Tx gossip | Model A — direct P2P session | `GossipTxMsg` via `ConsensusMsg` | None (direct) |
| State sync | Model A — direct P2P session | Sync request/response via `ConsensusMsg` | None (direct) |
| Escrow events (funded/proposal/approval/resolve) | Model B — NXMS mailbox | `NxmsPayloadV2` + `SealedPacket` | `nxms-mailbox` (Tor) |
| Marketplace events (offer/accept/inference) | Model B — NXMS mailbox | `NxmsPayloadV2` + `SealedPacket` | `nxms-mailbox` (Tor) |
| Proof service (request/response) | Model B — NXMS mailbox | `NxmsPayloadV2` + `SealedPacket` | `nxms-mailbox` (Tor) |
| Bundle exchange (offer/request/delivery) | Model B — NXMS mailbox | `NxmsPayloadV2` + `SealedPacket` | `nxms-mailbox` (Tor) |

---

## 5. Explicit Non-Goals

### 5.1. Do NOT mix validator P2P with NXMS control-plane

Validator session transport (`HandshakeMsg`, `ConnectionPool`, encrypted frames) and NXMS envelope transport (`NxmsPayloadV2`, `SealedPacket`, mailbox) are two independent paths. They share low-level primitives (Falcon, FrodoKEM, Tor connectivity) but MUST NOT share wire format, session lifecycle, or relay infrastructure.

### 5.2. Do NOT conflate escrow Stage A/B with validator session transport

Escrow Stage A (control-plane/proposal) and Stage B (wallet/prover/final assembly) are application-layer concepts. They ride on the NXMS mailbox transport (Model B). The validator session transport (Model A) only carries the resulting `TransferNoteTx` via gossip/consensus after Stage B completes.

### 5.3. Do NOT use mailbox relay for consensus networking

Consensus messages (Proposal, Vote, QC, ViewChange) require low-latency direct delivery. The mailbox's pull-lease-ack model introduces unacceptable latency for consensus liveness.

### 5.4. Do NOT redesign the crypto stack

Both models use the same PQC primitives (Falcon, FrodoKEM, XChaCha20-Poly1305). This memo does not propose new cryptographic assumptions or algorithms.

### 5.5. Do NOT extract `privai-p2p` crate in v1

The session layer extraction to a separate crate is a desirable future step but is not a v1 blocker. The `ValidatorSessionTransport` facade already provides the right API boundary.

---

## 6. Implementation Backlog

Tasks are ordered by dependency. Each task notes whether it is docs/spec-level or code/runtime-level.

### T-01: Register v1 transport freeze in Decision Register
**Type:** docs
**Description:** Add entries in `PRIVAI_DECISION_REGISTER.md` recording the v1 freeze decisions from this memo (§4.1–4.4). Cross-reference this memo.
**Dependency:** None.

### T-02: Update TRANSPORT_AND_P2P_SPLIT with v1 freeze status
**Type:** docs
**Description:** Add a section to `PRIVAI_TRANSPORT_AND_P2P_SPLIT.md` marking which checkpoints (§9) are now frozen vs still in-progress, reflecting v1 freeze. Update checkpoint 0 as accepted if the freeze is approved.
**Dependency:** T-01.

### T-03: Build mailbox pull loop in privai-node runtime
**Type:** code/runtime
**Description:** Implement an async task in `privai-node` that: (1) uses `nxms-mailbox-client` to pull from a configured mailbox endpoint, (2) decodes `NxmsPayloadV2` via `handle_nxms_payload`, (3) handles lease/ack after successful processing, (4) respects anti-replay dedup. This closes DRIFT-01.
**Dependency:** T-01 (freeze decision accepted).

### T-04: Route proof sidecar bodies through node ingress
**Type:** code/runtime
**Description:** Extend `handle_privai_body` in `privai-node/src/node.rs` to handle `ProofServiceRequest` and `ProofServiceResponse` bodies instead of returning `Ignored`. Define where proof artifacts are stored/forwarded. This closes DRIFT-04.
**Dependency:** T-03 (mailbox loop exists).

### T-05: Add cross-reference warning to nexum-core README
**Type:** docs
**Description:** Add a short note in `nexum-core/README.md` clarifying that `nxms-transport` wire format and `nxms-mailbox` are the escrow/control-plane transport stack, NOT the validator consensus wire protocol. Reference `PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`. This mitigates DRIFT-03.
**Dependency:** None (can run in parallel with T-01).

### T-06: Specify escrow event delivery contract
**Type:** docs
**Description:** Write a short section in `PRIVAI_ESCROW_OBJECT_MODEL.md` or a new focused doc specifying how each escrow object (`EscrowFundingDescriptor`, `EscrowSpendProposal`, `EscrowApprovalBody`) is delivered at runtime: which actor pushes it to mailbox, which actor pulls it, what the ack/retry semantics are. This closes DRIFT-05.
**Dependency:** T-01 (freeze decision).

### T-07: Validator session test regression pack
**Type:** code/test
**Description:** Implement the test cases listed in `PRIVAI_VALIDATOR_SESSION_INVARIANTS.md` §19: handshake success, version reject, unknown peer reject, Falcon sig reject, ban list reject, rate limit reject, stale rebuild, queue timeout. These tests confirm the v1 session transport baseline.
**Dependency:** None (can run in parallel).

### T-08: Validate NXMS payload ingress end-to-end
**Type:** code/test
**Description:** Write an integration test that: (1) pushes an `EscrowFundedBody` as `NxmsPayloadV2` to a test mailbox, (2) pulls it via `nxms-mailbox-client`, (3) feeds it to `handle_nxms_payload`, (4) verifies `EscrowStageStore` state. This validates the full Model B path.
**Dependency:** T-03.

### T-09: Clean naming boundary between privai-nxms and nxms-transport
**Type:** docs
**Description:** Add module-level doc comments to `privai-nxms/src/lib.rs` and `nxms-transport/src/lib.rs` clarifying the boundary: `nxms-transport` = wire + crypto + network primitives; `privai-nxms` = privAI application protocol bodies on top. This mitigates DRIFT-07.
**Dependency:** None.

### T-10: (Future) Extract validator session layer to privai-p2p
**Type:** code/refactor
**Description:** Extract `session_impl.rs`, `session_transport.rs`, and supporting types from `privai-node` into a new `privai-p2p` crate per `PRIVAI_TRANSPORT_AND_P2P_SPLIT.md` §10. This is a post-v1 task.
**Dependency:** T-07 (tests exist as safety net).

---

## 7. Backlog Summary

| # | Task | Type | Priority | Parallel? |
|---|------|------|----------|-----------|
| T-01 | Register v1 freeze in Decision Register | docs | **P0** | — |
| T-02 | Update TRANSPORT_AND_P2P_SPLIT | docs | P0 | After T-01 |
| T-03 | Mailbox pull loop in privai-node | code | **P0** | After T-01 |
| T-04 | Route proof sidecar bodies | code | P1 | After T-03 |
| T-05 | nexum-core README cross-reference | docs | P1 | Parallel |
| T-06 | Escrow event delivery contract | docs | P1 | After T-01 |
| T-07 | Validator session test pack | code/test | P1 | Parallel |
| T-08 | NXMS ingress e2e test | code/test | P2 | After T-03 |
| T-09 | Naming boundary docs | docs | P2 | Parallel |
| T-10 | Extract privai-p2p crate | code/refactor | Post-v1 | After T-07 |

---

## 8. Checklist

- [ ] v1 transport freeze recorded in Decision Register (T-01)
- [ ] TRANSPORT_AND_P2P_SPLIT reflects freeze (T-02)
- [ ] Mailbox pull loop exists in privai-node runtime (T-03)
- [ ] Proof sidecar bodies are routed, not ignored (T-04)
- [ ] nexum-core README clarifies scope (T-05)
- [ ] Escrow event delivery contract is specified (T-06)
- [ ] Validator session regression tests exist (T-07)
- [ ] End-to-end NXMS ingress test exists (T-08)
- [ ] Module-level boundary docs added (T-09)

## 9. Exit Criteria

This memo's analysis is complete when:
- current-state map covers all five paths (validator, control-plane, gossip, proof sidecar, marketplace),
- drift list names concrete files and line numbers,
- v1 freeze recommendation is a single coherent model,
- non-goals are explicit,
- backlog is ordered and typed.

Implementation of the backlog tasks is tracked separately.
