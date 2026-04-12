# privAI V0: Finalna Architektura Private Compute

**Status:** propozycja architektury kierunkowej V0
**Data:** 2026-04-11
**Źródło:** synteza rozmowy P-T037-XIAOMI
**Zakres:** finalna architektura direction-level, bez wire formatów, bez kodu, bez legacy docs

Ten dokument zapisuje po polsku kierunkową propozycję finalnej architektury `privAI V0`.

Nie jest to spec implementacyjny.
Nie definiuje formatów binarnych.
Nie definiuje struktur Rust.
Nie twierdzi, że opisane mechanizmy są już zaimplementowane.
Nie korzysta z legacy marketplace docs.

---

## 1. Krótkie Podsumowanie Architektury

`privAI V0` to post-kwantowa sieć prywatnego obliczeniowego AI, w której użytkownik prywatnie wynajmuje izolowany runtime od compute minera i płaci w `PVA` przez FullPrivacy escrow na chainie. Settlement jest deterministyczny: `receipts -> policy -> split`. Tożsamość opiera się na ukrytym root credential i scoped keys, discovery jest prywatne i resource-based, transport opiera się o NXMS / mailbox / relay / Tor-gated, a role sieciowe są rozdzielone: validator, compute miner, relay, mailbox i exit node.

Cały system jest zbudowany wokół jednej zasady:

```text
Chain widzi commitmenty i receipts.
Chain nie widzi workloadów, outputów ani ludzi.
```

---

## 2. Finalne Warstwy Systemu

### 2.1 User / Client Layer

**Odpowiedzialność:** inicjuje lease, wybiera privacy class, akceptuje lease policy, trzyma receipts, składa settlement claim i ocenia output prywatnie.

**Input:** ComputeOffering z discovery, lease policy od minera, metering receipts z sesji.

**Output:** escrow lock transaction, settlement claim, opcjonalny session ack.

**Założenie zaufania:** user ufa egzekucji chaina, ale nie ufa minerowi bez dowodu dostarczenia zasobów.

**Ryzyko prywatności:** leakage przez discovery query, timing płatności, timing sesji.

**Wymagane docs/spec przed kodem:** identity model, private discovery direction.

### 2.2 Runtime / Compute Miner Layer

**Odpowiedzialność:** dostarcza izolowany runtime slice: VM, container, sandbox albo GPU slice; produkuje receipts; odpowiada na heartbeats i challenges.

**Input:** lease policy, potwierdzenie escrow lock, session keys.

**Output:** signed metering receipts, session-ready signal, heartbeat/challenge responses.

**Założenie zaufania:** miner ufa, że chain wypłaci po valid receipts. Protokół nie ufa minerowi na słowo.

**Ryzyko prywatności:** miner może widzieć metadata runtime, zależnie od privacy class.

**Wymagane docs/spec przed kodem:** metering protocol direction, runtime privacy classes direction.

### 2.3 Metering / Receipt Layer

**Odpowiedzialność:** mierzy dostarczone zasoby, podpisuje receipts, przechowuje evidence i umożliwia settlement.

**Input:** measurements runtime, miner signature, optional user ack, challenges.

**Output:** signed receipts, receipt commitment, challenge responses.

**Założenie zaufania:** self-reported receipt wystarcza tylko jako Phase 1 lab. Finalny operatorless settlement wymaga silniejszego trust modelu.

**Ryzyko prywatności:** timing, częstotliwość i rozmiar receipts mogą leakować activity pattern.

**Wymagane docs/spec przed kodem:** metering protocol direction, metering receipt schema, receipt availability spec, metering trust/challenge spec.

### 2.4 Lease Policy Layer

**Odpowiedzialność:** definiuje zasady lease: resource class, duration, price, timeout, privacy class, meter version i settlement formula reference.

**Input:** ComputeOffering, akceptacja usera, warunki minera.

**Output:** lease policy commitment bound do escrow.

