# PrivAI — Simple 3: Jak przejść z obecnego kodu do V0

**Opis możliwości, funkcje, problemy, zależności**

---

## Możliwości

Obecny kod działa. Ma escrow, konsensus, transport, wallet. Ale jest zbudowany na modelu marketplace. V0 zmienia model na prywatne compute. Trzeba przejść z jednego do drugiego bez łamania tego co działa.

---

## Co zostaje (nie zmieniamy)

### Konsensus — zostaje

**Funkcja:** PC-BFT, bloki co ~30s, validatorzy, stake-weighted voting.

**Dlaczego nie zmieniamy:** Działa. V0 nie zmienia jak bloki powstają. Zmienia co jest w blokach.

### Escrow 2-of-3 — zostaje jako bridge

**Funkcja:** Release (buyer + operator), Refund (merchant + operator), RecoveryRelease (buyer + merchant).

**Dlaczego nie zmieniamy:** Jest przetestowany. 20+ testów. 757 linii walidacji. Zmiana = łamanie existing notes. V0 Phase 1 używa go zamiast.

**Problem:** Jest zbudowany na marketplace (buyer/merchant/operator). Ale mechanika działa — zmieniamy tylko semantykę.

### RecoveryRelease — zostaje jako fundament

**Funkcja:** User + miner podpisują. Bez operatora. Po timeout.

**Dlaczego nie zmieniamy:** To jest JEDYNA akcja która działa bez operatora. To jest dowód że operatorless jest możliwe. To jest template dla całego Phase 2.

### NXMS mailbox — zostaje jako transport

**Funkcja:** Push/pull/ack zaszyfrowanych envelope'ów. SQLite. Rate limiting.

**Dlaczego nie zmieniamy:** Działa. Będzie rozszerzony o discovery queries.

### NXMS transport — zostaje

**Funkcja:** FrodoKEM + XChaCha20Poly1305 + Falcon. Tor SOCKS5. Framed TCP.

**Dlaczego nie zmieniamy:** Post-kwantowe. Działa. Będzie rozszerzony o relay (future).

### Halo2 scaffold — zostaje

**Funkcja:** Chips dla LWE amount, nullifier, note commit, noise class.

**Dlaczego nie zmieniamy:** Jest punktem startowym dla ZK proofs. Będzie rozszerzony o receipt commitment proof.

---

## Co dodajemy (nowe rzeczy)

### Nowy SpendPolicy: ComputeLeaseEscrow

**Funkcja:** Blokuje PVA na compute lease. Ma pola: user_pk_hash, miner_pk_hash, lease_policy_commit, timeout, settlement_mode.

**Problem:** Nie istnieje. Ale SpendPolicy enum jest designed do dodawania variants. Nowy tag (0x04). Osobna walidacja. Zero impact na Escrow2of3.

**Zależy od:** Decyzja o pól. Decyzja o tagu numerycznym.

### Nowy EscrowAction: ProRataSplit

**Funkcja:** Dzieli escrow na 2 outputy: jeden do minera (earned), jeden do usera (reszta).

**Problem:** Nie istnieje. Ale EscrowAction enum jest `#[repr(u8)]`. Nowy variant (0x04). Current `target_recipient()` obsługuje tylko `One` i `Either` — trzeba dodać `Two`.

**Zależy od:** Nowej walidacji. Nowego TargetRecipient variant.

### Nowe typy

```
LedgerAmount = u64          — kwota na chainie (Amount14 zostaje dla LWE)
NetworkMode enum            — 4 wartości: Isolated, NxmsOnly, TorGated, InternetExit
SettlementMode enum         — 2 wartości: AllOrNothing, ProRata
PrivacyClass enum           — 4 wartości: VM, Container, Sandbox, ConfidentialRuntime
RoleType enum               — 5 wartości: Validator, ComputeMiner, Relay, Mailbox, ExitNode
ComputeLeasePolicy struct   — warunki umowy
ComputeOffering struct      — co miner oferuje
ComputeLeaseReceipt struct  — dowód z sesji
HiddenRootCredential struct — fundament tożsamości
RoleKey struct              — klucz roli
```

**Problem:** Żaden z tych typów nie istnieje.

**Zależy od:** Niczego w existing code. Są additive.

### Window protocol

**Funkcja:** Co 60 bloków: challenge → response → PASS/FAIL. Po sesji: aggregate receipt.

**Problem:** Nie istnieje. Ale jest off-chain — nie zmienia chaina.

**Zależy od:** Bloków (jako zegar), kluczy minera, lease policy.

### Compute miner moduł

**Funkcja:** Provisionuje VM, mierzy zasoby, odpowiada na challengi, tworzy receipts.

