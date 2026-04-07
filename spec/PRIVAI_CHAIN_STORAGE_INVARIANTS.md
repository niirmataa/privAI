# privAI Chain Storage Invariants

Status: support doc anti-drift for current chain storage and persistence behavior.
Canonicality: non-overriding support doc. This document does not define new protocol, format, consensus or product semantics; it maps the current code behavior of the storage/persistence layer so that devs and agents can reason about what is durable, what is rebuildable, and what is ephemeral — without guessing.
Owner: privAI ledger, storage and node runtime.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_CHAIN_STORAGE_AND_RECOVERY_NEXT_STEPS.md`
- `spec/PRIVAI_GAP_REGISTER.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zapisac jeden kanoniczny opis zachowania warstwy storage/persistence,
- odciac zgadywanie o tym, co jest durable, rebuildable czy ephemeral,
- rozdzielic:
  - co jest `Current canonical`,
  - co jest `Current non-conformity`,
  - co jest `Unresolved`,
  - co jest `Future target requiring migration`,
- dac devom i agentom jeden punkt odniesienia przed pisaniem testow, migracji albo refactorow storage.

To nie jest dokument wire format.
To nie jest tez dokument semantyki consensus ani proof.
To jest dokument o zachowaniu warstwy trwalosci pomiedzy canonical objects a runtime.

## 2. How Devs And Agents Should Use This Doc

Ten dokument nalezy czytac razem z:
- `spec/PRIVAI_PROTOCOL_CORE.md` dla znaczenia obiektow (note, nullifier, block),
- `spec/PRIVAI_CONSENSUS.md` dla block validity, state commitments, finality,
- `spec/PRIVAI_CHAIN_STORAGE_AND_RECOVERY_NEXT_STEPS.md` dla planu kolejnych prac.

Regula interpretacji:
- jesli zachowanie jest tutaj oznaczone jako `Current canonical`, nie wolno go lokalnie "ulepszac" bez jawnej decyzji i migracji,
- jesli zachowanie jest oznaczone jako `Unresolved`, nie wolno go dopowiadac przez implementacje "najbardziej sensownej" wersji,
- jesli zachowanie jest oznaczone jako `Current non-conformity`, nie wolno go sprzedawac jako finalnego invariantu; trzeba je albo naprawic, albo jawnie utrzymac jako luka,
- jesli kod i ten dokument sie rozjezdzaja, nalezy zglosic rozjazd albo doprowadzic kod do zgodnosci, a nie wybierac wygodniejszej wersji.

## 3. Scope And Boundary

### 3.1. Dokument obejmuje

- trwalosc stanu ledgera (durable state),
- granice miedzy canonical state, derived indexes a ephemeral runtime,
- source of truth dla poszczegolnych zbiorow danych,
- atomicity groups write operations,
- aktualny model storage backends (MemoryStore, FileSystemStore, RocksDBStore),
- aktualny model recovery po restarcie.

### 3.2. Dokument nie obejmuje

- wire format obiektow (nalezy do `PRIVAI_CANONICAL_FORMATS.md`),
- block validity i finality rules (nalezy do `PRIVAI_CONSENSUS.md`),
- note/UTXO semantyka (nalezy do `PRIVAI_PROTOCOL_CORE.md`),
- compaction, tuning RocksDB, performance knobs (chyba ze wplywaja na correctness),
- validator session transport (nalezy do `PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`).

## 4. Current Code Map

### 4.1. privai-ledger

