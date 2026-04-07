# privAI TransferNoteTx Proof Semantics

Status: focused support doc for current `TransferNoteTx` proof semantics.
Canonicality: anti-drift semantics doc for the current `TransferNoteTx` proof-bearing path. This document does not override canonical protocol, formats, consensus or product semantics; it records the current proof-relevant meaning of `TransferNoteTx` and names what is still unresolved.
Owner: privAI proof, ledger and consensus architecture.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_PROOF_BOUNDARIES.md`
- `spec/PRIVAI_PROOF_COMPLETION_PLAN.md`
- `spec/PRIVAI_REFERENCE_VECTORS.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zapisac aktualna semantyke proof path dla `TransferNoteTx`,
- odciac lokalne zgadywanie relacji:
  - `statement_commit`
  - `public_inputs`
  - witness
  - `ExecutionBundle`
  - ledger validation,
- zamknac, co dzis jest `Current canonical`,
- nazwac, co pozostaje `Unresolved`.

To nie jest nowy canonical core.
To nie jest dokument o wszystkich railach.
To jest waski support doc tylko dla proof-bearing path `TransferNoteTx`.

## 2. Current Canonical Today

Aktualny stan kanoniczny dzis:
- `TransferNoteTx` jest jedyna obecna kanoniczna proof-covered tx class.
- canonical bytes `TransferNoteTx` sa canonical bytes `TxCore`.
- `statement_commit` jest jawna czescia `TxCore`.
- current statement model dla `TransferNoteTx` jest wyprowadzany z:
  - input note commits
  - input nullifiers
  - output note commits
  - fee
- current public inputs model dla `TransferNoteTx` jest wyprowadzany z:
  - `tx_id`
  - `statement_commit`
  - input note commits
  - input nullifiers
  - output note commits
  - fee
- `public_inputs_hash_for_transaction()` ma current canonical runtime answer dla `Transaction::TransferNote`.

Wazna zasada:
- samo istnienie `statement_commit` w `TxCore` nie oznacza, ze wszystkie tx classes maja juz finalnie zamrozony proof model.

## 3. Current Canonical Statement Rule

Current statement object:
- `TransferStatement`

Current statement fields:
1. `input_note_commits: Vec<Hash32>`
2. `input_nullifiers: Vec<Nullifier>`
3. `output_note_commits: Vec<Hash32>`
4. `fee: u64`

Current derivation rule:
- statement jest budowany bezposrednio z `TransferNoteTx`
- bierze:
  - `tx.core.inputs[*].note_commit`
  - `tx.core.input_nullifiers`
  - `tx.core.outputs[*].note_commit`
  - `tx.core.fee`

Current statement commitment rule:
- `statement_commit = H_transfer_statement_v0(canonical(TransferStatement))`

Current domain tag:
- `privai:proof:transfer-statement:v0`

Current canonical interpretation:
- `statement_commit` binduje publicznie widoczny statement transakcji,
- current preimage `statement_commit` nie zawiera witness,
- current preimage `statement_commit` nie zawiera auth bytes,
- current preimage `statement_commit` nie zawiera pelnych output plaintext openings.

## 4. Current Canonical Public Inputs Rule

Current public inputs object:
- `TransferPublicInputs`

Current public inputs fields:
1. `tx_id: Hash32`
2. `statement_commit: Hash32`
3. `input_note_commits: Vec<Hash32>`
4. `input_nullifiers: Vec<Nullifier>`
5. `output_note_commits: Vec<Hash32>`
6. `fee: u64`

Current derivation rule:
- public inputs sa budowane bezposrednio z `TransferNoteTx`
- `tx_id` jest liczone jako canonical transaction hash
- `statement_commit` jest brane z `tx.core.statement_commit`

Current public inputs hash rule:
- `public_inputs_hash = H_transfer_public_inputs_v0(canonical(TransferPublicInputs))`

Current domain tag:
- `privai:proof:transfer-public-inputs:v0`

Current canonical interpretation:
- public inputs hash binduje to, co proof path wystawia publicznie dla `TransferNoteTx`,
- witness nie jest czescia public inputs,
- auth nie jest czescia current public inputs hash,
- current public inputs sa proof-facing public summary, a nie pelnym ledger execution record.

## 5. Current Witness Rule

Current witness objects:
- `TransferInputWitness`
- `TransferOutputWitness`
- `TransferWitness`

Current witness fields:

`TransferInputWitness`
1. `amount: Amount14`
2. `witness_seed: Hash32`
3. `nullifier_key: Hash32`
4. `spend_policy_opening: Vec<u8>`
5. `aux_opening: Vec<u8>`

`TransferOutputWitness`
1. `note_commit: Hash32`
2. `recipient_opening: RecipientBoxPlaintext`

`TransferWitness`
1. `input: TransferInputWitness`
2. `outputs: Vec<TransferOutputWitness>`

Current witness commitment rule:
- `witness_commit = H_transfer_witness_v0(canonical(TransferWitness))`

Current domain tag:
- `privai:proof:transfer-witness:v0`

Current canonical interpretation:
- witness jest proving-side artifact, nie public input,
- current code ma pojedynczy `TransferInputWitness`,
- ten dokument nie dopowiada z samego tego faktu finalnej multi-input witness semantics ponad to, co obecny kod juz robi.

