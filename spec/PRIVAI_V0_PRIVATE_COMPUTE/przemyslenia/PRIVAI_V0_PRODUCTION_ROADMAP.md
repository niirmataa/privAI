# privAI V0: Production Roadmap — Co mamy, co brakuje, jak działa

**Status:** punkt po punkcie — od kodu do production  
**Data:** 2026-04-12  
**Zakres:** każdy element systemu: co mamy, co brakuje, jak działa, jak ma być w kodzie

---

## Zasada

Każdy punkt opisuje:
- **MAMY:** co już jest w kodzie (plik, testy)
- **BRAKUJE:** czego jeszcze nie ma
- **JAK DZIAŁA:** co ten element robi w systemie
- **JAK MA BYĆ W KODZIE:** co trzeba napisać, gdzie, jakie zależności

---

## 1. Types (klasy zasobów, polityka, receipt)

### MAMY
```
privai-chain/src/compute_lease.rs — 380+ linii, 8 tests

Zawartość:
  GpuClass enum (A100, H100, V100, T4, Generic)
  CpuTier enum (X86_64, Arm64, Generic)
  ResourceClass enum (Gpu, Cpu, Memory, Composite)
  PrivacyClass enum (Vm, Container, Sandbox, ConfidentialRuntime)
  NetworkMode enum (Isolated, NxmsOnly, TorGated, InternetExit)
  SettlementMode enum (AllOrNothing, ProRata)
  RoleType enum (Validator, ComputeMiner, Relay, Mailbox, ExitNode)
  HeartbeatStatus enum (Active, Missed, Terminated)
  ComputeLeasePolicy struct (14 pól, commitment(), effective_windows())
  ComputeOffering struct (12 pól)
  ComputeLeaseReceipt struct (9 pól, commitment())
  calculate_settlement() — integer arithmetic, remainder to user
  SettlementResult struct (miner_share, user_share, effective, total)

Testy:
  ✓ settlement_full_completion
  ✓ settlement_full_refund
  ✓ settlement_pro_rata
  ✓ settlement_with_degraded
  ✓ settlement_remainder_goes_to_user
  ✓ policy_commitment_is_deterministic
  ✓ receipt_commitment_is_deterministic
  ✓ enum_roundtrips
```

### BRAKUJE
- ResourceClass granularity (ile GPU classes? dokładne VMB values?)
- Benchmark floor definitions per class
- ComputeOffering validation logic

### JAK DZIAŁA
Types definiują "co" system operuje. Każda inna warstwa importuje te types. ComputeLeasePolicy jest hashed na chain. ComputeLeaseReceipt jest dowodem settlement. ResourceClass jest filter w discovery.

### JAK MA BYĆ W KODZIE
Gotowe. Importowane przez inne moduły jako `privai_chain::compute_lease::*`.

---

## 2. Escrow (ComputeLeaseEscrow)

### MAMY
```
privai-chain/src/compute_escrow.rs — 144 linii, 1 test

Zawartość:
  ComputeLeaseEscrowPolicy struct (tag 0x04, user/miner/operator pk_hash,
    lease_policy_commit, locked_amount, timeout_block_height)
  evaluate_settlement() — weryfikuje receipt, policz settlement
  CanonicalEncode — encoding zgodny z privai-chain pattern

Test:
  ✓ test_compute_lease_escrow_settlement (9/10 passed → 900/1000)
```

### BRAKUJE
- Walidacja w ledger (validate_compute_lease_escrow_auth)
- Routing: policy_tag 0x03 → existing, 0x04 → nowa funkcja
- ProRataSplit execution (1 input → 2 outputs)
- Stage A/B integration (automated operator buduje proposal)

### JAK DZIAŁA
User lockuje PVA w ComputeLeaseEscrow. Chain zapisuje policy commitment + amount + timeout. Po sesji, user submituje settlement claim z receipt. Chain weryfikuje receipt → evaluate_settlement() → dzieli pieniądze.