- `LedgerSnapshot` (`privai-ledger/src/state.rs`) — jedyna jednostka stanu. Zawiera caly stan ledgera w jednej strukturze.
- `LedgerStore` trait (`privai-ledger/src/store.rs`) — `load() -> Option<LedgerSnapshot>` i `save(&LedgerSnapshot)`.
- `MemoryStore` — in-memory, do testow.
- `FileSystemStore` — JSON na dysku, atomic temp+rename.
- `RocksDBStore` — RocksDB, ale uzywa CF `ledger_state` jako jednego keya `b"snapshot"`.
- `Ledger` (`privai-ledger/src/ledger.rs`) — state machine: open, apply_block, flush, update_consensus_safety.
- `Mempool` (`privai-ledger/src/mempool.rs`) — in-memory only, niepersistowany.
- `compute_state_root` (`privai-ledger/src/ledger.rs`) — oblicza merkle root z notes + nullifiers.

### 4.2. privai-chain

- `Block`, `BlockHeader`, `BlockBody` — canonical block objects.
- `Transaction` enum — TransferNote, MarketplaceBatch, LiteTransfer, SettlementTx, ModelTx, StakeTx.
- `QuorumCertificate` — persisted in snapshot.
- `Nullifier` — spent marker.

### 4.3. privai-node

- `PrivaiNode` (`privai-node/src/node.rs`) — otacza `Ledger<S, V>`, trzyma vote tracking, QC emission state.
- `ConsensusLoop` (`privai-node/src/consensus_loop.rs`) — dispatch, broadcast, QC collection.
- `state_sync` (`privai-node/src/state_sync.rs`) — sync request/response, import blocks.

## 5. Current Canonical Durable State

Aktualny model: `LedgerSnapshot` jest zapisywany jako JEDNA wartosc (whole-snapshot serialization).

### 5.1. Durable state set (co jest zapisywane)

Nastepujace dane sa trwale zapisywane przez `LedgerStore::save()`:

| Pole | Typ | Opis |
|------|-----|------|
| `chain_id` | `u32` | ID chaina, identyfikuje siec |
| `height` | `u64` | Wysokosc ostatniego zatwierdzonego bloku |
| `tip_hash` | `Hash32` | Hash ostatniego bloku |
| `blocks` | `BTreeMap<u64, Block>` | Cache blokow (max 128) |
| `notes` | `BTreeMap<Hash32, NoteRecord>` | Wszystkie note'y z ich statusem |
| `spent_nullifiers` | `BTreeSet<Nullifier>` | Spent note nullifiers |
| `spent_ticket_nullifiers` | `BTreeSet<Nullifier>` | Spent ticket nullifiers (marketplace) |
| `qcs` | `BTreeMap<u64, QuorumCertificate>` | QCs po height |
| `consensus_safety` | `ConsensusSafetyState` | Stan bezpieczenstwa consensusu |

### 5.2. Format persystencji

- **FileSystemStore**: `ledger-state.json`, zapis atomic temp+rename.
- **RocksDBStore**: CF `ledger_state`, key `b"snapshot"`, wartosc = JSON `LedgerSnapshot`, po `put_cf` nastepuje `flush()`.
- **MemoryStore**: clone w RAM, nie trwaly.

Current canonical: caly `LedgerSnapshot` jest serializowany jako JSON i zapisywany jako jedna operacja.

## 6. Rebuildable Derived State

Nastepujace dane sa rebuildable z durable state (LedgerSnapshot):

| Dane | Z czego rebuildable | Uwagi |
|------|---------------------|-------|
| `state_root` | `compute_state_root(snapshot)` z notes + nullifiers + ticket_nullifiers | Mozna przeliczyc w kazdym momencie z pelnego snapshotu |

Current canonical: `state_root` jest jedynym istotnym derived state, ktore mozna przeliczyc z durable snapshot bez zewnetrznego zrodla.

Ponizsze dane NIE sa rebuildable z durable state i sa opisane w sekcji 7 (Ephemeral):
- Mempool
- Vote tracking (prevotes/precommits)
- Connection/network state
- QC emission tracking

## 7. Ephemeral Runtime State

Nastepujace dane sa ephemeral — nie przetrwaja restartu:

