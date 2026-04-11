# V0 Compute Lease Settlement Direction

> **Date:** 2026-04-11
> **Type:** `frozen direction` — V0 canonical settlement model
> **Author:** Opus (systemowy senior)
> **Scope:** Docs only. No code changes. No wire format definitions. No new implementation claims.
> **Replaces (at direction level):** `PRIVAI_CONTRACT_VERIFICATION_AND_SETTLEMENT_DIRECTION.md`
> **Companion to:** `PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md`, `PRIVAI_V0_DIAGRAMS.md`
> **Authority:** Where this doc and the old contract/verification/settlement direction conflict on settlement semantics, this doc wins. Where this doc and the Stage A/B Contract Freeze or Halo2 Proof Boundary Freeze conflict on implementation facts, the code-confirmed docs win.

---

## 1. Core Rule

```
FullPrivacy compute lease escrow settles resource delivery, not AI output quality.
```

Settlement is determined by:
- whether the leased runtime/compute capacity was made available,
- whether metering receipts prove delivered resource units,
- whether the lease policy conditions were met,
- whether timeouts were respected.

Settlement is **not** determined by:
- subjective quality of AI output,
- semantic correctness of model answers,
- human review of artifacts,
- marketplace reputation of the compute miner.

This is the foundational rule. Everything below derives from it.

---

## 2. What Replaces Contract / Quality Settlement

The old `PRIVAI_CONTRACT_VERIFICATION_AND_SETTLEMENT_DIRECTION.md` described a contract-first, artifact-delivery, quality-settlement model. V0 replaces that mental model entirely for the compute lease context.

| Old Model (superseded) | V0 Model (canonical) |
|------------------------|---------------------|
| Task contract defines scope of work | **Compute lease policy** defines resource commitment |
| Artifact delivery is the settlement primitive | **Private compute session** delivery is the settlement primitive |
| Proof of delivery = artifact hash committed on-chain | **Metering receipts** = signed evidence of delivered resource units |
| Semantic quality review (Level 3 verification) | **Not part of settlement** — the user evaluates their own output privately |
| 4-level verification (mechanical → contract → semantic → settlement) | **2-level verification** (receipt validity → settlement execution) |
| Provider delivers skill pack result | **Compute miner** provides isolated runtime slice |
| Buyer buys AI task result | **User / compute lessee** leases private compute capacity |
| Contract is the primary unit of evaluation | **Lease policy** is the primary unit of evaluation |
| Skill pack structure (metadata + contract + verifier pack) | **ComputeOffering + lease policy** (resource class + terms + metering version) |
| Scope change = new contract + new escrow | **Lease extension = new lease** (same principle, different object) |
| Provider can re-submit artifact within timeout | **Session delivers continuously** — no re-submission model |
| Refund because artifact is bad quality | **Refund because resource was not delivered** |

### What the old model got right (preserved)

- Escrow locks before service starts → **preserved**: escrow locks before session starts.
- No lock without acceptance → **preserved**: no escrow lock without lease policy acceptance.
- Timeout leads to recovery path → **preserved**: timeout leads to recovery/claim path.
- Recovery is buyer + merchant, no operator → **preserved**: recovery is user + miner, no operator.
- Settlement is downstream of acceptance → **preserved**: settlement is downstream of lease policy acceptance.

---

## 3. Compute Lease Policy

A compute lease policy defines the terms of the resource commitment. It is accepted by both parties before escrow lock.

### Direction-level fields

The following fields describe what a lease policy must capture at direction level. **This is not a wire format definition.** Exact struct, encoding, and on-chain representation are future protocol specs.

| Field | Purpose |
|-------|---------|
| **resource_class** | What compute resource is leased: GPU class, CPU tier, RAM, VRAM, storage class. |
| **runtime_privacy_class** | Privacy guarantee of the runtime: what the miner can/cannot observe about workload contents. |
| **network_mode** | `isolated` / `nxms_only` / `tor_gated` / `internet_exit`. Determines session network access. |
| **lease_duration** | Minimum and/or maximum session time. May be expressed in blocks, wall-clock, or compute units. |
| **lease_quota** | Optional: maximum compute units (if quota-based rather than time-based). |
| **price_aPVA** | Total price in aPVA for the lease. All protocol accounting uses aPVA. |
| **timeout_blocks** | Maximum blocks after escrow lock before timeout claim path activates. |
| **settlement_formula** | How delivered units map to payout: linear pro-rata, tiered, threshold-based. Direction-level only. |
| **failure_rules** | What happens on miner fault: full refund, partial refund, penalty/slash if bond posted. |
| **receipt_requirements** | Minimum heartbeat interval, required receipt fields, challenge requirements. |
| **collateral_bond** | Optional: miner bond/collateral for penalty-backed availability. |
| **meter_protocol_version** | Which metering protocol version receipts must use. |
| **lease_policy_version** | Version of this lease policy format. |

