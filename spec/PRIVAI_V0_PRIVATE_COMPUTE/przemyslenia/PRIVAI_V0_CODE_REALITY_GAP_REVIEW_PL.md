# privAI V0: Code Reality Gap Review

**Status:** code-reality review / V0 gap map
**Data:** 2026-04-11
**Źródło:** kontynuacja rozmowy z Xiaomi po P-T037
**Zakres:** porównanie V0 direction z aktualną rzeczywistością kodu

Ten dokument zapisuje po polsku pierwsze twarde porównanie architektury V0 z kodem.

To nie jest legacy doc.
To nie jest implementation spec.
To nie jest zgoda na zmiany w kodzie.
To jest mapa luk między `V0 private compute direction` a aktualnym repo.

---

## 1. Ocena Końcowa

Kierunek V0 jest poprawny, ale odległość od kodu jest duża.

Aktualny kod implementuje solidny, przetestowany system escrow `2-of-3` z Falcon signatures, walidacją Stage A/B i PC-BFT consensus. To jest realna baza techniczna. Jednocześnie ten kod nadal nosi ciężar starego modelu: Buyer / Merchant / Operator, marketplace-oriented escrow, operator jako canonical co-signer.

V0 direction mówi o innym produkcie:

```text
private compute lease
metering receipts
operatorless settlement
hidden-root identity
private discovery
runtime privacy classes
```

Tych elementów w kodzie jeszcze nie ma.

Wniosek:

```text
Direction jest właściwy.
Implementacja jest na początku drogi.
Nie wolno udawać, że V0 jest już zaimplementowane.
```

---

## 2. Najsilniejsze Decyzje Architektoniczne Potwierdzone Kodem

### 2.1 Escrow 2-of-3 Jest Solidną Bazą

Kod escrow w `privai-chain` i `privai-ledger` jest dobrze zaprojektowany:

- frozen rule table,
- kanoniczna kolejność sygnatariuszy,
- walidacja output targets,
- anti-siphoning,
- recovery timeout enforcement,
- Falcon signature verification,
- bogata walidacja ledger.

To oznacza, że V0 nie startuje od zera.

Nowy obowiązek:

```text
V0 musi budować na istniejącej mechanice albo świadomie wprowadzić nowy policy/escrow type.
```

### 2.2 Stage A/B Boundary Jest Potwierdzone

`EscrowApprovalBundle` działa jako granica między control plane i on-chain settlement plane.

To dobrze pasuje do V0:

- Phase 1 automated operator może działać w control plane,
- Stage B / ledger validation może pozostać stabilna na początku,
- logika decyzyjna może ewoluować bez natychmiastowego rozbijania ledger mechanics.

### 2.3 Falcon Jest Realnie Wszędzie

Falcon jest używany dla:

- consensus votes,
- escrow authorization,
- transaction signing,
- node identity.

To wzmacnia post-quantum kierunek systemu.

Napięcie:

```text
Kod traktuje Falcon public key jako identity.
V0 mówi: Falcon jest narzędziem podpisu, nie publiczną tożsamością.
```

To będzie wymagać migration path.

### 2.4 NXMS Mailbox Już Istnieje

Mailbox ma:

- HTTP push/pull/ack,
- SQLite storage,
- rate limiting,
- notification system.

To jest dobry fundament pod przyszłe mailbox-based private discovery.

Napięcie:

```text
Obecny mailbox to transport bazowy.
V0 "transport is the shield" wymaga metadata hardening, onion/multi-hop i envelope privacy.
```

### 2.5 Halo2 Proof Scaffold Istnieje

Kod ma realny scaffold:

- LWE amount chip,
- nullifier chip,
- note commitment chip,
- noise class chip,
- `PrivaiTxSkeletonCircuit`.

To jest punkt startowy, ale nie dowód pełnej prywatności transferu.

---

## 3. Najbardziej Niebezpieczne Luki

### 3.1 Receipt Truth Nie Istnieje W Kodzie

Nie ma:

- compute lease receipt struct,
- receipt validation,
- metering protocol,
- challenge/response,
- receipt availability model.

Istniejące `Receipt` w small payments nie oznacza compute lease metering.

Wniosek:

```text
Receipt truth jest direction-only.
```

To jest największy pojedynczy gap V0.

### 3.2 Pro-Rata Note Splitting Nie Istnieje

Aktualny escrow jest all-or-nothing.

`target_recipient()` prowadzi do jednego recipienta.
`validate_output_target()` oczekuje, że outputy idą do dozwolonego jednego celu.

