# privAI V0: Build-Once Domain Types Review

**Status:** candidate type review / build-once strategy validation
**Data:** 2026-04-11
**Źródło:** P-T045-XIAOMI
**Zakres:** przegląd każdego proponowanego typu pod kątem potrzebności, finalności, timing, overengineering

---

## 1. Candidate Type Review

| Type | Purpose | Status | Why | Blockers | Code Audit Needed | Privacy Risk | Migration Risk |
|---|---|---|---|---|---|---|---|
| **LedgerAmount** | Kwota w aPVA na poziomie ledger. U64 alias dla escrow/settlement economics. | **KEEP** | Audyt P-T041 potwierdził: kod JUŻ używa u64 dla fees (TxCore.fee), lite notes (LiteOutputNote.amount), receipts (Receipt.amount), settlement (SettlementBatchSummary). LedgerAmount = u64 jest naturalne rozszerzenie existing pattern. Amount14 zostaje dla proof lane. | Decyzja o max supply PVA (u64 vs u128). Ale u64 wystarcza do ~18.4B PVA — wystarczające na długo. | `privai-chain/src/primitives.rs` — sprawdzić czy Amount14 jest jedynym amount type. Audyt DONE w P-T041. | ZERO — jest to alias na existing type. | MINIMALNY — nowy alias, nie zmienia existing code. |
| **ResourceClass** | Opis zasobu: `{ gpu_class, cpu_tier, ram_mb, vram_mb, storage_class }`. | **KEEP** | V0 master §2 definiuje compute jako GPU/CPU/RAM/VRAM. ComputeOffering musi reklamować jaki zasób jest dostępny. User musi filtrować po resource class. To jest fundamentalny typ dla discovery i lease. | Brak definicji granularity (ile GPU classes? jak nazwać?). Ale struktura jest jasna. | Brak — nie istnieje w kodzie. Nowy typ. | NISKI — resource class jest metadata oferty, nie identity. | MINIMALNY — nowy typ, nie zmienia existing. |
| **PrivacyClass** | Klasa izolacji: `{ VM, Container, Sandbox, ConfidentialRuntime }`. | **KEEP** | V0 master §2 mówi "miner should not learn plaintext workload. Privacy class must say so." ComputeOffering musi deklarować privacy class. User wybiera based on sensitivity. | OPEN z P-T040: granularity nie jest zdefiniowana. Co dokładnie miner widzi per class? To wymaga Runtime Privacy Classes Direction doc. | Brak — nie istnieje w kodzie. | ŚREDNI — jeśli privacy class jest overclaimed, user myśli że ma więcej prywatności niż ma. | MINIMALNY — nowy typ. |
| **NetworkMode** | Dostęp sieciowy: `{ Isolated, NxmsOnly, TorGated, InternetExit }`. | **KEEP** | V0 master §2 explicitnie definiuje 4 tryby. FROZEN_CANDIDATE w P-T040. Kod ma Tor integration (connect_via_tor). To jest najprostszy enum — 4 wartości, zdefiniowane. | Brak. | `nxms-transport/src/tor_net.rs` — sprawdzić czy TorGated i InternetExit są rozróżnialne. Ale to jest minor. | NISKI — network mode jest metadata, nie exposure. | MINIMALNY — nowy enum. |
| **SettlementMode** | Tryb rozliczania: `{ AllOrNothing, ProRata }`. | **KEEP** | V0 master §3 mówi "split is allowed and should be expected for compute leases." P-T042 potwierdził że ComputeLeaseEscrow SpendPolicy potrzebuje settlement_mode pola. To jest 2-value enum — minimalny, potrzebny. | Pro-rata note split nie istnieje. Ale typ jest needed żeby SpendPolicy mógł deklarować tryb. | `privai-ledger/src/escrow.rs` — validate_output_target. Audyt DONE w P-T042. | NISKI. | MINIMALNY — nowy enum. |
| **SettlementFormula** | Wzór obliczania podziału: `{ LinearProRata, ... }`. | **TOO_EARLY** | V0 mówi "pro-rata" ale nie definiuje exact formula. LinearProRata jest direction-level "likely default." Ale edge cases (rounding, 0 delivered, 1 unit delivered) nie są zdefiniowane. Formuła powinna być frozen w spec, nie w type. Dodanie enum variant przed spec = overengineering. | Brak frozen formula. Zależy od aPVA precision freeze. | Brak. | NISKI. | NISKI — ale risk overengineering. |
| **HiddenRootCredential** | Root credential z którego derivowane są wszystkie klucze. Nigdy nie exposeowany. | **KEEP** | V0 master §1 definiuje "hidden root credential" jako fundament identity. P-T040: BLOCKED_BY_CODE_AUDIT — kod traktuje Falcon PK jako identity. Ale typ jest needed bo definiuje co jest root. Nawet jeśli nie jest używany przez consensus na start, struktura definiuje direction. | Audyt użycia Falcon PK hash w całym kodzie. Decyzja czy obecny Falcon PK = role key. | `privai-node/src/identity_provider.rs` — PQCIdentity. `node.rs` — node_pk_hash. Audyt NEEDED. | ŚREDNI — root credential jest najwrażliwszym obiektem. Format musi być secure. | ŚREDNI — zmiana identity model. Ale typ sam w sobie jest additive. |
| **RoleKey** | Klucz roli: `{ role: RoleType, falcon_pk }`. Derived z hidden root. | **KEEP** | V0 master §1 mówi "scoped role/session/epoch identities." Role key jest pierwszym poziomem derivation. Nawet jeśli obecny Falcon PK staje się ValidatorRoleKey semantically, struktura definiuje model. | Jak wyżej — identity audit. Ale struktura jest prosta: rola + klucz. | `identity_provider.rs`, `node.rs`. | NISKi — role key jest mniej wrażliwy niż root. | ŚREDNI — ale additive. |
| **SessionKey** | Klucz sesji: `{ session_id, falcon_pk }`. Per-lease, discarded after session. | **TOO_EARLY** | V0 mówi o session keys ale session lifecycle nie jest zdefiniowany. MeteringSession nie istnieje. ComputeLease nie istnieje. SessionKey bez session jest meaningless. Dodanie go teraz = overengineering. | Metering session spec. Lease lifecycle spec. | Brak. | NISKI. | NISKI — ale risk overengineering. |
| **EpochKey** | Klucz epoki: `{ epoch, role, falcon_pk }`. Rotated periodically. | **TOO_EARLY** | V0 mówi o epoch rotation ale rotation protocol nie jest zdefiniowany. Epoch key bez rotation policy jest meaningless. Phase 7 concern. | Identity Model Direction doc. Epoch rotation policy. | Brak. | NISKI. | NISKI — ale risk overengineering. |
| **ScopedOfferingId** | Identyfikator oferty: `Hash32`. Rotowalny, nie-linked z root. | **TOO_EARLY** | V0 master §5 mówi "scoped-offering-id." Ale discovery protocol nie istnieje. ComputeOffering nie istnieje. ScopedOfferingId bez discovery jest meaningless. | Private discovery protocol. | Brak. | NISKI. | NISKI — ale risk overengineering. |
| **ComputeLeasePolicy** | Polityka lease: resource class, duration, price, privacy, network mode, settlement mode, meter version, timeout. | **KEEP** | P-T042 potwierdził że ComputeLeaseEscrow SpendPolicy potrzebuje `lease_policy_commit` (hash polityki). Polityka jest potrzebna żeby commitment miał co hashować. To jest fundament settlement — bez polityki nie ma settlement formula reference. | Brak frozen fields (które pola są required vs optional). Ale struktura jest directionally clear. | Brak — nowy typ. | NISKI — polityka jest hashed na chain, nie plaintext. | ŚREDNI — polityka jest input do commitment. Zmiana pól = zmiana commitment. Ale to jest additive. |
| **ComputeOffering** | Oferta compute: resource class, cena, privacy, network mode, scoped offering ID, availability. | **TOO_EARLY** | V0 master §5 definiuje ComputeOffering. Ale discovery protocol nie istnieje. ComputeOffering bez discovery jest pustym obiektem. Dodanie go teraz = overengineering. Zależy od Private Discovery Direction doc. | Private discovery protocol. ScopedOfferingId. | Brak. | NISKI. | NISKI — ale risk overengineering. |
| **ComputeLeaseReceipt** | Receipt: session_id, resource_class, units_delivered, heartbeat_status, miner_signature. | **KEEP** | P-T040: OPEN bo receipt infrastructure nie istnieje. Ale: small payments `Receipt` (u64 amount, merchant_commit, session_commit) istnieje i pokazuje że receipt pattern jest w kodzie. ComputeLeaseReceipt jest needed bo jest dowodem settlement. Bez niego settlement jest bez dowodu. Nawet jeśli metering protocol nie istnieje, struktura definiuje co jest evidence. | Metering Protocol Direction doc. Receipt schema freeze. | `privai-chain/src/small_payments.rs` — existing Receipt. Sprawdzić czy pattern jest reuse'owalny. | ŚREDNI — receipt zawiera session metadata. Musi nie zawierać workload contents. | ŚREDNI — receipt jest input do settlement. Zmiana format = zmiana settlement. |
| **HeartbeatStatus** | Status liveness: `{ Active, Missed, Terminated }`. | **TOO_EARLY** | V0 mówi o heartbeats ale heartbeat protocol nie istnieje. HeartbeatStatus bez heartbeat protocol jest meaningless. Dodanie go teraz = overengineering. | Metering protocol direction. Heartbeat spec. | Brak. | NISKI. | NISKI — ale risk overengineering. |
| **DiscoveryQuery** | Zapytanie discovery: resource class filter, max price, min privacy, network mode, max results. | **TOO_EARLY** | V0 mówi o private discovery ale discovery protocol nie istnieje. DiscoveryQuery bez discovery protocol jest meaningless. | Private Discovery Direction doc. | Brak. | NISKI — query jest encrypted. | NISKI — ale risk overengineering. |
| **DiscoveryResponse** | Odpowiedź discovery: matching offerings + timestamp. | **TOO_EARLY** | Jak wyżej. | Private Discovery Direction doc. | Brak. | NISKI. | NISKI — ale risk overengineering. |
| **TargetRecipient::Two** | Nowy variant `Two(SignerRole, SignerRole)` dla pro-rata split (2 outputs). | **KEEP** | P-T042 potwierdził że pro-rata wymaga 2 outputs do różnych odbiorców. Current `TargetRecipient` ma tylko `One` i `Either`. `Two` jest needed dla ComputeLeaseEscrow ProRataSplit. To jest minimalne rozszerzenie existing enum. | Pro-rata note split spec. Ale variant sam w sobie jest prosty. | `privai-ledger/src/escrow.rs` — validate_output_target. Audyt DONE w P-T042. | NISKI. | MINIMALNY — additive variant. |
| **EscrowAction::ProRataSplit** | Nowa akcja `0x04` — 1 input → 2 outputs (miner + user). | **KEEP** | P-T042 potwierdził że pro-rata wymaga nowej akcji. Current EscrowAction ma tylko Release/Refund/RecoveryRelease. ProRataSplit jest needed dla ComputeLeaseEscrow. To jest minimalne rozszerzenie existing enum. | Pro-rata note split spec. Ale variant sam w sobie jest prosty. | `privai-chain/src/escrow.rs` — EscrowAction enum. Audyt DONE w P-T042. | NISKI. | MINIMALNY — additive variant. |
| **EscrowAction::TimeoutAutoRefund** | Nowa akcja `0x05` — protocol-level timeout enforcement. | **TOO_EARLY** | V0 mówi o timeout auto-refund ale to jest Phase 2 concern. W Phase 0/1 refund wymaga sygnatariuszy. TimeoutAutoRefund jest needed dopiero gdy protokół (nie operator) egzekwuje refund. Dodanie go teraz = overengineering. | Operatorless Escrow Direction doc. Phase 2 spec. | Brak. | NISKI. | NISKI — ale risk overengineering. |