### What lease policy is NOT

- Lease policy is not a skill pack contract.
- Lease policy does not describe expected AI output quality.
- Lease policy does not contain validation conditions for semantic correctness.
- Lease policy does not define a verifier pack for human review.
- Lease policy does not specify artifact delivery hash.

---

## 4. Settlement Outcomes

### 4.1 Full Release

**Condition:** Lease completed. Metering receipts prove full delivery of committed resource units / duration.

**Flow:**
- User (or protocol) submits settlement claim with receipt evidence.
- Receipts validated against lease policy.
- Full escrow amount released to compute miner.

**Direction note:** In current code, Release requires Buyer + Operator signatures (2-of-3). V0 target is protocol-validated release from receipts (operatorless). See §6.

### 4.2 Full Refund

**Condition:** Lease never started, or miner completely failed to provide resources (no valid receipts exist).

**Flow:**
- User submits refund claim (or timeout triggers refund path).
- No valid receipts → full escrow amount returned to user.

**Direction note:** In current code, Refund requires Merchant + Operator signatures (2-of-3). V0 target is protocol-validated refund from absence of receipts (operatorless). See §6.

### 4.3 Pro-rata Split

**Condition:** Lease partially completed. Metering receipts prove N of M committed units/duration were delivered.

**Flow:**
- User (or protocol) submits settlement claim with partial receipt evidence.
- Settlement formula calculates miner share and user remainder.
- Miner receives earned aPVA. User receives remainder aPVA.

**Example:**
```
Lease: 100 aPVA for 10 hours GPU class X.
Valid receipts: 7 hours delivered.
Settlement formula: linear pro-rata.
→ Miner receives 70 aPVA.
→ User receives 30 aPVA.
```

**Direction note:** Pro-rata split is the V0 target for normal compute lease settlement. **Current escrow code does not implement pro-rata split.** Current mechanics are all-or-nothing per action (Release sends full amount to merchant, Refund sends full amount to buyer). Pro-rata requires new escrow note mechanics — likely splitting one input note into two output notes with different recipients. This is a future protocol spec item.

### 4.4 Penalty / Slash

**Condition:** Miner posted collateral/bond, and receipts prove miner fault (missed heartbeats, early shutdown, resource mismatch).

**Flow:**
- Direction-level only: miner's bond is partially or fully slashed according to failure rules in lease policy.
- Slashed amount may go to user as compensation, or to protocol treasury, or both.

**Direction note:** Penalty/slash mechanics are not defined at protocol level yet. This is a future spec item. Current escrow code does not support bond/slash.

### 4.5 Recovery After Timeout

**Condition:** Timeout elapsed. Settlement not completed through normal path. Neither full release nor full refund was executed.

**Flow:**
- User + miner sign recovery together (no operator required).
- Peer-resolved: the two parties agree on split.

**Code-confirmed:** RecoveryRelease action already exists. Buyer + Merchant sign, no Operator. Timeout enforcement is implemented in `privai-ledger/src/escrow.rs` and tested (`reject_recovery_before_timeout`).

### 4.6 No Quality Dispute Rule

```
There is no AI quality dispute settlement path in V0.
```

If the user is unhappy with the quality of AI output produced during the leased session, that is **not a settlement matter**. The settlement question is: "was the resource delivered?" not "was the AI output good?"

The user evaluates their own output privately. The chain does not see the output. The escrow does not know what the model produced.

If the miner provided the contracted resource class for the contracted duration, and receipts prove it, the miner has earned their payment.

---

## 5. Receipt / Metering Evidence

Receipts are the settlement evidence in V0. They replace artifact delivery proofs.

### What a receipt is (direction-level)

A metering receipt is a signed attestation that a unit of compute resource was delivered during a session. It is produced by the compute miner (or jointly by miner and user in a challenge/response model).

### Receipt evidence (direction-level fields)

