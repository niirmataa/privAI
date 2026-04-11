# privAI Contract, Verification, and Settlement Direction

Date: 2026-04-10
Status: `frozen direction` — canonical product-layer note

---

## 1. Purpose

Freeze the currently intended production model for:

- contract-first execution
- the relationship between delivery and quality
- what verification means in the `privAI` marketplace
- what a skill / task contract must declare
- how settlement maps to the contract model

This note exists because the production docs now freeze system direction, diagrams, TOR-GATED network direction, and Stage A / Stage B boundary — but the product-layer contract model was still spread across discussion and partial docs. This note makes it canonical.

It does **not** replace:
- `PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md` (system direction, frozen decisions register)
- `PRIVAI_PRODUCTION_SYSTEM_DIAGRAMS.md` (visual companion)
- `PRIVAI_STAGE_A_STAGE_B_CONTRACT_FREEZE.md` (escrow protocol boundary)
- `PRIVAI_SYSTEM_PRODUCT_FOUNDATION.md` (product foundation)

It is the reference for "what contract, verification, and settlement mean as a product-layer model."

---

## 2. Canonicality / Status

This document is `frozen direction`. It captures the intended product-layer model as derived from:

- `PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md` (frozen decisions #1–#22)
- `PRIVAI_PRODUCTION_SYSTEM_DIAGRAMS.md` (verification model, settlement mapping, skill pack structure)
- `PRIVAI_SYSTEM_PRODUCT_FOUNDATION.md` (marketplace settlement, escrow as primitive)
- `PRIVAI_V1_PRODUCTION_PATH.md` (production baseline requirements)

Status labels used in this document:

- `current production model` — what is implemented and code-confirmed
- `frozen direction` — product decision that is settled and binding
- `future strengthening` — known direction, not yet implemented, not a blocker
- `not yet frozen` — open design space, not claimed as decided

---

## 3. Contract-First Execution Model

### 3.1 The contract is primary

`frozen direction`

The unit of evaluation in the `privAI` marketplace is **the contract**, not the model, not the provider's reputation, not an abstract quality score.

The marketplace does not rate models in the abstract. It evaluates whether a specific contracted artifact / task result satisfies the specific contract that was accepted before escrow lock.

This is the foundational rule:

- the contract is the binding object,
- the contract is accepted before escrow lock,
- settlement is judged against the contract, not vague "model quality."

### 3.2 Contract lifecycle

`frozen direction`

```
Provider declares contract and verification conditions
  → Buyer reviews contract
  → Buyer accepts or rejects
  → Escrow locks only after acceptance
  → Provider executes
  → Artifact delivered
  → Verification against contract
  → Settlement (release / refund / recovery)
```

The clean rule is: **no acceptance, no lock**. Escrow never locks on a contract the buyer has not explicitly accepted.

### 3.3 Scope change = new contract

`frozen direction`

If scope changes mid-task, there is no mid-task requote mechanism. The clean path is:

1. Refund the old escrow (provider signs Refund).
2. Negotiate a new contract with new scope and new price.
3. Lock a new escrow.

This preserves the invariant: **locked amount is fixed for the lifetime of an escrow**.

There is no partial release, no mid-task adjustment, no renegotiation inside a locked escrow.

### 3.4 What the contract governs

`frozen direction`

The contract governs:

- what work will be done (scope)
- what artifacts will be produced (expected outputs)
- what conditions must be met for acceptance (validation conditions / DoD)
- what is explicitly out of scope or forbidden (non-goals / do-not-touch rules)
- what timeout applies
- what price is agreed (off-chain, privately between buyer and provider)

Settlement is downstream of contract acceptance. Delivery is downstream of contract acceptance. Verification is against the contract.

---

## 4. Delivery vs Quality

### 4.1 The distinction

`frozen direction`

Delivery and quality are not the same thing. The system must never pretend otherwise.

**Proof of delivery** proves:

- the artifact exists
- the artifact hash matches the committed `delivery_hash`
- the artifact has the expected format / structure
- the artifact was delivered within the agreed SLA / timeout

**Proof of quality** would need to prove:

- semantic correctness
- usefulness to the buyer
- architectural fit
- that the result actually solves the stated problem

### 4.2 What current mechanics prove

`frozen direction`

Current proof and settlement mechanics prove **delivery and protocol validity**. They do NOT automatically prove semantic quality.

Specifically:

- The `delivery_hash` committed as a native ledger transaction proves that a specific artifact was committed by the provider. This is a protocol-level proof of commitment, not a proof of quality.
- Mechanical verification (Level A) proves build/test/lint passability, not semantic correctness.
- Contract verification (Level B) proves scope was respected and DoD checklist items are met, but DoD is a structural checklist, not a semantic judgment.
- Semantic quality may require reviewer judgment or explicitly agreed validation conditions that go beyond mechanical and contract checks.

### 4.3 Honest first delivery model

`frozen direction`

The honest first delivery model is:

1. Provider commits artifact hash (native ledger transaction).
2. Artifact is delivered to buyer.
3. Buyer confirms receipt / acceptance.
4. Escrow settles through release / refund / recovery.

Marketplace boundary:

- delivery evidence belongs to the contract / evidence layer, not to marketplace-specific ledger state
- if a delivery commitment is anchored on-chain, it must be a generic `delivery_hash` / `delivery_commit` binding for private contract settlement
- the chain must not infer skill pack, marketplace category, task text, artifact contents, or provider profile semantics from that commitment
- see `PRIVAI_MARKETPLACE_CHAIN_BOUNDARY_FREEZE.md`

Current proof certificates should be treated as **protocol-validity proofs**, not as final semantic proof that a skill was performed correctly.

### 4.4 Future strengthening

`future strengthening`

Semantic quality verification may be strengthened over time through:

- more explicit validation conditions in the contract (e.g., "output must compile with zero warnings," "output must pass test suite X")
- reviewer packs that automate semantic checks where possible
- human review integrated into the verification workflow
- reputation and contract-history signals (not as a replacement for contract-based evaluation, but as an additional signal)

But the frozen rule remains: **the contract is the binding object, not an abstract quality score**.

---

## 5. Verification Layers

### 5.1 The four-level model

`frozen direction`

Work is verified in four levels. Each level is distinct. No level subsumes another.

#### Level 1: Mechanical verification

Verifies structural / mechanical properties of the delivered artifact:

- build succeeds
- tests pass
- lint passes
- schema validation passes
- forbidden files are untouched
- artifact format matches expected structure

This level is automatable. It does not require human judgment.

**Failure path:** Provider can re-submit with a new `delivery_hash`. Escrow stays locked. No limit on re-submissions within timeout.

#### Level 2: Contract verification

Verifies that the delivered artifact satisfies the contract:

- scope is respected (no out-of-scope changes)
- task pill / DoD checklist is completed
- non-goals are respected (forbidden changes not made)
- timeout constraints are met

This level is partially automatable (where DoD items are checkable) and partially requires judgment (where scope boundaries are ambiguous).

**Failure path:** Buyer can request fix (provider re-submits) or request Refund.

#### Level 3: Semantic verification

Verifies the deeper quality of the delivered artifact:

- semantic correctness
- architectural fit
- usefulness to the stated problem
- coherence with the broader project context

This level typically requires human or reviewer-level judgment. It cannot be fully automated in the general case.

**Failure path:** Refund if provider agrees (signs Refund). If provider disagrees → timeout → Recovery (peer resolution).

#### Level 4: Settlement execution

Executes the financial settlement based on verification outcome:

- **Release:** buyer accepts → buyer signs Release → operator co-signs → PVA to provider
- **Refund:** rejected + provider agrees → provider signs Refund → operator co-signs → PVA to buyer
- **Recovery:** timeout / unresolved → buyer + provider sign Recovery → peer resolution, no operator

This level is protocol-enforced. It does not make quality judgments — it executes based on the outcome of Levels 1–3.

### 5.2 Verification failure paths (summary)

`frozen direction`

| Level | Failure | Path |
|-------|---------|------|
| Level 1 (Mechanical) | Build/test/lint fails | Re-submit with new `delivery_hash`. Escrow stays locked. No limit within timeout. |
| Level 2 (Contract) | Scope/DoD not met | Buyer requests fix or requests Refund. |
| Level 3 (Semantic) | Quality rejected | Refund if provider agrees. If not → timeout → Recovery. |
| Any | Timeout without resolution | Recovery path (buyer + provider sigs, no operator). |

### 5.3 What every production skill / task offering must have

`frozen direction`

Every production skill or task offering in the marketplace must declare:

- **a contract** — what work will be done, what scope is covered
- **validation conditions** — what conditions must be met for acceptance (mechanical + contract level at minimum)
- **expected artifacts** — what outputs the buyer should expect
- **non-goals / forbidden changes** — what is explicitly out of scope or must not be changed

These declarations are part of the skill pack (see Section 6). A skill offering without a contract, validation conditions, and expected artifacts is not a production offering.

---

## 6. What a Production Contract Must Declare

### 6.1 Contract declaration model

`frozen direction`

A production contract (as part of a skill pack) must declare:

| Field | Purpose |
|-------|---------|
| **Scope** | What work will be performed. Bounded description of the task. |
| **Expected artifacts** | What outputs the buyer receives. Typed where possible. |
| **Validation conditions** | What must be true for acceptance. Mechanical (Level 1) + contract (Level 2) at minimum. |
| **Non-goals** | What is explicitly out of scope. |
| **Forbidden changes** | What must not be modified (e.g., "do not touch config X," "do not change dependency Y"). |
| **Timeout** | Maximum execution time. After timeout, recovery path activates. |
| **Locality** | `offline-capable` / `online-required` / `network-optional`. |
| **Pricing model** | `per-task-fixed` / `per-task-quoted` / `per-compute-unit`. |
| **Settlement policy** | Timeout blocks, delivery mechanism (hash-commit), dispute path (timeout → recovery). |

### 6.2 Skill pack structure

`frozen direction`

A skill pack bundles:

- **metadata** — name, version, locality, capability requirements
- **task contract** — input/output schema, do-not-touch rules, scope definition, timeout
- **verifier pack** — Level 1 (build, test, lint), Level 2 (scope check, DoD), Level 3 (human review recommended)
- **pricing** — model type, estimate range
- **settlement policy** — timeout blocks, delivery hash-commit, dispute: timeout → recovery

This structure is the frozen direction. The exact serialization format is `not yet frozen` and belongs to Phase 1 (skill protocol) implementation.

### 6.3 Non-goals of this section

`frozen direction`

This section freezes the **model** — what a contract must declare — not a speculative serialization format. The exact JSON / protobuf / binary schema for skill packs is a Phase 1 implementation concern, not a product-level invariant.

---

## 7. Settlement Mapping

### 7.1 Settlement is judged against the contract

`frozen direction`

Settlement is not judged against vague "model quality" or reputation scores. Settlement is judged against **the specific contract that was accepted before escrow lock**.

The question at settlement time is not "is this provider good?" The question is: **"does this artifact satisfy this contract?"**

### 7.2 Settlement outcomes

`frozen direction`

| Verification outcome | Settlement path | Signatures required |
|---------------------|-----------------|---------------------|
| Accepted (contract satisfied) | **Release** — PVA to provider | Buyer + Operator |
| Rejected (provider agrees) | **Refund** — PVA to buyer | Provider + Operator |
| Timeout / unresolved | **Recovery** — peer resolution | Buyer + Provider (no operator) |

### 7.3 Release

`frozen direction`

Release means the contract is satisfied. The buyer accepts the delivered artifact.

- Buyer signs Release (Falcon signature).
- Operator validates and co-signs (Falcon signature).
- PVA transfers to provider.

Operator validation checks (for Release):
1. Contract hash matches locked contract.
2. Delivery hash was committed by provider.
3. Buyer signed acceptance.
4. Timeout constraints respected.
5. All signatures are valid Falcon (PQ).

### 7.4 Refund

`frozen direction`

Refund means the contract is not satisfied. The provider agrees the artifact does not meet the contract.

- Provider signs Refund (Falcon signature).
- Operator validates and co-signs (Falcon signature).
- PVA returns to buyer.

Operator validation checks (for Refund):
1. Contract hash matches locked contract.
2. Provider signed acceptance of refund.
3. Timeout constraints respected.
4. All signatures are valid Falcon (PQ).

### 7.5 Recovery

`frozen direction`

Recovery exists for the case where buyer and provider cannot agree and the timeout has elapsed. It is the escape path from operator lock and from deadlocked disputes.

- Buyer + Provider sign Recovery (Falcon signatures).
- No operator signature required.
- Peer resolution: the two parties resolve between themselves.

Why recovery exists:
- It prevents operator from becoming a permanent lock point.
- It gives both parties agency after timeout.
- It keeps the system contract-first: if the contract cannot be satisfied and the parties cannot agree, the timeout recovery path resolves the deadlock without requiring a third-party moderator.

Recovery is **not** a dispute resolution mechanism in the traditional sense. It is a timeout-based escape hatch. The frozen direction is that normal disputes (where provider agrees the artifact is bad) use Refund. Recovery is for the harder case where agreement cannot be reached.

### 7.6 Why this stays contract-first, not reputation-first

`frozen direction`

The settlement model is contract-first:

- The contract is the binding object, not the provider's reputation.
- A new provider with a good contract can settle cleanly. A reputable provider with a bad contract result gets Refund or Recovery.
- Reputation is a future signal (weighted quorum selection, trust layer), not the basis for settlement decisions.
- The operator validates protocol rules, not provider reputation.

Reputation / trust layer is `future strengthening`. It may later influence quorum selection, dispute weighting, or discovery ranking — but it does not change the fundamental rule: **settlement is judged against the contract**.

---

## 8. What Is Frozen Now

The following are `frozen direction` — binding product decisions:

| # | Decision |
|---|----------|
| 1 | The contract is the primary unit of evaluation, not the model or provider reputation. |
| 2 | Contract terms are accepted before escrow lock. No lock without acceptance. |
| 3 | Settlement is judged against the contract, not vague "model quality." |
| 4 | Scope change = new contract + new escrow (no mid-task requote). |
| 5 | Delivery and settlement are downstream of contract acceptance. |
| 6 | Proof of delivery is not proof of quality. The system never pretends otherwise. |
| 7 | Four-level verification: mechanical → contract → semantic → settlement. |
| 8 | Every production skill / task offering must declare: contract, validation conditions, expected artifacts, non-goals / forbidden changes. |
| 9 | Settlement outcomes: Release (accepted), Refund (rejected + provider agrees), Recovery (timeout / unresolved). |
| 10 | Normal Release and Refund require operator signature. Recovery is buyer + provider, no operator. |
| 11 | Recovery exists as timeout escape, not as traditional dispute resolution. |
| 12 | Reputation is not the basis for settlement. Contract-first, not reputation-first. |
| 13 | Provider can re-submit within timeout after Level 1 failure. No limit on re-submissions. |
| 14 | Semantic verification may require reviewer judgment or explicitly agreed validation conditions. |
| 15 | Skill pack structure: metadata + task contract + verifier pack + pricing + settlement policy. |

These decisions are consistent with the 22 frozen decisions in `PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md` and the code-confirmed Stage A / Stage B boundary in `PRIVAI_STAGE_A_STAGE_B_CONTRACT_FREEZE.md`.

---

## 9. What Remains Intentionally Open

The following are `not yet frozen` — open design space, not claimed as decided:

| Area | Status | Notes |
|------|--------|-------|
| Exact skill pack serialization format | `not yet frozen` | Phase 1 implementation concern. JSON / protobuf / binary schema not decided. |
| Reviewer pack format | `not yet frozen` | How semantic verification packs are structured and distributed. |
| Automated semantic verification depth | `not yet frozen` | How much of Level 3 can be automated vs requires human judgment. |
| Dispute quorum implementation | `future strengthening` | Independent quorum of providers, Falcon-signed verdicts. Referenced in `PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md` but not this doc's subject. |
| Trust / reputation scoring model | `future strengthening` | Weighted by stake, contract history, time in system, etc. Not the basis for settlement. |
| Operator implementation phases | `future strengthening` | Phase 0 = dev keypair. Phase 1-2 = automated service. More validation may move to protocol later. |
| Per-token pricing | `frozen direction: deferred` | Requires trusted metering. Not solved in privacy context. Deferred. |
| Subscription pricing | `frozen direction: deferred` | Recurring escrow pattern. Future. |

---

## 10. Non-Goals / Do Not Infer

This document does **NOT** claim:

- That current cryptographic proofs prove the whole quality of a skill result. They prove delivery and protocol validity.
- That the marketplace rates models in the abstract. It evaluates contracted artifacts against contracts.
- That semantic verification is fully automatable today. It may require human judgment.
- That the operator is a discretionary moderator. The operator is a rule executor.
- That recovery is a dispute resolution system. It is a timeout escape hatch.
- That reputation is irrelevant. It is `future strengthening`, not the current basis for settlement.
- That this doc redesigns operator / dispute quorum. It cross-references them briefly; they are covered in `PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md`.
- That this doc defines a skill pack serialization format. It freezes the model, not the serialization.
- That Phase 0 operator (dev keypair) is trustless. It is "honest about trust."
- That escrow belongs to any rail other than `FullPrivacy`. Escrow 2-of-3 belongs to `FullPrivacy`, not `MarketplaceSmallPaymentsRail` and not `OnChainLite`.

---

*Document version: 2026-04-10. Derives from PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md (22 frozen decisions), PRIVAI_PRODUCTION_SYSTEM_DIAGRAMS.md (verification model, settlement mapping, skill pack structure), PRIVAI_SYSTEM_PRODUCT_FOUNDATION.md (marketplace settlement, escrow as primitive), and PRIVAI_V1_PRODUCTION_PATH.md (production baseline). This note freezes the contract/verification/settlement product layer. It does not override protocol, format, proof, ledger, or network specs.*
