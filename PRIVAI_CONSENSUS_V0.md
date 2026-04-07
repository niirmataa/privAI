# privAI Consensus v0

Status: legacy consensus reference in migration.
Canonicality: non-canonical. Ten dokument nie moze nadpisywac canonical final spec set.
Owner: privAI consensus.
Depends on: `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`.
Superseded by: planowany `spec/PRIVAI_CONSENSUS.md`.

## 1. Cel

Ten dokument opisuje autorski szkic konsensusu dla `privAI`.

Nie idziemy w prosty klon gotowego BFT ani w czyste PoW.
Rdzen finality ma byc stake-based, ale blok ma byc `proof-carrying`, a ekonomia bloku ma uwzgledniac prace proverow.

Robocza nazwa:

- `PrivAI Proof-Carrying BFT`
- skrot: `PC-BFT`

## 2. Co zachowujemy z wczesniejszych zapiskow

To, co bylo mocne we wczesniejszym planie, zostaje:

- `FrodoKEM-640-SHAKE + Falcon-1024CT` dla transportu i autoryzacji,
- `UTXO / note-based ledger`,
- `escrow 2-of-3`,
- `marketplace local AI models`,
- `Tor / NXMS / store-and-forward`,
- `stake`, reputacja i odpowiedzialnosc operatorow.

Najwieksza korekta dotyczy samego konsensusu:

- nie budujemy czystego `PoUW` jako jedynego mechanizmu finality,
- budujemy wlasny `PC-BFT`,
- `PoPG` i inne useful-work mechanizmy wchodza jako warstwa pracy i nagrod, nie jako jedyny filar bezpieczenstwa sieci.

## 3. Zasada nadrzedna

Konsensus `privAI` sklada sie z czterech plaszczyzn:

1. `Safety plane`
   stake-weighted finality i quorum.
2. `Proof plane`
   blok niesie dowod poprawnosci przejsc stanu.
3. `Work plane`
   proverzy generuja useful work i dostaja nagrody.
4. `Threshold plane`
   Shamir obsluguje operacje progowe klucza sieciowego.

To daje wlasny system:

- finality nie zalezy od samej pracy proverow,
- blok nie jest poprawny bez proof-carrying execution,
- siec moze zyc, nawet jesli chwilowo jest malo proof jobs,
- useful work ma realna wartosc ekonomiczna.

## 4. Role

### 4.1. Validator

Validator:

- lockuje stake,
- uczestniczy w finality,
- utrzymuje pelny stan,
- weryfikuje tx, proofy i bloki,
- glosuje `prevote` i `precommit`,
- moze dodatkowo byc proverem.

### 4.2. Prover

Prover:

- rejestruje sie w sieci,
- przyjmuje prywatne `proof jobs`,
- generuje proofy dla `statement_commit`,
- publikuje `proof artifacts`,
- dostaje fee albo reward za useful work.

Prover nie musi byc walidatorem.

### 4.3. Threshold committee

Threshold committee:

- to podzbior albo caly validator set,
- trzyma Shamir shares klucza sieciowego,
- realizuje `DKG`, `reshare`, `threshold ops`,
- nie uczestniczy w zwyklej autoryzacji user tx.

### 4.4. Operator modelu

Operator modelu:

- rejestruje model w marketplace,
- stawia stake,
- przyjmuje zlecenia przez NXMS,
- korzysta z `privAI` do settlementu i escrow.

Operator nie musi byc walidatorem ani proverem, ale moze laczyc role.

## 5. Epochs

Siec dziala w epokach.

Kazda epoka zamraza:

- `validator_set`,
- `stake_weights`,
- `prover_registry`,
- `threshold_committee`,
- `epoch_params`,
- `epoch_seed`.

Przykladowy zestaw parametrow epoki:

```rust
struct EpochParams {
    epoch_number: u64,
    start_height: u64,
    end_height: u64,
    min_validator_stake: u64,
    min_prover_bond: u64,
    max_block_bytes: u32,
    max_block_statements: u32,
    min_proof_coverage: u32,
}
```

`epoch_seed` dla v0 jest liczony deterministycznie:

```text
epoch_seed = BLAKE3("privai:epoch-seed:v0" || last_epoch_qc_hash || epoch_number)
```

W kolejnych wersjach mozna zastapic to threshold beaconem opartym o Shamir.

## 6. Wybor proposera

W `privAI` proposer selection ma byc autorski, ale bez ryzykowania safety.

Dla kazdego walidatora liczymy score:

```text
score_i = stake_weight_i * availability_i * proof_score_i
```

Gdzie:

- `stake_weight_i` wynika ze stake,
- `availability_i` wynika z aktywnosci w poprzedniej epoce,
- `proof_score_i` wynika z historii poprawnie dostarczonych proofow.

Dla v0:

- `availability_i` i `proof_score_i` sa ograniczone do malego przedzialu,
- stake pozostaje glownym skladnikiem,
- safety nie zalezy od score, tylko od quorum `2/3`.

Round proposer:

```text
proposer = WeightedRoundRobin(epoch_seed, round, validator_scores)
```