| Evidence | Purpose |
|----------|---------|
| **session_id** | Identifies which lease session this receipt belongs to. |
| **resource_class** | What resource was delivered (must match lease policy). |
| **duration_delivered** | How much time / how many compute units were delivered in this interval. |
| **heartbeat_status** | Whether miner responded to heartbeats/challenges during this interval. |
| **meter_protocol_version** | Which metering protocol produced this receipt. |
| **miner_signature** | Compute miner's Falcon signature over receipt contents. |
| **user_ack** | Optional: user's acknowledgment of receipt (strengthens evidence). |
| **timestamp** | Block height or wall-clock reference for ordering. |
| **challenge_response** | Optional: proof that miner answered a liveness/capability challenge. |

### What a receipt is NOT

- A receipt is not an artifact delivery proof.
- A receipt does not describe what the user's prompt was.
- A receipt does not describe what the model produced.
- A receipt does not contain AI output quality metrics.
- A receipt does not contain workload contents.
- A receipt is not a human review judgment.

### Receipt validity rules (direction-level)

- Receipt must reference a valid session bound to a locked escrow.
- Receipt `resource_class` must match or exceed lease policy requirements.
- Receipt must be signed by the miner whose key is bound to the escrow/lease.
- Receipts must cover the claimed duration without gaps exceeding the maximum allowed heartbeat interval.
- Duplicate receipts for the same interval are rejected.
- Receipt `meter_protocol_version` must match the version declared in the lease policy.

### What is NOT defined here

- Exact binary format of receipts.
- Exact challenge protocol.
- Exact Rust struct for metering data.
- How receipts are transmitted (in-band vs separate channel).
- Storage requirements for receipt archives.

These are future protocol spec items.

---

## 6. Operator Boundary

### V0 Target

```
Operatorless escrow by design.
```

The canonical V0 settlement model does not require a third-party operator keypair for normal settlement. Settlement should be validated by the protocol from receipts + lease policy + timeout rules.

### Current Reality

Current escrow code uses 2-of-3 multi-sig with an operator role:
- **Release:** Buyer + Operator sign → PVA to Merchant.
- **Refund:** Merchant + Operator sign → PVA to Buyer.
- **RecoveryRelease:** Buyer + Merchant sign → no Operator needed.

This is `code-confirmed` in Stage A/B freeze document and tested in e2e tests.

### Transition Direction

| Phase | Description | Status |
|-------|-------------|--------|
| **Phase 0** (current) | Operator co-signs Release and Refund. All-or-nothing. Dev keypair. | `code-confirmed` |
| **Phase 1** (next) | Automated operator: validates receipts against lease policy, co-signs mechanically. Still a keypair, but rule-bound. | `future direction` |
| **Phase 2** (V0 target) | Operatorless: protocol validates receipts, settlement executes without operator keypair. Pro-rata split supported. | `future direction` |

### What this means for agents and code

- Do NOT remove operator from current 2-of-3 mechanics. Current code works and is tested.
- Do NOT claim operatorless settlement is implemented.
- Do NOT claim pro-rata split is implemented.
- Recovery path (Buyer + Merchant, no Operator) already aligns with V0 direction — it is the one action that is already operatorless.
- Phase 1 (automated operator) is the most likely near-term implementation step.

### Operator is NOT

- Operator is not a marketplace moderator.
- Operator is not a quality reviewer.
- Operator is not a dispute arbitrator.
- Operator is not a trust anchor for the system.
- Operator is not required for RecoveryRelease (already operatorless).

---

## 7. Interaction With Existing Escrow Mechanics

This section is honest about the gap between V0 direction and current implementation.

### What remains unchanged (code-confirmed mechanical truth)

| Mechanic | Status | Reference |
|----------|--------|-----------|
| Stage A / Stage B boundary | `code-confirmed` | `PRIVAI_STAGE_A_STAGE_B_CONTRACT_FREEZE.md` |
| `EscrowApprovalBundle` as A→B handoff | `code-confirmed` | `privai-nxms/src/lib.rs:229` |
| `TX_SIGNING_HASH_STAGE_A` sentinel | `code-confirmed` | `privai-nxms/src/lib.rs:227` |
| Final `tx_signing_hash` computed in Stage B | `code-confirmed` | `escrow_builder.rs:286` |
| `policy_opening` required in Stage B auth | `code-confirmed` | `node.rs:648-651` |
| Submit gate 10-point validation | `code-confirmed` | `node.rs:593-756` |
| Real Falcon signatures over final hash | `code-confirmed` | e2e tests |
| Release = Buyer + Operator | `code-confirmed` | `privai-chain/src/escrow.rs` rule table |
| Refund = Merchant + Operator | `code-confirmed` | `privai-chain/src/escrow.rs` rule table |
| RecoveryRelease = Buyer + Merchant | `code-confirmed` | `privai-chain/src/escrow.rs` rule table |
| Timeout enforcement in ledger | `code-confirmed` | `privai-ledger/src/escrow.rs` |
| Persistence across node restarts | `code-confirmed` | `escrow_persistence.rs` tests |
| EscrowAction values (0x01, 0x02, 0x03) | `code-confirmed` | `privai-chain/src/escrow.rs` |

