# T-058-XIAOMI — Final Accepted Decisions Draft

**Status:** final candidate decision document  
**Data:** 2026-04-11  
**Źródło:** synteza audytów P-T040–P-T051, task outputs T-046–T-054, Codex review T-046, Gemini scan T-057, canonical V0 docs  
**Zakres:** finalny candidate decision set dla privAI V0 — co akceptować, co odkładać, co odrzucać

---

## 1. Verdict

V0 private compute model jest spójny i gotowy do dalszej pracy. Audyty kodu potwierdzają że istniejąca baza (Escrow2of3, Falcon, NXMS, Halo2 scaffold) jest solidna i może służyć jako fundament. Strategia "add new alongside old" jest poprawna — zero impact na existing code, nowe V0 primitives obok.

Korekty z recenzji (Codex, Gemini): nie zamrażać tagów numerycznych, nie traktować Opusa jako gatekeepera, nie wdrażać typów przed specami, oddzielić potrzebę typu od wyboru backing type (u64 vs u128).

Poniższy dokument jest candidate input dla canonical V0 decision document. Nie jest sam w sobie canonical.

---

## 2. Accepted Now

Decyzje które mogą być zaakceptowane natychmiast:

- **V0 = post-kwantowa sieć prywatnego obliczeniowego AI.** Nie marketplace. Privacy jest produktem. Compute jest dostawą. PVA jest incentywem. Chain jest settlement. Transport jest tarczą.

- **Escrow2of3 jest bridge (Phase 0/1).** Niezmieniony. Frozen rule table. 20+ tests. Nie dotykać.

- **RecoveryRelease jest operatorless anchor.** Buyer+Merchant sign, no Operator. Timeout enforced. Code-confirmed. Jedyna już-operatorless akcja.

- **Strategia: add new alongside old.** Nowe V0 primitives obok existing code. ZERO zmian w Escrow2of3, Amount14, consensus, transport.

- **ComputeLeaseEscrow jest nowym SpendPolicy variantem (nie rozszerzeniem Escrow2of3).** Osobna walidacja. Osobne tests. Rozszerzenie Escrow2of3 zmienia commitment hash = katastrofalne.

- **Exit node = opt-in, nigdy default.** V0 master explicitnie definiuje.

- **V0-only MCP/RAG.** Legacy docs = quarantine. Golden tests (30 pytań) egzekwują.

- **Amount14 nie jest typem ekonomii ledgerowej.** Jest tylko dla proof/plaintext lane (LWE encryption). Ledger economics używa u64 (już potwierdzone w kodzie: TxCore.fee, LiteOutputNote.amount, Receipt.amount, SettlementBatchSummary).

- **LedgerAmount jest needed.** Potrzebny typ dla escrow/settlement economics. Backing type (u64 vs u128) jest blocked by operator decyzja o max supply.

- **MarketplaceBatchTx i SpendPolicy::MarketplaceSettlement nie są usuwane.** Oznaczenie jako #[deprecated] jest pierwszym bezpiecznym krokiem. Usunięcie łamie Transaction enum exhaustiveness.

- **Falcon PK = ValidatorRoleKey (semantyczna zmiana).** Zero zmian w kodzie. Zmiana jest tylko w dokumentacji i komentarzach.

- **Hidden root + role keys = docelowy model identity.** Koncept zaakceptowany. Implementacja struktury opóźniona do Identity Model Direction doc.

- **Recovery timeout jest enforced.** Test `reject_recovery_before_timeout` jest code-confirmed.

- **Settlement jest deterministyczny.** Frozen rule table. Same inputs → same outputs.

---

## 3. Accepted Directionally, But Not Frozen In Code

Koncepty zaakceptowane kierunkowo, ale nie zamrożone w kodzie:

- **ComputeLeaseEscrow SpendPolicy** — koncept (nowy variant, osobna walidacja) jest zaakceptowany. Dokładne pola, tag numeryczny, commitment scheme — wymagają spec.

- **NetworkMode enum** — 4 wartości (Isolated, NxmsOnly, TorGated, InternetExit) są zaakceptowane. V0 master definiuje. Ale dodanie do kodu czeka na Faza 0a (po spec gate).

- **SettlementMode enum** — 2 wartości (AllOrNothing, ProRata) są zaakceptowane. Ale ProRata execution wymaga spec (note split).

- **PrivacyClass enum** — 4 wartości (VM, Container, Sandbox, ConfidentialRuntime) są zaakceptowane. Ale per-class guarantees (co miner widzi) wymagają Runtime Privacy Direction.

- **RoleType enum** — 5 wartości (Validator, ComputeMiner, Relay, Mailbox, ExitNode) są zaakceptowane. V0 master definiuje.

- **HiddenRootCredential struct** — koncept (root seed → role key derivation) jest zaakceptowany. Ale struktura w kodzie czeka na Identity Model Direction.

