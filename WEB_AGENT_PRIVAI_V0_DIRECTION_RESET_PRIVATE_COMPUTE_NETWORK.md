# V0 Direction Reset — privAI Final System Model

> **Date:** 2026-04-11
> **Status:** Canonical master direction — supersedes all prior marketplace/product framing
> **Authority:** This document wins over any older doc where they conflict on business model, terminology, or settlement semantics.

---

## Core Direction

`privAI` is **not** an AI model marketplace.

`privAI` is a **post-quantum FullPrivacy private AI compute network**.

```text
Privacy is the product.
Compute is the supply.
PVA is the incentive.
Chain is the settlement.
Transport is the shield.
```

Users privately lease AI-capable compute/runtime capacity.

Compute miners / machine operators provide isolated resource slices.

Validators secure the chain.

Relays/mailboxes provide private encrypted transport.

Exit nodes are optional explicit network roles.

**The chain must not become a public marketplace, registry, profile directory, service graph, or reputation billboard.**

---

## Terminology Reset

Use this language going forward:

| Old term | New term |
|----------|----------|
| buyer | user / compute lessee |
| provider | compute miner / machine operator |
| marketplace | private compute discovery / resource lease layer |
| skill pack marketplace | not core |
| artifact marketplace delivery | not core |
| reputation | deterministic machine reliability scoring |
| operator | not required as escrow decision-maker |
| escrow | FullPrivacy PQ compute lease escrow |

Currency:
- **PVA** — privAI coin
- **aPVA** — atomic ledger unit of PVA
- `1 PVA = N aPVA` — final precision decision still needed
- All protocol/ledger accounting uses aPVA, never floats

---

## 1. Identity Model

Identity is not just a public Falcon key.

### V0 answer

Identity = **hidden root credential + scoped role/session/epoch identities**.

Falcon/PQ keys are signing tools, not the entire identity model.

### User / Compute Lessee

- The user is **private by default**.
- The user should not have a public marketplace profile.
- The user uses wallet keys, scoped session keys, note ownership, and PQ signatures only where protocol actions require them.

### Compute Miner / Machine Operator

- The compute miner has a **hidden provider/operator root credential**.
- The network should expose scoped node/session/epoch identities, not a permanent public identity as the baseline.

### Validator

- Validator identity is **separate** from compute miner identity.
- The same operator may run both roles, but the protocol must not merge the roles.

### Onboarding

There is onboarding to become a compute miner, but it is not "publish a public provider profile".

It is **registration/proof of role capability**:
> this node can offer compute resources under protocol rules

### Stake / Bond

- Stake may be required for validators.
- Compute miners may need collateral/bond for higher trust classes, larger leases, or penalty-backed availability.
- Stake/bond should be **selectively proven** where possible, not exposed as public identity history.

### Falcon

Falcon is used for hard protocol signatures:
- escrow authorization
- role registration
- session receipts
- lease claims
- validator actions

**Falcon must not become a stable public marketplace identity by itself.**

### Do Not Infer

- No public nick = identity.
- No public Falcon key = full identity.
- No public provider profile as privacy baseline.
- No human reputation identity as core.

---

## 2. Where Artifacts / Workloads Live

In the new model, "artifact delivery" is not the center of the system.

The core object is: **private compute session**.

Users do not primarily buy AI artifacts from providers. Users **lease private runtime/compute capacity**.

### Data Location

Prompts, workloads, outputs, temporary files, model interaction data, and session workspaces live **off-chain** inside the private compute/runtime session.

The chain must not store:
- artifacts
- prompts
- outputs
- model names
- task text
- skill pack contents
- marketplace history
- provider profiles

At most, the chain may see **generic commitments** required for escrow settlement.

### Transport

Model B / NXMS mailbox / Tor-gated transport is the preferred baseline because it avoids direct endpoint coupling and supports async/private delivery.

Direct P2P/KEM may exist as an optimization, but it is not required as the privacy baseline.

### Storage

- Artifacts/results are **ephemeral by default**.
- Persistent storage is an explicit lease capability, not automatic marketplace storage.
- Large outputs should be encrypted under user-controlled keys and stored off-chain if storage is requested.

### User Confirmation

The user does not need to confirm "artifact receipt" on-chain as marketplace semantics.

Settlement uses:
- lease receipts
- signed acknowledgements
- timeout rules
- protocol claims
- metering evidence

### Miner Visibility

- Compute miner **should not learn** plaintext workload contents.
- If a runtime cannot technically guarantee that, its privacy class must say so explicitly.

### VM / Runtime Clarification

The user does not rent the whole physical machine.

The user rents an **isolated runtime/resource slice**:
- VM
- container
- sandbox
- GPU slice
- CPU/RAM allocation
- session slot

Internet exit is not default. It is a separate explicit role/capability.

---

## 3. Recovery / Escrow Payout

Recovery must not be "who wins a subjective dispute about AI quality".

### V0 answer

Recovery payout is defined by the **compute lease policy**. The policy is rule-bound, not human-moderated.

