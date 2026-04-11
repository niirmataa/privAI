# V0 Legacy Docset Rewrite Plan

> **Date:** 2026-04-11
> **Type:** Senior review memo + migration plan
> **Author:** Opus (systemowy senior)
> **Scope:** Docs only. No code changes. No new protocol claims.

---

## 1. Verdict

V0 Direction Reset spełnia swój cel. Jest to kanoniczny reset framingu biznesowo-produktowego.

**Odpowiada na 6 luk:** tak, wszystkie 6 zamknięte na poziomie kierunku (direction-level).

**Jest canonical direction reset:** tak. Jasno zastępuje stary framing marketplace/provider/artifact/quality-settlement/operator-canonical.

**Czego nie zamyka:**
- Żadnego protocol freeze na poziomie implementacji (brak compute lease object model, brak metering receipt schema, brak operatorless escrow mechanics w kodzie).
- Nie definiuje aPVA precision.
- Nie definiuje hidden root / scoped identity formatu.
- Nie definiuje reliability scoring formuły jako protocol spec.
- Nie definiuje private discovery protocol.
- Nie zmienia kodu.

V0 jest latarnią nawigacyjną. Nie jest implementacją.

---

## 2. Six Gaps Assessment

| # | Gap | V0 answer | Status | Follow-up |
|---|-----|-----------|--------|-----------|
| 1 | **Identity model** | Hidden root credential + scoped session/epoch keys. Falcon = signing tool. Validator ≠ miner. Onboarding = proof of capability. | `answered at direction level` | Needs protocol spec: credential format, scoping mechanism, epoch rotation, how hidden root relates to wallet keys. |
| 2 | **Workloads / artifacts** | Off-chain in private compute session. Chain sees only generic commitments. Ephemeral by default. | `answered at direction level` | Needs protocol spec: session lifecycle object, what "generic commitment" looks like on-chain for compute lease vs current escrow note. |
| 3 | **Recovery / payout** | Pro-rata split by receipts. Lease policy decides. Operatorless by design. | `answered at direction level` | Needs protocol spec: receipt schema, split formula, how pro-rata maps to escrow note value (single note → split into two outputs?). Current escrow 2-of-3 is all-or-nothing per action — split requires new mechanics. |
| 4 | **Fees / incentives** | 5 roles separated. PVA/aPVA. Metering protocol. Reliability scoring. | `answered at direction level` | Needs protocol spec: fee distribution mechanism, metering receipt schema, staking parameters, relay/mailbox payment channel. |
| 5 | **Discovery / registry** | Private/encrypted/credential-gated. Resource-based. Off-chain baseline. | `answered at direction level` | Needs protocol spec: discovery protocol format, ComputeOffering schema, how credential-gating works with hidden identities. |
| 6 | **Protocol versioning** | 12 version domains. Activation by height/epoch/handshake. No silent downgrade. | `answered at direction level` | Needs protocol spec: version negotiation protocol, activation mechanics, backward compatibility matrix. |

---

## 3. Supersession Rule

```
V0 supersedes product/business marketplace framing.
V0 does not supersede deep-spec mechanical truth.
```

### What this means — examples:

| Claim | V0 wins? | Why |
|-------|----------|-----|
| "privAI is an AI marketplace" | **V0 wins** — it is a private compute network | Product framing |
| "Provider publishes skill packs on-chain" | **V0 wins** — discovery is off-chain, resource-based | Product framing |
| "Operator is canonical normal-path co-signer" | **V0 wins** — operatorless by design, operator is bootstrap | Product framing |
| "Settlement is based on quality of AI output" | **V0 wins** — settlement is receipt/metering-based | Product framing |
| "Escrow uses 2-of-3 with Release/Refund/Recovery actions" | **Deep spec wins** — code-confirmed mechanics unchanged | Implementation truth |
| "Stage A uses sentinel tx_signing_hash" | **Deep spec wins** — code-confirmed in escrow_stage.rs | Implementation truth |
| "Halo2 circuit is 1-in/1-out scaffold" | **Deep spec wins** — code-confirmed in tx_skeleton.rs | Implementation truth |
| "Submit gate validates 10 points" | **Deep spec wins** — code-confirmed in node.rs | Implementation truth |
| "Timeout enforcement is in privai-ledger" | **Deep spec wins** — code-confirmed with tests | Implementation truth |
| "Validator transport and NXMS mailbox are separate" | **Both agree** — V0 reinforces, deep spec confirms | Aligned |

