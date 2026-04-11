# P-T046-XIAOMI — Decision Matrix Draft

**Status:** decision matrix / first V0 decision classification
**Data:** 2026-04-11
**Źródło:** synteza wszystkich audytów P-T040–P-T045
**Zakres:** klasyfikacja decyzji V0 na podstawie dotychczasowych audytów

---

## 1. Decision Matrix

| Decision | Recommended Status | Reason | Supporting Docs | Blockers | Risk If Frozen Too Early | Risk If Delayed | Owner |
|---|---|---|---|---|---|---|---|
| **V0 product model (private compute, not marketplace)** | FROZEN_CANDIDATE | V0 master explicitly defines. Direction jest canonical. | V0 Direction Reset | Brak | Brak — to jest już frozen direction. | V0 framing erodes if not frozen. | Operator (already decided) |
| **Add-new-alongside-old strategy** | STRONG_CANDIDATE | Migration Architecture PL potwierdza: "add new alongside old, don't refactor in place." Kod ma existing Escrow2of3, existing tests, existing validation. Nowe V0 primitives obok. | Migration Architecture PL, Code Reality Gap Review PL | Brak | Brak — strategy jest additive. | Brak nowych typów = brak V0 progress. | Operator + Xiaomi |
| **Escrow2of3 as bridge (Phase 0/1)** | FROZEN_CANDIDATE | Kod jest solidny: 757 linii walidacji, 20+ tests, frozen rule table, Stage A/B boundary. V0 Phase 1 automated operator używa existing mechanics. | SpendPolicy Audit PL, Code Reality Gap Review PL | Brak | Brak — to jest existing code, nie nowa decyzja. | Brak bridge = brak path do operatorless. | Operator (existing code) |
| **RecoveryRelease as operatorless anchor** | FROZEN_CANDIDATE | Kod potwierdza: required_signers = Buyer+Merchant (no Operator). Test `reject_recovery_before_timeout` jest code-confirmed. To jest jedyna już-operatorless akcja. | SpendPolicy Audit PL, V0 Direction Reset §3 | Brak | Brak — to jest existing code. | Brak anchor = brak proof że operatorless jest możliwe. | Operator (existing code) |
| **Amount14 proof lane only** | STRONG_CANDIDATE | Audyt P-T041: kod JUŻ używa u64 dla ledger economics (fee, lite notes, receipts, settlement). Amount14 jest tylko w proof/plaintext lane (RecipientBoxPlaintext, AuxWitness, TransferInputWitness). NIE zmieniać. | Amount14 Audit PL | Max supply PVA decyzja (u64 vs u128) | Brak — Amount14 niezmieniony. | Brak separacji = Amount14 bottleneck dla escrow economics. | Xiaomi + Operator |
| **LedgerAmount (u64) for economics** | STRONG_CANDIDATE | Audyt P-T041: kod JUŻ używa u64 w TxCore.fee, LiteOutputNote.amount, Receipt.amount, SettlementBatchSummary. LedgerAmount = u64 jest naturalne rozszerzenie. | Amount14 Audit PL | Max supply PVA decyzja | Brak — alias na existing type. | Brak LedgerAmount = escrow amount limited do 16,383. | Operator (supply decision) |
| **u64 vs u128 for LedgerAmount** | BLOCKED_BY_OPERATOR | Zależy od max supply PVA. u64 wystarcza do ~18.4B PVA. Jeśli supply > 18.4B, u128 required. Operator musi zdecydować ceiling. | Amount14 Audit PL | Decyzja operatora o max supply | Brak — u64 jest default. | Jeśli u64 za małe później = migration kosztowna. | Operator |
| **ComputeLeaseEscrow as new SpendPolicy (tag 0x04)** | STRONG_CANDIDATE | Audyt P-T042: SpendPolicy enum jest designed do dodawania variants. Nowy variant z tag 0x04 = ZERO impact na Escrow2of3. Osobna validation function. Osobne tests. | SpendPolicy Audit PL | Pro-rata note split spec | Brak — additive variant. | Brak nowej SpendPolicy = brak V0 escrow path. | Opus (spec) + Xiaomi (type) |
| **Pro-rata as future path** | CANDIDATE | V0 mówi "split should be expected for timed compute." Ale pro-rata wymaga: TargetRecipient::Two, nową akcję ProRataSplit=0x04, nową walidację, nową note split mechanics. To jest future, nie teraz. | SpendPolicy Audit PL, V0 Direction Reset §3 | Pro-rata note split spec | Pro-rata variant dodany za wcześnie = placeholder bez implementation. | Brak pro-rata = all-or-nothing dla compute lease = za gruboziarniste. | Opus (spec) |
| **MarketplaceBatchTx fate** | BLOCKED_BY_OPERATOR | V0 odrzuca marketplace. Ale MarketplaceBatchTx jest w Transaction enum, ma sign_marketplace_batch() w node, publish_settlement_batch() w wallet. Pierwszy krok: #[deprecated]. Potem: feature gate lub legacy module. | Marketplace Types Audit PL | Decyzja operatora | Deprecated za wcześnie = warning noise. | Marketplace types zatruwają agenci drift. | Operator |
| **MarketplaceSettlement fate** | BLOCKED_BY_OPERATOR | V0 odrzuca marketplace settlement. Ledger JUŻ odrzuca w FullPrivacy mode ("MarketplaceSettlement unsupported in FullPrivacy"). Pierwszy krok: #[deprecated]. | Marketplace Types Audit PL | Decyzja operatora | Deprecated za wcześnie = warning noise. | Marketplace types zatruwają agenci drift. | Operator |
| **Falcon PK as ValidatorRoleKey (semantyczna zmiana)** | STRONG_CANDIDATE | Audyt P-T043: obecny Falcon PK = validator role key. ZERO zmian w kodzie. Zmiana jest tylko semantyczna: docs + komentarze. Consensus, escrow, transport niezmienione. | Identity Migration Audit PL | Brak | Brak — to jest komentarz, nie kod. | Brak semantycznej zmiany = Falcon jest nadal "identity" w głowach agentów. | Xiaomi (komentarze) |
| **HiddenRootCredential additive later** | CANDIDATE | V0 definiuje hidden root jako fundament identity. Ale root nie istnieje w kodzie. Dodanie root jest additive (nowe TLV tagi w vault, opcjonalne ładowanie). Phase M7+ concern. | Identity Migration Audit PL | Identity Model Direction doc (T-033) | Root dodany za wcześnie = placeholder bez derivation logic. | Brak root = brak fundamentu identity hierarchy. | Opus (spec) |
| **small payments Receipt not reused directly** | STRONG_CANDIDATE | Audyt P-T041 + P-T044: small payments Receipt ma inne pola (merchant_commit, session_commit, grant_commit) niż compute lease potrzebuje (session_id, resource_class, units_delivered). Ale PATTERN (signed receipt + commitment) jest reuse'owalny. ComputeLeaseReceipt jest osobnym typem. | Amount14 Audit PL, Build-Once Types Review PL | Metering Receipt Schema | Brak — separacja jest naturalna. | Brak separacji = dwa use cases w jednym typie = confusion. | Xiaomi + Opus |
| **nxms-escrow-orchestrator as automated operator bridge** | CANDIDATE | Orchestrator ma state machine (Funded→TxSignPending→TxSignedQuorum), ledger observation, proposal building. Najbliższa rzecz do Phase 1 automated operator. Ale orchestrator ma mock signatures i nie ma receipt validation. | Migration Architecture PL | Operatorless Escrow Direction doc (T-032) | Orchestrator oznaczony jako "final operator" = centralization risk. | Brak bridge = brak path do operatorless. | Opus (spec) |
| **NXMS mailbox as discovery transport base** | STRONG_CANDIDATE | Mailbox ma HTTP push/pull/ack, SQLite, rate limiting. To jest realna baza dla mailbox-based discovery queries. V0 mówi "private/encrypted/credential-gated discovery." Mailbox jest naturalnym transportem. | Migration Architecture PL, Code Reality Gap Review PL | Private Discovery Direction doc | Brak — to jest existing code, nie nowa decyzja. | Brak transport base = discovery musi budować transport od zera. | Opus (spec) |
| **VM as default privacy class** | CANDIDATE | V0 mówi "miner should not learn plaintext workload. VM = strongest isolation." VM jako default jest zgodne z "privacy is the product." Ale granularity klas (co dokładnie miner widzi per class) nie jest zdefiniowana. | V0 Direction Reset §2 | Runtime Privacy Classes Direction doc | VM default bez specyfikacji = overclaimed privacy guarantee. | Brak default = użytkownik nie wie jaką prywatność dostaje. | Opus (spec) |
| **Exit node opt-in only** | FROZEN_CANDIDATE | V0 master explicitnie mówi "internet exit is not default. It is a separate explicit role/capability." To jest zdefiniowane i jasne. | V0 Direction Reset §2 | Brak | Brak — to jest direction-level, nie implementation. | Brak = exit node jest accidentally default = privacy leak. | Operator (already decided) |
| **V0-only MCP/RAG context** | FROZEN_CANDIDATE | Single Source Of Truth Context Plan jest frozen. 8 MCP tools defined. Golden tests defined. V0-only jest canonical policy. | Single Source Of Truth Context Plan, MCP Server Direction | MCP implementation (blocked) | Brak — to jest direction-level, nie implementation. | Brak = legacy docs zatruwają agenci. | Codex (implementation) |

