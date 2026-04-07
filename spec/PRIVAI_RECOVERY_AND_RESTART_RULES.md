# privAI Recovery And Restart Rules

Status: support doc anti-drift for current restart/recovery behavior.
Canonicality: non-overriding support doc. This document does not define new protocol, format, consensus or product semantics; it maps the current code behavior of the restart/recovery path so that devs and agents can reason about what survives a restart, what is lost, and where the recovery contract has gaps — without guessing.
Owner: privAI ledger, storage and node runtime.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`
- `spec/PRIVAI_CHAIN_STORAGE_INVARIANTS.md`
- `spec/PRIVAI_CHAIN_STORAGE_AND_RECOVERY_NEXT_STEPS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zapisac jeden opis aktualnego restart/recovery behavior,
- dac devom i agentom gotowa mape: co przetrwa restart, co jest tracone, co jest rebuildable,
- odciac zgadywanie o recovery contract,
- rozdzielic:
  - co jest `Current canonical`,
  - co jest `Current non-conformity`,
  - co jest `Unresolved`.

To nie jest dokument wire format ani semantyki consensus.
To jest dokument o zachowaniu node'a po restarcie.

## 2. How Devs And Agents Should Use This Doc

Czytaj razem z:
- `spec/PRIVAI_CHAIN_STORAGE_INVARIANTS.md` — co jest durable, co ephemeral,
- `spec/PRIVAI_CHAIN_STORAGE_AND_RECOVERY_NEXT_STEPS.md` — plan kolejnych prac,
- `spec/PRIVAI_CONSENSUS.md` — finality, block validity, state commitments.

Regula interpretacji:
- jesli zachowanie jest `Current canonical`, nie wolno go lokalnie zmieniac bez decyzji,
- jesli zachowanie jest `Unresolved`, nie wolno go dopowiadac przez "najbardziej sensowna" implementacje,
- jesli jest `Current non-conformity`, trzeba je naprawic albo jawnie utrzymac.

## 3. Scope And Boundary

### 3.1. Dokument obejmuje

- current startup/load sequence,
- co jest odtwarzane z durable state,
- co jest tracone po restarcie,
- state_root revalidation,
- tip/height recovery,
- QC/safety recovery,
- recovery po brakujacych indexach/cache,
- failure model (crash scenarios).

### 3.2. Dokument nie obejmuje

- wire format obiektow (nalezy do `PRIVAI_CANONICAL_FORMATS.md`),
- block validity i finality rules (nalezy do `PRIVAI_CONSENSUS.md`),
- validator session transport recovery (nalezy do `PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`),
- compaction, tuning RocksDB.

## 4. Current Code Map (restart/recovery)

- `Ledger::open()` (`privai-ledger/src/ledger.rs:70`) — laduje `LedgerSnapshot` z store'a, fallback na genesis.
- `PrivaiNode::open_with_components()` (`privai-node/src/node.rs:131`) — tworzy node z ledgerem.
- `LedgerStore::load()` (`privai-ledger/src/store.rs`) — laduje snapshot (JSON) z MemoryStore/FileSystemStore/RocksDBStore.
- `LedgerStore::save()` (`privai-ledger/src/store.rs`) — zapisuje caly snapshot.
- `Ledger::flush()` — alias na `store.save(snapshot)`.
- `Ledger::update_consensus_safety()` — zapisuje safety state + flush.
- `compute_state_root()` (`privai-ledger/src/ledger.rs:21`) — przelicza merkle root z snapshotu.
- `state_sync::handle_sync_response()` (`privai-node/src/state_sync.rs:174`) — import blokow z peerow.

## 5. Current Startup / Load Sequence

### 5.1. `Ledger::open()` — sekwencja

```
1. store.load() → Option<LedgerSnapshot>
2. Jesli None: uzyj LedgerSnapshot::genesis(chain_id)
3. store.save(snapshot)  // flush genesis lub loaded state
4. Ok(Ledger { store, snapshot, mempool: Mempool::new(), proof_verifier })
```

### 5.2. `PrivaiNode::open_with_components()` — sekwencja

