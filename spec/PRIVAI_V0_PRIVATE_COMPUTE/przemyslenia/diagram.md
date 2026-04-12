# V0 Private Compute Network — Diagrams (Updated)

> **Date:** 2026-04-12
> **Companion to:** `PRIVAI_V0_SIMPLE_4_JAK_FINALNIE_WYGLADA_SYSTEM.md`, `PRIVAI_V0_SIMPLE_5_SYSTEM_CORE_DRAFT.md`
> **Status:** Visual companion updated after system core draft
> **Rule:** Where this doc and old `PRIVAI_PRODUCTION_SYSTEM_DIAGRAMS.md` conflict on framing, this doc wins.

---

## 1. V0 System Stack

```mermaid
flowchart TD
    U["User
    wallet keys
    private by default"] --> D["Private Discovery
    encrypted query via NXMS mailbox
    resource-based
    off-chain"]

    D --> T["Transport
    NXMS mailbox (envelope storage)
    Tor SOCKS5 (P2P direct)
    FrodoKEM + XChaCha20Poly1305"]

    T --> C["Private Compute Session
    VM / container / GPU slice
    window-based metering
    signed agent telemetry
    ephemeral by default"]

    C --> E["Chain
    zegar księgowy prywatności
    PVA settlement
    escrow lock/release/refund/pro-rata
    nullifiers + commitments
    blocks co ~30s"]

    E --> I["Incentives
    validators: block rewards
    compute miners: lease PVA
    mailboxes: storage PVA
    relays: routing PVA
    exit nodes: egress PVA (opt-in)"]
```

Rules:
- Chain is a privacy accountant — sees commitments, not workloads.
- Discovery is private via NXMS mailbox, not a public registry.
- Sessions are divided into windows — metering is window-based.
- Transport: FrodoKEM heavy once, XChaCha20Poly1305 fast after.

---

## 2. Node Roles

```mermaid
flowchart TD
    NO["Physical Machine
    (one machine / entity)"] --> V["Validator
    secures chain
    writes blocks
    earns block rewards"]

    NO --> CM["Compute Miner
    provides runtime slices
    runs agent (signed telemetry)
    earns lease PVA"]

    NO --> MB["Mailbox
    stores encrypted envelopes
    async delivery
    earns storage PVA"]

    NO --> R["Relay
    routes encrypted traffic
    sees only prev/next hop
    earns routing PVA (future)"]

    NO --> EX["Exit Node
    (explicit opt-in only)
    internet egress
    higher risk / higher reward
    earns egress PVA (future)"]
```

Rules:
- One machine may run multiple roles. Protocol must not merge them.
- Validator ≠ compute miner at protocol level. Two independent keys.
- Exit node is never default. Explicit opt-in only.
- Compute miner runs an agent daemon for signed telemetry.

---

## 3. Compute Lease Lifecycle

```mermaid
sequenceDiagram
    participant U as User
    participant NXMS as NXMS Mailbox
    participant CM as Compute Miner
    participant L as Chain
    participant AGENT as Miner Agent

    U->>NXMS: encrypted discovery query
    NXMS->>CM: delivery (mailbox cannot read content)
    CM->>NXMS: ComputeOffering response (encrypted)
    NXMS->>U: delivery

    U->>NXMS: lease acceptance (encrypted)
    NXMS->>CM: delivery
    U->>L: escrow lock (ComputeLeaseEscrow)

    Note over L: ESCROW LOCKED
    Note over L: lease_policy_commit + amount + timeout

    CM->>CM: provision VM/container/sandbox
    CM->>NXMS: session ready (encrypted credentials)
    NXMS->>U: delivery

    U->>CM: P2P connection via Tor (FrodoKEM handshake → XChaCha20Poly1305)
    Note over U,CM: Alice uses VM privately

    loop Every 60 blocks (≈30 min)
        L->>U: new block height (clock tick)
        U->>CM: challenge (based on block_hash — unpredictable)
        AGENT->>CM: signed telemetry (availability, performance)
        CM->>U: response + miner_signature
        U->>U: verify PASS/FAIL
    end

    Note over U: session ends

    CM->>NXMS: aggregate receipt + ZK proof (encrypted)
    NXMS->>U: delivery

    alt User accepts receipt
        U->>L: settlement claim
        L->>CM: earned PVA
        L->>U: remainder PVA
    else User disputes
        U->>L: dispute
        L->>CM: show window-by-window ZK proof
        L->>L: verify → loser pays dispute fee
    end
```

