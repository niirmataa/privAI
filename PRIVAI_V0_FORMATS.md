# privAI V0 Concrete Formats (Updated)

**Status:** updated to current system state  
**Data:** 2026-04-12  
**Zakres:** konkretne formaty obiektów i payloadów dla privAI V0 — zgodne z kodem

---

## 1. Canonical primitives

Wszystkie hashe: BLAKE3.  
Wszystkie liczby: little-endian.  
Wszystkie ciągi bajtów: `u32_len || bytes`.

```
Hash32      = [u8; 32]
BundleId    = [u8; 16]
ContextId   = [u8; 16]
BlockHeight = u64
Amount14    = u16   where value < 16384  (proof lane only)
LedgerAmount = u64  (escrow/settlement economics)
Flags8      = u8
```

Funkcje domenowe:

```
H_note(x)      = BLAKE3("privai:note:v0"                || x)
H_policy(x)    = BLAKE3("privai:policy:v0"               || x)
H_bundle(x)    = BLAKE3("privai:bundle:v0"               || x)
H_nullifier(x) = BLAKE3("privai:nullifier:v0"            || x)
H_stmt(x)      = BLAKE3("privai:stmt:v0"                 || x)
H_aux(x)       = BLAKE3("privai:aux:v0"                  || x)
H_falcon(x)    = BLAKE3("privai:falcon-pk:v0"            || x)
H_lease(x)     = BLAKE3("privai:compute-lease-policy:v1" || x)
H_receipt(x)   = BLAKE3("privai:compute-lease-receipt:v1"|| x)
H_window(x)    = BLAKE3("privai:window-telemetry:v1"     || x)
H_env(x)       = BLAKE3("privai:env-fingerprint:v1"      || x)
H_manifest(x)  = BLAKE3("privai:startup-manifest:v1"     || x)
```

---

## 2. Resource Classification

```rust
enum GpuClass { A100 = 0x01, H100 = 0x02, V100 = 0x03, T4 = 0x04, Generic = 0xFF }
enum CpuTier  { X86_64 = 0x01, Arm64 = 0x02, Generic = 0xFF }

enum ResourceClass {
    Gpu { class: GpuClass, vram_mb: u32 },              // tag 0x01
    Cpu { tier: CpuTier, cores: u16 },                  // tag 0x02
    Memory { ram_mb: u32 },                             // tag 0x03
    Composite { gpu, cpu, ram_mb, storage_mb },          // tag 0x04
}

enum PrivacyClass { Vm = 0x01, Container = 0x02, Sandbox = 0x03, ConfidentialRuntime = 0x04 }
enum NetworkMode  { Isolated = 0x01, NxmsOnly = 0x02, TorGated = 0x03, InternetExit = 0x04 }
enum SettlementMode { AllOrNothing = 0x01, ProRata = 0x02 }
enum RoleType { Validator = 0x01, ComputeMiner = 0x02, Relay = 0x03, Mailbox = 0x04, ExitNode = 0x05 }
enum HeartbeatStatus { Active = 0x01, Missed = 0x02, Terminated = 0x03 }
```

---

## 3. ComputeLeasePolicy

```rust
struct ComputeLeasePolicy {
    version: u8,
    resource_class: ResourceClass,
    min_duration_units: u64,
    max_duration_units: u64,
    price_aPVA_per_unit: LedgerAmount,
    privacy_class: PrivacyClass,
    network_mode: NetworkMode,
    settlement_mode: SettlementMode,
    meter_version: u8,
    timeout_blocks: u64,
    window_duration_blocks: u64,
    total_windows: u32,
    benchmark_floor_ms: u32,
    benchmark_interval: u32,
    degraded_weight_permille: u16,
}

commitment = H_lease(canonical(ComputeLeasePolicy))
```

---

## 4. ComputeOffering

```rust
struct ComputeOffering {
    resource_class: ResourceClass,
    price_aPVA_per_unit: LedgerAmount,
    min_duration_units: u64,
    max_duration_units: u64,
    network_mode: NetworkMode,
    privacy_class: PrivacyClass,
    scoped_offering_id: Hash32,
    availability_start: u64,
    availability_end: u64,
    meter_version: u8,
    miner_role_key_hash: Hash32,
    benchmark_floor_ms: u32,
}
```