### JAK MA BYĆ W KODZIE
```
privai-ledger/src/escrow.rs:
  fn validate_compute_lease_escrow_auth(auth, policy_commit, outputs, height):
    1. Decode policy → must be ComputeLeaseEscrow
    2. Verify policy commitment matches input note
    3. Decode action (Release/Refund/ProRataSplit)
    4. Verify signer count (2 for bridge, 1+protocol for operatorless)
    5. Verify signer combination against rule table
    6. Recovery timeout check
    7. Falcon signature verification
    8. Output target validation (One or Two depending on action)

Routing w existing validate_transaction:
  match policy_tag:
    0x03 → validate_escrow_auth() [NIEZMIENIONA]
    0x04 → validate_compute_lease_escrow_auth() [NOWA]
```

---

## 3. Metering Agent

### MAMY
```
privai-node/src/metering.rs — 420+ linii, 11 tests

Zawartość:
  EnvironmentFingerprint (binary_hash, config_hash, os_kernel_hash, ...)
  StartupManifest (session_id, env_commit, miner_key, timestamp, signature)
  WindowTelemetryRecord (window_index, challenge_hash, response_time,
    availability, performance, heartbeat, metrics, previous_record_hash, signature)
  MeteringAgent (session state, window records, chain verification)
  MeteringAgent::new() — create agent for session
  MeteringAgent::record_window() — record single window (hash-chained)
  MeteringAgent::verify_chain() — verify hash chain integrity
  MeteringAgent::count_results() — count passed/degraded
  MeteringAgent::finalize() — produce ComputeLeaseReceipt with Merkle root

Testy:
  ✓ agent_records_windows_sequentially
  ✓ agent_rejects_non_sequential_index
  ✓ hash_chain_is_valid
  ✓ count_results_full_completion
  ✓ count_results_with_degraded
  ✓ count_results_n_a_performance_is_passed
  ✓ finalize_produces_receipt
  ✓ finalize_fails_on_empty_session
  ✓ finalize_fails_on_broken_chain
  ✓ receipt_matches_settlement_calculation
  ✓ window_hashes_root_is_deterministic
```

### BRAKUJE
- Real system monitoring (nvidia-smi, node_exporter, fio integration)
- Actual Falcon signing of records (nie placeholder)
- Storage persistence (hash chain na dysk, nie tylko w pamięci)
- Benchmark integration (real benchmarks per resource class)

### JAK DZIAŁA
Agent działa na maszynie minera. Co okno: zbiera measurements (GPU util, CPU, RAM, disk), podpisuje, dodaje do hash chain. Po sesji: finalize() → ComputeLeaseReceipt z Merkle root. Hash chain jest tamper-evident: przerwanie łańcucha → finalize() failuje.

### JAK MA BYĆ W KODZIE
```
privai-node/src/metering.rs — GOTOWE (struktury + logika)

Do dodania:
  1. nvidia-smi wrapper → gpu_utilization
  2. node_exporter client → cpu_utilization, ram_used_mb
  3. fio wrapper → disk benchmark (jeśli storage class)
  4. iperf3 wrapper → network benchmark (jeśli network mode ≠ isolated)
  5. Falcon signing → miner signs each record
  6. SQLite persistence → hash chain na dysk
```

---

## 4. Session Loop

### MAMY
```
privai-node/src/compute_session.rs — 104 linii

Zawartość:
  BlockClock trait — wait_for_next_block() → (hash, height)
  NetworkTransport trait — wait_for_challenge(), send_telemetry()
  HardwareBenchmark trait — run_benchmark() → ms
  ComputeSessionRunner<C, T, B> — generic over traits
  run_session_loop() — pętla okien: clock → challenge → benchmark → record → send
```

### BRAKUJE
- Real implementations of BlockClock (chain connection)
- Real implementations of NetworkTransport (NXMS/P2P)
- Real implementations of HardwareBenchmark (nvidia-smi, fio)
- Integration z MeteringAgent (obecnie agent jest w struct ale nie jest używany w loop)

