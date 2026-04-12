# privAI V0 Operatorless Escrow Direction

**Status:** canonical V0 direction
**Date:** 2026-04-12
**Scope:** Defines the strategic move to native, protocol-enforced operatorless escrow settlement for compute leases.

---

## 1. Executive Summary

This document sets the final architectural direction for the `privAI` compute lease escrow. 

The core decision: **We are skipping the intermediary "Automated Operator" (Phase 1) and attacking the final "Protocol-Native Operatorless Escrow" (Phase 2) directly.**

The L1 Chain will natively understand `ComputeLeaseReceipt` and automatically execute a `ProRataSplit` without any third-party operator signature.

*Compliance Check:* Operatorless settlement is the target direction. It is **not yet fully implemented** at the consensus level (the `ProRataSplit` transaction semantics remain an active gap).

---

## 2. Current Code Reality (Phase 0: The Bridge)

Currently, `privai-chain/src/escrow.rs` implements `Escrow2of3` (tag `0x03`).
- It requires a canonical `Operator` to co-sign `Release` and `Refund` actions.
- Actions are all-or-nothing (no native pro-rata split).
- This mechanic remains untouched. It serves as a legacy bridge and fallback, but it is **not** the vehicle for V0 compute leases.

---

## 3. The Pivot: Skipping Phase 1

Originally, Phase 1 proposed keeping the operator keypair but automating it (a server blindly signing receipts). 

We reject Phase 1 because:
1.  **Centralization Risk:** A hot operator key processing all receipts is a massive single point of failure and censorship vector.
2.  **Unnecessary Complexity:** We already have the deterministic integer math (`calculate_settlement` in `compute_lease.rs`). The L1 validators are perfectly capable of running this math themselves during block execution.

Therefore, the target is **Phase 2: Protocol-Native Validation**.

---

## 4. Target Mechanics (Phase 2: Operatorless Protocol)

### 4.1. New Spend Policy
A new escrow policy tag is introduced: `COMPUTE_LEASE_ESCROW_TAG = 0x04`.
When funds are locked under this tag, they are immutably bound to a `lease_policy_commit` (hash of `ComputeLeasePolicy`). There is no `operator_pk_hash` field in this policy.

### 4.2. Consensus-Level Validation
When a session ends, either the User or the Miner submits a `ComputeSettlementTx` to the mempool.
The payload is the `ComputeLeaseReceipt`.

During block validation, the Validator node:
1. Verifies the `ComputeLeaseReceipt` signature (must match the Miner's `miner_role_key_hash`).
2. Verifies the receipt matches the locked `lease_policy_commit`.
3. Calls the deterministic `calculate_settlement(amount, receipt, policy)` function.

### 4.3. Native Pro-Rata Split
Because the settlement is verified by consensus, the transaction natively creates two outputs from the single escrow input:
- `Output 1 (Miner Share):` Paid to Miner.
- `Output 2 (User Share):` Refunded to User.

This requires extending the ledger state machine to support 1-to-2 Pro-Rata splitting.

---

## 5. Dispute and Recovery Mechanics

In an operatorless system, the protocol must handle edges cases natively:

1.  **Miner No-Show (Timeout):** If the `timeout_block_height` is reached and no valid `ComputeLeaseReceipt` has been submitted, the User can submit a `TimeoutClaimTx`. The protocol refunds 100% of the PVA to the User.
2.  **Disputed Receipt:** If the User claims the Miner forged the `Aggregate Receipt`, the Miner must submit a `DisputeEvidencePackage` containing the ZK Proof of the hash-chained `WindowTelemetryRecords`. Validators verify the ZK Proof natively. If valid, settlement proceeds. If invalid or missing, the Miner is slashed (dispute fee) and the User is refunded.

---

## 6. Required Implementation Path

To land this Phase 2 architecture, the following engineering tasks must follow (Tier 5):

1.  **Transaction Semantics:** Define `ComputeSettlementTx` and `TimeoutClaimTx` in `privai-chain/src/tx.rs`.
2.  **Ledger State Machine:** Implement the `ProRataSplit` execution logic in the ledger so it can safely spend a single Escrow UTXO into two destination UTXOs based on the `SettlementResult`.
3.  **Consensus Hook:** Wire `privai-node` block validation to parse the receipt and execute the ZK Proof verification if challenged.

*This document fulfills the Tier 2 requirement for Operatorless Escrow Direction.*
