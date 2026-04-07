# privAI Proof Completion Plan

Status: focused completion roadmap for proof plane semantics, coverage rules and validation boundaries.
Canonicality: supporting architecture and execution plan. This document does not override canonical protocol, formats, consensus or product semantics; it records the agreed direction for finishing the proof plane without allowing local guesswork.
Owner: privAI proof, ledger and consensus architecture.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_PROOF_BOUNDARIES.md`
- `spec/PRIVAI_REFERENCE_VECTORS.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zapisac, co dokladnie trzeba domknac w proof plane,
- odciac dalsze zgadywanie relacji:
  - `statement_commit`
  - `public_inputs`
  - `ExecutionBundle`
  - `ProofCertificate`
  - block validation,
- ustalic kolejnosc prac,
- zapisac czego nie wolno teraz udawac jako finalnego.

To nie jest dokument o kryptografii od zera.
To nie jest tez dokument o consensus jako calosci.
To jest dokument o domknieciu proof-carrying execution dla `privAI`.

## 2. Current Canonical Today

Aktualny stan kanoniczny dzis:
- `TransferNoteTx` jest jedyna obecna kanoniczna proof-covered tx class.
- `ExecutionBundle` ma current bytes i current runtime role.
- `ProofCertificate` ma current bytes i current runtime role.
- `MarketplaceBatchTx` nie jest traktowany jako proof-equivalent do `TransferNoteTx`.
- `OnChainLite` pozostaje `experimental`.
- samo istnienie current bytes nie oznacza jeszcze finalnego domkniecia semantyki.

## 3. Unresolved Today

Niezamkniete dzis:
- finalna pelna semantyka `ExecutionBundle`,
- finalna pelna semantyka `ProofCertificate`,
- finalna per-class odpowiedz dla `public_inputs_hash_for_transaction()`,
- finalny proof contract dla `OnChainLite`,
- finalna explicit boundary:
  - co waliduje proof,
  - co nadal waliduje ledger niezaleznie od proof.

## 4. Current Direction

Aktualny kierunek pozostaje:
- proof plane ma byc naprawde domkniety najpierw dla `FullPrivacy`,
- `TransferNoteTx` pozostaje glowna proof-covered tx class,
- `MarketplaceBatchTx` nie jest wciskany na sile w ten sam proof model co `FullPrivacy`,
- `OnChainLite` pozostaje `experimental` dopoki nie ma finalnego modelu proof coverage i pelnego spiecia z execution/state commitments,
- najpierw domykamy semantics i boundaries, potem vectors i testy.

## 5. What Proof Plane Is Supposed To Do

Docelowo proof plane ma:
- wiazac execution z `statement_commit`,
- wiazac public inputs z tym, co ledger i consensus widza publicznie,
- egzekwowac coverage dla tx classes oznaczonych jako proof-required,
- dawac jednoznaczna relacje:
  - tx class
  - statement
  - public inputs
  - bundle
  - proof certificate
  - block roots

Proof plane nie ma:
- udawac auth layer tam, gdzie auth jest jeszcze ledger-side,
- na sile obejmowac wszystkie raile jednym modelem,
- zastępowac ledger validation tam, gdzie rail jest jawnie public/ledger-validated,
- zastępowac transaction shape validation.

## 6. Main Proof Layers To Complete

### 6.1. Statement layer

Do domkniecia:
- co dokladnie statement twierdzi,
- jaki witness odpowiada statementowi,
- jakie sa public inputs,
- jak liczony jest `statement_commit`,
- czy `statement_commit` jest per-tx required czy class-dependent.

### 6.2. Execution bundle layer

Do domkniecia:
- czym semantycznie jest `ExecutionBundle`,
- co oznacza `covered_tx_indexes`,
- co znaczy complete proof coverage,
- jak liczy sie `statement_root`,
- jak liczy sie `public_inputs_root`,
- jaka jest relacja bundle do block validation.

### 6.3. Proof certificate layer

Do domkniecia:
- czym semantycznie jest `ProofCertificate`,
- co certyfikuje:
  - bundle
  - statement root
  - public inputs root
  - proof artifact hash