| Dane | Gdzie | Dlaczego |
|------|-------|----------|
| Mempool entries | `Mempool` w `Ledger` | Nie jest czescia `LedgerSnapshot` |
| Mempool reserved_inputs/nullifiers | `Mempool` | Jw. |
| Vote tracking: prevotes, precommits | `PrivaiNode` | Nie jest zapisywany |
| QC emission dedup (`qc_emitted`) | `PrivaiNode` | Nie jest zapisywany |
| View change state | `PrivaiNode` | Nie jest zapisywany |
| Round timing (`round_start_time_ms`) | `PrivaiNode` | Nie jest zapisywany |
| Connection pool state | `ValidatorSessionTransport` | Session layer |
| Ban list / rate limiter state | `ValidatorSessionTransport` | Session layer |
| Incoming sync state | `state_sync` | Runtime |

## 8. Source-Of-Truth Mapping

### 8.1. Note status (Unspent / Spent)

- Source of truth: `LedgerSnapshot.notes: BTreeMap<Hash32, NoteRecord>`
- Kazdy `NoteRecord` ma pole `status: NoteStatus` (`Unspent` lub `Spent { nullifier, spent_in_block }`)
- Zmiana statusu: `apply_transaction` przy apply_block
- Trwalosc: tak, czesc `LedgerSnapshot` zapisywanego przez `LedgerStore::save()`

### 8.2. Note commitments / note set

- Source of truth: `LedgerSnapshot.notes` — kluczem jest `Hash32` = `note_commit`
- Dodawanie: `apply_transaction` dodaje nowe outputy
- Nie ma dedykowanego indeksu — `notes` jest jednoczesnie zbiorem i indeksem
- Trwalosc: tak

### 8.3. Spent note nullifiers

- Source of truth: `LedgerSnapshot.spent_nullifiers: BTreeSet<Nullifier>`
- Dodawanie: `apply_transaction` wstawia nullifier przy spend
- Sprawdzanie: `validate_transaction` sprawdza czy nullifier juz istnieje (anti-double-spend)
- Trwalosc: tak

### 8.4. Spent ticket nullifiers

- Source of truth: `LedgerSnapshot.spent_ticket_nullifiers: BTreeSet<Nullifier>`
- Osobny zbior — unika kolizji z note nullifiers
- Dodawanie: `apply_transaction` dla `MarketplaceBatch`
- Sprawdzanie: `validate_transaction` dla `MarketplaceBatch`
- Trwalosc: tak

### 8.5. Block store

- Current canonical: `LedgerSnapshot.blocks: BTreeMap<u64, Block>` jest recent-block cache, nie pelny archival store
- Persystencja: bloki sa zapisywane jako czesc calego `LedgerSnapshot` — kazdy `save()` trwale zapisuje to, co jest w `blocks`
- Eviction: po `apply_block`, jesli `blocks.len() > 128`, bloki ponizej cutoff sa usuwane z pamieci
- Po restarcie: `LedgerStore::load()` wczytuje snapshot — bloki ktore byly w `blocks` przed ostatnim `save()` sa dostepne, o ile nie byly wyevictowane przed zapisem
- State sync: handler `serve_sync_response` uzywa `block_cache` (in-memory wektor w `ConsensusLoop`) z fallbackiem na `ledger.snapshot().blocks` — wiec synced bloki sa serwowane z pamieci
- Uwaga: eviction nie zmienia semantyki reszty snapshotu (notes, nullifiers, height, tip_hash) — zmienia tylko lokalna dostepnosc historii blokow

### 8.6. Tip / head

- Source of truth: `LedgerSnapshot.tip_hash` + `LedgerSnapshot.height`
- Aktualizacja: `apply_block` ustawia `height` i `tip_hash` po kazdym bloku
- Trwalosc: tak
- Relacja: `tip_hash` == hash bloku na `height`

### 8.7. Finalized head

