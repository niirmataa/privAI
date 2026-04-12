# P-T049-XIAOMI — Implementation-Blocking Decisions

**Status:** implementation blockers identification  
**Data:** 2026-04-11  
**Źródło:** synteza audytów P-T040–P-T048  
**Zakres:** decyzje które blokują jakikolwiek sensowny kod V0

---

## 1. Blocking Decisions Ranked

| Rank | Decision | Why Blocking | Affected Modules | Blocked Future Docs | Blocked Code Tasks | Owner | Minimum Evidence Needed |
|---|---|---|---|---|---|---|---|
| **1** | **Max supply PVA (u64 vs u128)** | Blokuje aPVA precision freeze. Blokuje LedgerAmount type finalizację. Jeśli u64 za małe później = migration kosztowna. Jeśli u128 za duże = overhead. | `privai-chain` (primitives), escrow, settlement, proofs | aPVA Precision Spec, Settlement Formula Spec | Faza 0a: LedgerAmount type. Faza 3: ComputeLeaseEscrow SpendPolicy. Faza 5: Pro-rata. | Operator | Decyzja: "Max supply PVA ≤ X PVA" → determinuje u64 vs u128 |
| **2** | **Operatorless Escrow Direction (T-032)** | Definiuje Phase 0/1/2 bridge. Bez niego nie wiadomo jak automated operator działa, jak protocol waliduje receipts, jak operator znika z path. Blokuje całą ścieżkę settlement. | `privai-ledger` (escrow validation), `privai-node` (orchestrator), `privai-wallet` (settlement building) | Metering Protocol Direction, Pro-rata Spec, ComputeLeaseEscrow Spec | Faza 5: Pro-rata. Faza 7: Automated operator bridge. Faza 3: ComputeLeaseEscrow validation. | Opus (T-032) | Direction doc z Phase 0/1/2 definitions, automated operator validation rules, operatorless protocol bridge |
| **3** | **ComputeLeaseEscrow SpendPolicy spec** | Definiuje nowy SpendPolicy variant (tag 0x04). Bez niego nie ma V0 escrow path — tylko legacy Escrow2of3. Blokuje: escrow lock, settlement, pro-rata, lease policy binding. | `privai-chain` (note.rs — SpendPolicy enum), `privai-ledger` (escrow.rs — nowa validation), `privai-wallet` (escrow_builder.rs — nowy builder) | Pro-rata Spec, Lease Policy Spec | Faza 3: Nowa walidacja. Faza 5: Pro-rata. Faza 4: Receipt integration. | Opus (spec) | SpendPolicy fields, validation rules, commitment scheme, signer rules, output target rules |
| **4** | **Metering Protocol Direction (T-035)** | Definiuje receipts, heartbeats, trust model, challenge/response direction. Bez niego nie ma receipt infrastructure — settlement nie ma dowodów. Blokuje: receipt production, receipt validation, trust model, challenge protocol. | Nowy moduł `privai-metering` (nie istnieje), `privai-ledger` (receipt validation) | Receipt Schema Spec, Metering Trust/Challenge Spec, Receipt Availability Spec | Faza 4: Receipt infrastructure. Faza 7: Automated operator receipt validation. | Opus (T-035) | Direction doc z receipt fields direction, heartbeat direction, trust model direction (self-reported vs challenge), receipt availability direction |
| **5** | **Identity Model Direction (T-033)** | Definiuje hidden root, role keys, session keys, epoch keys, Falcon boundaries. Bez niego identity jest "Falcon PK = identity" — nie ma separacji ról, nie ma session keys, nie ma epoch rotation. Blokuje: identity hierarchy, compute miner identity, session management. | `privai-node` (identity_provider.rs), vault format, escrow (signer identity), transport (peer identity) | Private Discovery Direction (discovery używa scoped IDs) | Faza 6: Identity foundation. Faza 8: Discovery (depends on scoped IDs). | Opus (T-033) | Direction doc z hidden root definition, key derivation rules, Falcon boundaries, epoch/session lifecycle direction |
| **6** | **Private Discovery Direction** | Definiuje discovery architecture (encrypted registry vs mailbox vs hybrid). Bez niego nie ma discovery — user nie może znaleźć compute. Blokuje: ComputeOffering, DiscoveryQuery/Response, scoped offering IDs, bootstrap coordinator. | Nowy moduł `privai-discovery` (nie istnieje), NXMS mailbox (transport base) | Discovery Protocol Spec, ComputeOffering Spec | Faza 8: Discovery infrastructure. | Opus (direction doc) | Direction doc z discovery mode, bootstrap coordinator spec, minimal discovery data, architecture tradeoffs |
| **7** | **MarketplaceBatchTx fate** | MarketplaceBatchTx jest w Transaction enum — jego obecność sugeruje że marketplace jest core. Blokuje: agent clarity, clean V0 codebase. Nie blokuje kodu V0 per se, ale zatruwa kontekst. | `privai-chain/src/tx.rs` (Transaction enum), `privai-node/src/node.rs` (sign_marketplace_batch), `privai-wallet/src/operator.rs` | Marketplace Types Cleanup doc | Faza 1: #[deprecated] markers. | Operator | Decyzja: deprecated, renamed, feature-gated, legacy module? |
| **8** | **Kill criteria dla Phase 1 automated operator** | Bez kill criteria Phase 1 staje się permanentnym central point. Blokuje: operatorless transition plan, Phase 2 start. | `nxms-escrow-orchestrator` (Phase 1 implementation) | Operatorless Escrow Protocol Bridge spec | Faza 7: Automated operator. Phase 2 start. | Operator | Decyzja: max N miesięcy / M settlements bez manual override |
| **9** | **Production/escrow phase naming** | "Phase 1" = "Compatibility Bridge" (production) i "Automated Operator" (escrow). Collision powoduje agent confusion. Blokuje: clear communication, task prompts. | Wszystkie V0 docs, task logs, prompts | Wszystkie future docs (muszą używać correct naming) | Nie blokuje kodu, ale blokuje clear task issuance. | Operator + Opus | Decyzja: rename production phases (0-5) lub escrow phases (0/1/2) na inne nazwy |
| **10** | **ResourceClass granularity** | ComputeLeasePolicy i ComputeOffering potrzebują ResourceClass. Ale ile GPU classes? Jak nazwać? Pola struct nie są zdefiniowane. Blokuje: ComputeLeasePolicy, ComputeOffering, discovery filtering. | `privai-chain` (nowy typ), escrow (lease policy), discovery (offering filtering) | Compute Lease Object Spec, Private Discovery Direction | Faza 3: ComputeLeaseEscrow (pole lease_policy_commit). Faza 8: Discovery. | Opus (direction doc) | Decyzja: ile GPU classes, czy CPU tiers, czy composite resource descriptions |

