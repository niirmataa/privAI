# privAI V0: Amount14 / LedgerAmount Audit

**Status:** technical audit / amount system analysis
**Data:** 2026-04-11
**Źródło:** P-T041-XIAOMI
**Zakres:** audyt techniczny Amount14 vs aPVA/LedgerAmount

---

## 1. Amount14 Usage Map

| File | Symbol/Function | How Used | Proof Lane or Ledger Economics | Risk |
|---|---|---|---|---|
| `privai-chain/src/primitives.rs:26` | `struct Amount14(u16)` | Definicja typu. Max = `PLAINTEXT_SPACE_P - 1 = 16383`. | Proof lane — constraint z LWE plaintext space. | Fundamentalny bottleneck jeśli używany do economics. |
| `privai-chain/src/primitives.rs:30` | `Amount14::new()` | Sprawdza `value < PLAINTEXT_SPACE_P`. | Proof lane. | Odrzuca wartości ≥ 16384. |
| `privai-chain/src/note.rs:199` | `RecipientBoxPlaintext.amount: Amount14` | Kwota w plaintext wewnątrz encrypted recipient box. | Proof lane — szyfrowana kwota w nocie. | Zmiana Amount14 wymaga zmiany recipient box format. |
| `privai-chain/src/note.rs:224` | `AuxWitness.amount: Amount14` | Auxiliary witness dla note commitment. | Proof lane — witness dla commitment computation. | Zmiana wymaga zmiany commitment scheme. |
| `privai-chain/src/note.rs:245` | `OutputNote.ct_amt: LweCiphertext` | Zaszyfrowana kwota w nocie (LWE). Derived z Amount14. | Proof lane — encrypted representation. | Kwota szyfrowana jako `Δ * amount + noise`. Max plaintext = 16383. |
| `privai-proof/src/transfer.rs:122` | `TransferInputWitness.amount: Amount14` | Witness dla input note w proof. | Proof lane — prover potrzebuje Amount14. | Zmiana Amount14 = zmiana proof circuit. |
| `privai-wallet/src/escrow_builder.rs:248-258` | `build_escrow_transfer_note_from_assembly_inputs` | Konwertuje dostępne środki na Amount14: `u16::try_from(output_amount)` → `Amount14::new(narrowed)`. | Proof lane + economics bridge — wallet buduje notes z Amount14. | **KRYTYCZNE:** escrow amount konwertowany na Amount14. Max escrow = 16,383. |
| `privai-wallet/src/builder.rs:246` | `TransferOutputPlan.amount: Amount14` | Plan outputu w transfer note. | Proof lane. | Jak wyżej. |
| `privai-wallet/src/state.rs:44` | `SpendMaterial.amount: Amount14` | Śledzenie kwoty spendable note. | Proof lane. | Jak wyżej. |
| `privai-chain/src/note.rs:351` | `LiteOutputNote.amount: u64` | **JAWNA kwota** — zamiast LWE ciphertext. | **Ledger economics** — u64, nie Amount14. | **ZERO risk** — już używa u64. |
| `privai-chain/src/tx.rs:96` | `TxCore.fee: u64` | Fee transakcji. | **Ledger economics** — u64. | **ZERO risk** — już używa u64. |
| `privai-chain/src/small_payments.rs:117` | `Receipt.amount: u64` | Kwota receipt w small payments rail. | **Ledger economics** — u64. | **ZERO risk** — już używa u64. |
| `privai-chain/src/small_payments.rs:153-165` | `SettlementBatchSummary.total_gross/fee/refund_amount: u64` | Kwoty w batch settlement. | **Ledger economics** — u64. | **ZERO risk** — już używa u64. |
| `privai-chain/src/small_payments.rs:37-39` | `ServicePaymentPolicy.min_deposit/max_spend: u64` | Polityka płatności. | **Ledger economics** — u64. | **ZERO risk** — już używa u64. |

### Kluczowy wniosek z mapy:

**Amount14 jest używany WYŁĄCZNIE w proof/plaintext lane (LWE encryption). Ledger economics JUŻ używa u64.**

---

## 2. Ledger Amount Reality

### Czy kod ma już u64/u128 amount gdziekolwiek?

**TAK, w wielu miejscach:**