**Założenie zaufania:** obie strony akceptują policy przed lockiem escrow; protokół wykonuje policy, nie ocenia jakości AI.

**Ryzyko prywatności:** sam hash policy nic nie ujawnia, ale timing escrow lock może ujawniać istnienie lease.

**Wymagane docs/spec przed kodem:** compute lease object spec.

### 2.5 Escrow / Settlement Layer

**Odpowiedzialność:** lockuje PVA/aPVA, egzekwuje timeouty, waliduje claims, wykonuje release/refund/pro-rata.

**Input:** lease policy commitment, escrow amount, receipts, settlement claim, timeout.

**Output:** release, refund, recovery albo pro-rata split.

**Założenie zaufania:** Phase 0 używa operatora w 2-of-3; Phase 1 używa automated operatora jako bridge; finalnie protokół waliduje receipts bez operatora.

**Ryzyko prywatności:** chain widzi settlement timing i commitments, ale nie workload ani output.

**Wymagane docs/spec przed kodem:** operatorless escrow direction, pro-rata note split spec, operatorless escrow protocol bridge.

### 2.6 Chain / Validator Layer

**Odpowiedzialność:** consensus, commitment tree, nullifiers, tx validation, settlement enforcement, rewards.

**Input:** transactions, proofs/statements, escrow policy, fees.

**Output:** confirmed blocks, settled escrows, rewards.

**Założenie zaufania:** standardowe założenie konsensusu: honest majority / poprawność validator set.

**Ryzyko prywatności:** timing, commitments, nullifiers i fee patterns są on-chain metadata.

**Wymagane docs/spec przed kodem:** protocol versioning direction, później code landing docs.

### 2.7 Identity / Credential Layer

**Odpowiedzialność:** hidden root credential, role keys, session keys, epoch keys, scoped offering IDs, selective reliability proof.

**Input:** root credential, role registration, session negotiation, epoch rotation.

**Output:** scoped keys, Falcon signatures, scoped offering IDs, selective proofs.

**Założenie zaufania:** hidden root nigdy nie jest publiczny; Falcon jest narzędziem podpisu, nie publiczną tożsamością.

**Ryzyko prywatności:** korelacja identity przez timing, powtarzalne scoped IDs albo podpisy użyte w wielu domenach.

**Wymagane docs/spec przed kodem:** identity model direction, identity credential schema.

### 2.8 Discovery Layer

**Odpowiedzialność:** pozwala userowi znaleźć compute capacity po resource class, cenie, privacy class i network mode.

**Input:** ComputeOfferings od minerów, zapytania userów.

**Output:** matching offerings ze scoped offering IDs i opcjonalnym selective reliability proof.

**Założenie zaufania:** discovery jest private/encrypted/credential-gated; bootstrap coordinator jest tylko mostem, nie finalnym modelem.

**Ryzyko prywatności:** leakage tego, kto czego szuka i kiedy.

**Wymagane docs/spec przed kodem:** private discovery direction, private discovery protocol spec.

### 2.9 Transport / Mailbox / Relay Layer

**Odpowiedzialność:** dostarcza encrypted envelopes, route'uje wiadomości, separuje origin od destination, obsługuje mailbox i relay.

**Input:** encrypted envelopes, routing requests, mailbox messages.

**Output:** delivered encrypted messages, relay hops, Tor-gated routing.

**Założenie zaufania:** mailbox nie widzi treści; relay widzi tylko poprzedni/następny hop.

**Ryzyko prywatności:** timing analysis, volume analysis, envelope size leakage.

**Wymagane docs/spec przed kodem:** transport/mailbox privacy direction.

### 2.10 Exit Node Layer

**Odpowiedzialność:** public internet egress dla sesji, które jawnie tego chcą.

**Input:** opt-in session traffic requiring internet.

**Output:** public internet egress przez Tor-gated path.

**Założenie zaufania:** exit node jest osobną rolą i nigdy nie jest default.