- **RoleKey struct** — koncept (role + falcon_pk) jest zaakceptowany. Ale struktura w kodzie czeka na Identity Model Direction.

- **TargetRecipient::Two variant** — koncept (2 outputs dla pro-rata) jest zaakceptowany. Ale walidacja dla 2 outputs wymaga spec.

- **EscrowAction::ProRataSplit** — koncept (nowa akcja dla partial delivery) jest zaakceptowany. Ale execution wymaga spec (note split mechanics).

- **ComputeLeaseReceipt typ** — koncept (osobny od small payments Receipt) jest zaakceptowany. Ale schema wymaga Metering Protocol Direction.

- **LedgerAmount backing type** — potrzeba LedgerAmount jest zaakceptowana. Wybór u64 vs u128 — blocked by operator decyzja o max supply. Nie zamrażać dopóki max supply nie jest znany.

- **Numeric enum tags** — koncept (nowe tagi 0x04+) jest zaakceptowany. Ale konkretne wartości wymagają protocol/version registry confirmation. Nie zamrażać.

- **Marketplace types deprecation** — kierunek (deprecated, nie deleted) jest zaakceptowany. Ale timing i dokładna metoda — blocked by operator.

---

## 4. Deferred / Not Accepted Yet

Elementy odłożone — nie akceptować teraz:

- **SettlementFormula details** — brak frozen formula. LinearProRata jest "likely default" ale edge cases nie zdefiniowane. Zależy od aPVA precision.

- **SessionKey struct** — brak session lifecycle. MeteringSession nie istnieje. Za wcześnie.

- **EpochKey struct** — brak rotation protocol. Phase 7 concern. Za wcześnie.

- **ScopedOfferingId type** — brak discovery protocol. Meaningless bez discovery.

- **ComputeOffering struct** — brak discovery protocol. Meaningless bez discovery.

- **HeartbeatStatus enum** — brak heartbeat protocol. Meaningless bez metering.

- **DiscoveryQuery / DiscoveryResponse structs** — brak discovery protocol.

- **EscrowAction::TimeoutAutoRefund** — Phase 2 concern. Refund wymaga sygnatariuszy w Phase 0/1.

- **Automated operator implementation** — blocked by Operatorless Escrow Direction spec.

- **Pro-rata note split implementation** — blocked by Pro-rata Note Split spec.

- **Receipt production infrastructure** — blocked by Metering Protocol Direction spec.

- **Private discovery infrastructure** — blocked by Private Discovery Direction spec.

- **Hidden root implementation in code** — blocked by Identity Model Direction spec.

- **Relay node role implementation** — brak onion routing protocol. Phase 8+.

- **Traffic padding / timing obfuscation** — metadata hardening jest future.

---

## 5. Rejected Or Explicitly Not Allowed

- **Extending Escrow2of3 z nowymi polami.** Zmienia commitment hash = existing notes nieważne. ODRZUCONE.

- **Usuwanie MarketplaceBatchTx z Transaction enum.** Łamie enum exhaustiveness. ODRZUCONE.

- **Usuwanie SpendPolicy::MarketplaceSettlement.** Łamie SpendPolicy enum. ODRZUCONE.

- **Zmiana Amount14 z u16 na u32/u64.** Zmienia LWE plaintext space = rewrite proof circuit. ODRZUCONE.

- **Zmiana PLAINTEXT_SPACE_P (16384).** Fundamentalny LWE parameter. Zmiana = nowe klucze, nowe ciphertexty, nowy circuit. ODRZUCONE.

- **Zmiana falcon_pk_hash() domain string ("privai:falcon-pk:v0").** Zmiana = zmiana wszystkich hashes w systemie. ODRZUCONE.

- **Zmiana consensus identity (node_pk_hash, validator_pk).** Łamie voting, block building, proposer selection. ODRZUCONE.

- **Traktowanie Opusa jako jedynego gatekeepera decyzji.** "Requires direction/spec/reviewer decision" — każdy qualified reviewer. ODRZUCONE.

- **Traktowanie task output jako canonical doc.** Tasks = workspace. Main V0 folder = accepted docs. ODRZUCONE.

- **RAG ingest legacy docs.** V0-only. Legacy = quarantine. ODRZUCONE.

- **Nowy Transaction variant (ComputeLeaseTx) na teraz.** Za duży scope. ComputeLeaseEscrow jako SpendPolicy wystarczy. ODRZUCONE jako teraz, DEFERRED jako future.

- **Separate settlement layer na teraz.** Za duży scope, duplikacja escrow mechanics. ODRZUCONE jako teraz, DEFERRED jako future.

---

## 6. Required Operator Decisions

Decyzje należące do operatora/projektu, nie do modelu:

1. **Max supply PVA** — u64 (do ~18.4B) czy u128? Determinuje LedgerAmount backing type.

2. **MarketplaceBatchTx / MarketplaceSettlement fate** — #[deprecated] teraz? Feature gate później? Legacy module?

