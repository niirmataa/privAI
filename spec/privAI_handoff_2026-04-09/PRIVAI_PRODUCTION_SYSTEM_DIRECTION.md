# privAI Production System Direction

## Core Rule

We are not designing separate products for "v1", "v2", and "v5".
We are designing **one production system** and shipping it incrementally.

That means:
- the rollout can be staged,
- some mechanisms can remain simpler at first,
- but the architectural direction must stay single and explicit.

See also:
- `PRIVAI_PRODUCTION_SYSTEM_DIAGRAMS.md`
- `PRIVAI_TOR_GATED_NETWORK_DIRECTION.md` (detailed topology design note — not a frozen product invariant)

## What We Are Building

`privAI` is moving toward one integrated system with four layers:

1. **Protocol layer**
   - FullPrivacy coin
   - Escrow 2-of-3
   - NXMS transport / mailbox
   - ledger-enforced settlement rules

2. **Workspace layer** (container — knows how to display, not what)
   - project-first environment
   - files, panels, sidebars, terminal, diff viewer, status bar
   - task graph view (workspace renders, skill layer provides data)
   - chat as one view, not the center
   - fully usable offline as a workspace; marketplace and settlement features add online capability

3. **Skill / extension layer** (content — knows how to do work, not how to display)
   - skill packs with declared capability requirements (not hardwired model names)
   - task contracts + verifier packs
   - role-based execution (system-architect, bounded-coder, reviewer — roles, not models)
   - locality declaration: offline-capable / online-required / network-optional

4. **Marketplace layer**
   - discovery
   - contract acceptance (before escrow lock)
   - payment lock (private PVA escrow)
   - execution (local / remote API / sandbox rental / hybrid)
   - delivery (hash-commit + buyer confirmation)
   - verification (mechanical → contractual → semantic → settlement)
   - settlement (private on-chain settlement, rule-bound validation, PQ Falcon signatures)

## UI Direction

The production direction is:
- **protocol-first**
- with **VSCodium / VS Code style extensions as the first home**
- **without a shell fork for now**

Why:
- we avoid rebuilding an editor from scratch,
- we get panels, trees, diffs, terminal and workspace UX immediately,
- we stay closer to the real work environment,
- we can fork later only if the extension model becomes a real blocker.

## Contract Model

The center of the marketplace is not "a good model".
The center is:
- a **task contract**,
- accepted before escrow lock,
- with explicit verification rules,
- and explicit settlement conditions.

The clean rule is:
- provider declares contract and verification conditions,
- buyer accepts or rejects them,
- escrow locks only after acceptance.

That contract is the binding object.

### Scope change = new contract

If scope changes mid-task, there is no mid-task requote mechanism.
The clean path is:
- refund the old escrow (provider signs Refund),
- negotiate a new contract with new scope and new price,
- lock a new escrow.

This preserves the invariant: locked amount is fixed for the lifetime of an escrow.

## Verification Model

Work is verified in four levels:

1. **Mechanical**
   - build, tests, lint, schema, forbidden files untouched

2. **Contract**
   - scope respected, task pill completed, DoD met

3. **Semantic**
   - human or reviewer-level judgment where needed

4. **Settlement**
   - release / refund / recovery

Important:
- proof of delivery is not the same as proof of quality
- the system should never pretend otherwise

### Verification failure paths

- **Level A FAIL** (build/test/lint) → provider can re-submit with new `delivery_hash`. Escrow stays locked. No limit on re-submissions within timeout.
- **Level B FAIL** (scope/DoD not met) → buyer can request fix (provider re-submits) or request Refund.
- **Level C REJECT** (semantic judgment) → Refund if provider agrees (signs Refund). If provider disagrees → timeout → Recovery (peer resolution).
- **Timeout without resolution** → Recovery path (buyer_sig + provider_sig, no operator).

## Delivery Model

The honest first delivery model is:
- provider commits artifact hash,
- artifact is delivered,
- buyer confirms receipt / acceptance,
- escrow settles through release / refund / recovery.

Current proof certificates should be treated as protocol-validity proofs,
not as final semantic proof that a skill was performed correctly.

### Delivery hash is a ledger transaction

Provider commits `delivery_hash` as a **native ledger transaction** (not an off-chain oracle).
Escrow validation reads `delivery_hash` from **ledger state** — same chain, native lookup.
This is not a smart contract reading external state. It is protocol-level validation
reading protocol-level data. No oracle needed.

## On-Chain Privacy Model

`privAI` privacy has two distinct layers and both matter:

1. **Transport privacy**
   - who is talking to whom
   - mailbox / Tor / KEM / multi-hop routing

2. **On-chain privacy**
   - what the transaction contains
   - hidden amount, hidden recipient, hidden policy details

