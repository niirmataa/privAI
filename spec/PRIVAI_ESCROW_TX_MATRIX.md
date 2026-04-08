# privAI Escrow Tx Matrix

Status: execution-level action matrix for escrow transactions in privAI.
Canonicality: supporting escrow-execution document. This document does not override canonical protocol, formats, consensus or product semantics; it translates the high-level escrow model into action-level semantics: who authorizes what, under which mode, with which policy constraints, and what ledger must verify. Binary formats and proof integration are out of scope.
Owner: privAI tx/auth, ledger and escrow architecture.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_EXECUTION_SPINE.md`
- `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
- `spec/PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`
- `spec/PRIVAI_ESCROW_FINAL_MODEL.md`
- `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_DECISION_REGISTER.md`
- `spec/PRIVAI_GAP_REGISTER.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- przelozyc high-level escrow model z `PRIVAI_ESCROW_FINAL_MODEL.md` na execution-level action matrix,
- zdefiniowac dla kazdej akcji escrow: kto autoryzuje, pod jakim trybem, z jakimi policy constraints, co ledger musi zweryfikowac,
- dac jednoznaczna baze pod implementacje ledger validation i nexum-core orchestration,
- odciac wieloznacznosc miedzy akcjami, trybami i rolami.

Ten dokument nie jest:
- binary format spec (field layouts, canonical encoding sa poza scope),
- proof integration doc (to bedzie `PRIVAI_ESCROW_PROOF_INTEGRATION.md`),
- implementacja Rust.

## 2. Current Direction

Escrow tx matrix operuje w ramach nastepujacych zamrozonych regul:
- escrow jest note-based, policy-constrained 2-of-3 multisig na railu `FullPrivacy`,
- normal mode wymaga Operatora jako jednego z sygnatariuszy,
- recovery mode to Buyer + Merchant bez Operatora,
- podpisy sa action-bound przez `tx_signing_hash`,
- threshold auth jest weryfikowany wzgledem policy zrekonstruowanej z `policy_opening`,
- high-level signer/action intent jest zamrozony w `PRIVAI_ESCROW_FINAL_MODEL.md` i ten dokument go nie odwraca.

## 3. Action Inventory

System escrow definiuje nastepujace akcje:

| Action | Description | Mode |
|--------|-------------|------|
| `EscrowFund` | Buyer deponuje noty do escrow — tworzy zablokowana note z policy 2-of-3 | n/a (single-signer) |
| `ReleaseToMerchant` | Uwolnienie srodkow do Merchanta po potwierdzeniu dostawy/uslugi | Normal |
| `RefundToBuyer` | Zwrot srodkow do Buyera po akceptacji reklamacji | Normal |
| `RecoveryRelease` | Awaryjne uwolnienie srodkow bez Operatora | Recovery |

## 4. Action: `EscrowFund`

### 4.1. Semantyka

Buyer tworzy `FullPrivacy` note z `spend_policy_commit` wiazacym polityke 2-of-3. Od momentu fundingu nota jest zablokowana i moze byc wydana tylko zgodnie z regula progu.

### 4.2. Authorization

| Field | Value |
|-------|-------|
| Mode | n/a — to nie jest akcja escrow spend, to jest funding |
| Required signers | Buyer (1-of-1, standardowy spend istniejacych not Buyera) |
| Operator required | nie |
| Threshold | 1-of-1 (standard `FullPrivacy` spend auth) |

### 4.3. Policy constraints

- Output note musi miec `spend_policy_commit` wiazacy polityke 2-of-3.
- `spend_policy_commit` musi byc poprawnym commitmentem nad:
  - `buyer_pk_hash`
  - `merchant_pk_hash`
  - `operator_pk_hash`
  - `timeout_block` (jesli applicable)
- Buyer wydaje swoje wczesniejsze noty przez standardowy spend auth.

### 4.4. Ledger checks

- Standard `FullPrivacy` spend validation dla inputow Buyera.
- Output note creation validation.
- Policy family/version validation: ledger musi zweryfikowac, ze `spend_policy_commit` odnosi sie do obslugiwanego typu policy (np. escrow 2-of-3) i wersji wspieranej przez protokol. Bez tego mozna fundowac noty pod nieobslugiwane lub bezsensowne policy, tworzac stan, ktorego ledger nie umie pozniej poprawnie egzekwowac.
- Nullifier/replay protection dla inputow Buyera.

### 4.5. What `tx_signing_hash` must bind

- tx version, tx class
- input references (noty Buyera)
- output commitments (escrow note z `spend_policy_commit`)
- fee
- action semantics (standard FullPrivacy transfer, nie escrow action)

### 4.6. Public vs private

| Field | Visibility |
|-------|------------|
| Input nullifiers | public (ledger must verify) |
| Output note commitment | public (ledger must verify) |
| `spend_policy_commit` | embedded in output note commitment — not separately public |
| Amount | private (proof-carried, standard FullPrivacy) |
| Buyer identity | private (standard FullPrivacy) |
| Merchant/Operator identity | not visible at funding time |

## 5. Action: `ReleaseToMerchant`

### 5.1. Semantyka

Srodki z escrow note sa uwalniane do Merchanta. Buyer potwierdza, ze ustalenia zostaly spelnione (dostawa/usluga). Operator wspolautoryzuje jako arbiter.

### 5.2. Authorization

| Field | Value |
|-------|-------|
| Mode | Normal |
| Required signers | Buyer + Operator |
| Operator required | tak |
| Threshold | 2-of-3 (Buyer + Operator z policy signer set) |

### 5.3. Policy constraints

- Action type musi byc `release`.
- Output destination musi byc zgodne z Merchant target zdefiniowanym w policy lub proposal.
- Signers musza nalezec do signer set zrekonstruowanego z `policy_opening`.
- Signer ordering musi byc canonical (wg signer index in policy).
- Duplicate signer rejection.
- Operator musi byc obecny (normal mode rule).

### 5.4. Ledger checks

- Input escrow note existence i unspent status.
- `policy_opening` match z `spend_policy_commit` escrow note.
- Reconstruction of signer set i threshold z `policy_opening`.
- Falcon signature verification dla kazdego signera.
- Signer membership w policy signer set.
- Quorum: co najmniej 2 poprawne, rozne signery.
- Operator presence check (normal mode).
- Action type binding w `tx_signing_hash`.
- Output destination validation (srodki do Merchanta, nie do dowolnego adresu).
- Canonical signer ordering.
- Duplicate signer rejection.
- Nullifier / replay protection.
- Standard state transition validation.

### 5.5. What `tx_signing_hash` must bind

- tx version, tx class
- action type: `release`
- input references (escrow note)
- output commitments (note do Merchanta)
- fee
- policy-relevant fields (policy tag, signer roles)
- escrow-specific context (proposal hash lub snapshot reference, jesli applicable)

### 5.6. Public vs private

| Field | Visibility |
|-------|------------|
| Input nullifier | public (ledger must verify) |
| `policy_opening` | public to ledger (ledger must reconstruct policy) |
| Signer identities (Buyer, Operator) | public to ledger (ledger must verify membership and signatures) |
| Signatures | public to ledger |
| Action type (`release`) | public to ledger (bound in `tx_signing_hash`) |
| Output note commitment | public (ledger must verify) |
| Amount | may remain private (proof-carried in future) |
| Merchant receive address details | embedded in output note — may remain private depending on proof model |

Uwaga o v1 visibility: w v1 escrow dziala przez ledger-enforced public validation, wiec dane oznaczone "public to ledger" sa w praktyce widoczne dla walidatorow i obserwatorow transakcji. Docelowy proof model (scope `PRIVAI_ESCROW_PROOF_INTEGRATION.md`) moze pozniej ukryc czesc tych danych przed publicznym obserwatorem zachowujac przejrzystosc w warstwie weryfikatora — ale to nie jest stan v1.

## 6. Action: `RefundToBuyer`

### 6.1. Semantyka

Srodki z escrow note sa zwracane do Buyera. Merchant akceptuje reklamacje lub Operator mediuje i wspolnie z Merchantem autoryzuje zwrot.

### 6.2. Authorization

| Field | Value |
|-------|-------|
| Mode | Normal |
| Required signers | Merchant + Operator |
| Operator required | tak |
| Threshold | 2-of-3 (Merchant + Operator z policy signer set) |

### 6.3. Policy constraints

- Action type musi byc `refund`.
- Output destination musi byc zgodne z Buyer target.
- Signers musza nalezec do signer set zrekonstruowanego z `policy_opening`.
- Signer ordering canonical (wg signer index in policy).
- Duplicate signer rejection.
- Operator musi byc obecny (normal mode rule).

### 6.4. Ledger checks

- Input escrow note existence i unspent status.
- `policy_opening` match z `spend_policy_commit`.
- Reconstruction of signer set i threshold z `policy_opening`.
- Falcon signature verification.
- Signer membership.
- Quorum: co najmniej 2 poprawne, rozne signery.
- Operator presence check (normal mode).
- Action type binding w `tx_signing_hash`.
- Output destination validation (srodki do Buyera).
- Canonical signer ordering.
- Duplicate signer rejection.
- Nullifier / replay protection.
- Standard state transition validation.

### 6.5. What `tx_signing_hash` must bind

- tx version, tx class
- action type: `refund`
- input references (escrow note)
- output commitments (note do Buyera)
- fee
- policy-relevant fields
- escrow-specific context

### 6.6. Public vs private

Analogicznie do `ReleaseToMerchant` (sekcja 5.6), z ta roznica, ze:
- signer identities to Merchant + Operator (zamiast Buyer + Operator),
- output destination to Buyer (zamiast Merchant).

## 7. Action: `RecoveryRelease`

### 7.1. Semantyka

Awaryjne uwolnienie srodkow bez udzialu Operatora. Buyer i Merchant wspolnie decyduja o przekazaniu srodkow — to jest ostateczna sciezka ratunkowa, dostepna dopiero po spelnieniu recovery precondition (timeout).

### 7.2. Authorization

| Field | Value |
|-------|-------|
| Mode | Recovery |
| Required signers | Buyer + Merchant |
| Operator required | nie |
| Threshold | 2-of-3 (Buyer + Merchant z policy signer set) |

### 7.3. Policy constraints

- Action type musi byc `recovery_release`.
- Signers musza nalezec do signer set zrekonstruowanego z `policy_opening`.
- Signer ordering canonical.
- Duplicate signer rejection.
- Operator NIE jest wymagany — to jest swiadome odroznienie od normal mode.
- Output destination musi byc zgodne z celem uzgodnionym miedzy Buyerem i Merchantem (v1: cala kwota do jednej strony; split distribution jest future extension).

Recovery precondition (kiedy recovery mode jest dozwolony):
- recovery jest dozwolony dopiero po uplynieciu `timeout_block` zdefiniowanego w policy,
- przed timeout: tylko normal mode (operator-required) jest dostepny,
- po timeout: Buyer + Merchant moga wspolnie podpisac bez Operatora,
- to gwarantuje, ze normal mode jest realna regula, nie tylko preferowana sciezka — Operator ma czas na mediacje zanim strony moga go ominac.

Uzasadnienie:
- bez timeout precondition strony moga zawsze natychmiast ominac Operatora, co czyni operator-required normal mode fikcyjnym,
- timeout daje Operatorowi okno na dzialanie, a jednoczesnie zapobiega trwalemu zablokowaniu not.

### 7.4. Ledger checks

- Input escrow note existence i unspent status.
- `policy_opening` match z `spend_policy_commit`.
- Reconstruction of signer set i threshold.
- Falcon signature verification.
- Signer membership.
- Quorum: co najmniej 2 poprawne, rozne signery.
- Operator presence check: **NIE wymagany** (recovery mode).
- Action type binding w `tx_signing_hash`.
- Output destination validation.
- Canonical signer ordering.
- Duplicate signer rejection.
- Nullifier / replay protection.
- Standard state transition validation.
- Recovery precondition check: current block height >= `timeout_block` z policy.

### 7.5. What `tx_signing_hash` must bind

- tx version, tx class
- action type: `recovery_release`
- input references (escrow note)
- output commitments (note do Buyera lub Merchanta; v1: cala kwota do jednej strony)
- fee
- policy-relevant fields
- recovery-specific context (jesli applicable)

### 7.6. Public vs private

Analogicznie do normal mode actions (sekcja 5.6), z ta roznica, ze:
- signer identities to Buyer + Merchant,
- brak Operatora w signer set,
- v1: output to cala kwota do jednej strony (Buyer lub Merchant); split distribution jest future extension.

## 8. Consolidated Action Matrix

| Action | Mode | Signers | Operator req | Threshold | Output target | Action type |
|--------|------|---------|--------------|-----------|---------------|-------------|
| `EscrowFund` | n/a | Buyer | no | 1-of-1 | escrow note | standard spend |
| `ReleaseToMerchant` | Normal | Buyer + Operator | yes | 2-of-3 | Merchant | `release` |
| `RefundToBuyer` | Normal | Merchant + Operator | yes | 2-of-3 | Buyer | `refund` |
| `RecoveryRelease` | Recovery | Buyer + Merchant | no | 2-of-3 | Buyer or Merchant (v1: whole amount) | `recovery_release` |

## 9. Consolidated Ledger Check Matrix

| Check | EscrowFund | Release | Refund | Recovery |
|-------|------------|---------|--------|----------|
| Input existence + unspent | yes | yes | yes | yes |
| `policy_opening` match | n/a (funding) | yes | yes | yes |
| Policy reconstruction | n/a | yes | yes | yes |
| Falcon sig verification | yes (1-of-1) | yes | yes | yes |
| Signer membership | yes (standard) | yes (policy) | yes (policy) | yes (policy) |
| Quorum check | 1-of-1 | 2-of-3 | 2-of-3 | 2-of-3 |
| Operator presence | n/a | required | required | not required |
| Action type in `tx_signing_hash` | standard | `release` | `refund` | `recovery_release` |
| Output destination validation | escrow note | Merchant | Buyer | Buyer or Merchant |
| Canonical signer ordering | n/a (single) | yes | yes | yes |
| Duplicate signer rejection | n/a (single) | yes | yes | yes |
| Nullifier / replay | yes | yes | yes | yes |
| Recovery precondition (timeout) | n/a | n/a | n/a | block height >= `timeout_block` |

## 10. Relationship to `tx_signing_hash`

Dla kazdej akcji escrow `tx_signing_hash` musi wiazac:
- action type (rozrozniajacy `release` / `refund` / `recovery_release`),
- input references (escrow note),
- output commitments (destination note(s)),
- fee,
- policy-relevant fields.

To gwarantuje:
- podpis zlozony na `release` nie moze byc uzyty do `refund` ani `recovery_release`,
- podpis zlozony na konkretne output commitments nie moze byc uzyty do przekierowania srodkow,
- replay protection jest wbudowana w action binding.

## 11. Relationship to `policy_opening`

Dla kazdej akcji escrow spend (release, refund, recovery):
- ledger rekonstruuje polityke z `policy_opening`,
- signer set, threshold i role constraints pochodza z zrekonstruowanej policy,
- ledger sprawdza, czy `policy_opening` po commitcie daje `spend_policy_commit` escrow note,
- bez poprawnego `policy_opening` zadna akcja escrow spend nie przechodzi.

`EscrowFund` nie wymaga `policy_opening` — to jest funding, nie spend.

## 12. Relationship to Proof Integration

Ten dokument definiuje co jest "public to ledger" per action.

Docelowy proof model moze:
- ukryc czesc tych danych przed publicznym obserwatorem,
- zachowac przejrzystosc w warstwie weryfikatora,
- zmienic public/private split dla konkretnych pol.

To jest scope `PRIVAI_ESCROW_PROOF_INTEGRATION.md` i nie jest zdefiniowane tutaj.

Twarda zasada:
- v1 escrow dziala z ledger-enforced public validation,
- proof-aware privacy jest rozszerzeniem, nie prerequisite.

## 13. What This Document Does Not Define

- Binary field layouts i canonical encoding per action.
- Proof circuit details per action.
- Exact dispute on-chain semantics beyond timeout (timeout precondition for recovery is defined in this document).
- `EscrowSnapshot` / `EscrowSpendProposal` / `EscrowApprovalBundle` field-level formats.
- nexum-core orchestration protocol details.

## 14. Checklist

- [ ] Potwierdzic spojnosc action matrix z high-level intent z `PRIVAI_ESCROW_FINAL_MODEL.md`.
- [ ] Zdefiniowac exact action type encoding (canonical action tags).
- [ ] Zdefiniowac exact `policy_opening` field set per policy type.
- [ ] Zdefiniowac exact output destination validation rules per action.
- [ ] Zdefiniowac future dispute semantics / extra recovery conditions beyond timeout (v1 timeout precondition is already defined in section 7).
- [ ] Zdefiniowac exact operator presence check semantics.
- [ ] Dodac regression test scenarios per action (wg CP-07 z ESCROW_2OF3_ADAPTATION).
- [ ] Zsynchronizowac z `PRIVAI_CANONICAL_FORMATS.md` po ustaleniu field layouts.

## 15. Exit Criteria

Faza escrow tx matrix jest domknieta, gdy:
- kazda akcja ma jednoznaczny signer set, threshold, policy constraints i ledger checks,
- action matrix jest zgodna z high-level intent z ESCROW_FINAL_MODEL,
- action type binding w `tx_signing_hash` jest jednoznaczne per action,
- `policy_opening` reconstruction path jest jednoznaczny per action,
- public vs private split jest jasny per action (nawet jesli proof model go pozniej zmodyfikuje),
- regression scenarios sa zdefiniowane.