- Unresolved: nie ma osobnego pola `finalized_height` ani `finalized_hash`
- W obecnej implementacji `height` == finalized head (kazdy zastosowany blok jest traktowany jako finalny)
- Jesli model finality jest bardziej zlozony (QC-gated), to finalized head nie jest jawnie rozdzielony od tip

### 8.8. QC / safety state

- `qcs`: `LedgerSnapshot.qcs: BTreeMap<u64, QuorumCertificate>` — mapuje height na QC
  - Dodawanie: `finalize_block_with_qc` w `node.rs:571` — wstawia QC i flushuje
  - Uzywane przez state_sync (serwowanie QC w `SyncResponse`)
  - Trwalosc: tak, jako czesc snapshotu
  - Brak eviction — `qcs` rosnie bez limitu (unresolved)
- `consensus_safety`: `LedgerSnapshot.consensus_safety: ConsensusSafetyState`
  - Aktualizacja: `Ledger::update_consensus_safety()` — zapisuje caly snapshot
  - Wywolywana z `PrivaiNode`: przed vote send (`last_voted_view`) i po view change quorum (`current_round`)
  - Czesciowo maintained — patrz sekcja 10.3

### 8.9. Last persisted but not fully indexed block

- Unresolved: nie ma mechanizmu "partially persisted block"
- `apply_block` jest atomic: validate → apply txs → update height/tip → insert block → save snapshot
- Nie ma etapu "persisted but not indexed" w obecnym modelu

## 9. Atomicity Groups

### 9.1. Block application (current canonical)

Operacja `apply_block` wykonuje nastepujace kroki w tej kolejnosci:

1. `validate_block(block, snapshot, ...)` — sprawdza block validity na klonie stanu
2. Dla kazdej tx w bloku: `apply_transaction(tx, height, snapshot)` — modyfikuje stan
3. `snapshot.height = block.header.height`
4. `snapshot.tip_hash = block.hash()`
5. `snapshot.blocks.insert(height, block)`
6. Eviction cache (jesli >128 blokow)
7. `mempool.remove_committed_block(txs)`
8. `store.save(snapshot)` — ZAPIS CALOŚCI

Current canonical: caly `LedgerSnapshot` jest zapisywany w jednej operacji save. Oznacza to, że dzisiaj wszystkie powyższe kroki dla wdrożenia bloku są zabezpieczone, ponieważ ewentualny crash przerywający procedurę po kroku 7. odrzuci wdrożony do pamięci blok poprzez ponowne podniesienie stanu z dysku na kolejnym starcie (nie zachowując zmutowanych danych w systemie lokalnym). Stan po zrzucie snapshot'a gwarantuje poprawną atomowość - jednak jest skrajnie kosztowny wydajnościowo dla postępującego rozmiaru drzewa ledgera.
- FileSystemStore: atomic temp+rename
- RocksDBStore: `put_cf` + `flush()` w jednym kluczu `ledger_state`.

### 9.2. Consensus safety update

Operacja `update_consensus_safety()`:
1. `snapshot.consensus_safety = new_state`
2. `store.save(snapshot)` — zapis calosci

