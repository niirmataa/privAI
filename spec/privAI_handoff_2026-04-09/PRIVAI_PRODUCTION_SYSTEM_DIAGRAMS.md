# privAI Production System Diagrams

This file is the visual companion to:
- `PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md`
- `PRIVAI_TOR_GATED_NETWORK_DIRECTION.md` (topology design note — diagrams here show direction, not final hop count)

It exists to keep the production direction easy to scan without re-reading long prose.

## 1. Production Stack [frozen direction]

```mermaid
flowchart TD
    A["Protocol Layer
    FullPrivacy coin
    Escrow 2-of-3
    NXMS transport
    Ledger enforcement"] --> B["Workspace Layer
    Files
    Agents
    Task graph
    Diffs / reviews
    Memory / terminal"]
    B --> C["Skill / Extension Layer
    Skill packs
    Verifier packs
    Role-based execution
    Contracts"]
    C --> D["Marketplace Layer
    Discovery
    Contract acceptance
    Private payment lock
    Delivery
    Verification
    Settlement"]
```

## 2. Service Lifecycle [frozen direction]

```mermaid
flowchart LR
    A["Browse"] --> B["Select"]
    B --> C["Contract"]
    C --> D["Lock"]
    D --> E["Execute"]
    E --> F["Deliver"]
    F --> G["Verify"]
    G --> H["Settle"]
```

Notes:
- `privAI` already has strong primitives for `Lock -> Execute -> Verify -> Settle`
- the weakest / least-defined areas are `Browse -> Select -> Contract`

## 3. Verification Model [frozen direction]

```mermaid
flowchart TD
    A["Level A
    Mechanical validation"] --> B["Level B
    Contract validation"]
    B --> C["Level C
    Semantic validation"]
    C --> D["Level D
    Settlement"]
```

### Level A
- build
- tests
- lint
- schema
- forbidden files untouched

### Level B
- task pill completed
- scope respected
- DoD met

### Level C
- human or reviewer-level judgment
- architecture fit
- semantic correctness

### Level D
- release
- refund
- recovery

## 4. Delivery vs Quality [frozen direction]

```mermaid
flowchart LR
    A["Proof of delivery"] --> B["Artifact exists"]
    A --> C["Artifact hash matches"]
    A --> D["Format / structure checks"]
    A --> E["SLA met"]

    Q["Proof of quality"] --> R["Semantic correctness"]
    Q --> S["Usefulness"]
    Q --> T["Architectural fit"]
```

Rule:
- proof of delivery is **not** proof of quality

## 5. Settlement Mapping [frozen direction]

```mermaid
flowchart TD
    A["Contract accepted"] --> B["Escrow lock"]
    B --> C["Provider executes"]
    C --> D["Artifact delivered"]
    D --> E{"Verification outcome"}
    E -->|accepted| F["Release"]
    E -->|rejected| G["Refund"]
    E -->|timeout / unresolved| H["Recovery"]
```

## 5A. On-Chain Privacy Model [frozen direction]

```mermaid
flowchart TD
    A["Transport privacy"] --> A1["Tor / KEM / mailbox / multi-hop"]
    B["On-chain privacy"] --> B1["Hidden amount commitment / ciphertext"]
    B --> B2["Committed policy details"]
    B --> B3["Encrypted recipient data"]
    B --> B4["Validity proofs without plaintext disclosure"]
```

Observer model:
- buyer + provider know the quoted price and contract terms off-chain
- observers see a valid settlement transition, not plaintext amount or recipient
- on-chain marketplace activity must preserve `privAI` privacy, not publish business metadata

## 6. Operator And Dispute Model [frozen direction]

```mermaid
flowchart TD
    A["Operator
    system program
    rule executor"] --> B["Normal Release / Refund
    require operator signature"]
    B --> C["Recovery stays peer path
    after timeout"]

    D["Dispute path
    future direction"] --> E["Independent quorum"]
    E --> F["Rule-based evaluation"]
    F --> G["Falcon-signed verdict"]
```

Rules:
- operator is not a casual moderator
- operator is a protocol-role executor
- normal Release / Refund are operator-signed
- future dispute quorum should be procedural, not discretionary

## 7. Frontend Direction [frozen direction]

```mermaid
flowchart TD
    A["Headless protocol / engine"] --> B["VS Code / VSCodium adapter"]
    A --> C["Terminal adapter (future)"]
    A --> D["Web adapter (future)"]
    A --> E["Other editor adapters (future)"]
```

Production direction:
- protocol-first
- first adapter in VS Code / VSCodium style environment
- no shell fork for now

## 8. Connectivity Modes [frozen direction]

```mermaid
flowchart LR
    A["Offline-capable mode"] --> B["Local model execution"]
    A --> C["Sandboxed local skill packs"]

    D["Online-capable mode"] --> E["Marketplace access"]
    D --> F["Remote models"]
    D --> G["Mailbox / Tor transport"]
    D --> H["P2P + KEM session paths"]
```