**Problem:** Nie istnieje. Cały nowy moduł.

**Zależy od:** Kluczy minera, transportu, runtime (VM/container).

---

## Co oznaczamy jako deprecated

### MarketplaceBatchTx

**Dlaczego:** V0 odrzuca marketplace. Typ istnieje w Transaction enum. Oznaczamy `#[deprecated]`. Nie usuwamy — łamie enum exhaustiveness.

### SpendPolicy::MarketplaceSettlement

**Dlaczego:** V0 odrzuca marketplace settlement. Ledger JUŻ go odrzuca w FullPrivacy mode. Oznaczamy `#[deprecated]`.

### sign_marketplace_batch()

**Dlaczego:** V0 odrzuca marketplace. Node nie powinien podpisywać batch settlement. Oznaczamy `#[deprecated]`.

---

## Co zmieniamy semantycznie (zero zmiany w kodzie)

### Falcon PK = ValidatorRoleKey

Obecnie: Falcon PK jest tożsamością.

V0: Falcon PK jest kluczem roli validatora.

**Zmiana:** Komentarze. Nic w kodzie.

### Buyer/Merchant/Operator = User/Miner/Operator

Obecnie: SignerRole ma Buyer (0x00), Merchant (0x01), Operator (0x02).

V0: Te same indeksy. Nowe nazwy w dokumentacji.

**Zmiana:** Komentarze. Nic w kodzie.

---

## Kolejność migracji

```
Krok 1: Oznacz deprecated (6 zmian)
  MarketplaceBatchTx → #[deprecated]
  MarketplaceSettlement → #[deprecated]
  sign_marketplace_batch() → #[deprecated]
  + 3 komentarze V0
  
  Impact: zero. Tylko warning w kompilacji.

Krok 2: Dodaj nowe typy (14 typów)
  LedgerAmount, NetworkMode, SettlementMode, PrivacyClass, RoleType,
  SpendPolicyTag::ComputeLeaseEscrow, TargetRecipient::Two,
  EscrowAction::ProRataSplit, HiddenRootCredential, RoleKey,
  ComputeLeasePolicy, ComputeOffering, ComputeLeaseReceipt
  
  Impact: additive. Nowe types, nowe enums. Zero zmiany w existing.

Krok 3: Nowa SpendPolicy walidacja
  validate_compute_lease_escrow_auth() — osobna funkcja
  Routing: match policy_tag { 0x03 => existing, 0x04 => nowa }
  
  Impact: additive. Osobna funkcja. Osobne tests.

Krok 4: Window protocol (off-chain)
  Challenge generation, response verification, availability, performance
  
  Impact: off-chain. Nie zmienia chaina.

Krok 5: Receipt infrastructure
  ComputeLeaseReceipt production, storage, ZK proof
  
  Impact: nowy moduł. Nie zmienia existing.

Krok 6: Pro-rata execution
  ProRataSplit action, 1→2 output split
  
  Impact: nowa akcja w escrow. Osobna walidacja.

Krok 7: Identity foundation
  Hidden root, role key derivation
  
  Impact: additive. Obecny Falcon PK = validator role key.

Krok 8: Discovery
  Mailbox-based queries, ComputeOffering publish
  
  Impact: additive. Rozszerzenie NXMS mailbox.

Krok 9: Devnet
  End-to-end test
  
  Impact: integration, nie refactoring.
```

---

## Czego NIE robić

```
NIE zmieniać Escrow2of3         — frozen, bridge
NIE zmieniać Amount14            — proof lane only
NIE zmieniać consensus           — frozen
NIE zmieniać falcon_pk_hash()    — frozen hasher
NIE usuwać MarketplaceBatchTx    — deprecated, nie deleted
NIE zmieniać node_pk_hash        — core identifier
NIE definiować wire formatów      — przed spec
NIE implementować onion routing   — future
NIE implementować DHT/gossip      — future
```

---

## Podsumowanie

```
Co zostaje:     konsensus, escrow bridge, recovery, mailbox, transport, halo2
Co dodajemy:    nowy SpendPolicy, nowe akcje, nowe typy, window protocol, miner
Co oznaczamy:   marketplace types → deprecated
Co zmieniamy:   semantycznie (komentarze), zero kodu

Kolejność:      deprecated → types → validation → window → receipt → pro-rata → identity → discovery → devnet

Ryzyko:         minimalne. Większość jest additive. Jedyne ryzyko to ProRataSplit (nowa mechanika note split).

Czas:           zależy od speców. Types mogą być dodane natychmiast. Validation po spec. Window protocol po spec.
```
