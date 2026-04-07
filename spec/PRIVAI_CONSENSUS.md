# privAI Consensus

Status: draft canonical consensus doc in migration.
Canonicality: intended canonical source of truth for consensus semantics, block validity and finality rules. Product-level policy remains governed by `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`. Protocol-core semantics remain governed by `spec/PRIVAI_PROTOCOL_CORE.md`. Exact bytes remain governed by `spec/PRIVAI_CANONICAL_FORMATS.md`. Pliki poza `spec/` nie sa normatywnym source of truth.
Owner: privAI consensus.
Depends on: `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`, `spec/PRIVAI_PROTOCOL_CORE.md`, `spec/PRIVAI_CANONICAL_FORMATS.md`, `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`.
Supersedes: `PRIVAI_CONSENSUS_V0.md` na poziomie finalnej semantyki consensus.

## 1. Cel

Ten dokument opisuje finalny target i aktualna semantyke consensus layer systemu `privAI`:
- model finality,
- role validator/prover/operator,
- kanoniczne obiekty consensus,
- reguly walidacji blokow,
- state commitments,
- relacje miedzy consensus, ledger i proof plane,
- rozdzial miedzy stanem finalnym a stanem obecnej implementacji.

To nie jest dokument produktu.
To nie jest tez dokument semantyki not i walleta.
Exact field bytes dla obiektow consensus naleza do `spec/PRIVAI_CANONICAL_FORMATS.md`.

Ten dokument ma odpowiedziec jasno:
- kiedy blok jest poprawny,
- co consensus musi egzekwowac,
- jaka jest relacja miedzy proof-carrying execution a stake-based finality,
- co jest juz zamrozonym kierunkiem,
- a co nadal wymaga domkniecia implementacyjnego.

## 1.1. How Devs And Agents Should Use This Doc

Ten dokument jest zrodlem prawdy dla consensus semantics, block validity i finality modelu.

