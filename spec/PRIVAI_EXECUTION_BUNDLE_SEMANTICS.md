# privAI ExecutionBundle Semantics

Status: focused support doc for current `ExecutionBundle` semantics.
Canonicality: anti-drift semantics doc for the current `ExecutionBundle` bytes and runtime role. This document does not override canonical protocol, formats, consensus or product semantics; it records the current meaning of `ExecutionBundle` in code and names what is still unresolved.
Owner: privAI proof, ledger and consensus architecture.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_PROOF_BOUNDARIES.md`
- `spec/PRIVAI_PROOF_COMPLETION_PLAN.md`
- `spec/PRIVAI_TRANSFER_NOTE_PROOF_SEMANTICS.md`
- `spec/PRIVAI_REFERENCE_VECTORS.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zapisac current bytes i current runtime role `ExecutionBundle`,
- odciac lokalne zgadywanie semantyki:
  - `statement_commits`
  - `covered_tx_indexes`
  - `public_inputs_root`
  - `execution_mode`,
- zapisac, co dzis jest `Current canonical`,
- nazwac, co pozostaje `Unresolved`.

To nie jest finalny freeze wszystkich raili.
To nie jest dokument o `ProofCertificate`.
To jest waski support doc tylko dla current `ExecutionBundle`.

## 2. Current Canonical Today

Aktualny stan kanoniczny dzis:
- `ExecutionBundle` ma current canonical bytes.
- `ExecutionBundle` ma current canonical reference vector.
- `ExecutionBundle` jest current canonical czescia block body.
- `ExecutionBundle` ma current runtime role w proof plane.
- samo istnienie canonical bytes nie oznacza jeszcze finalnie domknietej semantyki wszystkich raili i wszystkich execution modes.

Current canonical fields:
1. `statement_commits: Vec<Hash32>`
2. `covered_tx_indexes: Vec<u32>`
3. `public_inputs_root: Hash32`
4. `execution_mode: ExecutionMode`

Current canonical execution modes:
- `FullBatchProof = 0x01`
- `MultiProofBundle = 0x02`
- `Housekeeping = 0x03`

## 3. Current Bytes Rule

Current canonical encoding `ExecutionBundle`:
1. `statement_commits: vec<Hash32>`
2. `covered_tx_indexes: vec<u32_le>`
3. `public_inputs_root: [u8; 32]`
4. `execution_mode: u8`

Current canonical fact:
- minimalny frozen vector dla pustego `ExecutionBundle` w trybie `Housekeeping` jest zapisany w `spec/PRIVAI_REFERENCE_VECTORS.md`.

Current canonical interpretation:
- current bytes sa frozen,
- current bytes nie przesadzaja same przez sie o finalnej polityce proof coverage dla wszystkich tx classes.

## 4. Current Runtime Construction Rule

Current runtime builders:
- `build_execution_bundle_from_transactions()`
- `build_execution_bundle_from_transfer_proofs()`

Current canonical runtime behavior `build_execution_bundle_from_transactions()`:
- jesli `txs` jest puste:
  - `statement_commits = []`
  - `covered_tx_indexes = []`
  - `public_inputs_root = merkle_root(empty)`
  - `execution_mode = Housekeeping`
- jesli batch zawiera tx uznane przez current code za proof-requiring:
  - bundle zbiera ich `statement_commit`
  - bundle zbiera ich indeksy w oryginalnym batchu tx
  - bundle liczy `public_inputs_root` z `public_inputs_hash`
- jesli po filtracji nie ma proof-requiring tx:
  - `execution_mode` jest wymuszane na `Housekeeping`

Current canonical runtime behavior `build_execution_bundle_from_transfer_proofs()`:
- builder wymaga, aby dla kazdego proving data:
  - `public_inputs.statement_commit == statement.commitment()`
- `covered_tx_indexes` sa ustawiane kolejno jako `0..n-1`
- `public_inputs_root` jest liczone jako `merkle_root(public_inputs_hashes)`
- `execution_mode` pozostaje tym, co podano na wejsciu

## 5. Current Meaning Of Fields

### 5.1. `statement_commits`

Current canonical interpretation:
- to lista statement commitments tx objetych current proof path,
- current code wypelnia ja z `tx.statement_commit()` albo z proving data statement commitments,
- kolejnosc odpowiada kolejnosci covered elements w danym builderze.

