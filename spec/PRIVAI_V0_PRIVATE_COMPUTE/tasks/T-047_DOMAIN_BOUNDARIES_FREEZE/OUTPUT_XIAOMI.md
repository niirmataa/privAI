# P-T047-XIAOMI — Final Domain Boundaries Freeze Candidate

**Status:** domain boundaries freeze candidate
**Data:** 2026-04-11
**Źródło:** synteza audytów P-T040–P-T046
**Zakres:** finalne granice domenowe V0, gotowe do zamrożenia przed kodem

---

## 1. Boundary List

### 1.1 Chain / Consensus

| Field | Value |
|---|---|
| **Name** | Chain / Consensus |
| **Responsibility** | Konsensus, bloki, transakcje, nullifiers, commitment tree, fee, rewards, validator voting, block production |
| **Explicitly NOT responsible for** | Workload contents, receipts, metering, lease policy, quality judgment, provider profiles, discovery, runtime provisioning |
| **Input** | Transactions, nullifiers, proof/statement commitments, votes, fees |
| **Output** | Confirmed blocks, settled escrows, distributed rewards, state root |
| **Owner module candidate** | `privai-chain`, `privai-node` (consensus loop) |
| **Status** | FROZEN — code-confirmed. PC-BFT, stake-weighted voting, Falcon signatures. V0 nie zmienia konsensusu. |
| **Blockers** | Brak |

### 1.2 Escrow / Settlement

| Field | Value |
|---|---|
| **Name** | Escrow / Settlement |
| **Responsibility** | Lockuje środki (PVA/aPVA). Waliduje escrow auth (podpisy, polityka, signers). Wykonuje release/refund/recovery/pro-rata. Enforce timeouty. Stage A/B boundary. |
| **Explicitly NOT responsible for** | Produkcja receipts, metering, measurement delivery, workload execution, quality judgment, discovery, identity management |
| **Input** | SpendPolicy commitment, escrow lock amount, settlement claims, receipts (future), timeout triggers, signer auth |
| **Output** | Settlement execution (Release, Refund, RecoveryRelease, ProRataSplit), nullifiers, settlement authorization signatures |
| **Owner module candidate** | `privai-chain` (types), `privai-ledger` (validation), `privai-wallet` (building) |
| **Status** | STRONG_CANDIDATE — existing Escrow2of3 jest code-confirmed bridge. ComputeLeaseEscrow jako nowy SpendPolicy variant (tag 0x04) jest audytowany i rekomendowany (P-T042). |
| **Blockers** | ComputeLeaseEscrow SpendPolicy spec (Opus). Pro-rata note split spec (Opus). |

### 1.3 Metering / Receipts

| Field | Value |
|---|---|
| **Name** | Metering / Receipts |
| **Responsibility** | Mierzy dostarczone zasoby. Produkuje signed receipts. Obsługuje heartbeaty i challengi (future). Przechowuje receipts. |
| **Explicitly NOT responsible for** | Settlement execution, escrow mechanics, chain consensus, identity management, discovery, transport, runtime provisioning |
| **Input** | Resource measurements from runtime, miner Falcon signature, optional user ack, challenge requests (future) |
| **Output** | Signed ComputeLeaseReceipts, receipt commitment hash, challenge responses (future) |
| **Owner module candidate** | Nowy moduł `privai-metering` (nie istnieje) |
| **Status** | OPEN — receipt infrastructure nie istnieje. ComputeLeaseReceipt type jest rekomendowany (P-T045). Metering Protocol Direction doc (T-035) jest needed. |
| **Blockers** | Metering Protocol Direction doc (Opus/T-035). Receipt schema spec. Metering trust model (self-reported vs challenge). |

### 1.4 Lease Policy

| Field | Value |
|---|---|
| **Name** | Lease Policy |
| **Responsibility** | Definiuje warunki lease: resource class, duration, price, privacy class, network mode, settlement mode, meter version, timeout. Generuje commitment hash. |
| **Explicitly NOT responsible for** | Settlement execution, receipt production, identity management, discovery, transport, runtime provisioning, chain consensus |
| **Input** | ComputeOffering z discovery, user acceptance, miner confirmation |
| **Output** | ComputeLeasePolicy struct, lease_policy_commit (Hash32), settlement formula reference |
| **Owner module candidate** | `privai-chain` (types), `privai-wallet` (building) |
| **Status** | STRONG_CANDIDATE — ComputeLeasePolicy type jest rekomendowany (P-T045). Fundamentalny dla ComputeLeaseEscrow SpendPolicy commitment. |
| **Blockers** | Compute Lease Object spec (Opus). ResourceClass/PrivacyClass/NetworkMode granularity. |

