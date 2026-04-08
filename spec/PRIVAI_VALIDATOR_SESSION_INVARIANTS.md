# privAI Validator Session Invariants

Status: focused anti-drift invariants doc for the current validator session transport layer.
Canonicality: non-overriding support doc for current validator session behavior inside `privai-node`. This document does not create a new top-level source of truth and does not override canonical product, protocol, formats or consensus semantics; it records how the current session layer should be interpreted and where agents must not guess missing behavior.
Owner: privAI networking and validator session architecture.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`
- `spec/PRIVAI_DECISION_REGISTER.md`
- `spec/PRIVAI_GAP_REGISTER.md`
- `spec/PRIVAI_CONSENSUS.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zapisac jeden anti-drift opis zachowania validator session layer,
- odciac zgadywanie handshake, reconnect i failure behavior "z intuicji",
- rozdzielic:
  - co jest `Frozen`,
  - co jest `Current canonical`,
  - co jest `Unresolved`,
  - co jest `Current non-conformity`,
- dac devom i agentom jeden punkt odniesienia przed pisaniem testow albo refactorow.

To nie jest dokument wire format transakcji.
To nie jest tez dokument finality ani proof semantics.
To jest dokument o zachowaniu warstwy session transport pomiedzy shared transport primitives a consensus overlay.
To nie jest nowy rownorzedny source of truth obok canonical spec set.

## 2. How Devs And Agents Should Use This Doc

Ten dokument nalezy czytac razem z:
- `spec/PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`

Regula interpretacji:
- canonical source of truth dla architektury pozostaje w glownym secie `spec/` oraz w `spec/PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`,
- ten dokument sluzy do anty-driftowego opisania current validator session behavior, nie do ustanawiania nowej architektury,
- jesli zachowanie jest tutaj oznaczone jako `Current canonical`, nie wolno go lokalnie "ulepszac" bez jawnej decyzji i migracji,
- jesli zachowanie jest oznaczone jako `Unresolved`, nie wolno go dopowiadac przez implementacje "najbardziej sensownej" wersji,
- jesli zachowanie jest oznaczone jako `Current non-conformity`, nie wolno go sprzedawac jako finalnego invariantu; trzeba je albo naprawic, albo jawnie utrzymac jako luka,
- jesli kod i ten dokument sie rozjezdzaja, nalezy zglosic rozjazd albo doprowadzic kod do zgodnosci, a nie wybierac wygodniejszej wersji.

## 3. Status Vocabulary Used Here

### Frozen

- validator session transport pozostaje osobna warstwa od escrow/control-plane packet transport,
- `NXMS/1`, `NXMS/2`, `NxmsEnvelope*` i `SealedPacket` nie sa validator session protocol,
- `consensus_loop`, `gossip` i `state_sync` sa warstwa wyzsza i nie naleza do low-level session mechanics.

### Current canonical

- obecny kod w `privai-node/src/session_impl.rs` i `privai-node/src/session_transport.rs` definiuje normatywny behavior session layer,
- handshake uzywa Falcon signature + FrodoKEM-based session secret derivation,
- outgoing side utrzymuje `ConnectionPool` z actor-style writer task i bounded queue,
- incoming side wymusza allowlist through `PeerBook`, rate limiting i ban checks.

### Future target requiring migration

- wydzielenie session layer do osobnego crate albo docelowej warstwy poza `privai-node`,
- message-class specific limits / priorities / retry policy,
- bogatsza observability i metrics plane.

### Unresolved

- jednoznaczny finalny model retry policy per message class,
- finalny peer lifecycle beyond current lazy-connect + stale rebuild behavior,
- finalny validator-session wire extraction target.

### Current non-conformity

- listener-side post-handshake receive path nie wykonuje jeszcze jawnego decrypt using the derived shared secret before deserializing `ConsensusMsg`,
- outgoing handshake path nie weryfikuje jawnie, ze `peer_handshake.peer_id` equals the dialed `Peer.id`,
- `BanList` currently gates incoming path, not the outgoing dial path.

## 4. Scope And Boundary