### Default Settlement Logic

- If lease **never starts** → refund user.
- If lease **completes** → release earned amount to compute miner.
- If lease **partially completes** → split pro-rata by valid delivered compute/session receipts.
- If miner **faults early** → refund unused amount and optionally apply penalty if contract/bond rules allow it.

### Split

Split is allowed and should be expected for compute leases. All-or-nothing settlement is too crude for timed compute.

Example:
```
User locks 100 PVA for 10 hours of GPU class X.
Valid receipts prove 7 hours delivered.
Settlement pays 70 PVA to miner and returns 30 PVA to user,
minus any rule-defined penalty if the miner caused failure.
```

### Who Decides

- Not a human.
- Not a marketplace moderator.
- Not a discretionary operator.
- The **lease contract/policy** decides.
- The **protocol** executes the formula.

### Operator

**Escrow should be operatorless by design.**

Any operator-assisted path is bootstrap/compatibility/automation only, not the canonical FullPrivacy compute lease escrow path.

### Open Follow-Up

If one party disappears, the system needs timeout claim paths using receipts, pre-authorization, or protocol evidence.

This is a **protocol design item**, not a human dispute process.

---

## 4. Fees / Incentives For Nodes

Roles must be separated, even if one physical machine/operator performs multiple roles.

### V0 answer

**PVA rewards useful private infrastructure.** Different node roles earn PVA for different work.

### Roles

Node Operator = entity/machine operator that may run one or more roles:

| Role | Function |
|------|----------|
| Validator | secures chain / validates blocks |
| Compute Miner | rents compute/runtime resources |
| Relay | routes encrypted traffic |
| Mailbox | stores/delivers encrypted envelopes |
| Exit Node | optional public internet egress role |

One machine may run several roles, but the protocol must not confuse them.

### Fees

- Transaction fees are paid in **aPVA** by the transaction submitter or reserved from the session/escrow budget.
- All ledger accounting uses aPVA.
- **No floating-point protocol accounting.**

### Validator Rewards

Validators should receive PVA/aPVA for block validation, chain security, and transaction fees according to consensus economics.

Consensus stays separate from compute rental.

### Compute Miner Rewards

Compute miners receive PVA for valid delivered compute/runtime capacity.

More real usage of their hardware means more PVA earned.

### Relay / Mailbox / Exit Rewards

- Relays may earn PVA for routing.
- Mailboxes may earn PVA for encrypted storage/delivery.
- Exit nodes may earn PVA for internet egress, but exit is explicit opt-in and higher risk.

### Staking

- Validators likely require stake/bond.
- Compute miners may require collateral for larger leases or higher reliability classes.
- Relay/mailbox/exit roles may require role-specific bond if abuse resistance needs it.

### Metering

Every compute miner should use the same metering protocol, but not necessarily the same physical sensor.

Metering should include:
- session duration
- heartbeat/challenge status
- allocated CPU/GPU/RAM/VRAM/storage
- resource class
- valid compute units
- early shutdowns
- missed heartbeats
- energy/power telemetry where available
- meter version
- miner signature

### Energy

Energy usage is useful, but should not be a naive self-reported payment source.

Payment should be based on **proven compute lease units**.

Energy telemetry supports:
- scoring
- fraud detection
- efficiency classification
- audit

### Reliability Scoring

This is **not human reputation**. This is **deterministic machine reliability scoring**.

```
score += uptime
score += completed sessions
score += delivered compute units
score += valid heartbeats

score -= early shutdowns
score -= missed heartbeats
score -= failed challenges
score -= resource mismatch
score -= settlement faults
```

### Do Not Infer

- No subjective AI quality score.
- No human reputation marketplace.
- No public leaderboard as baseline.
- No self-reported energy usage as trusted settlement truth.

---

## 5. Discovery / Registry

**This is the biggest product correction.**

There is no classical public AI marketplace discovery as the baseline.

### V0 answer

- No public AI model marketplace as core.
- No public provider profile as baseline.
- No public skill pack registry as core.

### What Is Discovered

Users discover **compute capacity**, not AI service personalities.

The offer is not:
> "I am a great Python/coding/model provider."

The offer is:
> "I can provide resource class X, for time Y, at price Z,
> under privacy/runtime/network mode R, with reliability credential C."

Possible object names:
- `ComputeOffering`
- `ResourceLeaseOffer`
- `PrivateComputeOffer`

### Offer Contents

- resource class
- GPU/CPU/RAM/VRAM/storage class
- price in aPVA/PVA
- minimum/maximum lease duration
- network mode
- runtime privacy class
- availability window
- meter protocol version
- lease policy version
- scoped offering ID
- selective reliability proof

### Discovery Mode

- Discovery is **private/encrypted/credential-gated by default**.
- Public discovery is lower-privacy opt-in, not baseline.
- User searches by resource requirements, not human/provider reputation.

Examples:
```
GPU class >= X
VRAM >= Y
runtime privacy class >= Z
price <= limit
reliability threshold met
network_mode = nxms_only or tor_gated
```

