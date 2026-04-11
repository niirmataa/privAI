# P-T048-XIAOMI — Minimal Types Freeze Candidate

**Status:** minimal types freeze candidate
**Data:** 2026-04-11
**Źródło:** synteza audytów P-T040–P-T045, Build-Once Types Review PL
**Zakres:** minimalny zestaw typów do "build-once compatible types" — pierwsza fala

---

## 1. Minimal Type Set

| Type | Why Needed Now | Status | Depends On | Safe To Add Now? | Reason | Risk If Added Too Early | Risk If Delayed |
|---|---|---|---|---|---|---|---|
| **`type LedgerAmount = u64`** | Escrow, settlement, receipts potrzebują kwotę większą niż Amount14 (max 16,383). Kod JUŻ używa u64 dla fees, lite notes, receipts, settlement. | STRONG_CANDIDATE | Max supply PVA decyzja (u64 vs u128) | **YES** — alias na existing type. Zero proof impact. Zero code break. | Audyt P-T041: kod JUŻ używa u64 w TxCore.fee, LiteOutputNote.amount, Receipt.amount, SettlementBatchSummary. LedgerAmount = u64 jest naturalne. | ZERO — to jest alias, nie nowy typ. | Escrow amount limited do 16,383. Pro-rata na małych kwotach = niepraktyczne. |
| **`enum NetworkMode { Isolated, NxmsOnly, TorGated, InternetExit }`** | ComputeLeasePolicy potrzebuje deklaracji trybu sieciowego. ComputeOffering reklamuje network mode. User wybiera. | FROZEN_CANDIDATE | Brak | **YES** — V0 master explicitnie definiuje 4 tryby. Zero ambiguity. | V0 Direction Reset §2: "isolated / nxms_only / tor_gated / internet_exit." To jest zdefiniowane i jasne. | ZERO — 4 wartości, zdefiniowane. | Brak = oferta nie deklaruje trybu sieciowego = user nie wie co dostaje. |
| **`enum SettlementMode { AllOrNothing, ProRata }`** | ComputeLeaseEscrow SpendPolicy potrzebuje pola settlement_mode. Bez niego escrow nie wie jak rozliczać. | STRONG_CANDIDATE | Brak | **YES** — 2-value enum. Fundamentalny dla SpendPolicy. | P-T042: ComputeLeaseEscrow potrzebuje settlement_mode pola. P-T046: V0 mówi "split should be expected." | ZERO — 2 wartości. | Brak = ComputeLeaseEscrow nie ma settlement mode = nie wie czy all-or-nothing czy pro-rata. |
| **`enum PrivacyClass { VM, Container, Sandbox, ConfidentialRuntime }`** | ComputeLeasePolicy potrzebuje deklaracji klasy izolacji. ComputeOffering reklamuje privacy class. User wybiera based on sensitivity. | CANDIDATE | Runtime Privacy Classes Direction doc (Opus) | **YES jako enum, NIE jako enforcement** — enum istnieje, ale per-class guarantees wymagają spec. | V0 mówi "privacy class must say so." Ale granularity (co dokładnie miner widzi) nie jest zdefiniowana. | Privacy class enum istnieje ale nie ma enforcement = overclaimed guarantee. | Brak = oferta nie deklaruje prywatności = user nie wie co miner widzi. |
| **`enum RoleType { Validator, ComputeMiner, Relay, Mailbox, ExitNode }`** | Identity model wymaga rozróżnienia ról. HiddenRootCredential derivuje role keys per rola. Validator identity jest osobna od compute miner. | STRONG_CANDIDATE | Brak | **YES** — 5-value enum. V0 master explicitnie definiuje 5 ról. | V0 Direction Reset §2: "five separated node roles." Identity Migration Audit: Falcon PK = ValidatorRoleKey. | ZERO — 5 wartości, zdefiniowane. | Brak = brak separacji ról na poziomie typów. |
| **`struct HiddenRootCredential { root_seed }`** | Fundament identity hierarchy. Root → role keys → session keys. Bez root struktury nie ma definicji co jest źródłem derivacji. | CANDIDATE | Identity Model Direction doc (Opus/T-033). Vault TLV extension. | **YES jako struktura, NIE jako wymaganie** — root jest optional. System działa bez root (backward compatible). | V0 definiuje hidden root. Ale root nie istnieje w kodzie. Struktura definiuje direction. | Root istnieje ale nie jest używany = placeholder. Ale to jest OK — struktura definiuje model. | Brak root = brak fundamentu identity hierarchy. Falcon jest nadal "identity." |
| **`struct RoleKey { role: RoleType, falcon_pk }`** | Identity model wymaga klucza roli. Obecny Falcon PK = ValidatorRoleKey. Nowe role (compute miner) mają nowe klucze. | CANDIDATE | RoleType enum | **YES** — struktura jest prosta: rola + klucz. Obecny Falcon PK staje się RoleKey { role: Validator, falcon_pk }. | Identity Migration Audit: Falcon PK = role key. Struktura definiuje model. | ZERO — to jest wrapper na existing klucz. | Brak = brak definicji klucza roli = Falcon jest nadal "identity." |
| **`SpendPolicyTag::ComputeLeaseEscrow = 0x04`** | Nowy SpendPolicy variant wymaga tagu. Routing: `match policy_tag { 0x03 => Escrow2of3, 0x04 => ComputeLeaseEscrow }`. | STRONG_CANDIDATE | Brak | **YES** — additive tag. SpendPolicyTag jest `#[repr(u8)]`. Tag 0x04 jest następny wolny. | P-T042: SpendPolicy enum jest extensible. Tag 0x04 = ZERO impact na existing tags. | ZERO — additive. | Brak = ComputeLeaseEscrow nie ma tagu = nie może istnieć. |
| **`TargetRecipient::Two(role_a, role_b)`** | Pro-rata wymaga 2 outputs do różnych odbiorców. Current `TargetRecipient` ma tylko `One` i `Either`. | STRONG_CANDIDATE | Pro-rata note split spec (Opus) | **YES jako variant, NIE jako enforcement** — variant istnieje, ale validation dla 2 outputs wymaga spec. | P-T042: pro-rata wymaga 2 outputs. TargetRecipient::Two jest additive variant. | Variant istnieje ale nie jest używany = placeholder. Ale to jest OK. | Brak = pro-rata nie ma target representation. |
| **`EscrowAction::ProRataSplit = 0x04`** | Pro-rata split jest osobną akcją od Release/Refund/RecoveryRelease. EscrowAction enum musi ją reprezentować. | STRONG_CANDIDATE | Pro-rata note split spec (Opus) | **YES jako variant, NIE jako enforcement** — variant istnieje, ale execution wymaga spec. | P-T042: pro-rata wymaga nowej akcji. EscrowAction jest `#[repr(u8)]`. 0x04 jest następny wolny. | Variant istnieje ale nie jest executed = placeholder. Ale to jest OK. | Brak = pro-rata nie ma akcji representation. |

