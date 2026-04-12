# T-054-XIAOMI — Final Reviewer Brief Without Opus Gate

**Status:** final reviewer brief  
**Data:** 2026-04-11  
**Zakres:** brief dla dowolnego senior reviewera/modelu (Gemini, Xiaomi, Claude/Opus, Codex)  
**Źródło:** V0 docs, audyty P-T040–P-T051, task outputs, Codex review T-046

---

## 1. Executive Summary

1. **privAI V0 to post-kwantowa sieć prywatnego obliczeniowego AI, nie marketplace.** Privacy jest produktem. Compute jest dostawą. PVA jest incentywem. Chain jest settlement. Transport jest tarczą.

2. **Kod istnieje i działa** — escrow 2-of-3, Falcon signatures, NXMS mailbox, Halo2 scaffold. Ale kod implementuje "stary model" (marketplace-era). V0 buduje na tej bazie, nie obok niej.

3. **Strategia migracji: add new alongside old.** Escrow2of3 zostaje jako bridge. Nowy ComputeLeaseEscrow SpendPolicy (tag 0x04) obok. ZERO zmian w existing Escrow2of3 validation.

4. **Amount14 jest tylko dla proof lane.** Ledger economics JUŻ używa u64 (fees, lite notes, receipts). LedgerAmount = u64 jest naturalne. Max supply PVA decyduje o u64 vs u128.

5. **10 typów do "types first"** — LedgerAmount, NetworkMode, SettlementMode, PrivacyClass, RoleType, SpendPolicyTag::ComputeLeaseEscrow, TargetRecipient::Two, EscrowAction::ProRataSplit, HiddenRootCredential, RoleKey.

6. **Receipt truth jest najważniejszym ryzykiem.** Self-reported receipts z podpisem = honest-but-curious. Phase 2 wymaga challenge/response. To nie jest solved.

7. **8 direction docs brakuje** — Operatorless Escrow, Metering Protocol, Identity Model, Node Roles, Private Discovery, Runtime Privacy, Transport Privacy, Exit Node, aPVA Denomination, Protocol Versioning.

8. **Marketplace types istnieją w kodzie** — MarketplaceBatchTx w Transaction enum, sign_marketplace_batch() w node. Pierwszy krok: #[deprecated] + komentarze. Nie usuwać.

9. **Identity = Falcon PK jako identity.** V0 chce hidden root + scoped keys. Ale obecny Falcon PK staje się ValidatorRoleKey. Consensus identity niezmienione.

10. **MCP/RAG jest V0-only.** Legacy docs = quarantine. Golden tests (30 pytań) egzekwują poprawne odpowiedzi.

---

## 2. What Is Settled

| Topic | Status | Evidence |
|---|---|---|
| V0 product model = private compute, not marketplace | **SETTLED** | V0 master direction doc, canonical |
| Escrow2of3 as Phase 0/1 bridge (niezmieniony) | **SETTLED** | Code-confirmed: 757 lines validation, 20+ tests, frozen rule table |
| RecoveryRelease as operatorless anchor | **SETTLED** | Code-confirmed: Buyer+Merchant sign, no Operator. Test exists. |
| Amount14 = proof lane only, u64 = ledger economics | **SETTLED** | Audit P-T041: code uses u64 for fees, lite notes, receipts, settlement |
| ComputeLeaseEscrow as new SpendPolicy (not extension of Escrow2of3) | **SETTLED** | Audit P-T042: extending Escrow2of3 changes commitment hash = katastrofalne |
| Exit node = opt-in only, never default | **SETTLED** | V0 master explicitly defines |
| V0-only MCP/RAG, legacy quarantine | **SETTLED** | Context plan frozen, MCP direction frozen |
| NetworkMode = 4 values (Isolated/NxmsOnly/TorGated/InternetExit) | **SETTLED** | V0 master explicitly defines |
| SettlementMode = 2 values (AllOrNothing/ProRata) | **SETTLED** | V0 master says "split should be expected" |
| Falcon PK = ValidatorRoleKey (semantic change, zero code impact) | **SETTLED** | Audit P-T043: zero code change, docs/comments only |

---

## 3. What Is Not Settled

| Topic | Why Not Settled | What Needs To Happen |
|---|---|---|
| **Max supply PVA (u64 vs u128)** | Operator decyzja nie podjęta. Blokuje LedgerAmount finalizację. | Operator decyzja: "Max supply PVA ≤ X" |
| **Operatorless Escrow Direction** | Doc nie istnieje (T-032). Phase 0/1/2 bridge nie jest zdefiniowany. | Direction doc: phase definitions, automated operator rules, operatorless protocol bridge |
| **Metering Protocol Direction** | Doc nie istnieje (T-035). Receipt schema, trust model, challenge/response nie są zdefiniowane. | Direction doc: receipt fields, heartbeat, trust model, receipt availability |
| **Identity Model Direction** | Doc nie istnieje (T-033). Hidden root, key derivation, epoch/session lifecycle nie są zdefiniowane. | Direction doc: root definition, derivation rules, Falcon boundaries, lifecycle |
| **Private Discovery Direction** | Doc nie istnieje. Discovery protocol, bootstrap coordinator, architecture tradeoffs nie są zdefiniowane. | Direction doc: discovery mode, bootstrap spec, minimal data, tradeoffs |
| **MarketplaceBatchTx fate** | Operator decyzja nie podjęta. Deprecated/renamed/feature-gated? | Operator decyzja |
| **Kill criteria dla Phase 1 automated operator** | Operator decyzja nie podjęta. Ile miesięcy/settlements? | Operator decyzja |
| **ResourceClass granularity** | Ile GPU classes? Pola struct nie są zdefiniowane. | Direction doc lub operator decyzja |
| **Runtime Privacy Classes granularity** | Co dokładnie miner widzi per class? VM/container/sandbox spec nie jest zdefiniowany. | Direction doc |
| **Production/escrow phase naming** | "Phase 1" = dwa różne rzeczy. Collision powoduje confusion. | Rename decyzja |