```
1. Ledger::open(store, chain_id, verifier) → ledger
2. PrivaiNode {
     config,
     ledger,
     proof_artifacts,
     artifact_verifier,
     prevotes: HashMap::new(),        // puste — ephemeral
     precommits: HashMap::new(),      // puste — ephemeral
     qc_emitted: HashSet::new(),      // puste — ephemeral
     current_round: 0,                // NIE odtwarzane z snapshotu
     round_start_time_ms: 0,          // NIE odtwarzane z snapshotu
     view_changes: HashMap::new(),    // puste — ephemeral
     falcon_sk: None,                 // ladowane osobno (with_falcon_key)
   }
```

### 5.3. Current canonical startup behavior

- `LedgerSnapshot` jest ladowany z store'a jako calosc.
- Jesli store jest pusty, startujemy z genesis (height=0, empty notes, empty nullifiers).
- `PrivaiNode` NIE odtwarza `current_round`, `round_start_time_ms`, `prevotes`, `precommits`, `qc_emitted` — te pola sa inicjalizowane na wartosci domyslne (0, puste mapy).
- `consensus_safety` jest ladowany ze snapshotu — ale jego aktualnosc zalezy od tego, kiedy ostatni `update_consensus_safety()` byl wywolany.

## 6. Current Restart Assumptions

Current canonical: node zaklada po restarcie, ze:

1. **Dane w store sa kompletne i spojne.** Nie ma walidacji spojnosci po ladowaniu (brak cross-check `state_root`, brak weryfikacji `height` vs `blocks.len()`, brak weryfikacji `tip_hash` vs ostatni blok w `blocks`).

2. **Ostatni zapisany snapshot jest konsystentny.** Nie ma mechanizmu "replay from last good snapshot" jesli ostatni JSON jest uszkodzony.

3. **Ephemeral state moze byc zresetowane.** Mempool, vote tracking, QC emission — wszystko startuje od zera.

4. **Block cache jest taki, jaki byl przy ostatnim save.** Bloki w `blocks` sa dostepne; bloki poza `blocks` sa niedostepne lokalnie.

5. **State sync moze nadrobic zalegle bloki.** Jesli node jest z tyłu, `state_sync` moze pobrac bloki od peerow.

## 7. Durable Data Required For Safe Resume

### 7.1. Co jest trwale zapisywane (z `LedgerSnapshot`)

| Pole | Do czego potrzebne po restarcie | Czy jest? |
|------|-------------------------------|-----------|
| `chain_id` | Identyfikacja sieci | Tak |
| `height` | Pozycja w chainie | Tak |
| `tip_hash` | Weryfikacja kolejnego bloku (prev_block_hash check) | Tak |
| `notes` | Anti-double-spend, note set | Tak |
| `spent_nullifiers` | Anti-double-spend | Tak |
| `spent_ticket_nullifiers` | Marketplace anti-double-spend | Tak |
| `blocks` (≤128) | Local block history, state sync serving | Tak (ale ograniczone) |
| `qcs` | State sync, finality verification | Tak (rosnie bez limitu) |
| `consensus_safety` | Double-sign prevention | Czesciowo (patrz nizej) |

### 7.2. Minimalny set do bezpiecznego resume

Do bezpiecznego wznawienia pracy node potrzebuje:
- `chain_id` — aby potwierdzic ze laczymy do wlasciwej sieci
- `height` + `tip_hash` — aby kontynuowac chain od wlasciwego miejsca
- `notes` + `spent_nullifiers` + `spent_ticket_nullifiers` — aby nie podwojnie wydawac
- `consensus_safety.last_voted_view` — aby nie podpisywac dwa razy w tej samej rundzie

`blocks`, `qcs`, `consensus_safety.current_round` — sa uzyteczne ale nie sa krytyczne do resume (mozna nadrobic przez state sync).

## 8. Ephemeral State Lost On Restart

Nastepujace dane sa tracone po restarcie i nie maja recovery path:

| Dane | Konsekwencja utraty |
|------|---------------------|
| Mempool entries | Transakcje zlozone przed restartem sa tracone; nowe musza byc ponownie zlozone |
| Mempool reserved_inputs/nullifiers | Brak wplywu — odbudowywane przy nowych insertach |
| Vote tracking (prevotes, precommits) | Node startuje od nowa; moze ponownie glosowac w biezacej rundzie |
| QC emission dedup (`qc_emitted`) | Brak wplywu — odbudowywane przy nowych QC |
| View change state | Brak wplywu — odbudowywane przy nowych view change |
| Round timing (`round_start_time_ms`) | Reset do 0 — timeout checker zaczyna od nowa |
| Connection pool state | Nowe polaczenia budowane od zera |
| Ban list / rate limiter | Reset — wczesniej zbanowani peery sa unbanned |
| Block cache (wektor `block_cache` w ConsensusLoop) | Pusty — serwerowanie blokow spada na `ledger.snapshot().blocks` |
| Falcon secret key | Musi byc zaladowany osobno (`with_falcon_key`) |

## 9. State Root Revalidation — Current Behavior

### 9.1. Co dzieje sie po restarcie

Current canonical: **state_root jest czesciowo rewalidowany po restarcie.**

`Ledger::open()` laduje snapshot i sprawdza:
- czy obliczony `state_root` w załadowanym bloku (pod numerem `snapshot.height`) odpowiada korzeniowi `compute_state_root(snapshot)`. Testowane jest to na wlocie; niespójność rzuca błąd przed inicjalizacją node'a (`StateRootMismatch`).

Ograniczenie tej boot-time weryfikacji leży jednak w konieczności posiadania lokalnie ostatniego dodanego bloku: jeśli plik nie przechowuje bloku pod indexem w tablicy cache'u bloków (`snapshot.blocks.get(&snapshot.height)`) — procedura całkowicie ignoruje krok wyliczenia root state'a, w konsekwencji czego węzeł ładuje się jako w pełni zdolny do pracy.

`Ledger::open()` nadal nie weryfikuje pełnej osi historycznej ani spójności logicznej:
- czy `notes`, `spent_nullifiers`, `spent_ticket_nullifiers` sa w pelni logicznie spójne po uszkodzeniu wewnętrznym i podmianie danych dla mniejszych historycznie height (testy potwierdzają, że "logicznie częściowy" zrzut w postaci powołanej Noty bez przypisanego na siłę nagłówka czy w oderwaniu od historii przechodzi przez blokady).

### 9.2. Gdzie state_root jest weryfikowany

`state_root` jest weryfikowany TYLKO w `validate_block()` — przy importcie nowego bloku:
```rust
let computed_state_root = compute_state_root(&temp);
if computed_state_root != block.header.state_root {
    return Err(ValidationError::StateRootMismatch { ... });
}
```

Ta weryfikacja dotyczy NOWEGO bloku, nie istniejacego stanu po restarcie.

### 9.3. Konsekwencja

Mechanika rozruchu wykrywa teraz asymetrię state_root pomiędzy głównym stanem węzła a nagłówkiem ostatnio dodanego bloku. Chroni to przed używaniem node'a na uszkodzonym zrzucie z dysku tam, gdzie cache bloków podtrzymuje najnowszy head w pamięci.

## 10. Tip/Head Recovery — Current Behavior

### 10.1. Co jest odtwarzane

- `height` — ladowany ze snapshotu
- `tip_hash` — ladowany ze snapshotu

### 10.2. Co NIE jest weryfikowane

- czy `tip_hash` jest hashem bloku ktory jest w `blocks[height]`
- czy `height` jest spojne z `blocks.len()` i `blocks.keys().max()`
- czy chain od bloku 0 do `height` jest kompletny

### 10.3. Current canonical behavior

Node uzywa `tip_hash` do weryfikacji kolejnego bloku:
```rust
if block.header.prev_block_hash != snapshot.tip_hash {
    return Err(ValidationError::InvalidParent);
}
```

Podobnie jak the `state_root`, **tip_hash nie jest weryfikowany w locie podczas uruchomienia `Ledger::open()`.** 
Oznacza to, że uszkodzony snapshot z błędnym `tip_hash` (niebędący autentycznym hashem pliku w cache, z którym współpracuje the ledger height) omija check.
Node odrzuci go tylko przy imporcie dopiero w przypadku odpytywania czy dołączania nowych poprawnych bloków do node'a. Zepsuty tip_hash akceptuje uruchomienie węzła jako poprawny zapis!

## 11. QC/Safety Recovery — Current Behavior

### 11.1. QC data