Rules:
- Discovery is private via NXMS mailbox.
- Escrow locks before session starts.
- Windows are off-chain — chain is only the clock (block height).
- Challenge uses block_hash — unpredictable before block, deterministic after.
- Receipt is aggregate (total_windows, passed_windows) + ZK proof.
- Settlement = passed/total × amount. Integer arithmetic. Remainder to user.

---

## 4. On-Chain vs Off-Chain Boundary

```mermaid
flowchart LR
    subgraph ON_CHAIN["Chain (zegar księgowy prywatności)"]
        OC1["Escrow lock (committed amount)"]
        OC2["Lease policy commitment (hash)"]
        OC3["Nullifiers (no double-spend)"]
        OC4["Settlement authorization signatures"]
        OC5["Receipt commitment (hash — not full receipt)"]
        OC6["Block height (clock for windows)"]
        OC7["Fee/reward distribution"]
    end

    subgraph OFF_CHAIN["Off-Chain (private / ephemeral)"]
        OF1["Prompts / workloads / outputs"]
        OF2["VM / container / session workspace"]
        OF3["Discovery queries and responses"]
        OF4["ComputeOfferings"]
        OF5["Window challenges and responses"]
        OF6["Agent telemetry (signed measurements)"]
        OF7["Full receipts (aggregate + per-window)"]
        OF8["Lease negotiation"]
        OF9["Miner profiles (none — no public profiles)"]
    end
```

Rules:
- Chain sees commitments, not workloads.
- Chain sees receipt hash, not full receipt.
- Chain sees block height, not window details.
- All off-chain data is ephemeral by default.
- Chain is the clock — windows are built on block heights.

---

## 5. Escrow Boundary

```mermaid
flowchart TD
    subgraph V0_MODEL["V0: Compute Lease Escrow"]
        V0A["ComputeLeaseEscrow SpendPolicy (tag 0x04)"]
        V0B["Settlement by window-based receipt"]
        V0C["Pro-rata = passed/total × amount"]
        V0D["Operatorless target"]
        V0E["Phase 1: Release + Refund sequence (bridge)"]
        V0F["Phase 2: ProRataSplit action (proper)"]
    end

    subgraph CURRENT_CODE["Current Code"]
        CC1["Escrow2of3 (bridge — untouched)"]
        CC2["Release / Refund / RecoveryRelease"]
        CC3["Operator co-signs Release/Refund"]
        CC4["Stage A / Stage B boundary (code-confirmed)"]
        CC5["All-or-nothing per action"]
        CC6["RecoveryRelease = already operatorless anchor"]
    end

    V0_MODEL -.->|"new SpendPolicy alongside"| CURRENT_CODE
    CURRENT_CODE -.->|"foundation"| V0_MODEL
```

Rules:
- Escrow2of3 stays untouched — it's the bridge.
- ComputeLeaseEscrow is a new SpendPolicy variant, not an extension.
- Phase 1 pro-rata = Release + Refund sequence (existing mechanics).
- Phase 2 pro-rata = ProRataSplit action (new mechanics).
- RecoveryRelease is the operatorless anchor — proof it works.

---

## 6. Window-Based Metering

