# privAI V0: Marketplace Types Fate Audit

**Status:** technical audit / legacy type classification
**Data:** 2026-04-11
**Źródło:** P-T044-XIAOMI
**Zakres:** co zrobić z marketplace-era typami w kodzie żeby nie zatruwały V0 private compute

---

## 1. Marketplace Type Map

### Definicje typów

| Symbol | File | Current Purpose | Referenced By | Active or Dead | V0 Conflict Level |
|---|---|---|---|---|---|
| `MarketplaceBatchTx` | `privai-chain/src/tx.rs:313` | Batch settlement dla marketplace receipts (operator podpisuje, ticket_nullifiers, summary) | `tx.rs` (Transaction enum), `node.rs` (sign_marketplace_batch), `wallet/operator.rs` (publish_settlement_batch), `proof/batch.rs` (rejected as unsupported), `ledger.rs` (test), `lib.rs` (re-export) | **Active** — kompiluje się, jest w Transaction enum, ma tests | **HIGH** — V0 mówi "MarketplaceBatchTx nie definiuje produktu." Jego obecność w Transaction enum sugeruje że marketplace jest core. |
| `SpendPolicy::MarketplaceSettlement` | `privai-chain/src/note.rs:80` | 3-party settlement (buyer/seller/moderator pk_hash + timeout) | `note.rs` (SpendPolicy enum, tag(), encode()), `decode.rs` (deserialization), `ledger.rs` (explicit rejection in FullPrivacy mode), `lib.rs` (re-export) | **Semi-active** — kompiluje się, jest w SpendPolicy enum, ale ledger odrzuca go w FullPrivacy mode z komunikatem "MarketplaceSettlement unsupported in FullPrivacy" | **HIGH** — V0 odrzuca marketplace settlement. Obecność w SpendPolicy enum jest source of confusion. Ale ledger już go odrzuca. |
| `SpendPolicyTag::MarketplaceSettlement` | `privai-chain/src/note.rs:17` | Tag `0x02` dla MarketplaceSettlement | `note.rs` (SpendPolicy::tag()), `decode.rs` (deserialization dispatch), `ledger.rs` (policy_tag check) | **Semi-active** — tag istnieje w enum, jest używany w dispatch | **MEDIUM** — tag jest w enum ale deserialization dla niego jest edge case. |
| `marketplace_context: ContextId` | `privai-chain/src/tx.rs:242` | Pole w `SettlementTx` — kontekst marketplace | `tx.rs` (SettlementTx struct, encode(), tx_signing_hash()) | **Active** — SettlementTx używa go | **MEDIUM** — SettlementTx jest innym tx type niż MarketplaceBatchTx. `marketplace_context` jest polem, nie osobnym typem. |
| `sign_marketplace_batch()` | `privai-node/src/node.rs:253` | Node podpisuje MarketplaceBatchTx kluczem Falcon | `node.rs` (metoda na PrivaiNode), `wallet/operator.rs` (wywoływana?) | **Active** — metoda istnieje, kompiluje się | **HIGH** — Node jako "MarketplaceOperator" podpisujący batch jest antywzorzec dla V0. |
| `publish_settlement_batch()` | `privai-wallet/src/operator.rs:81` | Wallet buduje MarketplaceBatchTx z receipts | `operator.rs` (metoda na SmallPaymentOperator), `operator.rs` (Receipt, SpendGrant usage) | **Active** — metoda istnieje, kompiluje się | **HIGH** — To jest implementacja marketplace settlement flow. V0 odrzuca marketplace. |
| `SmallPaymentOperator` | `privai-wallet/src/operator.rs` | Struktura zarządzająca marketplace receipts i batch settlement | `operator.rs` (cały moduł — 240 linii) | **Active** — moduł istnieje, kompiluje się | **HIGH** — Cały moduł jest marketplace-era. V0 nie używa small payments w tym sensie. |
| `merchant_commit` / `merchant_sig` | `privai-chain/src/small_payments.rs:33,111` | Commitment i podpis merchanata w Receipt i SpendGrant | `small_payments.rs` (Receipt, SpendGrant, SettlementBatchSummary, ServicePaymentPolicy) | **Active** — używane w small payments rail | **MEDIUM** — "merchant" jest marketplace terminology. V0 mówi "compute miner." Ale small payments rail może być osobną rzeczą od compute lease. |
| `operator_commit` / `operator_sig` | `privai-chain/src/small_payments.rs:154` / `tx.rs:317` | Commitment i podpis operatora w SettlementBatchSummary / MarketplaceBatchTx | `small_payments.rs`, `tx.rs`, `node.rs`, `operator.rs` | **Active** | **HIGH** — "operator" w marketplace kontekście = "marketplace operator." V0 mówi "operator jest bridge, nie canonical." |
| `buyer` / `seller` / `moderator` | `note.rs:81-83` | Pola w SpendPolicy::MarketplaceSettlement | `note.rs` (SpendPolicy), `decode.rs` | **Semi-active** — pola istnieją ale ledger odrzuca | **HIGH** — V0 odrzuca buyer/seller/moderator jako product framing. |
| `Buyer` / `Merchant` / `Operator` | `privai-chain/src/escrow.rs:47-54` | SignerRole enum dla Escrow2of3 | `escrow.rs` (required_signers, target_recipient), `ledger/escrow.rs` (signer identification, validation), `escrow_builder.rs`, `node.rs` | **ACTIVE — core** — to jest fundament istniejącego escrow | **NISKI dla Escrow2of3** — V0 Phase 0/1 używa Escrow2of3 jako bridge. Nazwy (Buyer/Merchant/Operator) są legacy ale mechanika jest solidna. ComputeLeaseEscrow używa nowych nazw (User/Miner/Protocol). |
| `TX_TYPE_MARKETPLACE_BATCH` | `privai-chain/src/tx.rs` | Constant dla marketplace batch tx type | `tx.rs`, `operator.rs`, `node.rs` | **Active** | **HIGH** — Tx type constant sugeruje że marketplace batch jest valid tx type. |
| `ledger_rejects_marketplace_batch_double_spend` test | `privai-ledger/src/ledger.rs:576` | Test że ledger odrzuca double-spend na MarketplaceBatchTx | `ledger.rs` | **Active test** | **LOW** — Test jest dla bezpieczeństwa ledger, nie dla product direction. Ale jego obecność sugeruje że MarketplaceBatchTx jest "supported enough to test." |
| `fullprivacy_marketplacesettlement_unsupported` test | `privai-ledger/src/ledger.rs:955` | Test że FullPrivacy mode odrzuca MarketplaceSettlement | `ledger.rs` | **Active test** | **LOW** — Test potwierdza że ledger odrzuca MarketplaceSettlement. To jest ZGODNE z V0. |