### What V0 direction adds (not yet implemented)

| V0 Target | Gap from current code | Future work |
|-----------|----------------------|-------------|
| Pro-rata split settlement | Current escrow is all-or-nothing per action | New escrow note mechanics: 1 input → 2 outputs with different recipients |
| Receipt-validated settlement | No metering receipt object exists in code | Receipt struct, validation logic, chain/ledger integration |
| Operatorless Release/Refund | Current Release/Refund require operator | Phase 1 (automated operator) then Phase 2 (protocol-only) |
| Compute lease policy object | No lease policy struct in code | Lease policy definition, commitment scheme, on-chain binding |
| Settlement formula execution | No formula engine in escrow | Formula evaluator that maps receipts → split amounts |
| Penalty/slash mechanics | No bond/slash infrastructure | Bond tracking, slash rules, penalty distribution |

### Bridge path (direction-level)

Future implementation must bridge current escrow to V0 settlement:

1. **Receipt object** — define metering receipt struct that can be validated by escrow logic.
2. **Automated operator** — operator co-signs mechanically based on receipt validation (Phase 1).
3. **Lease policy binding** — bind lease policy to escrow note at lock time (extend current `SpendPolicy::Escrow2of3` or create new policy type).
4. **Pro-rata note splitting** — new escrow action that produces two output notes from one input.
5. **Remove operator** — once receipt validation is protocol-level, remove operator from Release/Refund paths (Phase 2).

**This bridge path is direction only. It does not define implementation order or exact code changes.**

---

## 8. Privacy Boundary

### What the chain MUST NOT see

| Data | Rule |
|------|------|
| Prompts / workloads | Never on-chain. Ephemeral in session. |
| Model outputs / AI results | Never on-chain. User evaluates privately. |
| Runtime session files | Never on-chain. Ephemeral by default. |
| Public marketplace history | No marketplace exists. |
| Public provider/miner profile | Hidden root + scoped identity is baseline. |
| Workload contents in receipts | Receipts attest resource delivery, not workload. |
| AI quality scores | Not a settlement primitive. |

### What the chain MAY see (as commitments)

| Data | Form | Purpose |
|------|------|---------|
| Escrow locked amount | Encrypted/committed aPVA | Settlement value |
| Lease policy commitment | Hash/commitment of policy | Bind settlement rules to escrow |
| Timeout | Block height | Enable timeout claims |
| Nullifiers | Hash | Prevent double-spend |
| Settlement authorization | Falcon signatures | Authorize release/refund/recovery |
| Proof/statement commitments | Merkle roots, hashes | Proof coverage binding |
| Receipt commitment (future) | Hash/commitment | Bind receipt evidence to settlement claim |

### Privacy rule

```
The chain settles compute leases. The chain does not learn what computation was performed.
```

---

## 9. Non-Goals / Do Not Infer

This document does **NOT** claim or intend:

1. **No subjective AI quality settlement.** If the miner delivered the resource and receipts prove it, the miner has earned payment. Quality of AI output is the user's private concern.

2. **No human marketplace dispute.** There is no human dispute resolution panel. Settlement is rule-bound from receipts + lease policy + timeout.

3. **No public artifact delivery baseline.** There is no on-chain artifact hash, no public delivery commitment for "task result." Settlement evidence is metering receipts, not artifact hashes.

4. **No operator as canonical decision-maker.** Operator is bootstrap/compatibility. V0 target is operatorless.

5. **No claim current code implements pro-rata.** Current escrow is all-or-nothing per action. Pro-rata requires new mechanics.

6. **No claim metering receipt schema is frozen.** Receipt fields described in §5 are direction-level. Exact format is a future protocol spec.

7. **No proof completeness overclaim.** Current Halo2 layer is a real scaffold (1-in/1-out, Poseidon commitments, range checks, inter-chip wiring) but does NOT prove full transfer privacy, balance conservation, consumed-note opening, or spend validity. See `PRIVAI_HALO2_PROOF_BOUNDARY_FREEZE.md`.

