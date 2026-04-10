# PRIVAI HALO2 PROOF BOUNDARY FREEZE

> **Date:** 2026-04-10
> **Status:** Canonical freeze note — grounded in real code
> **Scope:** `privai-proof` crate only

---

## 1. Purpose

This document freezes the current Halo2 proof boundary as it actually exists in code.
It is not a redesign. It is a precise, honest statement of:

- what the current Halo2 layer **does** prove (code-confirmed)
- what the current Halo2 layer **does not** yet prove (explicitly not yet proven)
- how three different verification layers relate to each other
- what this means for the broader product and protocol

This note exists so that product docs, protocol specs, and handoff materials do not overclaim
what the current proof infrastructure delivers.

---

## 2. Canonicality / Status

| Field             | Value                                                      |
|-------------------|------------------------------------------------------------|
| Authoritative     | `privai-proof/src/halo2/circuits/tx_skeleton.rs` + chips   |
| Frozen from       | Real code snapshot, not aspirational design                |
| Supersedes        | Informal Halo2 claims in product docs                      |
| Do not infer      | Future scope from current TODO comments                    |

This document is **code-confirmed** where stated. Everything marked **not yet proven** is
backed by explicit TODO comments or absence of constraint gates in the source.

---

## 3. Current Halo2 Proof Scope

### 3.1 Composed circuit: `PrivaiTxSkeletonCircuit`

**code-confirmed** — `tx_skeleton.rs:27-29`

The current composed transaction skeleton circuit is **`1-in, 1-out`**.

```
Current scope is intentionally `1-in, 1-out`. The target v0 transaction
circuit will duplicate consumed and created sections into `2-in, 2-out`
once wiring and well-formedness constraints are stabilized.
```

The target is `2-in, 2-out`. That is **not yet implemented**.

### 3.2 Composed chips

The skeleton composes four real Halo2 chips:

| Chip               | Config              | What it proves (code-confirmed)                                    |
|--------------------|---------------------|--------------------------------------------------------------------|
| `LweAmountChip`    | `LweAmountConfig`   | Canonical `u32 → Fp` packing, `u32` range checks (16-bit lookup), `ct_amt_commit = Poseidon(pack(u,v))`, `t_commit = Poseidon(pack(t))`, dot-product scaffold, mod-q reduction, noise-adjusted mod-q relation |
| `NoiseClassChip`   | `NoiseClassConfig`  | Noise class lookup table, centered signed `e1`/`e2` decomposition into noise-class-bounded witnesses |
| `NoteCommitChip`   | `NoteCommitConfig`  | `note_commit = Poseidon(spend_policy_commit, ct_amt_commit, aux_commit, recipient_box_commit, blinding)` constrained against public instance |
| `NullifierChip`    | `NullifierConfig`   | `nullifier = Poseidon(note_commit, nullifier_key)` constrained against public instance |

### 3.3 Cell-to-cell wiring that is real

**code-confirmed** — `tx_skeleton.rs:115-122`

There is real cell-to-cell wiring from `LweAmountChip` into `NoteCommitChip`:

- `amount_outputs.ct_amt_commit` (output of LweAmountChip Poseidon) is passed as a copied cell into `NoteCommitChip::assign_with_cells(...)` as the `ct_amt_commit` input.
- `noise_outputs.e1_cells` and `noise_outputs.e2_cell` are wired from `NoiseClassChip` into `LweAmountChip::assign_with_noise_cells(...)`.

This is real inter-chip wiring. It is not a placeholder.

### 3.4 Public instance columns

**code-confirmed** — `tx_skeleton.rs:209-220` (test defines the instance layout)

```
[0] LweAmountChip::ct_amt_commit       — Poseidon hash of packed (u, v)
[1] LweAmountChip::t_commit             — Poseidon hash of packed t
[2] NoteCommitChip::note_commit         — Poseidon hash of note components
[3] NullifierChip::{consumed_note_commit, nullifier}  — consumed note commit + derived nullifier
```

### 3.5 What nullifier relation is actually proven

**code-confirmed** — `nullifier.rs:76-79`, `tx_skeleton.rs:127-143`