---

## 2. Freeze-Ready Decisions

Decyzje które mogą być prawie zamrożone:

1. **V0 product model** — already frozen in V0 master.
2. **Escrow2of3 as bridge** — code-confirmed, existing code.
3. **RecoveryRelease as operatorless anchor** — code-confirmed, existing code.
4. **Exit node opt-in only** — explicitly defined in V0 master.
5. **V0-only MCP/RAG context** — frozen in context plan.
6. **Amount14 proof lane only** — audyt potwierdził: kod JUŻ używa u64 dla economics.
7. **Falcon PK as ValidatorRoleKey** — semantyczna zmiana, zero code impact.
8. **small payments Receipt not reused directly** — separacja jest naturalna.

---

## 3. Decisions That Must Wait

Decyzje których nie wolno jeszcze zamrażać:

1. **u64 vs u128** — blocked by operator decyzja o max supply.
2. **Pro-rata note split mechanics** — blocked by spec.
3. **SettlementFormula details** — blocked by aPVA freeze.
4. **SessionKey/EpochKey** — blocked by identity spec.
5. **ScopedOfferingId/ComputeOffering** — blocked by discovery spec.
6. **HeartbeatStatus/MeteringUnits** — blocked by metering spec.
7. **DiscoveryQuery/DiscoveryResponse** — blocked by discovery spec.
8. **EscrowAction::TimeoutAutoRefund** — blocked by operatorless spec.
9. **VM privacy class granularity** — blocked by runtime privacy spec.