---

## 2. Minimal Type Set (pierwsza faza)

### Typy do dodania TERAZ (Faza 0 — types & enums):

```
KEEP — dodaj teraz:
  1.  type LedgerAmount = u64                    — alias, zero impact
  2.  enum ResourceClass { ... }                 — fundament discovery/lease
  3.  enum PrivacyClass { VM, Container, ... }   — fundament lease policy
  4.  enum NetworkMode { Isolated, ... }         — fundament lease policy (FROZEN_CANDIDATE)
  5.  enum SettlementMode { AllOrNothing, ProRata } — fundament escrow
  6.  struct ComputeLeasePolicy { ... }          — fundament escrow commitment
  7.  struct ComputeLeaseReceipt { ... }         — fundament settlement evidence
  8.  struct HiddenRootCredential { root_seed }  — fundament identity (struktura, nie implementacja)
  9.  struct RoleKey { role, falcon_pk }         — fundament identity
  10. enum RoleType { Validator, ComputeMiner, ... } — fundament identity
  11. TargetRecipient::Two(role_a, role_b)       — fundament pro-rata
  12. EscrowAction::ProRataSplit = 0x04          — fundament pro-rata
  13. SpendPolicyTag::ComputeLeaseEscrow = 0x04  — fundament nowego escrow
  14. SpendPolicy::ComputeLeaseEscrow { ... }    — fundament nowego escrow
```