Rule:
- the system should be offline-capable, not offline-only
- the system should be online-capable, not online-required for every workflow

## 9. Trust Direction [frozen direction]

```mermaid
flowchart TD
    A["Trust inputs"] --> B["Stake / bond"]
    A --> C["Correct contract history"]
    A --> D["Time in system"]
    A --> E["Validator behavior"]
    A --> F["Dispute behavior"]
    A --> G["Availability / reliability"]
    B --> H["Future trust layer"]
    C --> H
    D --> H
    E --> H
    F --> H
    G --> H
    H --> I["Future weighted quorum selection"]
```

Rule:
- trust should not start as a naive score
- trust should be expensive to gain and expensive to lose

## 10. Execution Modes (A-D) [frozen direction]

```mermaid
flowchart TD
    subgraph A["Mode A: Local (sovereign)"]
        A1["User"] --> A2["Local GPU / own model"]
        A2 --> A3["Result"]
    end

    subgraph B["Mode B: Remote API (provider)"]
        B1["User"] -->|"NXMS / Tor"| B2["Provider model"]
        B2 --> B3["Result"]
    end

    subgraph C["Mode C: Sandbox (sovereign + remote)"]
        C1["User"] -->|"P2P / KEM"| C2["Sandbox VM
        user's model on
        provider's hardware"]
        C2 --> C3["Result"]
    end

    subgraph D["Mode D: Hybrid (split)"]
        D1["User"] -->|sensitive| D2["Local (A)"]
        D1 -->|heavy| D3["Sandbox (C)"]
        D1 -->|cheap| D4["Remote (B)"]
    end
```

Key:
- Mode A: no network, maximum privacy
- Mode B: provider sees prompt (decrypted for inference)
- Mode C: provider sees resource usage only, NOT content/model/prompts
- Mode D: routing logic defined in skill contract

## 11. Sandbox Network: ISOLATED (default) [frozen direction]

```mermaid
flowchart LR
    U["User"] <-->|"P2P / Tor / KEM"| S["Sandbox VM"]
    S -.-x I["Internet"]

    style I stroke-dasharray: 5 5
```

- VM has NO network interface except tunnel to user
- Provider sees: KEM traffic volume, CPU/GPU usage
- Provider does NOT see: content, model, prompts
- Internet: **NONE**

## 12. Sandbox Network: TOR-GATED (direction) [design direction]

```mermaid
flowchart LR
    U["User"] <-->|"P2P/Tor/KEM
    (work tunnel)"| S["Sandbox VM"]

    S -->|"encrypted route"| R["privAI relay
    path (one or more hops)"]
    R -->|"exit to Tor"| ER["exit-capable relay /
    validator (opt-in role)"]
    ER -->|"Tor circuit"| TOR["Tor network"]
    TOR --> INT["Internet"]
```

Visibility direction:

| Who | Sees | Does NOT see |
|-----|------|-------------|
| Provider | encrypted outbound traffic | destination, content, route details |
| privAI relay (any hop) | previous + next hop only | content, full route, contract details |
| Exit-capable relay / validator | traffic toward Tor / internet | origin identity, full route, content |
| Tor Exit | destination IP | provider identity, private route metadata |
| Destination | Tor Exit IP | everything else |

Frozen properties:
- Sandbox speaks ONLY privAI protocol (no direct Tor client in VM)
- Relay hop count and topology are network-design parameters (see `PRIVAI_TOR_GATED_NETWORK_DIRECTION.md`)
- Internet-exit is an explicit opt-in network role
- Each relay knows only its immediate neighbors (onion model)
- No single node learns the full path
- Provider cannot determine whether traffic is destined for the internet

## 13. FullPrivacy Marketplace / Generic Escrow Lifecycle [frozen direction]

```mermaid
sequenceDiagram
    participant B as Buyer
    participant M as Marketplace / Transport
    participant O as Operator
    participant L as Generic Ledger / Escrow
    participant PR as Provider

    PR->>M: publish discoverable offering off-chain
    B->>M: browse / discover off-chain
    B->>PR: negotiate + accept contract over encrypted transport
    B->>L: lock PVA in generic FullPrivacy escrow

    Note over L: ESCROW LOCKED
    Note over L: contract_commit + private amount commitment + timeout
    Note over L: no marketplace semantics on-chain

    PR->>PR: execute task
    PR->>M: deliver artifact / evidence off-chain
    PR->>L: optional generic delivery_commit if required by contract

    B->>B: verify (Level 1 → Level 2 → Level 3)

    alt Accepted
        B->>O: sign Release (Falcon)
        O->>L: validate + co-sign Release
        L->>PR: generic settlement output to provider
    else Rejected (provider agrees)
        PR->>O: sign Refund (Falcon)
        O->>L: validate + co-sign Refund
        L->>B: generic settlement output to buyer
    else Timeout / unresolved
        B->>L: Recovery (buyer_sig + provider_sig)
        Note over L: peer resolution, no operator
    end
```