| Field | Type | Location | Purpose |
|---|---|---|---|
| `TxCore.fee` | u64 | `tx.rs:96` | Fee każdej transakcji |
| `LiteOutputNote.amount` | u64 | `note.rs:351` | Jawna kwota w lite notes |
| `Receipt.amount` | u64 | `small_payments.rs:117` | Kwota receipt w small payments |
| `SettlementBatchSummary.total_gross_amount` | u64 | `small_payments.rs:162` | Kwota settlement batch |
| `SettlementBatchSummary.total_fee_amount` | u64 | `small_payments.rs:163` | Fee settlement batch |
| `SettlementBatchSummary.total_refund_amount` | u64 | `small_payments.rs:164` | Refund settlement batch |
| `ServicePaymentPolicy.min_deposit_required` | u64 | `small_payments.rs:37` | Min deposit |
| `ServicePaymentPolicy.max_spend_per_session` | u64 | `small_payments.rs:38` | Max spend per session |
| `ServicePaymentPolicy.max_spend_per_window` | u64 | `small_payments.rs:39` | Max spend per window |
| `SpendGrant.spend_cap` | u64 | `small_payments.rs:77` | Cap na grant |
| `ModelTx.amount_delta` | i64 | `tx.rs:298` | Delta w model tx |

**Wniosek: Ledger-level economics już używa u64. Amount14 jest tylko dla proof/plaintext lane.**

### Czy escrow używa Amount14 bezpośrednio?

**Pośrednio — tak.** Escrow builder konwertuje escrow amount na Amount14:

```rust
// privai-wallet/src/escrow_builder.rs:248-258
let output_amount = available.checked_sub(assembly.fee)?;
let narrowed = u16::try_from(output_amount)?;
let amount14 = Amount14::new(narrowed)?;
```

To oznacza że **current escrow output note jest limited do 16,383**. Ale to jest constraint na output note, nie na escrow amount jako taki.

### Czy amount jest częścią commitment?

**TAK — w dwojaki sposób:**

1. `OutputNote.note_commit` zawiera `aux_commit` który zawiera `AuxWitness.amount: Amount14`. Zmiana typu amount = zmiana commitment computation.
2. `ct_amt: LweCiphertext` jest w `OutputNote` i jest derived z Amount14 (szyfrowanie LWE). Zmiana Amount14 = zmiana LWE encryption.

### Czy output notes ograniczają amount?

**TAK — `OutputNote` jest limited do Amount14 (max 16,383).**

ALE: `LiteOutputNote` używa `amount: u64` — jawna kwota, bez LWE. To jest istniejący "track" w kodzie który obsługuje większe kwoty. Kod już ma dwa tracki amount — to jest dowód że to jest możliwe.

---

## 3. aPVA Compatibility

### Matematyka LWE:

```
PLAINTEXT_SPACE_P = 16,384
Amount14 max = 16,383
LWE_MODULUS_Q = 4,294,967,291 (2^32 - 5)
DELTA = floor(q / p) = floor(4,294,967,291 / 16,384) = 262,143
LWE_DIMENSION = 1024

LWE encryption: v = t^T * r + e2 + Δ * amount
Δ * amount_max = 262,143 * 16,383 = 4,294,694,949
```

### Jeśli 1 PVA = 10^12 aPVA, Amount14 bezpośrednio = aPVA:

```
Max Amount14 = 16,383
Max aPVA per note = 16,383
Max PVA per note = 16,383 / 10^12 = 0.000000000016383 PVA
```

**Absurdalnie małe. Kompletnie niepraktyczne.**

### Ale Amount14 nie musi być = aPVA directly:

Amount14 może być "jednostkami w nocie" gdzie każda jednostka = N aPVA.

| Unit size (aPVA per Amount14 unit) | Max PVA per note | Pro-rata accuracy |
|---|---|---|
| 10^6 (1M) | 0.000016383 PVA | 0.000001 PVA |
| 10^8 (100M) | 1.6383 PVA | 0.00000001 PVA |
| 10^9 (1B) | 16.383 PVA | 0.000000001 PVA |
| 10^10 (10B) | 163.83 PVA | 0.0000000001 PVA |
| 10^11 (100B) | 1,638.3 PVA | 0.00000000001 PVA |

### Realna sytuacja:

```
Escrow dla compute lease = 100 PVA = 100 * 10^12 aPVA = 10^14 aPVA

Z unit = 10^10 aPVA:
  100 PVA = 10,000,000,000,000 aPVA = 1,000,000,000 units
  Max per note = 16,383 units = 163.83 PVA
  100 PVA mieści się w 1 nocie ✓
  Pro-rata accuracy = 10^10 aPVA = 0.00001 PVA ✓

Z unit = 10^11 aPVA:
  100 PVA = 1,000,000,000 units
  Max per note = 16,383 units = 1,638.3 PVA
  100 PVA mieści się w 1 nocie ✓
  Pro-rata accuracy = 10^11 aPVA = 0.000001 PVA ✓
```

