# privAI Escrow Proof Integration

Status: focused support doc for escrow-to-proof integration boundaries in privAI.
Canonicality: supporting escrow-proof boundary document. This document does not override canonical protocol, formats, consensus, proof or product semantics; it records the current and target relationship between escrow actions and the proof plane, names what is proof-covered, what is ledger-only, and what remains unresolved. This document does not invent a new proof system.
Owner: privAI proof, escrow, ledger and consensus architecture.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_EXECUTION_SPINE.md`
- `spec/PRIVAI_ESCROW_FINAL_MODEL.md`
- `spec/PRIVAI_ESCROW_TX_MATRIX.md`
- `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
- `spec/PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`
- `spec/PRIVAI_PROOF_BOUNDARIES.md`
- `spec/PRIVAI_PROOF_COMPLETION_PLAN.md`
- `spec/PRIVAI_TRANSFER_NOTE_PROOF_SEMANTICS.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_CONSENSUS.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- spiąc escrow z istniejacym proof modelem, a nie wymyslac proof od nowa,
- dla kazdej escrow action opisac:
  - co proof moze juz coverowac,
  - czego proof nie coveruje,
  - co zostaje ledger-side,
- zapisac uczciwa granice miedzy proof-covered, ledger-only i mixed,
- odciac overclaiming o full PQ privacy dla escrow,
- dac baze pod przyszly proof-aware escrow bez blokowania v1.

Ten dokument nie jest:
- nowym proof systemem,
- binary format spec,
- implementacja,
- dokumentem marketplace rail.

## 2. Current Direction

Escrow w privAI jest na railu `FullPrivacy` i naturalnie dziedziczy proof path `TransferNoteTx`.

Ale:
- escrow spend ma dodatkowa semantyke, ktorej obecny proof path `TransferNoteTx` nie pokrywa:
  - threshold auth (2-of-3),
  - policy reconstruction z `policy_opening`,
  - action type binding,
  - operator presence check (normal mode),
  - recovery precondition (timeout),
- dlatego v1 escrow jest **mixed**: czesciowo proof-covered (note/spend semantics), czesciowo ledger-only (auth/policy/action/mode semantics).

Kierunek:
- v1: ledger-enforced threshold auth + istniejacy proof path dla note semantics,
- future: proof-aware threshold auth, ukrywajacy role/quorum/policy przed publicznym obserwatorem.

## 3. Relationship to Existing Proof Model

### 3.1. TransferNoteTx proof path

Current canonical proof path (`PRIVAI_TRANSFER_NOTE_PROOF_SEMANTICS.md`):
- `TransferStatement`: input note commits, input nullifiers, output note commits, fee,
- `TransferPublicInputs`: tx_id, statement_commit, input note commits, input nullifiers, output note commits, fee,
- `TransferWitness`: amounts, seeds, nullifier keys, spend policy openings, aux openings, output openings,
- `statement_commit = H_transfer_statement_v0(canonical(TransferStatement))`,
- `public_inputs_hash = H_transfer_public_inputs_v0(canonical(TransferPublicInputs))`.

### 3.2. Co escrow dziedziczy

Escrow spend (release, refund, recovery) jest `TransferNoteTx` na poziomie tx class — wydaje noty i tworzy nowe noty. Dlatego:
- note commitment consistency (input commit vs witness),
- nullifier derivation correctness,
- output commitment correctness,
- balance proof (sum of inputs >= sum of outputs + fee),
- statement/public inputs binding

sa juz pokryte przez istniejacy proof path, jesli escrow spend jest realizowany jako `TransferNoteTx`.

### 3.3. Czego escrow NIE dziedziczy z obecnego proof path

Obecny proof path `TransferNoteTx` nie pokrywa:
- threshold auth semantics (2-of-3 quorum),
- policy reconstruction z `policy_opening`,
- action type binding (`release` / `refund` / `recovery_release`),
- signer identity / membership validation,
- operator presence check,
- recovery precondition (timeout),
- destination constraint enforcement (srodki do Merchanta vs Buyera).

To wszystko jest v1 ledger-only.

## 4. Escrow Action -> Proof/Ledger Split

### 4.1. EscrowFund

| Aspect | Coverage |
|--------|----------|
| Classification | **mixed with minimal ledger add-on** (standard FullPrivacy spend + policy family validation) |
| Proof covers | note commitment, nullifier, balance, statement/public inputs — standardowy TransferNoteTx proof path |
| Ledger covers | policy family/version validation na output note, standard state transition |
| Unresolved | brak — funding jest standardowym FullPrivacy spend z dodatkowym ledger check |

EscrowFund jest standardowym `TransferNoteTx` z jednym dodatkowym ledger-side check: walidacja policy family/version na output note. Proof path pokrywa note semantics, ale ledger musi dodatkowo zweryfikowac, ze output note odnosi sie do obslugiwanego typu policy.

### 4.2. ReleaseToMerchant

| Aspect | Coverage |
|--------|----------|
| Classification | **mixed** (proof + ledger) |
| Proof covers | note commitment consistency, nullifier derivation, output commitment, balance proof, statement/public inputs binding |
| Ledger covers | `policy_opening` match, policy reconstruction, signer membership, Falcon sig verification, quorum 2-of-3, operator presence, action type validation, output destination validation, canonical signer ordering, duplicate signer rejection |
| Unresolved | czy threshold auth moze byc przeniesione do proof w przyszlosci |

### 4.3. RefundToBuyer

| Aspect | Coverage |
|--------|----------|
| Classification | **mixed** (proof + ledger) |
| Proof covers | identycznie jak ReleaseToMerchant |
| Ledger covers | identycznie jak ReleaseToMerchant, z ta roznica ze output destination to Buyer |
| Unresolved | identycznie jak ReleaseToMerchant |

### 4.4. RecoveryRelease

| Aspect | Coverage |
|--------|----------|
| Classification | **mixed** (proof + ledger) |
| Proof covers | identycznie jak ReleaseToMerchant |
| Ledger covers | identycznie jak ReleaseToMerchant, z ta roznica ze: operator presence NIE jest wymagany, recovery precondition (block height >= `timeout_block`) jest dodatkowym ledger check |
| Unresolved | identycznie jak ReleaseToMerchant + czy timeout moze byc przeniesiony do proof |

## 5. Consolidated Proof/Ledger Boundary Matrix

| Check | Proof-covered | Ledger-only | Notes |
|-------|---------------|-------------|-------|
| Note commitment consistency | yes | — | standard TransferNoteTx proof |
| Nullifier derivation | yes | — | standard TransferNoteTx proof |
| Output commitment correctness | yes | — | standard TransferNoteTx proof |
| Balance proof (inputs >= outputs + fee) | yes | — | standard TransferNoteTx proof |
| Statement/public inputs binding | yes | — | standard TransferNoteTx proof |
| `policy_opening` match | — | yes | ledger reconstructs policy |
| Policy reconstruction | — | yes | ledger derives signer set, threshold, roles |
| Falcon signature verification | — | yes | ledger verifies each signer |
| Signer membership | — | yes | ledger checks against policy signer set |
| Quorum check (2-of-3) | — | yes | ledger counts valid unique signers |
| Operator presence | — | yes | ledger enforces normal mode |
| Action type validation | — | yes | ledger checks action bound in tx_signing_hash |
| Output destination validation | — | yes | ledger checks target per action |
| Canonical signer ordering | — | yes | ledger rejects misordered |
| Duplicate signer rejection | — | yes | ledger rejects duplicates |
| Recovery precondition (timeout) | — | yes | ledger checks block height >= timeout_block |
| Fee validation | — | yes | ledger checks fee rules |

## 6. Relationship to Statement Semantics

Current `TransferStatement` juz binduje:
- input note commits,
- input nullifiers,
- output note commits,
- fee.

To jest wystarczajace dla note-level proof coverage escrow.

Czego current statement NIE binduje i co pozostaje ledger-side:
- action type,
- policy tag,
- signer identities,
- threshold rule,
- operator role,
- timeout.

Future target:
- extended statement model, ktory binduje action type i/lub policy-relevant fields, moze byc wprowadzony w przyszlosci,
- ale nie jest wymagany dla v1 escrow.

## 7. Relationship to Public Inputs

Current `TransferPublicInputs` zawiera:
- `tx_id`, `statement_commit`, input note commits, input nullifiers, output note commits, fee.

Dla v1 escrow to jest wystarczajace — public inputs binduja note-level facts.

Auth/policy/action level nie jest czescia current public inputs i pozostaje ledger-verified.

Future target:
- rozszerzenie public inputs o action type hash lub policy-relevant fields moze pozniej umozliwic proof-aware action binding,
- ale to nie jest wymagane dla v1.

## 8. Relationship to `tx_signing_hash`

`tx_signing_hash` binduje action type, inputs, outputs, fee, policy-relevant fields.

Ale `tx_signing_hash` jest auth-layer artifact, nie proof-layer artifact:
- proof path binduje statement i public inputs,
- auth path binduje `tx_signing_hash`,
- te dwie sciezki sa komplementarne, nie zamienne.
- oba sa obliczane w Stage B (wallet/prover/final assembly), nie w control-plane.

V1 escrow polega na obu:
- proof gwarantuje note-level correctness,
- auth (nad `tx_signing_hash`) gwarantuje action-level authorization.

## 9. Relationship to `policy_opening`

`policy_opening` jest auth-layer artifact:
- ledger uzywa go do rekonstrukcji policy i signer set,
- current proof path konsumuje `spend_policy_opening` jako czesc `TransferInputWitness`, ale nie egzekwuje escrow-specific policy semantics (threshold rule, signer roles, action constraints) — to pozostaje ledger-side.

Rozroznienie:
- proof path widzi `spend_policy_opening` jako witness field i moze weryfikowac commitment consistency,
- ale escrow policy semantics (2-of-3, operator presence, action binding) sa egzekwowane przez ledger, nie przez proof circuit.

Future target:
- proof-aware policy verification moglaby przenosic pelna policy reconstruction i threshold verification do proof circuit,
- ukrywajac signer identities i threshold rule przed publicznym obserwatorem,
- ale v1 tego nie wymaga.

## 10. Relationship to ExecutionBundle

Current `ExecutionBundle` role:
- `build_execution_bundle_from_transactions()` dodaje `TransferNoteTx` do:
  - `statement_commits`, `covered_tx_indexes`, `public_inputs_root`.

Escrow spend (jako `TransferNoteTx`) wchodzi do `ExecutionBundle` standardowa sciezka:
- escrow spend jest proof-relevant participant bundla,
- `statement_commit` escrow spend jest w `statement_commits`,
- `public_inputs_hash` escrow spend jest w `public_inputs_root`.

Dodatkowe escrow-specific auth/policy/action checks NIE wchodza do `ExecutionBundle` — pozostaja ledger-side.

Caveat: to jest current runtime/canonical today path. Pelna finalna semantyka `ExecutionBundle` nie jest jeszcze w 100% zamrozona — patrz `PRIVAI_PROOF_COMPLETION_PLAN.md` sekcja 6.2.

## 11. Relationship to ProofCertificate

Current `ProofCertificate` role:
- certyfikuje bundle: statement root, public inputs root, proof artifact hash.

Escrow spend (jako `TransferNoteTx`) jest objety certyfikatem standardowa sciezka:
- proof certificate nie widzi escrow-specific auth/policy/action semantics,
- proof certificate potwierdza note-level proof coverage.

To jest wystarczajace dla v1.

Caveat: to jest current runtime/canonical today path. Pelna finalna semantyka `ProofCertificate` nie jest jeszcze w 100% zamrozona — patrz `PRIVAI_PROOF_COMPLETION_PLAN.md` sekcja 6.3.

## 12. Current Canonical Today

- Escrow spend jest `TransferNoteTx` i dziedziczy istniejacy proof path.
- Note-level proof coverage (commitments, nullifiers, balance) jest juz current canonical.
- Auth/policy/action level jest ledger-only i nie jest czescia current proof plane.
- `ExecutionBundle` i `ProofCertificate` obejmuja escrow spend standardowa sciezka.

## 13. Unresolved Today

- Czy threshold auth semantics moze byc przeniesiony do proof circuit w przyszlosci (proof-aware threshold auth).
- Czy action type binding moze byc przeniesiony do statement/public inputs.
- Czy policy reconstruction moze byc ukryta w proof (zero-knowledge policy verification).
- Czy signer identities moga byc ukryte przed publicznym obserwatorem.
- Czy recovery precondition (timeout) moze byc przeniesiony do proof.
- Finalny end-to-end privacy model dla escrow (co widzi publiczny obserwator vs co widzi tylko weryfikator).

Twarda zasada:
- dopoki te pytania nie sa zamkniete, nie wolno claimowac full PQ privacy dla escrow,
- dopoki te pytania nie sa zamkniete, v1 escrow dziala z ledger-enforced public validation + note-level proof coverage.

## 13a. Stage B: Final Assembly Elements Required for Hash/Proof Path

Do obliczenia finalnego `tx_signing_hash` i pokrycia proof path potrzebne sa elementy Stage B (wallet/prover/final assembly), ktore NIE sa dostepne na etapie control-plane (Stage A):

| Element | Stage | Owner |
|---------|-------|-------|
| final nullifier (derivation z note_commit + nullifier_key) | B | wallet/prover |
| final `statement_commit` (over TransferStatement) | B | wallet/prover |
| final output note (`OutputNote`) | B | wallet/prover |
| final `RecipientBox` (sealing) | B | wallet/prover |
| final auth insertion ordering (canonical signer ordering in tx) | B | wallet/prover |
| final `tx_signing_hash` (from canonical tx body) | B | wallet/prover |
| final `TransferNoteTx` assembly | B | wallet/prover |

Te elementy naleza do Stage B — orchestrator / control-plane (Stage A) ich nie produkuje i nie oblicza nad nimi `tx_signing_hash`.

`tx_signing_hash` jest obliczany dopiero po skompletowaniu WSZYSTKICH powyzszych elementow. Zadna komorka control-plane nie ma dostepu do finalnych danych potrzebnych do tego obliczenia.

Relationship to proof:
- proof path (TransferStatement, public_inputs_hash, proof artifact) jest budowany w Stage B,
- proof wymaga finalnych outputow, nullifierow i statement_commit,
- te dane sa kompletne dopiero po final assembly,
- dlatego proof integration jest inherentnie Stage B.

## 14. What We Are Not Doing Now

- Nie przenosimy threshold auth do proof circuit w v1.
- Nie ukrywamy signer identities w proof w v1.
- Nie wymyslamy nowego proof systemu dla escrow.
- Nie mieszamy marketplace rail proof semantics z escrow proof semantics.
- Nie claimujemy full PQ privacy dla escrow bez audytu.
- Nie blokujemy v1 escrow oczekiwaniem na proof-aware auth.

## 15. Future Proof-Aware Escrow — Direction Only

Docelowy proof-aware escrow moze:
- przenosic policy reconstruction i threshold verification do proof circuit,
- ukrywac signer identities i role assignments przed publicznym obserwatorem,
- bindzic action type w statement lub public inputs,
- zachowywac pelna weryfikowalnosc w warstwie weryfikatora,
- dawac prawdziwa privacy dla escrow flows.

To wymaga:
- rozszerzenia statement model o policy/action fields,
- rozszerzenia public inputs o action-relevant commitments,
- nowego circuit dla threshold auth verification,
- audytu bezpieczenstwa.

To jest future target, nie v1.

## 16. Checklist

- [ ] Potwierdzic, ze escrow spend jako `TransferNoteTx` przechodzi istniejacy proof path.
- [ ] Potwierdzic, ze `ExecutionBundle` poprawnie obejmuje escrow spend.
- [ ] Potwierdzic, ze `ProofCertificate` poprawnie certyfikuje bundle z escrow spend.
- [ ] Zweryfikowac, ze ledger-only checks (auth/policy/action) sa kompletne i nie polegaja na proof.
- [ ] Dodac regression test: escrow spend z poprawnym proof ale blednym auth -> reject.
- [ ] Dodac regression test: escrow spend z poprawnym auth ale blednym proof -> reject.
- [ ] Dodac regression test: escrow spend z poprawnym proof i auth ale bledna policy -> reject.
- [ ] Zdefiniowac scope future proof-aware escrow jako osobny research/architecture task.

## 17. Exit Criteria

Faza escrow proof integration jest domknieta, gdy:
- kazda escrow action ma jednoznaczna klasyfikacje: proof-covered, ledger-only, lub mixed,
- granica proof vs ledger jest jawna i nie ma unresolved overlaps,
- istniejacy proof path (`TransferNoteTx`) obejmuje escrow spend na note level,
- auth/policy/action level jest jawnie ledger-only w v1,
- v1 escrow nie jest blokowany przez brak proof-aware auth,
- future proof-aware escrow ma jawny scope pointer bez overclaiming.
