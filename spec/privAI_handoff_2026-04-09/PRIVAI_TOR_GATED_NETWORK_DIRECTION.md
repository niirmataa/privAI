# privAI TOR-GATED Network Direction

## 1. Purpose

This note captures the **network-design level** detail for the TOR-GATED sandbox internet-access path. It exists separately from `PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md` because the production-direction doc freezes product invariants (Tor-gated, multi-hop, provider-blind, opt-in exit), while this note tracks the topology parameters that are still being designed.

This document does **not** claim protocol-level finality. It records design direction and candidate parameters.

## 2. Scope

This note covers:
- relay hop topology for sandbox outgoing internet traffic
- relay role definitions
- exit-node role and opt-in policy
- Tor usage pattern within the privAI relay path
- suitability for batch vs interactive workloads

This note does **not** cover:
- escrow / settlement mechanics
- operator / dispute model
- marketplace pricing
- skill-pack format
- validator consensus networking (covered by `PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`)

## 3. Frozen High-Level Invariants

These are frozen product-direction decisions. They are stated here for reference; the authoritative source is `PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md`.

| # | Invariant |
|---|-----------|
| F1 | Sandbox outgoing internet traffic routes through privAI relays plus Tor (Tor-gated). |
| F2 | The provider is blind to the destination — cannot learn where traffic is going. |
| F3 | No single relay learns the full route (onion model). |
| F4 | Internet-exit is an explicit opt-in role, declared by the relay / validator operator. |
| F5 | Interactive chat (Mode B) should prefer direct NXMS, not the TOR-GATED path. |
| F6 | TOR-GATED is primarily suited for sandbox and batch-style workloads. |
| F7 | Sandbox speaks ONLY the privAI relay protocol — no direct Tor client in the VM. |
| F8 | Default sandbox network is ISOLATED (no internet). TOR-GATED is an opt-in upgrade declared in contract. |

## 4. Design Parameters Left Intentionally Open

The following are **not frozen** and should be treated as design parameters subject to iteration:

- exact number of privAI relay hops
- whether relay count is fixed or configurable per contract
- whether KEM layering and Tor circuit layering are separate hops or combined
- exit-node selection policy (random, round-robin, stake-weighted, contract-specified)
- relay incentive / compensation model for exit duties
- whether relay path is selected by the user, the provider, or the protocol
- failure handling when a relay goes offline mid-path

## 5. Candidate Topology Dimensions

### 5.1. Hop Count

A prior design sketch used 3 KEM hops + 3 Tor hops. This is a **candidate**, not a frozen decision.

Considerations for hop count:
- more hops = more privacy (harder to correlate entry to exit)
- more hops = more latency (relevant for interactive workloads, less critical for batch)
- more hops = more relay capacity needed in the network
- minimum viable: 1 privAI relay + Tor exit is functional but weaker privacy
- sweet spot is an open design question

The production direction doc does not constrain hop count. This note records the candidate and the trade-offs.

### 5.2. Relay Roles

Candidate role split:

| Role | Description | Opt-in? |
|------|-------------|---------|
| **Relay** | forwards encrypted traffic to next hop | yes — operator declares relay participation |
| **Exit-capable relay** | can forward traffic from privAI network to Tor | yes — explicit opt-in, separate declaration |
| **Validator (dual role)** | a validator node that also participates as a relay or exit-capable relay | yes — separate opt-in for network duties |

Separation of duties:
- Provider != Relay != Exit-capable relay
- A single node may hold multiple roles, but each role requires separate opt-in
- The exit role is the most sensitive and should carry additional incentive

### 5.3. Exit Role

The internet-exit node is the point where privAI-encrypted traffic enters the Tor network.

Requirements (frozen):
- exit participation must be explicit opt-in
- exit node must not log or leak origin identity
- exit node sees traffic toward Tor but not the full route or content

Open design questions:
- how exit nodes are discovered by the relay path
- whether exit selection is random, user-directed, or protocol-assigned
- whether exit nodes have additional staking / bonding requirements
- whether exit capacity is declared per-contract or per-epoch

### 5.4. Tor Usage Pattern

The current direction is that privAI relays feed into the Tor network at the exit point. The VM does not run a Tor client directly.

Open design questions:
- does the exit node create a fresh Tor circuit per request, per session, or per contract?
- does the exit node use Tor onion services for additional privacy?
- is there a persistent Tor circuit for long-running sandbox sessions?
- what is the fallback if Tor is unavailable (deny vs degraded)?

### 5.5. Batch vs Interactive Suitability

Frozen direction: TOR-GATED is for batch and sandbox workloads. Interactive chat uses Mode B (direct NXMS).

Rationale:
- multi-hop relay paths add latency
- interactive chat benefits from direct low-latency transport
- sandbox workloads (code execution, data processing) are less latency-sensitive

Open questions:
- what is the acceptable latency budget for TOR-GATED sandbox traffic?
- can a single sandbox session mix batch and interactive operations, or must they be separated?
- should there be a "light TOR-GATED" mode with fewer hops for semi-interactive workloads?

## 6. Non-Goals

This note does **not** propose:
- changes to the escrow / settlement model
- changes to the operator / dispute model
- changes to the marketplace pricing model
- changes to the validator consensus networking (covered by `PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`)
- a specific cryptographic protocol for relay-to-relay framing (that is implementation-level)

## 7. What Is Not Frozen Yet

The following decisions remain open and should be frozen before implementation of the TOR-GATED relay path:

- [ ] relay hop count (fixed vs configurable)
- [ ] relay path selection algorithm
- [ ] exit-node discovery mechanism
- [ ] exit-node incentive model
- [ ] Tor circuit lifecycle per sandbox session
- [ ] relay failure / mid-path recovery
- [ ] relay bandwidth / capacity declaration
- [ ] contract-level TOR-GATED configuration format (how the buyer/provider agree on TOR-GATED in the contract)

These should be frozen in a dedicated decision register entry before the relay path is implemented.

---

*Document version: 2026-04-10. Companion to PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md. This note tracks network-design parameters; the production direction doc freezes product-level invariants only.*