The marketplace contract price is negotiated **off-chain** between buyer and provider
over the encrypted control plane.

The on-chain settlement amount remains **private to observers**:
- amount is carried as a hidden commitment / ciphertext, not as public plaintext
- recipient data is encrypted for the real recipient, not public to validators
- policy details are committed, not exposed as plaintext policy metadata
- validity proofs show correctness without exposing the contract contents

This means:
- **buyer + provider** know the negotiated price and contract details
- **observers / validators / auditors** see that a valid settlement transition happened
- **observers do not learn** plaintext amount, plaintext recipient, or plaintext contract terms

The production rule is:
- marketplace economics are agreed off-chain,
- settlement is enforced on-chain,
- and on-chain settlement must preserve `privAI` privacy guarantees rather than exposing
  marketplace details as public metadata.

## Pricing Model

v1 pricing models (frozen):

Pricing visibility rule:
- quoted price is agreed off-chain between buyer and provider
- on-chain escrow amount remains private to observers

- **per task (fixed)** — bounded tasks with known scope. Escrow locks a privately committed amount.
- **per task (quoted)** — open-ended/research tasks. Provider quotes price, buyer accepts or rejects before escrow lock. No trusted metering needed.
- **per compute unit** — sandbox/VM rental. Metered by wallclock (GPU-hours, VM-hours). Simple, trustless metering.

Deferred pricing models:
- **subscription** — recurring escrow pattern (monthly lock + renewal). Future.
- **per token** — requires trusted metering (who counts tokens?). Not solved in privacy context. Deferred.
- **bounty / per outcome** — variant of per-task with open participation. Optional, no new mechanics needed.

Currency: **PVA**.

## Operator Model

Operator is not "a platform moderator".
Operator is a **system program executing hard rules**.

### Frozen decision: rule-bound operator signature on normal escrow paths

In the production direction, normal escrow settlement still requires operator participation:

- Release: buyer_sig + operator_sig → merchant
- Refund: merchant_sig + operator_sig → buyer
- Recovery: buyer_sig + merchant_sig (after timeout, no operator)

Operator validation checks:
1. Contract hash matches locked contract.
2. Delivery hash was committed by provider.
3. Buyer signed acceptance (for Release) or merchant signed acceptance (for Refund).
4. Timeout constraints respected.
5. All signatures are valid Falcon (PQ).

This means:
- operator is not a discretionary moderator,
- operator is a rule executor,
- and there is no normal Release / Refund without operator signature.

### Operator implementation can evolve without changing escrow semantics

- **Phase 0 (now):** operator = dev-team keypair (centralized). This is honest: early escrow requires a trusted operator. Users must know this.
- **Phase 1-2:** operator = automated service with published rules. Still a keypair, but rule-bound.
- **Later strengthening:** more validation may move deeper into protocol logic, but that is future strengthening, not the current frozen settlement invariant.

This transition is known and intentional. Phase 0 is not "trustless" — it is "honest about trust".

## Dispute Direction

The production direction is:
- no fake centralized moderation layer,
- no pretending disputes are automatically solved by magic,
- clear contract,
- clear timeout,
- release / refund / recovery enforced by rules.

A future direction is:
- dispute quorum of independent providers,
- selected under protocol rules,
- signing verdicts with Falcon,
- possibly later weighted by trust / reputation.

But that is future strengthening, not a blocker for the core direction.

## Online / Offline Direction

Frozen rule: **execution can be fully offline, settlement must be online**.

Workspace engine is fully usable offline as a workspace. Online adds: marketplace, settlement, remote execution.
Offline is never degraded workspace — it is full workspace without marketplace features.

### Execution modes (frozen)

- **Mode A: Local** — user runs model on own hardware. No network. Maximum privacy.
- **Mode B: Remote API** — user sends task to model provider. Connection: NXMS mailbox over Tor.
- **Mode C: Rented compute (sandbox)** — user rents raw compute, deploys own model in sandboxed VM. Provider sees resource usage only, not content.
- **Mode D: Hybrid** — sensitive reasoning local, heavy compute rented, cheap tasks remote. Routing logic in skill contract.

### Sandbox network modes (frozen)

- **ISOLATED (default)** — VM has no internet. User ↔ VM via P2P/Tor tunnel only. User uploads everything through tunnel.
- **TOR-GATED** — VM outgoing routes through privAI multi-hop relays plus Tor before reaching the internet. The frozen production direction is: Tor-gated, multi-hop, provider-blind internet access. Exact hop-count, relay topology, and exit-node policy belong to dedicated network design docs, not to this product-direction doc. Interactive chat should prefer Mode B (direct NXMS). TOR-GATED is primarily for sandbox and batch-style workloads.

### Privacy direction by role (TOR-GATED)

