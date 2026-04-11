# privAI V0: SpendPolicy / Escrow Compatibility Audit

**Status:** technical audit / escrow extensibility analysis
**Data:** 2026-04-11
**Źródło:** P-T042-XIAOMI
**Zakres:** czy ComputeLeaseEscrow powinien być nowym SpendPolicy, rozszerzeniem Escrow2of3, osobną transakcją, czy osobnym settlement layer

---

## 1. Current Escrow Model

### SpendPolicy Variants (note.rs:76-97)

```
SpendPolicyTag:
  Single = 0x01
  MarketplaceSettlement = 0x02  (legacy)
  Escrow2of3 = 0x03

SpendPolicy:
  Single { falcon_pk_hash }
  MarketplaceSettlement { buyer, seller, moderator, timeout }  (legacy)
  Escrow2of3 { buyer_pk_hash, merchant_pk_hash, operator_pk_hash, timeout_block }
```

**Commitment:** `domain_hash(POLICY_DOMAIN, &[self.to_canonical_bytes()])` — hash z canonical encoding.

**Tag dispatch:** `SpendPolicy::tag()` zwraca `SpendPolicyTag`. Tag jest używany w `InputAuth.policy_tag` żeby wskazać jaką validation path uruchomić.

### EscrowAction Variants (escrow.rs:21-27)

```
EscrowAction:
  Release = 0x01
  Refund = 0x02
  RecoveryRelease = 0x03
```

`from_u8()` — hardcoded match na 0x01/0x02/0x03. Wartości > 0x03 zwracają `None`.

### SignerRole (escrow.rs:47-54)

```
SignerRole:
  Buyer = 0x00    (index 0 w polityce)
  Merchant = 0x01 (index 1)
  Operator = 0x02 (index 2)
```

### required_signers() (escrow.rs:64-70) — FROZEN RULE TABLE

```
Release:         Buyer + Operator
Refund:          Merchant + Operator
RecoveryRelease: Buyer + Merchant
```

Hardcoded. Brak parametryzacji. Każda akcja ma fixed pair.

### target_recipient() (escrow.rs:87-95) — FROZEN RULE TABLE

```
Release:         TargetRecipient::One(Merchant)
Refund:          TargetRecipient::One(Buyer)
RecoveryRelease: TargetRecipient::Either(Buyer, Merchant)
```

**Kluczowa obserwacja:** `TargetRecipient` ma tylko 2 warianty: `One` (dokładnie jeden odbiorca) i `Either` (jeden z dwóch). **Nie ma `Two` (dwóch odbiorców jednocześnie).** To jest blocker dla pro-rata.

### validate_output_target() (ledger/escrow.rs:182-214)

```rust
fn validate_output_target(action, outputs, buyer_pk_hash, merchant_pk_hash, operator_pk_hash):
    target = target_recipient(action)
    allowed_commits = match target:
        One(role) => [single_commit(pk_hash_for_role(role))]
        Either(a, b) => [single_commit(pk_hash_for_role(a)), single_commit(pk_hash_for_role(b))]
    for output in outputs:
        if output.spend_policy_commit NOT IN allowed_commits:
            return EscrowOutputTargetMismatch
```

**WSZYSTKIE outputy muszą iść do dozwolonego odbiorcy.** To jest anti-siphoning guard. Ale jednocześnie blokuje 2-output split (pro-rata).

### validate_escrow_auth() (ledger/escrow.rs:44-176) — 12-STEP VALIDATION

```
1.  Require policy_opening bytes
2.  Require escrow_action byte
3.  Decode action (EscrowAction::from_u8)
4.  Decode policy (SpendPolicy::from_canonical_bytes)
5.  Hardcoded match: policy MUST be Escrow2of3 (rejects anything else)
6.  Verify policy commitment matches input note spend_policy_commit
7.  Require exactly 2 signers
8.  Identify signer roles by hashing PKs against policy fields
9.  Reject duplicate signers
10. Check canonical ordering (ascending by index)
11. Check signer combination against frozen rule table
12. Recovery timeout check
13. Falcon signature verification against tx_signing_hash
14. Output target validation
```