**Ryzyko prywatności:** IP/legal/abuse risk; exit node to najniższa klasa prywatności.

**Wymagane docs/spec przed kodem:** exit node direction.

### 2.11 RAG / MCP / Agent Context Layer

**Odpowiedzialność:** utrzymuje jedno źródło prawdy dla Xiaomi, Opus, Codex i przyszłych agentów.

**Input:** V0 docs, task log, prompt log, docs tree, context plan.

**Output:** reading order, guardrails, task context packs, correction pills, current status.

**Założenie zaufania:** agenci ufają MCP/RAG, więc MCP/RAG musi być V0-only i przejść golden tests.

**Ryzyko:** contamination przez legacy docs albo marketplace framing.

**Wymagane docs/spec przed kodem:** istnieją `PRIVAI_V0_SINGLE_SOURCE_OF_TRUTH_CONTEXT_PLAN.md` i `PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md`; implementation później.

---

## 3. Finalny Przepływ End-To-End

1. User chce prywatny compute i wybiera wymagania: resource class, privacy class, network mode, limit ceny.
2. User wykonuje prywatne discovery przez encrypted registry, mailbox query albo inny V0-approved path.
3. Discovery zwraca ComputeOfferings ze scoped offering IDs, bez public provider profile.
4. User i compute miner negocjują lease policy przez encrypted transport.
5. User lockuje środki w FullPrivacy escrow, a chain zapisuje commitment do lease policy.
6. Miner uruchamia izolowany runtime slice i wysyła session-ready signal.
7. Podczas sesji miner produkuje signed metering receipts.
8. User i miner przechowują receipts niezależnie; chain widzi tylko commitments albo evidence required for settlement.
9. Po zakończeniu lub timeout składany jest settlement claim.
10. Phase 1: automated operator waliduje receipts i współpodpisuje mechanicznie.
11. Finalnie: protokół waliduje receipts bez operatora.
12. Funds są release/refund/split w aPVA.
13. Session keys wygasają, epoch keys rotują, scoped offering IDs mogą zostać odświeżone.
14. V0 logs/MCP/RAG context są aktualizowane dla agentów.

---

## 4. Granica On-Chain / Off-Chain / Transport / Context

| Element | Gdzie żyje | Dlaczego | Nie może ujawniać | Otwarte ryzyko |
|---------|------------|----------|-------------------|----------------|
| Lease policy commitment | on-chain | binduje warunki settlement | pełnej policy | timing lease |
| Escrow note | on-chain | lock/value/timeout/nullifier | workloadu i outputu | korelacja amount/timing |
| Receipt commitment | on-chain | binduje evidence | pełnych receipts | timing activity |
| Full receipt body | off-chain | evidence dla settlement | workload contents | availability |
| Workload/prompt | off-chain/session | prywatność produktu | wszystko | miner visibility |
| Output/artifact | off-chain/session | user ocenia prywatnie | wszystko | persistent storage opt-in |
| Runtime metadata | off-chain/miner | operacja runtime | activity patterns | side-channels |
| Scoped offering ID | discovery/off-chain | oferta bez public identity | hidden root link | korelacja ofert |
| Hidden root credential | never exposed | root identity | wszystko | total compromise |
| Falcon signatures | tam gdzie wymagane | authorization | cross-domain identity | signature correlation |
| Reliability proof | off-chain/selective | trust bez leaderboard | pełnej historii | proof schema open |
| Discovery query | off-chain/encrypted | private resource search | query intent | metadata leakage |
| Transport envelope | mailbox/relay | encrypted communication | content | timing/size |
| Exit traffic | opt-in exit | internet access | user identity | legal/IP risk |
| V0 docs context | RAG/MCP | agent source of truth | legacy framing | contamination |

---

## 5. Finalny Settlement Model

### Full Release

