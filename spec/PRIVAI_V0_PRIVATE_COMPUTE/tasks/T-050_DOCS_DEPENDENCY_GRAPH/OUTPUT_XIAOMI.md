# P-T050-XIAOMI — V0 Docs Dependency Graph

**Status:** docs dependency graph  
**Data:** 2026-04-11  
**Źródło:** inventory folderu `spec/PRIVAI_V0_PRIVATE_COMPUTE/`, docs tree, audyty P-T040–P-T049  
**Zakres:** dependency graph dokumentów V0 — kolejność pisania, co blokuje co

---

## 1. Current Docs Inventory

### Tier 0: Control & Navigation

| Filename | Role | Status | Depends On | Unlocks |
|---|---|---|---|---|
| `PRIVAI_V0_DOCS_TREE.md` | Docs planning map. 5 tiers, production phase plan (Phase 0-5). | DONE (Opus, 2026-04-10) | V0 master | Wszystkie task prompts. Phase plan. |
| `PRIVAI_V0_TASK_LOG.md` | V0 task tracking. 14 zadań (T-001 do T-014). | DONE (Codex, 2026-04-11) | Brak | Task monitoring. |
| `PRIVAI_V0_PROMPT_LOG.md` | V0 prompt tracking. 24 prompty (P-T031 do P-T050). | DONE (Codex, 2026-04-11) | Brak | Prompt monitoring. |

### Tier 1: Canonical Direction

| Filename | Role | Status | Depends On | Unlocks |
|---|---|---|---|---|
| `PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md` | Master V0 direction. Najwyższa authority. Identity, workloads, escrow, fees, discovery, versioning. | DONE (Opus, 2026-04-10) | Brak | Wszystkie inne docs. Canonical source of truth. |
| `PRIVAI_V0_LEGACY_DOCSET_REWRITE_PLAN.md` | Senior review memo. Legacy classification, rewrite priority, 10 guardrails, 8 open follow-ups. | DONE (Opus, 2026-04-10) | V0 master | Legacy migration guidance. |
| `PRIVAI_V0_DIAGRAMS.md` | 15 Mermaid diagrams. System stack, node roles, lease lifecycle, escrow, discovery, transport, metering, scoring, versioning, identity. | DONE (Opus, 2026-04-10) | V0 master | Visual architecture reference. |

### Tier 2: Required Direction

| Filename | Role | Status | Depends On | Unlocks |
|---|---|---|---|---|
| `PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md` | Settlement direction. Receipt/metering-based. Lease policy. Settlement outcomes. Operator boundary. Privacy boundary. | DONE (Opus, 2026-04-10) | V0 master | Operatorless Escrow Direction, Metering Protocol Direction. |
| `PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md` | MCP server direction. 8 tools, FileStore v1, 3 RAG phases, golden tests. | DONE (Opus, 2026-04-10) | Context Plan | MCP implementation (blocked na direction baseline). |
| `PRIVAI_V0_SINGLE_SOURCE_OF_TRUTH_CONTEXT_PLAN.md` | Agent context management. Single source of truth. Legacy quarantine. V0-only RAG. | DONE (Opus, 2026-04-10) | V0 master | MCP Server Direction. Agent integration. |
| `PRIVAI_V0_XIAOMI_5_CONTEXT_PROMPTS.md` | 5 context lock prompts for Xiaomi. | DONE (Opus, 2026-04-10) | V0 master | Xiaomi onboarding. |
| `TASK_031_OPUS_WORKING_CONTEXT.md` | Working handoff for Opus chat. | DONE (2026-04-10) | V0 master | Opus session context. |

### Xiaomi Audits (P-T034–P-T050 — generated during session)

