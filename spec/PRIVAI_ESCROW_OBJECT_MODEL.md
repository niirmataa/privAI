# privAI Escrow Object Model

Status: canonical object model for escrow control-plane and on-chain objects in privAI.
Canonicality: supporting escrow-object document. This document does not override canonical protocol, formats, consensus, proof or product semantics; it defines the canonical objects required for escrow orchestration and execution, their semantics, their boundaries (on-chain vs off-chain vs control-plane) and their relationships. Binary encoding is out of scope.
Owner: privAI escrow, auth, ledger and integration architecture.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_EXECUTION_SPINE.md`
- `spec/PRIVAI_ESCROW_FINAL_MODEL.md`
- `spec/PRIVAI_ESCROW_TX_MATRIX.md`
- `spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md`
- `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md`
- `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
- `spec/PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_GAP_REGISTER.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zdefiniowac canonical object model dla escrow w privAI,
- zamknac GAP-013 na poziomie semantyki obiektow,
- zapisac, ktore obiekty sa on-chain, ktore off-chain, a ktore control-plane,
- zapisac relacje miedzy obiektami,
- dac baze pod przyszle binary encoding i implementacje.

Ten dokument nie jest:
- binary encoding spec (field layouts i canonical encoding sa poza scope),
- proof integration doc (to jest `PRIVAI_ESCROW_PROOF_INTEGRATION.md`),
- implementacja Rust,
- marketplace v0 object model.

## 2. Current Direction

Escrow object model operuje w ramach zamrozonych regul z `PRIVAI_ESCROW_FINAL_MODEL.md`:
- escrow jest note-based, policy-constrained 2-of-3 na railu `FullPrivacy`,
- normal mode jest operator-required,
- recovery mode jest buyer+merchant po timeout,
- `nexum-core` koordynuje, ledger weryfikuje,
- obiekty dzielimy na on-chain (ledger-visible), off-chain (persistent but not on-chain) i control-plane (nexum-core orchestration).

## 3. Object Inventory

| Object | Layer | Persistence | Purpose |
|--------|-------|-------------|---------|
| `EscrowFundingDescriptor` | off-chain | persistent | instrukcja fundingu dla buyer wallet |
| `EscrowSnapshot` | off-chain / control-plane | persistent | canonical state kontraktu escrow |
| `EscrowSpendProposal` | off-chain / control-plane | persistent | konkretna propozycja wydania escrow note |
| `EscrowApprovalBundle` | off-chain / control-plane -> on-chain | transient -> submitted | zebrane approvals do submitu na chain |
| `EscrowPolicy` | on-chain (as commitment) | durable | policy 2-of-3 commitowana w escrow note |

## 4. Object: `EscrowFundingDescriptor`

### 4.1. Semantyka

Paczka danych, ktora buyer wallet potrzebuje, zeby ufundowac escrow note. Nie trafia na chain — sluzy do skonstruowania poprawnego `FullPrivacy OutputNote` z wlasciwym `spend_policy_commit`.

### 4.2. Layer

Off-chain. Generowany przez escrow runtime / nexum-core, przekazywany do buyer wallet.

### 4.3. Canonical fields

- `escrow_id`: unique identifier escrow kontraktu
- `receive_bundle`: jednorazowy `ReceiveBundle` wygenerowany przez escrow runtime (zawiera recipient public material potrzebny do zbudowania output note)
- `spend_policy_commit`: commitment nad policy 2-of-3
- `expiry`: czas waznosci descriptora (nie timeout escrow — to jest w policy)
- `context_commit`: opcjonalny commitment nad dodatkowym kontekstem (np. order reference)
- `descriptor_version`: wersja formatu descriptora

### 4.4. Lifecycle

1. nexum-core otwiera escrow i ustala role, timeout, warunki.
2. Escrow runtime generuje jednorazowy `ReceiveBundle`.
3. System tworzy policy 2-of-3 i liczy `spend_policy_commit`.
4. Z tych danych powstaje `EscrowFundingDescriptor`.
5. Descriptor jest przekazywany do buyer wallet.
6. Buyer wallet buduje `FullPrivacy OutputNote` do descriptora.
7. Po fundingu descriptor nie jest juz potrzebny do dalszych operacji (ale moze byc zachowany dla audytu).

### 4.5. Relationship to on-chain

Descriptor sam nie trafia na chain. Jego skutkiem jest output note na chain z `spend_policy_commit`.

## 5. Object: `EscrowSnapshot`

### 5.1. Semantyka

Off-chain canonical object opisujacy pelny stan kontraktu escrow. Snapshot jest wspolnym punktem odniesienia — wszyscy uczestnicy podpisuja proposal odnoszacy sie do tego samego snapshotu.

### 5.2. Layer

Off-chain / control-plane. Zarzadzany przez nexum-core. Nie trafia na chain bezposrednio, ale `snapshot_hash` jest referencja w proposalach.

### 5.3. Canonical fields

- `escrow_id`: unique identifier escrow kontraktu
- `buyer_id`: canonical signer identity Buyera
- `merchant_id`: canonical signer identity Merchanta
- `operator_id`: canonical signer identity Operatora
- `buyer_pk`: Falcon public key Buyera
- `merchant_pk`: Falcon public key Merchanta
- `operator_pk`: Falcon public key Operatora
- `funding_note_commit`: commitment ufundowanej escrow note (po fundingu)
- `spend_policy_commit`: commitment nad policy 2-of-3 (musi matchowac note)
- `policy_tag`: canonical policy tag dla escrow (np. `escrow-2of3-v1`)
- `policy_version`: wersja policy
- `timeout_block`: block height po ktorym recovery mode jest dozwolony
- `fee_cap`: maksymalny dozwolony fee
- `context_commit`: opcjonalny commitment nad kontekstem biznesowym
- `created_at`: timestamp lub block height utworzenia
- `snapshot_hash`: canonical hash calego snapshotu

Uwaga:
- `release_rule` i `refund_rule` nie sa osobnymi canonical fields snapshotu.
- Canonical action rules wynikaja z `policy_tag` + `policy_version` oraz zrekonstruowanego `EscrowPolicy`.
- Snapshot moze co najwyzej trzymac pochodna reprezentacje tych regul jako local metadata poza canonical hash, jesli control-plane tego potrzebuje.

### 5.4. Lifecycle

1. Tworzony po otwarciu escrow i ustaleniu rol/warunkow.
2. Aktualizowany po fundingu (dodanie `funding_note_commit`).
3. Referencyjny w kazdym `EscrowSpendProposal` (przez `snapshot_hash`).
4. Immutable po fundingu — nie moze byc zmieniany bez nowego escrow.

### 5.5. Invariants

- `snapshot_hash` musi byc deterministyczny i canonical.
- Zmiana jakiegokolwiek pola snapshotu po fundingu jest niedozwolona.
- Wszyscy uczestnicy musza operowac na tym samym `snapshot_hash`.
- `policy_tag`, `policy_version` i `spend_policy_commit` musza byc spojne z canonical `EscrowPolicy`.
- Snapshot nie moze stac sie drugim source of truth dla action rules; action rules wynikaja z `EscrowPolicy` / `policy_tag`, nie z lokalnych pol snapshotu.

## 6. Object: `EscrowSpendProposal`

### 6.1. Semantyka

Off-chain canonical object opisujacy konkretna propozycje wydania escrow note. Proposal jest tym, co signerzy faktycznie podpisuja (przez `tx_signing_hash` finalnej transakcji, ale `proposal_hash` sluzy jako control-plane reference).

### 6.2. Layer

Off-chain / control-plane. Tworzony przez nexum-core. Sluzy do koordynacji approvals.

### 6.3. Canonical fields

- `escrow_id`: unique identifier escrow kontraktu
- `snapshot_hash`: hash snapshotu, do ktorego proposal sie odnosi
- `action`: canonical action type (`release`, `refund`, `recovery_release`)
- `input_note_commits`: commitments escrow not do wydania
- `output_plan`: opis docelowych outputow (kto dostaje srodki)
- `fee`: proponowany fee
- `timeout_context`: block height lub timeout-relevant context (dla recovery)
- `proposal_hash`: canonical hash calego proposalu

### 6.4. Action types

V1 actions:
- `release` — srodki do Merchanta (normal mode: Buyer + Operator)
- `refund` — srodki do Buyera (normal mode: Merchant + Operator)
- `recovery_release` — srodki do Buyera lub Merchanta (recovery mode: Buyer + Merchant, po timeout)

Future extensions:
- `partial_release`
- `dispute_resolution`

### 6.5. Lifecycle

1. nexum-core tworzy proposal na podstawie zadania strony.
2. Proposal jest rozsylany do wymaganych signerow.
3. Signerzy weryfikuja proposal i skladaja approvals.
4. Po zebraniu quorum, proposal jest realizowany jako transakcja.

### 6.6. Invariants

- `proposal_hash` musi byc deterministyczny i canonical.
- `snapshot_hash` w proposalu musi matchowac aktualny snapshot.
- Proposal nie moze byc zmodyfikowany po rozpoczeciu zbierania approvals.
- Jeden proposal = jedna akcja = jeden zestaw outputow.

## 7. Object: `EscrowApprovalBundle`

### 7.1. Semantyka

Bundle approvals zebranych od signerow przez nexum-core. To jest control-plane artifact, ktory po skompletowaniu quorum jest przeksztalcany w on-chain auth package w `TransferNoteTx`.

### 7.2. Layer

Control-plane (nexum-core) -> on-chain (jako czesc auth w `TransferNoteTx`).

### 7.3. Canonical fields

- `proposal_hash`: hash proposalu, do ktorego approvals sie odnosza
- `signer_entries`: lista zatwierdzonych approvals, kazdy zawierajacy:
  - `signer_index`: canonical signer index w policy
  - `signer_role`: rola signera (buyer / merchant / operator) — **control-plane metadata only**, nie autorytatywne zrodlo prawdy dla weryfikacji; ledger ustala role z policy reconstructed from `policy_opening`
  - `signer_pk`: Falcon public key signera
  - `signature`: Falcon signature nad `tx_signing_hash`
- `created_at`: timestamp zebrania bundle

### 7.4. V1 simplifications

- Brak kryptograficznej agregacji threshold signatures — V1 uzywa dwoch osobnych podpisow Falcon.
- Signer ordering w bundle jest canonical wg signer index in policy.
- Duplicate signer material jest odrzucane.

### 7.5. Lifecycle

1. nexum-core zbiera approval od pierwszego signera.
2. nexum-core zbiera approval od drugiego signera.
3. Po osiagnieciu quorum (2-of-3), nexum-core sklada `EscrowApprovalBundle`.
4. Escrow runtime przeksztalca bundle w auth package `TransferNoteTx`.
5. `TransferNoteTx` jest broadcastowana do sieci.
6. Ledger weryfikuje auth package (signer membership, quorum, ordering, action binding).

### 7.6. Invariants

- Wszystkie approvals musza odnosic sie do tego samego `proposal_hash`.
- Wszystkie signatures musza byc nad tym samym `tx_signing_hash`.
- Signer ordering musi byc canonical.
- Duplicate signers musza byc odrzuceni.
- Quorum musi byc spelnione przed submitem.

### 7.7. Relationship to on-chain auth

`EscrowApprovalBundle` jest control-plane representation tego, co na chainie staje sie auth package w `TransferNoteTx`:
- `signer_pks` -> auth signer identities
- `signatures` -> auth signatures
- `policy_opening` jest dostarczany osobno (nie jest czescia approval bundle, ale czescia auth package transakcji)

## 8. Object: `EscrowPolicy`

### 8.1. Semantyka

Policy 2-of-3 commitowana w escrow note przez `spend_policy_commit`. Nie jest osobnym on-chain obiektem — istnieje jako commitment w note i jest rekonstruowana z `policy_opening` przy spendzie.

### 8.2. Layer

On-chain (as commitment in note). Policy content jest off-chain, udostepniany jako `policy_opening` przy spendzie.

### 8.3. Canonical fields (policy content)

- `policy_tag`: canonical tag identyfikujacy typ policy (np. `escrow-2of3-v1`)
- `buyer_pk_hash`: hash Falcon public key Buyera
- `merchant_pk_hash`: hash Falcon public key Merchanta
- `operator_pk_hash`: hash Falcon public key Operatora
- `timeout_block`: block height po ktorym recovery mode jest dozwolony
- `policy_version`: wersja policy (dla forward compatibility)

### 8.3a. Implied rule table

`policy_tag` + `policy_version` implikuja zamrozona tabele regul egzekwowanych przez ledger. Ledger nie odczytuje release/refund/recovery constraints z pol policy — odczytuje je z tabeli regul przypisanej do danego `policy_tag`.

Dla `escrow-2of3-v1` zamrozona tabela regul to:
- `release`: wymaga Buyer + Operator (normal mode), output do Merchanta
- `refund`: wymaga Merchant + Operator (normal mode), output do Buyera
- `recovery_release`: wymaga Buyer + Merchant (recovery mode), dostepne po `timeout_block`, output do Buyera lub Merchanta

Z `policy_opening` ledger odczytuje:
- kto jest w signer set (pk hashes),
- kiedy recovery jest dozwolony (timeout_block).

Z `policy_tag` ledger odczytuje:
- jaki threshold obowiazuje,
- jakie action/signer combinations sa dozwolone,
- czy operator jest wymagany per action.

To rozdzielenie zapobiega sytuacji, gdzie ktos konstruuje policy z dowolnymi action rules — dozwolone sa tylko rules zdefiniowane przez protocol dla danego `policy_tag`.

### 8.4. On-chain representation

- `spend_policy_commit = H_spend_policy(canonical(EscrowPolicy))`
- `spend_policy_commit` jest czescia output note commitment
- Przy spendzie: `policy_opening` jest dostarczany w auth package
- Ledger sprawdza: `H_spend_policy(canonical(policy_opening)) == spend_policy_commit`

### 8.5. Invariants

- `spend_policy_commit` jest immutable po fundingu.
- `policy_opening` musi jednoznacznie odtwarzac policy content.
- Ledger nie akceptuje spendu bez poprawnego `policy_opening`.

## 9. Object Relationships

```
EscrowSnapshot
  |
  |-- contains: escrow_id, roles, keys, timeout, policy identifiers
  |-- references: funding_note_commit (after funding)
  |-- referenced by: EscrowSpendProposal (via snapshot_hash)
  |
  v