**Krok 5 jest najważniejszy:** `validate_escrow_auth` hardcoded odrzuca wszystko co nie jest `Escrow2of3`. To oznacza że nowy `ComputeLeaseEscrow` **NIE może używać tej samej funkcji** bez modyfikacji.

### Stage A/B Assumptions

- **Stage A** (control plane): `EscrowApprovalBundle` z operator/orchestrator. Walidacja w node (10 kroków w `node.rs:593-755`).
- **Stage B** (on-chain): `validate_escrow_auth()` w ledger. Finalna walidacja.
- Escrow używa `TransferNoteTx` proof path — **nie ma osobnego proof path** dla escrow.

### Transaction Types (tx.rs:351-358)

```
Transaction:
  TransferNote(TransferNoteTx)      — escrow używa tego!
  Settlement(SettlementTx)
  Model(ModelTx)
  Stake(StakeTx)
  MarketplaceBatch(MarketplaceBatchTx)  (legacy)
  LiteTransfer(LiteTransferTx)
```

**Escrow NIE jest osobnym Transaction variant.** Escrow jest `TransferNoteTx` z `InputAuth` mającym `policy_tag == Escrow2of3`. To jest kluczowa architektoniczna decyzja — escrow reuses transfer note proof path.

---

## 2. ComputeLease Requirements (z V0 docs)

| Requirement | Current Escrow2of3 | Gap |
|---|---|---|
| Lease policy commitment | Brak — Escrow2of3 nie ma pola na policy | Nowe pole w SpendPolicy |
| Receipt requirements | Brak — settlement nie wymaga receipts | Nowa logika validation |
| Settlement mode (AllOrNothing/ProRata) | Brak — tylko all-or-nothing | Nowe pole + nowa logika |
| Pro-rata split (2 outputs) | Brak — target_recipient = One lub Either, nie Two | Nowy TargetRecipient variant |
| Timeout auto-refund | Brak — refund wymaga Merchant + Operator | Nowa akcja lub protocol-level enforcement |
| Operator bridge (Phase 1) | Partial — operator jest canonical signer | Zmiana semantyki: operator validates, nie decides |
| Operatorless final path (Phase 2) | Brak — tylko RecoveryRelease jest operatorless | Nowa akcja lub nowa validation path |
| User + Protocol sign (Phase 2) | Brak — wymaga 2 z 3 human signers | Nowa reguła required_signers |

---

## 3. Compatibility Options

### Option A: Extend Escrow2of3

**Opis:** Dodaj nowe pola do `SpendPolicy::Escrow2of3` (np. `lease_policy_commit: Option<Hash32>`, `settlement_mode: Option<SettlementMode>`). Rozszerz `required_signers()` i `target_recipient()` o nowe cases.

**Benefit:**
- Jeden SpendPolicy type — mniej complexity w dispatch
- Existing notes mają backward compatible encoding (nowe pola jako `Option` = None)

**Risk:**
- **KATASTROFALNE dla backward compatibility.** `SpendPolicy::commitment()` = `domain_hash(POLICY_DOMAIN, &[self.to_canonical_bytes()])`. Dodanie pól zmienia canonical encoding = zmienia commitment hash = existing notes mają nieważne commitmenty.
- `validate_escrow_auth` hardcoded match na `Escrow2of3` — rozszerzenie go wymaga zmiany 12-step validation.
- `required_signers()` jest frozen — rozszerzenie łamie frozen contract.
- `target_recipient()` jest frozen — rozszerzenie łamie frozen contract.
- **Jeden błąd w Escrow2of3 łamie istniejące escrow notes.**

