# privAI V0: Domain Model Candidate Classification

**Status:** domain model classification / candidate review
**Data:** 2026-04-11
**Źródło:** P-T040-XIAOMI
**Zakres:** klasyfikacja elementów finalnego modelu domenowego V0

Każdy element jest sklasyfikowany jako:

- **FROZEN_CANDIDATE** — element jest dobrze zdefiniowany, V0 direction jest jasny, kod potwierdza feasibility, gotowy na spec
- **CANDIDATE** — element jest zdefiniowany kierunkowo, ma sens, ale wymaga więcej detali lub decyzji przed specem
- **OPEN** — element istnieje w V0 direction ale ma znaczące otwarte pytania
- **BLOCKED_BY_CODE_AUDIT** — element wymaga audytu kodu żeby określić feasibility
- **REJECTED** — element nie powinien być częścią modelu

---

## 1. Entity Classification

| Entity | Definition | Status | Why | Blockers | Code Areas To Audit |
|---|---|---|---|---|---|
| **User** | Podmiot wynajmujący compute. Ma HiddenRootCredential. Derivuje klucze sesji. Składa roszczenia. | FROZEN_CANDIDATE | V0 master §1 definiuje user jako compute lessee. Kod ma wallet z budowaniem transakcji. User jest najprostszą encją. | Brak. | `privai-wallet/src/` — sprawdzić czy wallet obsługuje nowe SpendPolicy types. |
| **ComputeMiner** | Podmiot dostarczający zasoby. Ma osobny HiddenRootCredential. Derivuje ComputeMinerRoleKey. Produkuje receipts. | CANDIDATE | V0 master §2 definiuje compute miner jako dostawcę runtime slice. Ale kod nie ma compute miner jako osobnej encji. Node jest validator. Compute miner jest nową rolą. | Brak definicji w kodzie. Musi być nowy moduł. | Brak — nie istnieje w kodzie. Nowy moduł `privai-compute-miner/`. |
| **Validator** | Podmiot zabezpieczający chain. Ma ValidatorRoleKey (obecny Falcon PK). Głosuje, produkuje bloki. | FROZEN_CANDIDATE | Kod już ma validator: PC-BFT consensus, stake-weighted voting, node_pk_hash. V0 nie zmienia konsensusu. Validator jest code-confirmed. | Brak. | `privai-node/src/node.rs` — consensus loop, voting. |
| **Relay** | Węzeł routujący encrypted envelopes. Widzi prev/next hop. Osobna rola. | OPEN | V0 master §2 definiuje relay jako osobną rolę. Ale kod nie ma relay node. Transport ma single-hop Tor SOCKS5, nie onion chain. | Brak onion routing protocol. Brak relay node implementation. Brak PVA incentive model. | `nxms-transport/src/tor_net.rs` — sprawdzić czy można rozszerzyć o multi-hop. |
| **MailboxNode** | Węzeł przechowujący/dostarczający encrypted envelopes. Nie widzi treści. Obsługuje push/pull/ack. | CANDIDATE | Kod ma nxms-mailbox z HTTP push/pull/ack, SQLite, rate limiting. To jest realna baza. Ale MailboxNode jako osobna rola z PVA incentive jest nowa. | Brak PVA incentive model dla mailbox. Brak discovery endpoint. | `nxms-mailbox/src/` — sprawdzić czy można dodać `/v1/discover`. |
| **ExitNode** | Węzeł zapewniający public internet egress. Osobna rola, jawny opt-in. | OPEN | V0 master §2 definiuje exit node jako explicit opt-in. Ale kod nie ma exit node jako osobnej roli. Tor SOCKS5 jest client-side. Risk model nie jest zdefiniowany. | Brak definicji risk model. Brak pricing model. Brak implementation. | `nxms-transport/src/tor_net.rs` — sprawdzić czy connect_via_tor jest client-side only. |

---

## 2. Value Object Classification

