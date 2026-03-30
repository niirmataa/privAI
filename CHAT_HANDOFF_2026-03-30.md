# privAI Handoff 2026-03-30

This file is the handoff for a new chat after a long implementation and audit session.

## Canonical environment

- Canonical runtime and build environment: `WSL Alpine`
- Canonical workspace: `/home/nxms-server/privAI`
- Windows path is only the front-end/editor view of the same work.
- Real builds, tests, fuzzing and audit evidence were done on Alpine.

## High-level result of this conversation

We did not only discuss architecture. A lot of code was implemented, tested and audited.

The major completed areas are:

1. `privai-*` crates were scaffolded into a coherent chain/proof/wallet stack.
2. `nxms-transport` was generalized beyond escrow-specific naming and got a real audit lane.
3. Falcon CT hot-path hardening was completed with a prepared signer path.
4. `nexum-cli` was brought onto the prepared signer path and now builds on Alpine.
5. We now have enough recorded evidence to continue the next chat directly from the NXMS audit.

## Completed work by area

### 1. `privai-nxms`

Completed:

- Canonical JSON encoding for `PRIVAI/1` message bodies.
- Stable body hashing using domain-separated `blake3`.
- Shared hash/primitives helpers for:
  - `body_hash`
  - `context_id`
  - `bundle_id`
  - `proof_job_id`
  - `statement_commit`
  - `falcon_pk_hash`

Key file:

- `privai-nxms/src/lib.rs`

State:

- Tests were green in Alpine.
- This crate is the canonical place for message-body formatting and hashes.

### 2. Chain / ledger / proof / wallet

Completed:

- `privai-chain`
- `privai-ledger`
- `privai-node`
- `privai-proof`
- `privai-wallet`

Important outcomes:

- Structural proof verification layer exists.
- Block-side `ExecutionBundle` and batch proof plumbing were added.
- Wallet supports hidden-note state and spend material derivation.
- Recipient box verification was hardened.
- Real wallet integration tests were added.
- End-to-end `wallet -> tx -> proof input -> block` plumbing was pushed forward.

Important note:

- `privai-proof` is still a structural/proof-envelope layer, not yet a final cryptographic ZK verifier.
- This is known and intentional.

### 2A. Custom LWE privacy model: hidden amounts, hidden addresses, noise-budget accounting

This part is important for the next chat because it is not just "privacy by transport".
The chain model itself already assumes hidden-value notes and hidden recipients.

The intended v0 model is:

- each output note contains a fresh LWE ciphertext `ct_amt`,
- recipient identity is hidden behind a one-time `ReceiveBundle`,
- ownership recovery data lives inside `RecipientBox`,
- spendability is controlled by `SpendPolicy`,
- replay / double-spend prevention uses `Nullifier`,
- future proof statements bind all of the above through commitments.

Core objects:

- `OutputNote`
  - `note_commit`
  - `spend_policy_commit`
  - `ct_amt`
  - `aux_commit`
  - `recipient_box`
- `RecipientBox`
  - on-chain encrypted payload for the receiver
- `RecipientBoxPlaintext`
  - receiver-side opening with bundle binding, witness seed, nullifier key and policy opening
- `AuxWitness`
  - private witness material bound by `aux_commit`

The privacy split is deliberate:

- `ct_amt` hides the amount,
- `ReceiveBundle` + `RecipientBox` hide the destination,
- `SpendPolicy` can encode single-owner or escrow semantics without exposing private receiver state,
- `note_commit` binds the entire output object so nothing can be swapped independently.

#### Hidden amounts

Amounts are not stored as plaintext balances.
Each note carries its own fresh LWE ciphertext.

Important design point:

- we are not treating LWE as an endlessly reused homomorphic account balance,
- we treat it as a per-note hidden amount container,
- spending a note means consuming it and creating new fresh notes.

This keeps the model closer to private note/UTXO accounting than to a long-lived encrypted account state.

#### Hidden addresses