**Current reality:** Release wymaga buyer/user + operator w istniejącej mechanice 2-of-3.
**Final direction:** receipts dowodzą pełnego dostarczenia zasobów, a protokół release'uje całość do minera.
**Future follow-up:** operatorless release.
**Open:** finalny trust model receiptów.

### Full Refund

**Current reality:** Refund wymaga merchant/miner + operator w istniejącej mechanice.
**Final direction:** brak valid receipts lub lease never started oznacza full refund do usera.
**Future follow-up:** protocol-validated refund po timeout.
**Open:** auto-refund, jeśli user też znika.

### Pro-Rata Split

**Current reality:** niezaimplementowane; escrow jest all-or-nothing.
**Final direction:** partial delivery daje split: miner dostaje earned share, user dostaje remainder.
**Future follow-up:** 1 input escrow note -> 2 output notes.
**Open:** nowy EscrowAction vs extension istniejącego modelu.

### Timeout Recovery

**Current reality:** RecoveryRelease jest już operatorless: user/buyer + miner/merchant po timeout.
**Final direction:** zachować jako anchor dla operatorless mindset.
**Future follow-up:** doprecyzowanie interaction z pro-rata i auto-claims.
**Open:** co jeśli jedna strona odmawia podpisu.

### Penalty / Slash

**Current reality:** niezaimplementowane.
**Final direction:** future strengthening dla bond/slash przy fraud/fault.
**Future follow-up:** bond tracking, slash conditions, penalty distribution.
**Open:** user vs treasury vs split.

### Dlaczego Quality Disputes Są Wykluczone

Settlement ocenia dostarczenie zasobów, nie jakość AI outputu.
Jakość outputu jest prywatna, subiektywna i nie powinna być widoczna dla chaina.
Jeżeli miner dostarczył zakontraktowany compute, miner zarobił.

---

## 6. Receipt Truth Architecture

Największe ryzyko systemu to nie podpis receiptu, tylko prawdziwość receiptu.

```text
Miner signature proves: miner claimed X.
Miner signature does not prove: miner actually delivered X.
```

### Layer 1: Signed Receipts

Miner podpisuje receipts. To jest baseline evidence i wystarcza jako laboratorium Phase 1, ale samo w sobie jest self-reporting.

### Layer 2: User Acknowledgment

User może opcjonalnie potwierdzić receipt. To wzmacnia evidence, ale nie może być wymagane zawsze, bo user może być offline albo zniknąć.

### Layer 3: Challenge / Response

Finalny operatorless model powinien wymagać challenge/response albo równoważnego mechanizmu, który direction-level dowodzi:

- miner ma zadeklarowany resource class,
- miner utrzymuje liveness,
- miner odpowiada w wymaganym oknie,
- delivery nie jest tylko deklaracją.

Nie definiujemy tu wire formatu challenge.

### Availability

Receipts powinny być przechowywane przez obie strony.
Chain powinien widzieć commitment do receipt evidence, nie pełny body.
Retention powinien obejmować lease duration + timeout + settlement window.

### Ewolucja

```text
Phase 1: signed receipts + automated operator validation
Phase 1.5: optional challenges
Phase 2: mandatory challenge/response for operatorless settlement
Future: TEE / third-party attestation / slash bond as strengthening
```

---

## 7. Identity I Discovery

### Identity

Finalny model:

```text
hidden root -> role keys -> epoch keys -> session keys -> scoped offering IDs
```

Falcon jest narzędziem podpisu, nie public identity.

Role są rozdzielone:

- validator identity,
- compute miner identity,
- relay identity,
- mailbox identity,
- exit node identity.

Te role nie mogą być automatycznie linkowane nawet jeśli ta sama maszyna wykonuje kilka ról.

### Discovery

Discovery nie jest marketplace.

User szuka:

- resource class,
- price_aPVA,
- duration,
- network mode,
- runtime privacy class,
- availability window,
- meter protocol version.

User nie szuka:

- providera po nazwie,
- public reputation,
- public profile,
- public Falcon key,
- historii zleceń.

### Fazy Discovery