3. **Kill criteria dla Phase 1 automated operator** — max N miesięcy / M settlements bez manual override?

4. **Production/escrow phase naming** — rename żeby uniknąć collision ("Phase 1" = dwa różne rzeczy)?

5. **Czy Faza 0a (10 typów koncepcyjnych) może startować** — typy są additive, zero break, ale operator musi potwierdzić.

6. **Czy MCP Sprint 0 (local skeleton) może startować** — 6 core docs istnieją.

7. **Czy T-032/T-033/T-035 powinny być assigned do reviewera** — wymagają direction/spec/reviewer decision.

---

## 7. Required Direction / Spec Documents

Dokumenty kierunkowe/specyfikacyjne potrzebne przed kodem:

1. **Operatorless Escrow Direction** — Phase 0/1/2 bridge, automated operator validation, operatorless protocol.

2. **Metering Protocol Direction** — Receipt fields, heartbeat, trust model, receipt availability.

3. **Identity Model Direction** — Hidden root, key derivation, Falcon boundaries, epoch/session lifecycle.

4. **ComputeLeaseEscrow SpendPolicy spec** — Fields, validation, commitment, signer rules.

5. **Private Discovery Direction** — Discovery mode, bootstrap coordinator, minimal data.

6. **Runtime Privacy Classes Direction** — VM/container/sandbox granularity, miner visibility.

7. **Pro-rata Note Split spec** — 1→2 output mechanics, settlement formula, rounding.

8. **aPVA Denomination Direction** — Precision evaluation, rounding rules (blocked by max supply).

9. **Receipt Truth Direction** — 3-layer model, fraud vectors (after Metering Direction).

10. **Protocol Versioning Direction** — 12 domains, activation mechanics, tag registry.

---

## 8. Required Code Audits

Audyty kodu potrzebne przed implementacją:

1. **nxms-escrow-orchestrator** — czy state machine jest compatible z receipt validation flow? Czy można użyć jako automated operator base bez centralization risk?

2. **SpendPolicy enum serialization** — czy dodanie nowego variant (tag 0x04) wymaga zmiany serialization format? Czy jest backward compatible?

3. **validate_output_target extensibility** — czy można dodać nową gałąź dla 2-output split bez zmiany existing 1-output validation?

4. **vault TLV format extensibility** — czy nowe tagi (T_HIDDEN_ROOT) są additive bez breaking existing vault files?

5. **Transaction enum exhaustiveness** — czy wszystkie match statements na Transaction obsługują nowy variant (jeśli ComputeLeaseTx kiedyś powstanie)?

---

## 9. Canonical Doc Outline Proposal

Proposed section titles dla `PRIVAI_V0_FINAL_DOMAIN_AND_MIGRATION_DECISIONS_PL.md`:

```text
# PRIVAI_V0_FINAL_DOMAIN_AND_MIGRATION_DECISIONS_PL.md

## 1. Verdict
## 2. Accepted Now
## 3. Accepted Directionally
## 4. Deferred
## 5. Rejected
## 6. Operator Decisions
## 7. Direction/Spec Documents Required
## 8. Code Audits Required
## 9. Minimal Type Set (Faza 0a)
## 10. Type Dependency Graph
## 11. Boundary Freeze List
## 12. Migration Phases
## 13. Anti-Patterns
## 14. Red Lines
## 15. Next Task Order
## 16. Final Self-Check
```

---

## 10. Red Lines

Czego żaden reviewer/model nie powinien robić:

1. Nie traktować task output jako canonical doc.
2. Nie traktować żadnego modelu jako gatekeepera.
3. Nie mówić "AI marketplace."
4. Nie twierdzić że operatorless jest implemented.
5. Nie twierdzić że pro-rata jest implemented.
6. Nie twierdzić że receipt schema jest frozen.
7. Nie twierdzić że hidden root identity istnieje w kodzie.
8. Nie zmieniać Escrow2of3.
9. Nie zmieniać Amount14.
10. Nie zmieniać consensus identity.
11. Nie zmieniać falcon_pk_hash() domain string.
12. Nie usuwać MarketplaceBatchTx z Transaction enum.
13. Nie definiować wire formatów przed spec.
14. Nie czytać legacy docs jako source of truth.
15. Nie merge'ować compute miner i validator.
16. Nie zamrażać tagów numerycznych przed version registry.
17. Nie zamrażać u64 przed max supply decyzją.
18. Nie wdrażać typów przed specami.

---

## 11. Final Self-Check

- **Czy czytałem legacy docs:** NIE
- **Czy task output treated as canonical:** NIE — tasks/ = workspace, main V0 folder = accepted docs
- **Czy edytowałem pliki inne niż output:** NIE (tylko OUTPUT_XIAOMI.md)
- **Czy definiowałem wire formaty:** NIE
- **Czy Opus treated as blocker:** NIE — "requires direction/spec/reviewer decision," nie "Opus must decide"