---

## 4. Code Reality

Co kod potwierdza:

| Code Element | Confirmation | File |
|---|---|---|
| **Escrow2of3** — 3 variants (Release/Refund/RecoveryRelease), frozen rule table, 12-step validation | **Code-confirmed** | `privai-chain/src/escrow.rs`, `privai-ledger/src/escrow.rs` |
| **RecoveryRelease** — Buyer+Merchant sign, no Operator, timeout enforced | **Code-confirmed** | `privai-ledger/src/escrow.rs:125-140` |
| **All-or-nothing output target** — `TargetRecipient::One` or `Either`, not `Two` | **Code-confirmed** | `privai-chain/src/escrow.rs:78-95` |
| **Amount14 = u16, max 16,383** — used in proof lane (RecipientBoxPlaintext, AuxWitness) | **Code-confirmed** | `privai-chain/src/primitives.rs:26` |
| **u64 for ledger economics** — TxCore.fee, LiteOutputNote.amount, Receipt.amount, SettlementBatchSummary | **Code-confirmed** | `privai-chain/src/tx.rs:96`, `note.rs:351`, `small_payments.rs:117,162` |
| **Falcon PK = identity** — PQCIdentity loads from vault, node_pk_hash = domain_hash("privai:falcon-pk:v0", &[falcon_pk]) | **Code-confirmed** | `privai-node/src/identity_provider.rs`, `node.rs:234-235` |
| **MarketplaceBatchTx exists** — in Transaction enum, has sign_marketplace_batch() in node, publish_settlement_batch() in wallet | **Code-confirmed** | `privai-chain/src/tx.rs:313`, `node.rs:253`, `wallet/operator.rs:81` |
| **MarketplaceSettlement rejected** — ledger rejects in FullPrivacy mode | **Code-confirmed** | `privai-ledger/src/ledger.rs:323-326` |
| **NXMS mailbox** — HTTP push/pull/ack, SQLite, rate limiting | **Code-confirmed** | `nxms-mailbox/src/lib.rs` |
| **Tor SOCKS5** — single-hop connect_via_tor, NOT onion routing | **Code-confirmed** | `nxms-transport/src/tor_net.rs` |
| **Halo2 scaffold** — LWE amount chip, nullifier chip, note commit chip, noise class chip | **Code-confirmed** | `privai-proof/src/halo2/` |

---

## 5. Critical Blockers

| Rank | Blocker | Who Can Resolve | Impact |
|---|---|---|---|
| **1** | Max supply PVA (u64 vs u128) | **Operator** | Blokuje: LedgerAmount, aPVA freeze, SettlementFormula |
| **2** | Operatorless Escrow Direction (T-032) | **Requires direction/spec/reviewer decision** | Blokuje: ComputeLeaseEscrow spec, Pro-rata spec, Automated operator |
| **3** | Metering Protocol Direction (T-035) | **Requires direction/spec/reviewer decision** | Blokuje: Receipt schema, Trust model, Receipt availability |
| **4** | Identity Model Direction (T-033) | **Requires direction/spec/reviewer decision** | Blokuje: Discovery (uses scoped IDs), Session/epoch keys |
| **5** | MarketplaceBatchTx fate | **Operator** | Blokuje: agent clarity, clean V0 framing |

---

## 6. Operator Decisions

Decyzje należące do operatora/projektu, nie do modelu:

1. **Max supply PVA** — u64 (do ~18.4B) czy u128? Determinuje LedgerAmount type.

2. **MarketplaceBatchTx fate** — deprecated, renamed, feature-gated, legacy module? Pierwszy krok: #[deprecated] na 3 typach + 3 komentarze (6 zmian, zero break).

3. **SpendPolicy::MarketplaceSettlement fate** — deprecated? Ledger już odrzuca w FullPrivacy.

4. **Kill criteria dla Phase 1 automated operator** — max N miesięcy / M settlements bez manual override?

5. **Production/escrow phase naming** — rename żeby uniknąć collision ("Phase 1" = dwa różne rzeczy)?

6. **Czy #[deprecated] na marketplace types teraz** — first safe cleanup?

7. **Czy LedgerAmount = u64 jest wystarczające** — jeśli max supply ≤ ~18.4B PVA, tak. Jeśli więcej, u128.