Validator session layer obejmuje:
- handshake
- session key establishment
- encrypted frame transport
- connection pool
- reconnect / stale detection
- ban list
- rate limiting
- bounded queues / writer task behavior

Ta warstwa nie obejmuje:
- consensus semantics
- gossip policy
- state sync semantics
- escrow/control-plane packet transport
- public discovery / DHT / open peer marketplace

## 5. Current Code Map

### 5.1. Public boundary

`privai-node/src/session_transport.rs`
- `ValidatorSessionTransport`
- API used by higher-level modules

### 5.2. Compatibility facade

`privai-node/src/net.rs`
- thin re-export layer for current session internals

### 5.3. Concrete implementation

`privai-node/src/session_impl.rs`
- `HandshakeMsg`
- `ConnectionPool`
- `ConnectionMeta`
- `BanList`
- `RateLimiter`
- `run_listener`

## 6. Handshake Invariants

### 6.1. Frozen

- handshake musi byc signed Falconem,
- handshake musi zawierac validator identity (`peer_id`) oraz PQ public keys,
- validator session handshake nie jest escrow envelope ani `NXMS` packet.

### 6.2. Current canonical: incoming path

Incoming path in `run_listener` zachowuje sie dzis tak:

1. incoming connection przechodzi najpierw przez rate limiter keyed by `source = addr.to_string()`,
2. incoming connection jest odrzucane, jesli semaphore `MAX_INCOMING_CONNECTIONS = 10` jest pelny,
3. listener generuje losowy nonce i wysyla `HandshakeMsg::Challenge` z timeoutem zapisu `10s`,
4. listener czeka do `10s` na `HandshakeMsg::Init` od peera,
5. `version` musi byc rowne `HANDSHAKE_VERSION = 1` oraz nonce musi sie zgadzac z wyslanym w Challenge,
6. wstepnie sprawdzany jest `BanList` oraz obecnosc `peer_id` w `PeerBook` i zgodnosc kluczy (niezgodnosci skutkuja przerwaniem polaczenia, ale nie wpisuja na BanList przed autoryzacja),
7. Falcon signature peera na transcript (zawierajacym nonce serwera) musi przejsc verify (kryptograficzne potwierdzenie tozsamosci),
8. listener robi FrodoKEM encapsulation against peer KEM public key,
9. listener wysyla swoj signed `HandshakeMsg::Response` z `kem_ct` z timeoutem zapisu `10s`,
10. po sukcesie listener wchodzi do message loop i odszyfrowuje nadchodzace ramki przy uzyciu derived shared secret.

### 6.3. Current canonical: outgoing path

Outgoing path in `ConnectionPool::establish_connection` zachowuje sie dzis tak:

1. Tor connect ma timeout `30s`,
2. outgoing side (klient) odbiera najpierw `HandshakeMsg::Challenge` od serwera z timeoutem odczytu `10s`,
3. klient wysyla `HandshakeMsg::Init` podpisany kluczem Falcon w powiazaniu z nonce serwera, z timeoutem zapisu `10s`,
4. klient czeka na podpisany `HandshakeMsg::Response` od peera z timeoutem odczytu `10s`,
5. peer reply musi miec `version = 1` oraz nonce serwera musi byc spojny w transcriptach,
6. Falcon signature peera musi przejsc verify,
7. peer handshake musi zawierac `kem_ct_b64`,
8. shared secret jest derivowany przez `kem_decaps(my_kem_sk, peer_kem_ct)`,
9. po sukcesie tworzony jest bounded writer channel,
10. `ConnectionMeta` dostaje peer keys, shared secret i `handshake_done = true`,
11. dopiero wtedy polaczenie jest zapisywane w `connections`.

### 6.4. Unresolved

- finalny transcript-binding model poza obecnym signed JSON body,
- finalny stable session transcript hash,
- finalna separacja validator handshake bytes od obecnej impl-specific JSON serialization.

### 6.5. Current non-conformities

(Usunięto dawne niezgodności: incoming decrypt gap oraz weryfikacja `peer_id` zostały naprawione w modelu `Challenge -> Init -> Response`).

## 7. Session Establishment And Active Session Rules

### 7.1. Current canonical