| Filename | Role | Status | Depends On | Unlocks |
|---|---|---|---|---|
| `PRIVAI_V0_FINAL_ARCHITECTURE_PROPOSAL_PL.md` | Final architecture proposal. 11 layers, data flow, on/off-chain boundary, settlement, receipt truth, identity/discovery, transport/runtime. | DONE (Xiaomi, P-T037) | V0 master, settlement direction, diagrams | Migration architecture, domain model, all subsequent audits. |
| `PRIVAI_V0_CODE_REALITY_GAP_REVIEW_PL.md` | Code vs V0 gap review. Table: V0 claim vs code reality. 45+ items. | DONE (Xiaomi, P-T038) | V0 docs, code | Migration architecture, all code-related audits. |
| `PRIVAI_V0_MIGRATION_ARCHITECTURE_PL.md` | Migration strategy. Keep/bridge/deprecate/replace/new primitive matrix. Amount, escrow, marketplace, identity, receipt, orchestrator, transport migration. | DONE (Xiaomi, P-T039) | Final architecture, code gap review | Domain model classification, all specific audits. |
| `PRIVAI_V0_DOMAIN_MODEL_CLASSIFICATION_PL.md` | Domain model classification. Entities, value objects, aggregates, events, invariants — each classified as FROZEN/CANDIDATE/OPEN/BLOCKED/REJECTED. | DONE (Xiaomi, P-T040) | Migration architecture | Decision matrix, minimal types freeze. |
| `PRIVAI_V0_AMOUNT14_AUDIT_PL.md` | Amount14 vs LedgerAmount audit. Usage map, aPVA compatibility, 5 strategy options. | DONE (Xiaomi, P-T041) | Code gap review, migration architecture | Decision matrix, minimal types freeze. |
| `PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md` | SpendPolicy/Escrow compatibility audit. 4 options (extend Escrow2of3 / new SpendPolicy / new Tx / separate layer). | DONE (Xiaomi, P-T042) | Code gap review, migration architecture | Decision matrix, minimal types freeze. |
| `PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md` | Identity migration audit. Current Falcon identity map, V0 requirements, 5 migration options. | DONE (Xiaomi, P-T043) | Code gap review, migration architecture | Decision matrix, minimal types freeze. |
| `PRIVAI_V0_MARKETPLACE_TYPES_AUDIT_PL.md` | Marketplace types fate audit. Type map, 7 options, recommendation (#[deprecated] teraz). | DONE (Xiaomi, P-T044) | Code gap review, migration architecture | Decision matrix. |
| `PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md` | Build-once domain types review. 20 types classified KEEP/CHANGE/TOO_EARLY/REJECT. | DONE (Xiaomi, P-T045) | Domain model, all audits | Minimal types freeze. |

### Task Outputs (in tasks/ folder)

| Filename | Role | Status | Depends On | Unlocks |
|---|---|---|---|---|
| `tasks/T-046_DECISION_MATRIX_DRAFT/OUTPUT_XIAOMI.md` | 19 decisions classified. Freeze-ready, must-wait, operator, Opus, code audit decisions. | DONE (Xiaomi, P-T046) | All PL audits | Final decisions doc. |
| `tasks/T-047_DOMAIN_BOUNDARIES_FREEZE/OUTPUT_XIAOMI.md` | 10 domain boundaries. Invariants, anti-patterns, freeze recommendation. | DONE (Xiaomi, P-T047) | All PL audits | Boundary freeze. |
| `tasks/T-048_MINIMAL_TYPES_FREEZE/OUTPUT_XIAOMI.md` | 10 minimal types for Faza 0a/0b. Dependency graph. 11 types to delay. | DONE (Xiaomi, P-T048) | Build-Once Types Review, all audits | Types first implementation. |
| `tasks/T-049_IMPLEMENTATION_BLOCKERS/OUTPUT_XIAOMI.md` | 10 blocking decisions ranked. Critical path. Parallelizable. Next 3 decisions. | DONE (Xiaomi, P-T049) | All PL audits, decision matrix | Unblock code. |
| `tasks/T-050_DOCS_DEPENDENCY_GRAPH/OUTPUT_XIAOMI.md` | This document. | IN PROGRESS (Xiaomi, P-T050) | All existing docs | Doc writing order. |

---

## 2. Missing Docs Inventory

| Proposed Filename | Why Needed | Depends On | Unlocks | Priority |
|---|---|---|---|---|
| **`PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md`** | Definiuje Phase 0/1/2 bridge. Automated operator validation. Operatorless protocol. Dispute quorum replacement. | V0 master, settlement direction, diagrams | ComputeLeaseEscrow spec, Pro-rata spec, Automated operator implementation, Phase 2 start | **1 (HIGHEST)** |
| **`PRIVAI_V0_METERING_PROTOCOL_DIRECTION.md`** | Receipt fields direction. Heartbeat direction. Trust model (self-reported vs challenge). Receipt availability. | V0 master, settlement direction | Receipt schema spec, Metering trust/challenge spec, Receipt availability spec, Faza 4 (receipts) | **2 (HIGH)** |
| **`PRIVAI_V0_IDENTITY_MODEL_DIRECTION.md`** | Hidden root definition. Key derivation rules. Falcon boundaries. Epoch/session lifecycle. Onboarding semantics. | V0 master, diagrams (identity layers) | Private Discovery Direction (uses scoped IDs), Identity credential schema, Faza 6 (identity) | **3 (HIGH)** |
| **`PRIVAI_V0_NODE_ROLES_AND_INCENTIVES_DIRECTION.md`** | Role separation. PVA incentive per role. Staking/bond direction. Metering integration. | V0 master, settlement direction | Node roles spec, Incentive spec | **4 (MEDIUM)** |
| **`PRIVAI_V0_PRIVATE_DISCOVERY_DIRECTION.md`** | Discovery mode. Bootstrap coordinator. Minimal discovery data. Architecture tradeoffs. | V0 master, Identity Model Direction | Discovery protocol spec, ComputeOffering spec, Faza 8 (discovery) | **5 (MEDIUM)** |
| **`PRIVAI_V0_RUNTIME_PRIVACY_CLASSES_DIRECTION.md`** | VM/container/sandbox granularity. Miner visibility per class. User selection criteria. | V0 master, diagrams | Runtime privacy spec, ComputeLeasePolicy privacy_class field | **6 (MEDIUM)** |
| **`PRIVAI_V0_TRANSPORT_MAILBOX_PRIVACY_DIRECTION.md`** | NXMS/relay metadata hardening direction. Onion routing direction. Traffic analysis resistance. | V0 master | Transport hardening spec | **7 (MEDIUM)** |
| **`PRIVAI_V0_EXIT_NODE_DIRECTION.md`** | Exit node role. Risk model. Pricing direction. Tor integration. Legal/liability. | V0 master | Exit node spec | **8 (LOW-MEDIUM)** |
| **`PRIVAI_V0_APVA_DENOMINATION_DIRECTION.md`** | Precision evaluation (10^12). u64 vs u128. Rounding rules. Interaction with pro-rata. | V0 master. Max supply decision (Operator) | aPVA precision freeze, Settlement formula spec | **9 (BLOCKED by operator)** |
| **`PRIVAI_V0_PROTOCOL_VERSIONING_DIRECTION.md`** | 12 version domains. Activation mechanics. Backward compatibility. No silent downgrade. | V0 master | Version registry spec | **10 (LOW)** |
| **`PRIVAI_V0_RECEIPT_TRUTH_DIRECTION.md`** | 3-layer receipt truth model. Fraud vectors. Evolution from Phase 1 to operatorless. | V0 master, settlement direction, Metering Protocol Direction | Receipt schema spec, Challenge protocol spec | **11 (MEDIUM — po T-035)** |
| **`PRIVAI_V0_PRODUCTION_PHASE_NAMING_CLARIFICATION.md`** | Resolves naming collision between production phases (0-5) and escrow phases (0/1/2). | Docs tree, settlement direction | Clear communication | **12 (LOW — ale szybkie)** |

---

## 3. Dependency Graph

```
PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md (MASTER)
  |
  ├──> PRIVAI_V0_LEGACY_DOCSET_REWRITE_PLAN.md
  ├──> PRIVAI_V0_DIAGRAMS.md
  ├──> PRIVAI_V0_SINGLE_SOURCE_OF_TRUTH_CONTEXT_PLAN.md
  │      └──> PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md
  ├──> PRIVAI_V0_DOCS_TREE.md
  ├──> PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md
  │      ├──> PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md [MISSING]
  │      │      ├──> ComputeLeaseEscrow SpendPolicy Spec
  │      │      ├──> Pro-rata Note Split Spec
  │      │      └──> Automated Operator Implementation
  │      └──> PRIVAI_V0_METERING_PROTOCOL_DIRECTION.md [MISSING]
  │             ├──> Receipt Schema Spec
  │             ├──> Metering Trust/Challenge Spec
  │             └──> Receipt Availability Spec
  │
  ├──> PRIVAI_V0_IDENTITY_MODEL_DIRECTION.md [MISSING]
  │      ├──> PRIVAI_V0_PRIVATE_DISCOVERY_DIRECTION.md [MISSING]
  │      │      ├──> Discovery Protocol Spec
  │      │      └──> ComputeOffering Spec
  │      └──> Identity Credential Schema
  │
  ├──> PRIVAI_V0_NODE_ROLES_AND_INCENTIVES_DIRECTION.md [MISSING]
  │      └──> Node Roles Spec
  │
  ├──> PRIVAI_V0_RUNTIME_PRIVACY_CLASSES_DIRECTION.md [MISSING]
  │      └──> Runtime Privacy Spec
  │
  ├──> PRIVAI_V0_TRANSPORT_MAILBOX_PRIVACY_DIRECTION.md [MISSING]
  │      └──> Transport Hardening Spec
  │
  ├──> PRIVAI_V0_EXIT_NODE_DIRECTION.md [MISSING]
  │      └──> Exit Node Spec
  │
  ├──> PRIVAI_V0_APVA_DENOMINATION_DIRECTION.md [MISSING]
  │      └──> aPVA Precision Freeze
  │
  ├──> PRIVAI_V0_PROTOCOL_VERSIONING_DIRECTION.md [MISSING]
  │      └──> Version Registry Spec
  │
  └──> PRIVAI_V0_RECEIPT_TRUTH_DIRECTION.md [MISSING]
         └──> (depends on Metering Direction)
```

### Parallel tracks (no dependencies between them):

```
Track A: Settlement (depends on settlement direction — DONE)
  PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md
  PRIVAI_V0_METERING_PROTOCOL_DIRECTION.md

Track B: Identity/Discovery (identity before discovery)
  PRIVAI_V0_IDENTITY_MODEL_DIRECTION.md -> PRIVAI_V0_PRIVATE_DISCOVERY_DIRECTION.md

Track C: Roles/Incentives (independent)
  PRIVAI_V0_NODE_ROLES_AND_INCENTIVES_DIRECTION.md

Track D: Runtime/Transport/Exit (independent)
  PRIVAI_V0_RUNTIME_PRIVACY_CLASSES_DIRECTION.md
  PRIVAI_V0_TRANSPORT_MAILBOX_PRIVACY_DIRECTION.md
  PRIVAI_V0_EXIT_NODE_DIRECTION.md

Track E: Denomination/Versioning (independent)
  PRIVAI_V0_APVA_DENOMINATION_DIRECTION.md (blocked by operator supply decision)
  PRIVAI_V0_PROTOCOL_VERSIONING_DIRECTION.md
```

---

## 4. Parallel Tracks

### Can write in parallel (Opus after 2026-04-15):

```
Parallel group 1 (no dependencies between them):
  T-032: PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md
  T-033: PRIVAI_V0_IDENTITY_MODEL_DIRECTION.md
  T-035: PRIVAI_V0_METERING_PROTOCOL_DIRECTION.md
  PRIVAI_V0_NODE_ROLES_AND_INCENTIVES_DIRECTION.md
  PRIVAI_V0_RUNTIME_PRIVACY_CLASSES_DIRECTION.md
  PRIVAI_V0_TRANSPORT_MAILBOX_PRIVACY_DIRECTION.md
  PRIVAI_V0_EXIT_NODE_DIRECTION.md
  PRIVAI_V0_PROTOCOL_VERSIONING_DIRECTION.md
```

### Must be sequential:

```
Identity Model Direction -> Private Discovery Direction
  (discovery uses scoped IDs from identity)

Metering Protocol Direction -> Receipt Truth Direction
  (receipt truth builds on metering direction)

Operatorless Escrow Direction -> ComputeLeaseEscrow Spec -> Pro-rata Spec
  (each builds on previous)

aPVA Denomination Direction -> aPVA Precision Freeze -> Settlement Formula Spec
  (each builds on previous)

Compute Lease Object Spec -> ComputeLeaseEscrow Spec (lease_policy_commit field)
  (lease policy struct feeds into escrow)
```

---

## 5. Critical Path To Code

Shortest path od obecnego stanu do pierwszego bezpiecznego code task:

```
TERAZ (Operator decyzje):
  1. Max supply PVA decyzja (u64 vs u128)
  2. MarketplaceBatchTx fate (#[deprecated]?)

PO 2026-04-15 (Opus — 3 docs w parallel):
  3. T-032: Operatorless Escrow Direction
  4. T-033: Identity Model Direction
  5. T-035: Metering Protocol Direction

PO 3 docs (Opus — kolejne 3):
  6. ComputeLeaseEscrow SpendPolicy spec
  7. Private Discovery Direction
  8. aPVA Denomination Direction (jeśli supply decyzja zrobiona)

PO specs (KOD — Faza 0a):
  9. 10 typów: LedgerAmount, NetworkMode, SettlementMode, PrivacyClass, RoleType,
     SpendPolicyTag::ComputeLeaseEscrow, TargetRecipient::Two, EscrowAction::ProRataSplit,
     HiddenRootCredential, RoleKey

PO Faza 0a (KOD — Faza 3):
  10. ComputeLeaseEscrow SpendPolicy variant (tag 0x04)
  11. validate_compute_lease_escrow_auth() — nowa walidacja
  12. Tests dla nowej walidacji
```

**Pierwszy bezpieczny code task: Faza 0a (10 typów) — po 3 Opus docs + 2 operator decyzjach.**

Ale: **LedgerAmount = u64 alias + NetworkMode enum** mogą być dodane JUŻ (zero dependencies, zero risk). To jest absolute minimum "code start" — 2 typy, zero impact.

---

## 6. Critical Path To MCP/RAG

Shortest path do bezpiecznego `privai-context-mcp`:

```
TERAZ (direction baseline — 12 docs exist + 9 missing):
  DONE: V0 master, settlement direction, diagrams, context plan, MCP direction,
        docs tree, all PL audits, all task outputs (14 istniejących)
  MISSING: 9 direction docs (T-032, T-033, T-035, node roles, discovery,
           runtime privacy, transport, exit node, aPVA, versioning)

MINIMUM viable MCP (nie wymaga ALL docs):
  MCP może startować z istniejącymi docs:
  - V0 master ✅
  - Settlement direction ✅
  - Diagrams ✅
  - Context plan ✅
  - MCP direction ✅
  - Docs tree ✅
  - All PL audits ✅
  
  To jest wystarczające do Sprint 0 (local MCP skeleton, FileStore, 8 tools).

BLOKER dla MCP production:
  - Golden tests wymagają comprehensive V0 docs (brak 9 direction docs)
  - RAG ingest wymaga complete direction baseline

REKOMENDACJA:
  MCP Sprint 0 (local skeleton) — może startować TERAZ (6 core docs istnieją)
  MCP Sprint 1 (golden tests) — po T-032, T-033, T-035 (3 kluczowe direction docs)
  MCP Sprint 2 (RAG ingest) — po complete direction baseline (all 13 direction docs)
```

---

## Final Self-Check

- **Czy czytałem legacy docs:** NIE
- **Czy edytowałem pliki inne niż output:** NIE (tylko OUTPUT_XIAOMI.md)