| Value Object | Definition | Status | Why | Blockers | Code Areas To Audit |
|---|---|---|---|---|---|
| **ResourceClass** | `{ gpu_class, cpu_tier, ram_mb, vram_mb, storage_class }` — opis zasobu. | CANDIDATE | V0 master §2 mówi "VM/container/sandbox/GPU slice/CPU/RAM." Ale kod nie ma enum ani struct reprezentującej resource class. Musi być nowy typ. | Brak definicji w kodzie. Dekompozycja na pola wymaga decyzji. | Brak — nie istnieje. |
| **PrivacyClass** | `{ VM, Container, Sandbox, ConfidentialRuntime }` — klasa izolacji. | OPEN | V0 master §2 mówi "miner should not learn plaintext workload. If runtime cannot guarantee that, privacy class must say so." Ale granularity klas nie jest zdefiniowana. | Brak definicji granularności. Brak specyfikacji co miner widzi per klasa. | Brak — nie istnieje. |
| **NetworkMode** | `{ Isolated, NxmsOnly, TorGated, InternetExit }` — dostęp sieciowy runtime. | FROZEN_CANDIDATE | V0 master §2 explicitnie definiuje 4 tryby. To jest zdefiniowane i jasne. | Brak. | `nxms-transport/src/tor_net.rs` — sprawdzić czy TorGated i InternetExit są rozróżnialne. |
| **SettlementMode** | `{ AllOrNothing, ProRata }` — tryb rozliczania. | CANDIDATE | V0 master §3 mówi "split is allowed and should be expected for compute leases." Ale kod jest all-or-nothing. Nowy enum. | Pro-rata note split nie istnieje. | `privai-ledger/src/escrow.rs` — sprawdzić validate_output_target. |
| **SettlementFormula** | `{ LinearProRata, ... }` — wzór obliczania podziału. | OPEN | V0 mówi "valid receipts prove N hours delivered, settlement pays N%." Ale exact formula nie jest zdefiniowana. Zależy od aPVA precision. | Brak frozen formula. Zależy od aPVA precision. | Brak — nie istnieje. |
| **LedgerAmount** | Kwota w aPVA na poziomie ledger. U64 lub U128. | BLOCKED_BY_CODE_AUDIT | V0 rozważa 10^12 aPVA. Kod ma Amount14 (max 16,383) dla LWE. Czy LedgerAmount jest osobnym type? Wymaga audytu. | Decyzja o max supply PVA. Audyt czy Amount14 jest rail-specific. | `privai-chain/src/primitives.rs` — Amount14. `privai-chain/src/note.rs` — OutputNote uses Amount14. |
| **ScopedOfferingId** | Identyfikator oferty compute, rotowalny, nie-linked z root. | OPEN | V0 master §5 mówi "scoped-offering-id, rotatable." Ale kod nie ma żadnego scoped ID. Discovery protocol nie istnieje. | Zależy od private discovery protocol. | Brak — nie istnieje. |
| **HeartbeatStatus** | `{ Active, Missed, Terminated }` — status liveness minera. | OPEN | V0 settlement direction §5 mówi o heartbeats. Ale heartbeat protocol nie istnieje. | Zależy od metering protocol. | Brak — nie istnieje. |
| **MeteringUnits** | Jednostki dostarczonego compute. | OPEN | V0 mówi "compute units" ale nie definiuje co to jest. Sekundy GPU? CPU cycles? | Brak definicji jednostek. Zależy od metering protocol direction. | Brak — nie istnieje. |

---

## 3. Aggregate Classification