8. **Czy Faza 0a (10 typów) może startować** — typy są additive, zero break. Ale operator musi potwierdzić.

9. **Czy MCP Sprint 0 (local skeleton) może startować** — 6 core docs istnieją. Direction baseline nie jest complete ale MCP może działać na istniejących.

10. **Czy T-032/T-033/T-035 powinny być assigned do reviewera** — wymagają direction/spec/reviewer decision. Nie muszą być Opus — każdy qualified reviewer.

---

## 7. Missing Direction / Spec Decisions

Decyzje wymagające direction/spec/reviewer decision (nie "Opus must decide"):

1. **Operatorless Escrow Direction** — Phase 0/1/2 bridge, automated operator validation rules, operatorless protocol bridge, dispute quorum replacement. **Requires direction/spec/reviewer decision.**

2. **Metering Protocol Direction** — Receipt fields direction, heartbeat direction, trust model (self-reported vs challenge), receipt availability direction. **Requires direction/spec/reviewer decision.**

3. **Identity Model Direction** — Hidden root definition, key derivation rules, Falcon boundaries, epoch/session lifecycle. **Requires direction/spec/reviewer decision.**

4. **ComputeLeaseEscrow SpendPolicy spec** — Fields, validation rules, commitment scheme, signer rules, output target rules. **Requires direction/spec/reviewer decision.**

5. **Private Discovery Direction** — Discovery mode, bootstrap coordinator, minimal data, architecture tradeoffs. **Requires direction/spec/reviewer decision.**

6. **ResourceClass granularity** — Ile GPU classes, CPU tiers, composite descriptions? **Requires direction/spec/reviewer decision.**

7. **Runtime Privacy Classes granularity** — Miner visibility per class (VM/container/sandbox), user selection criteria. **Requires direction/spec/reviewer decision.**

8. **Pro-rata note split spec** — 1 input → 2 output mechanics, settlement formula, rounding rules. **Requires direction/spec/reviewer decision** (after ComputeLeaseEscrow spec).

9. **aPVA Denomination Direction** — Precision evaluation (10^12), rounding rules, interaction with pro-rata. **Requires direction/spec/reviewer decision** (blocked by max supply).

10. **Receipt Truth Direction** — 3-layer model, fraud vectors, evolution from Phase 1 to operatorless. **Requires direction/spec/reviewer decision** (after Metering Direction).

---

## 8. Recommended Next Task Order

```
TERAZ (Operator — może być zrobione natychmiast):
  1. Operator: Max supply PVA decyzja
  2. Operator: MarketplaceBatchTx fate (#[deprecated]?)
  3. Operator: Kill criteria dla Phase 1

TERAZ (Xiaomi/Gemini/Codex — bez Opusa):
  4. T-052: operator decyzje zebrane w jeden doc (jeśli jeszcze nie ma)
  5. T-055: Faza 0a types draft (10 typów — additive, zero break)
  6. T-056: ComputeLeaseEscrow SpendPolicy candidate (fields, validation direction)
  7. T-057: Receipt Truth Direction draft (3-layer model, fraud vectors)

PO OPERATOR DECYZJACH + REVIEWER:
  8. T-058: Metering Protocol Direction draft (po T-057)
  9. T-059: Identity Model Direction draft
  10. T-060: Private Discovery Direction draft (po T-059)
```

---

## 9. Red Lines

Czego żaden reviewer/model nie powinien robić:

1. **Nie traktować task output jako canonical doc.** Tasks = workspace. Main V0 folder = accepted docs. Output musi być reviewed przed promocją.

2. **Nie traktować Opusa jako gatekeepera.** "Requires direction/spec/reviewer decision" — nie "Opus must decide." Każdy qualified reviewer może zatwierdzić.

3. **Nie mówić "AI marketplace."** V0 = private compute network.

4. **Nie twierdzić że operatorless jest implemented.** Tylko RecoveryRelease jest operatorless.

5. **Nie twierdzić że pro-rata jest implemented.** All-or-nothing only.

6. **Nie twierdzić że receipt schema jest frozen.** Direction-level, nie wire format.

7. **Nie zmieniać Escrow2of3.** Frozen. Nowy ComputeLeaseEscrow obok.

8. **Nie zmieniać Amount14.** Proof lane only. LedgerAmount = u64 obok.

9. **Nie usuwać MarketplaceBatchTx z Transaction enum.** Deprecated, nie deleted.

10. **Nie definiować wire formatów przed spec.** Direction → spec → format.

11. **Nie czytać legacy docs jako source of truth.** V0-only.

12. **Nie merge'ować compute miner i validator.** Separate roles.

---

## 10. Final Self-Check

- **Czy czytałem legacy docs:** NIE
- **Czy task output treated as canonical:** NIE — tasks/ = workspace, main V0 folder = accepted docs
- **Czy edytowałem pliki inne niż output:** NIE (tylko OUTPUT_XIAOMI.md)
- **Czy definiowałem wire formaty:** NIE
- **Czy Opus treated as blocker:** NIE — "requires direction/spec/reviewer decision," nie "Opus must decide"
