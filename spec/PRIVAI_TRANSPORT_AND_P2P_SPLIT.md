# privAI Transport And P2P Split

Status: focused architecture target for transport, validator session networking and consensus overlay.
Canonicality: binding architectural split for `nxms-transport`, validator session transport and `privai-node` networking responsibilities. This document does not override canonical product, protocol, formats, marketplace or consensus semantics; it defines where the network stack boundaries live and how future refactors must preserve them.
Owner: privAI networking and transport architecture.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md`
- `spec/PRIVAI_DECISION_REGISTER.md`
- `spec/PRIVAI_GAP_REGISTER.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- rozdzielic escrow/control-plane transport od validator P2P,
- zatrzymac mieszanie packet transportu z live consensus networking,
- zapisac jeden docelowy podzial odpowiedzialnosci zanim ruszy refactor,
- uniknac dalszego duplikowania session crypto w `privai-node` i `nxms-transport`.

To nie jest dokument o formatach kanonicznych transakcji.
To nie jest tez dokument o finality ani semantyce blokow.
To jest dokument o granicach warstw sieciowych.

## 2. Status interpretacji

### Frozen

- `nxms-transport` nie jest docelowym wire protocol dla validator consensus messaging.
- escrow/control-plane i validator P2P to dwa rozne transport problems i nie wolno ich scalac w jeden protokol "dla wygody".
- `privai-node` ma pozostac wlascicielem semantyki consensus/gossip/sync, a nie niskopoziomowych prymitywow packet crypto dla escrow.

### Current canonical

- `nxms-transport` zawiera dzis:
  - `wire`
  - `crypto`
  - `tor_net`
  - `peers`
- `privai-node` uzywa dzis z `nxms-transport` glownie:
  - `tor_net`
  - `peers`
  - prymitywow z `crypto`
- `privai-node::net` ma juz zalazek wlasnej warstwy validator session:
  - `HandshakeMsg`
  - `ConnectionPool`
  - encrypted frames
  - rate limiting
  - ban list
  - reconnect/maintenance

### Future target requiring migration

- validator session transport powinien zostac wydzielony z `privai-node::net` do osobnej warstwy:
  - preferowany kierunek: nowy crate `privai-p2p`
  - dopuszczalny etap przejsciowy: `nxms-transport::session`

### Forbidden inference

- nie wolno zakladac, ze `NXMS/1`, `NXMS/2`, `NxmsEnvelope*` albo `SealedPacket` sa docelowym validator wire protocol.
- nie wolno mieszac `ConsensusMsg` z escrow packet transport tylko dlatego, ze oba ida przez Tor i uzywaja Falcon/Frodo.
- nie wolno przenosic logiki consensus/gossip/sync do `nxms-transport`.

## 3. Current code map

### 3.1. `nxms-transport`

Aktualne moduly:
- `wire.rs`
  - `NXMS/1`
  - `NXMS/2`
  - `NxmsEnvelope`
  - `NxmsEnvelopeV2`
  - `TxSignReqBody`
  - `TxSignRespBody`
  - `ContractPropose`
  - `ContractSig`
  - `EscrowBody`
- `crypto.rs`
  - `SealedPacket`
  - packet encrypt/decrypt
  - Falcon sign/verify
  - FrodoKEM encap/decap
  - XChaCha20-Poly1305 helpers
- `tor_net.rs`
  - framed TCP helpers
  - SOCKS5h/Tor connect
- `peers.rs`
  - `Peer`
  - `PeerBook`

Interpretacja:
- crate jest dzis mieszany:
  - escrow wire
  - packet crypto
  - generic network helpers

### 3.2. `privai-node`

Aktualne moduly:
- `net.rs`
  - validator handshake
  - connection pool
  - encrypted frame transport
  - rate limiter
  - ban list
  - maintenance
- `gossip.rs`
  - tx gossip semantics
  - fanout
  - loop/hops policy
- `state_sync.rs`
  - sync request/response semantics
  - import path
- `consensus_loop.rs`
  - `Proposal`
  - `Prevote`
  - `Precommit`
  - `QuorumCert`
  - `ViewChange`
  - `Ping`
  - `Gossip`
  - `SyncRequest`
  - `SyncResponse`
  - `GetPeers`
  - `PeersList`