---

## 4. Legacy Docs Classification

| Doc | Classification | Why | Next action |
|-----|----------------|-----|-------------|
| `PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md` | **canonical** | Master direction doc. Highest authority for product/business framing. | None — source of truth |
| `PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md` | **partially superseded** | Marketplace/provider/operator framing superseded. Escrow mechanics, transport modes, sandbox network, on-chain privacy commitment structure — still valid reference. Frozen decision #5 (operator canonical) explicitly reversed by V0. | Already marked superseded. No rewrite yet — use as mechanical reference until individual rewrites replace sections. |
| `PRIVAI_PRODUCTION_SYSTEM_DIAGRAMS.md` | **needs rewrite** | ~10 of 19 diagrams use old marketplace/provider/quality framing. §5A (on-chain privacy), §10-12 (execution/sandbox), §15 (network topology) still valid. | Rewrite task: new V0-aligned diagrams. |
| `PRIVAI_CONTRACT_VERIFICATION_AND_SETTLEMENT_DIRECTION.md` | **needs rewrite** | Entire doc built around "contract-first quality settlement" and "skill pack" model. Core premise invalidated: settlement is now receipt/metering-based, not quality-based. 4-level verification model (mechanical → contractual → semantic → settlement) does not apply to compute leases. | Rewrite as "Compute Lease Settlement Direction". |
| `PRIVAI_OPERATOR_AND_DISPUTE_QUORUM_DIRECTION.md` | **needs rewrite** | "Operator = rule executor on normal path" is now "operator = bootstrap only". "Dispute quorum" loses context when there are no subjective quality disputes. Core framing inverted. | Rewrite as "Operatorless Escrow Direction + Bootstrap Operator Spec". |
| `PRIVAI_STAGE_A_STAGE_B_CONTRACT_FREEZE.md` | **still valid mechanical reference** | Code-confirmed escrow boundary. V0 does not change Stage A/B mechanics. Terminology (buyer/merchant) is code-level and stays. | Eventually: add V0 context note at top. No rewrite needed. |
| `PRIVAI_HALO2_PROOF_BOUNDARY_FREEZE.md` | **still valid mechanical reference** | Code-confirmed proof boundary. V0 does not change circuit reality. §9 product implications reference "marketplace" — minor terminology, not structural. | Eventually: update §9 terminology. No rewrite needed. |
| `PRIVAI_TOR_GATED_NETWORK_DIRECTION.md` | **still valid mechanical reference** | Network topology unchanged by V0. "Provider" terminology → "compute miner" is cosmetic. Frozen invariants F1-F8 still hold. | Eventually: terminology update. No rewrite needed. |
| `PRIVAI_SYSTEM_PRODUCT_FOUNDATION.md` | **partially superseded** | §1 says "privacy-first marketplace and settlement system for local AI models". §2D says "Marketplace settlement" as foundation. §3 roles (Buyer, Merchant, Marketplace operator) use old framing. Code-level mechanics (§G wallet, §H node/ledger, §I transport) still valid. | Rewrite needed but lower priority — agents read V0 first now. |
| `PRIVAI_PROJECT_ENTRYPOINT.md` | **partially superseded** | "first honest escrow v1 end-to-end" is still valid as code-level goal. Mental model (two planes, two stages) still valid. But §1 description is old framing. | Light update: add V0 reference at top + update §1 description. |
| `PRIVAI_V1_READINESS_AND_GAPS.md` | **still valid mechanical reference** | Status table is code-level truth. "Ready/Partially Ready/Open" classifications are about implementation, not product framing. | No change needed. |
| `PRIVAI_NEXT_DIRECTION.md` | **still valid mechanical reference** | Priority order (mailbox → refund → recovery → timeout → freeze) is implementation-level. Not affected by product reframe. Agent split still valid. | No change needed. |
| `PRIVAI_DOCS_INDEX.md` | **partially superseded** | Already updated with V0 at top. Old sections still reference marketplace framing. | Already updated. Will need another pass after individual doc rewrites. |
| `PRIVAI_START_HERE.md` | **partially superseded** | Already updated with V0 in Etap 4. "Co to jest privAI" updated. Reading order updated. | Already updated. Will need another pass after individual doc rewrites. |