| Aggregate | Root | Definition | Status | Why | Blockers | Code Areas To Audit |
|---|---|---|---|---|---|---|
| **ComputeLease** | `lease_id` | Spójna grupa: polityka, escrow, user, miner, session, receipts. | CANDIDATE | V0 master §2 mówi "private compute session is the core object." Ale ComputeLease jako agregat nie istnieje w kodzie. | Brak definicji w kodzie. Nowy moduł. | Brak — nie istnieje. |
| **EscrowNote** | `note_commit` | Obecny: SpendPolicy, amount, timeout, nullifier. V0: dodaje lease_policy_commit, settlement_mode. | BLOCKED_BY_CODE_AUDIT | Obecny Escrow2of3 jest code-confirmed. Nowy ComputeLeaseEscrow wymaga audytu czy SpendPolicy enum jest extensible i czy validate_output_target obsługuje 2 outputs. | Audyt extensibility SpendPolicy. Audyt validate_output_target. | `privai-chain/src/note.rs` — SpendPolicy enum. `privai-ledger/src/escrow.rs` — validate_output_target, validate_escrow_auth. |
| **ComputeOffering** | `scoped_offering_id` | Oferta compute: resource class, cena, privacy, network mode. | OPEN | V0 master §5 mówi o ComputeOffering. Ale discovery protocol nie istnieje. ComputeOffering bez discovery jest pustym obiektem. | Zależy od private discovery protocol. | Brak — nie istnieje. |
| **IdentityBundle** | `hidden_root_credential` | Root → role keys → epoch keys → session keys. | BLOCKED_BY_CODE_AUDIT | V0 master §1 definiuje hidden root + scoped keys. Ale kod traktuje Falcon PK jako identity. Migration wymaga audytu jak Falcon PK hash jest używany. | Audyt użycia Falcon PK hash jako identifier. | `privai-node/src/identity_provider.rs` — PQCIdentity. `privai-node/src/node.rs` — node_pk_hash. `privai-chain/src/escrow.rs` — falcon_pk_hash. `nxms-transport/src/peers.rs` — sig_pk_b64. |
| **MeteringSession** | `session_id` | Lease + receipts[] + heartbeat_log[]. | OPEN | V0 mówi o sessions produkujących receipts. Ale metering protocol nie istnieje. Session lifecycle nie jest zdefiniowany. | Zależy od metering protocol direction. | Brak — nie istnieje. |

---

## 4. Event Classification

| Event | When Emitted | Status | Why | Blockers |
|---|---|---|---|---|
| **LeaseNegotiated** | User i miner uzgodnili politykę lease | OPEN | Negocjacja wymaga discovery i transport. Ani discovery ani lease negotiation protocol nie istnieją. | Private discovery. Lease negotiation protocol. |
| **EscrowLocked** | Środki zablokowane na chainie | FROZEN_CANDIDATE | Kod już ma escrow lock. Mechanika jest code-confirmed. Nowy SpendPolicy dodaje nowy typ locka. | Nowy SpendPolicy type. |
| **SessionStarted** | Runtime provisionowany | OPEN | Kod nie ma runtime provisioning. ComputeMiner nie istnieje. | ComputeMiner moduł. Runtime provider. |
| **ReceiptProduced** | Miner zmierzył delivery, podpisał receipt | OPEN | Receipt struct nie istnieje. Metering protocol nie istnieje. | Metering protocol direction. Receipt schema. |
| **HeartbeatObserved** | Liveness check | OPEN | Heartbeat protocol nie istnieje. | Metering protocol direction. |
| **ChallengeIssued** | Protokół challenge'uje minera | OPEN | Challenge/response nie istnieje. Phase 2+. | Metering trust/challenge spec. |
| **ChallengeResponded** | Miner odpowiada na challenge | OPEN | Jak wyżej. | Metering trust/challenge spec. |
| **SessionTerminated** | Sesja zakończona | OPEN | Session lifecycle nie jest zdefiniowany. | ComputeMiner moduł. Runtime provider. |
| **SettlementClaimed** | User/miner składa claim z receipts | CANDIDATE | Kod ma escrow submit path. Mechanika składania transakcji jest code-confirmed. Ale claim z receipts wymaga receipt infrastructure. | Receipt infrastructure. |
| **SettlementValidated** | Protokół/operator waliduje receipts → decyzja | CANDIDATE | Kod ma 10-punktową walidację escrow. Walidacja jest code-confirmed. Ale walidacja receipts wymaga receipt schema i lease policy. | Receipt schema. Lease policy binding. |
| **SettlementExecuted** | Escrow wykonuje release/refund/split | FROZEN_CANDIDATE | Kod ma execution dla Release/Refund/RecoveryRelease. Mechanika jest code-confirmed i tested. | Pro-rata note split (nowa akcja). |
| **OfferingPublished** | Miner publikuje ofertę compute | OPEN | ComputeOffering nie istnieje. Discovery protocol nie istnieje. | Private discovery protocol. |
| **OfferingQueried** | User szuka compute | OPEN | Discovery query nie istnieje. | Private discovery protocol. |