**Wniosek: 1 PVA = 10^12 aPVA JEST kompatybilne z kodem — ale wymaga unit conversion, nie direct Amount14 = aPVA.**

---

## 4. Strategy Options

### Option A: Keep Amount14 for proof lane, introduce LedgerAmount for economics

```
Amount14 (u16, max 16383) → proof/plaintext lane ONLY. Nigdy nie zmieniać.
LedgerAmount (u64)         → ledger/escrow/settlement economics. Dodatkowy typ.
```

**Benefit:**
- Amount14 niezmieniony — proof circuit nie wymaga zmian
- LedgerAmount (u64) reprezentuje pełną precyzję aPVA na ledger
- Escrow commitment zawiera LedgerAmount, nie Amount14
- `LiteOutputNote` już używa u64 — pattern jest w kodzie
- `Receipt.amount` już używa u64 — pattern jest w kodzie
- `TxCore.fee` już używa u64 — pattern jest w kodzie
- Pro-rata na ledger level = integer arithmetic na u64 = proste

**Risk:**
- Dwóch reprezentacji amount (Amount14 w notes, LedgerAmount w ledger) = complexity
- Unit conversion (LedgerAmount → Amount14 chunks) wymaga spec
- Wallet musi zarządzać multi-note aggregation jeśli escrow > max single note

**Code impact:** Minimalny. Nowy typ `LedgerAmount = u64`. Nowy SpendPolicy `ComputeLeaseEscrow` używa LedgerAmount. Existing Amount14 niezmieniony. Existing proof circuit niezmieniony.

**Proof impact:** **ZERO.** Amount14 niezmieniony. Halo2 circuit niezmieniony.

### Option B: Multi-note chunking (keep Amount14, split large amounts into many notes)

**Benefit:**
- Zero zmian w typach
- Zero zmian w proof circuit
- Naturalnie pasuje do privacy (trudniej skorelować)

**Risk:**
- 100 PVA = ~7 notes (z unit = 10^10 aPVA)
- Pro-rata na multi-note = complex (jak dzielić notes?)
- Więcej nullifiers = więcej on-chain data
- Więcej transactions = więcej fees

**Code impact:** Medium — wallet multi-note aggregation.

**Proof impact:** **ZERO.**

### Option C: Change LWE plaintext space (increase PLAINTEXT_SPACE_P)

**Benefit:**
- Jedna reprezentacja amount
- Większy Amount14 = większe kwoty per note

**Risk:**
- **KATASTROFALNE.** PLAINTEXT_SPACE_P jest fundamentalnym LWE parametrem:
  - Nowy DELTA = floor(q / new_p)
  - Nowy security parameter
  - Nowy Halo2 circuit (każdy gate parametryzowany na q/p)
  - Nowe klucze
  - Nowy ciphertext format
  - Wszystkie existing notes nieważne
- **To jest rewrite proof system, nie zmiana typu.**

**Code impact:** **KATASTROFALNY.** Cały proof system do przebudowy.

**Proof impact:** **TOTALNY.** Nowy circuit, nowe klucze, nowe ciphertexty.

### Option D: Reject 10^12 precision (use smaller aPVA)

**Benefit:**
- Mniejsza precyzja = większe kwoty w Amount14 direct
- Np. 1 PVA = 10^8 aPVA → max = 16,383 PVA per note

**Risk:**
- Mniejsza precyzja = mniej dokładność dla compute metering (ms-level)
- Relay fees mogą wymagać większej precyzji
- Nie jest zgodne z V0 direction (10^12 jest "strong candidate")

**Code impact:** Minimalny — tylko zmiana denominacji.

**Proof impact:** **ZERO.**

### Option E: Hybrid — Amount14 for proof, LedgerAmount for ledger, unit conversion

Identyczne z Option A. Różnica jest tylko w nazwie.

---

## 5. Recommendation

**Rekomendacja: Option A / E**

```
Amount14 (u16, max 16383) → proof/plaintext lane ONLY. Nigdy nie zmieniać.
LedgerAmount (u64)         → ledger/escrow/settlement economics. Dodatkowy typ.
Unit conversion            → 1 Amount14 unit = N aPVA (N do ustalenia).
```