---

## 5. Rewrite Priority Order

### T-030: V0-Aligned Diagrams

- **Target files:** new `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIAGRAMS.md`
- **Goal:** Create new diagram set reflecting V0 framing: compute network topology, compute lease lifecycle, receipt-based settlement, 5 node roles, operatorless escrow, private discovery, reliability scoring. Reuse valid diagrams from old set (§5A, §10-12, §15) with terminology update.
- **Forbidden:** Do not edit old diagrams doc. Do not create protocol specs.
- **DoD:** New doc with ≥12 mermaid diagrams. All use V0 terminology. Zero "marketplace/provider/skill pack/quality" in framing text.

### T-031: Compute Lease Settlement Direction

- **Target files:** new `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md`
- **Goal:** Replace CONTRACT_VERIFICATION_AND_SETTLEMENT_DIRECTION with V0-aligned lease settlement model: lease policy, receipt-based verification, pro-rata split, metering evidence, timeout claims. Not a protocol spec — direction level only.
- **Forbidden:** Do not define receipt schema. Do not define metering wire format. Do not touch escrow code.
- **DoD:** Doc covers: lease policy model, settlement logic (complete/partial/fault), what replaces 4-level verification, timeout/claim paths. References V0 master.

### T-032: Operatorless Escrow Direction

- **Target files:** new `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md`
- **Goal:** Replace OPERATOR_AND_DISPUTE_QUORUM_DIRECTION with V0-aligned operatorless model: what "operatorless by design" means for current 2-of-3 mechanics, how bootstrap operator transitions to protocol-only settlement, what replaces dispute quorum in compute lease context.
- **Forbidden:** Do not redesign escrow code. Do not claim operatorless is implemented. Do not remove recovery path.
- **DoD:** Doc covers: operatorless target, bootstrap operator reality, transition path, what happens to 2-of-3 mechanics, receipt-based resolution vs quality disputes.

### T-033: Identity Model Direction

- **Target files:** new `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_IDENTITY_MODEL_DIRECTION.md`
- **Goal:** Expand V0 §1 into standalone direction doc: hidden root credential, scoped session/epoch keys, how identity relates to wallet keys and Falcon, onboarding flow for compute miners, validator identity separation.
- **Forbidden:** Do not define wire format. Do not define credential proof circuits. Do not touch nexum-cli code.
- **DoD:** Doc covers: identity layers (root → role → session → epoch), Falcon usage boundaries, what is NOT identity (no public profile, no reputation nick), onboarding semantics.

### T-034: System Product Foundation V0 Update

- **Target files:** `spec/privAI_handoff_2026-04-09/PRIVAI_SYSTEM_PRODUCT_FOUNDATION.md`
- **Goal:** Update §1-§3 to reflect V0 framing. Keep §4-§8 (honest system state, reading path) intact. Add supersession note. Update "marketplace for local AI models" → "private compute network".
- **Forbidden:** Do not rewrite code-level mechanics sections. Do not remove role descriptions that match code (wallet, node, ledger).
- **DoD:** §1 one-line definition matches V0. §2D reflects compute lease not marketplace settlement. §3 roles updated (compute miner, not marketplace operator as core). V0 reference added.

### T-035: Light Updates (batch)