```mermaid
sequenceDiagram
    participant L as Chain (clock)
    participant U as User
    participant CM as Compute Miner
    participant AGENT as Miner Agent

    Note over L,U,CM: Session starts at block N

    loop Every 60 blocks
        L->>L: new block produced
        L->>U: block_hash(height) is known
        U->>U: challenge = hash(session_id || window_id || block_hash)
        U->>CM: challenge via NXMS
        AGENT->>CM: signed telemetry snapshot
        CM->>CM: compute response
        CM->>U: response + miner_signature
        U->>U: verify response
        U->>U: availability check (response in timeout?)
        U->>U: performance check (time < benchmark_floor?)
        U->>U: window_pass = availability AND (performance OR N/A)
    end

    Note over U: Session ends at block N + (1440 × 60)

    CM->>U: aggregate receipt {total_windows, passed_windows, ZK proof}
    U->>U: compare with own data
```

Rules:
- Block height is the clock. Windows are built on blocks.
- Challenge uses block_hash — unpredictable before block, deterministic after.
- Miner agent provides signed telemetry (nvidia-smi, node_exporter, fio).
- User verifies independently.
- Receipt is aggregate, not per-window stream.
- ZK proof confirms consistency, not absolute truth.

---

## 7. Settlement Formula

```mermaid
flowchart TD
    R["Aggregate Receipt
    total_windows: 1440
    passed_windows: 1368
    degraded_windows: 40"]
    
    P["Lease Policy
    degraded_weight: 500 (50%)
    amount: 48 PVA"]
    
    R --> E["effective = passed + (degraded × weight / 1000)
    = 1368 + (40 × 0.5) = 1388"]
    
    P --> E
    
    E --> M["miner_share = amount × effective / total
    = 48 × 1388 / 1440 = 46 PVA"]
    
    E --> U["user_share = amount - miner_share
    = 48 - 46 = 2 PVA"]
    
    M --> S["Chain executes: 46 PVA → miner, 2 PVA → user"]
    U --> S
```

Rules:
- Integer arithmetic only. No floats.
- Remainder always goes to user.
- Degraded windows have configurable weight (default 50%).
- Formula is deterministic — same inputs always give same output.

---

## 8. Receipt Truth Architecture

```mermaid
flowchart TD
    AGENT["Miner Agent Daemon
    nvidia-smi + node_exporter + fio
    signed measurements
    hash-chained records"] --> TELEMETRY["Private Telemetry
    only user + miner see"]

    TELEMETRY --> ZK["ZK Proof
    proves consistency between
    telemetry and receipt
    hides raw measurements"]

    ZK --> RECEIPT["Aggregate Receipt
    total_windows
    passed_windows
    degraded_windows
    window_hashes_root
    miner_signature"]

    RECEIPT --> SETTLEMENT["Settlement
    passed/total × amount"]

    RECEIPT --> DISPUTE_PATH{"Dispute?"}
    
    DISPUTE_PATH -->|"no"| SETTLEMENT
    DISPUTE_PATH -->|"yes"| WINDOW["Window-by-Window ZK Proof
    per-window PASS/FAIL
    hides raw measurements
    reveals consistency"]

    WINDOW --> VERDICT["Chain verifies
    loser pays dispute fee"]
```

Rules:
- Agent is immutable daemon — signed, hash-chained, tamper-evident.
- ZK proof confirms consistency — does not prove absolute truth.
- Model must be sensible for ZK proof to be meaningful.
- Dispute resolution uses window-by-window proof.
- Benchmark tools (MLPerf, LINPACK, fio, iperf3) are existing industry standards.

---

## 9. Identity (Phase 0-5)

```mermaid
flowchart TD
    V["Validator
    Falcon PK from vault
    = node_pk_hash (frozen)"] --> CHAIN["Chain
    consensus voting
    block production"]

    CM["Compute Miner
    separate Falcon PK
    generated independently"] --> SIG["Signs
    receipts
    lease claims
    agent telemetry"]

    V -.->|"independent"| CM

    subgraph FUTURE["Phase 6+ (future)"]
        HR["Hidden Root Credential"]
        HR --> V_KEY["Validator Role Key"]
        HR --> CM_KEY["Compute Miner Role Key"]
        HR --> SESSION["Session Keys"]
        HR --> EPOCH["Epoch Keys"]
    end
```