---

## 2. Types To Delay

| Type | Why Delay | Required Prior Decision | Possible Later Owner Doc |
|---|---|---|---|
| **`SettlementFormula`** | Brak frozen formula. LinearProRata jest direction-level "likely default." Edge cases nie zdefiniowane. Zależy od aPVA precision. | aPVA precision freeze. Operator decyzja o max supply. | Settlement Formula Spec (Opus) |
| **`ComputeLeasePolicy`** | Polityka wymaga ResourceClass, PrivacyClass, NetworkMode granularity. Pola nie są zdefiniowane (required vs optional). Commitment hash zależy od canonical encoding. | ResourceClass granularity. PrivacyClass granularity. Compute Lease Object spec. | Compute Lease Object Spec (Opus) |
| **`ComputeLeaseReceipt`** | Receipt wymaga metering protocol. Session binding wymaga session lifecycle. Heartbeat status wymaga heartbeat protocol. | Metering Protocol Direction (T-035). Receipt schema spec. Metering trust model. | Metering Protocol Direction (Opus/T-035) |
| **`ResourceClass`** | V0 mówi "GPU/CPU/RAM/VRAM" ale granularity nie jest zdefiniowana. Ile GPU classes? Jak nazwać? Pola struct nie są frozen. | ResourceClass granularity decyzja. | Compute Lease Object Spec (Opus) |
| **`SessionKey`** | Session lifecycle nie jest zdefiniowany. SessionKey bez session jest meaningless. MeteringSession nie istnieje. | Metering session spec. Lease lifecycle spec. | Metering Protocol Direction (Opus) |
| **`EpochKey`** | Rotation protocol nie jest zdefiniowany. Epoch key bez rotation policy jest meaningless. Phase 7 concern. | Epoch rotation policy. Identity Model Direction. | Identity Model Direction (Opus/T-033) |
| **`ScopedOfferingId`** | Discovery protocol nie istnieje. ScopedOfferingId bez discovery jest meaningless. | Private Discovery Direction doc. | Private Discovery Direction (Opus) |
| **`ComputeOffering`** | Discovery protocol nie istnieje. ComputeOffering bez discovery jest pustym obiektem. | Private Discovery Direction doc. ComputeOffering fields. | Private Discovery Direction (Opus) |
| **`HeartbeatStatus`** | Heartbeat protocol nie istnieje. HeartbeatStatus bez protocol jest meaningless. | Metering protocol direction. Heartbeat spec. | Metering Protocol Direction (Opus) |
| **`DiscoveryQuery` / `DiscoveryResponse`** | Discovery protocol nie istnieje. Query/Response bez protocol są meaningless. | Private Discovery Direction doc. | Private Discovery Direction (Opus) |
| **`EscrowAction::TimeoutAutoRefund`** | Phase 2 concern. W Phase 0/1 refund wymaga sygnatariuszy. TimeoutAutoRefund jest needed dopiero gdy protokół egzekwuje. | Operatorless Escrow Direction doc. Phase 2 spec. | Operatorless Escrow Direction (Opus/T-032) |

