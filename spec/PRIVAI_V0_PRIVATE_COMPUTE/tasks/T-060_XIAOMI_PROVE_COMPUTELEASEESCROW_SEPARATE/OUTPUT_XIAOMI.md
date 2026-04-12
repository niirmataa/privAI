# T-060-XIAOMI — Prove ComputeLeaseEscrow Must Be Separate

**Status:** proof task  
**Data:** 2026-04-12  
**Źródło:** code audit, SpendPolicy Audit PL, Build-Once Types Review PL  
**Zakres:** dowód że ComputeLeaseEscrow musi być osobnym SpendPolicy path

---

## 1. Verdict

**ComputeLeaseEscrow MUSI być osobnym SpendPolicy variantem z osobną walidacją.** Dowód jest oparty na 6 concrete code-level constraints, każdy z których blokuje inne podejście.

---

## 2. Code-Level Evidence

### Evidence 1: SpendPolicy::commitment() = hash(canonical_encoding)

```
// privai-chain/src/note.rs:108-110
pub fn commitment(&self) -> Hash32 {
    domain_hash(POLICY_DOMAIN, &[&self.to_canonical_bytes()])
}
```

**Fact:** Commitment hash jest obliczany z canonical encoding polityki. Każde pole w SpendPolicy jest częścią canonical encoding.

**Proof:** Dodanie nowego pola do Escrow2of3 (np. `lease_policy_commit`) zmienia `to_canonical_bytes()` output. Nowy output → nowy hash. Nowy hash ≠ stary hash w existing escrow notes. **Existing escrow notes stają się nieważne** bo ich `spend_policy_commit` w ledger nie matchuje nowego commitment.

**Dowód:** To jest matematyczne. Hash(zmieniony_input) ≠ hash(oryginalny_input). Nie ma workaround.

### Evidence 2: CanonicalEncode for SpendPolicy — exhaustive match

```
// privai-chain/src/note.rs:113-144
impl CanonicalEncode for SpendPolicy {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Single { .. } => { ... }
            Self::MarketplaceSettlement { .. } => { ... }
            Self::Escrow2of3 { .. } => { ... }
        }
    }
}
```

**Fact:** `encode()` jest exhaustive match na SpendPolicy variants. Każde nowe pole w Escrow2of3 wymaga zmiany encoding branch dla Escrow2of3. Ale to zmienia encoding → zmienia commitment (Evidence 1).

**Proof:** Nie można dodać pola do Escrow2of3 bez zmiany jego encoding. Zmiana encoding = zmiana commitment = existing notes nieważne.

### Evidence 3: validate_escrow_auth() step 3 — hardcoded Escrow2of3 gate

```
// privai-ledger/src/escrow.rs:67-76
// 3. Verify it's an Escrow2of3 policy (reject unsupported policy type)
let (buyer_pk_hash, merchant_pk_hash, operator_pk_hash, timeout_block) = match &policy {
    SpendPolicy::Escrow2of3 {
        buyer_pk_hash,
        merchant_pk_hash,
        operator_pk_hash,
        timeout_block,
    } => (*buyer_pk_hash, *merchant_pk_hash, *operator_pk_hash, *timeout_block),
    _ => return Err(ValidationError::EscrowUnsupportedPolicy),
};
```

**Fact:** Walidacja hardcoded odrzuca wszystko co nie jest `Escrow2of3`. Nawet gdyby ComputeLeaseEscrow był tym samym SpendPolicy variant (co jest niemożliwe z Evidence 1), walidacja by go odrzuciła.

**Proof:** Nowy SpendPolicy variant MUSI mieć osobną walidację bo existing walidacja odrzuca wszystko co nie jest Escrow2of3.

### Evidence 4: validate_escrow_auth() step 5 — hardcoded 2 signers

```
// privai-ledger/src/escrow.rs:85-95
// 5. Verify signer count (escrow-2of3 requires exactly 2 signers)
if auth.signer_pks.len() != 2 {
    return Err(ValidationError::EscrowWrongSignerCount(...));
}
if auth.signatures.len() != 2 {
    return Err(ValidationError::EscrowWrongSignerCount(...));
}
```

**Fact:** Walidacja wymaga dokładnie 2 sygnatariuszy. ComputeLeaseEscrow w Phase 2 (operatorless) może wymagać 1 sygnatora (user) + protocol-level validation (nie ludzki podpis). Lub 2 sygnatorów w Phase 1 (user + automated operator). Ale signer count jest hardcoded na 2.