**Łącznie: 14 typów/variants.** To jest cały minimalny zestaw żeby Faza 0 (types & enums) była kompletna.

### Dlaczego te, nie inne:

- **LedgerAmount** — audyt P-T041: kod JUŻ używa u64. Alias jest zero risk.
- **ResourceClass, PrivacyClass, NetworkMode** — fundament ComputeLeasePolicy. Bez nich polityka nie ma co zawierać.
- **SettlementMode** — fundament ComputeLeaseEscrow SpendPolicy. 2-value enum.
- **ComputeLeasePolicy** — fundament commitment. Bez niej lease_policy_commit nie ma co hashować.
- **ComputeLeaseReceipt** — fundament settlement evidence. Bez niego settlement nie ma dowodu.
- **HiddenRootCredential, RoleKey, RoleType** — fundament identity model. Nawet jeśli nie jest używany przez consensus, struktura definiuje direction.
- **TargetRecipient::Two, EscrowAction::ProRataSplit** — fundament pro-rata. P-T042 potwierdził że są needed.
- **SpendPolicyTag/SpendPolicy::ComputeLeaseEscrow** — fundament nowego escrow. P-T042 potwierdził że nowy SpendPolicy jest correct approach.

---

## 3. Types To Delay

```
TOO_EARLY — nie dodawać teraz:
  1.  SettlementFormula enum        — brak frozen formula. Dodaj po spec.
  2.  SessionKey struct             — brak session lifecycle. Dodaj po metering spec.
  3.  EpochKey struct               — brak rotation protocol. Dodaj po identity spec.
  4.  ScopedOfferingId type         — brak discovery protocol. Dodaj po discovery spec.
  5.  ComputeOffering struct         — brak discovery protocol. Dodaj po discovery spec.
  6.  HeartbeatStatus enum          — brak heartbeat protocol. Dodaj po metering spec.
  7.  DiscoveryQuery struct         — brak discovery protocol. Dodaj po discovery spec.
  8.  DiscoveryResponse struct      — brak discovery protocol. Dodaj po discovery spec.
  9.  EscrowAction::TimeoutAutoRefund — Phase 2 concern. Dodaj po operatorless spec.
```