---

## 5. Invariant Classification

| Invariant | Status | Why | Blockers | Test Implication |
|---|---|---|---|---|
| **Escrow lock wymaga zaakceptowanej polityki lease** | FROZEN_CANDIDATE | Kod już ma: escrow lock wymaga SpendPolicy commitment. Nowy ComputeLeaseEscrow ma lease_policy_commit jako pole. | Nowy SpendPolicy type. | Test: lock bez polityki = rejected. Lock z hash mismatch = rejected. |
| **Settlement wymaga receipts** | OPEN | V0 mówi że receipts są settlement evidence. Ale receipt infrastructure nie istnieje. W Phase 0/1 settlement nie wymaga receipts. | Receipt infrastructure. Metering protocol. | Test: settlement bez receipts = refund after timeout. |
| **Receipt jest signed przez miner role key** | OPEN | Receipt nie istnieje. Miner role key nie istnieje. | Receipt schema. Miner role key. | Test: unsigned receipt = invalid. Wrong signer = invalid. |
| **Receipt jest bound do session** | OPEN | Session nie istnieje. | Session management. | Test: receipt z wrong session_id = invalid. |
| **Privacy class jest enforced** | OPEN | Privacy class nie istnieje jako typ. | PrivacyClass definition. Runtime isolation spec. | Test: miner claims VM but delivers container = receipt invalid. |
| **Network mode jest enforced** | OPEN | Network mode istnieje jako koncepcja ale nie implemented. | NetworkMode definition. Runtime network spec. | Test: runtime has internet in Isolated mode = violation. |
| **Settlement jest deterministyczny** | FROZEN_CANDIDATE | Kod już ma deterministyczną walidację: frozen rule table, kanoniczna kolejność. | Pro-rata formula (musi być deterministyczna). | Test: same inputs → same outputs. Always. |
| **Operator jest bridge, nie decydent** | CANDIDATE | V0 mówi "operatorless by design." Kod ma operatora jako canonicalnego signera. Phase 1 zmienia logikę decyzji. | Operatorless Escrow Direction doc. | Test: automated operator produces same decision as rule table. |
| **Recovery wymaga timeout** | FROZEN_CANDIDATE | Kod ma timeout enforcement: reject_recovery_before_timeout test jest code-confirmed. | Brak. | Test: recovery before timeout = rejected. Test istnieje. |
| **Scoped offering ID jest rotowalny** | OPEN | ScopedOfferingId nie istnieje. | Private discovery. | Test: rotate ID → old ID no longer valid. |

---

## 6. Rejected / Too Early