---

## 2. Options Analysis

### Option 1: Do Nothing

**Benefit:** Zero risk. Zero compatibility break. Wszystko działa jak dziś.

**Risk:** Agenci widząc `MarketplaceBatchTx` w `Transaction` enum, `sign_marketplace_batch()` w node, `publish_settlement_batch()` w wallet — będą budować na marketplace mental model. V0 framing erode przez accumulated drift.

**Compatibility:** Pełna.

**Agent-drift impact:** **HIGH.** Każdy nowy agent zobaczy marketplace types i pomyśli "to jest core."

### Option 2: Docs-Only Warning

**Benefit:** Zero code risk. V0 docs mówią "MarketplaceBatchTx nie definiuje produktu. V0 używa ComputeLeaseEscrow."

**Risk:** Agenci nie czytają docs przed kodem. Warning jest ignorowany. Marketplace types nadal kompilują się i są w Transaction enum.

**Compatibility:** Pełna.

**Agent-drift impact:** **MEDIUM.** Lepsze niż do-nothing, ale niewystarczające.

### Option 3: `#[deprecated]` Attribute

**Benefit:** Kompilator generuje warning. Agenci widząc `#[deprecated]` wiedzą że to nie jest V0. Zero code break (tylko warning). Easy to implement.

**Risk:** Warning jest ignorowany przez `#[allow(deprecated)]`. Deprecated types nadal istnieją w kodzie. Ale jest to silniejszy sygnał niż docs-only.

**Compatibility:** Pełna (deprecated = warning, nie error).

**Agent-drift impact:** **LOW-MEDIUM.** Silniejszy sygnał niż docs-only.

### Option 4: Feature Gate (`#[cfg(feature = "legacy-marketplace")]`)

**Benefit:** Marketplace types nie kompilują się domyślnie. Trzeba explicitnie włączyć feature. Agenci nie widzą typów w default build.

**Risk:** Testy które używają marketplace types nie kompilują się bez feature. CI musi testować z feature enabled. Więcej complexity.

**Compatibility:** Partial — domyślny build nie ma marketplace types.

**Agent-drift impact:** **LOW.** Agenci w default build nie widzą marketplace types.

### Option 5: Move to Legacy Module

**Benefit:** Marketplace types żyją w `privai-chain/src/legacy/` lub `privai-chain/src/marketplace_legacy/`. Czysta separacja. Kod jest czytelny: "to jest legacy."

**Risk:** Ścieżki import się zmieniają (`use privai_chain::tx::MarketplaceBatchTx` → `use privai_chain::legacy::MarketplaceBatchTx`). Wszystkie references muszą być zaktualizowane. Breaks existing code paths.