**Dlaczego:**

1. `LiteOutputNote.amount: u64` już istnieje — kod pokazuje że dwa tracki są możliwe.
2. `Receipt.amount: u64` w small payments — ledger economics używa u64.
3. `TxCore.fee: u64` — fees używają u64.
4. Halo2 circuit jest scaffold — zmiana Amount14 = rewrite circuit = nie teraz.
5. V0 aPVA 10^12 jest potrzebne dla compute metering precision.
6. Unit conversion jest tractowalne w wallet.
7. Pro-rata na LedgerAmount (u64) jest proste — integer arithmetic.

**Status: CANDIDATE** — likely correct, ale wymaga:

- Decyzji operatora o max supply PVA (u64 czy u128 — jeśli supply > ~18.4B PVA, u128 required)
- Decyzji o unit conversion granularity (ile aPVA = 1 Amount14 unit)
- Audytu czy escrow commitment scheme obsługuje LedgerAmount w commitment hash
- Testu pro-rata z Amount14 chunks w multi-note scenarios

**NIE jest FROZEN bo:**

- Unit conversion granularity nie jest zdefiniowana
- Multi-note escrow split nie jest zaimplementowany
- Pro-rata na Amount14 chunks nie jest tested
- Decyzja o max supply PVA nie jest podjęta

---

## 6. Red Lines

1. **Nie zmieniać Amount14 z u16 na u32/u64.** To zmienia LWE plaintext space = rewrite proof circuit. Katastrofalne.

2. **Nie zmieniać PLAINTEXT_SPACE_P (16384).** To jest fundamentalny LWE parameter. Zmiana = nowe klucze, nowe ciphertexty, nowy circuit.

3. **Nie używać Amount14 do reprezentowania kwot escrow bezpośrednio.** Amount14 jest dla encrypted notes. Escrow amount jest LedgerAmount.

4. **Nie zakładać że Amount14 = aPVA directly.** Amount14 jest jednostką w LWE plaintext. Unit conversion jest required.

5. **Nie usuwać LiteOutputNote.** To jest istniejący pattern "jawna kwota u64 w nocie" — dowód że dwa tracki amount są możliwe.

6. **Nie twierdzić że aPVA 10^12 jest incompatible z kodem.** Jest incompatible z Amount14 directly, ale LedgerAmount (u64) obsługuje 10^12 aPVA do ~18.4B PVA supply.

7. **Nie implementować multi-note chunking przed unit conversion spec.** Chunking wymaga definicji granularity (ile aPVA = 1 unit).

8. **Nie twierdzić że Amount14 jest "broken" lub "za małe."** Amount14 jest poprawnie zaprojektowane dla proof lane. Problem jest tylko jeśli ktoś używa Amount14 do economics.

---

## 7. Summary — co to oznacza dla V0

### Wniosek fundamentalny:

```
Kod ma JUŻ dwa tracki amount:
  Track 1 (proof):  Amount14 (u16) + LweCiphertext — privacy-preserving notes
  Track 2 (ledger): u64 — fees, settlements, receipts, lite notes

V0 compute lease powinien używać:
  Track 2 (LedgerAmount = u64) dla escrow amounts, settlement, pro-rata
  Track 1 (Amount14) dla encrypted note representation (unit conversion)

To NIE jest nowa architektura. To jest rozszerzenie existing pattern.
```

### Co trzeba zrobić:

1. Dodać `type LedgerAmount = u64` (alias lub nowy struct)
2. Nowy `ComputeLeaseEscrow` SpendPolicy używa `LedgerAmount` dla commitment
3. Unit conversion spec: `1 Amount14 unit = N aPVA`
4. Pro-rata działa na `LedgerAmount` (u64 integer arithmetic)
5. Wallet konwertuje `LedgerAmount` → `Amount14` chunks dla note building

### Co NIE trzeba robić:

1. Zmieniać Amount14
2. Zmieniać PLAINTEXT_SPACE_P
3. Zmieniać Halo2 circuit
4. Zmieniać LWE encryption
5. Zmieniać existing OutputNote format

---

**Czy edytowano pliki:** NIE (poza zapisem tego pliku)
**Czy czytano legacy docs:** NIE
**Czy zdefiniowano wire formaty:** NIE
**Czy odpowiedź jest technical audit:** TAK — code-level analysis of Amount14 constraints.