To daje autorski mechanizm:

- nie czysty random,
- nie czysty round-robin,
- nie kopiowany 1:1 z innego systemu.

## 7. Obiekty konsensusu

### 7.1. BlockHeader

```rust
struct BlockHeader {
    version: u8,
    chain_id: u32,
    height: u64,
    epoch: u64,
    round: u32,
    timestamp_ms: u64,
    prev_block_hash: [u8; 32],
    tx_root: [u8; 32],
    note_root: [u8; 32],
    nullifier_root: [u8; 32],
    statement_root: [u8; 32],
    proof_cert_root: [u8; 32],
    proposer_pk_hash: [u8; 32],
    epoch_seed_hash: [u8; 32],
    parent_qc_hash: [u8; 32],
}
```

### 7.2. BlockBody

```rust
struct BlockBody {
    txs: Vec<TransferLikeTx>,
    execution_bundle: ExecutionBundle,
    proof_certificates: Vec<ProofCertificate>,
    extra_receipts: Vec<ConsensusReceipt>,
}
```

### 7.3. ExecutionBundle

`ExecutionBundle` spina transakcje i dowody.

```rust
struct ExecutionBundle {
    statement_commits: Vec<[u8; 32]>,
    covered_tx_indexes: Vec<u32>,
    public_inputs_root: [u8; 32],
    execution_mode: u8,
}
```

`execution_mode`:

- `0x01` full batch proof
- `0x02` multi-proof bundle
- `0x03` housekeeping / no-user-state-change

W v0 preferowany jest `0x01`.

### 7.4. ProofCertificate

```rust
struct ProofCertificate {
    proof_system_id: u8,
    statement_root: [u8; 32],
    public_inputs_root: [u8; 32],
    proof_bytes_hash: [u8; 32],
    prover_ids: Vec<[u8; 32]>,
    proof_meta_hash: [u8; 32],
}
```

Blok moze nie przenosic pelnych proof bytes w header.
Moze niesc:

- hash proofa,
- reference do proof artifact,
- albo same proof bytes w body.

### 7.5. Vote i QC

```rust
struct Vote {
    height: u64,
    round: u32,
    block_hash: [u8; 32],
    vote_type: u8, // 0x01 prevote, 0x02 precommit
    validator_pk: Vec<u8>,
    falcon_sig: Vec<u8>,
}

struct QuorumCertificate {
    height: u64,
    round: u32,
    block_hash: [u8; 32],
    vote_type: u8,
    signers: Vec<Vec<u8>>,
    signatures: Vec<Vec<u8>>,
}
```

W v0 nie potrzebujemy od razu agregacji podpisow.
Mozemy trzymac pelny zestaw sygnatur Falcon.

## 8. Warunek poprawnosci bloku

Blok jest wazny tylko jesli:

1. `prev_block_hash` wskazuje prawidlowego rodzica.
2. proposer byl uprawniony dla danego `(epoch, round)`.
3. wszystkie transakcje sa syntaktycznie poprawne.
4. wszystkie `nullifier` sa nowe.
5. `statement_root` odpowiada transakcjom.
6. `proof_certificates` pokrywaja wymagane przejscia stanu.
7. proof verification przechodzi.
8. finality glosy osiagaja quorum `>= 2/3 stake weight`.

To jest esencja `proof-carrying block`.

## 9. Round flow

### 9.1. Proposal

Proposer:

1. pobiera tx z mempoola,
2. pobiera proof artifacts od proverow,
3. buduje `ExecutionBundle`,
4. konstruuje kandydacki blok,
5. podpisuje header Falconem,
6. rozglasza `Proposal`.

### 9.2. Prevote

Kazdy walidator:

1. sprawdza uprawnienie proposera,
2. waliduje tx,
3. waliduje proof coverage,
4. waliduje proof certificate,
5. jesli wszystko jest poprawne, wysyla `Prevote`.

### 9.3. Precommit

Po zobaczeniu `>= 2/3` prevote:

1. walidator lockuje block hash dla `(height, round)`,
2. wysyla `Precommit`.

### 9.4. Commit

Po `>= 2/3` precommit:

1. tworzony jest `QuorumCertificate`,
2. blok jest finalny,
3. stan zostaje zapisany,
4. rewardy sa naliczane.

## 10. PoPG w konsensusie

`Proof of Proof Generation` nie jest osobnym meta-dowodem.
W `privAI` oznacza rynek useful work:

- uzytkownik albo builder tworzy `statement_commit`,
- witness trafia prywatnie do provera przez NXMS,
- prover buduje proof,
- proposer umieszcza proof certificate w bloku,
- siec weryfikuje proof,
- prover dostaje zaplate.

### 10.1. Proof job

```rust
struct ProofJob {
    job_id: [u8; 32],
    statement_commit: [u8; 32],
    job_fee: u64,
    deadline_height: u64,
    requester_hint: [u8; 32],
}
```

### 10.2. Proof artifact

```rust
struct ProofArtifact {
    job_id: [u8; 32],
    statement_commit: [u8; 32],
    proof_system_id: u8,
    proof_bytes: Vec<u8>,
    public_inputs_hash: [u8; 32],
    prover_pk_hash: [u8; 32],
}
```