```text
Phase 0: encrypted bootstrap coordinator, trust-limited
Phase 1: mailbox-based query + encrypted registry hybrid
Phase 2: gossip / DHT / hybrid, zależnie od wielkości sieci
```

Finalna architektura discovery nie powinna być zamrożona za wcześnie.

---

## 8. Transport I Runtime Privacy

### Transport

NXMS mailbox przechowuje i dostarcza encrypted envelopes.
Relay chain działa onion-style: każdy relay zna tylko poprzedni i następny hop.
Tor-gated / exit node jest explicit opt-in.
P2P/KEM może być optymalizacją sesji, ale nie baseline privacy story.

### Metadata

Content encryption nie wystarcza.
Trzeba jawnie traktować jako ryzyka:

- timing,
- volume,
- envelope size,
- query frequency,
- session duration.

Future strengthening:

- padding,
- timing obfuscation,
- constant-rate messaging,
- mixnet-like query routing.

### Runtime Privacy Classes

Proponowany direction-level model:

| Klasa | Prywatność | Koszt | Uwagi |
|-------|------------|-------|-------|
| VM | najwyższa | najwyższy | default |
| Container | średnia | niższy | shared kernel / side-channel risk |
| Sandbox | niższa | niski | application-level isolation |
| GPU slice | zależna od hypervisora | zmienna | wymaga deklaracji realnych guarantees |

Domyślnie: VM jako najwyższa i najbezpieczniejsza klasa.
Słabsze klasy tylko jako świadomy opt-in usera.

---

## 9. Finalne Fazy

### Phase D0: Documentation Freeze

**Goal:** zamknąć kierunek V0.
**Entry:** V0 master, diagrams, settlement direction, docs tree, context/MCP direction.
**Exit:** komplet Tier 2 direction docs.
**Must not happen:** kod, wire formats, legacy.

### Phase D1: Direction Docs Completion

**Goal:** dopisać wszystkie brakujące direction docs.
**Entry:** Opus wraca / albo operator decyduje, że Xiaomi/Codex mogą przygotować drafts.
**Exit:** direction baseline complete.
**Must not happen:** protocol specs przed direction docs.

### Phase S1: Protocol Specs

**Goal:** exact specs dla receipts, lease object, aPVA, pro-rata, identity, discovery, versioning.
**Entry:** relevant direction doc accepted.
**Exit:** key specs frozen.
**Must not happen:** spec bez direction doc.

### Phase C1: Context MCP/RAG

**Goal:** `privai-context-mcp`, V0-only, read-only, golden tests.
**Entry:** direction baseline stable.
**Exit:** MCP działa z 8 tools i nie miesza legacy.
**Must not happen:** legacy ingest, write tools, edit-code tools.

### Phase L1: Code Landing

**Goal:** mapowanie specs -> code areas, test matrix, implementation plans.
**Entry:** key protocol specs frozen.
**Exit:** landing zones i tests accepted.
**Must not happen:** kod bez landing plan.

### Phase T1: Devnet

**Goal:** automated operator, receipt validation, lease policy binding, pro-rata tests, bootstrap discovery.
**Entry:** code landing accepted.
**Exit:** devnet stable.
**Must not happen:** production claims.

### Phase P1: Production Readiness

**Goal:** final readiness, no silent downgrade, docs/code alignment, role incentives, testnet/devnet evidence.
**Entry:** stable devnet.
**Exit:** production checklist passed.
**Must not happen:** launch before checklist.

---

## 10. Decyzje Do Zamrożenia Przed Kodem