- jak `ProofCertificate` wchodzi do block roots,
- jak consensus interpretuje brak albo mismatch proof certificates.

## 7. Tx Class Matrix To Complete

### 7.1. TransferNoteTx

Status:
- `current canonical proof-covered tx class`

Do zamkniecia:
- finalny `statement_commit`
- finalny `public_inputs_hash`
- finalny proof coverage rule
- finalna relacja witness -> statement -> public inputs

### 7.2. MarketplaceBatchTx

Status:
- `ledger/auth/nullifier/batch validated rail`

Current canonical interpretation:
- nie traktowac tej klasy jako proof-equivalent do `TransferNoteTx`,
- nie dopowiadac ZK coverage, jesli go finalnie nie ma,
- proof plane nie moze zakladac, ze marketplace rail dziedziczy full-privacy proof semantics.

Do zamkniecia:
- czy batch ma jakiekolwiek proof-related artifact binding w finalnym modelu,
- czy pozostaje wylacznie ledger/auth/nullifier/batch-check validated.

### 7.3. OnChainLite

Status:
- `experimental`

Current canonical interpretation:
- nie wolno uznac tego raila za finalny bez:
  - jednoznacznego modelu proof coverage albo jawnego finalnego braku proof coverage,
  - pelnego spiecia z `note_root`,
  - pelnego spiecia z `state_root`,
  - pelnego spiecia z `ExecutionBundle`,
  - finalnej consensus rule dla threshold / acceptance.

## 8. What Must Be Completed Next

### 8.1. Public inputs rule

Trzeba zamknac dla kazdej proof-relevant tx class jedna z trzech odpowiedzi:
- `final proof-covered rule`
- `not proof-covered`
- `experimental / unresolved`

Nie kazda klasa musi dostac finalny `public_inputs_hash`, jesli finalnie nie jest proof-covered.

### 8.2. Statement commit rule

Trzeba zamknac:
- czy wszystkie proof-required tx maja `statement_commit`,
- co jest preimage statement commit,
- jak statement commit wchodzi do coverage i rootów.

### 8.3. Proof coverage rule

Trzeba zamknac:
- ktore tx classes sa proof-required,
- czy brak coverage dla proof-required tx = reject,
- czy partial coverage jest dopuszczalne,
- czy coverage liczy sie per statement, per tx class, czy per execution mode.

### 8.4. Execution bundle semantics

Trzeba zamknac:
- meaning of `covered_tx_indexes`,
- meaning of `execution_mode`,
- one relation:
  - covered statements
  - `statement_root`
  - `public_inputs_root`
  - proof artifacts.

### 8.5. Proof certificate semantics

Trzeba zamknac:
- dokladnie co hashujemy,
- co certyfikat potwierdza,
- jak consensus sprawdza spojnosc bundle/certificate/root.

### 8.6. Ledger-vs-proof validation boundary

Trzeba jawnie zamknac dla kazdej tx class:
- ktore warunki pozostaja ledger-validated niezaleznie od proof,
- czego proof nie zastępuje,
- ktore checks nie moga byc poluzowane tylko dlatego, ze istnieje proof artifact.

## 9. Current Gaps To Remove

Najwazniejsze obecne luki:
- `public_inputs_hash_for_transaction()` nie daje jeszcze finalnej odpowiedzi dla wszystkich proof-relevant klas,
- batch builder / proof coverage / ledger validation nie sa jeszcze w 100% zsynchronizowane,
- `OnChainLite` nie jest jeszcze finalnie podpiety do proof plane,
- semantics `ExecutionBundle` i `ProofCertificate` wymagaja domkniecia jako semantyka, nie tylko bytes.

## 10. What We Are Not Doing Now

- [ ] Nie probujemy teraz robic proof-unification dla wszystkich raili.
- [ ] Nie wciskamy `MarketplaceBatchTx` do `FullPrivacy` proof modelu bez jawnej decyzji.
- [ ] Nie uznajemy `OnChainLite` za finalny.
- [ ] Nie probujemy teraz ukrywac threshold auth escrow w proof systemie.
- [ ] Nie zamieniamy ledger validation na "proof zrobi wszystko".