### 1.5 Identity / Credentials

| Field | Value |
|---|---|
| **Name** | Identity / Credentials |
| **Responsibility** | Hidden root credential. Key derivation (root → role keys → session keys → epoch keys). Signing at each level. Scoped offering IDs. Selective reliability proof (future). |
| **Explicitly NOT responsible for** | Consensus voting (uses keys, doesn't define voting), settlement execution, receipt production, discovery, transport routing, runtime provisioning |
| **Input** | Root credential, role registration, session initiation, epoch rotation triggers |
| **Output** | Scoped keys per role/session/epoch, Falcon signatures, scoped offering IDs |
| **Owner module candidate** | `privai-node/src/identity_provider.rs` (extend), nowy moduł `privai-identity` (future) |
| **Status** | CANDIDATE — Falcon PK = ValidatorRoleKey (semantyczna zmiana, zero code impact). HiddenRootCredential additive later (P-T043). Identity Model Direction doc (T-033) needed. |
| **Blockers** | Identity Model Direction doc (Opus/T-033). Vault TLV format extension. Epoch/session key lifecycle. |

### 1.6 Discovery

| Field | Value |
|---|---|
| **Name** | Discovery |
| **Responsibility** | User znajduje compute capacity po resource class, cenie, privacy class, network mode. Miner publikuje ofertę. Private/encrypted/credential-gated. |
| **Explicitly NOT responsible for** | Lease negotiation, settlement, receipt production, identity management, transport routing, runtime provisioning, chain consensus |
| **Input** | ComputeOfferings from miners, DiscoveryQuery from users |
| **Output** | Matching ComputeOfferings ze scoped offering IDs, selective reliability proofs |
| **Owner module candidate** | Nowy moduł `privai-discovery` (nie istnieje). NXMS mailbox jako transport base. |
| **Status** | OPEN — discovery protocol nie istnieje. NXMS mailbox jest rekomendowanym transport base (P-T046). Private Discovery Direction doc needed. |
| **Blockers** | Private Discovery Direction doc (Opus). ComputeOffering/DiscoveryQuery/DiscoveryResponse types (za wcześnie — P-T045). |

### 1.7 Transport / Mailbox / Relay

| Field | Value |
|---|---|
| **Name** | Transport / Mailbox / Relay |
| **Responsibility** | Wysyła/odbiera encrypted envelopes (FrodoKEM + XChaCha20Poly1305 + Falcon). Mailbox przechowuje/dostarcza (push/pull/ack). Relay routuje (onion model — future). Tor-gated connections. |
| **Explicitly NOT responsible for** | Content visibility (nie widzi treści), settlement, receipt production, identity management, discovery, runtime provisioning, chain consensus |
| **Input** | Encrypted envelopes from users/miners, routing requests |
| **Output** | Delivered encrypted envelopes, relay hops (future), Tor-gated connections |
| **Owner module candidate** | `nxms-transport` (existing), `nxms-mailbox` (existing) |
| **Status** | STRONG_CANDIDATE — NXMS transport + mailbox jest code-confirmed base. Onion routing i metadata hardening są future (Phase 8+). |
| **Blockers** | Transport/Mailbox Privacy Direction doc (Opus). Onion routing protocol (future). |

### 1.8 Runtime / Compute

| Field | Value |
|---|---|
| **Name** | Runtime / Compute |
| **Responsibility** | Provisionuje izolowany runtime slice (VM/container/sandbox/GPU). Wykonuje workload usera. Resource usage tracking. Ephemeral by default. |
| **Explicitly NOT responsible for** | Settlement, receipt signing (metering produces receipts, not runtime), identity management, discovery, transport, chain consensus |
| **Input** | ComputeLease, ComputeLeasePolicy, workload from user |
| **Output** | Computed results (off-chain, ephemeral), resource usage metrics (input do metering) |
| **Owner module candidate** | Nowy moduł `privai-compute-miner` (nie istnieje) |
| **Status** | OPEN — runtime nie istnieje. Privacy class granularity nie jest zdefiniowana. Runtime Privacy Classes Direction doc needed. |
| **Blockers** | Runtime Privacy Classes Direction doc (Opus). VM/container/sandbox isolation spec. ComputeMiner moduł. |

### 1.9 Wallet / Client

| Field | Value |
|---|---|
| **Name** | Wallet / Client |
| **Responsibility** | Zarządza kluczami usera. Buduje transakcje (escrow lock, settlement claims). Składa roszczenia. Trzyma receipts. Client-side discovery queries. |
| **Explicitly NOT responsible for** | Consensus, block production, receipt production (trzyma, nie produkuje), runtime provisioning, transport routing, identity derivation (używa, nie definiuje) |
| **Input** | User keys, lease policy, receipts, discovery results |
| **Output** | Transactions (escrow lock, settlement claims), stored receipts, discovery queries |
| **Owner module candidate** | `privai-wallet` (existing — extend) |
| **Status** | STRONG_CANDIDATE — wallet istnieje, ma escrow_builder, builder, state management. V0 dodaje nowe SpendPolicy support. |
| **Blockers** | ComputeLeaseEscrow SpendPolicy support w wallet. LedgerAmount support. |

### 1.10 RAG / MCP / Agent Context

| Field | Value |
|---|---|
| **Name** | RAG / MCP / Agent Context |
| **Responsibility** | Utrzymuje jedno źródło prawdy dla agentów (Xiaomi, Opus, Codex, future). V0-only RAG. Golden tests. Controlled context server. |
| **Explicitly NOT responsible for** | Protokół, settlement, identity, discovery, transport, runtime, chain — nie robi nic z tych rzeczy, tylko dostarcza kontekst |
| **Input** | V0 docs, task log, prompt log, docs tree, context plan |
| **Output** | Reading order, guardrails, task context packs, correction pills, current status, question routing |
| **Owner module candidate** | `privai-context-mcp` (planned, nie implemented) |
| **Status** | FROZEN — Single Source Of Truth Context Plan i MCP Server Direction są frozen. Implementation blocked until direction baseline complete. |
| **Blockers** | MCP implementation (blocked). Direction baseline completion. |

---

## 2. Boundary Invariants

### Chain / Consensus
1. Chain nie widzi workloadów, promptów, outputów, model names.
2. Chain nie ocenia jakości AI outputu.
3. Chain nie widzi full receipts — tylko commitment hashes.
4. Validator identity = Falcon PK hash (frozen).
5. Consensus nie zmienia się dla V0.

### Escrow / Settlement
1. Escrow lock wymaga SpendPolicy commitment.
2. Settlement jest deterministyczny — same inputs → same outputs.
3. Recovery wymaga timeout — nie można recovery przed timeout_block.
4. Escrow2of3 jest bridge — nie zmieniać.
5. ComputeLeaseEscrow jest additive — nie wpływa na Escrow2of3.
6. Pro-rata jest future — all-or-nothing na teraz.
7. Operator jest bridge (Phase 0/1), nie canonical decision-maker (Phase 2).

### Metering / Receipts
1. Receipt jest signed przez miner role key.
2. Receipt jest bound do session.
3. Receipt nie zawiera workload contents.
4. Receipt jest dowodem delivery, nie jakości.
5. Self-reported receipts = honest-but-curious (Phase 1). Challenge/response = stronger (Phase 2).
6. Receipt storage = both parties independently.

### Lease Policy
1. Polityka jest agreed przed escrow lock.
2. Polityka jest hashed na chain (commitment), nie plaintext.
3. Polityka definiuje: resource, duration, price, privacy, network, settlement mode, meter version, timeout.
4. Zmiana polityki = nowa polityka commitment = nowy escrow.

### Identity / Credentials
1. Hidden root jest nigdy exposeowany.
2. Falcon jest narzędziem podpisu, nie publiczną tożsamością.
3. Obecny Falcon PK = ValidatorRoleKey (semantyczna zmiana).
4. Compute miner ma osobny klucz od validatora.
5. Session keys są ephemeral — discarded after session.
6. Epoch keys rotują periodicznie.
7. Scoped offering IDs są rotowalne, nie-linked z root.

### Discovery
1. Discovery jest private/encrypted/credential-gated jako baseline.
2. Brak publicznego registry jako default.
3. Brak public provider profiles.
4. Brak public reputation leaderboard.
5. User szuka po resource class, nie po reputacji.
6. Scoped offering ID nie ujawnia root identity.

### Transport / Mailbox / Relay
1. Mailbox nie widzi treści encrypted envelopes.
2. Relay widzi tylko prev/next hop (onion model — future).
3. Tor hides user IP from exit node.
4. Transport jest encrypted (FrodoKEM + XChaCha20Poly1305 + Falcon).
5. Content privacy ≠ metadata privacy (metadata hardening jest future).

### Runtime / Compute
1. Runtime jest ephemeral by default.
2. Miner nie widzi plaintext workload (VM = strongest isolation).
3. Privacy class jest deklarowany, nie assumed.
4. Network mode jest enforced — runtime nie może mieć więcej dostępu niż deklarowany.
5. Persistent storage jest explicit opt-in.

### Wallet / Client
1. Wallet buduje transakcje, nie produkuje receipts.
2. Wallet trzyma receipts, nie podpisuje jako miner.
3. Wallet jest client-side — nie robi consensusu.
4. Wallet używa existing TransferNoteTx proof path.

### RAG / MCP / Agent Context
1. V0-only — brak legacy docs w kontekście.
2. Golden tests egzekwują poprawne odpowiedzi.
3. MCP jest read-only (na teraz).
4. MCP jest source of truth, nie implementation spec.

---

## 3. Boundary Anti-Patterns

| Anti-Pattern | Why Wrong |
|---|---|
| **Metering settles** | Metering produkuje receipts. Settlement wykonuje escrow. Dwa różne boundaries. |
| **Transport understands lease policy** | Transport dostarcza envelopes. Nie rozumie treści. Lease policy jest application layer. |
| **Discovery exposes public identity** | Discovery jest private/resource-based. Public identity = marketplace model = rejected. |
| **Chain sees workload** | Chain widzi commitmenty i receipts. Nie workloady, prompty, outputy. |
| **Identity decides trust** | Identity provides keys. Trust jest decided by protocol (receipt validation, challenge/response), nie identity. |
| **Runtime signs receipts** | Metering podpisuje receipts. Runtime produkuje measurements. Dwa różne boundaries. |
| **Wallet validates receipts** | Ledger waliduje. Wallet buduje transakcje. Separation of concerns. |
| **Consensus uses compute miner identity** | Consensus używa validator identity. Compute miner jest osobną rolą. |
| **Escrow sees metering internals** | Escrow widzi receipts (dowód). Nie widzi jak metering mierzył. |
| **Discovery stores lease history** | Discovery jest ephemeral query. Lease history jest privacy-sensitive. Nigdy w discovery. |
| **Operator decides settlement** | Operator (Phase 1) validates receipts mechanically. Nie decyduje discretionary. Protocol (Phase 2) waliduje bez operatora. |
| **Exit node is default** | Exit jest opt-in, nigdy default. Default exit = privacy leak. |
| **MCP contains legacy docs** | MCP jest V0-only. Legacy = quarantine. |

---

## 4. Freeze Recommendation

### Boundaries do zamrożenia TERAZ (7):

| Boundary | Dlaczego freeze teraz |
|---|---|
| **Chain / Consensus** | Code-confirmed. V0 nie zmienia. Frozen. |
| **Escrow / Settlement (Escrow2of3 bridge)** | Code-confirmed. Frozen rule table. 20+ tests. |
| **RecoveryRelease as operatorless anchor** | Code-confirmed. Test exists. Frozen. |
| **Transport / Mailbox (base)** | Code-confirmed. NXMS + mailbox działa. |
| **Wallet / Client (base)** | Code-confirmed. Escrow builder, builder, state management. |
| **RAG / MCP / Agent Context** | Direction frozen. Context plan frozen. MCP direction frozen. |
| **Exit node opt-in only** | Explicitly defined in V0 master. Frozen. |

### Boundaries do zamrożenia PO SPEC (5):

| Boundary | Dlaczego czeka | Blocked by |
|---|---|---|
| **Escrow / Settlement (ComputeLeaseEscrow)** | Nowy SpendPolicy variant wymaga spec. | ComputeLeaseEscrow spec (Opus) |
| **Lease Policy** | ComputeLeasePolicy fields wymagają spec. | Compute Lease Object spec (Opus) |
| **Metering / Receipts** | Receipt schema, trust model, challenge/response wymagają spec. | Metering Protocol Direction (Opus/T-035) |
| **Identity / Credentials** | Hidden root, role keys, session/epoch lifecycle wymagają spec. | Identity Model Direction (Opus/T-033) |
| **Discovery** | Protocol, ComputeOffering, query model wymagają spec. | Private Discovery Direction (Opus) |

### Boundaries za wcześnie (2):

| Boundary | Dlaczego za wcześnie |
|---|---|
| **Runtime / Compute** | Privacy class granularity nie jest zdefiniowana. VM/container/sandbox isolation nie jest spec'd. ComputeMiner moduł nie istnieje. |
| **Relay (w transport)** | Onion routing protocol nie istnieje. Relay jako separate role nie jest zdefiniowany. Phase 8+. |

---

## Final Self-Check

- **Czy czytałem legacy docs:** NIE
- **Czy czytałem kod:** TAK (w poprzednich audytach P-T040–P-T046)
- **Czy edytowałem pliki inne niż output:** NIE (tylko OUTPUT_XIAOMI.md)
- **Czy definiowałem wire formaty:** NIE