1. **Settlement primitive:** resource delivery, not AI quality.
2. **Operatorless target:** operator jest bridge, nie destination.
3. **Private discovery baseline:** no public marketplace.
4. **Hidden root identity:** Falcon is signing tool, not identity.
5. **aPVA precision candidate:** `1 PVA = 10^12 aPVA`, blocked by max supply/type.
6. **Receipt truth model:** signed receipt + optional user ack + challenge/response.
7. **Receipt dual storage:** user i miner trzymają evidence.
8. **Five node roles:** validator, compute miner, relay, mailbox, exit node.
9. **No silent downgrade:** FullPrivacy downgrade nigdy nie może być cichy.
10. **V0-only agent context:** RAG/MCP bez legacy.
11. **VM as default privacy class:** weaker modes opt-in.
12. **Exit node opt-in only:** nigdy default.

---

## 11. Decyzje, Których Nie Wolno Zamrażać Jeszcze

1. **Discovery architecture:** registry vs mailbox vs gossip vs DHT.
2. **u64 vs u128 dla aPVA:** wymaga max supply decision.
3. **Challenge/response details:** wymaga metering trust spec.
4. **Pro-rata note split mechanics:** wymaga code audit i pro-rata spec.
5. **Identity credential format:** wymaga identity schema/proof decisions.
6. **Reliability scoring weights:** wymaga danych z devnet.
7. **Transport metadata hardening level:** wymaga threat modelu.

---

## 12. Rekomendowana Kolejność Następnych Dokumentów

1. `PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md`
2. `PRIVAI_V0_METERING_PROTOCOL_DIRECTION.md`
3. `PRIVAI_V0_IDENTITY_MODEL_DIRECTION.md`
4. `PRIVAI_V0_NODE_ROLES_AND_INCENTIVES_DIRECTION.md`
5. `PRIVAI_V0_PRIVATE_DISCOVERY_DIRECTION.md`
6. `PRIVAI_V0_RUNTIME_PRIVACY_CLASSES_DIRECTION.md`
7. `PRIVAI_V0_TRANSPORT_MAILBOX_PRIVACY_DIRECTION.md`
8. `PRIVAI_V0_EXIT_NODE_DIRECTION.md`
9. `PRIVAI_V0_APVA_DENOMINATION_DIRECTION.md`
10. `PRIVAI_V0_PROTOCOL_VERSIONING_DIRECTION.md`
11. `PRIVAI_V0_RECEIPT_TRUTH_DIRECTION.md`
12. `PRIVAI_V0_PRODUCTION_PHASE_NAMING_CLARIFICATION.md`

Najważniejszy dodany dokument to `PRIVAI_V0_RECEIPT_TRUTH_DIRECTION.md`, ponieważ receipt truth jest największym pojedynczym ryzykiem dla całego V0.

---

## 13. Finalne Red Lines

Natychmiast zatrzymać task, jeśli agent:

- używa `AI marketplace` jako opisu privAI,
- proponuje public provider profiles,
- traktuje public discovery jako baseline,
- twierdzi, że operatorless escrow jest już implemented,
- twierdzi, że pro-rata jest już implemented,
- definiuje wire format bez protocol spec task,
- proponuje code changes przed direction/spec docs,
- chce ingestować legacy docs do RAG/MCP,
- merge'uje compute miner i validator jako jedną rolę,
- traktuje self-reported energy jako settlement truth,
- robi internet exit default,
- pomija Phase 1 i chce direct Phase 0 -> Phase 2,
- traktuje automated operatora jako permanentny element,
- akceptuje silent downgrade,
- twierdzi, że receipt truth jest solved.

---

## 14. Blokujące Pytania

1. Jaki jest maksymalny supply PVA?
   - Blokuje wybór `u64` vs `u128`.

2. Czy Phase 2 wymaga challenge/response dla receipt truth?
   - Blokuje operatorless bridge i metering trust spec.

3. Czy production phases i escrow transition phases trzeba przemianować?
   - Blokuje jasną komunikację z agentami.

---

## 15. Self-Check

```text
Czy czytano legacy docs: NIE
Czy czytano kod: NIE
Czy edytowano kod: NIE
Czy zdefiniowano wire formaty: NIE
Poziom dokumentu: direction-level
Status: propozycja architektury, nie implementation spec
```