The nullifier is computed as:
```
nullifier = Poseidon(consumed_note_commit, consumed_nullifier_key)
```

The consumed note commitment enters the circuit as an **assigned advice cell** loaded via
a simple `assign_advice` call (`tx_skeleton.rs:127-137`). There is **no opening constraint**
on this consumed note commitment — it is not proven to be the Poseidon hash of any
particular note contents.

### 3.6 What note-commit relation is actually proven

**code-confirmed** — `note_commit.rs:84-99`, `tx_skeleton.rs:115-122`

The output note commitment is:
```
note_commit = Poseidon(spend_policy_commit, ct_amt_commit, aux_commit, recipient_box_commit, blinding)
```

This is constrained to equal a public instance value. The `ct_amt_commit` input is
wired from the LweAmountChip output (real cross-chip wiring).

### 3.7 Consumed-note entry — CHECKPOINT 2 confirmation

**code-confirmed** — `tx_skeleton.rs:124-126`

```
// TODO(privai-v0): consumed notes currently enter the skeleton as
// public note commitments for nullifier derivation only. Add consumed
// note opening constraints once the full spend relation is wired.
```

Consumed notes currently enter as public note commitments **for nullifier derivation only**.
No consumed-note opening is proven in-circuit.

---

## 4. Current Chips and Composed Circuit Boundary

### 4.1 LweAmountChip — what is constrained, what is scaffold

**Constrained (code-confirmed):**
- `u32` limb decomposition into 16-bit halves with u16 lookup table
- `value < LWE_MODULUS_Q` range check (slack decomposition)
- Canonical packing: seven `u32` limbs → one `Fp` element
- `ct_amt_commit = Poseidon(pack(u, v))` constrained to instance column
- `t_commit = Poseidon(pack(t))` constrained to instance column
- Dot-product multiply steps (fixed-coefficient and advice-coefficient)
- Dot-product accumulator steps
- Mod-q reduction: `full = reduced + quotient * q`
- Quotient bit decomposition (boolean bits, accumulator binding)
- Noise-adjusted mod-q relation: `output = reduced + noise - q * (wrap_pos - wrap_neg)` with boolean wrap flags

**Not yet constrained (code-confirmed):**
- `u = A^T r + e1` — the full LWE well-formedness relation for the vector `u` is not closed (`lwe_amount.rs:41-42`)
- `v = t^T r + e2 + Δ·amount` — the scalar `v` well-formedness gate is not wired (`lwe_amount.rs:647-651`)
- The `e2` noise cell is copied into the chip but its relation to `v` is not constrained

### 4.2 NoiseClassChip

**Constrained (code-confirmed):**
- Lookup table for noise class bounds
- Centered signed `e1[i]` and `e2` decomposition
- Noise class assignment with bounded witnesses

### 4.3 NoteCommitChip

**Constrained (code-confirmed):**
- Poseidon hash of 5 inputs against public instance

**Not yet constrained:**
- No consumed-note opening (the consumed note is a separate, unconstrained cell)

### 4.4 NullifierChip

**Constrained (code-confirmed):**
- Poseidon hash of `(note_commit, nullifier_key)` against public instance
- `note_commit` exposed as additional public instance for binding

---

## 5. Witness / Statement Consistency Layer

**This is NOT Halo2 proving. This is a separate, code-confirmed consistency layer.**

`TransferProvingData::from_tx_and_witness(...)` (`transfer.rs:186-227`) performs:

1. Reconstructs `TransferStatement` from the transaction
2. Verifies `tx.core.statement_commit == statement.commitment()` — rejects `StatementCommitMismatch`
3. Verifies `tx.core.outputs.len() == witness.outputs.len()` — rejects `OutputWitnessCountMismatch`
4. For each output: verifies `note.note_commit == output_witness.note_commit` — rejects `OutputNoteCommitMismatch`
5. Verifies `note.payload_commit() == output_witness.recipient_opening.note_payload_commit` — rejects `OutputPayloadCommitMismatch`
6. Verifies `note.recipient_box.hint == output_witness.recipient_opening.bundle_id` — rejects `OutputBundleHintMismatch`