- **Target files:** STAGE_A_STAGE_B_CONTRACT_FREEZE, HALO2_PROOF_BOUNDARY_FREEZE, TOR_GATED_NETWORK_DIRECTION, PROJECT_ENTRYPOINT
- **Goal:** Add V0 context note at top of each. Minimal terminology updates where "provider" → "compute miner" in product-framing text (not in code references).
- **Forbidden:** Do not change code-confirmed content. Do not change technical descriptions.
- **DoD:** Each file has V0 context note. `grep "AI marketplace"` in these files returns 0 hits.

---

## 6. Non-Negotiable Guardrails

1. **No public AI marketplace as baseline.** privAI is a private compute network.
2. **No public discovery as baseline.** Discovery is private/encrypted/credential-gated.
3. **No public provider profile as baseline.** Identity is hidden root + scoped keys.
4. **No subjective AI quality settlement.** Settlement is receipt/metering-based.
5. **No operator as canonical escrow decision-maker.** Operatorless by design; operator is bootstrap.
6. **No silent downgrade from FullPrivacy.** Hard protocol rule.
7. **No code changes before V0 docs migration plan is accepted.** Docs first, code second.
8. **No product reset overriding proof/ledger implementation facts.** V0 is direction, not implementation truth.
9. **No "marketplace" terminology in new V0-era docs.** Use "private compute network", "resource lease", "compute offering".
10. **No public reputation/leaderboard as baseline.** Reliability = deterministic machine scoring.

---

## 7. Open Protocol Follow-Ups

These items are named by V0 but require their own protocol-level specs before implementation:

| Item | What V0 says | What's missing |
|------|-------------|----------------|
| **aPVA precision** | `1 PVA = N aPVA`, no floats | Final N value, denomination rules |
| **Compute lease object model** | User leases isolated runtime, session receipts | Lease struct, on-chain representation, how it maps to current EscrowNote |
| **Metering receipt schema** | Heartbeats, challenges, resource class, miner sig | Wire format, what constitutes valid receipt, challenge protocol |
| **Hidden root / scoped identity** | Root credential + session/epoch scoping | Credential format, key derivation, epoch rotation, revocation |
| **Reliability scoring formula** | Additive/subtractive deterministic score | Exact weights, score storage (on-chain? off-chain?), privacy of score |
| **Private discovery protocol** | Resource-based, credential-gated, off-chain | Discovery message format, matching protocol, bootstrap coordinator spec |
| **Operatorless escrow mechanics** | Escrow should be operatorless by design | How to remove operator from 2-of-3 without breaking existing code; receipt-validated auto-settlement |
| **Version registry** | 12 version domains, activation by height/epoch | Registry format, negotiation protocol, migration mechanics |

None of these should be attempted before the docs migration (T-030 through T-035) is complete.

---

## 8. Final Recommendation

**Przejść do rewrite docs:** Tak. V0 master jest na miejscu. START_HERE i DOCS_INDEX zaktualizowane. Agenci od następnej sesji widzą V0 jako pierwszy direction doc. Można bezpiecznie zacząć kontrolowane rewrite'y.

**Pierwszy rewrite task:** T-030 (V0-Aligned Diagrams). Powód: diagramy to najszybszy sposób na utrwalenie V0 framingu w głowach agentów. Jeden plik, zero zmian w legacy docs, bounded scope.

**Czego nie robić teraz:**
- Nie przepisywać SYSTEM_PRODUCT_FOUNDATION i PROJECT_ENTRYPOINT przed T-030/T-031/T-032 — te docs i tak wskazują teraz na V0 przez START_HERE.
- Nie zaczynać protocol specs (metering, identity, discovery) przed zamknięciem docs migration — kierunek musi być stabilny zanim definiujemy wire formats.
- Nie zmieniać kodu escrow/node/ledger pod V0 framing — istniejąca mechanika 2-of-3 jest nadal valid i testowana; operatorless jest direction, nie implementation.
- Nie kasować ani archiwizować legacy docs — są nadal potrzebne jako mechanical reference dopóki rewrite'y nie zastąpią ich sekcji.