---

## 4. Operator Decisions

Decyzje które wymagają operatora, nie modelu:

1. **Max supply PVA** — u64 (do ~18.4B) czy u128? Blokuje aPVA freeze.
2. **MarketplaceBatchTx fate** — deprecated, renamed, feature-gated, legacy module?
3. **MarketplaceSettlement fate** — deprecated?
4. **Kill criteria dla Phase 1 automated operator** — ile miesięcy/settlements?
5. **Czy #[deprecated] na marketplace types teraz** — first safe cleanup?

---

## 5. Opus Decisions

Decyzje które powinien zatwierdzić Opus:

1. **Operatorless Escrow Direction doc (T-032)** — definiuje Phase 0/1/2 bridge, automated operator validation, operatorless protocol.
2. **Identity Model Direction doc (T-033)** — definiuje hidden root, role keys, session keys, epoch keys, Falcon boundaries.
3. **Metering Protocol Direction doc (T-035)** — definiuje receipts, heartbeats, trust model, challenge/response direction.
4. **Private Discovery Direction doc** — definiuje discovery architecture (encrypted registry vs mailbox vs hybrid).
5. **Runtime Privacy Classes Direction doc** — definiuje VM/container/sandbox granularity i miner visibility per class.
6. **ComputeLeaseEscrow SpendPolicy spec** — definiuje fields, validation, commitment.
7. **Pro-rata note split spec** — definiuje 1→2 output mechanics.
8. **Production/escrow phase naming** — rename żeby uniknąć collision (Phase 1 = Compatibility Bridge vs Automated Operator).