### JAK DZIAŁA
Miner uruchamia sesję po otrzymaniu escrow lock. Pętla: czeka na blok → odbiera challenge → uruchamia benchmark → wysyła telemetry → powtarza. Po N oknach: finalize → receipt → settlement.

### JAK MA BYĆ W KODZIE
```
privai-node/src/compute_session.rs — GOTOWE (traits + loop)

Do dodania:
  impl BlockClock for ChainConnection { ... }  // connection do node
  impl NetworkTransport for NxmsTransport { ... } // NXMS mailbox
  impl HardwareBenchmark for SystemBenchmark { ... } // nvidia-smi + fio
```

---

## 5. Identity

### MAMY
```
privai-node/src/identity.rs — 29 linii

Zawartość:
  ValidatorRoleKey { falcon_pk_hash }
  ComputeMinerRoleKey { falcon_pk_hash, new(), ... }
  Dwa niezależne klucze. Phase 0-5 model.
```

### BRAKUJE
- Key generation (generate independent miner key)
- Key registration (miner registeruje swój klucz w systemie)
- Key signing (miner podpisuje receipts swoim ComputeMinerRoleKey)
- Hidden root (Phase 6+ — future)

### JAK DZIAŁA
Validator używa obecnego Falcon PK (frozen). Compute miner generuje osobny Falcon PK. Dwa klucze nie są powiązane. Miner podpisuje receipts, lease claims, agent telemetry swoim kluczem. User weryfikuje podpisy.

### JAK MA BYĆ W KODZIE
```
privai-node/src/identity.rs — GOTOWE (structs)

Do dodania:
  fn generate_miner_key() -> ComputeMinerRoleKey  // generacja
  fn sign_with_miner_key(data) -> Vec<u8>          // podpisywanie
  fn register_miner_key(key) -> RegistrationTx     // rejestracja (future)
```

---

## 6. Discovery

### MAMY
```
privai-nxms/src/discovery.rs — 128 linii, 1 test

Zawartość:
  DiscoveryQuery struct (min_resource_class, max_price, duration, network, privacy, response_kem_pk, expires)
  NxmsEnvelope struct (recipient_mailbox_id, kem_ciphertext, aead_nonce, encrypted_payload, auth_tag)
  CanonicalEncode for DiscoveryQuery (exact encoding matching ResourceClass)
```

### BRAKUJE
- ComputeOffering encoding (miner odpowiada ofertą)
- Mailbox endpoint /v1/discover (routing queries to miners)
- Query matching logic (which miners match the query?)
- Response encryption (miner szyfruje odpowiedź dla usera)

### JAK DZIAŁA
User szyfruje DiscoveryQuery → wysyła do NXMS mailbox → mailbox nie widzi treści → miners pullują → każdy próbuje odszyfrować → pasujący odpowiada ComputeOfferingiem → user otrzymuje matches.

### JAK MA BYĆ W KODZIE
```
privai-nxms/src/discovery.rs — GOTOWE (query + envelope)

Do dodania:
  1. ComputeOffering.encode() — matching ResourceClass encoding
  2. Mailbox /v1/discover endpoint — w nxms-mailbox
  3. Miner discovery handler — odbiera query, sprawdza match, odpowiada
  4. User discovery client — wysyła query, odbiera responses, filtruje
```

---

## 7. Transport (Relay)

### MAMY
```
nxms-transport/src/relay.rs — onion routing structs, 5 tests

Zawartość:
  RelayLayer (next_hop, next_hop_port, encrypted_payload, is_final)
  OnionMessage (outer_layer, first_hop_kem_ct, first_hop_nonce)
  RelayRoute (hops, destination, destination_port)
  RelayHop (peer_id, host, port, kem_pk)
  RelayForward (next_hop, payload, next_hop_kem_ct, next_hop_nonce, is_final)
  RelayEnvelope (kem_ct, nonce, encrypted_layer, tag)
```

### BRAKUJE
- Actual onion wrapping (build_onion z encryption)
- Relay processing (process_relay_layer z decryption)
- Integration z NXMS transport (wysyłanie onion message)
- Tor integration dla relay hops

