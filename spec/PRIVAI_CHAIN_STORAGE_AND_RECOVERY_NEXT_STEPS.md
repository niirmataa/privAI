# privAI Chain Storage And Recovery Next Steps

Status: focused working doc for chain storage, persistence and recovery follow-up.
Canonicality: supporting architecture and execution checklist. This document does not override canonical protocol, formats, consensus or product semantics; it records the agreed next steps for chain storage, RocksDB usage, persistence boundaries and recovery behavior.
Owner: privAI ledger, storage and node runtime.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_REFERENCE_VECTORS.md`
- `spec/PRIVAI_GAP_REGISTER.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zapisac, co robimy dalej po stronie chain/storage,
- domknac kontrakt storage zamiast tylko "miec RocksDB",
- odciac domysly o tym, co jest canonical state, a co tylko cache/index,
- zapisac checklisty pod persistence, restart, recovery i testing.

To nie jest dokument o wire format.
To nie jest dokument o semantyce consensus od zera.
To jest dokument wykonawczy i porzadkujacy dla storage/runtime layer.

## 2. Current Direction

Aktualny kierunek pozostaje:
- `privai-chain` definiuje canonical objects i hashes,
- `privai-ledger` odpowiada za state transition i durable state semantics,
- `privai-node` odpowiada za networking, sync i consensus overlay,
- RocksDB pozostaje sensownym embedded durable store dla node/runtime,
- `RocksDB` jest obecnym wyborem implementacyjnym dla runtime durability, a nie wymaganiem na poziomie protokolu,
- najwazniejszy kolejny temat nie brzmi "czy RocksDB", tylko:
  - co jest persisted,
  - co jest recoverable,
  - co jest cache,
  - jak restart/recovery zachowuje zgodnosc stanu.

## 3. Why This Is Coherent With The Existing Docs

Ten kierunek jest spojny z reszta docs, bo:
- nie zmienia canonical objects ani ich encoding,
- nie zmienia consensus semantics,
- nie zmienia root formulas,
- doprecyzowuje tylko runtime/persistence contract,
- wspiera juz zamrozone zasady typu:
  - `state_root` must match executed state,
  - consensus cannot ignore ledger validation,
  - safety/finality state cannot be reduced to "just blocks".

Ten dokument pasuje szczegolnie do:
- `PRIVAI_CONSENSUS.md`
- `PRIVAI_REFERENCE_VECTORS.md`
- `PRIVAI_REENTRY_GUIDE.md`
- `PRIVAI_GAP_REGISTER.md`

## 4. Main Next Tasks

### 4.1. Storage invariants

Trzeba zapisac jeden jawny kontrakt:
- jakie dane sa durable,
- jakie dane sa canonical state,
- jakie dane sa derived indexes,
- jakie dane sa ephemeral cache,
- ktore dane musza byc atomowo aktualizowane razem.

### 4.2. Recovery and restart rules

Trzeba zapisac:
- co node musi odtworzyc po restarcie,
- jak odtwarza snapshot,
- jak odtwarza `state_root`,
- jak odtwarza QC/finality-related state,
- co jest required for safe resume,
- co mozna przebudowac z durable state.

### 4.3. Storage layout discipline

Trzeba doprecyzowac:
- namespace / column-family strategy,
- key spaces,
- versioning strategy,
- granice miedzy ledger store i node-local auxiliary data.

### 4.4. Persistence tests

Trzeba dodac testy:
- persist -> restart -> reload -> same effective state,
- same `state_root` after recovery,
- blocks remain importable after restart,
- QC / finality state is not silently lost,
- indexes/cache can be rebuilt without changing canonical state.

### 4.5. Failure model

Trzeba zapisac:
- co znaczy partial write failure,
- co znaczy crash between validation and persistence,
- co znaczy crash after block accept but before all indexes are rebuilt,
- co jest recoverable,
- co wymaga explicit repair or rebuild.

## 5. What We Need To Freeze Next

### 5.1. Durable state set

#### required for canonical replay
- blocks / headers needed for local replay,
- state snapshot data needed to reproduce `state_root`,
- spent note nullifiers,
- spent ticket nullifiers,
- relevant note commitments / note status data,
- durable data needed to recompute canonical state commitments.

#### required for safe consensus resume
- consensus safety state that must survive restart,
- QC-related data required for correct resume,
- finality-relevant consensus runtime data,
- any persisted view/round/locked-QC-related state if the model requires it.

### 5.2. Rebuildable data

Do jawnego oznaczenia jako rebuildable:
- convenience indexes,
- caches,
- temporary sync data,
- connection/network state,
- any derived material that can be recomputed from durable state.