---

## 2. Critical Path

Kolejność w której decyzje muszą zapaść:

```
Krok 1 (Operator — natychmiast):
  ├── Max supply PVA (u64 vs u128)
  ├── MarketplaceBatchTx fate (#[deprecated]?)
  ├── Kill criteria dla Phase 1
  └── Production/escrow phase naming

Krok 2 (Opus — po 2026-04-15):
  ├── T-032: Operatorless Escrow Direction
  ├── T-033: Identity Model Direction
  └── T-035: Metering Protocol Direction

Krok 3 (Opus — po Krok 2):
  ├── ComputeLeaseEscrow SpendPolicy spec
  ├── Private Discovery Direction
  └── ResourceClass granularity

Krok 4 (Opus — po Krok 3):
  ├── Pro-rata note split spec
  ├── Receipt schema spec
  └── Metering trust/challenge spec

Krok 5 (Kod — po Krok 4):
  ├── Faza 0a: 10 typów (LedgerAmount, enums, variants)
  ├── Faza 3: ComputeLeaseEscrow validation
  ├── Faza 4: Receipt infrastructure
  └── Faza 5: Pro-rata implementation
```

---

## 3. Parallelizable Decisions

Decyzje które można rozwiązywać równolegle:

```
Równolegle 1 (Operator — teraz):
  Max supply PVA || MarketplaceBatchTx fate || Kill criteria || Phase naming

Równolegle 2 (Opus — po 2026-04-15):
  T-032 (Operatorless Escrow) || T-033 (Identity) || T-035 (Metering)
  
Równolegle 3 (Opus — po T-032/T-033/T-035):
  ComputeLeaseEscrow spec || Private Discovery Direction || ResourceClass granularity

NIE równolegle:
  Pro-rata spec MUSI być po ComputeLeaseEscrow spec
  Receipt schema MUSI być po Metering Direction
  Discovery MUSI być po Identity Direction (scoped IDs)
  ComputeLeasePolicy MUSI być po ResourceClass granularity
```