### JAK DZIAŁA
User buduje onion message inside-out: plaintext → encrypt dla ostatniego hopa → wrap RelayLayer → encrypt dla poprzedniego → ... Każdy relay decrypts jedną warstwę, widzi next_hop + encrypted_payload. Nie widzi final destination ani plaintext.

### JAK MA BYĆ W KODZIE
```
nxms-transport/src/relay.rs — GOTOWE (structs)

Do dodania:
  fn build_onion(route, plaintext) -> OnionMessage  // wrapping
  fn process_relay_layer(encrypted, kem_sk) -> RelayForward  // unwrapping
  Relay integration z nxms-transport (send/receive onion messages)
```

---

## 8. Versioning

### MAMY
```
privai-chain/src/versioning.rs — 9 domen, 10 tests

Zawartość:
  VersionDomain enum (9 variants: ChainProtocol, TxVersion, EscrowPolicy, ProofSystem,
    NxmsTransport, MailboxProtocol, ComputeLease, MeterProtocol, DiscoveryProtocol)
  VersionActivation enum (ChainActivated, Handshake, DeclaredPerSession)
  Version(u16) — version number
  VersionRegistry (get/set/negotiate, no-downgrade enforcement)
  VersionError::DowngradeRejected

Testy:
  ✓ all_nine_domains_exist
  ✓ domain_roundtrip
  ✓ unknown_domain_returns_none
  ✓ activation_types_are_correct
  ✓ domain_names_are_stable
  ✓ registry_new_v1_has_all_domains
  ✓ registry_set_and_get
  ✓ no_downgrade_rejected
  ✓ version_ordering
  ✓ domain_name_stability_across_versions
```

### BRAKUJE
- Integration z chain (chain-activated versions activate by height)
- Integration z transport (handshake negotiation)
- Integration z lease (per-session declaration)

### JAK DZIAŁA
Każda domena jest wersjonowana niezależnie. Chain versions aktywują się przez height/epoch. Transport versions negocjują w handshake. Lease/meter/discovery versions są deklarowane per session. Twarda reguła: żadnego downgrade z FullPrivacy.

### JAK MA BYĆ W KODZIE
```
privai-chain/src/versioning.rs — GOTOWE (types + registry)

Do dodania:
  Chain integration: version activation by height
  Transport integration: handshake negotiation
  Lease integration: version declaration in ComputeLeasePolicy
```

---

## 9. ZK Proof Circuit

### MAMY
```
privai-proof/src/halo2/receipt_circuit.rs — circuit, 3 tests

Zawartość:
  ReceiptCircuitConfig (availability, performance, passed_count, degraded_count, count_selector)
  ReceiptConsistencyCircuit (total_windows, window_availability, window_performance, ...)
  ReceiptPublicInputs (total_windows, passed_windows, degraded_windows, merkle_root, miner_share, user_share)
  Constraint: passed_count[i] = passed_count[i-1] + availability[i]
  Constraint: degraded_count[i] = degraded_count[i-1] + avail * (1 - perf)
```

### BRAKUJE
- Full Halo2 prover/verifier integration
- Merkle proof constraints (hash chain verification in circuit)
- Settlement formula constraints (miner_share calculation in circuit)
- Public input verification (circuit outputs match claimed values)

### JAK DZIAŁA
Circuit bierze private inputs (window telemetry) i public inputs (aggregate claims). Dowodzi że public claims są konsystentne z private records. Nie ujawnia raw telemetry. Chain weryfikuje proof.

### JAK MA BYĆ W KODZIE
```
privai-proof/src/halo2/receipt_circuit.rs — GOTOWE (scaffold)

Do dodania:
  1. Merkle proof constraints w circuit
  2. Settlement formula constraints w circuit
  3. MockProver tests (verify constraints pass/fail)
  4. Real prover/verifier integration (future)
```

---

## 10. Ledger Integration (NIE MA — największy bloker)