Na outgoing side session jest uznana za aktywna tylko wtedy, gdy:
- `ConnectionMeta` istnieje in `connections`,
- `handshake_done == true`,
- `needs_rebuild == false`.

`send_message` traktuje polaczenie jako nieaktywne i rebuild-required, jesli:
- entry nie istnieje,
- `needs_rebuild == true`,
- `handshake_done == false`.

### 7.2. Current canonical asymmetry

Obecny model jest asymetryczny:
- outgoing side utrzymuje persistent `ConnectionPool`,
- incoming side trzyma connection lifecycle inside spawned listener task and does not register the session in `ConnectionPool`.

Incoming session is active only inside the spawned per-connection task after successful handshake and until read failure, timeout or channel shutdown.

To jest current behavior, nie nalezy z tego wyprowadzac finalnego symmetrical session modelu bez osobnej decyzji.

## 8. Connection Lifecycle

### 8.1. Current canonical

- incoming session task ma total timeout `300s`,
- incoming message frames sa czytane z limitem `1 MiB`,
- handshake frame limit is `64 KiB`,
- writer task exits on:
  - write failure,
  - explicit `Shutdown`,
  - channel close,
- `remove_connection` usuwa peer entry z `ConnectionPool`,
- `meta.touch()` updates `last_activity` and increments `ops_count` after successful enqueue to writer channel.

### 8.2. Frozen

- session layer is allowed to be best-effort and failure-prone under Tor,
- higher layers must not assume permanent connectivity,
- low-level connection lifecycle remains transport-layer behavior, not consensus semantics.

## 9. Reconnect And Maintenance Rules

### 9.1. Current canonical

Default `ConnectionPoolConfig` today:
- `idle_timeout_secs = 120`
- `max_age_secs = 600`
- `health_check_interval_secs = 30`
- `auto_reconnect = true`

Maintenance tick today:
- runs every `30s`,
- computes `max_age_with_jitter = base 600s + [0..119]s`,
- marks peer stale if:
  - `needs_rebuild == true`, or
  - age exceeds jittered max age, or
  - idle exceeds `120s`,
- updates pool stats,
- attempts reconnect only for peers that already have an entry in `connections`,
- does not eagerly connect to peers absent from the pool.

### 9.2. Frozen

- current reconnect model is lazy-connect plus stale rebuild,
- current maintenance model is transport hygiene, not consensus liveness policy.

### 9.3. Unresolved

- final backoff schedule for repeated reconnect failures,
- final rule for abandoning a peer after repeated failures,
- final relation between peer health and consensus scoring/policy.

## 10. Ban List Rules

### 10.1. Current canonical

- ban duration is `3600s`,
- `BanList::is_banned()` checks current time against expiry,
- `BanList::cleanup()` removes expired entries,
- listener rejects an incoming peer if `BanList` already contains its `peer_id`,
- listener odrzuca peera, ale **NIE banuje go** na wczesnym etapie handshake (przed kryptograficznym proof-of-identity) by zapobiec atakom "ban poisoning",
- ban list chroni przed zautoryzowanymi i znanymi peerami, którzy okazali się złośliwi na wyższym poziomie.

### 10.2. Current non-conformity

- outgoing dial path does not currently consult `BanList` before connect / handshake.

## 11. Rate Limiting Rules

### 11.1. Current canonical

- rate limiting applies to incoming connections only,
- key is `source_addr = addr.to_string()`,
- limit is `5` connections per `60s` window,
- if the window expires, the counter resets,
- rate-limited connection is dropped before handshake processing.

### 11.2. Frozen

- rate limiting is a transport protection mechanism,
- it is not a consensus policy mechanism.

### 11.3. Unresolved

- final limiter granularity for validator identities vs transport source strings,
- final interaction between rate limiter and ban list in a separate extracted session layer.

## 12. Queue And Backpressure Rules

### 12.1. Current canonical

- each outgoing peer connection uses one bounded MPSC writer channel,
- `WRITER_CHANNEL_CAPACITY = 64`,
- `send_message` waits up to `10s` to enqueue a frame,
- on enqueue timeout, `send_message` returns transport error,
- on writer task closed, `send_message` returns transport error,
- max concurrent Tor circuit builds is limited by semaphore `MAX_CONCURRENT_BUILDS = 6`.