Receiver privacy is based on one-time receive bundles rather than stable public addresses.

The intended flow is:

1. receiver generates many `ReceiveBundle`s,
2. sender picks one unused bundle,
3. sender creates a note whose receiver-side recovery data is encrypted to that bundle,
4. chain only sees the encrypted `RecipientBox` plus a non-secret scan `hint`,
5. wallet identifies ownership by being able to open the box.

This means:

- chain observers do not see a stable public recipient address,
- recipient reuse is discouraged,
- receiver discovery is wallet-side, not account-index-side.

#### Commitments and private openings

The current note structure is commitment-heavy on purpose.

- `note_commit` binds the full output payload,
- `spend_policy_commit` binds the spend rule,
- `aux_commit` binds private witness material needed later for spending/proving,
- `RecipientBoxPlaintext` contains openings needed by the owner but not by the public chain.

This gives us a layered privacy model:

- public chain sees commitments and encrypted boxes,
- owner reconstructs the real spend data from bundle keys,
- prover later shows correctness of spending without publishing those openings.

#### Custom noise-budget accounting

The intended LWE accounting model is not "let ciphertext noise drift forever and hope decoding survives".
We explicitly track and reset the usable decoding margin through note lifecycle events.

The practical design direction is:

- every newly created output uses fresh encryption noise,
- every spend consumes old notes and creates new notes with new fresh noise,
- we do not rely on arbitrary long homomorphic accumulation inside one ciphertext,
- private witness data records enough metadata to classify the allowed decoding margin of that note.

In the current format direction this is represented by `AuxWitness`, which already contains:

- `amount`
- `witness_seed`
- `noise_class`
- `bundle_id`

Meaning of `noise_class` in the design:

- it is a compact wallet/prover-side classification of the admissible noise regime for that note,
- it allows proofs and wallet logic to reason about which decoding safety envelope was used,
- it prevents the system from treating all ciphertexts as if they had identical noise history.

The accounting idea is:

1. fresh outputs start in a known safe noise class,
2. proof/witness generation knows the class and witness seed,
3. spend proofs show that the consumed notes were valid under their declared class,
4. outputs are re-randomized / freshly encrypted so the budget is reset on the new notes.

So the chain-level privacy accounting is closer to:

- "consume hidden notes, prove valid openings and conservation, issue fresh hidden notes"

than to:

- "mutate one encrypted balance in place forever".

#### ZK relation we are moving toward

The final intended proof relation is roughly:

- prover knows openings for consumed notes,
- prover knows plaintext amounts hidden inside input `ct_amt`,
- prover knows nullifier secrets and spend-policy witnesses,
- prover proves input conservation against outputs and fee,
- prover proves each output note commitment matches its hidden payload,
- prover proves each output ciphertext is well-formed and within the allowed noise/decode regime,
- prover proves each nullifier is derived correctly and is unlinkable to a public receiver identity.

In short:

- hidden amounts come from LWE ciphertexts,
- hidden addresses come from one-time bundle addressing plus encrypted recipient boxes,
- commitments bind every note component,
- ZK is the layer that proves conservation and correctness without opening recipients or values,
- custom noise-budget accounting is the rule that keeps LWE-based hidden amounts decodable and auditable across note creation/spend cycles.

### 3. `nxms-transport` protocol and runtime

Completed:

- `NXMS/2` neutral wire format alongside legacy escrow-centric flow.
- Context-oriented helpers:
  - `encrypt_for_context`
  - `decrypt_for_context`
- Runtime fixes for Alpine/musl.
- Canonical tests and roundtrip tests.

Key files:

- `nxms-transport/src/wire.rs`
- `nxms-transport/src/crypto.rs`
- `nxms-transport/build.rs`
- `nxms-transport/tests/crypto_roundtrip.rs`

Important finding:

- Earlier `SIGSEGV` was narrowed to a musl/static-runtime issue, not a protocol bug.
- The fix was to avoid the problematic static-runtime path for that PQ/native stack.

### 4. NXMS audit and fuzzing