**Łącznie: 9 typów do delay.** Każdy z nich wymaga spec przed dodaniem.

### Dlaczego delay, nie teraz:

- **SettlementFormula** — formula musi być frozen w spec. Dodanie enum variant przed spec = risk że variant jest wrong.
- **SessionKey, EpochKey** — lifecycle nie jest zdefiniowany. Dodanie struktury przed lifecycle = overengineering.
- **ScopedOfferingId, ComputeOffering, DiscoveryQuery, DiscoveryResponse** — discovery protocol nie istnieje. Dodanie typów przed protocol = overengineering.
- **HeartbeatStatus** — heartbeat protocol nie istnieje.
- **TimeoutAutoRefund** — Phase 2 concern. Za wcześnie.

---

## 4. Final Recommendation

### Czy "types first" jest nadal dobrym pomysłem?

**TAK.** Ale z precyzją:

**Types first = 14 typów z §2. Nie 20+ typów.**

```
Faza 0 (types):     14 typów — teraz
Faza 3 (escrow):    SpendPolicy::ComputeLeaseEscrow + validation + tests
Faza 4 (receipts):  ComputeLeaseReceipt + ReceiptStorage + tests
Faza 5 (pro-rata):  TargetRecipient::Two + ProRataSplit + tests
Faza 6 (identity):  HiddenRootCredential + RoleKey + tests
Faza 8 (discovery): ComputeOffering + ScopedOfferingId + DiscoveryQuery + DiscoveryResponse
```