---

## 3. Types To Reject

**none** — wszystkie typy z poprzednich propozycji mają sens directionally. Żaden nie jest fundamentally wrong. Niektóre są tylko za wcześnie (§2).

---

## 4. Type Dependency Graph

```
LedgerAmount (u64 alias)
  ├── BEFORE: ComputeLeasePolicy (polityka zawiera price_aPVA)
  ├── BEFORE: ComputeLeaseReceipt (receipt zawiera amount)
  └── BEFORE: Pro-rata settlement formula

NetworkMode (enum)
  ├── BEFORE: ComputeLeasePolicy (polityka zawiera network_mode)
  └── BEFORE: ComputeOffering (oferta reklamuje network_mode)

SettlementMode (enum)
  ├── BEFORE: ComputeLeaseEscrow SpendPolicy (pole settlement_mode)
  └── BEFORE: ComputeLeasePolicy (polityka zawiera settlement_mode)

PrivacyClass (enum)
  ├── BEFORE: ComputeLeasePolicy (polityka zawiera privacy_class)
  └── BEFORE: ComputeOffering (oferta reklamuje privacy_class)

RoleType (enum)
  ├── BEFORE: RoleKey (klucz zawiera role: RoleType)
  ├── BEFORE: HiddenRootCredential (root derivuje klucze per rola)
  └── BEFORE: ComputeLeaseEscrow SpendPolicy (miner_pk_hash)

HiddenRootCredential (struct)
  ├── BEFORE: RoleKey derivation
  └── BEFORE: SessionKey derivation (future)

RoleKey (struct)
  ├── BEFORE: ComputeLeaseEscrow SpendPolicy (user_pk_hash, miner_pk_hash)
  └── BEFORE: ComputeLeaseReceipt (miner_signature)

SpendPolicyTag::ComputeLeaseEscrow
  └── BEFORE: SpendPolicy::ComputeLeaseEscrow (tag dispatch)

TargetRecipient::Two
  └── BEFORE: Pro-rata output validation

EscrowAction::ProRataSplit
  └── BEFORE: Pro-rata settlement execution

SettlementFormula
  └── AFTER: aPVA precision freeze

ComputeLeasePolicy
  └── AFTER: ResourceClass, PrivacyClass, NetworkMode granularity

ComputeLeaseReceipt
  └── AFTER: Metering Protocol Direction

ComputeOffering
  └── AFTER: Private Discovery Direction

SessionKey / EpochKey
  └── AFTER: Identity Model Direction

HeartbeatStatus
  └── AFTER: Metering Protocol Direction

DiscoveryQuery / DiscoveryResponse
  └── AFTER: Private Discovery Direction
```

**Dependency order (build sequence):**