---

## 5. DiscoveryQuery

```rust
struct DiscoveryQuery {
    min_resource_class: ResourceClass,
    max_price_aPVA_per_unit: LedgerAmount,
    min_duration_units: u64,
    preferred_network_mode: NetworkMode,
    preferred_privacy_class: PrivacyClass,
    response_kem_pk: Vec<u8>,
    expires_at_unix: u64,
}
```

---

## 6. ComputeLeaseReceipt

```rust
struct ComputeLeaseReceipt {
    session_id: Hash32,
    total_windows: u32,
    passed_windows: u32,
    degraded_windows: u32,
    window_hashes_root: Hash32,     // Merkle root of per-window hashes
    lease_policy_commit: Hash32,
    miner_role_key_hash: Hash32,
    meter_version: u8,
    miner_signature: Vec<u8>,       // Falcon
}

commitment = H_receipt(canonical(ComputeLeaseReceipt))
```

---

## 7. Window Telemetry (off-chain, hash-chained)

```rust
struct WindowTelemetryRecord {
    window_index: u32,
    window_start_height: u64,
    challenge_hash: Hash32,
    response_time_ms: u32,
    availability: bool,
    performance: Option<bool>,   // None = N/A (no benchmark this window)
    heartbeat: HeartbeatStatus,
    gpu_utilization: Option<u8>,
    cpu_utilization: Option<u8>,
    ram_used_mb: Option<u32>,
    previous_record_hash: Hash32,  // hash chain
    miner_signature: Vec<u8>,      // Falcon
}

record_hash = H_window(canonical(WindowTelemetryRecord))
```

---

## 8. Startup Manifest (off-chain)

```rust
struct EnvironmentFingerprint {
    binary_hash: Hash32,
    config_hash: Hash32,
    os_kernel_hash: Hash32,
    gpu_driver_hash: Option<Hash32>,
    compute_runtime_hash: Option<Hash32>,
    process_hashes: Vec<Hash32>,
    disk_fingerprint: Hash32,
    timestamp_unix: u64,
}

struct StartupManifest {
    session_id: Hash32,
    env_fingerprint_commit: Hash32,
    miner_role_key_hash: Hash32,
    start_timestamp_unix: u64,
    start_block_height: u64,
    miner_signature: Vec<u8>,      // Falcon
}

env_commit = H_env(canonical(EnvironmentFingerprint))
manifest_commit = H_manifest(canonical(StartupManifest))
```

---

## 9. SpendPolicy (updated)

```rust
enum SpendPolicy {
    Single { falcon_pk_hash: Hash32 },                              // tag 0x01
    MarketplaceSettlement { ... },                                  // tag 0x02 — DEPRECATED
    Escrow2of3 { buyer, merchant, operator, timeout_block },        // tag 0x03 — BRIDGE
    ComputeLeaseEscrow {                                            // tag 0x04 — NEW
        user_pk_hash: Hash32,
        miner_pk_hash: Hash32,
        lease_policy_commit: Hash32,
        timeout_block: u64,
        settlement_mode: SettlementMode,
    },
}

commitment = H_policy(canonical(SpendPolicy))
```

---

## 10. EscrowAction (updated)

```rust
enum EscrowAction {
    Release = 0x01,
    Refund = 0x02,
    RecoveryRelease = 0x03,     // already operatorless
    ProRataSplit = 0x04,        // NEW — 1 input → 2 outputs
}
```

---

## 11. TargetRecipient (updated)

```rust
enum TargetRecipient {
    One(SignerRole),                  // output to one recipient
    Either(SignerRole, SignerRole),   // output to either (recovery)
    Two(SignerRole, SignerRole),      // output to both (pro-rata) — NEW
}
```

---

## 12. Settlement

