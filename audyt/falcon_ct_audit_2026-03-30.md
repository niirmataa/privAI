# Falcon CT audit note for NXMS transport

Data audytu: 2026-03-30
Aktualizacja: 2026-03-31

## Scope

This note closes the current Falcon-side audit lane for NXMS transport as a production-usage decision, not as a claim that every historical signing path is equally acceptable.

In scope:
- `nxms-transport/native/nexum_cli_src/pqc_falcon.c`
- `nxms-transport/native/nexum_cli_src/pqc_falcon.h`
- `nxms-transport/src/crypto.rs`
- Falcon wrapper KATs
- Falcon Round 3 package baseline and vendor sync
- prepared signer transport integration
- timing smoke lanes
- `ctgrind` comparison of encoded-key wrapper path vs prepared-key sign path

Out of scope:
- formal proof of Falcon CT security
- replacing Falcon as an algorithm
- complete verification of every legacy caller outside the current prepared-path migration model

## Current verdict

Falcon is closed for the current NXMS production build under the prepared signer path.

That means:
- accepted production hot path: `prepare once -> sign many` via prepared signer state
- accepted verification path: current `ff_falcon_verify()` wrapper path
- audit-only raw path: repeated encoded-key signing through `ff_falcon_sign_ct()` / `ff_falcon_sign_ct_seeded()` compiled only behind explicit audit switches

No new P1/P2 vulnerability was found in the prepared-path Falcon usage model during this pass.

## Fresh evidence collected in this step

Executed on Alpine in `/home/nxms-server/privAI/nxms-transport`:

1. `cargo test --features crypto --test crypto_roundtrip --test crypto_negative --test crypto_reference_vectors --test crypto_proptest`
   - runtime signer-only transport lane: `PASS`

2. `cargo test --no-default-features --features crypto,falcon-audit-raw-api --test falcon_wrapper_kat`
   - wrapper seeded KAT lane remained green under explicit audit feature gate

3. `sh fuzz_c/run_falcon_ct_verification.sh 2048`
   - script exits `0`
   - runtime gate is now the prepared signer lane

4. Fresh summary from `fuzz_c/coverage/falcon_freeze/falcon_objdump_summary.txt`:
   - `legacy_ctgrind_sk_status=99`
   - `legacy_ctgrind_msg_status=0`
   - `prepared_ctgrind_sk_status=0`
   - `prepared_ctgrind_msg_status=0`
   - `sign_msg_welch_t=0.248778`
   - `sign_keyclass_welch_t=0.859842`
   - `verify_welch_t=0.391850`

5. Fresh Round 3 sync / KAT summary from `fuzz_c/coverage/falcon_freeze/falcon_round3_reference/falcon_round3_reference_summary.txt`:
   - `native/vendor/falcon` is byte-to-byte aligned with `falcon-round3/Extra/c`
   - packaged `falcon512/1024` `.req/.rsp` files reproduce exactly

## Established guarantees

1. Prepared signing path is functionally aligned with the wrapper reference lane.
   - `falcon_prepared_seeded_matches_reference_wrapper` already proves seeded prepared signing matches the seeded wrapper reference output.
   - transport roundtrip with `PreparedTransportSigner` is green.

2. Prepared signing path is clean under the current `ctgrind` lane.
   - both `sk` and `msg` prepared lanes returned `0 errors from 0 contexts`.
   - this is the decisive implementation-side security property for the accepted hot path.

3. Encoded-key wrapper path is not clean under the current `ctgrind` lane.
   - fresh logs still show secret-dependent findings through decode / private completion stages when the encoded secret key is marked secret.
   - representative stack points still include `falcon_inner_trim_i8_decode` and `falcon_inner_complete_private`.
   - this lane now exists only behind explicit audit switches and no longer defines the production build surface.

4. Timing smoke evidence for the compiled wrapper lanes remains practically calm.
   - no meaningful class split was observed in the current message, key-class, or verify valid/invalid lanes.
   - this is useful evidence, but it does not override the encoded-key `ctgrind` finding.

5. Verification path remains acceptable in the current model.
   - current verify timing smoke is calm
   - no new memory-safety issue surfaced in this pass

6. Falcon Round 3 is now the canonical upstream package baseline for this audit lane.
   - `falcon-round3/Extra/c` is the source-of-truth upstream tree
   - `native/vendor/falcon` is required to stay byte-to-byte aligned and is checked by script

## Production decision boundary

### Accepted

1. `PreparedTransportSigner` and the native prepared signer context are the production signing baseline.
2. Runtime callers should prepare Falcon secret state once and reuse it for request-time signing.
3. `ff_falcon_verify()` stays accepted as the verification path in the current transport model.
4. Raw encoded-key signing is not part of the default production build surface anymore.

### Not accepted as production hot path

1. Re-decoding encoded Falcon secret key bytes on every signature request.
2. Enabling `falcon-audit-raw-api` / `NXMS_FALCON_AUDIT_RAW_API` outside explicit audit work.
3. Claiming the whole Falcon surface is equally closed without distinguishing prepared-path vs encoded-key path.

## Assumptions

1. We continue to rely on the Falcon reference implementation rather than swapping algorithms.
2. We treat the prepared-key usage model as the relevant operational security boundary.
3. We do not claim that `ctgrind` alone proves constant-time behavior; it is one strong implementation-side signal among several.
4. We accept timing-smoke results as practical evidence, not as a formal side-channel proof.

## Residual risks

1. The encoded-key signing path still exists in code as audit surface.
   - It is acceptable only because it is feature-gated and not part of the default runtime build.

2. Falcon audit is closed operationally, not formally.
   - There is still no formal proof document equivalent to a complete cryptographic side-channel proof.

3. Future source updates must preserve Round 3 sync.
   - The system is safe to the extent that `native/vendor/falcon` continues to match `falcon-round3/Extra/c`.

## Final closure statement for this phase

Falcon is now closed for NXMS in the only way that matters operationally:
- prepared signer path: accepted
- encoded-key raw path: retained only as explicit audit/test surface
- verification path: accepted with current evidence

So the correct final sentence is not:
- "all Falcon paths are clean"

The correct final sentence is:
- "the prepared Falcon signing path is the closed production path; the encoded-key signing path is audit-only and does not define the system security baseline."