- `qcs: BTreeMap<u64, QuorumCertificate>` jest ladowany ze snapshotu
- QCs sa uzywane przez `state_sync::handle_sync_response()` do finalizacji synced blokow
- Brak weryfikacji QC po restarcie — sa traktowane jako zaufane

### 11.2. ConsensusSafetyState

- `consensus_safety` jest ladowany ze snapshotu
- `last_voted_view` — jesli byl zapisany, chroni przed double-sign
- `current_round` — jesli byl zapisany, node kontynuuje od tej rundy
- `locked_qc` — nigdy nie jest zapisywany, po restarcie jest `None`
- `current_view` — nie jest jawno aktualizowany, po restarcie moze byc outdated

### 11.3. Current canonical behavior

`PrivaiNode::open_with_components()` prawidłowo odtwarza zmienną konfiguracyjną i nie gubi stanu wybudzenia:
```rust
let current_round = ledger.snapshot().consensus_safety.current_round;
```

To oznacza ze jeśli `consensus_safety.current_round` jest pomyślnie zrzucane na dysk pod `PrivaiNode` przy `view_change`, nowy Node po wybudzeniu i rewalidacji powróci do zapisanej rundy chroniąc spójność działania node'a pod `state_sync` (nie gubiąc wypracowanej historii sesji z sieci po nagłych awariach).

## 12. Recovery After Missing Indexes/Cache

### 12.1. Brakujacy block cache

Jesli `blocks` w snapshotie jest mniejszy niz 128 blokow (bo wczesniej byl eviction), node startuje z tym co ma.
- Bloki ktore sa w `blocks` sa dostepne lokalnie
- Bloki ktorych nie ma — mozna pobrac przez `state_sync`

### 12.2. Brakujacy mempool

Mempool jest zawsze pusty po restarcie. Nie ma recovery. Uzytkownicy musza ponownie zlozyc transakcje.

### 12.3. Brakujace QCs

Jesli `qcs` nie ma QC dla niektorych height, state sync moze nie byc w stanie sfinalizowac synced blokow.
Ale node moze kontynuowac prace — QCs sa potrzebne glownie do state sync serving i finalizacji.

### 12.4. Brakujace proof artifacts

`ProofArtifactStore` jest osobna warstwa. Jesli `MemoryProofArtifactStore` jest uzywany, artifacts sa tracone po restarcie. Jesli jest inna implementacja, zalezy od niej.

## 13. Failure Model

### 13.1. Crash przed `store.save()`

Jesli node crashuje przed `apply_block` wywola `store.save()`:
- Stan na dysku jest niezmieniony — ostatni zapisany snapshot jest konsystentny
- Blok ktory byl w trakcie przetwarzania jest tracony
- Node kontynuuje od ostatniego zapisanego stanu

Current canonical: to jest bezpieczne — `save()` jest po apply, wiec crash przed save oznacza "blok nie byl zastosowany".

### 13.2. Crash w trakcie `store.save()`

- **FileSystemStore**: temp+rename — jesli crash przed rename, stary plik jest nietkniety, temp moze zostac (ale jest ignorowany bo ma inne rozszerzenie). Atomic na wiekszosci FS.
- **RocksDBStore**: `put_cf` + `flush()` — jesli crash przed flush, RocksDB WAL moze pomoc. Ale nie ma testow tego scenariusza.

Current canonical: FileSystemStore jest atomic. RocksDBStore jest prawdopodobnie atomic dzieki WAL, ale nie jest zweryfikowane testami.

### 13.3. Crash po `store.save()` ale przed response do caller

- Stan na dysku jest konsystentny (save sie udal)
- Caller nie wie ze sie udalo — moze sprobowac ponownie
- `apply_block` moze failowac przy ponownej probie (blok juz zastosowany, duplicate nullifier/output)

Unresolved: nie ma idempotency path dla ponownego apply po crash.

### 13.4. Uszkodzony JSON na dysku