Rules:
- marketplace is the off-chain application/protocol layer
- escrow is on-chain but generic
- the chain does not store skill pack, discovery, task text, or marketplace profile semantics
- `MarketplaceBatchTx` is optional aggregate rail, not the FullPrivacy marketplace baseline

## 14. Skill Pack Structure [frozen direction]

```mermaid
flowchart TD
    SP["Skill Pack"] --> META["metadata
    name, version
    locality: offline-capable /
    online-required / network-optional
    capability_requirements:
    reasoning, context_window, etc."]

    SP --> TC["task_contract
    input/output schema
    do-not-touch rules
    scope definition
    timeout"]

    SP --> VP["verifier_pack
    Level A: build, test, lint
    Level B: scope check, DoD
    Level C: human review recommended"]

    SP --> PR["pricing
    model: per-task-fixed /
    per-task-quoted /
    per-compute-unit
    estimate range"]

    SP --> SET["settlement_policy
    timeout_blocks
    delivery: hash-commit
    dispute: timeout → recovery"]
```

## 15. Full Network Topology (direction) [design direction]

```mermaid
flowchart TD
    U["User"] <-->|"P2P/Tor/KEM
    work channel"| S["Sandbox VM
    (provider machine)"]

    U <-->|"NXMS mailbox (Model B)
    control plane"| MB["Mailbox Server
    (store & forward)"]

    S -->|"encrypted multi-hop"| NA["privAI relay
    path (one or more hops)"]
    NA -->|"exit to Tor"| EV["exit-capable relay /
    validator (opt-in role)"]

    EV -->|"Tor circuit"| TOR["Tor Network"]

    TOR --> INT["Internet"]

    subgraph provider_machine ["Provider Machine"]
        S
    end

    subgraph privai_network ["privAI Network"]
        NA
        EV
        MB
    end
```

Provider visibility: KEM-encrypted packets leaving sandbox, CPU/GPU metrics.
Provider does NOT see: destinations, content, Tor traffic, routing path.
privAI relay visibility: only immediate predecessor and successor (onion model).
Exit-capable relay / validator visibility: traffic to route via Tor (not origin).
Separation of duty: Provider != relays != exit-capable relay / validator.

## 16. Rollout Phases [frozen direction]

```mermaid
flowchart LR
    P0["Phase 0 (NOW)
    Protocol layer
    escrow 3/3 e2e
    transport close
    mailbox wired"] --> P1["Phase 1
    Skill protocol
    task contract format
    verifier format
    (no UI needed)"]
    P1 --> P2["Phase 2
    Workspace adapter
    VS Code / VSCodium
    extension
    panels, task view
    works offline"]
    P2 --> P3["Phase 3
    Marketplace v1
    discovery
    per-task settlement
    compute rental
    operator-signed escrow"]
```

---

## 17. Verification Failure Paths [frozen direction]

```mermaid
flowchart TD
    D["Artifact delivered"] --> LA{"Level A
    Mechanical"}

    LA -->|PASS| LB{"Level B
    Contractual"}
    LA -->|FAIL| RESUB["Provider re-submits
    (new delivery_hash)
    Escrow stays locked
    No limit within timeout"]

    RESUB --> D

    LB -->|PASS| LC{"Level C
    Semantic"}
    LB -->|FAIL| FIX["Buyer requests fix
    OR requests Refund"]

    FIX -->|fix| RESUB
    FIX -->|refund| REF["Refund
    (provider signs)"]

    LC -->|ACCEPT| REL["Release
    (buyer signs)
    PVA → provider"]
    LC -->|REJECT| DISP{"Provider agrees?"}

    DISP -->|yes| REF
    DISP -->|no, timeout| REC["Recovery
    (buyer + provider sigs)
    peer resolution
    no operator"]
```

## 18. Operator Transition [frozen direction]

```mermaid
flowchart LR
    P0["Phase 0 (now)
    Operator = dev keypair
    centralized
    honest about trust"] --> P12["Phase 1-2
    Operator = automated service
    published rules
    still a keypair"]
    P12 --> P3["Later strengthening
    more protocol-side validation
    and / or quorum-assisted dispute
    without changing normal escrow semantics"]
```

## 19. Scope Change Protocol [frozen direction]

```mermaid
sequenceDiagram
    participant B as Buyer
    participant P as Protocol
    participant PR as Provider

    Note over B,PR: Scope changes mid-task

    B->>PR: "Scope changed, need requote"
    PR->>P: sign Refund on old escrow
    P->>B: PVA returned (old escrow closed)

    PR->>P: publish new contract (new scope, new price)
    B->>P: accept new contract
    B->>P: lock PVA in new escrow

    Note over P: New escrow, clean invariant
    Note over P: Locked amount is fixed per escrow
```

---

*Document version: 2026-04-10. Reflects 22 frozen decisions from PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md. Multi-hop sandbox network direction — exact relay topology is a design parameter, not a product invariant (see PRIVAI_TOR_GATED_NETWORK_DIRECTION.md).*