```
Faza 0a (no dependencies):
  1. LedgerAmount = u64
  2. NetworkMode enum
  3. SettlementMode enum
  4. PrivacyClass enum
  5. RoleType enum
  6. SpendPolicyTag::ComputeLeaseEscrow = 0x04
  7. TargetRecipient::Two variant
  8. EscrowAction::ProRataSplit = 0x04

Faza 0b (depends on Faza 0a):
  9. HiddenRootCredential struct (depends on nothing, but meaningful after RoleType)
  10. RoleKey struct (depends on RoleType)
```

---

## 5. Final Recommendation

### Czy "types first" jest bezpieczne?

**TAK.** Ale z precyzją: **10 typów, nie więcej.**

### Które 10 typów (Faza 0a + 0b):

```
Faza 0a (8 typów — zero dependencies, zero risk):
  1.  type LedgerAmount = u64
  2.  enum NetworkMode { Isolated, NxmsOnly, TorGated, InternetExit }
  3.  enum SettlementMode { AllOrNothing, ProRata }
  4.  enum PrivacyClass { VM, Container, Sandbox, ConfidentialRuntime }
  5.  enum RoleType { Validator, ComputeMiner, Relay, Mailbox, ExitNode }
  6.  SpendPolicyTag::ComputeLeaseEscrow = 0x04
  7.  TargetRecipient::Two(role_a, role_b)
  8.  EscrowAction::ProRataSplit = 0x04

Faza 0b (2 typy — depends on Faza 0a):
  9.  struct HiddenRootCredential { root_seed }
  10. struct RoleKey { role: RoleType, falcon_pk }
```

### Dlaczego te 10, nie inne:

- **LedgerAmount** — audyt P-T041: kod JUŻ używa u64. Alias jest zero risk.
- **NetworkMode** — FROZEN_CANDIDATE. V0 explicitnie definiuje 4 tryby.
- **SettlementMode** — fundament ComputeLeaseEscrow. 2-value enum.
- **PrivacyClass** — fundament ComputeLeasePolicy. Enum istnieje, enforcement jest future.
- **RoleType** — fundament identity model. 5 wartości, zdefiniowane.
- **SpendPolicyTag::ComputeLeaseEscrow** — fundament nowego escrow. Additive tag.
- **TargetRecipient::Two** — fundament pro-rata. Additive variant.
- **EscrowAction::ProRataSplit** — fundament pro-rata. Additive variant.
- **HiddenRootCredential** — fundament identity hierarchy. Struktura definiuje model.
- **RoleKey** — fundament role separation. Wrapper na existing klucz.

### Dlaczego NIE więcej:

- ComputeLeasePolicy — wymaga ResourceClass granularity (nie zdefiniowana)
- ComputeLeaseReceipt — wymaga Metering Protocol Direction (nie istnieje)
- ComputeOffering — wymaga Private Discovery Direction (nie istnieje)
- SessionKey/EpochKey — wymaga Identity Model Direction (nie istnieje)
- HeartbeatStatus — wymaga Metering Protocol Direction (nie istnieje)
- DiscoveryQuery/DiscoveryResponse — wymaga Private Discovery Direction (nie istnieje)
- SettlementFormula — wymaga aPVA precision freeze (nie podjęta)

### Co to oznacza praktycznie:

Faza 0 = 10 typów/variants. Każdy jest:
- Additive (nie zmienia existing code)
- Self-contained (nie wymaga innego nowego typu poza Faza 0a)
- Directionally correct (V0 docs potwierdzają potrzebę)
- Zero proof impact (Amount14 niezmieniony, Halo2 niezmieniony)
- Zero consensus impact (consensus niezmieniony)

Po Faza 0, kolejne typy są dodawane w fazach które je potrzebują:
- Faza 3 (escrow): ComputeLeaseEscrow SpendPolicy struct (depends on Faza 0)
- Faza 4 (receipts): ComputeLeaseReceipt (depends on Metering Direction)
- Faza 5 (pro-rata): SettlementFormula (depends on aPVA freeze)
- Faza 8 (discovery): ComputeOffering, ScopedOfferingId, DiscoveryQuery, DiscoveryResponse (depends on Discovery Direction)

---

## Final Self-Check

- **Czy czytałem legacy docs:** NIE
- **Czy czytałem kod:** TAK (w poprzednich audytach P-T040–P-T045)
- **Czy edytowałem pliki inne niż output:** NIE (tylko OUTPUT_XIAOMI.md)
- **Czy definiowałem wire formaty:** NIE