This is the most important continuation lane for the next chat.

Completed:

- Audit notes were written and updated:
  - `audyt AEAD.md`
  - `audyt/nxms_ms_transport_aead_audit_2026-03-27.md`
- Independent Python reference implementation:
  - `audyt/nxms_ms_transport_reference_impl.py`
  - `audyt/nxms_ms_transport_reference_vectors_v1.json`
- `HASH(kem_ct)` versus full-binding tradeoff was explicitly documented.
- Security claim / residual risk framing was started.
- `cargo-fuzz` crash was narrowed down as a tooling/runtime finding on musl, not a confirmed NXMS functional bug.
- AFL++ based C harness suite was built and used as the main deep fuzzing path.

Important fuzz / audit infrastructure now exists in:

- `nxms-transport/fuzz_c/README.md`
- `nxms-transport/fuzz_c/run_nxms_fuzz_suite.sh`
- `nxms-transport/fuzz_c/run_nxms_fuzz_campaign.sh`
- `nxms-transport/fuzz_c/run_nxms_deep_fuzz_campaign.sh`
- `nxms-transport/fuzz_c/run_nxms_sanitizer_suite.sh`
- `nxms-transport/fuzz_c/run_nxms_coverage_report.sh`

Important harness families already exist:

- parser-only harness
- raw decrypt harness
- structured decrypt mutation harness
- KEM decapsulation harness
- KEM structured mutation harness
- Falcon sign/verify mutation harnesses
- Falcon CT/timing/ctgrind harnesses

Important conclusion:

- Native NXMS C code held up under AFL++ and sanitizer replay without crashes.
- `cargo-fuzz` on musl remains a tooling issue to keep documented separately.

### 5. Falcon CT audit

This was one of the deepest parts of the session.

Completed:

- Production CT build was forced:
  - `FALCON_FPEMU=1`
  - `FALCON_FPNATIVE=0`
- Falcon wrapper fuzzing and coverage were expanded.
- Active `sign_dyn` path coverage was driven into the real critical internals.
- Timing lanes were added for:
  - sign: fixed message vs random message
  - sign: key class A vs key class B
  - verify: valid relation vs invalid relation

Important result:

- Falcon correctness evidence is strong.
- Timing evidence on the compiled CT wrapper path is good.
- The real blocker was `ctgrind` on the encoded secret-key signing path.

### 6. Prepared signer solution

This is the key security hardening result of the whole session.

We stayed with the Falcon reference implementation, but changed how it is used.

Before:

- each signing call used encoded secret key bytes directly
- decode / reconstruct private key happened in the hot path
- `ctgrind` on the signing path was not clean

After:

- secret key is prepared once into signer state
- request-time signing uses prepared `sign_dyn`
- encoded-key signing remains only as a fallback path

This was implemented in:

- `nxms-transport/native/exum_cli_src/pqc_falcon.h`
- `nxms-transport/native/exum_cli_src/pqc_falcon.c`
- `nxms-transport/native/nxms_ms_transport.h`
- `nxms-transport/native/nxms_ms_transport.c`
- `nxms-transport/src/crypto.rs`

Important result:

- encoded-key path still shows `ctgrind` findings
- prepared signing path is clean under the current `ctgrind` lane

This is the main reason the NXMS audit can continue without changing Falcon as an algorithm.

### 7. `nexum-cli`

We did not leave it half-broken.

Completed:

- Real runtime call sites were migrated to the prepared signer path.
- `vault_load -> signer prepared once -> runtime signing -> vault_free` lifecycle now exists.
- `dm-send`, `prekeys`, `register`, `login`, `respond` and manifest signing use the prepared path when available.
- `nexum-cli` now has its own Alpine build lane:
  - `nexum-cli/Makefile`
  - `nexum-cli/README.md`

Important files:

- `nexum-cli/src/pqc_falcon.h`
- `nexum-cli/src/pqc_falcon.c`
- `nexum-cli/src/vault.h`
- `nexum-cli/src/vault.c`
- `nexum-cli/src/dm.h`
- `nexum-cli/src/dm.c`
- `nexum-cli/src/cli_ext_cmds.c`
- `nexum-cli/src/cli_auth_core_cmds.c`

Important note:

- `nexum-cli` still contains legacy escrow/operator-heavy command surface.
- We explicitly decided not to optimize around that old architecture right now.
- The current status is: it builds and the user-side/signing path is hardened.

## Verified commands that passed

### `nexum-cli`

Executed on Alpine:

```sh
cd /home/nxms-server/privAI/nexum-cli
make deps-check
make -j2
./nexum
```

Result:

- build succeeded
- binary linked successfully
- usage output prints correctly

### `nxms-transport`

Over the session, the following classes of checks were run successfully on Alpine:

- `cargo test --features crypto`
- dedicated regression tests
- AFL++ fuzz campaigns
- sanitizer replay
- Falcon timing harnesses
- prepared-path `ctgrind` lane

The exact scripts live under:

- `nxms-transport/fuzz_c/`

## Current security / audit position

### Strongly established

- NXMS native C transport is stable under current AFL++ and sanitizer campaigns.
- Falcon CT build policy is enforced.
- Prepared signer path materially improves implementation-side security.
- `nexum-cli` and `nxms-transport` now align on that prepared signer model.
- `HASH(kem_ct)` is a conscious v1 design choice and is documented as such.

## What was closed at the end of this chat

These points were the final concrete closures before context overflow:

1. `nexum-cli` was not left half-finished.
   - Alpine build lane exists.
   - The binary builds and runs.
   - Real signing call sites now prefer the prepared Falcon signer path.

2. Falcon was not changed as an algorithm.
   - We stayed with the Falcon reference implementation.
   - The hardening came from changing the runtime usage model:
     - encoded key path remains as fallback
     - prepared signer path is the intended hot path

3. The meaningful audit interpretation is now:
   - correctness evidence for Falcon is strong
   - timing evidence for the compiled prepared path is good
   - `ctgrind` findings on the old encoded-key signing path should not drive production usage
   - the prepared signer path is the security baseline going forward

4. The next chat should not re-open old escrow-centric product discussion by default.
   - The right continuation lane is NXMS audit depth.
   - `nexum-cli` slimming can happen later as a separate cleanup project.

### Known open items

1. Continue the NXMS audit, not broad product architecture.
2. Keep documenting NXMS as an autorski transport systemowy for `privAI`, not as a generic off-the-shelf AEAD clone.
3. Finish the remaining transport-security documentation in the language of:
   - scope
   - guarantees
   - assumptions
   - residual risks
4. Continue with FrodoKEM-side audit depth analogous to Falcon.
5. Later, slim `nexum-cli` away from legacy escrow/operator surface instead of strengthening that old flow.

## Best next step in a new chat

The next chat should start from:

1. `CHAT_HANDOFF_2026-03-30.md`
2. `audyt/nxms_ms_transport_aead_audit_2026-03-27.md`
3. `audyt AEAD.md`
4. `nxms-transport/fuzz_c/README.md`
5. `nxms-transport/native/nxms_ms_transport.c`
6. `nxms-transport/src/crypto.rs`

Recommended first sentence for the next chat:

`Wracamy do audytu NXMS. Pracujemy na Alpine w /home/nxms-server/privAI. Punkt startowy: prepared signer path jest wdrożony, nexum-cli build jest zielony, cargo-fuzz/musl to finding narzędziowy, a teraz chcemy kontynuować docelowy audit naszego autorskiego transportu systemowego.`

## Final practical summary

If the next chat only remembers five things, they are:

1. Alpine is the source of truth.
2. Prepared Falcon signing path is the correct hot path.
3. `nexum-cli` is built and not left broken.
4. NXMS audit now has real fuzz/sanitizer/timing infrastructure.
5. The next work item is to continue NXMS audit depth, not to re-open old escrow architecture.