**Compatibility:** Partial — import paths się zmieniają.

**Agent-drift impact:** **LOW.** Agenci widząc `legacy/` w ścieżce wiedzą że to nie jest V0.

### Option 6: Rename / Repurpose

**Benefit:** Np. `MarketplaceBatchTx` → `SmallPaymentSettlementTx`. Nazwa jest neutralna, nie "marketplace."

**Risk:** Wszystkie references się zmieniają. Serialization format może się zmienić jeśli nazwa jest częścią canonical encoding. Breaking change.

**Compatibility:** **ZERO** — rename łamie wszystkie import paths i potentially serialization.

**Agent-drift impact:** **LOW** — ale koszt jest duży.

### Option 7: Delete

**Benefit:** Zero marketplace types w kodzie. Najczystsze rozwiązanie.

**Risk:** **KATASTROFALNE.** MarketplaceBatchTx jest w Transaction enum — usunięcie łamie enum exhaustiveness. Node ma `sign_marketplace_batch()` — usunięcie łamie node. Wallet ma `publish_settlement_batch()` — usunięcie łamie wallet. Ledger ma tests używające MarketplaceBatchTx — usunięcie łamie tests. `SettlementTx.marketplace_context` — usunięcie łamie SettlementTx.

**Compatibility:** **ZERO** — katastrofalne.

**Agent-drift impact:** **ZERO** — ale koszt jest nieakceptowalny.

---

## 3. Recommendation

### Rekomendacja: Sekwencyjna — Option 3 teraz, Option 4 lub 5 później

**Krok 1 (teraz): `#[deprecated]` na kluczowych typach**

```rust
#[deprecated(note = "V0 legacy: MarketplaceBatchTx is not part of private compute network. Use ComputeLeaseEscrow.")]
pub struct MarketplaceBatchTx { ... }

#[deprecated(note = "V0 legacy: MarketplaceSettlement is not part of private compute network. Use ComputeLeaseEscrow.")]
pub enum SpendPolicy { ... MarketplaceSettlement { ... } ... }

#[deprecated(note = "V0 legacy: Use sign_compute_lease() instead.")]
pub fn sign_marketplace_batch() { ... }
```

**Krok 2 (po Phase M1): Feature gate lub move to legacy module**

Po tym jak ComputeLeaseEscrow jest implemented i testowany, marketplace types mogą być przeniesione do legacy module lub feature-gated.

**Dlaczego nie delete, rename, lub feature gate teraz:**

1. MarketplaceBatchTx jest w `Transaction` enum — usunięcie/rename łamie enum exhaustiveness
2. `sign_marketplace_batch()` jest w node — usunięcie łamie node
3. `publish_settlement_batch()` jest w wallet — usunięcie łamie wallet
4. Ledger ma tests używające MarketplaceBatchTx — usunięcie łamie tests
5. Small payments rail (Receipt, SpendGrant, SettlementBatchSummary) może być osobną rzeczą od compute lease — nie należy usuwać bez pewności

**Dlaczego `#[deprecated]` teraz:**

1. Zero break — tylko warning
2. Silny sygnał dla agentów — "to nie jest V0"
3. Łatwe do wdrożenia — dodanie attribute
4. Odwracalne — można usunąć deprecated później
5. Nie blokuje development — wszystko działa jak dziś

---

## 4. Minimal Safe Cleanup

### Co można zrobić jako pierwszy bezpieczny krok (po akceptacji operatora):

1. **`#[deprecated]` na `MarketplaceBatchTx`** — 1 linia zmiany, zero break
2. **`#[deprecated]` na `SpendPolicy::MarketplaceSettlement`** — 1 linia, zero break
3. **`#[deprecated]` na `sign_marketplace_batch()`** — 1 linia, zero break
4. **Komentarz V0 w `Transaction` enum** — "MarketplaceBatch jest legacy. V0 używa TransferNoteTx z ComputeLeaseEscrow SpendPolicy."
5. **Komentarz V0 w `SpendPolicy` enum** — "MarketplaceSettlement jest legacy. V0 używa ComputeLeaseEscrow."
6. **Komentarz V0 w `SignerRole` enum** — "Buyer/Merchant/Operator są legacy nazwami. V0 ComputeLeaseEscrow używa User/Miner/Protocol."

**To są 6 zmian komentarzy/attributes. Zero break. Zero risk. Silny sygnał.**

### Co NIE robić teraz:

1. Nie usuwać MarketplaceBatchTx z Transaction enum
2. Nie zmieniać nazw
3. Nie przenosić do legacy module
4. Nie feature gate'ować
5. Nie zmieniać small_payments.rs (Receipt, SpendGrant, SettlementBatchSummary — mogą być osobną rzeczą)

---