**Test impact:** **KATASTROFALNY.** Wszystkie existing escrow tests mogą się złamać bo commitment hash się zmienia.

**Backward compatibility:** **ZERO.** Existing notes z Escrow2of3 commitment stają się nieważne.

**Complexity:** Niska na start, ale ryzyko jest nieakceptowalne.

### Option B: Add ComputeLeaseEscrow as new SpendPolicy variant

**Opis:** Nowy `SpendPolicy::ComputeLeaseEscrow { user_pk_hash, miner_pk_hash, lease_policy_commit, timeout_block, settlement_mode }`. Nowy `SpendPolicyTag::ComputeLeaseEscrow = 0x04`. Nowa walidacja `validate_compute_lease_escrow_auth()` — osobna funkcja.

**Benefit:**
- **ZERO impact na existing Escrow2of3.** Escrow2of3 jest niezmieniony.
- Nowa validation path — osobna funkcja, osobne testy.
- Naturalne rozszerzenie: `SpendPolicy` enum jest designed do dodawania variants.
- `TargetRecipient::Two` jest dodawany tylko dla nowej validation path.
- Stage A/B boundary niezmieniona — nowa validation żyje w tym samym ledger module.
- Escrow nadal używa `TransferNoteTx` proof path — nie nowy Transaction variant.

**Risk:**
- Nowa walidacja jest niezależna — musi być equally solid jak existing.
- `validate_escrow_auth()` jest hardcoded na Escrow2of3 — musi być dispatch point który routuje `policy_tag == 0x04` do nowej funkcji.
- `TargetRecipient::Two` wymaga zmiany `validate_output_target` (ale tylko dla nowej path).

**Test impact:** **MINIMALNY.** Existing tests dla Escrow2of3 niezmienione. Nowe tests dla ComputeLeaseEscrow.

**Backward compatibility:** **PEŁNA.** Existing Escrow2of3 notes działają niezmienione.

**Complexity:** Średnia — nowa validation function + nowy SpendPolicy variant + nowy TargetRecipient variant.

### Option C: Add ComputeLeaseTx as new Transaction variant

**Opis:** Nowy `Transaction::ComputeLease(ComputeLeaseTx)` — osobny typ transakcji z własnym core, własnym proof path, własnym validation.

**Benefit:**
- Complete separation — zero coupling z existing escrow.
- ComputeLeaseTx może mieć pola specyficzne dla lease (receipts, policy, metering).
- Osobny proof path — może dowodzić rzeczy specyficzne dla lease.

**Risk:**
- **Duży scope.** Nowy Transaction variant wymaga:
  - Nowy `ComputeLeaseTx` struct
  - Nowy `tx_signing_hash()` branch
  - Nowy `tx_id()` branch
  - Nowa validation w node
  - Nowa validation w ledger
  - Nowy proof path (lub reuse existing)
  - Zmiana w `Transaction::core()` match
  - Zmiana w block building
  - Zmiana w state management
- Escrow2of3 nadal potrzebuje nowej SpendPolicy (bo ComputeLeaseTx nadal używa OutputNote z SpendPolicy).
- **Overkill na teraz.** ComputeLeaseTx jest potrzebne dopiero gdy compute lease wymaga fundamentally different tx structure.

**Test impact:** **DUŻY.** Nowy tx type = nowe tests w wielu modułach.

**Backward compatibility:** **PEŁNA** (existing tx types niezmienione).

**Complexity:** **DUŻA.** Nowy tx type touchuje chain, ledger, node, wallet, proof.

### Option D: Separate settlement layer

**Opis:** Osobny moduł `privai-settlement` który nie jest częścią escrow. Compute lease settlement żyje poza SpendPolicy, poza EscrowAction, poza ledger escrow validation.

**Benefit:**
- Complete separation — zero coupling.
- Compute lease settlement może mieć zupełnie inną architekturę.