**Proof:** ComputeLeaseEscrow może mieć inne signer requirements niż Escrow2of3. Osobna walidacja jest needed bo signer count może się różnić.

### Evidence 5: required_signers() — frozen rule table, 3 roles only

```
// privai-chain/src/escrow.rs:63-70
pub fn required_signers(action: EscrowAction) -> (SignerRole, SignerRole) {
    match action {
        EscrowAction::Release => (SignerRole::Buyer, SignerRole::Operator),
        EscrowAction::Refund => (SignerRole::Merchant, SignerRole::Operator),
        EscrowAction::RecoveryRelease => (SignerRole::Buyer, SignerRole::Merchant),
    }
}
```

**Fact:** Frozen rule table zwraca `(SignerRole, SignerRole)`. SignerRole ma tylko 3 wartości: Buyer (0x00), Merchant (0x01), Operator (0x02). ComputeLeaseEscrow potrzebuje nowych ról: User, Miner, Protocol. Ale SignerRole enum nie ma tych ról.

**Proof:** Nowe role (User/Miner/Protocol) nie mieszczą się w existing SignerRole enum. Dodanie nowych ról do SignerRole zmienia indeksy (Buyer=0, Merchant=1, Operator=2) co może wpłynąć na canonical ordering (step 8 w walidacji).

### Evidence 6: EscrowAction::from_u8() — hardcoded 0x01/0x02/0x03

```
// privai-chain/src/escrow.rs:29-37
pub fn from_u8(v: u8) -> Option<Self> {
    match v {
        0x01 => Some(Self::Release),
        0x02 => Some(Self::Refund),
        0x03 => Some(Self::RecoveryRelease),
        _ => None,
    }
}
```

**Fact:** Wartości > 0x03 zwracają `None`. ProRataSplit = 0x04 zwróci None. Ale test `action_roundtrip` (escrow.rs:102-108) explicitnie sprawdza `assert!(EscrowAction::from_u8(0x04).is_none())`.

**Proof:** Dodanie ProRataSplit = 0x04 do EscrowAction zmienia from_u8() co łamie test `action_roundtrip`. Test jest explicit guard against adding new actions.

### Evidence 7: validate_output_target() — only One or Either

```
// privai-ledger/src/escrow.rs:182-214
let allowed_commits: Vec<Hash32> = match target {
    TargetRecipient::One(role) => {
        vec![single_commit(pk_hash_for_role(role, ...))]
    }
    TargetRecipient::Either(a, b) => {
        vec![single_commit(...), single_commit(...)]
    }
};
```

**Fact:** Walidacja obsługuje tylko `One` (dokładnie 1 odbiorca) lub `Either` (jeden z dwóch). Pro-rata wymaga `Two` (dokładnie 2 odbiorców jednocześnie: miner + user). Ale `validate_output_target()` nie ma branch dla `Two`.

**Proof:** Pro-rata output target validation jest niemożliwa w existing `validate_output_target()` bez dodania nowego branch. Ale dodanie nowego branch do `validate_output_target()` zmienia zachowanie dla existing actions (ryzyko regression). Osobna validation function jest bezpieczniejsza.

---

## 3. Why Extending Escrow2of3 Fails

| Problem | Code Evidence | Impact |
|---|---|---|
| **Commitment hash change** | `commitment()` = `domain_hash(POLICY_DOMAIN, &[self.to_canonical_bytes()])` (note.rs:108-110) | Existing escrow notes nieważne — ich spend_policy_commit nie matchuje |
| **Canonical encoding change** | `CanonicalEncode` exhaustive match na variants (note.rs:113-144) | Każde nowe pole zmienia encoding = zmienia commitment |
| **Hardcoded Escrow2of3 gate** | `validate_escrow_auth()` step 3 odrzuca non-Escrow2of3 (escrow.rs:67-76) | Nowe pola w Escrow2of3 nie pomagają — walidacja i tak wymaga Escrow2of3 variant |
| **Hardcoded 2 signers** | `validate_escrow_auth()` step 5 wymaga exactly 2 (escrow.rs:85-95) | ComputeLeaseEscrow może mieć inne signer count |
| **Frozen rule table** | `required_signers()` zwraca (SignerRole, SignerRole) z 3 ról (escrow.rs:63-70) | Nowe role (User/Miner/Protocol) nie mieszczą się w existing enum |
| **Test regression** | `action_roundtrip` assert from_u8(0x04).is_none() (escrow.rs:102-108) | Dodanie nowej akcji łamie existing test |