Mechanizm ten gwarantuje twarde utrwalenie flag ochronnych consensusu, niemniej operuje on na ogromnych kosztach IO (wymusza na każdym przepięciu w View Change'ach lub utworzeniu Vote ponowne przeładowanie na dysk pełnego zrzutu całego `LedgerSnapshot`).
W warunkach produkcyjnych, zapis zapewnia 100% atomowość - state update + store flush jest scalony pod pojedynczym zapisem IO bez mutacji częściowych, co uniemożliwia equivocation lub podwójne podpisanie na wadliwych wybudzeniach po błędach hardwaru/crashach.

### 9.3. Atomicity guarantee

Current canonical: kazdy `save()` zapisuje caly snapshot jako jedna wartosc.
- FileSystemStore: temp+rename daje atomicnosc na poziomie FS
- RocksDBStore: `put_cf` + `flush()` daje atomicnosc na poziomie RocksDB write batch (ale jest to single key write)

## 10. Current Non-Conformities

### 10.1. Whole-snapshot serialization as single value

LedgerSnapshot jest zapisywany jako jeden JSON blob:
- RocksDB ma CF `blocks`, `notes`, `nullifiers`, `ticket_nullifiers`, `qcs` — ale NIE SA UZYWANE
- Wszystko idzie do CF `ledger_state` pod key `b"snapshot"`
- Rosniecy stan oznacza rosnacy JSON i coraz drozszy save/load
- To jest `Current non-conformity` wzgledem rozsadnego docelowego layoutu z CF-based key-value access

### 10.2. Unused RocksDB column families

RocksDBStore otwiera 6 column families:
- `ledger_state` (uzywana)
- `blocks` (nieuzywana)
- `notes` (nieuzywana)
- `nullifiers` (nieuzywana)
- `ticket_nullifiers` (nieuzywana)
- `qcs` (nieuzywana)

Pozostale 5 CF sa tworzone ale nie maja zadnego read/write path.
To jest `Current non-conformity` — marnotrawstwo zasobow i mylacy kod.

### 10.3. ConsensusSafetyState — partially maintained, incomplete contract

`ConsensusSafetyState` jest w `LedgerSnapshot` i jest persystowany — ale tylko w wybranych sciezkach:

**Co jest dzis persystowane:**
- `last_voted_view` — zapisywany w `node.rs:257` (metoda `create_vote_for_proposal`) przed wysylka vote. Cel: zapobieganie double-sign po restarcie.
- `current_round` — zapisywany w `node.rs:451` (obsługa view change quorum). Cel: zapobieganie equivocation po restarcie.
- Obie sciezki klonuja `consensus_safety`, modyfikuja jedno pole, i wywoluja `update_consensus_safety()` ktory zapisuje caly snapshot.

`current_round` po modyfikacji ładuje się bezpośrednio podczas startu węzła z zrzuconego pod `LedgerSnapshot` pola. Stanowi to zachowanie zgodne (Current Canonical).

**Co NIE jest dzis persystowane:**
- `locked_qc` — nigdy nie jest zapisywany. Po restarcie bedzie `None`.
- `current_view` — nie jest jawno aktualizowany w tych sciezkach.

**Czego brakuje do pelnego contractu:**
- Safety state nie jest periodicznie synchronizowany w glownym flow consensusu — jest zapisywany tylko "przy okazji" (vote creation, view change)
- Brak jawnej persist sciezki po kazdej rundzie consensusu
- `locked_qc` nie przetrwuje restartu — jesli model PC-BFT wymaga `locked_qc` for safety, to jest to luka (`Current non-conformity`).

### 10.4. Block cache eviction — local history availability

`blocks` w `LedgerSnapshot` jest recent-block cache (max 128 blokow).
- Bloki sa trwale zapisywane jako czesc `LedgerSnapshot` przy kazdym `save()` — jesli blok jest w `blocks` w momencie zapisu, przetrwa restart
- Eviction w pamieci: po `apply_block`, jesli `blocks.len() > 128`, bloki ponizej `height - 128` sa usuwane z pamieci
- Eviction nie kasuje blokow z dysku w biezacym save — kasuje je z pamieci, a nastepny `save()` zapisuje snapshot juz bez evictowanych blokow
- Brak osobnego archival block store — jesli blok zostal wyevictowany i zapisany snapshot juz go nie zawiera, jest nieodwracalnie tracony z lokalnego node'a

To jest `Current non-conformity` jesli node potrzebuje pelnej historii blokow lokalnie (np. archival mode, state sync od zera bez peerow).

### 10.5. Mempool is purely in-memory

Mempool nie jest czescia `LedgerSnapshot` ani `LedgerStore`.
Po restarcie mempool jest pusty — nie ma recovery.
To jest `Current canonical` behavior (mempool jest z definicji transient), ale nalezy to miec swiadomosc:
- transakcje zlozone przed restartem sa tracone
- nie ma persist ani recovery path dla mempoola
- jesli node wymaga mempool persistence, to jest to `Unresolved`

### 10.6. Lite outputs excluded from note_root

Zgodnie z `PRIVAI_PROTOCOL_CORE.md` §12:
- `Transaction::outputs()` zwraca pusto dla `LiteTransfer`
- `note_root()` opiera sie tylko o `tx.outputs()`
- Lite outputs nie sa trackowane w `LedgerSnapshot.notes` przez standardowy path
- To jest `Current non-conformity` opisana juz w protocol core

## 11. Unresolved Gaps

### 11.1. Finalized head vs tip

Nie ma jawnego rozroznienia miedzy `tip` (ostatni znany blok) a `finalized_head` (ostatni blok zatwierdzony przez QC).
W obecnym modelu kazdy `apply_block` traktuje blok jako finalny.
Jesli model finality bedzie QC-gated, trzeba dodac `finalized_height` i `finalized_hash`.

### 11.2. State_root computation timing

`state_root` jest obliczane:
- w `validate_block` na klonie stanu (do weryfikacji)
- w `compute_state_root` jako merkle root z notes + nullifiers
- na rozruchu podczas fazy inicjalizacyjnej bazy (Boot-time verification wewnątrz procedury `Ledger::open()`), zabezpieczając bazę node'a w locie, w przypadku uszkodzenia spójności (Mismatched Roots - `Current canonical`). Ograniczeniem boot-time weryfikacji w locie pozostaje przypadek braku buforowanego bloku na weryfikowanym statusie (weryfikacja `state_root` na zrzucie stanu uderza w dysk na boot-time, tylko pod warunkiem, że bufor `blocks` mapujący hash headerowy pod wskazanym kluczem z ID height jest tam osadzony).
- nie jest cache'owane — przeliczane za kazdym razem

Unresolved: czy `state_root` powinno byc computed-on-write (raz, przy apply_block) czy computed-on-read (za kazdym razem).

### 11.3. QC data lifecycle

`qcs` w `LedgerSnapshot` rosnie bez limitu.
Nie ma eviction ani cleanup strategii dla starych QC.
Dla state sync starsze QC moga byc potrzebne, ale nie ma granicy.

### 11.4. RocksDB CF layout future

5 z 6 CF jest nieuzywanych w pliku konfiguracyjnym store.rs. Stanowi to pozostałość (scaffolding) po początkowej wizji systemu z modelem wielokluczowym. W logice Ledger::open wszystkie te CF są pominięte poza ledger_state (a snapshot serializowany jest jako jeden spłaszczony JSON blob per RocksDB::put).
Current Canonical: należy zostawić ten model na razie jako główny mechanizm odtworzeniowy RocksDB, a po ustaleniu procedur dla refaktoru bazy stworzyć osobny "Migration Task" mający na celu oczyszczenie kodu z nieużywanych definicji CF lub zaadaptowanie go na przyrostowe pisanie (incremental writes) zgodnie z celami zapisanymi w `PRIVAI_CHAIN_STORAGE_AND_RECOVERY_NEXT_STEPS.md`. Nie należy przeprowadzać "cichego" usuwania tych CF teraz, jako że może zdestabilizować istniejące buildy.

### 11.5. Recovery after partial write

FileSystemStore: temp+rename daje atomicnosc na wiekszosci FS.
RocksDBStore: `put_cf` + `flush()` — atomic na poziomie single key.
Dopelniona weryfikacja logiki fail behavior w paczce `persistence_restart_tests.rs`: uszkodzony plik przerywa budowe Node'a bez modyfikowania istniejacego stanu i zwraca jasny błąd serializacji. Usuniety plik stanu uruchamia ledgera czysto z Genesis bloku.
Nadal brakuje:
- WAL-level recovery testow
- explicit repair/rebuild procedury po panic na starcie (np. spadek z archiwum block storage zamiast padniecia bez odtwarzania)

### 11.6. Ledger state vs node-local auxiliary state

Granica nie jest jawnie opisana:
- `LedgerSnapshot` trzyma `consensus_safety` — czy to ledger state czy consensus state?
- `qcs` sa w ledger snapshot — czy to ledger state czy consensus state?
- Vote tracking jest w `PrivaiNode` — nie jest zapisywany
- Proof artifacts sa w `ProofArtifactStore` — osobna warstwa

Unresolved: jasna granica miedzy tym, co nalezy do ledgera, a co do node-local auxiliary state.

### 11.7. Block store completeness

`blocks` w `LedgerSnapshot` jest jednoznacznie zdefiniowane na obecnym etapie projektu jako **recent-block cache** z evictowaniem starych danych (z reguły twardy limit = 128 bloków) i nie funkcjonuje jako węzeł archiwalny. Odpowiada za serwowanie bloków świeżo dołączającym do sieci walidatorom. Na dzisiaj zamyka to kontrakt węzła jako "full validator bez historii archiwalnej". Odtwarzanie logiki State Sync zależy od synchronizacji bieżącej tablicy bloków i zrzuconego stanu spłaszczonego w store, wymuszając w przyszłości budowę odrębnego Archival Node'a do przechowywania pełnej osi od Genesis.

## 12. Future Targets Requiring Migration

### 12.1. CF-based RocksDB layout

Docelowy model: kazdy typ danych w osobnej CF, z dedykowanymi key spaces:
- `CF_BLOCKS`: key = height, value = Block
- `CF_NOTES`: key = note_commit, value = NoteRecord
- `CF_NULLIFIERS`: key = nullifier, value = () (existence check)
- `CF_TICKET_NULLIFIERS`: key = nullifier, value = ()
- `CF_QCS`: key = height, value = QuorumCertificate
- `CF_LEDGER_STATE`: key = metadata (tip, height, chain_id, consensus_safety)

To wymaga migracji z obecnego single-key modelu.

### 12.2. Incremental state writes

Docelowy model: zamiast zapisywac caly snapshot, zapisywac tylko zmiany:
- nowe note'y
- nowe nullifiery
- nowy height/tip
- nowy blok
- updated consensus_safety

To wymaga transactional write batch w RocksDB.

### 12.3. Recovery and restart rules

Docelowy model: jawne reguly recovery opisane w `spec/PRIVAI_RECOVERY_AND_RESTART_RULES.md`:
- restart sequence
- state_root revalidation
- tip/height recovery
- safety state recovery
- index rebuild procedures

## 13. What Requires Explicit Freeze Update

Nastepujace zmiany nie moga byc robione po cichu:
- zmiana formatu persystencji (JSON → binary, single-key → CF-based),
- zmiana atomicity groups (co jest zapisywane razem),
- dodanie nowego pola do `LedgerSnapshot` (zmienia format),
- zmiana eviction strategii dla blokow,
- zmiana modelu `state_root` computation timing,
- wprowadzenie partial write / incremental persist,
- rozdzielenie `consensus_safety` z `LedgerSnapshot`,
- usuniecie lub przeksztalcenie unused CF w RocksDB.

## 14. Frozen Rules

- `LedgerSnapshot` jest jedyna jednostka stanu persistowanego — nie ma osobnych partial saves,
- `state_root` musi pasowac do `compute_state_root(snapshot)` po apply_block,
- anti-double-spend opiera sie o `spent_nullifiers` i `spent_ticket_nullifiers` w snapshot,
- `NoteRecord.status` jest source of truth dla note lifecycle,
- `height` + `tip_hash` sa source of truth dla pozycji chaina,
- mempool jest z definicji ephemeral.
