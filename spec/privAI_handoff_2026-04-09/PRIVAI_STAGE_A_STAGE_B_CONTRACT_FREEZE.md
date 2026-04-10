# privAI Stage A / Stage B Contract Freeze

Date: 2026-04-10
Status: `code-confirmed` freeze document
Canonicality: derived from running code and passing tests. Does not override protocol, format, proof, or ledger specs. Freezes the current A/B boundary as implemented.

---

## 1. Purpose

Freeze the current Stage A / Stage B boundary contract for the escrow v1 workflow. The boundary is derived from real code in `privai-node` and `privai-wallet`, confirmed by:

- `privai-node/tests/escrow_e2e_release.rs`
- `privai-node/tests/escrow_e2e_refund.rs`
- `privai-node/tests/escrow_e2e_recovery_release.rs`
- `privai-node/tests/escrow_persistence.rs`
- `privai-node/tests/escrow_submit_gate_negative.rs`

This document does **not** redesign the system. It freezes what is already true.

---

## 2. Canonicality / Source-of-Truth Status

This document is `code-confirmed`. Every claim below maps to a specific function, struct, constant, or test assertion in the current codebase.

It does **not** replace:
- `spec/PRIVAI_ESCROW_FINAL_MODEL.md`
- `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
- `spec/PRIVAI_ESCROW_FULLPRIVACY_BOUNDARY_DECISION_MEMO.md`

It is the reference for "what actually crosses A -> B right now."

---

## 3. What Stage A Owns

Stage A is the **control-plane staging layer** inside `privai-node`. It owns everything up to (but not including) final transaction assembly.

| Responsibility | Code location | `code-confirmed` |
|---|---|---|
| Funded escrow ingest & tracking | `escrow_stage.rs:143` (`ingest_funded`) | yes |
| Proposal staging | `escrow_stage.rs:169` (`ingest_proposal`) | yes |
| Approval ingestion | `escrow_stage.rs:206` (`ingest_approval`) | yes |
| Session / escrow consistency checks | `escrow_stage.rs:176-184` | yes |
| Duplicate approval rejection | `escrow_stage.rs:219` | yes |
| Quorum readiness (>= 2 approvals) | `escrow_stage.rs:227` (`is_quorum_ready`) | yes |
| Approval bundle construction | `node.rs:525` (`build_escrow_approval_bundle`) | yes |
| Persistence of staged state across restarts | `escrow_stage.rs:304-355` (`load_from_path` / `save_to_path`) | yes |
| Atomic snapshot write (temp + rename) | `escrow_stage.rs:341-346` | yes |
| Idempotent funded/proposal re-ingest | `escrow_stage.rs:146-154`, `186-193` | yes |

Stage A does **not**:
- construct final `TransferNote`
- compute `tx_signing_hash`
- produce real Falcon signatures over transaction content
- interact with the ledger spend/import path
- hold funding note spend material

---

## 4. What Stage A Outputs

Stage A produces one primary output artifact: the `EscrowApprovalBundle`.

Defined in `privai-nxms/src/lib.rs:229`:

```
EscrowApprovalBundle {
    proposal_hash: Hash32,
    tx_signing_hash: Hash32,       // <-- always TX_SIGNING_HASH_STAGE_A
    signer_pks: Vec<Hash32>,       // hashed Falcon PKs, sorted ascending
    signatures: Vec<Vec<u8>>,      // control-plane approval sigs (NOT ledger sigs)
}
```

Key properties:
- Built via `from_approvals_sorted()` at `privai-nxms/src/lib.rs:237`
- `tx_signing_hash` is always set to `TX_SIGNING_HASH_STAGE_A` (all-zero sentinel)
- `signer_pks` are `Hash32` (hashed Falcon PKs), not raw PKs
- `signatures` are control-plane authorization material, not final ledger signatures
- `validate()` checks length parity and no duplicate signers

**This bundle is authorization material for Stage B, not a final ledger-ready auth package.**

---

## 5. What Stage B Owns

Stage B is the **final assembly + submit + ledger path**. It spans `privai-wallet` (assembly) and `privai-node` (submit gate + import).

| Responsibility | Code location | `code-confirmed` |
|---|---|---|
| Funding note / spend context re-entry | `escrow_builder.rs:225` (`build_escrow_transfer_note_from_assembly_inputs`) | yes |
| `FinalAssemblyInputs` construction | `escrow_builder.rs:28` | yes |
| `AuthMaterial` (including `policy_opening`) | `escrow_builder.rs:19` | yes |
| `policy_opening` present in auth | `escrow_builder.rs:140`, `node.rs:648-651` | yes |
| Policy reconstruction from `policy_opening` | `escrow_builder.rs:140-156`, `node.rs:653-677` | yes |
| Canonical final `tx_signing_hash` computation | `escrow_builder.rs:286` | yes |
| Real Falcon signatures over `tx_signing_hash` | e2e tests: `escrow_e2e_release.rs:251-256` | yes |
| Final `TransferNoteTx` construction | `escrow_builder.rs:273-278` | yes |
| Submit gate validation | `node.rs:593` (`validate_escrow_transfer_note`) | yes |
| Signature verification against `tx_signing_hash` | `node.rs:763` (`verify_escrow_tx_signatures`) | yes |
| Ledger submission / mempool entry | `node.rs:816` (`submit_transaction`) | yes |
| Block proposal, import, finalization | `node.rs:358`, `node.rs:414` | yes |

---

## 6. Boundary Contract: A -> B

### 6.1 Data that crosses the A -> B boundary

| Artifact | From | To | Nature |
|---|---|---|---|
| `EscrowApprovalBundle` | Stage A (node) | Stage B (wallet/node) | `code-confirmed` |
| `proposal_hash` | Stage A | Stage B | `code-confirmed` |
| `bundle.signer_pks` (hashed PKs) | Stage A | Stage B | `code-confirmed` |
| `bundle.signatures` (control-plane) | Stage A | Stage B | `code-confirmed` |

### 6.2 Data that does NOT cross from A -> B (Stage B constructs independently)

| Artifact | Why | `code-confirmed` |
|---|---|---|
| `tx_signing_hash` (final) | Computed by wallet from assembled `TransferNoteTx` | yes |
| `policy_opening` (canonical bytes) | Reconstructed by wallet from `SpendPolicy::Escrow2of3` | yes |
| `InputAuth` (final auth struct) | Built by `build_final_input_auth()` in wallet | yes |
| Real Falcon signatures | Signed over final `tx_signing_hash` by participants | yes |
| Output note, nullifier, proof scaffolding | Derived by wallet during assembly | yes |

### 6.3 What gets revalidated in the submit gate

The submit gate (`validate_escrow_transfer_note` at `node.rs:593`) revalidates **from scratch** against the Stage A store:

1. Proposal exists in staging store
2. Quorum is met (>= 2 approvals)
3. Stage A bundle can be built
4. Transaction is `Transaction::TransferNote`
5. Exactly one `Escrow2of3` auth entry exists
6. `policy_opening` is present and decodes to `SpendPolicy::Escrow2of3`
7. Decoded policy fields (buyer/merchant/operator pk_hash, timeout_block) match the funding descriptor
8. `escrow_action` in auth matches proposal action (with offset: proposal 0/1/2 -> tx 1/2/3)
9. Signer PK set in tx auth matches Stage A bundle signer set (after hashing tx raw PKs)
10. All Falcon signatures verify against `tx_signing_hash`

After submit gate passes, `verify_escrow_tx_signatures` (`node.rs:763`) verifies every auth entry's signatures against the canonical `tx.tx_signing_hash()`.

---

## 7. Sentinel vs Final Values

### `TX_SIGNING_HASH_STAGE_A` — sentinel, not final

`code-confirmed`: `TX_SIGNING_HASH_STAGE_A` is defined as `[0u8; 32]` at `privai-nxms/src/lib.rs:227`.

Every `EscrowApprovalBundle` produced by Stage A has `tx_signing_hash == TX_SIGNING_HASH_STAGE_A`. This is enforced by `from_approvals_sorted()` at line 256.

The `is_stage_a()` method at line 289 explicitly checks for this sentinel.

**`TX_SIGNING_HASH_STAGE_A` is a sentinel indicating "not yet computed." It is NOT a final ledger signing message. It carries no cryptographic binding to the transaction.**

### Final `tx_signing_hash` — Stage B only

`code-confirmed`: The final `tx_signing_hash` is computed by `Transaction::TransferNote(tx).tx_signing_hash()` at `escrow_builder.rs:286` after the wallet assembles the complete `TransferNoteTx` including auth, outputs, nullifiers, and all core fields.

Real Falcon signatures are produced over this final hash. This is the canonical non-circular signing message that excludes auth signatures (avoiding circular dependency).

The e2e tests confirm this: real signatures are produced over `assembled.tx_signing_hash` (e.g. `escrow_e2e_release.rs:251-256`), then attached to the tx auth.

---

## 8. Revalidation in Submit Gate / Ledger Path

### `policy_opening` is required

`code-confirmed`: The submit gate requires `policy_opening` to be present in the escrow auth entry. See `node.rs:648-651`:

```rust
let policy_opening_bytes = auth
    .policy_opening
    .as_ref()
    .ok_or_else(|| NodeError::EscrowSubmit("escrow auth missing policy_opening".into()))?;