Zidentyfikowano różne scenariusze stanu awaryjnego po stronie dyskowej:
- **Corrupted / Truncated JSON** (np. `key must be a string`, błąd składniowy, przecięty plik z nagłym EOF): `store.load()` zwraca błąd deserializacji Serde → rzuca to twardy panic `LedgerError::Store` przy `Ledger::open()`. Odrzucenie to następuje jednoznacznie (tzw. zachowanie **fail-fast**), uniemożliwiając uruchomienie node'a na brudnych danych. Zachowanie to oznacza się jako `Current canonical`.
- **Missing / Empty Store** (usunięto plik lub nie powstał jeszcze stan po pierwszym starcie): odtwarza gładko stan fallback z pustego pliku `genesis` i height 0. Zachowanie przypisane jako **empty-store genesis fallback** stanowi `Current canonical`.
- **Semantycznie niespójny, ale parsowalny JSON** (np. puste/losowo wdrożone Notatki lub niespójny powiązaniami cache bez osi w blocks): Plik parsuje się w Serde JSON-ie na twardo w locie i akceptuje dziury w powiązaniach miedzy historiami. Stanowi to lukę **loadable-but-incomplete behavior**, podpadającą pod status `Current non-conformity`. Brak procedur sprawdzania powiązań pozwala odpalić node na takim pliku.
- **Brak procedur explicit repair/rebuild** dla stanów z uszkodzonymi JSON-ami pozostaje na statusie `Unresolved` (w kwestii tego czy chcemy wrzucić automatyczny discard).

## 14. Current Non-Conformities

### 14.1. No cross-field consistency check

Nie ma weryfikacji po restarcie:
- `tip_hash` vs `blocks[height]`
- `height` vs `blocks.keys().max()`
- `notes` vs `spent_nullifiers` consistency
- `qcs` vs `blocks` consistency

### 14.4. No corruption recovery path

Jesli snapshot jest uszkodzony błędem odczytu I/O (truncated, bad structure JSON), node nie startuje. Nie ma:
- automatic fallback na wczesniejszy snapshot
- repair procedury
- rebuild from blocks procedury
W kwestiach logiki (omijanie sprawdzenia root_mismatch dla brakującego pliku z tablicy bloków, brak cross-checking) patrz punkt `13.4. Uszkodzony JSON na dysku`.

## 15. Unresolved Gaps

### 15.1. State_root revalidation on boot

Unresolved: czy `state_root` powinno byc weryfikowane po kazdym restarcie. Koszt: przeliczenie `compute_state_root(snapshot)` — O(n) od liczby notes + nullifiers.

### 15.2. Tip_hash vs blocks consistency

Unresolved: czy node powinien weryfikowac `tip_hash == blocks[height].hash()` po restarcie.

### 15.3. Recovery from corrupted snapshot

Unresolved: jaka jest procedura jesli snapshot jest uszkodzony. Opcje:
- fallback na genesis (traci stan)
- replay from blocks (wymaga pelnego block store)
- repair from peer (wymaga state sync)

### 15.4. Idempotency after crash-during-save

Unresolved: co jesli `save()` sie udal ale caller nie wie. Ponowny `apply_block` moze failowac na duplicate check.

### 15.5. Safety state round recovery

Unresolved: czy `current_round` powinno byc odtwarzane z `consensus_safety`. Obecnie jest ignorowane.

### 15.6. QC verification after restart

Unresolved: czy QCs w snapshotie powinny byc weryfikowane po restarcie, czy sa traktowane jako zaufane.

### 15.7. Mempool recovery

Unresolved: czy mempool powinien byc persistowany, czy z definicji ephemeral jest wystarczajace.

## 16. What Requires Explicit Freeze Update

Nastepujace zmiany nie moga byc robione po cichu:
- zmiana w logice przeliczania state_root po starcie poza ograniczeniami braku head w cache (`blocks` cache eviction),
- dodanie tip_hash vs blocks consistency check,
- usuwanie mechanizmu `current_round` load z `consensus_safety` na starcie,
- dodanie corruption recovery (fallback na wczesniejszy snapshot, replay from blocks),
- dodanie idempotency path dla apply_block,
- zmiana failure modelu (co znaczy crash w trakcie save).

## 17. Frozen Rules

- `Ledger::open()` laduje caly `LedgerSnapshot` z store'a — nie ma partial load,
- jesli store jest pusty, startujemy z genesis — nie ma implicit recovery,
- mempool jest z definicji ephemeral — nie ma recovery,
- `state_root` jest weryfikowany tylko przy importcie nowego bloku, nie po restarcie,
- `apply_block` jest jedyna sciezka zmiany stanu — nie ma innego write path.