Czytaj go razem z:
- `spec/PRIVAI_SPEC_INDEX.md` jako punktem wejscia do calego frozen setu,
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md` dla polityki raili i progow,
- `spec/PRIVAI_PROTOCOL_CORE.md` dla znaczenia tx classes i state objects,
- `spec/PRIVAI_CANONICAL_FORMATS.md` dla exact bytes consensus objects,
- `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md` dla marketplace-specific settlement semantics.

Zamrozona zasada pracy:
- jesli task dotyczy block validity, state commitments, proof coverage, rail enforcement albo finality, to ten dokument jest punktem wyjscia,
- consensus egzekwuje tylko jawnie committed rules, a nie domysly produktowe,
- kod consensus, ledger i proof plane poza `spec/` jest referencja implementacyjna, a nie rownorzednym source of truth,
- jesli kod rozjezdza sie z tym dokumentem, nalezy traktowac to jako implementation gap albo `current non-conformity`, a nie jako powod do lokalnej reinterpretacji consensus.

Jawnego freeze update wymagaja zawsze:
- zmiana block validity rules,
- zmiana state commitments,
- zmiana finality modelu,
- zmiana rail enforcement rules,
- zmiana proof coverage semantics per tx class,
- zmiana authority modelu dla marketplace settlement.

Interpretation rule for this document:
- status labels sa zdefiniowane w `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`,
- jesli consensus rule nie ma jawnej etykiety statusu albo jawnej formuly, nalezy traktowac ja jako `unresolved`,
- `unresolved` nie wolno wypelniac przez zgadywanie block validity, proof coverage, threshold enforcement ani authority checks.

## 2. Finalny model consensus

Finalnym kierunkiem `privAI` pozostaje autorski model:
- stake-based finality,
- proof-carrying execution,
- osobna warstwa pracy proverow,
- osobna warstwa threshold operations.

Robocza nazwa z wczesniejszych docs pozostaje trafnym opisem kierunku:
- `PrivAI Proof-Carrying BFT`
- skrot: `PC-BFT`

Zamrozona interpretacja:
- `privAI` nie jest czystym PoW,
- `privAI` nie jest czystym useful-work-finality chainem,
- finality nie moze opierac sie tylko o prace proverow,
- blok nie moze byc uznany za poprawny bez proof-carrying execution,
- useful work pozostaje warstwa ekonomiczna i operacyjna, a nie jedyny filar safety.

## 3. Cztery plaszczyzny consensus

Consensus `privAI` sklada sie z czterech plaszczyzn:

1. `Safety plane`
   stake-weighted finality, prevote/precommit oraz quorum.
2. `Proof plane`
   blok niesie commitments i certyfikaty potrzebne do proof-carrying execution.
3. `Work plane`
   proverzy generuja proof artifacts i pobieraja za to fee lub reward.
4. `Threshold plane`
   committee wykonuje threshold operations, DKG i reshare dla kluczy sieciowych.

Zamrozona zasada:
- oslabienie jednej plaszczyzny nie moze po cichu zmieniac wymagan pozostalych,
- proof plane nie zastepuje safety plane,
- safety plane nie usuwa potrzeby poprawnych proof commitments,
- threshold plane nie jest zwyklym auth layer user tx.

## 4. Role consensus

### 4.1. Validator

Validator:
- lockuje stake,
- utrzymuje pelny stan,
- waliduje transakcje, proof artifacts i bloki,
- bierze udzial w `prevote` i `precommit`,
- moze przechowywac `locked_qc` i lokalny safety state,
- moze dodatkowo pelnic role provera.

### 4.2. Prover

Prover:
- przyjmuje proof jobs,
- generuje proofy dla `statement_commit`,
- publikuje proof artifacts albo proof certificates,
- uczestniczy w work plane,
- nie musi byc walidatorem.

### 4.3. Threshold committee

Threshold committee:
- trzyma shares klucza sieciowego,
- realizuje `DKG`, `reshare` i threshold ops,
- nie zastepuje zwyklej autoryzacji user tx,
- nie jest synonimem validator setu, choc moze byc jego podzbiorem albo pokrywac sie z nim.

### 4.4. Marketplace operator

Marketplace operator:
- nie jest automatycznie walidatorem ani proverem,
- moze publikowac `MarketplaceBatchTx`,
- dziala w ramach marketplace raila,
- podlega consensus przez shape validation, auth checks i nullifier-spend rules,
- nie otrzymuje specjalnego "bypassu" consensus tylko dlatego, ze jest operatorem.

## 5. Epochs i parametry systemowe

Siec dziala w epokach.

Kazda epoka zamraza co najmniej:
- `validator_set`,
- `stake_weights`,
- `prover_registry`,
- `threshold_committee`,
- `epoch_params`,
- `epoch_seed`.

Aktualny kanoniczny obiekt z kodu:

```rust
struct EpochParams {
    epoch_number: u64,
    start_height: u64,
    end_height: u64,
    min_validator_stake: u64,
    min_prover_bond: u64,
    min_fee: u64,
    max_block_bytes: u32,
    max_block_statements: u32,
    min_proof_coverage: u32,
}
```

Semantyka finalna:
- `min_validator_stake` i `min_prover_bond` naleza do polityki wejscia do odpowiednich rol,
- `min_fee` jest egzekwowany przez ledger przy walidacji tx,
- `max_block_bytes` jest egzekwowany na canonical bytes bloku,
- `max_block_statements` ogranicza liczbe tx w bloku,
- `min_proof_coverage` okresla minimalna wymagana obecnosc proof certificates zgodnie z przyjeta polityka execution plane.

## 6. Proposer selection i round model

Finalny kierunek pozostaje epoch-seeded i stake-aware.

Zamrozona semantyka:
- proposer selection nie moze podwazyc safety quorum,
- round model musi wspierac timeout i `ViewChange`,
- liveness nie moze zalezec od jednego stalego proposera,
- proposer selection moze uwzgledniac stake, availability i proof history, ale safety nadal zalezy od quorum, a nie od scoringu.

Stan obecny:
- kod ma juz `round`, `epoch_seed_hash`, `Vote`, `ViewChange` i `QuorumCertificate`,
- dokladna finalna formula weighted proposer rotation nie jest jeszcze osobno zamrozona w kanonicznym dokumencie.

## 7. Kanoniczne obiekty consensus

### 7.1. ExecutionMode

Kanoniczne tryby execution plane:
- `FullBatchProof = 0x01`
- `MultiProofBundle = 0x02`
- `Housekeeping = 0x03`

Interpretacja:
- `FullBatchProof` jest preferowanym trybem dla zwyklego batch execution,
- `MultiProofBundle` dopuszcza wiecej niz jeden proof artifact,
- `Housekeeping` sluzy blokom, ktore nie niosa user-state-changing proof workload.

### 7.2. ExecutionBundle

`ExecutionBundle` spina warstwe tx z proof plane.

Kanoniczne pola:
- `statement_commits`
- `covered_tx_indexes`
- `public_inputs_root`
- `execution_mode`

Semantyka finalna:
- bundle mowi, ktore tx sa pokryte przez proof plane,
- `public_inputs_root` wiaze public inputs do execution batcha,
- consensus nie powinien akceptowac rozjazdu miedzy `statement_commit` tx a execution bundle.

### 7.3. ProofCertificate

`ProofCertificate` jest kanonicznym bytem reprezentujacym wynik proof plane.

Kanoniczne pola:
- `proof_system_id`
- `statement_root`
- `public_inputs_root`
- `proof_bytes_hash`
- `prover_ids`
- `proof_meta_hash`

Semantyka finalna:
- certificate nie musi niesc pelnych proof bytes,
- ale consensus musi miec commitment do artefaktu i public inputs,
- proof verifier musi miec mozliwosc oceny poprawnosci certificate coverage dla bloku.

### 7.4. ConsensusReceipt

`ConsensusReceipt` jest dodatkowym receipt-em consensus layer.

Stan obecny:
- ma pola `receipt_type` i `payload_hash`.

Interpretacja:
- to byt pomocniczy consensus layer,
- nie jest tym samym co `Receipt` z marketplace raila,
- nie wolno mieszac tych dwoch semantyk.

### 7.5. BlockHeader

Kanoniczny header bloku z kodu zawiera:
- `version`
- `chain_id`
- `height`
- `epoch`
- `round`
- `timestamp_ms`
- `prev_block_hash`
- `tx_root`
- `note_root`
- `nullifier_root`
- `statement_root`
- `proof_cert_root`
- `state_root`
- `proposer_pk_hash`
- `epoch_seed_hash`
- `parent_qc_hash`

Zamrozona interpretacja:
- `state_root` nalezy do finalnej semantyki headera,
- nie wolno wracac do starszych opisow, ktore pomijaly `state_root`,
- header hash jest nadrzednym identyfikatorem bloku.

### 7.6. BlockBody i Block

`BlockBody` zawiera:
- `txs`
- `execution_bundle`
- `proof_certificates`
- `extra_receipts`

`Block` sklada sie z:
- `header`
- `body`

Semantyka finalna:
- body niesie wszystko, co jest potrzebne do odtworzenia commitments z headera,
- blok jest poprawny tylko wtedy, gdy jego roots i state commitments zgadzaja sie z body i wynikiem wykonania.

### 7.7. Vote, ViewChange i QuorumCertificate

`Vote`:
- identyfikuje `height`, `round`, `block_hash`, `vote_type`,
- niesie `validator_pk` i `falcon_sig`.

`ViewChange`:
- identyfikuje `height` i `new_round`,
- niesie auth validatora,
- sluzy liveness path po timeout.

`QuorumCertificate`:
- identyfikuje `height`, `round`, `block_hash`, `vote_type`,
- niesie liste signerow i signatures.

Zamrozona semantyka:
- `Prevote` i `Precommit` pozostaja kanonicznymi etapami glosowania,
- `QC` jest materializowanym wynikiem quorum,
- `ViewChange` pozostaje jawna sciezka timeout/liveness.

### 7.8. ConsensusMsg

Kanoniczny envelope P2P obejmuje co najmniej:
- `Proposal`
- `Prevote`
- `Precommit`
- `QuorumCert`
- `ViewChange`
- `Ping`
- `SyncRequest`
- `SyncResponse`
- `Gossip`
- `GetPeers`
- `PeersList`

Semantyka finalna:
- message layer nalezy do consensus i state sync,
- `Gossip` jest transportem tx, nie finality vote,
- `SyncResponse` moze niesc bloki i QCs,
- peer discovery nie moze podwazac auth wymagan consensus.

## 8. State commitments

### 8.1. tx_root

`tx_root`:
- jest merkle root z `Transaction::tx_id()` dla tx w bloku.

### 8.2. note_root

Docelowa semantyka:
- `note_root` ma byc commitmentem do wszystkich note outputs, ktore consensus uznaje za czesc canonical state transition.

Stan obecny kodu:
- `note_root()` liczy root tylko po `tx.outputs()`,
- czyli obecnie nie obejmuje lite outputs z `LiteTransfer`.

Interpretacja:
- to jest obecna luka implementacyjna,
- nie jest to finalna semantyka dla przyszlego zamrozonego `OnChainLite`,
- finalny `OnChainLite` nie moze wejsc do pelnego freeze bez pelnego spiecia z `note_root`, `state_root` i `execution bundle`.

### 8.3. nullifier_root

`nullifier_root`:
- jest merkle root z `input_nullifiers`,
- daje publiczny commitment do note spends w bloku.

### 8.4. statement_root

`statement_root`:
- jest rootem `execution_bundle.statement_commits`, jesli bundle je podaje,
- w przeciwnym razie jest rootem `Transaction::statement_commit()`.

### 8.5. proof_cert_root

`proof_cert_root`:
- jest merkle root z hashy `ProofCertificate`.

### 8.6. state_root

`state_root`:
- jest commitmentem do stanu po wykonaniu bloku,
- w aktualnym ledgerze wynika z:
  - note commits + ich status,
  - spent note nullifiers,
  - spent ticket nullifiers.

Zamrozona interpretacja:
- consensus nie moze przyjac bloku, ktorego `state_root` nie zgadza sie z wynikiem wykonania na lokalnym snapshot.

## 9. Reguly walidacji blokow

Finalna semantyka walidacji bloku obejmuje co najmniej:
- zgodnosc `chain_id`,
- poprawna wysokosc i parent linkage,
- zgodnosc z granicami aktywnej epoki,
- limit liczby tx w bloku,
- limit canonical block size,
- zgodnosc wszystkich rootow z body,
- poprawnosc proof plane,
- poprawnosc state transition po zastosowaniu tx,
- egzekwowanie finalnych zasad tx validation.

Stan obecny kodu egzekwuje w szczegolnosci:
- `chain_id == snapshot.chain_id`
- `height == snapshot.height + 1`
- `prev_block_hash == snapshot.tip_hash`
- `height <= epoch_params.end_height`
- `block.body.txs.len() <= epoch_params.max_block_statements`
- `block.to_canonical_bytes().len() <= epoch_params.max_block_bytes`
- `block.roots_match()`
- `proof_verifier.verify_block(block)`
- `computed_state_root == block.header.state_root`

## 10. Reguly walidacji transakcji przez consensus/ledger

Consensus opiera sie na walidacji tx wykonywanej przez ledger/state machine.

Kanoniczne reguly stanu obecnego obejmuja:
- `tx.validate_shape()`
- egzekwowanie `min_fee`
- dla zwyklych note-based tx:
  - brak duplicate inputs,
  - brak duplicate input nullifiers,
  - input musi istniec w snapshot,
  - input nie moze byc juz spent,
  - output note_commit nie moze kolidowac ze stanem,
  - auth entries sa weryfikowane przez Falcon, jesli sa obecne
- dla `MarketplaceBatchTx`:
  - duplicate `ticket_nullifier` sa zakazane,
  - ticket nullifier nie moze byc juz spent,
  - operator signature jest sprawdzana, jesli auth material jest obecny.

Zamrozona interpretacja:
- consensus nie moze "omijac" ledger validation dla zadnego raila,
- marketplace rail nie dostaje slabszych zasad double-spend protection,
- `MarketplaceBatchTx` w modelu finalnym zawsze wymaga operator auth i settlement authority verification,
- obecny warunkowy auth-path dla marketplace batcha pozostaje current non-conformity, a nie finalny stan,
- auth-path moze byc przejsciowo niepelny, ale nie wolno tego nazywac finalnym stanem bez jawnego freeze update.

### 10.1. Rail enforcement rules

Consensus egzekwuje tylko jawnie committed rules.

Zamrozona zasada:
- product layer klasyfikuje flow,
- policy layer commitmentuje te reguly,
- consensus egzekwuje tylko to, co wynika z:
  - tx class,
  - committed policy flags,
  - jawnych pol/rules w danych transakcji albo policy,
- consensus nie zgaduje semantyki produktu poza committed data model.

Finalne reguly rail enforcement:
- `FullPrivacy` pozostaje dozwolone dla dowolnej kwoty,
- `OnChainLite` nie moze byc uznany za finalny rail powyzej `MAX_LITE_TX_AMOUNT_PVA`,
- `MarketplaceBatchTx` nie podlega tej samej threshold rule co `OnChainLite`, bo jest osobna klasa marketplace settlement,
- flow-sensitive exceptions moga byc egzekwowane tylko wtedy, gdy wynikaja z committed tx class lub policy rules.

### 10.2. Proof coverage by tx class

Finalna zasada:
- kazda klasa tx musi miec jednoznacznie zdefiniowany model proof coverage,
- zadna klasa nie moze pozostac w stanie "istnieje w kodzie, ale proof plane nie wie co z nia zrobic".

Aktualna macierz interpretacyjna:

| Tx class | Rail | `statement_commit` | Proof coverage final target | Current status |
|----------|------|--------------------|-----------------------------|----------------|
| `TransferNoteTx` | `FullPrivacy` | wymagany | proof-covered | semantycznie glowna klasa proof-covered |
| finalny tx `OnChainLite` | `OnChainLite` | do jawnego zamrozenia | do jawnego zamrozenia osobna decyzja | nadal `experimental`, nie moze byc pol-finalny |
| `MarketplaceBatchTx` | `MarketplaceSmallPaymentsRail` | obecny w `TxCore`, ale nie sluzy tej samej proof semantyce co `FullPrivacy` | auth/nullifier/batch checks, nie ten sam model co `FullPrivacy` | semantycznie odrebny od lite p2p |
| `SettlementTx` / `ModelTx` / `StakeTx` | provisional | provisional | do osobnych docs | provisional |

## 11. Finality i safety state

Finalny model safety pozostaje oparty o:
- `prevote`
- `precommit`
- `QuorumCertificate`
- `locked_qc`
- view/round progression z timeout path.

Stan obecny persystuje w ledger state:
- `current_view`
- `last_voted_view`
- `current_round`
- `locked_qc`

Zamrozona zasada:
- safety state nalezy do canonical consensus state,
- state sync musi umiec odtworzyc nie tylko bloki, ale tez relewantne `QuorumCertificate`,
- nie wolno redukowac finality do samego "mamy blok i proof", bez jawnej warstwy quorum/finality.

## 12. Marketplace rail w consensus

`MarketplaceBatchTx` jest normalna kanoniczna klasa tx consensus, ale ma inna semantyke stanu niz note spends.

Consensus musi dla tego raila co najmniej:
- shape-validate batch,
- egzekwowac brak duplicate `ticket_nullifier`,
- odrzucac replay juz-spent `ticket_nullifier`,
- zapisac zuzycie `ticket_nullifier` do canonical state,
- uwzgledniac batch w `tx_root` i `state_root`.

Stan obecny:
- ledger juz spala `ticket_nullifier`,
- `compute_state_root()` uwzglednia spent ticket nullifiers,
- finalne checks summary/licznikow/operator auth sa jeszcze do domkniecia.

## 13. Current non-conformities wzgledem finalnego consensus

Najwazniejsze obecne niezgodnosci:
- `note_root()` obejmuje tylko `tx.outputs()`, a nie pelny zbior output classes przyszlego systemu,
- lite path nie jest jeszcze spiety end-to-end z canonical state commitments,
- `build_execution_bundle_from_transactions()` traktuje `LiteTransfer` jako proof-requiring, ale `public_inputs_hash_for_transaction()` nie umie jeszcze dla niego policzyc public inputs hash,
- walidacja proof coverage liczy obecnie tylko `TransferNote`, nie odzwierciedlajac jeszcze finalnego stanu wszystkich raili,
- operator signature i checks summary w marketplace railu pozostaja czesciowo przejsciowe,
- exact final proposer scoring formula nie zostala jeszcze osobno zamrozona.

## 14. Co trzeba zrobic, aby consensus byl pelnie finalny

- domknac `spec/PRIVAI_CANONICAL_FORMATS.md` dla wszystkich consensus objects i commitments,
- dopiac finalny model `OnChainLite` do `note_root`, `state_root`, execution bundle i proof coverage,
- zsynchronizowac batch builder, proof coverage i ledger validation dla wszystkich tx classes,
- dopisac finalne checks dla `MarketplaceBatchTx` summary, licznikow i operator auth,
- zamrozic finalna proposer selection formule albo jawnie wyniesc ja do osobnego parametryzowanego dokumentu consensus policy,
- dopisac golden vectors i canonical tests dla block/consensus objects,
- zsynchronizowac state sync i QC persistence z finalna semantyka safety state.

## 15. Twarde zasady dla dalszych zmian

- nie wolno zmieniac semantyki block validity bez update canonical consensus docs,
- nie wolno zmieniac state commitments przez sam kod bez jawnej decyzji spec,
- nie wolno usuwac `state_root` z finalnego header modelu,
- nie wolno oslabic quorum-based finality pod pretekstem useful work,
- nie wolno oslabic proof-carrying execution pod pretekstem szybszego throughputu,
- nie wolno mieszac `Receipt` marketplace z `ConsensusReceipt`,
- nie wolno traktowac docs spoza canonical final spec set jako rownorzednego zrodla prawdy dla consensus.