EscrowFundingDescriptor
  |
  |-- contains: escrow_id, receive_bundle, spend_policy_commit
  |-- consumed by: buyer wallet to create funding note
  |-- results in: on-chain escrow note with spend_policy_commit
  |
EscrowSpendProposal
  |
  |-- references: snapshot_hash, action, input_note_commits, output_plan
  |-- signed by: signers (via tx_signing_hash of resulting tx)
  |-- referenced by: EscrowApprovalBundle (via proposal_hash)
  |
  v
EscrowApprovalBundle
  |
  |-- references: proposal_hash
  |-- contains: signer approvals (signatures over tx_signing_hash)
  |-- transforms into: on-chain auth package in TransferNoteTx
  |
  v
TransferNoteTx (on-chain)
  |
  |-- contains: inputs, outputs, auth (from approval bundle), policy_opening
  |-- verified by: ledger (policy reconstruction, threshold, action binding)
```

## 10. On-chain vs Off-chain vs Control-plane

| Object | On-chain | Off-chain | Control-plane |
|--------|----------|-----------|---------------|
| `EscrowFundingDescriptor` | — | persistent (wallet) | generated by nexum-core |
| `EscrowSnapshot` | — | persistent | managed by nexum-core |
| `EscrowSpendProposal` | — | persistent | managed by nexum-core |
| `EscrowApprovalBundle` | transforms into auth package | transient | managed by nexum-core |
| `EscrowPolicy` | as `spend_policy_commit` in note | as `policy_opening` in auth | — |
| `TransferNoteTx` | yes (final tx) | — | submitted by escrow runtime |

Twarda zasada:
- nexum-core zarzadza off-chain/control-plane objects,
- ledger widzi tylko `TransferNoteTx` z auth package i `policy_opening`,
- ledger nie widzi snapshot, proposal ani approval bundle bezposrednio.

## 11. Relationship to `tx_signing_hash`

- Signerzy podpisuja `tx_signing_hash` finalnej `TransferNoteTx`, nie `proposal_hash`.
- `proposal_hash` jest control-plane reference — sluzy do koordynacji, nie do on-chain verification.
- Ledger weryfikuje auth package wzgledem `tx_signing_hash`, nie wzgledem `proposal_hash`.
- `tx_signing_hash` binduje action type, inputs, outputs, fee, policy-relevant fields.

Rozroznienie:
- `proposal_hash` = "co chcemy zrobic" (control-plane),
- `tx_signing_hash` = "co faktycznie autoryzujemy" (on-chain auth).

## 12. Relationship to `policy_opening`

- `policy_opening` jest canonical opening/encoding obiektu `EscrowPolicy` (sekcja 8), dolaczany do auth package transakcji przy spendzie.
- `EscrowPolicy` jest canonical off-chain policy object; `policy_opening` to jego serializowana forma dostarczana on-chain.
- `policy_opening` pozwala ledgerowi zrekonstruowac policy i zweryfikowac signer membership, threshold i action constraints.
- `spend_policy_commit` w escrow note musi matchowac `H_spend_policy(canonical(policy_opening))`.
- `EscrowPolicy` content (sekcja 8.3) definiuje, jakie pola `policy_opening` musi zawierac.

## 13. Relationship to nexum-core

nexum-core jest odpowiedzialne za:
- tworzenie i zarzadzanie `EscrowSnapshot`,
- tworzenie `EscrowFundingDescriptor` (we wspolpracy z escrow runtime),
- tworzenie `EscrowSpendProposal`,
- zbieranie approvals i skladanie `EscrowApprovalBundle`,
- workflow/state machine escrow (open -> funded -> proposed -> approved -> executed),
- replay/idempotency na poziomie control-plane.

nexum-core NIE jest odpowiedzialne za:
- weryfikacje poprawnosci auth package (to robi ledger),
- weryfikacje policy (to robi ledger),
- przechowywanie stanu on-chain,
- broadcast transakcji (to robi escrow runtime).

## 14. Replay and Idempotency

### 14.1. Control-plane level

- `escrow_id` jest unique per escrow kontakt.
- `proposal_hash` jest unique per proposal.
- nexum-core nie moze pozwolic na dwa rozne proposale z tym samym `proposal_hash`.
- nexum-core nie moze pozwolic na dwa rozne snapshoty z tym samym `escrow_id` po fundingu.

### 14.2. On-chain level

- Nullifier escrow note zapewnia, ze nota moze byc wydana dokladnie raz.
- Ledger odrzuca transakcje z juz zuzytym nullifierem.
- To jest niezalezne od control-plane replay protection.

## 15. What This Document Does Not Define

- Binary encoding i canonical field layouts (to bedzie czesc canonical formats).
- Proof integration details (to jest `PRIVAI_ESCROW_PROOF_INTEGRATION.md`).
- nexum-core internal state machine implementation.
- Marketplace v0 object model (oddzielny, nie powiazany).
- Exact hash functions i domain separators per object (to bedzie czesc canonical formats).

## 16. Checklist

- [ ] Zdefiniowac canonical hash function i domain separator dla `snapshot_hash`.
- [ ] Zdefiniowac canonical hash function i domain separator dla `proposal_hash`.
- [ ] Zdefiniowac canonical hash function i domain separator dla `spend_policy_commit`.
- [ ] Zdefiniowac exact `EscrowPolicy` field set i policy tag vocabulary.
- [ ] Zdefiniowac exact `policy_opening` wire format.
- [ ] Zdefiniowac exact `EscrowFundingDescriptor` wire format.
- [ ] Zdefiniowac exact `EscrowSnapshot` wire format.
- [ ] Zdefiniowac exact `EscrowSpendProposal` wire format.
- [ ] Zdefiniowac exact `EscrowApprovalBundle` wire format.
- [ ] Zdefiniowac mapping: `EscrowApprovalBundle` -> `TransferNoteTx` auth package.
- [ ] Zweryfikowac spojnosc object model z `PRIVAI_ESCROW_TX_MATRIX.md` per action.
- [ ] Dodac regression tests: snapshot immutability po fundingu.
- [ ] Dodac regression tests: proposal/approval consistency.
- [ ] Dodac regression tests: replay rejection (control-plane i on-chain).

## 17. Exit Criteria

Faza escrow object model jest domknieta, gdy:
- kazdy obiekt ma jednoznaczna semantyke, layer assignment i lifecycle,
- relacje miedzy obiektami sa jednoznaczne,
- on-chain vs off-chain vs control-plane split jest jawny,
- relationship do `tx_signing_hash` i `policy_opening` jest jednoznaczny,
- `proposal_hash` vs `tx_signing_hash` rozroznienie jest jawne,
- replay/idempotency boundaries sa jawne,
- follow-up wire format work jest nazwany w checklistcie.
