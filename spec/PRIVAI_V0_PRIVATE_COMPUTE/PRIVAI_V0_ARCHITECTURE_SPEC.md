# privAI V0 Private Compute Network — System Architecture Specification

> **Date:** 2026-04-12
> **Status:** Canonical Architecture Specification for V0
> **Based on:** `PRIVAI_V0_SIMPLE_4_JAK_FINALNIE_WYGLADA_SYSTEM.md` and `diagram.md`

---

## 1. Introduction & Core Philosophy

The privAI V0 Private Compute Network is a decentralized, privacy-preserving protocol for leasing computational resources (GPU, CPU, RAM). It fundamentally rejects the "public marketplace" paradigm in favor of a private, peer-to-peer "Dark Forest" model.

### 1.1. Axioms of V0
1. **Privacy is the Product:** Compute is the supply; PVA is the incentive. Workloads, outputs, and model data are strictly off-chain and ephemeral by default.
2. **Chain as a Privacy Accountant:** The L1 ledger (`FullPrivacy Chain`) sees only cryptographic commitments, escrow locks, and receipts. It serves as a secure clock (via block height) and a trustless settlement engine.
3. **No Public Discovery:** There are no public provider profiles, no reputation leaderboards, and no public service graphs. Discovery occurs through encrypted, async routing.
4. **Receipt-Based Settlement:** Payment is determined by deterministic, window-based availability and performance metering—not subjective "quality of AI answers."

---

## 2. Node Roles and Identity

The system enforces strict separation of concerns at the protocol level. While one physical machine may run multiple roles, they must not be merged cryptographically.

### 2.1. System Roles
*   **Validator:** Secures the FullPrivacy Chain, writes blocks, processes transactions, and earns block rewards.
*   **Compute Miner:** Provisions runtime slices (VM/container/GPU), runs the metering Agent, and earns lease PVA.
*   **Mailbox:** Stores encrypted envelopes (NXMS) for async delivery and earns storage PVA.
*   **Relay & Exit Node:** Routes encrypted traffic. Exit nodes provide optional internet egress via Tor (higher risk/reward).

### 2.2. Identity Model (Phase 0-5)
Complex identity hierarchies (e.g., Hidden Root Credentials) are deferred to Phase 6+. 
V0 utilizes a strictly bipartite identity model:
*   **Validator Role Key:** An independent Falcon PK (frozen as `node_pk_hash`).
*   **Compute Miner Role Key:** A separately generated Falcon PK used to sign receipts, offerings, and agent telemetry.
*   **Rule:** There is zero linkage between a Validator key and a Compute Miner key at the protocol level.

---

## 3. Discovery and Transport Layer

The network guarantees metadata minimization and secure routing for lease negotiation and runtime data.

### 3.1. Private Discovery (NXMS Mailbox)
Discovery is resource-based and credential-gated.
1.  **Query:** A User packages a `DiscoveryQuery` (e.g., "Need A100, >80GB VRAM, max 48 PVA") into an `NxmsEnvelope` encrypted via **FrodoKEM**.
2.  **Storage:** The envelope is routed to an NXMS Mailbox, which stores it blindly.
3.  **Response:** Miners poll the mailbox, attempt to decrypt the query, and if capable, reply with a `ComputeOffering` (also an encrypted `NxmsEnvelope`).

### 3.2. Session Transport (P2P / Tor)
Once a lease is negotiated, the heavy data plane begins:
*   **Handshake:** A single FrodoKEM handshake establishes a shared secret.
*   **Streaming:** The session upgrades to **XChaCha20Poly1305** for fast, symmetric P2P streaming over Tor/Relays.
*   **Modes:** Default is `isolated` (no internet). Others include `nxms_only`, `tor_gated`, and `internet_exit` (explicit opt-in required).

---

## 4. The Compute Lease Escrow (L1)

Before a compute session begins, the User must lock funds in a smart escrow on the FullPrivacy Chain.

### 4.1. ComputeLeaseEscrow SpendPolicy (Tag 0x04)
A new, purpose-built escrow policy (`COMPUTE_LEASE_ESCROW_TAG = 0x04`) replaces the generic 2-of-3 escrow.
*   **Locking:** The User locks a specific amount of PVA, committing to the hash of the off-chain `ComputeLeasePolicy`.
*   **Operatorless Target:** V0 aims for operatorless settlement. In Phase 1 (bootstrap), an Operator blindly co-signs the mathematically determined `Release` and `Refund` actions. Phase 2 introduces native `ProRataSplit`.

---

## 5. Window-Based Metering & The Miner Agent

Measuring exact GPU execution time (FLOPS) on a shared/virtualized GPU is technically unfeasible. V0 solves this using a **Challenge-Sampled Proof of Resource Possession**.

### 5.1. The Block Clock
The L1 Chain serves as the system clock. A "Window" is defined as a fixed number of blocks (e.g., 60 blocks ≈ 30 minutes).

### 5.2. Session Flow
1.  **Pre-flight (Startup Manifest):** The Miner provisions the environment and generates an `EnvironmentFingerprint` (hashing the OS, GPU driver, active processes, and binary). This is signed into a `StartupManifest`.
2.  **Challenge (User -> Miner):** At the start of every window, the User generates a challenge derived from the unpredictable `block_hash` of the chain's current height.
3.  **Telemetry (Miner -> User):** The Miner's Agent Daemon (using tools like `nvidia-smi`, `fio`) measures availability and performance.
4.  **Hash-Chain:** The Agent returns a signed `WindowTelemetryRecord` indicating PASS/FAIL. Crucially, each record hashes the previous record, creating an unbreakable, tamper-evident continuity chain.

---

## 6. Settlement & Dispute Resolution

Settlement is purely mathematical, based on the aggregate receipt produced at the end of the session.

### 6.1. Aggregate Receipt
The Miner compiles the session data into a single `ComputeLeaseReceipt` containing:
*   `total_windows`
*   `passed_windows`
*   `degraded_windows`
*   `window_hashes_root` (Merkle root of the hash-chained telemetry records)

### 6.2. Deterministic Settlement Formula
The chain (or the automated Phase 1 Operator) executes the following integer-only logic:
```
effective_windows = passed_windows + (degraded_windows * degraded_weight_permille / 1000)
miner_share = total_locked_amount * effective_windows / total_windows
user_share = total_locked_amount - miner_share
```
*Rule:* The remainder always defaults to the User.

### 6.3. Receipt Truth Architecture (ZK Disputes)
If the User disputes the Miner's `Aggregate Receipt`:
1.  The Miner is challenged on-chain.
2.  The Miner must submit a **Zero-Knowledge (ZK) Proof**.
3.  The ZK Proof mathematically bounds the private telemetry. It proves that the off-chain hash-chain of `WindowTelemetryRecords` perfectly reconstructs the `window_hashes_root` and correctly sums to the claimed `passed_windows`.
4.  The Chain verifies the ZK Proof. The loser pays the dispute fee.

---

*End of Specification.*