```

Without `policy_opening`, the submit gate rejects the transaction. This is tested by `escrow_submit_gate_negative.rs`.

### Policy reconstruction and validation

`code-confirmed`: `policy_opening` is decoded via `SpendPolicy::from_canonical_bytes()` at `node.rs:653-656`. The decoded policy must be `SpendPolicy::Escrow2of3`, and its fields (buyer_pk_hash, merchant_pk_hash, operator_pk_hash, timeout_block) are compared against the funding descriptor stored in Stage A. Any mismatch is a submit gate rejection.

### Signer set revalidation

`code-confirmed`: The submit gate hashes the tx auth's raw Falcon PKs using `domain_hash("privai:falcon-pk:v0", &[pk])` at `node.rs:740-741`, sorts them, and compares against the Stage A bundle's `signer_pks`. This catches any attempt to swap signers between Stage A and Stage B.

### Signature verification

`code-confirmed`: After submit gate validation, `verify_escrow_tx_signatures` at `node.rs:763-793` verifies every `(pk, sig)` pair in every auth entry against `tx.tx_signing_hash()`. This is the final ledger-path verification before mempool admission.

---

## 9. What Is Code-Confirmed Now

All of the following are `code-confirmed` — implemented, tested, and passing:

| Invariant | Evidence |
|---|---|
| Stage A produces `EscrowApprovalBundle` with sentinel hash | `privai-nxms/src/lib.rs:256`, `node.rs:549` |
| Stage A persists funded/proposal/approvals across restart | `escrow_persistence.rs` (15 tests) |
| Quorum threshold is >= 2 approvals | `escrow_stage.rs:229` |
| Stage B constructs `FinalAssemblyInputs` with `AuthMaterial` | `escrow_builder.rs:28-36` |
| Stage B requires `policy_opening` in auth | `escrow_builder.rs:140`, `node.rs:648` |
| Stage B computes final `tx_signing_hash` from assembled tx | `escrow_builder.rs:286` |
| Real Falcon signatures are over final `tx_signing_hash` | `escrow_e2e_release.rs:251`, `escrow_e2e_refund.rs:264`, `escrow_e2e_recovery_release.rs:269` |
| Submit gate revalidates proposal, quorum, policy, signers, action, signatures | `node.rs:593-756` |
| Release e2e passes (Buyer + Operator sign) | `escrow_e2e_release.rs` |
| Refund e2e passes (Merchant + Operator sign) | `escrow_e2e_refund.rs` |
| RecoveryRelease e2e passes (Buyer + Merchant sign, no Operator) | `escrow_e2e_recovery_release.rs` |
| Submit gate rejects wrong action, missing policy_opening, signer mismatch | `escrow_submit_gate_negative.rs` |
| Persistence continuation: full Release flow after node reopen without Stage A replay | `escrow_persistence.rs:549` |
| Cross-restart approval accumulation works | `escrow_persistence.rs:858` |

---

## 10. Frozen Direction vs Future Strengthening

### Frozen direction (`code-confirmed`)

- Stage A is control-plane staging; Stage B is final assembly + ledger
- `EscrowApprovalBundle` is the A -> B handoff artifact
- `TX_SIGNING_HASH_STAGE_A` is a sentinel, not cryptographic material
- `policy_opening` is required in Stage B for policy reconstruction
- Real signatures are over the final `tx_signing_hash`, not the sentinel
- Submit gate revalidates everything from scratch against Stage A store
- Escrow 2-of-3 signer sets follow action semantics:
  - Release: Buyer + Operator
  - Refund: Merchant + Operator
  - RecoveryRelease: Buyer + Merchant
- Persistence uses JSON snapshot with atomic temp+rename

### Future strengthening (NOT yet implemented)

- Full `FullPrivacy` auth-required enforcement for all inputs (Option B from boundary decision memo) — `frozen direction`, not yet in code
- Proof coverage of escrow auth semantics (currently proof covers note-level; auth/policy is ledger-side) — `future strengthening`
- Formal threshold signature scheme replacing placeholder control-plane signatures in Stage A — `future strengthening`
- Cross-node Stage A synchronization beyond single-node persistence — `future strengthening`
- Operator mailbox integration for real-time approval routing — `future strengthening` (T-010 in progress)
- Recovery path timeout enforcement at ledger level (currently validated at submit gate, not in proof) — `future strengthening`

---

## 11. Non-Goals / Do Not Infer

This document does NOT claim:

- Stage A signs the final transaction. Stage A produces control-plane authorization material. Final signatures are Stage B.
- Proof covers the whole escrow workflow. Proof covers note-level semantics. Auth/policy/threshold/timeout enforcement is ledger-side (`code-confirmed`).
- The sentinel hash has any cryptographic meaning. `TX_SIGNING_HASH_STAGE_A == [0u8; 32]` is a marker only.
- Stage A persistence implies cross-node replication. Persistence is single-node snapshot.
- The current Stage A approval signatures are final ledger-ready transaction signatures. They are proposal-level / control-plane signatures expressing signer intent over the proposal; real ledger-path signatures are produced in Stage B.
- Option B (auth for all FullPrivacy inputs) is implemented. It is a recommended direction from the boundary decision memo, not current code.

---

## Appendix: Key Constants and Types

| Name | Value / Type | Location |
|---|---|---|
| `TX_SIGNING_HASH_STAGE_A` | `[0u8; 32]` | `privai-nxms/src/lib.rs:227` |
| `EscrowApprovalBundle` | struct | `privai-nxms/src/lib.rs` (implicit, defined by fields in `from_approvals_sorted`) |
| `EscrowStageStore` | struct | `privai-node/src/escrow_stage.rs:131` |
| `FinalAssemblyInputs` | struct | `privai-wallet/src/escrow_builder.rs:28` |
| `AuthMaterial` | struct | `privai-wallet/src/escrow_builder.rs:19` |
| `EscrowAssembledTx` | struct | `privai-wallet/src/escrow_builder.rs:39` |
| Quorum threshold | >= 2 approvals | `privai-node/src/escrow_stage.rs:229` |
| Snapshot filename | `escrow_stage.json` | `privai-node/src/escrow_stage.rs:358` |