**Risk:**
- **Nie integruje się z existing note system.** Settlement musi operować na notes (lock, nullify, create outputs). Jeśli settlement jest osobne od escrow, musi mieć własny note management.
- **Duplikacja.** Duża część escrow mechanics (lock, timeout, signature verification) jest reuse'owalna.
- **Wymaga nowego proof path** — nie może używać existing TransferNoteTx.
- **Największy scope ze wszystkich opcji.**

**Test impact:** **NAJWIĘKSZY.** Nowy layer = nowe tests everywhere.

**Backward compatibility:** **PEŁNA** (ale irrelevant bo scope jest za duży).

**Complexity:** **NAJWIĘKSZA.**

---

## 4. Recommendation

### Rekomendacja: Option B — Nowy SpendPolicy variant ComputeLeaseEscrow

**Status: CANDIDATE** — likely correct, oparte na code analysis.

**Dlaczego Option B:**

1. **SpendPolicy enum jest designed do dodawania variants.** `tag()` dispatch, `commitment()` computation, `CanonicalEncode` — wszystko jest match-based. Dodanie `ComputeLeaseEscrow { ... }` z `SpendPolicyTag::ComputeLeaseEscrow = 0x04` jest naturalne.

2. **ZERO impact na Escrow2of3.** Escrow2of3 jest niezmieniony. `validate_escrow_auth()` jest niezmieniona. Frozen rule table jest niezmieniona. Wszystkie existing tests przechodzą.

3. **Nowa validation path jest osobną funkcją.** `validate_compute_lease_escrow_auth()` — osobna od `validate_escrow_auth()`. Routing: `match policy_tag { 0x03 => validate_escrow_auth(), 0x04 => validate_compute_lease_escrow_auth() }`.

4. **Escrow nadal używa TransferNoteTx proof path.** Nie potrzebny nowy Transaction variant. ComputeLeaseEscrow jest SpendPolicy, nie Transaction type.

5. **TargetRecipient::Two jest dodawany tylko dla nowej path.** Nowy variant `Two(SignerRole, SignerRole)` — używany tylko przez `validate_compute_lease_escrow_auth()`.

6. **Stage A/B boundary niezmieniona.** Nowa validation żyje w tym samym ledger module. Stage A (control plane) buduje approval bundle tak samo. Stage B (ledger) routuje do nowej validation.

7. **Pro-rata jest możliwa.** `TargetRecipient::Two` + nowa `validate_output_target` dla ComputeLeaseEscrow = 2 outputs (miner + user).

8. **LedgerAmount (u64) jest w SpendPolicy commitment, nie w Amount14.** ComputeLeaseEscrow commitment zawiera `lease_policy_commit` (Hash32), nie kwotę bezpośrednio. Kwota jest w OutputNote (Amount14) lub w commitment hash.

**Dlaczego NIE Option A (extend Escrow2of3):**
- Dodanie pól zmienia canonical encoding = zmienia commitment = existing notes nieważne.
- Frozen rule table jest frozen — nie wolno łamać.
- Ryzyko regression jest nieakceptowalne.

**Dlaczego NIE Option C (ComputeLeaseTx) na teraz:**
- Za duży scope na teraz.
- ComputeLeaseEscrow jako SpendPolicy wystarczy Phase 0-5.
- ComputeLeaseTx może być dodane później jeśli tx structure musi się fundamentalnie zmienić.

**Dlaczego NIE Option D (separate settlement layer):**
- Za duży scope.
- Duplikacja escrow mechanics.
- Nie integruje się z existing note system.

---

## 5. Tests That Must Not Break

### privai-chain/src/escrow.rs (141 lines, 5 tests)