### 12.2. Frozen

- bounded queue and build semaphore are invariants of the current session layer and must not be silently removed.

### 12.3. Unresolved

- per message class queue partitioning,
- per class priority,
- queue depth exposure as stable metric/API.

## 13. Message Delivery Model

### 13.1. Current canonical

- `send_message` serializes `msg` as JSON bytes,
- po udanym handshake validator session frames musza byc szyfrowane,
- brak `shared_secret` po `handshake_done == true` jest stanem niepoprawnym i nalezy go traktowac jako transport error / rebuild-required state,
- current outgoing path encrypts the frame before enqueue because active pooled sessions store a derived `shared_secret`,
- `broadcast_message` spawns one task per target peer and returns a vector of per-peer results,
- `broadcast_message` is best-effort and does not guarantee atomic success across peers,
- incoming listener forwards successfully parsed `ConsensusMsg` to `msg_tx`,
- malformed `ConsensusMsg` or read failure terminates the per-connection loop.

### 13.2. Frozen

- session layer transports messages; it does not define message-class retry semantics,
- transport success does not imply semantic acceptance by consensus/gossip/sync layers.

### 13.3. Unresolved

- per message class retry policy,
- per class size limits beyond current frame limits,
- per class prioritization and structured logging.

## 14. Failure Handling

### 14.1. Current canonical

- handshake failures abort connection establishment,
- handshake read/write/connect timeouts return transport errors on outgoing path,
- incoming handshake failure logs and drops the stream,
- writer task write failure closes the writer loop,
- incoming connection loop exits on frame read error, deserialize error, or closed `msg_tx`,
- stale connections are not immediately removed; they are marked `needs_rebuild`.

### 14.2. Frozen

- failure handling is fail-fast at transport boundary,
- no hidden retry loop exists inside individual message send paths beyond lazy reconnect.

## 15. Observability

### 15.1. Current canonical

`PoolStats` currently tracks:
- `total_connections`
- `active_connections`
- `stale_connections`
- `total_messages_sent`
- `total_reconnects`
- `total_circuit_builds`

Current logging includes:
- listener start
- rate-limit rejects
- ban events
- handshake success/failure
- stale/idle/old connection marking
- reconnect attempts
- periodic pool stats

### 15.2. Unresolved

- queue depth metrics
- handshake latency metrics
- decrypt/encrypt failure metrics
- structured metrics export instead of eprintln-only observability

## 16. Non-Responsibilities

Validator session layer does not own:
- `ConsensusMsg` semantics
- QC / vote / proposal policy
- gossip fanout policy
- state sync policy
- peer scoring and consensus-level liveness policy
- escrow packet protocol

## 17. Current Non-Conformities And Open Gaps

### 17.1. Current non-conformities

- outgoing path does not consult `BanList` before connect.

### 17.2. Most important unresolved gaps

1. one explicit session-established invariant shared by both outgoing and incoming paths,
2. one final retry / abandon / reconnect policy by message class,
3. one final extracted home for validator session transport outside the current mixed refactor state.

## 18. What Requires Explicit Freeze Update

Ponizsze zmiany nie moga byc robione po cichu:
- handshake transcript or signing rules,
- connect timeout,
- handshake read/write timeouts,
- session activation rule,
- handshake frame limit `64 KiB`,
- message frame limit `1 MiB`,
- ban duration or limiter model,
- `MAX_INCOMING_CONNECTIONS`,
- `MAX_CONCURRENT_BUILDS`,
- queue capacity or build semaphore removal,
- reconnect model,
- moving validator session transport onto escrow packet protocol,
- introducing message-class retry/priority semantics without spec update.

## 19. Required Tests

Ponizsze testy powinny istniec jako minimalny regression pack dla current validator session layer:
- handshake success,
- handshake version reject,
- unknown peer reject,
- Falcon signature reject,
- ban list reject,
- rate limit reject,
- stale connection rebuild,
- queue timeout path,
- incoming decrypt gap currently unresolved.