### 5.2. `covered_tx_indexes`

Current canonical interpretation:
- to indeksy tx w oryginalnym batchu, ktore current builder uznal za proof-covered dla danego `ExecutionBundle`,
- w transaction-based builderze sa to indeksy z oryginalnej listy tx,
- w proof-based builderze sa to kolejne indeksy lokalnej listy proving data.

Wazna zasada:
- current bytes `covered_tx_indexes` sa frozen,
- finalna pelna semantyka coverage dla wszystkich raili nie jest jeszcze przez to sama zamknieta.

### 5.3. `public_inputs_root`

Current canonical interpretation:
- to merkle root z current public inputs hashes tx objetych bundlem,
- w current code jest liczony z listy `public_inputs_hash_for_transaction()` albo z proving data public inputs hashes,
- dla pustej listy jest rowny `merkle_root(empty)`.

### 5.4. `execution_mode`

Current canonical interpretation:
- to execution-plane mode zapisany w bundle,
- `Housekeeping` oznacza blok bez current proof workload,
- current transaction-based builder wymusza `Housekeeping`, jesli po filtracji nie ma covered tx,
- current proof-based builder nie nadpisuje wejsciowego mode poza przypadkiem pustego inputu.

## 6. Current Relation To TransferNoteTx

Current canonical relation:
- `TransferNoteTx` jest obecnie glowna proof-covered tx class podlaczona do `ExecutionBundle`,
- dla `Transaction::TransferNote`:
  - istnieje current `public_inputs_hash_for_transaction()`
  - `statement_commit` wchodzi do bundle
  - `public_inputs_hash` wchodzi do `public_inputs_root`

Current canonical interpretation:
- dzisiejsza semantyka `ExecutionBundle` jest najsilniej domknieta wlasnie dla `TransferNoteTx`,
- nie wolno z tego dopowiadac, ze wszystkie inne raile maja juz rownie finalna relacje do bundle.

## 7. Current Non-Conformities That Matter

Najwazniejsze obecne luki:
- current transaction-based builder traktuje `LiteTransfer` jako proof-requiring,
  ale `public_inputs_hash_for_transaction()` nie ma dla `LiteTransfer` finalnie domknietej odpowiedzi tak jak dla `TransferNoteTx`,
- current final semantics `covered_tx_indexes` nie jest jeszcze zamknieta dla wszystkich raili,
- current relation `ExecutionBundle -> block validation` nie jest jeszcze domknieta jako pelna finalna semantyka dla wszystkich execution modes,
- current relation `ExecutionBundle -> ProofCertificate` nadal wymaga osobnego domkniecia.

## 8. Unresolved Today

Nadal niezamkniete:
- finalny globalny meaning proof coverage across all tx classes,
- finalna polityka partial coverage vs reject,
- finalna semantyka `MultiProofBundle`,
- finalna relacja `ExecutionBundle -> ProofCertificate`,
- finalna relacja `ExecutionBundle -> block acceptance`,
- finalna odpowiedz, jak `OnChainLite` ma byc odzwierciedlany w bundle semantics.

## 9. What Must Not Be Inferred From This Doc

Nie wolno z tego dokumentu wyprowadzac, ze:
- wszystkie tx classes sa juz proof-covered albo proof-classified finalnie,
- `covered_tx_indexes` samo przez sie daje finalna polityke reject rules,
- current bytes `ExecutionBundle` oznaczaja juz finalna semantyke wszystkich execution modes,
- current runtime role `ExecutionBundle` zastepuje osobne domkniecie `ProofCertificate`.

## 10. How To Use This Doc

Jesli task dotyczy `ExecutionBundle`:
- najpierw przeczytaj ten dokument,
- potem `spec/PRIVAI_PROOF_BOUNDARIES.md`,
- potem `spec/PRIVAI_PROOF_COMPLETION_PLAN.md`,
- potem `spec/PRIVAI_TRANSFER_NOTE_PROOF_SEMANTICS.md`,
- dopiero potem zejdz do:
  - `spec/PRIVAI_CONSENSUS.md`
  - `privai-proof/src/batch.rs`
  - `privai-chain/src/consensus.rs`

Jesli task dotyczy reject rules albo certificate semantics:
- nie dopowiadaj ich z samego `ExecutionBundle`,
- poczekaj na osobne domkniecie `ProofCertificate` semantics.