### 5.3. Non-durable runtime data

Do jawnego oznaczenia jako ephemeral:
- active network sessions,
- in-memory broadcast state,
- temporary per-run queues,
- short-lived pools and transient worker state.

## 6. Checklist: Storage Invariants

- [ ] Spisac canonical durable state set.
- [ ] Spisac rebuildable indexes.
- [ ] Spisac ephemeral runtime-only state.
- [ ] Okreslic granice ledger state vs node-local aux state.
- [ ] Okreslic atomowe grupy write operations.
- [ ] Okreslic, kiedy block is "validated", "persisted", "finalized", "fully indexed".
- [ ] Okreslic, czy `state_root` is computed-on-write, computed-on-read, czy mix.
- [ ] Okreslic source of truth dla note status.
- [ ] Okreslic source of truth dla nullifier sets.
- [ ] Okreslic source of truth dla QC/finality state.
- [ ] Okreslic source of truth dla `tip/head`.
- [ ] Okreslic source of truth dla `finalized head`.
- [ ] Okreslic source of truth dla `last persisted but not fully indexed block`.

## 7. Checklist: Recovery And Restart

- [ ] Spisac expected restart sequence.
- [ ] Spisac minimal durable data needed for safe restart.
- [ ] Spisac how `state_root` is revalidated after restart.
- [ ] Spisac how tip / height / prev linkage is recovered.
- [ ] Spisac how safety state is recovered.
- [ ] Spisac how QC data is recovered.
- [ ] Spisac what happens if indexes are missing but base state exists.
- [ ] Spisac what happens if caches are missing.
- [ ] Spisac what happens if last write is partial or interrupted.
- [ ] Spisac when explicit rebuild procedure is required.

## 8. Checklist: RocksDB / Storage Layout

- [ ] Opisac current DB layout / namespaces / column families.
- [ ] Opisac versioning strategy for stored data.
- [ ] Opisac compatibility assumptions across versions.
- [ ] Opisac compaction / performance-sensitive areas only if they affect correctness.
- [ ] Odcielic correctness invariants od tuning/performance knobs.
- [ ] Spisac which writes must be batched atomically.
- [ ] Spisac which state can be lazily materialized.

## 9. Checklist: Tests

- [ ] `persist_state_then_restart_same_root`
- [ ] `persist_block_then_restart_then_continue_import`
- [ ] `recover_tip_and_height_after_restart`
- [ ] `recover_spent_nullifiers_after_restart`
- [ ] `recover_ticket_nullifiers_after_restart`
- [ ] `recover_qc_or_safety_state_after_restart`
- [ ] `rebuild_indexes_without_state_drift`
- [ ] `state_root_matches_recomputed_state_after_reload`
- [ ] `crash_between_steps_does_not_silently_corrupt_resume`
- [ ] `restart_without_indexes_then_rebuild_same_root`
- [ ] `partial_index_loss_does_not_change_canonical_state`
- [ ] `restart_with_missing_ephemeral_state_still_safe`

## 10. What We Are Not Doing Now

- [ ] Nie zmieniamy canonical block/object formats w ramach storage work.
- [ ] Nie wymieniamy RocksDB tylko dlatego, ze storage layer nie jest jeszcze opisany.
- [ ] Nie mieszamy correctness invariants z tuningiem wydajnosci.
- [ ] Nie traktujemy local cache/index jako source of truth bez jawnego freeze.
- [ ] Nie redukujemy recovery do "mamy bloki, reszte sie dorobi".

## 11. Recommended Execution Order

1. Spisac `CHAIN_STORAGE_INVARIANTS`
2. Spisac `RECOVERY_AND_RESTART_RULES`
3. Zmapowac current RocksDB layout / namespaces
4. Dodac persistence/restart tests
5. Dopiero potem poprawiac lub rozszerzac storage internals

## 12. Practical Next Step For The Team

Najblizszy sensowny krok:
- zrobic osobny dokument support-doc / anti-drift:
  - `spec/PRIVAI_CHAIN_STORAGE_INVARIANTS.md`

Potem:
- osobny dokument support-doc / anti-drift:
  - `spec/PRIVAI_RECOVERY_AND_RESTART_RULES.md`

Dopiero po tym:
- wejsc w kod i testy storage/recovery.

## 13. Final Assessment

Tak, ten kierunek jest spojny z reszta docs.

Powod:
- nie otwiera nowej architektury,
- nie rozmywa canonical set,
- nie konkuruje z protocol/formats/consensus docs,
- dopowiada tylko runtime/storage contract, ktory i tak musi zostac jawnie domkniety.