---

## 4. Decisions Not Blocking Yet

Decyzje ważne, ale nie blokujące teraz:

| Decision | Why Not Blocking Yet |
|---|---|
| **SettlementFormula details** | Pro-rata jest future. LinearProRata jest "likely default" ale nie frozen. Nie blokuje Faza 0a typów. |
| **SessionKey/EpochKey lifecycle** | Identity Model Direction definiuje direction. Exact lifecycle jest future. Nie blokuje Faza 0a. |
| **Runtime privacy class enforcement** | PrivacyClass enum istnieje. Enforcement jest future. Nie blokuje Faza 0a. |
| **Relay onion routing protocol** | Transport base działa (NXMS + Tor SOCKS5). Onion routing jest Phase 8+. Nie blokuje nic teraz. |
| **Traffic padding / timing obfuscation** | Metadata hardening jest future. Nie blokuje nic teraz. |
| **Bond/slash mechanism** | Future strengthening. Nie blokuje nic teraz. |
| **Third-party receipt attestation** | Phase 2+ concern. Nie blokuje Phase 1. |
| **DHT/gossip for discovery** | Network size nie jest znana. Decyzja jest future. Nie blokuje bootstrap coordinator. |
| **Proof system expansion** | Halo2 scaffold istnieje. Full privacy proof jest Phase 6+. Nie blokuje Faza 0a typów. |
| **MCP/RAG implementation** | Direction jest frozen. Implementation jest blocked na direction baseline completion. Nie blokuje kodu V0. |

---

## 5. Immediate Next 3 Decisions

### Decyzja 1: Max supply PVA (Operator)
**Dlaczego TERAZ:** Blokuje LedgerAmount (u64 vs u128). Blokuje aPVA precision freeze. Blokuje SettlementFormula.
**Minimum:** Decyzja: "Max supply PVA ≤ 18,446,744,073 PVA" → u64. Lub większa → u128.
**Owner:** Operator.
**Jeśli nie zdecydujemy:** LedgerAmount jest stuck. aPVA jest stuck. Pro-rata jest stuck.

### Decyzja 2: MarketplaceBatchTx fate (Operator)
**Dlaczego TERAZ:** Marketplace types zatruwają agenci drift. Pierwszy bezpieczny krok (#[deprecated] na 3 typach + 3 komentarze) jest ready do wdrożenia.
**Minimum:** Decyzja: "Deprecated teraz, feature gate później."
**Owner:** Operator.
**Jeśli nie zdecydujemy:** Agenci nadal widzą marketplace types jako "core." V0 framing erode.

### Decyzja 3: T-032 Operatorless Escrow Direction (Opus)
**Dlaczego po 2026-04-15:** To jest fundament całego V0 settlement model. Bez niego nie wiadomo: jak automated operator działa, jak protocol waliduje receipts, jak operator znika, jakie są Phase 0/1/2 mechanics.
**Minimum:** Direction doc z: Phase definitions, automated operator validation rules, operatorless protocol bridge, bootstrap operator reality, dispute quorum replacement.
**Owner:** Opus (po 2026-04-15).
**Jeśli nie zdecydujemy:** Cała ścieżka settlement (Faza 3-7) jest blocked.

---

## Final Self-Check

- **Czy czytałem legacy docs:** NIE
- **Czy edytowałem pliki inne niż output:** NIE (tylko OUTPUT_XIAOMI.md)
- **Czy czytałem kod:** TAK (w poprzednich audytach P-T040–P-T048)
- **Czy definiowałem wire formaty:** NIE