Interpretacja:
- `privai-node` nie uzywa dzis escrow packet wire jako validator protocol.
- `privai-node` buduje wlasny session layer nad prymitywami z `nxms-transport`.
- session layer jest juz realny, ale nie jest jeszcze czysto wydzielony jako osobna warstwa architektoniczna.

## 4. Frozen target split

Docelowy podzial ma miec trzy warstwy:

1. shared transport primitives
2. validator session transport
3. consensus overlay

Escrow/control-plane transport pozostaje obok nich jako osobna sciezka packetowa.

### 4.1. Layer A: shared transport primitives

Ta warstwa moze pozostac w `nxms-transport`.

Nalezy do niej:
- Falcon sign/verify
- FrodoKEM encap/decap
- XChaCha20-Poly1305 helpers
- `Peer` / `PeerBook`
- low-level Tor framed IO
- domain/AAD helpers, jesli sa wspolne i nie niosa semantyki konkretnej aplikacji

Nie nalezy do niej:
- `ConsensusMsg`
- state sync semantics
- gossip fanout policy
- QC propagation rules
- escrow application message semantics

### 4.2. Layer B: escrow/control-plane packet transport

Ta warstwa pozostaje zwiazana z `nxms-transport`.

Nalezy do niej:
- `NXMS/1`
- `NXMS/2`
- `NxmsEnvelope*`
- `NxmsPayload*`
- `SealedPacket`
- `TxSignReqBody`
- `TxSignRespBody`
- `ContractPropose`
- `ContractSig`
- `EscrowBody`

Interpretacja:
- to jest packet transport dla escrow, mailboxa i control-plane,
- to nie jest validator session transport,
- to nie jest consensus wire protocol.

### 4.3. Layer C: validator session transport

Ta warstwa jest w trakcie budowy i docelowo powinna byc wydzielona z `privai-node::net`.

Nalezy do niej:
- `HandshakeMsg`
- identity binding peer-to-peer
- transcript binding
- session key derivation
- encrypted frame format
- writer/read loops
- reconnect/rebuild rules
- stale connection detection
- rate limiting
- ban list
- connection pool lifecycle

Docelowy dom:
- preferowany: `privai-p2p`
- przejsciowo dopuszczalne: `nxms-transport::session`

### 4.4. Layer D: consensus overlay

Ta warstwa pozostaje w `privai-node`.

Nalezy do niej:
- `ConsensusMsg`
- gossip semantics
- sync semantics
- vote/QC propagation
- peer scoring/policy
- retry policy na poziomie komunikatow consensusowych
- integration z timeout/view change/finality

Nie nalezy do niej:
- packet crypto escrow
- general PQ primitive wrappers
- raw Tor connection helpers

## 5. What must stay where

### 5.1. Must stay in `nxms-transport`

- generic Falcon/Frodo/XChaCha helpers
- packet transport crypto dla escrow/control-plane
- `wire.rs` escrow/control-plane bodies
- low-level framed Tor IO
- allowlist-style peer representation

### 5.2. Must move out of `privai-node::net`

Docelowo:
- `HandshakeMsg`
- encrypted validator frame protocol
- session maintenance lifecycle
- connection actor/writer abstractions
- per-peer session state

Interpretacja:
- to jest infrastructure layer,
- nie powinna byc definitywnie przyklejona do consensus loop implementation.

### 5.3. Must stay in `privai-node`

- `ConsensusMsg`
- consensus message handling
- tx gossip policy
- sync policy
- QC / vote verification flow
- proposer / timeout / liveness integration

## 6. API boundaries

### 6.1. Validator session transport API

Consensus overlay powinien widziec tylko interfejs w stylu:
- `send(peer_id, msg_bytes_or_typed_msg)`
- `broadcast(peer_set, msg)`
- `recv() -> (peer_id, msg)`
- `peer_status(peer_id)`
- `stats()`

Consensus overlay nie powinien widziec:
- handshake internals
- KEM ciphertext exchange
- raw nonce/tag handling
- writer task internals

### 6.2. Escrow/control-plane transport API

Escrow layer moze dalej widziec packet transport API:
- seal/open
- envelope encode/decode
- packet relay/store-forward

Escrow/control-plane nie powinien zalezec od validator session lifecycle.

### 6.3. Shared primitive API

Shared primitive layer ma pozostac niska:
- sign
- verify
- encaps
- decaps
- symmetric encrypt/decrypt
- connect/read/write frame