The frozen invariant is: no single participant learns the full route, and the provider is blind to the final destination.

| Role | Directional guarantee |
|------|----------------------|
| Provider | sees encrypted outbound traffic only — cannot determine destination, content, or route length |
| privAI relays (any hop) | each relay sees only its immediate predecessor and successor — cannot determine full route or content |
| Internet-exit node | sees traffic toward Tor / internet — cannot determine origin identity or full route |
| Tor Exit | sees destination IP — cannot identify provider or privAI route metadata |
| Destination | sees Tor Exit IP — learns nothing about the privAI path |
- **ALLOWLIST** — like TOR-GATED but with domain filter. Declared in contract, accepted before escrow lock.
- **CLEARNET (explicit opt-in)** — VM has direct internet. Provider IP leaks. Requires explicit provider acceptance + higher price.

### Network defaults (frozen)

- Tor is the default network mode for all online communication.
- Clearnet is an option, not a requirement.
- P2P (Model A) for heavy data after established session.
- Mailbox (Model B) for control plane.

### Skill pack locality (frozen)

Every skill pack must declare locality requirement:
- `offline-capable` — works with local model.
- `online-required` — requires remote model/service.
- `network-optional` — works offline, better online.

## Trust / Reputation Direction

Reputation should not start as a naive score.
The long-term direction is a richer trust layer built from:
- stake / bond,
- correct contract history,
- time in system,
- validator behavior,
- dispute behavior,
- availability / reliability.

This may later support:
- weighted quorum selection,
- stronger dispute resolution,
- ZK-backed trust credentials.

But it should not become a blocker for shipping the production system.

## Production Rule Of Thumb

We do not need the final form of every advanced mechanism on day one.
We do need:
- one clear production direction,
- one honest contract model,
- one honest settlement model,
- one honest UI direction,
- and one honest distinction between what exists now and what is future strengthening.

That is the standard we should keep.

## Rollout Phases

```
Phase 0 (now):   Protocol layer — escrow 3/3 e2e, transport close
Phase 1:         Skill protocol — task contract format, verifier format (no UI needed)
Phase 2a:        Minimal workspace — VS Code extension: file tree, task list, terminal, chat panel
Phase 2b:        Full workspace — diff view, agent view, memory view, skill pack browser
Phase 3:         Marketplace v1 — discovery + per-task settlement + compute rental
```

Phase 2a is shippable in months. Phase 2b is iterative. Phase 3 must not be blocked on Phase 2b.

## Frozen Decisions Register (2026-04-10)

| #  | Decision | Status |
|----|----------|--------|
| 1  | Workspace = container, Skill = content, task graph = bridge | FROZEN |
| 2  | Four execution modes: local / remote API / sandbox rental / hybrid | FROZEN |
| 3  | Sandbox user connection via Tor/P2P (existing transport stack) | FROZEN |
| 4  | Sandbox outgoing can use Tor-gated multi-hop through privAI relays plus Tor. Exact hop topology is a network-design detail, not a product invariant. | FROZEN |
| 5  | Operator = rule-bound system program with Falcon signature on normal escrow paths. Recovery remains peer path after timeout. | FROZEN |
| 6  | Pricing v1: per-task-fixed + per-task-quoted + per-compute-unit | FROZEN |
| 7  | Pricing deferred: subscription, per-token, bounty | FROZEN |
| 8  | Execution offline, settlement online | FROZEN |
| 9  | Tor = default network, clearnet = option | FROZEN |
| 10 | Skill packs declare locality: offline-capable / online-required / network-optional | FROZEN |
| 11 | Default sandbox network: ISOLATED (Tor tunnel to user only) | FROZEN |
| 12 | Tor-gated as opt-in upgrade (provider declares in contract) | FROZEN |
| 13 | Allowlist in contract, accepted before escrow lock | FROZEN |
| 14 | Clearnet only with explicit provider opt-in | FROZEN |
| 15 | Currency: PVA | FROZEN |
| 16 | Scope change = new contract + new escrow (no mid-task requote) | FROZEN |
| 17 | Delivery hash = native ledger transaction (no off-chain oracle) | FROZEN |
| 18 | Operator implementation may evolve, but normal escrow settlement remains operator-signed until re-frozen explicitly. | FROZEN |
| 19 | Phase 2a (minimal workspace) ships before Phase 2b (full). Phase 3 not blocked on 2b. | FROZEN |
| 20 | Verification failure: provider can re-submit within timeout. No limit on re-submissions. | FROZEN |
| 21 | Exit-capable relays / validators must opt in explicitly to internet-exit duties. | FROZEN |
| 22 | TOR-GATED is for batch workloads. Interactive chat uses Mode B (direct NXMS). | FROZEN |