```rust
fn calculate_settlement(total_amount, receipt, policy) -> SettlementResult {
    effective = receipt.passed + (receipt.degraded * policy.degraded_weight / 1000)
    miner_share = total_amount * effective / receipt.total_windows
    user_share = total_amount - miner_share
    // integer arithmetic, remainder to user
}

struct SettlementResult {
    miner_share: LedgerAmount,
    user_share: LedgerAmount,
    effective_windows: u64,
    total_windows: u64,
}
```

---

## 13. ComputeLeaseEscrowPolicy (on-chain)

```rust
struct ComputeLeaseEscrowPolicy {
    tag: u8,                         // 0x04
    user_pk_hash: Hash32,
    miner_pk_hash: Hash32,
    operator_pk_hash: Hash32,        // Phase 1 bridge only
    lease_policy_commit: Hash32,
    locked_amount: LedgerAmount,
    timeout_block_height: u64,
}

fn evaluate_settlement(receipt, policy) -> SettlementResult {
    // 1. Verify receipt matches locked policy
    // 2. Verify miner identity
    // 3. Calculate deterministic split
}
```

---

## 14. Identity (Phase 0-5)

```rust
struct ValidatorRoleKey { falcon_pk_hash: Hash32 }     // frozen, from vault
struct ComputeMinerRoleKey { falcon_pk_hash: Hash32 }   // independent, generated

// Two independent keys. No hidden root (Phase 6+).
// Falcon is signing tool, not identity.
```

---

## 15. Version Domains

```rust
enum VersionDomain {
    ChainProtocol     = 0x01,  // chain-activated
    TxVersion         = 0x02,  // chain-activated
    EscrowPolicy      = 0x03,  // chain-activated
    ProofSystem       = 0x04,  // chain-activated
    NxmsTransport     = 0x05,  // handshake
    MailboxProtocol   = 0x06,  // handshake
    ComputeLease      = 0x07,  // declared per session
    MeterProtocol     = 0x08,  // declared per session
    DiscoveryProtocol = 0x09,  // declared per session
}

// Hard rule: no silent downgrade from FullPrivacy.
// Enforced by VersionRegistry::negotiate().
```

---

## 16. Relay (onion routing)

```rust
struct RelayLayer {
    next_hop: String,
    next_hop_port: u16,
    encrypted_payload: Vec<u8>,
    is_final: bool,
}

struct RelayEnvelope {
    kem_ct: Vec<u8>,
    nonce: Vec<u8>,
    encrypted_layer: Vec<u8>,
    tag: Vec<u8>,
}

struct RelayRoute {
    hops: Vec<RelayHop>,
    destination: String,
    destination_port: u16,
}
```

---

## 17. On-Chain vs Off-Chain

**On-chain:**
- OutputNote (encrypted, Amount14)
- Nullifier
- TransferNoteTx
- ComputeLeaseEscrowPolicy
- Lease policy commitment (hash)
- Receipt commitment (hash)
- Settlement result
- Block height (clock)

**Off-chain:**
- ComputeLeasePolicy (full)
- ComputeOffering
- DiscoveryQuery / Response
- Window telemetry records (hash-chained)
- Startup manifest + environment fingerprint
- Aggregate receipt (full)
- Prompts / workloads / outputs
- VM session data

---

## 18. Deprecated (legacy marketplace)

```
MarketplaceBatchTx     — DEPRECATED, stays in code for compatibility
MarketplaceSettlement  — DEPRECATED, ledger rejects in FullPrivacy
MarketOfferBody        — DEPRECATED
MarketAcceptBody       — DEPRECATED
InferenceRequestBody   — DEPRECATED
InferenceResponseBody  — DEPRECATED
```

---

## 19. What is frozen

- `q = 2^32 - 5`, `p = 2^14`
- Note-based ledger
- Nullifier = H_nullifier(note_commit || nullifier_key)
- RecipientBox jako on-chain encrypted delivery
- statement_commit jako binding tx-proof
- falcon_pk_hash = BLAKE3("privai:falcon-pk:v0" || pk_bytes)
- Escrow2of3 as bridge (untouched)
- RecoveryRelease as operatorless anchor
- 9 version domains

---

*Document version: 2026-04-12. Updated from legacy marketplace formats to V0 private compute model. Matches current code state.*