### MAMY
- privai-ledger/src/escrow.rs — existing walidacja dla Escrow2of3 (757 linii, 20+ tests)

### BRAKUJE
- validate_compute_lease_escrow_auth() — nowa walidacja dla tag 0x04
- Routing w validate_transaction: 0x03 → existing, 0x04 → nowa
- ProRataSplit execution (1 input → 2 outputs)
- Receipt validation against lease policy

### JAK DZIAŁA
Chain otrzymuje transakcję z escrow. Match na policy_tag: 0x03 → existing walidacja, 0x04 → nowa walidacja. Nowa walidacja sprawdza: politykę, signers, action, output targets. Dla ProRataSplit: 2 outputs (miner + user).

### JAK MA BYĆ W KODZIE
```
privai-ledger/src/escrow.rs:
  NOWA FUNKCJA: validate_compute_lease_escrow_auth()
  NOWA GAŁĄŹ: match policy_tag { 0x03 => existing, 0x04 => new }
  NOWY VARIANT: validate_output_target dla Two(signer_a, signer_b)

NIE ZMIENIAĆ: existing validate_escrow_auth() dla Escrow2of3
```

---

## 11. Wallet Integration (NIE MA)

### MAMY
- privai-wallet/src/escrow_builder.rs — existing builder dla Escrow2of3 (724 linii)

### BRAKUJE
- Builder dla ComputeLeaseEscrow (nowy SpendPolicy)
- Receipt reception i verification
- Settlement claim building
- Discovery client integration

### JAK DZIAŁA
Wallet buduje ComputeLeaseEscrow transakcję z polityką lease. Odbiera receipts po sesji. Weryfikuje receipt vs swoje dane. Buduje settlement claim. Submituje do chaina.

### JAK MA BYĆ W KODZIE
```
privai-wallet/src/ — NOWY MODUŁ (compute_lease_client.rs lub rozszerzenie escrow_builder.rs)
  fn build_compute_lease_escrow(policy, miner_pk, amount) -> Transaction
  fn verify_receipt(receipt, my_data) -> Accept/Dispute
  fn build_settlement_claim(receipt, escrow_note) -> Transaction
  fn discover_compute(query) -> Vec<ComputeOffering>
```

---

## 12. Miner Runtime (NIE MA — największy bloker)

### MAMY
- compute_session.rs (session loop z traits)

### BRAKUJE
- VM/container provisioning
- nvidia-smi / node_exporter / fio integration
- Agent daemon (real system monitoring)
- Benchmark suite (MLPerf, LINPACK, fio, iperf3)
- Session lifecycle management (start, monitor, terminate)

### JAK DZIAŁA
Miner otrzymuje escrow lock → provisionuje VM/container → uruchamia agenta → sesja startuje → agent zbiera telemetry → po sesji finalize → receipt.

### JAK MA BYĆ W KODZIE
```
privai-compute-miner/ — NOWY CRATE (nie istnieje)
  src/runtime.rs       — VM/container provisioning
  src/agent_daemon.rs  — system monitoring (nvidia-smi, node_exporter, fio)
  src/benchmarks.rs    — benchmark suite (MLPerf, LINPACK, fio, iperf3)
  src/session.rs       — session lifecycle management

LUB:
  privai-node/src/compute_miner.rs — jeśli zostaje w node crate
```

---

## Summary

```
GOTOWE (testy przechodzą):     BRAKUJE (do zrobienia):
  1.  Types ✓                    10. Ledger integration
  2.  Escrow policy ✓            11. Wallet integration
  3.  Metering agent ✓           12. Miner runtime
  4.  Session loop ✓             13. Mailbox discovery endpoint
  5.  Identity ✓                 14. P2P session transport
  6.  Discovery query ✓          15. Automated operator
  7.  Relay structs ✓            16. Deprecation markers
  8.  Versioning ✓               17. End-to-end integration test
  9.  ZK circuit scaffold ✓      18. Benchmark suite

  76 tests, 0 regression         Największy bloker: miner runtime
```