### 10.3. Proof claim

```rust
struct ProofRewardClaim {
    job_id: [u8; 32],
    proof_bytes_hash: [u8; 32],
    prover_pk: Vec<u8>,
    falcon_sig: Vec<u8>,
}
```

## 11. Shamir w konsensusie

Shamir nie zastepuje finality.
Jest osobna warstwa progowa.

Zakres w konsensusie:

- `DKG` dla `pk_net`,
- `reshare` przy zmianie setu lub epoki,
- threshold ops dla systemowych operacji na nocie / refresh / migration,
- przyszly beacon losowosci.

Mozliwe rozszerzenie:

- epoch randomness z threshold beacon,
- threshold attestation dla szczegolnych operacji sieciowych.

Niewlasciwe zastosowania:

- zwykle user tx,
- zwykle glosy blokowe,
- escrow multisig.

## 12. Reward model

W `privAI` blok ma trzy klasy beneficjentow:

- proposer,
- validatorzy,
- proverzy.

### 12.1. Blokowa nagroda bazowa

Proponowany v0 split:

- `50%` validator pool
- `20%` proposer bonus
- `30%` prover pool

### 12.2. Fees

Proponowany split fee:

- `40%` proposer
- `40%` proof producers
- `20%` validator pool

W kolejnych wersjach procenty mozna stroic przez governance lub epoch params.

### 12.3. Proof coverage bonus

Jesli proposer dostarcza blok z wysokim `proof coverage` i dobrym `proof latency`, moze dostac dodatkowy bonus.

To wzmacnia charakter useful work bez ryzykowania safety.

## 13. Slashing

Slashable offenses:

### Validator

- double proposal w tej samej rundzie,
- double vote na konfliktujace bloki,
- podpisanie niepoprawnego QC,
- uporczywa nieaktywnosc.

### Prover

- swiadome skladanie falszywych proof claims,
- proof substitution po podpisanym claim,
- spamowanie niepoprawnymi artifacts przy aktywnym bondzie.

### Threshold member

- publikacja niepoprawnego partial share,
- podpisanie niepoprawnej threshold operacji,
- niewykonywanie resharing duties.

## 14. Liveness

Siec musi dzialac nawet przy slabym rynku proverow.

Dlatego v0 ma trzy zasady:

1. jesli nie ma proofa dla user-state transition, taka tx nie wchodzi do bloku,
2. proposer nadal moze emitowac pusty albo housekeeping block,
3. finality nie zalezy od stalej dostepnosci proverow.

To chroni liveness chaina.

## 15. Housekeeping blocks

Nie kazdy blok musi niesc user payments.

Dozwolone sa bloki:

- z samymi governance / staking ops,
- z epoch receipts,
- z threshold maintenance receipts,
- z pustym execution bundle.

Warunek:

- `execution_mode = 0x03`
- brak user-state transitions wymagajacych proofa.

## 16. Network messages

Nad warstwa P2P potrzebujemy co najmniej:

```rust
enum ConsensusMsg {
    Proposal,
    Prevote,
    Precommit,
    QuorumCert,
    NewRound,
    ProofOffer,
    ProofAccept,
    ProofReject,
}
```

Uwagi:

- glosy blokowe ida przez siec walidatorow,
- witnessy i proof jobs ida prywatnie przez NXMS,
- mailbox i store-and-forward sa pomocnicze, nie sa zrodlem finality.

## 17. Dlaczego to jest autorskie

`privAI` nie kopiuje gotowego schematu 1:1.

Wlasne elementy:

- `proof-carrying block` jako warunek waznosci bloku,
- osobna ekonomia proverow,
- proposer score zalezne od stake, availability i proof performance,
- zintegrowana warstwa threshold z kluczem sieciowym,
- zgodnosc z hidden notes, LWE amounts i marketplace AI.

To nie jest klasyczny chain z doklejonym ZK.
To jest blockchain, w ktorym proof execution jest czescia struktury bloku.

## 18. Roadmapa wdrozenia

### v0-a

- staly validator set
- `PC-BFT` w uproszczonej wersji
- proof verification per block
- bez dynamicznego prover market

### v0-b

- stake-weighted validator set
- prover registry i prover rewards
- epoch params i proposer score

### v1

- threshold beacon losowosci
- lepsze proof batching
- slashing dla proverow i threshold committee

### v2

- eksperymentalne AI-work score
- mozliwy wplyw benchmark useful work na ekonomie epoki
- dalsza integracja z marketplace metrics

## 19. Konkluzja

Docelowy konsensus `privAI` to:

- `stake-based BFT` dla finality,
- `proof-carrying execution` dla poprawnosci stanu,
- `PoPG` dla useful work i nagrod,
- `Shamir` dla operacji progowych i klucza sieciowego.

To daje autorski protokol, ktory jest bezpieczniejszy od czystego PoUW jako jedynego filaru, ale nadal zachowuje Wasza wizje wlasnej sieci, wlasnych proofow i wlasnej ekonomii pracy.