## 6. Current Statement-to-Witness Consistency Checks

Current proving path `TransferProvingData::from_tx_and_witness()` sprawdza:

1. `tx.core.statement_commit == TransferStatement::from_tx(tx).commitment()`
2. liczba `tx.core.outputs` jest rowna liczbie `witness.outputs`
3. dla kazdego outputu:
   - `tx.core.outputs[i].note_commit == witness.outputs[i].note_commit`
4. dla kazdego outputu:
   - `tx.core.outputs[i].payload_commit() == witness.outputs[i].recipient_opening.note_payload_commit`
5. dla kazdego outputu:
   - `tx.core.outputs[i].recipient_box.hint == witness.outputs[i].recipient_opening.bundle_id`

Current canonical interpretation:
- proving path juz dzis wymaga spojnosci statementu z tx bytes,
- proving path juz dzis wymaga spojnosci output witness z output commitments,
- current consistency checks nie oznaczaja jeszcze, ze caly ledger-side auth/state transition zostal zastapiony przez proof.

## 7. Current Relation To Execution Bundle

Current canonical facts:
- `public_inputs_hash_for_transaction()` zwraca hash dla `Transaction::TransferNote`
- `build_execution_bundle_from_transactions()` dodaje `TransferNoteTx` do:
  - `statement_commits`
  - `covered_tx_indexes`
  - `public_inputs_root`
- `build_execution_bundle_from_transfer_proofs()` wymaga, by:
  - `proving.public_inputs.statement_commit == proving.statement.commitment()`

Current canonical interpretation:
- `TransferNoteTx` jest proof-relevant participantem `ExecutionBundle`,
- current relation `TransferNoteTx -> public_inputs_hash -> public_inputs_root` jest realna w kodzie,
- current bytes i current runtime role `ExecutionBundle` nie oznaczaja jeszcze finalnie zamknietej semantyki wszystkich raili i wszystkich execution modes.

## 8. Ledger-vs-Proof Validation Boundary

Current canonical interpretation dla `TransferNoteTx`:

Proof-facing today:
- spojnosc `statement_commit` z current statement model,
- spojnosc `public_inputs` z tx-facing public summary,
- spojnosc output witness z output commitments i recipient opening binding,
- proof-related inclusion w `ExecutionBundle`.

Ledger-side today:
- auth validation,
- input existence / spendability checks,
- nullifier uniqueness enforcement,
- note set updates,
- state transition,
- `note_root` / `state_root` maintenance,
- block-level acceptance rules,
- consensus acceptance rules.
- fee validation rules,

Twarda zasada:
- proof nie zastepuje ledger validation,
- proof nie zastepuje transaction shape validation,
- proof nie zastepuje fee validation rules,
- istnienie proof artifact nie pozwala poluzowac ledger-side checks dla `TransferNoteTx`.

## 9. Current Non-Conformities That Matter

Najwazniejsze obecne luki istotne dla `TransferNoteTx` proof path:
- `ExecutionBundle` i `ProofCertificate` nie maja jeszcze pelnej finalnej semantyki dla wszystkich raili,
- `public_inputs_hash_for_transaction()` nie daje jeszcze finalnej odpowiedzi dla wszystkich proof-relevant klas,
- current bundle builder traktuje `LiteTransfer` jako proof-requiring, ale public inputs support dla lite raila nie jest domkniety tak jak dla `TransferNoteTx`,
- current witness shape jest zapisana w kodzie, ale ten dokument nie dopowiada ponad kod finalnej ogolnej multi-input semantics.

## 10. Unresolved Today

Nadal niezamykniete:
- finalna proof coverage rule dla wszystkich execution modes,
- finalna relacja `ExecutionBundle -> block validation`,
- finalna semantyka `ProofCertificate`,
- finalny end-to-end relation:
  - proof artifact
  - certificate
  - block roots
  - consensus reject rules,
- finalna odpowiedz, jak szeroko current witness shape ma generalizowac sie poza obecny kod `TransferNoteTx`.

## 11. Required Freeze Before Calling This Final

Zanim `TransferNoteTx` proof semantics bedzie mozna nazwac w pelni finalnym, trzeba jeszcze jawnie zamknac:
- final statement rule,
- final public inputs rule,
- final proof coverage rule,
- final relation do `ExecutionBundle`,
- final relation do `ProofCertificate`,
- explicit ledger-vs-proof validation boundary bez unresolved gaps.

## 12. How To Use This Doc

Jesli task dotyczy `TransferNoteTx` proof path:
- najpierw przeczytaj ten dokument,
- potem `spec/PRIVAI_PROOF_BOUNDARIES.md`,
- potem `spec/PRIVAI_PROOF_COMPLETION_PLAN.md`,
- dopiero potem zejdz do:
  - `privai-proof/src/transfer.rs`
  - `privai-proof/src/batch.rs`

Jesli task dotyczy innej tx class:
- nie rozszerzaj lokalnie tego dokumentu na `MarketplaceBatchTx` ani `OnChainLite`,
- najpierw sprawdz ich status w proof boundaries i proof completion plan.