| Test | Co testuje | Dlaczego nie może się złamać |
|---|---|---|
| `action_roundtrip` | EscrowAction::from_u8 dla 0x01/0x02/0x03, odrzucenie 0x00/0x04 | Frozen rule table — nowe akcje NIE zmieniają istniejących |
| `release_requires_buyer_and_operator` | required_signers(Release) = Buyer + Operator | Frozen rule table |
| `refund_requires_merchant_and_operator` | required_signers(Refund) = Merchant + Operator | Frozen rule table |
| `recovery_requires_buyer_and_merchant` | required_signers(RecoveryRelease) = Buyer + Merchant | Frozen rule table |
| `signer_role_indices_are_canonical` | Buyer=0, Merchant=1, Operator=2 | Frozen canonical ordering |

### privai-ledger/src/escrow.rs (757 lines, 20+ tests)

| Test Category | Tests | Co testują | Dlaczego nie mogą się złamać |
|---|---|---|---|
| Policy reconstruction | `reject_missing_policy_opening`, `reject_missing_action`, `reject_invalid_action_byte`, `reject_policy_mismatch`, `reject_non_escrow_policy_in_opening` | Walidacja polityki | Escrow2of3 niezmieniony |
| Signer validation | `reject_unknown_signer`, `reject_duplicate_signer`, `reject_wrong_signer_order`, `reject_wrong_signer_combination_for_release`, `reject_wrong_signer_count` | Walidacja sygnatariuszy | Frozen rule table |
| Recovery timeout | `reject_recovery_before_timeout` | Timeout enforcement | Frozen — nowe akcje nie wpływają |
| Output target | `output_target_validation_release_to_merchant`, `output_target_validation_refund_to_buyer`, `output_target_validation_recovery_accepts_either` | Target validation | Escrow2of3 target rules niezmienione |
| Anti-siphoning | `reject_mixed_outputs_siphoning`, `reject_empty_outputs` | ALL outputs must match | Escrow2of3 niezmieniony |

### privai-wallet/src/escrow_builder.rs (724 lines, 7 tests)

| Test | Co testuje |
|---|---|
| `test_escrow_builder_success` | Full escrow transfer note building |
| `test_escrow_builder_rejects_conflicting_signers` | Duplicate signer rejection |
| `test_escrow_builder_rejects_action_and_funding_mismatch` | Action/funding consistency |
| `test_escrow_builder_action_changes_hash` | Different actions → different hashes |
| `test_escrow_builder_amount_within_range_builds` | Amount within Amount14 range |
| `test_escrow_builder_amount_at_max_boundary_builds` | Max Amount14 = 16383 |
| `test_escrow_try_from_conversion_rejects_overflow` | u64 → u16 overflow guard |

### Co MUSI pozostać zielone:
- Wszystkie powyższe tests — łącznie ~32 tests
- Żaden z nich nie może być zmieniony, usunięty, ani zignorowany
- Nowe tests dla ComputeLeaseEscrow są ADDITIVE

---

## 6. Implementation Pattern dla Option B

Routing point (conceptual, nie code):

```
validate_transaction → find escrow auth → match policy_tag:
  0x03 (Escrow2of3) → validate_escrow_auth() [NIEZMIENIONA]
  0x04 (ComputeLeaseEscrow) → validate_compute_lease_escrow_auth() [NOWA]
  _ → error
```

Nowa validation function (conceptual):

```
validate_compute_lease_escrow_auth():
  1. Require policy_opening
  2. Decode policy → must be ComputeLeaseEscrow
  3. Verify policy commitment matches input note
  4. Decode action → must be valid ComputeLeaseEscrow action
  5. Verify signer count (2 for bridge, 1+protocol for operatorless)
  6. Identify signer roles
  7. Check signer combination against ComputeLeaseEscrow rule table
  8. Recovery timeout check
  9. Falcon signature verification
  10. Output target validation (One OR Two, depending on action)
  11. Lease policy commitment validation (if action requires receipts)
```

Nowe elementy:

```
SpendPolicyTag::ComputeLeaseEscrow = 0x04

SpendPolicy::ComputeLeaseEscrow {
  user_pk_hash: Hash32,
  miner_pk_hash: Hash32,
  lease_policy_commit: Hash32,
  timeout_block: u64,
  settlement_mode: SettlementMode,
}

EscrowAction (rozszerzony):
  ProRataSplit = 0x04
  TimeoutAutoRefund = 0x05

SignerRole (nowy):
  User = Buyer (reuse 0x00, semantically renamed)
  Miner = Merchant (reuse 0x01, semantically renamed)
  Protocol = nowy (0x03?) — dla operatorless Phase 2

TargetRecipient:
  Two(SignerRole, SignerRole) — NOWY variant dla pro-rata

required_signers() — NOWA tabela dla ComputeLeaseEscrow:
  Release:        User + Operator (bridge) lub User + Protocol (operatorless)
  Refund:         User + Operator (bridge) lub User + Protocol (operatorless)
  ProRataSplit:   User + Operator (bridge) lub User + Protocol (operatorless)
  RecoveryRelease: User + Miner (already operatorless)
  TimeoutAutoRefund: Protocol only? lub User + Protocol?
```

---

## 7. Final Red Lines

1. **Nie rozszerzać Escrow2of3 o nowe pola.** Canonical encoding change = commitment hash change = existing notes nieważne.

2. **Nie zmieniać `validate_escrow_auth()`.** Jest frozen dla Escrow2of3. Nowa validation jest osobną funkcją.

3. **Nie zmieniać `required_signers()` dla istniejących akcji.** Frozen rule table dla Release/Refund/RecoveryRelease.

4. **Nie zmieniać `target_recipient()` dla istniejących akcji.** Frozen rule table.

5. **Nie zmieniać `EscrowAction` dla 0x01/0x02/0x03.** Nowe akcje (0x04, 0x05) są additive.

6. **Nie usuwać `MarketplaceSettlement` z SpendPolicy bez deprecation period.** Backward compatibility.

7. **Nie tworzyć nowego Transaction variant na teraz.** ComputeLeaseEscrow jako SpendPolicy wystarczy. ComputeLeaseTx jest future.

8. **Nie twierdzić że pro-rata jest implemented.** Pro-rata wymaga `TargetRecipient::Two` + nowej validation + nowej akcji. To jest future.

9. **Nie łamać istniejących testów.** Wszystkie 32+ existing escrow tests muszą przechodzić bez zmian.

10. **Nie zmieniać Stage A/B boundary.** Nowa validation żyje w tym samym ledger module, routowana z tego samego dispatch point.

---

## Summary

| Option | Recommendation | Status | Impact na existing code |
|---|---|---|---|
| A: Extend Escrow2of3 | **REJECTED** | Commitment hash change = katastrofa | KATASTROFALNY |
| B: New SpendPolicy variant | **CANDIDATE** | Natural extension, zero impact | MINIMALNY |
| C: New Transaction variant | Za wcześnie | Za duży scope na teraz | DUŻY |
| D: Separate settlement layer | Za wcześnie | Za duży scope, duplikacja | NAJWIĘKSZY |

**Finalna rekomendacja: Option B — nowy `SpendPolicy::ComputeLeaseEscrow` z tag `0x04`, osobną walidacją `validate_compute_lease_escrow_auth()`, nowym `TargetRecipient::Two`, i nowymi akcjami `ProRataSplit`/`TimeoutAutoRefund`.**

---

**Czy edytowano pliki:** NIE (poza zapisem tego pliku)
**Czy czytano kod:** TAK — `privai-chain/src/escrow.rs` (pełny), `privai-ledger/src/escrow.rs` (pełny, 757 linii), `privai-chain/src/note.rs` (SpendPolicy), `privai-chain/src/tx.rs` (Transaction enum)
**Czy czytano legacy docs:** NIE
**Czy zdefiniowano wire formaty:** NIE
**Czy odpowiedź jest technical audit:** TAK