---

## 6. Code Audit Decisions

Decyzje które wymagają dalszego code audit:

1. **SpendPolicy enum extensibility** — audyt DONE (P-T042): jest extensible. Tag 0x04 additive.
2. **Amount14 scope** — audyt DONE (P-T041): proof lane only. Ledger economics używa u64.
3. **Identity usage across codebase** — audyt DONE (P-T043): Falcon PK = identity everywhere. Migration path defined.
4. **Marketplace types references** — audyt DONE (P-T044): MarketplaceBatchTx jest active w Transaction enum, node, wallet. MarketplaceSettlement odrzucony przez ledger.
5. **validate_output_target for 2 outputs** — audyt DONE (P-T042): current validation requires all outputs go to one recipient. Pro-rata wymaga nowej validation path.
6. **vault TLV format extensibility** — audyt z P-T043: nowe tagi (T_HIDDEN_ROOT = 0x0010) są additive. Existing tags (3,4,5,6) niezmienione.
7. **nxms-escrow-orchestrator as automated operator base** — audyt NEEDED: sprawdzić czy orchestrator state machine jest compatible z receipt validation flow.

---

## 7. Final Recommendation

Co powinno trafić do `PRIVAI_V0_FINAL_DOMAIN_AND_MIGRATION_DECISIONS_PL.md`:

### Decyzje do zamrożenia (8):

1. V0 = private compute network, not marketplace
2. Escrow2of3 as Phase 0/1 bridge (niezmieniony)
3. RecoveryRelease as operatorless anchor (niezmieniony)
4. Amount14 = proof lane only (niezmieniony)
5. LedgerAmount = u64 dla economics (nowy alias)
6. Falcon PK = ValidatorRoleKey (semantyczna zmiana)
7. Exit node = opt-in only
8. V0-only MCP/RAG context

### Decyzje do zatwierdzenia przez operatora (5):

1. Max supply PVA (u64 vs u128)
2. MarketplaceBatchTx fate (#[deprecated] teraz?)
3. MarketplaceSettlement fate (#[deprecated] teraz?)
4. Kill criteria dla Phase 1
5. First safe cleanup — 6 zmian (3x #[deprecated] + 3x komentarze)

### Decyzje do zatwierdzenia przez Opusa (8):

1. T-032: Operatorless Escrow Direction
2. T-033: Identity Model Direction
3. T-035: Metering Protocol Direction
4. Private Discovery Direction
5. Runtime Privacy Classes Direction
6. ComputeLeaseEscrow SpendPolicy spec
7. Pro-rata note split spec
8. Phase naming clarification

### Decyzje do dodania teraz (14 typów):

1. LedgerAmount (u64 alias)
2. ResourceClass enum
3. PrivacyClass enum
4. NetworkMode enum (FROZEN_CANDIDATE)
5. SettlementMode enum
6. ComputeLeasePolicy struct
7. ComputeLeaseReceipt struct
8. HiddenRootCredential struct
9. RoleKey struct
10. RoleType enum
11. TargetRecipient::Two variant
12. EscrowAction::ProRataSplit (0x04)
13. SpendPolicyTag::ComputeLeaseEscrow (0x04)
14. SpendPolicy::ComputeLeaseEscrow struct

### Decyzje do delay (9 typów):

1. SettlementFormula enum
2. SessionKey struct
3. EpochKey struct
4. ScopedOfferingId type
5. ComputeOffering struct
6. HeartbeatStatus enum
7. DiscoveryQuery struct
8. DiscoveryResponse struct
9. EscrowAction::TimeoutAutoRefund (0x05)

---

## Final Self-Check

- Czy czytałem legacy docs: **NIE**
- Czy czytałem kod: **TAK** (w poprzednich audytach P-T040–P-T045)
- Czy edytowałem pliki inne niż output: **NIE** (tylko OUTPUT_XIAOMI.md)
- Czy definiowałem wire formaty: **NIE**
- Czy to jest decision matrix, a nie implementation spec: **TAK**