**Confirmed by tests:** `tests/transfer_note_proof_semantics.rs` — 10 tests covering each rejection path.

This layer ensures the witness is structurally consistent with the transaction. It does NOT:
- Generate a Halo2 proof
- Verify any cryptographic circuit constraint
- Prove that the witness satisfies the spend relation

---

## 6. Structural Proof Verifier Layer

**This is NOT cryptographic proof verification. This is block-level structural validation.**

`StructuralProofVerifier` (`lib.rs:94-196`) validates:

1. Housekeeping blocks carry no transactions and no proof coverage
2. User blocks have non-empty statement commits, covered tx indexes, and proof certificates
3. Statement commits match the corresponding transaction's `statement_commit()`
4. All tx indexes are covered (no gaps, no duplicates)
5. Proof certificate `statement_root` matches the merkle root of statement commits
6. Proof certificate `public_inputs_root` matches the execution bundle's root
7. Proof certificates have non-zero `proof_system_id` and non-empty `prover_ids`

**Confirmed by tests:** `tests/structural_proof_verifier.rs` — 8 tests covering acceptance and each rejection path.

This layer does NOT:
- Verify any Halo2 proof
- Check cryptographic soundness of any circuit
- Verify that proof_bytes contain a valid ZK proof
- Confirm spend validity, balance conservation, or nullifier correctness

`SidecarProofVerifier` (`verify.rs:94-128`) chains structural verification with artifact validation
and a pluggable `BatchProofVerifierBackend`. The default `ProofEnvelopeVerifier` only checks that
`proof_bytes` is non-empty and coverage is non-empty. It does NOT verify the proof content.

---

## 7. What Is Code-Confirmed Now

Label: **code-confirmed**

The following are backed by real constraints in Halo2 circuits, verified by MockProver:

| # | What is proven                                              | Where                                      |
|---|-------------------------------------------------------------|---------------------------------------------|
| 1 | Output `(u, v)` are canonical `u32` values packed into `Fp` | `lwe_amount.rs` — `q_pack`, `q_limb` gates |
| 2 | Output `u[i] < q` and `v < q`                               | `lwe_amount.rs` — `q_lt_modulus` gate       |
| 3 | Output `t[i] < q`                                           | `lwe_amount.rs` — `q_lt_modulus` gate       |
| 4 | Output `r[i]` are canonical `u32` values                    | `lwe_amount.rs` — `q_limb` gate             |
| 5 | `ct_amt_commit = Poseidon(pack(u, v))`                      | `lwe_amount.rs` — Poseidon + instance       |
| 6 | `t_commit = Poseidon(pack(t))`                              | `lwe_amount.rs` — Poseidon + instance       |
| 7 | Noise `e1[i]`, `e2` are class-bounded centered values       | `noise_class` chip + lookup                 |
| 8 | Noise cells are wired into LweAmountChip                    | `tx_skeleton.rs:105-113`                    |
| 9 | `ct_amt_commit` is wired from LweAmountChip to NoteCommitChip | `tx_skeleton.rs:115-122`                 |
| 10 | `note_commit = Poseidon(spend_policy, ct_amt, aux, rbox, blinding)` | `note_commit.rs` — Poseidon + instance |
| 11 | `nullifier = Poseidon(consumed_note_commit, nullifier_key)` | `nullifier.rs` — Poseidon + instance        |

Additionally, the following are **code-confirmed** outside Halo2 (witness/statement consistency):

| # | What is verified                                             | Where                                              |
|---|--------------------------------------------------------------|-----------------------------------------------------|
| 12 | Statement commit matches tx                                  | `transfer.rs:192-196`                               |
| 13 | Output witness count matches tx outputs                      | `transfer.rs:199-204`                               |
| 14 | Output note commitments match between tx and witness         | `transfer.rs:206-213`                               |
| 15 | Output payload commits match                                 | `transfer.rs:214-216`                               |
| 16 | Output bundle hints match                                    | `transfer.rs:217-219`                               |

And **code-confirmed** structural block verification:

| # | What is verified                                             | Where                                              |
|---|--------------------------------------------------------------|-----------------------------------------------------|
| 17 | Housekeeping vs user block mode enforcement                  | `lib.rs:98-108`, `lib.rs:186-194`                   |
| 18 | Statement commit ↔ tx binding                                | `lib.rs:125-145`                                    |
| 19 | Complete tx coverage (no gaps, no duplicates)                | `lib.rs:124-152`                                    |
| 20 | Certificate roots match execution bundle                     | `lib.rs:162-180`                                    |

---

## 8. What Is Explicitly NOT Yet Proven

Label: **not yet proven**

| # | What is NOT proven                                           | Evidence                                              |
|---|--------------------------------------------------------------|-------------------------------------------------------|
| 1 | Full `2-in, 2-out` transaction circuit                       | `tx_skeleton.rs:28`: target is future; current is `1-in, 1-out` |
| 2 | Consumed-note opening constraints                            | `tx_skeleton.rs:124-126`: TODO comment explicitly says "Add consumed note opening constraints once the full spend relation is wired" |
| 3 | Plaintext conservation (amount balance)                      | `tx_skeleton.rs:145-146`: TODO comment explicitly says "add plaintext conservation once amount witness cells are shared" |
| 4 | Full spend relation in-circuit                               | `tx_skeleton.rs:126`: "once the full spend relation is wired" |
| 5 | LWE well-formedness `u = A^T r + e1`                         | `lwe_amount.rs:41-42`: "Full well-formedness... will be layered on top of this in the next iteration" |
| 6 | Scalar well-formedness `v = t^T r + e2 + Δ·amount`           | `lwe_amount.rs:647-651`: SECURITY_TODO: "the relation is not constrained yet" |
| 7 | `e2` noise relation to `v`                                   | `lwe_amount.rs:647-651`: "e2 is wired into the chip, but the relation is not constrained yet" |
| 8 | That proof_bytes contain a valid ZK proof                    | `verify.rs:33-47`: `ProofEnvelopeVerifier` only checks non-empty |
| 9 | Full cryptographic semantic soundness of any block           | `StructuralProofVerifier` is structural only, not cryptographic |
| 10 | Spend authorization in-circuit                               | Auth/policy is ledger-only; proof layer ignores it (`transfer.rs:434-457`) |
| 11 | Nullifier uniqueness / double-spend prevention               | Not proven in any circuit                              |
| 12 | Note commitment binding to consumed note contents            | Consumed note enters as raw field element, not opened   |

---

## 9. Product / Protocol Implications

Label: **current production-adjacent proof layer**

### 9.1 What the current proof infrastructure gives us

- A **real, composable Halo2 chip architecture** with inter-chip wiring (not a placeholder)
- **Poseidon-based commitments** for note commits, nullifiers, and amount ciphertexts — real cryptographic hashes inside the circuit
- **Range-checked LWE ciphertext components** with canonical packing
- **Noise-bounded witness decomposition** through lookup tables
- A **witness/statement consistency layer** that catches tx/witness mismatches before proving
- A **structural block verifier** that enforces proof coverage, certificate binding, and execution bundle integrity
- **Execution bundle construction** that correctly maps transactions to statement commits and public inputs roots

### 9.2 What it does NOT yet justify claiming

- **Do NOT claim** that the current Halo2 layer proves full transfer privacy semantics
- **Do NOT claim** that blocks are cryptographically verified end-to-end — structural verification is not cryptographic verification
- **Do NOT claim** that the proof system prevents double-spends — nullifier uniqueness is not proven in-circuit
- **Do NOT claim** that amount balance is enforced by the circuit — plaintext conservation is explicitly TODO
- **Do NOT claim** that the current circuit is the final `2-in, 2-out` transaction proof system

### 9.3 Interface with Stage A / Stage B boundary

- **Stage A** (current): The proof layer provides structural enforcement (proof coverage, certificate binding) and a growing set of Halo2 chip primitives. The proof system ID mechanism allows future strengthening without breaking the block format.
- **Stage B** (future): Will require the `2-in, 2-out` circuit, consumed-note opening, plaintext conservation, and full spend relation to be wired. The current code is a scaffold toward that, not a delivery of it.