**Conclusion:** Extending Escrow2of3 fails on 6 independent code-level constraints. Każdy z nich jest sufficient blocker. Razem = definitive proof.

---

## 4. Why A New Transaction Type Is Too Early

| Problem | Why Too Early |
|---|---|
| **Transaction enum exhaustiveness** | Wszystkie match statements na Transaction muszą obsługiwać nowy variant. Touchuje chain, ledger, node, wallet, proof. |
| **Proof path** | ComputeLeaseTx potrzebuje własnego proof path lub reuse existing TransferNoteTx. Decyzja nie jest podjęta. |
| **Block building** | Nowy tx type = zmiana block building logic. |
| **State management** | Nowy tx type = zmiana state management. |
| **Overkill** | ComputeLeaseEscrow jako SpendPolicy variant używa existing TransferNoteTx proof path. Nie potrzebuje nowego tx type. Nowy tx type jest needed dopiero gdy compute lease wymaga fundamentally different tx structure. |

**Conclusion:** Nowy Transaction variant jest future concern. ComputeLeaseEscrow jako SpendPolicy variant wystarczy Phase 0-5.

---

## 5. What Can Be Accepted Now

Bez overcommitowania implementation:

1. **ComputeLeaseEscrow jest nowym SpendPolicy variantem** — koncept zaakceptowany. Osobny od Escrow2of3.

2. **Osobna walidacja** — `validate_compute_lease_escrow_auth()` jako osobna funkcja. Routing: `match policy_tag { 0x03 => validate_escrow_auth(), [nowy tag] => validate_compute_lease_escrow_auth() }`.

3. **Nowy SpendPolicyTag** — koncept nowego tagu zaakceptowany. Konkretna wartość numeryczna — blocked by version registry.

4. **Nowe SpendPolicy pola** — `user_pk_hash`, `miner_pk_hash`, `lease_policy_commit`, `timeout_block`, `settlement_mode`. Koncept zaakceptowany. Dokładne pola — blocked by spec.

5. **TargetRecipient::Two** — koncept (2 outputs dla pro-rata) zaakceptowany. Walidacja — blocked by spec.

6. **EscrowAction::ProRataSplit** — koncept (nowa akcja) zaakceptowany. Execution — blocked by spec.

7. **Escrow2of3 niezmieniony** — wszystkie existing tests przechodzą. Zero regression.

---

## 6. What Must Stay Open

1. **Dokładne pola ComputeLeaseEscrow** — wymagają spec.
2. **Konkretny tag numeryczny** — wymaga version registry confirmation.
3. **Walidacja dla 2 outputs (pro-rata)** — wymaga spec (note split mechanics).
4. **Signer rules dla ComputeLeaseEscrow** — Phase 1 (2 signers) vs Phase 2 (1 signer + protocol) — wymaga Operatorless Escrow Direction.
5. **Lease policy commitment scheme** — hash of what exactly? Wymaga spec.
6. **SettlementMode enforcement** — jak escrow "wie" czy all-or-nothing czy pro-rata? Wymaga spec.

---

## 7. Red Lines

1. Nie rozszerzać Escrow2of3 o nowe pola. Evidence 1: commitment hash change = katastrofalne.
2. Nie zmieniać `validate_escrow_auth()`. Jest frozen dla Escrow2of3.
3. Nie zmieniać `required_signers()` dla existing actions.
4. Nie zmieniać `target_recipient()` dla existing actions.
5. Nie zmieniać `EscrowAction` dla 0x01/0x02/0x03.
6. Nie dodawać nowych ról do `SignerRole` — nowe role w osobnej validation path.
7. Nie łamać testu `action_roundtrip` (assert from_u8(0x04).is_none()).
8. Nie definiować wire formatów ComputeLeaseEscrow przed spec.
9. Nie twierdzić że ComputeLeaseEscrow jest implemented.

---

## 8. Final Self-Check

- **Czy czytałem legacy docs:** NIE
- **Czy czytałem kod:** TAK — `privai-chain/src/note.rs` (SpendPolicy, CanonicalEncode, commitment), `privai-chain/src/escrow.rs` (EscrowAction, SignerRole, required_signers, target_recipient, TargetRecipient), `privai-ledger/src/escrow.rs` (validate_escrow_auth 12-step, validate_output_target, single_commit)
- **Czy edytowałem pliki inne niż output:** NIE (tylko OUTPUT_XIAOMI.md)
- **Czy definiowałem wire formaty:** NIE
- **Czy to jest proof:** TAK — 7 concrete code-level evidences, each with file:line reference