| Element | Reason |
|---|---|
| **Public provider profiles** | V0 explicitly rejects. Private compute network, nie marketplace. |
| **Public marketplace discovery** | V0 explicitly rejects. Private/encrypted/credential-gated. |
| **Quality-based settlement** | V0 explicitly rejects. Settlement jest receipt/metering-based. |
| **Human dispute quorum** | V0 odrzuca. Settlement jest rule-bound. |
| **Reliability leaderboard (public)** | V0 mówi "selective proof," nie public broadcast. |
| **DHT discovery (na teraz)** | Za wcześnie. Nie znamy network size. Phase 8+. |
| **Gossip protocol (na teraz)** | Za wcześnie. Phase 8+. |
| **Traffic padding (na teraz)** | Za wcześnie. Phase 8+. |
| **Timing obfuscation (na teraz)** | Za wcześnie. Phase 8+. |
| **Onion multi-hop routing (na teraz)** | Za wcześnie. Wymaga relay node. Phase 8+. |
| **Full Halo2 privacy proof (na teraz)** | Za wcześnie. Scaffold istnieje. Phase 6+. |
| **Epoch key rotation protocol (na teraz)** | Za wcześnie. Phase 7. |
| **Bond/slash mechanism (na teraz)** | Za wcześnie. V0 mówi "future strengthening." Phase 7+. |
| **Third-party receipt attestation (na teraz)** | Za wcześnie. Phase 2+. |

---

## 7. Final Summary

### Co można prawie zamrozić (FROZEN_CANDIDATE):

1. **NetworkMode** enum (4 wartości) — V0 master explicitnie definiuje.
2. **User** jako compute lessee — V0 master explicitnie definiuje.
3. **Validator** jako chain security — code-confirmed, niezmieniony.
4. **EscrowLocked** event — mechanika lock jest code-confirmed.
5. **SettlementExecuted** event — mechanika execution jest code-confirmed.
6. **Settlement determinism** — frozen rule table jest code-confirmed.
7. **Recovery timeout enforcement** — test jest code-confirmed.

### Co wymaga code audit (BLOCKED_BY_CODE_AUDIT):

1. **LedgerAmount** — czy Amount14 jest rail-specific czy global? Audyt `primitives.rs`, `note.rs`, `escrow.rs`.
2. **EscrowNote (ComputeLeaseEscrow variant)** — czy SpendPolicy enum jest extensible? Czy validate_output_target obsługuje 2 outputs? Audyt `note.rs`, `escrow.rs`.
3. **IdentityBundle** — jak Falcon PK hash jest używany w consensus, escrow, transport? Audyt `node.rs`, `identity_provider.rs`, `escrow.rs`, `peers.rs`.

### Co wymaga decyzji operatora:

1. **LedgerAmount type** — u64 czy u128? Zależy od max supply PVA.
2. **MarketplaceBatchTx fate** — deprecated, renamed, repurposed?
3. **SpendPolicy::MarketplaceSettlement fate** — deprecated?
4. **Kill criteria dla Phase 1 automated operator** — ile miesięcy/settlements?

### Co wymaga Opusa:

1. **Operatorless Escrow Direction doc (T-032)** — definiuje Phase 0/1/2 bridge.
2. **Identity Model Direction doc (T-033)** — definiuje hidden root + scoped keys.
3. **Metering Protocol Direction doc (T-035)** — definiuje receipts, heartbeats, trust model.
4. **Private Discovery Direction doc** — definiuje discovery architecture.

### Co jest za wcześnie:

1. **Relay** jako encja — brak onion routing protocol.
2. **ExitNode** jako encja — brak risk/pricing model.
3. **PrivacyClass** granularity — brak definicji per-class guarantees.
4. **SettlementFormula** details — brak frozen formula, depends on aPVA.
5. **ScopedOfferingId** — meaningless bez discovery protocol.
6. **HeartbeatStatus** — meaningless bez metering protocol.
7. **MeteringUnits** — brak definicji jednostek.
8. **ComputeOffering** — meaningless bez discovery protocol.
9. **MeteringSession** — meaningless bez metering protocol.
10. **Większość eventów** (LeaseNegotiated, SessionStarted, ReceiptProduced, HeartbeatObserved, ChallengeIssued, ChallengeResponded, SessionTerminated, OfferingPublished, OfferingQueried) — zależą od infrastruktury która nie istnieje.

---

**Czy edytowano pliki:** NIE (poza zapisem tego pliku).
**Czy czytano legacy docs:** NIE
**Czy zdefiniowano wire formaty:** NIE
**Czy odpowiedź jest classification:** TAK