V0 pro-rata wymaga:

```text
1 escrow input -> 2 output notes
miner share + user remainder
```

To jest redesign note mechanics, nie mała zmiana arytmetyczna.

### 3.3 Identity Jest Obecnie Falcon-Centric

Aktualnie:

- `PQCIdentity` ładuje Falcon PK/SK z vault,
- node identity to Falcon PK hash,
- consensus i escrow używają Falcon PK hash jako identyfikatora.

V0 wymaga:

```text
hidden root -> role keys -> epoch keys -> session keys
```

To jest fundamentalna migracja identity model.

### 3.4 MarketplaceBatchTx Nadal Istnieje

W kodzie istnieją:

- `MarketplaceBatchTx`,
- `SpendPolicy::MarketplaceSettlement`,
- buyer/seller/moderator semantics.

V0 mówi, że marketplace nie definiuje produktu.

Ryzyko:

```text
Agenci widząc te typy w kodzie będą wracać do starego modelu.
```

Potrzebna jest jawna decyzja:

- deprecated,
- renamed,
- repurposed,
- isolated legacy rail.

### 3.5 Operator Jest Nadal Canonical W Release/Refund

Current reality:

- Release wymaga Buyer + Operator,
- Refund wymaga Merchant + Operator,
- RecoveryRelease wymaga Buyer + Merchant i jest jedyną operatorless ścieżką.

V0 target:

```text
operatorless normal settlement
```

To jest daleka przyszłość, nie current implementation.

### 3.6 Amount14 Jest Potencjalnym Krytycznym Bottleneckiem

`Amount14` ma max:

```text
16,383
```

V0 rozważa:

```text
1 PVA = 10^12 aPVA
```

Jeżeli `Amount14` miałby reprezentować escrow amount w aPVA, mismatch jest fundamentalny.

Ryzyko:

```text
max escrow = 16,383 aPVA = 0.000000000016383 PVA
```

To jest absurdalnie małe dla compute lease.

Otwarte pytanie:

```text
Czy Amount14 jest tylko dla privacy/plaintext LWE rail, czy dla wszystkich kwot escrow?
```

To musi być rozstrzygnięte przed aPVA freeze.

---

## 4. V0 Direction Vs Code Reality

| Element V0 | Status w kodzie | Ocena |
|------------|-----------------|-------|
| Escrow 2-of-3 | zaimplementowane i testowane | potwierdzone |
| RecoveryRelease operatorless | buyer + merchant, no operator | potwierdzone |
| Stage A/B boundary | EscrowApprovalBundle i node validation | potwierdzone |
| All-or-nothing settlement | target recipient = one role | potwierdzone |
| Falcon PQC | używany szeroko | potwierdzone, ale Falcon = identity w kodzie |
| Halo2 proof | scaffold chips/circuit | potwierdzone jako scaffold |
| NXMS mailbox | push/pull/ack | baza, nie full shield |
| Operator co-signs Release/Refund | wymagany | potwierdzone |
| MarketplaceBatchTx | istnieje | source of confusion |
| MarketplaceSettlement | istnieje | legacy type w kodzie |
| Operatorless normal escrow | tylko direction | nie implemented |
| Pro-rata split | brak | nie implemented |
| Metering receipts | brak dla compute lease | nie implemented |
| Compute lease policy | brak | nie implemented |
| Hidden root identity | brak | nie implemented |
| Scoped offering IDs | brak | nie implemented |
| Private discovery | brak | nie implemented |
| Runtime privacy classes | brak | nie implemented |
| Reliability scoring | brak | nie implemented |
| Automated operator | scaffold/orchestrator only | nie implemented |
| Challenge/response | brak | nie implemented |
| aPVA 10^12 | Amount14 max 16,383 | fundamentalny mismatch |
| Five separated node roles | brak jako pełny model | nie implemented |
| Protocol versioning domains | minimal constants only | nie implemented |

---

## 5. Co Jest Realne Dzisiaj

Realne i użyteczne jako baza:

- escrow 2-of-3,
- RecoveryRelease,
- Stage A/B boundary,
- Falcon signatures,
- NXMS mailbox,
- Halo2 scaffold,
- escrow orchestrator jako kandydat na Phase 1 bridge.

To są elementy, na których V0 może budować.

---

## 6. Co Wymaga Fundamentalnej Zmiany

### 6.1 Nowy SpendPolicy Type

`Escrow2of3` nie jest naturalnym miejscem na:

- lease policy commitment,
- receipt requirements,
- settlement formula reference,
- pro-rata split.

Rekomendacja kierunkowa:

```text
Rozważyć nowy SpendPolicy variant: ComputeLeaseEscrow.
```

### 6.2 Amount Type

`Amount14` może być zbyt małe dla compute lease economics.

Przed aPVA freeze trzeba ustalić:

- czy `Amount14` jest rail-specific,
- czy escrow potrzebuje większego amount type,
- jak LWE plaintext space wpływa na value representation.

### 6.3 Identity Redesign

Migracja:

```text
Falcon PK = identity
```

do:

```text
hidden root + scoped role/session/epoch keys
```

to jedna z największych zmian architektonicznych.

### 6.4 Receipt System

Compute lease receipts są nowym obiektem protokołowym.

Trzeba zdefiniować:

- receipt truth,
- receipt availability,
- metering protocol,
- challenge/response,
- settlement binding.

### 6.5 Marketplace Cleanup

`MarketplaceBatchTx` i `MarketplaceSettlement` wymagają jawnej decyzji.

Bez tego agenci będą wracać do marketplace.

---

## 7. Najważniejsze Ryzyka

### Ryzyko 1: Amount14 Bottleneck

Największy techniczny blocker.

Jeśli compute lease settlement ma używać aPVA, `Amount14` jest niewystarczające.

### Ryzyko 2: Pro-Rata Na Istniejącym Note Systemie

Current note validation zakłada jeden recipient target.

Pro-rata wymaga dwóch recipientów.

### Ryzyko 3: Identity Migration

Falcon PK hash jest dziś identyfikatorem systemowym.

V0 identity wymaga warstw scoped/hidden.

### Ryzyko 4: Marketplace Dead Code

Marketplace types w kodzie będą infekować agentów i future tasks, jeśli nie zostaną jawnie oznaczone.

---

## 8. Decyzje Blokujące Przed Kodem

1. Czy `Amount14` jest dopuszczalne dla escrow amounts?
2. Czy potrzebny jest większy amount type dla V0 compute lease?
3. Co robimy z `MarketplaceBatchTx`?
4. Co robimy z `SpendPolicy::MarketplaceSettlement`?
5. Czy tworzymy `ComputeLeaseEscrow` jako nowy SpendPolicy variant?
6. Czy Falcon PK staje się role key pod hidden root?
7. Czy `small_payments::Receipt` może być reuse'owany, czy compute lease dostaje osobny receipt type?
8. Czy `nxms-escrow-orchestrator` ewoluuje w Phase 1 automated operator?
9. Jaki scope ma Halo2 dla compute lease escrow?
10. Czy NXMS mailbox jest oficjalnym transportem discovery queries?

---

## 9. Pytania Do Opusa / Operatora

1. Czy `Amount14` jest wystarczający dla escrow amounts, czy potrzebny jest większy type?
2. Czy `MarketplaceBatchTx` i `MarketplaceSettlement` mają być deprecated, renamed czy repurposed?
3. Czy `ComputeLeaseEscrow` powinien być nowym `SpendPolicy`, zamiast rozszerzać `Escrow2of3`?
4. Czy `nxms-escrow-orchestrator` jest bazą dla Phase 1 automated operator?
5. Czy `small_payments::Receipt` ma być reuse'owany dla compute lease metering?

---

## 10. Finalny Wniosek

V0 direction jest architektonicznie dobry, ale nie jest blisko implementacji.

Aktualny kod daje mocną bazę:

- escrow,
- Falcon,
- Stage A/B,
- mailbox,
- proof scaffold.

Ale V0 core nadal jest przyszłością:

- compute lease policy,
- metering receipts,
- receipt truth,
- pro-rata,
- hidden-root identity,
- private discovery,
- operatorless normal settlement.

Najbardziej pilne jest nie pisanie kodu, tylko zamknięcie pięciu decyzji:

```text
Amount14 / amount type
MarketplaceBatchTx fate
ComputeLeaseEscrow policy type
Falcon identity migration
orchestrator -> automated operator
```

Bez tych decyzji V0 docs będą poprawne, ale nieprzekładalne na kod.

---

## 11. Self-Check

```text
Czy użyto legacy docs: NIE
Czy analizowano kod: TAK, według raportu Xiaomi
Czy edytowano kod: NIE
Czy zdefiniowano wire formaty: NIE
Poziom dokumentu: code-reality gap review, nie implementation spec
Status: materiał wejściowy do kolejnych V0 direction/spec tasks
```
