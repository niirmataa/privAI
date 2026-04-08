# privAI Execution Spine

Status: master execution spine for finishing the current architecture without letting code, specs and claims drift apart.
Canonicality: execution-control document only. This document does not override canonical protocol, formats, consensus, proof or product semantics. It exists to define the order of work, the dependency graph between documents, and the minimum exit criteria for each phase.
Owner: privAI architecture, ledger, proof, networking and tx/auth workstreams.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`
- `spec/PRIVAI_GAP_REGISTER.md`
- `spec/PRIVAI_DECISION_REGISTER.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- utrzymac jeden wspolny tor wykonawczy dla calego projektu,
- odciac przypadkowe skakanie miedzy warstwami,
- ustalic kolejnosc:
  - transport
  - auth
  - chain / ledger / node
  - proof
  - escrow
  - marketplace,
- sprawic, zeby wszystkie kolejne docs:
  - mialy jasne miejsce,
  - mialy jasny scope,
  - nie dublowaly sie,
  - nie przeczyly sobie.

To nie jest nowy dokument architektury.
To nie jest roadmap marketingowy.
To jest glowny dokument sterujacy pracami.

## 2. Main Rule

Nie rozwijamy nowych warstw, dopoki poprzednia warstwa nie ma:
- jasnych invariants,
- jasnej checklisty,
- minimalnych testow,
- i uczciwego opisu unresolved gaps.

Twarda zasada:
- najpierw spinamy semantics,
- potem kod,
- potem testy,
- potem nastepna warstwa.

## 3. Execution Order

Finalna kolejnosc prac jest nastepujaca:

1. Transport / P2P baseline
2. Auth / signing model
3. Chain / ledger / node baseline
4. Proof completion
5. Escrow final model
6. Marketplace / operator trust model

Kazda faza ma:
- osobne docs,
- osobne checklisty,
- osobne exit criteria.

## 4. Phase 1: Transport / P2P Baseline

Cel fazy:
- doprowadzic validator session transport do stanu, w ktorym:
  - dziala,
  - nie przecieka plaintextem,
  - nie startuje na placeholder keys,
  - nie daje prostych griefing/DoS footgunow.

### Docs in this phase

- `spec/PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`
- `spec/PRIVAI_VALIDATOR_SESSION_TEST_PLAN.md`
- `spec/PRIVAI_TRANSPORT_KEYS_AND_STARTUP_RULES.md`
- `spec/PRIVAI_TRANSPORT_REMAINING_BLOCKERS.md`

### Phase 1 checklist

- [ ] Incoming decrypt path jest naprawiony i opisany.
- [ ] Handshake freshness / transcript binding jest opisane zgodnie z kodem.
- [ ] No-plaintext-after-handshake jest twarda zasada.
- [ ] Transport nie startuje na placeholder / zero keys.
- [ ] Write timeouts po stronie serwera sa jawnie opisane.
- [ ] Regression pack session layer jest okreslony.
- [ ] Remaining blockers sa nazwane uczciwie:
  - ban poisoning
  - plaintext fallback
  - limiter semantics
  - cooldown / failed-handshake accounting
  - replay/order semantics per frame

### Phase 1 exit criteria

- wiadomo, jak wyglada final current session flow,
- wiadomo, co jest fixed,
- wiadomo, co zostalo,
- docs nie overclaimuja stanu kodu.

## 5. Phase 2: Auth / Signing Model

Cel fazy:
- naprawic i zamknac model autoryzacji tak, aby:
  - nie byl cykliczny,
  - wspieral threshold auth,
  - byl gotowy pod escrow.

### Docs in this phase