### Registry

- On-chain registry is **not** the baseline.
- Off-chain private discovery is the baseline.
- If a registry exists, it should be encrypted, scoped, credential-gated, or privacy-preserving.

### Phase 0

A bootstrap coordinator may exist for testing/early network formation, but it must be labeled bootstrap/trust-limited.

It must not redefine the production privacy baseline.

### Skill Packs

Skill packs may exist as **local/client-side extension artifacts**, but not as public marketplace objects on-chain.

Core privAI sells private compute access, not skill marketplace services.

---

## 6. Protocol Versioning

`proof_system_id` is not enough.

### V0 answer

`proof_system_id` versions proof systems only. The whole network needs **explicit protocol version domains**.

### Required version domains

| Domain | Scope |
|--------|-------|
| `chain_protocol_version` | consensus, block format |
| `tx_version` | transaction format |
| `escrow_policy_version` | escrow/lease policy rules |
| `compute_lease_protocol_version` | lease session protocol |
| `meter_protocol_version` | metering format |
| `proof_system_id` | ZK proof system |
| `credential_schema_version` | identity/credential format |
| `discovery_protocol_version` | discovery protocol |
| `nxms_transport_version` | NXMS transport |
| `mailbox_protocol_version` | mailbox protocol |
| `relay_protocol_version` | relay routing |
| `exit_policy_version` | exit node policy |

### Version Agreement

- Chain versions activate by height or epoch.
- Transport versions negotiate in handshake.
- Compute lease version is declared in the offer and contract.
- Meter version is declared in session receipts.
- Credential schema version is declared in proofs/credentials.
- Proof system version is declared by `proof_system_id`.

### Fork Model

- Consensus-breaking validation changes require explicit activation.
- Optional new formats use version gates.
- Privacy-critical changes must never downgrade silently.

### Backward Compatibility

- Older nodes may keep working only for roles and protocol versions they explicitly support.
- A node that does not support FullPrivacy compute lease cannot silently fall back to lower privacy.

**Hard rule: No silent downgrade from FullPrivacy to visible/lower-privacy mode.**

### Agent Guardrail

If a task introduces a new tx, proof, lease, transport, mailbox, credential, discovery, or metering format, it must state the protocol version impact.

---

## Network / Runtime Modes

Runtime network mode must be explicit.

| Mode | Description |
|------|-------------|
| `isolated` | No external network |
| `nxms_only` | Only private privAI/NXMS transport |
| `tor_gated` | Routed through Tor-gated / relay topology |
| `internet_exit` | Public internet egress. Explicit opt-in. Never default. |

Exit node is separate from compute miner. The same node may run both, but the role and risk must be explicit.

---

## Consensus

Consensus stays the same for now. We are not changing consensus because of the business model reset.

What changes is the product/economic layer above it.

Consensus validates:
- valid txs
- nullifiers
- proof/statement commitments
- escrow policy
- fees/rewards
- settlement rules

**Consensus must not know marketplace semantics.**

---

## Escrow

Escrow stays. But its meaning changes.

| Old framing | New framing |
|-------------|-------------|
| AI task / service quality escrow | FullPrivacy PQ compute lease escrow |

Escrow locks PVA/aPVA for a private compute/runtime lease.

Settlement is based on:
- session receipts
- metering evidence
- heartbeats/challenges
- timeouts
- lease policy
- valid signatures

**Not based on** subjective quality of AI output.

**Escrow should be operatorless by design.** Operator-assisted settlement is bootstrap/compatibility/automation only, not the canonical model.

---

## Final Do Not Infer

- Do not infer that privAI is an AI model marketplace.
- Do not infer that providers publish public skill packs on-chain.
- Do not infer that public discovery is the default.
- Do not infer that public reputation exists.
- Do not infer that quality-of-answer is part of settlement.
- Do not infer that escrow requires a human/operator decision.
- Do not infer that `MarketplaceBatchTx` defines the product.
- Do not infer that internet exit is enabled by default.
- Do not infer that Falcon public key is the whole identity.
- Do not infer that self-reported energy usage is trusted settlement truth.
- Do not infer that compute miner and validator are the same protocol role.

---

## V0 Final System Statement

> privAI is a post-quantum FullPrivacy private compute network where users privately lease isolated AI-capable runtime capacity from compute miners.
>
> The chain provides PVA/aPVA settlement through FullPrivacy compute lease escrow.
>
> Validators secure the chain.
>
> Compute miners earn PVA for delivered resources.
>
> Relays/mailboxes earn PVA for private transport.
>
> Exit nodes are optional explicit roles.
>
> Discovery is private and resource-based, not a public AI marketplace.
>
> Scoring measures machine reliability and availability, not subjective AI quality.
>
> Protocol versions must be explicit across chain, escrow, proof, transport, metering, credentials, discovery, relay, mailbox, and exit roles.
>
> **There must be no silent downgrade from FullPrivacy.**