**Każdy późniejszy typ jest dodawany w fazie która go potrzebuje.** Nie wszystko na raz.

### Overengineering risk:

**9 typów z §3 to jest overengineering risk jeśli dodane teraz.** Każdy z nich jest meaningless bez corresponding protocol spec. Dodanie ich teraz = "placeholder types" które mogą być wrong gdy spec się pojawi.

### Buduj to co jest needed TERAZ:

1. **LedgerAmount** — alias u64, zero risk
2. **ComputeLeasePolicy** — fundament escrow commitment
3. **ComputeLeaseReceipt** — fundament settlement evidence
4. **SettlementMode** — fundament escrow
5. **ResourceClass, PrivacyClass, NetworkMode** — fundament polityki
6. **HiddenRootCredential, RoleKey, RoleType** — fundament identity
7. **ComputeLeaseEscrow SpendPolicy** — fundament nowego escrow
8. **ProRataSplit + TargetRecipient::Two** — fundament pro-rata

**To jest 14 typów. To jest "types first" w minimalnej wersji.**

---

## Summary

| Kategoria | Ile | Przykłady |
|---|---|---|
| **KEEP — dodaj teraz** | 14 | LedgerAmount, ResourceClass, PrivacyClass, NetworkMode, SettlementMode, ComputeLeasePolicy, ComputeLeaseReceipt, HiddenRootCredential, RoleKey, RoleType, TargetRecipient::Two, EscrowAction::ProRataSplit, SpendPolicyTag::ComputeLeaseEscrow, SpendPolicy::ComputeLeaseEscrow |
| **TOO_EARLY — delay** | 9 | SettlementFormula, SessionKey, EpochKey, ScopedOfferingId, ComputeOffering, HeartbeatStatus, DiscoveryQuery, DiscoveryResponse, TimeoutAutoRefund |
| **REJECT** | 0 | Brak — wszystkie typy mają sens directionally, niektóre tylko za wcześnie |

---

**Czy edytowano pliki:** NIE (poza zapisem tego pliku)
**Czy czytano kod:** NIE (audyt oparty na poprzednich P-T040, P-T041, P-T042)
**Czy czytano legacy docs:** NIE
**Czy zdefiniowano wire formaty:** NIE
**Czy odpowiedź jest type review:** TAK