## 11. Checklist: Proof Completion

### 11.1. Statement layer

- [ ] Spisac final statement model dla `TransferNoteTx`.
- [ ] Spisac witness fields required by statement.
- [ ] Spisac final public inputs for `TransferNoteTx`.
- [ ] Spisac exact relation `statement_commit <-> witness <-> public inputs`.

### 11.2. Tx class coverage matrix

- [ ] Zamknac matrix `tx class -> proof-covered / not proof-covered / experimental`.
- [ ] Zamknac `TransferNoteTx`.
- [ ] Zamknac `MarketplaceBatchTx`.
- [ ] Zamknac status `OnChainLite` jako `experimental` z explicit boundaries.

### 11.3. Execution bundle

- [ ] Zamknac semantics `covered_tx_indexes`.
- [ ] Zamknac semantics `execution_mode`.
- [ ] Zamknac `statement_root`.
- [ ] Zamknac `public_inputs_root`.
- [ ] Zamknac relation `ExecutionBundle -> block validation`.

### 11.4. Proof certificate

- [ ] Zamknac semantics `ProofCertificate`.
- [ ] Zamknac relation certificate -> bundle.
- [ ] Zamknac relation certificate -> block roots.
- [ ] Zamknac reject rules for missing or mismatched certificates.

### 11.5. Ledger-vs-proof boundary

- [ ] Freeze explicit ledger-vs-proof validation boundary for each tx class.
- [ ] Zamknac which checks remain ledger-side for `TransferNoteTx`.
- [ ] Zamknac which checks remain ledger-side for `MarketplaceBatchTx`.
- [ ] Zamknac which checks remain unresolved/experimental for `OnChainLite`.

## 12. Checklist: Tests

- [ ] `transfer_note_public_inputs_hash_matches_reference`
- [ ] `transfer_note_statement_commit_matches_reference`
- [ ] `missing_proof_for_proof_required_tx_rejected`
- [ ] `partial_coverage_for_proof_required_tx_rejected_or_explicitly_allowed`
- [ ] `execution_bundle_statement_root_matches_reference`
- [ ] `execution_bundle_public_inputs_root_matches_reference`
- [ ] `proof_certificate_hash_matches_reference`
- [ ] `proof_certificate_root_matches_reference`
- [ ] `bundle_certificate_mismatch_rejected`
- [ ] `proof_required_tx_outside_coverage_rejected`

## 13. Checklist: Reference Vectors

- [ ] Final vectors for `TransferNoteTx` proof-related fields.
- [ ] Final vectors for `ExecutionBundle`.
- [ ] Final vectors for `ProofCertificate`.
- [ ] Final vectors for block roots that depend on proof artifacts.
- [ ] Explicit exclusion or `experimental` marking for `OnChainLite` vectors.

## 14. Recommended Execution Order

1. Zamknac `tx class -> proof semantics` matrix
2. Zamknac `TransferNoteTx` statement + public inputs
3. Zamknac `ExecutionBundle` semantics
4. Zamknac `ProofCertificate` semantics
5. Zamknac `ledger-vs-proof boundary`
6. Dodac vectors
7. Dodac tests
8. Dopiero potem wracac do `OnChainLite`

## 15. Practical Next Step For The Team

Pierwszym nastepnym dokumentem powinien byc:
- `spec/PRIVAI_TRANSFER_NOTE_PROOF_SEMANTICS.md`

Alternatywa dopuszczalna:
- dopisanie zamknietej sekcji `TransferNoteTx Proof Semantics` do istniejacego proof doc.

Najbardziej praktyczny ruch:
- najpierw domknac `TransferNoteTx`
- nie zaczynac od `OnChainLite`
- nie zaczynac od marketplace raila

## 16. Final Assessment

Tak, ten kierunek jest spojny z reszta docs.

Powod:
- nie otwiera nowej architektury poza proof plane,
- nie miesza proof semantics z auth/storage/network,
- nie rozmywa canonical set,
- pozwala domykac proof warstwa po warstwie,
- zostawia `MarketplaceBatchTx` i `OnChainLite` we wlasciwym statusie zamiast udawac finalnosc.