### 9.4 Interface with proof certificates

- `ProofCertificate` (`lib.rs`) carries `proof_system_id`, `statement_root`, `public_inputs_root`, `proof_bytes_hash`, `prover_ids`, and `proof_meta_hash`.
- The structural verifier checks root alignment but does NOT verify proof content.
- `proof_system_id` enables forward-compatible proof system upgrades.
- Current certificates are structurally valid envelopes, not cryptographically verified proofs.

### 9.5 Interface with execution bundle coverage

- `ExecutionBundle` correctly maps covered tx indexes to statement commits and derives public inputs roots.
- The `build_execution_bundle_from_transactions` function correctly filters proof-requiring transactions (TransferNote, LiteTransfer) from non-proof-requiring ones (MarketplaceBatch, Settlement, Model, Stake).
- Execution bundle coverage is structural: it ensures every tx is accounted for, not that the proof content is sound.

---

## 10. What Remains Future Strengthening

Label: **future strengthening**

The following items are in the codebase as TODOs or partial scaffolds. They represent
the planned next layer, not the current boundary:

1. **`2-in, 2-out` circuit duplication** — duplicate consumed and created sections once wiring stabilizes
2. **Consumed-note opening constraints** — prove that the consumed note commitment is the Poseidon hash of valid note contents
3. **Plaintext conservation** — enforce `input_amount = output_amount + fee` in-circuit
4. **Full spend relation wiring** — connect consumed and created amount witnesses through shared cells
5. **LWE well-formedness `u = A^T r + e1`** — close the encryption correctness proof
6. **Scalar well-formedness `v = t^T r + e2 + Δ·amount`** — close the amount encoding proof
7. **`e2` noise binding to `v`** — constrain the scalar noise to participate in the `v` relation
8. **Real proof backend** — replace `ProofEnvelopeVerifier` with actual Halo2 verification
9. **Nullifier uniqueness enforcement** — ensure nullifiers are unique across the ledger

---

## 11. Non-Goals / Do Not Infer

Label: **do not infer**

- **Do NOT infer** from the existence of Halo2 chips that full transfer privacy is cryptographically proven
- **Do NOT infer** from TODO comments that those features are implemented — TODO means not yet done
- **Do NOT infer** from MockProver tests that the proof system is production-ready — MockProver is a development tool, not a production verifier
- **Do NOT infer** from the `1-in, 1-out` circuit that multi-input/multi-output transfers are proven
- **Do NOT infer** from the structural verifier that block contents are cryptographically valid
- **Do NOT infer** from execution bundle coverage that individual transactions are proven sound
- **Do NOT infer** from the witness consistency layer that a Halo2 proof has been generated or verified
- **Do NOT infer** that auth/policy semantics are covered by the proof layer — they are ledger-only

---

## Summary

Three distinct layers exist and must not be conflated:

```
┌──────────────────────────────────────────────────────────────────┐
│  Layer 3: Halo2 Circuit Proving (current: 1-in, 1-out scaffold) │
│  - Real Poseidon commitments                                     │
│  - Real range checks and packing                                 │
│  - Real inter-chip wiring (LweAmount → NoteCommit)               │
│  - NOT yet: consumed-note opening, conservation, full spend      │
├──────────────────────────────────────────────────────────────────┤
│  Layer 2: Structural Proof / Execution Bundle Verification       │
│  - Block structure validation                                    │
│  - Certificate root alignment                                    │
│  - Coverage completeness                                         │
│  - NOT cryptographic proof verification                          │
├──────────────────────────────────────────────────────────────────┤
│  Layer 1: Witness / Statement Consistency                        │
│  - TransferProvingData::from_tx_and_witness(...)                 │
│  - Statement commit, note commit, payload commit checks          │
│  - Pure consistency, no circuit, no proving                      │
└──────────────────────────────────────────────────────────────────┘
```

The current Halo2 layer is a **real, growing scaffold** — not a placeholder, but also
not the final proof system. Product and protocol docs must reflect this boundary honestly.