## 5. Co z Small Payments Rail?

`small_payments.rs` zawiera:
- `ServicePaymentPolicy` — polityka płatności za usługi
- `SpendGrant` — grant na wydatki
- `Receipt` — receipt za usługę
- `SettlementBatchSummary` — podsumowanie batch settlement

**Czy to jest marketplace?**

Częściowo. Terminologia (merchant, operator, marketplace) jest marketplace-era. Ale **mechanika** (receipts, grants, settlement batch) może być reuse'owalna dla V0 small payments / service payments.

**Rekomendacja:** Nie usuwać small_payments.rs teraz. Oznaczyć terminologię (merchant → service_provider? operator → settlement_coordinator?) jako future rename. Ale mechanika może żyć jako osobny rail od compute lease.

---

## 6. Red Lines

1. **Nie usuwać MarketplaceBatchTx z Transaction enum.** Łamie enum exhaustiveness. Wszystkie match statements przestają kompilować.

2. **Nie usuwać SpendPolicy::MarketplaceSettlement.** Łamie SpendPolicy enum. Ledger ma explicit rejection — to jest wystarczająca ochrona.

3. **Nie zmieniać nazw buyer/merchant/operator w SignerRole.** Escrow2of3 jest bridge — te nazwy są legacy ale mechanika jest solidna. ComputeLeaseEscrow używa nowych nazw.

4. **Nie usuwać small_payments.rs.** Receipt/SpendGrant/SettlementBatchSummary mogą być osobną rzeczą od compute lease. Usunięcie łamie operator.rs i tests.

5. **Nie twierdzić że marketplace types są "usunięte" po dodaniu #[deprecated].** Deprecated ≠ removed. Deprecated = warning.

6. **Nie feature gate'ować bez testów.** Feature gate wymaga CI configuration i test coverage z both feature on i off.

7. **Nie rename'ować bez audytu serialization.** Jeśli marketplace_context jest częścią canonical encoding SettlementTx, rename łamie serialization.

8. **Nie zmieniać ledger rejection of MarketplaceSettlement.** To jest już zgodne z V0 — "MarketplaceSettlement unsupported in FullPrivacy." To jest dobry guard.

---

## Summary

| Typ | Conflict Level | Pierwszy krok | Drugi krok |
|---|---|---|---|
| `MarketplaceBatchTx` | HIGH | `#[deprecated]` | Feature gate lub legacy module |
| `SpendPolicy::MarketplaceSettlement` | HIGH | `#[deprecated]` | Feature gate lub legacy module |
| `SpendPolicyTag::MarketplaceSettlement` | MEDIUM | Komentarz V0 | Feature gate |
| `marketplace_context` w SettlementTx | MEDIUM | Komentarz V0 | Future rename (jeśli SettlementTx jest używany) |
| `sign_marketplace_batch()` | HIGH | `#[deprecated]` | Przenieść do legacy module |
| `publish_settlement_batch()` | HIGH | `#[deprecated]` | Przenieść do legacy module |
| `SmallPaymentOperator` | HIGH | `#[deprecated]` + komentarz | Przenieść do legacy module |
| `merchant_commit/sig` w small_payments | MEDIUM | Komentarz V0 | Future rename |
| `operator_commit/sig` w small_payments | MEDIUM | Komentarz V0 | Future rename |
| `buyer/seller/moderator` w MarketplaceSettlement | HIGH | Już odrzucone przez ledger | `#[deprecated]` na SpendPolicy |
| `Buyer/Merchant/Operator` w SignerRole | NISKI | Komentarz V0 (legacy names) | ComputeLeaseEscrow używa nowych nazw |
| `TX_TYPE_MARKETPLACE_BATCH` | HIGH | `#[deprecated]` | Feature gate |
| ledger rejection test | LOW | Zostawić — jest zgodne z V0 | Zostawić |

---

**Czy edytowano pliki:** NIE (poza zapisem tego pliku)
**Czy czytano kod:** TAK — `privai-chain/src/tx.rs` (MarketplaceBatchTx, Transaction enum), `privai-chain/src/note.rs` (SpendPolicy, SpendPolicyTag), `privai-chain/src/small_payments.rs` (Receipt, SpendGrant, SettlementBatchSummary), `privai-ledger/src/ledger.rs` (MarketplaceSettlement rejection, MarketplaceBatchTx test), `privai-ledger/src/escrow.rs` (SignerRole, validate_escrow_auth), `privai-node/src/node.rs` (sign_marketplace_batch), `privai-wallet/src/operator.rs` (publish_settlement_batch, SmallPaymentOperator)
**Czy czytano legacy docs:** NIE
**Czy zdefiniowano wire formaty:** NIE
**Czy odpowiedź jest technical audit:** TAK