Nie wolno tam przenosic semantyki:
- `release/refund/dispute`
- `Proposal/Prevote/Precommit`
- gossip fanout
- state sync windows

## 7. Failure model boundaries

### 7.1. Validator session transport odpowiada za:

- slow peer handling
- stale connection rebuild
- reconnect policy
- bounded queue backpressure
- session corruption / decrypt failure handling
- anti-sybil admission against `PeerBook`

### 7.2. Consensus overlay odpowiada za:

- duplicate message handling
- retry semantics at consensus level
- timeout-driven view change
- QC liveness and propagation policy
- sync retry policy

### 7.3. Escrow/control-plane transport odpowiada za:

- packet integrity
- sender authenticity
- mailbox/store-forward suitability
- replay discipline tied to packet semantics

## 8. Current non-conformities

Najwazniejsze obecne rozjazdy wzgledem docelowego splitu:

1. `privai-node::net` laczy:
   - validator handshake
   - encrypted frame transport
   - connection lifecycle
   - i czesc odpowiedzialnosci, ktore docelowo naleza do osobnej session layer

2. `nxms-transport` wyglada z zewnatrz jak jeden crate "od transportu", ale semantycznie zawiera:
   - packet transport escrow
   - shared crypto/network primitives

3. incoming path validator listenera czyta `read_frame()` i od razu robi `serde_json::from_slice::<ConsensusMsg>()`,
   zamiast przejsc przez jednoznacznie wydzielona warstwe session decrypt/read abstraction.

## 9. Required implementation checkpoints

### Checkpoint 0. Freeze the split

Acceptance:
- ten dokument jest zatwierdzony,
- `Decision Register` i `Gap Register` odzwierciedlaja split.

### Checkpoint 1. Stop semantic drift

Acceptance:
- nowe zmiany nie moga wprowadzac `ConsensusMsg` do `nxms-transport::wire`,
- nowe escrow packet messages nie moga trafic do validator session protocol.

### Checkpoint 2. Isolate validator session API

Acceptance:
- `privai-node` korzysta z jednego waskiego interfejsu session transport,
- handshake i frame crypto nie wyciekaja do consensus loop API.

### Checkpoint 3. Extract session layer

Acceptance:
- session-specific typy i lifecycle wychodza z `privai-node::net` do osobnej warstwy,
- `privai-node` zachowuje tylko consensus semantics.

### Checkpoint 4. Keep `nxms-transport` narrow

Acceptance:
- `nxms-transport` pozostaje packet/control-plane transportem plus shared primitives,
- nie staje sie miejscem dla consensus wire semantics.

### Checkpoint 5. End-to-end validator network invariants

Acceptance:
- bounded queues
- reconnect
- stale connection rebuild
- anti-sybil allowlist
- encrypted frames
- gossip/sync/QC path dzialaja przez nowy split bez regresji

## 10. Recommended refactor order

1. Zamrozic ten split w docs
2. Dodac waski trait/interfejs session transport dla `privai-node`
3. Przepiac `gossip`, `state_sync` i `consensus_loop` na interfejs, nie na `net` internals
4. Dopiero wtedy wydzielic session layer z `privai-node::net`
5. Na koncu zdecydowac, czy session layer laduje w:
   - `privai-p2p`
   - czy przejsciowo w `nxms-transport::session`

## 11. Preferred direction

Preferowany kierunek:
- `nxms-transport` zostaje transportem escrow/control-plane i wspolnymi prymitywami
- validator session layer staje sie osobnym crate `privai-p2p`
- `privai-node` pozostaje consensus overlay

Powod:
- najczystszy rozdzial semantyki,
- najmniejsze ryzyko rozmycia odpowiedzialnosci crate'ow,
- latwiejsza dalsza praca nad P2P jako realnym systemowym komponentem.

## 12. Forbidden inference

- nie wolno zakladac, ze wspolny Tor transport oznacza wspolny wire protocol
- nie wolno wciskac `SealedPacket` do validator P2P tylko dlatego, ze juz istnieje
- nie wolno nazywac `privai-node::net` finalnym p2p crate'em tylko dlatego, ze ma handshake i connection pool
- nie wolno przenosic escrow semantics do validator transport
- nie wolno przenosic consensus semantics do `nxms-transport`