8. **No claim operatorless escrow is implemented.** Only RecoveryRelease is currently operatorless. Release and Refund still require operator co-signature.

9. **No claim lease policy object exists in code.** Current escrow uses `SpendPolicy::Escrow2of3` with buyer/merchant/operator pk_hash + timeout_block. Compute lease policy is a V0 direction concept, not a current code struct.

10. **No claim penalty/slash/bond mechanics exist.** These are future protocol specs.

11. **No claim settlement formula engine exists.** Current escrow does not calculate splits. It executes predefined actions (Release/Refund/Recovery) as all-or-nothing.

12. **No 4-level verification model.** The old mechanical → contract → semantic → settlement verification hierarchy is replaced. V0 settlement has two verification steps: (1) receipt validity against lease policy, (2) settlement execution.

---

## 10. Open Protocol Follow-Ups

These items are named by this direction doc and by V0 master but require their own protocol-level specs before implementation:

| Item | What this doc says | What is missing |
|------|-------------------|-----------------|
| **Compute lease policy struct** | Direction-level fields in §3 | Exact Rust struct, on-chain encoding, commitment scheme, how it extends or replaces current `SpendPolicy::Escrow2of3` |
| **Metering receipt schema** | Direction-level fields in §5 | Exact binary format, signing message construction, challenge/response protocol |
| **Settlement formula** | Linear pro-rata mentioned as default | Exact formula, rounding rules (aPVA integer only), how tiered/threshold formulas are expressed |
| **Pro-rata note splitting** | 1 input → 2 outputs with different recipients | New `EscrowAction` or extension to existing actions, ledger validation rules, output target logic |
| **Automated operator (Phase 1)** | Operator validates receipts mechanically | Receipt validation logic in operator service, how operator accesses receipt evidence, failure modes |
| **Operatorless settlement (Phase 2)** | Protocol validates receipts without operator key | How to remove operator from `Escrow2of3` auth without breaking existing escrow notes, migration path |
| **Penalty / slash / bond** | Direction-level in §4.4 | Bond lock mechanics, slash conditions, penalty distribution, bond return path |
| **Timeout claim path** | User claims refund/split after timeout using receipts | Exact claim submission format, how receipts are presented on-chain (commitment? full? hash?), validation in ledger |
| **Receipt storage and availability** | Receipts are settlement evidence | Where receipts are stored, how long, who serves them, availability guarantees |
| **Lease extension / renewal** | New lease for extension (§2) | Whether consecutive leases share session state, how escrow transitions |

None of these should be attempted before this direction doc and the V0 docs migration (T-030 through T-035 in `PRIVAI_V0_LEGACY_DOCSET_REWRITE_PLAN.md`) are accepted.

---

## 11. Relationship to Other Docs

| Document | Relationship |
|----------|-------------|
| `PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md` | **Parent.** V0 master direction. This doc derives settlement semantics from V0. |
| `PRIVAI_V0_DIAGRAMS.md` | **Companion.** §3 (Compute Lease Lifecycle), §5 (FullPrivacy Escrow Boundary), §9 (Metering/Receipt Flow) visualize what this doc describes in text. |
| `PRIVAI_V0_LEGACY_DOCSET_REWRITE_PLAN.md` | **Companion.** This doc is T-031 from the rewrite plan. |
| `PRIVAI_CONTRACT_VERIFICATION_AND_SETTLEMENT_DIRECTION.md` | **Superseded at direction level.** Old quality/contract settlement model replaced by compute lease settlement. Remains as historical reference for old framing. |
| `PRIVAI_STAGE_A_STAGE_B_CONTRACT_FREEZE.md` | **Mechanical reference.** Code-confirmed Stage A/B boundary is not changed by this doc. |
| `PRIVAI_HALO2_PROOF_BOUNDARY_FREEZE.md` | **Mechanical reference.** Code-confirmed proof boundary is not changed by this doc. |
| `PRIVAI_OPERATOR_AND_DISPUTE_QUORUM_DIRECTION.md` | **Superseded at direction level.** Old operator/dispute model. Will be replaced by T-032 (Operatorless Escrow Direction). |

---

*Document version: 2026-04-11. V0 canonical compute lease settlement direction. This doc is target direction, not implementation proof. Current Stage A/B escrow remains mechanical reference. Current Halo2 proof boundary remains mechanical reference. Does not define wire formats, exact structs, or protocol schemas.*