Rules:
- Phase 0-5: two independent keys (validator + compute miner).
- Falcon is a signing tool, not identity.
- Validator identity is frozen — cannot be changed.
- Hidden root is Phase 6+ concern.
- Compute miner key is separate from validator key.

---

## 10. Transport

```mermaid
flowchart LR
    U["User"] -->|"NXMS envelope
    FrodoKEM → XChaCha20Poly1305
    Falcon signature"| MB["Mailbox
    stores encrypted envelopes
    cannot read content"]

    MB -->|"delivery"| CM["Compute Miner
    decrypts own session"]

    U2["User"] -->|"P2P via Tor
    FrodoKEM handshake (once)
    XChaCha20Poly1305 (streaming)"| CM2["Compute Miner
    VM session direct connection"]
```

Rules:
- NXMS mailbox: each envelope has own FrodoKEM. Heavy but independent.
- P2P direct: FrodoKEM handshake once, then XChaCha20Poly1305. Fast.
- Mailbox for: discovery, challenges, receipts.
- P2P for: VM session, streaming, terminal.
- Transport metadata hardening is future.

---

## 11. Protocol Version Domains

```mermaid
flowchart TD
    subgraph CHAIN["Chain-activated"]
        CV["chain_protocol_version (0x01)"]
        TV["tx_version (0x02)"]
        EP["escrow_policy_version (0x03)"]
        PS["proof_system_id (0x04)"]
    end

    subgraph NEGOTIATED["Session/handshake"]
        NT["nxms_transport_version (0x05)"]
        MP["mailbox_protocol_version (0x06)"]
    end

    subgraph DECLARED["Per offer/session"]
        CL["compute_lease_protocol_version (0x07)"]
        MV["meter_protocol_version (0x08)"]
        DP["discovery_protocol_version (0x09)"]
    end

    CHAIN --> RULE["Hard rule:
    No silent downgrade
    from FullPrivacy"]
    NEGOTIATED --> RULE
    DECLARED --> RULE
```

Rules:
- 9 version domains — all defined in `privai-chain/src/versioning.rs`.
- Each domain versioned independently.
- Chain-activated: by height or epoch.
- Handshake-negotiated: NXMS transport, mailbox.
- Declared per session: lease, meter, discovery.
- No silent downgrade from FullPrivacy. Ever. (enforced by `VersionRegistry::negotiate()`)

Code:
- `VersionDomain` enum (9 variants, tag 0x01-0x09)
- `VersionActivation` enum (ChainActivated, Handshake, DeclaredPerSession)
- `VersionRegistry` (get/set/negotiate, no-downgrade check)
- `VersionError::DowngradeRejected` (hard rule enforcement)
- 10 tests (roundtrip, activation, names, registry, no-downgrade)

---

## 12. What Is Rejected

```mermaid
flowchart LR
    subgraph REJECTED["NOT V0"]
        R1["Public AI marketplace"]
        R2["Public provider profile"]
        R3["Quality-of-answer settlement"]
        R4["Operator as canonical decision-maker"]
        R5["MarketplaceBatchTx as product center"]
        R6["Public reputation leaderboard"]
        R7["Artifact delivery as center"]
    end

    subgraph CANONICAL["V0"]
        C1["Private compute network"]
        C2["Window-based metering"]
        C3["Receipt / settlement by availability"]
        C4["Operatorless escrow by design"]
        C5["2 independent keys (Phase 0-5)"]
        C6["NXMS mailbox discovery"]
        C7["Chain = privacy accountant"]
    end

    REJECTED -->|"replaced by"| CANONICAL
```

---

*Document version: 2026-04-12. Updated after system core draft. Window-based metering, simple identity (2 keys), NXMS mailbox discovery, ZK proof receipt architecture, existing benchmark tools.*