- `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
- `spec/PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`

### Phase 2 checklist

- [ ] `tx_id` i `tx_signing_hash` sa rozdzielone.
- [ ] Canonical signing preimage jest jawnie zdefiniowany.
- [ ] Signer ordering jest canonical.
- [ ] Duplicate signer material nie liczy sie wielokrotnie.
- [ ] Identity binding signer -> auth artifact jest jawny.
- [ ] Threshold auth package ma jawna semantyke.
- [ ] `nexum-core` vs ledger responsibilities sa jawnie rozdzielone.

### Phase 2 exit criteria

- auth nie jest logicznie cykliczne,
- multisig / threshold auth ma twardy fundament,
- escrow ma na czym stac.

## 6. Phase 3: Chain / Ledger / Node Baseline

Cel fazy:
- zamknac storage, recovery i ownership boundaries zanim proof i escrow zaczna polegac na miekkim runtime.

### Docs in this phase

- `spec/PRIVAI_CHAIN_STORAGE_INVARIANTS.md`
- `spec/PRIVAI_RECOVERY_AND_RESTART_RULES.md`
- `spec/PRIVAI_LAYER_OWNERSHIP_AND_BOUNDARIES.md`

### Phase 3 checklist

- [ ] Durable state set jest zdefiniowany.
- [ ] Rebuildable indexes sa zdefiniowane.
- [ ] Ephemeral state jest zdefiniowany.
- [ ] Source of truth dla:
  - `tip/head`
  - `finalized head`
  - `last persisted but not fully indexed block`
  jest jawny.
- [ ] Recovery / restart sequence jest jawna.
- [ ] Partial write / partial index loss behavior jest jawny.
- [ ] Node vs ledger vs transport vs proof ownership boundaries sa jawne.

### Phase 3 exit criteria

- storage i recovery nie sa juz domyslem,
- ledger/state maja twardy kontrakt,
- kolejne warstwy nie opieraja sie na niejawnych zalozeniach runtime.

## 7. Phase 4: Proof Completion

Cel fazy:
- nie wymyslac proof od nowa,
- tylko opisac i domknac to, co juz istnieje,
- oraz nazwac granice proof vs ledger.

### Docs in this phase

- `spec/PRIVAI_PROOF_COMPLETION_PLAN.md`
- `spec/PRIVAI_TRANSFER_NOTE_PROOF_SEMANTICS.md`
- `spec/PRIVAI_TX_CLASS_PROOF_MATRIX.md`

### Phase 4 checklist

- [ ] `Current canonical today` jest jawnie spisane.
- [ ] `Unresolved today` jest jawnie spisane.
- [ ] `TransferNoteTx` ma opisane:
  - statement
  - public inputs
  - witness
  - statement-to-witness consistency
  - execution bundle relation
  - ledger-vs-proof boundary
- [ ] Tx class matrix rozroznia:
  - proof-covered
  - ledger-only
  - experimental
- [ ] `ExecutionBundle` semantics sa opisane.
- [ ] `ProofCertificate` semantics sa opisane.

### Phase 4 exit criteria

- wiadomo, co proof juz dzis znaczy,
- wiadomo, czego jeszcze proof nie zamyka,
- escrow nie bedzie integrowane z niedookreslonym proof layer.

## 8. Phase 5: Escrow Final Model

Cel fazy:
- zbudowac finalny model escrow jako logiczna konsekwencje:
  - auth,
  - note modelu,
  - proof boundary,
  - ledger validation.

### Docs in this phase

- `spec/PRIVAI_ESCROW_FINAL_MODEL.md`
- `spec/PRIVAI_ESCROW_TX_MATRIX.md`
- `spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md`
- `spec/PRIVAI_PQ_AND_PRIVACY_CLAIMS.md`

### Phase 5 checklist

- [ ] Role escrow sa jawne:
  - buyer
  - merchant
  - operator
- [ ] Trust model jest jawny.
- [ ] Escrow jest opisane jako note-based value lock.
- [ ] `2-z-3` semantics sa jawnie opisane.
- [ ] Operator-required normal mode jest jawnie opisany.
- [ ] Buyer+merchant recovery path jest jawnie opisany.
- [ ] `nexum-core` vs `privAI` responsibilities sa jawne.
- [ ] Escrow action matrix jest jawna.
- [ ] Escrow -> proof mapping jest jawny.
- [ ] Escrow -> ledger-only checks sa jawne.
- [ ] PQ/privacy claims sa uczciwie odciete od overclaimow.

### Phase 5 exit criteria

- escrow nie jest juz tylko intuicja,
- escrow nie miesza sie z marketplace rail,
- escrow ma jasny auth / proof / ledger split.

## 9. Phase 6: Marketplace / Operator Trust Model

Cel fazy:
- dopiero po escrow zdefiniowac uczciwie, jaka jest rola operatora i co znaczy marketplace v0 vs final.

### Docs in this phase

- `spec/PRIVAI_MARKETPLACE_TRUST_MODEL.md`

### Phase 6 checklist

- [ ] Operator role jest opisana uczciwie.
- [ ] Seller consent jest opisana uczciwie.
- [ ] User consent jest opisana uczciwie.
- [ ] Marketplace orchestration vs custody jest rozdzielone.
- [ ] `v0` custodial claims sa odciete od final target.
- [ ] Final operator role jest zgodna z escrow model.

### Phase 6 exit criteria

- marketplace nie psuje escrow semantics,
- operator trust model jest nazwany bez ukrytych zalozen.

## 10. Global Dependency Rules

### 10.1. Escrow depends on transport
Nie implementujemy final escrow bez sensownego transport/session baseline.

### 10.2. Escrow depends on auth
Nie implementujemy final escrow bez `tx_signing_hash` i threshold auth rules.

### 10.3. Escrow depends on proof clarity
Nie spinamy escrow z proofem, dopoki proof layer nie ma jasno opisanych semantics.

### 10.4. Marketplace depends on escrow
Nie finalizujemy roli operatora marketplace, dopoki nie wiemy, jaki jest finalny escrow model.

## 11. Writing Rules For All Future Docs

Kazdy nowy doc w tej spine musi miec:
- `Status`
- `Canonicality`
- `Owner`
- `Depends on`
- `Cel`
- `Checklist`
- `Exit criteria`

Kazdy nowy doc musi:
- byc waski tematycznie,
- nie nadpisywac cudzej warstwy,
- nie udawac finalnosci, jesli temat jest unresolved,
- wyraznie odrozniac:
  - current canonical
  - unresolved
  - future target

## 12. What We Are Not Doing

- [ ] Nie piszemy szerokich nowych specow bez miejsca w tej spine.
- [ ] Nie skaczemy od razu do escrow bez auth i proof boundary.
- [ ] Nie mieszamy transport, auth, proof i marketplace w jednym dokumencie.
- [ ] Nie overclaimujemy PQ / privacy / custody semantics.
- [ ] Nie pozwalamy, zeby docs odjechaly od kodu.

## 13. Recommended Immediate Next Docs

Najblizsza kolejnosc pisania:

1. `spec/PRIVAI_TRANSPORT_KEYS_AND_STARTUP_RULES.md`
2. `spec/PRIVAI_TRANSPORT_REMAINING_BLOCKERS.md`
3. `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
4. `spec/PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`
5. `spec/PRIVAI_ESCROW_FINAL_MODEL.md`

## 14. Final Assessment

Ta execution spine ma utrzymac jedna wspolna logike projektu:

- najpierw domykamy to, co juz istnieje,
- potem spinamy auth,
- potem doprecyzowujemy ledger/runtime ownership,
- potem zamykamy proof semantics,
- dopiero wtedy budujemy final escrow,
- a marketplace role opisujemy na koncu, bez mieszania v0 i final target.

To jest glowny tor, ktorym dalej idziemy.
