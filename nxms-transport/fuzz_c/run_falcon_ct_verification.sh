#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
OUT_DIR="$ROOT/fuzz_c/coverage/falcon_freeze"
SAMPLES="${1:-4096}"

mkdir -p "$OUT_DIR"

printf '[1/6] wrapper seeded KAT\n'
cargo test --no-default-features --features crypto,falcon-audit-raw-api --test falcon_wrapper_kat \
  >"$OUT_DIR/falcon_wrapper_kat.log" 2>&1

printf '[2/6] Round 3 packaged KAT reproduction\n'
sh "$ROOT/fuzz_c/run_falcon_round3_reference_lane.sh" \
  "$OUT_DIR/falcon_round3_reference" \
  >"$OUT_DIR/falcon_round3_reference.log" 2>&1

printf '[3/6] dudect-like timing smoke (%s samples)\n' "$SAMPLES"
cargo run --example emit_c_fuzz_fixture --features crypto -- "$ROOT/fuzz_c/corpus" \
  >/dev/null 2>&1
sh "$ROOT/fuzz_c/build_pqc_falcon_sign_ttest_harness.sh" >/dev/null
"$ROOT/fuzz_c/fuzz_pqc_falcon_sign_ttest" "$ROOT/fuzz_c/corpus/falcon_fixture.bin" "$SAMPLES" \
  >"$OUT_DIR/falcon_sign_msg_ttest_${SAMPLES}.log" 2>&1
sh "$ROOT/fuzz_c/build_pqc_falcon_sign_keyclass_ttest_harness.sh" >/dev/null
"$ROOT/fuzz_c/fuzz_pqc_falcon_sign_keyclass_ttest" "$SAMPLES" \
  >"$OUT_DIR/falcon_sign_keyclass_ttest_${SAMPLES}.log" 2>&1
sh "$ROOT/fuzz_c/build_pqc_falcon_verify_ttest_harness.sh" >/dev/null
"$ROOT/fuzz_c/fuzz_pqc_falcon_verify_ttest" "$SAMPLES" \
  >"$OUT_DIR/falcon_verify_ttest_${SAMPLES}.log" 2>&1

printf '[4/6] legacy encoded-key wrapper sign path (audit only)\n'
sh "$ROOT/fuzz_c/build_pqc_falcon_wrapper_ctgrind_harness.sh" >/dev/null
LEGACY_CTGRIND_SK_STATUS=0
LEGACY_CTGRIND_MSG_STATUS=0
valgrind --tool=memcheck --track-origins=yes --error-exitcode=99 \
  "$ROOT/fuzz_c/fuzz_pqc_falcon_wrapper_ctgrind" sk \
  >"$OUT_DIR/falcon_ctgrind_sk.log" 2>&1 || LEGACY_CTGRIND_SK_STATUS=$?
valgrind --tool=memcheck --track-origins=yes --error-exitcode=99 \
  "$ROOT/fuzz_c/fuzz_pqc_falcon_wrapper_ctgrind" msg \
  >"$OUT_DIR/falcon_ctgrind_msg.log" 2>&1 || LEGACY_CTGRIND_MSG_STATUS=$?

printf '[5/6] ctgrind prepared sign_dyn sign path\n'
sh "$ROOT/fuzz_c/build_pqc_falcon_prepared_dyn_ctgrind_harness.sh" >/dev/null
PREPARED_CTGRIND_SK_STATUS=0
PREPARED_CTGRIND_MSG_STATUS=0
valgrind --tool=memcheck --track-origins=yes --error-exitcode=99 \
  "$ROOT/fuzz_c/fuzz_pqc_falcon_prepared_dyn_ctgrind" sk \
  >"$OUT_DIR/falcon_prepared_dyn_ctgrind_sk.log" 2>&1 || PREPARED_CTGRIND_SK_STATUS=$?
valgrind --tool=memcheck --track-origins=yes --error-exitcode=99 \
  "$ROOT/fuzz_c/fuzz_pqc_falcon_prepared_dyn_ctgrind" msg \
  >"$OUT_DIR/falcon_prepared_dyn_ctgrind_msg.log" 2>&1 || PREPARED_CTGRIND_MSG_STATUS=$?

printf '[6/6] objdump review artifacts\n'
objdump --disassemble=ff_falcon_sign_ct \
        --disassemble=ff_falcon_verify \
        "$ROOT/fuzz_c/fuzz_pqc_falcon_wrapper_ctgrind" \
  >"$OUT_DIR/falcon_wrapper_objdump.txt"
objdump --disassemble=ff_falcon_sign_ct_prepared \
        "$ROOT/fuzz_c/fuzz_pqc_falcon_prepared_dyn_ctgrind" \
  >"$OUT_DIR/falcon_prepared_dyn_objdump.txt"
objdump --disassemble=falcon_inner_sign_dyn \
        --disassemble=do_sign_dyn \
        --disassemble=ffSampling_fft_dyntree.constprop.0 \
        "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_ttest" \
  >"$OUT_DIR/falcon_sign_path_objdump.txt"

SIGN_JCC="$(awk '/<falcon_inner_sign_dyn>:/,/^$/' "$OUT_DIR/falcon_sign_path_objdump.txt" | grep -Eo '\bj[a-z]+\b' | wc -l | tr -d ' ')"
SIGN_CMOV="$(awk '/<falcon_inner_sign_dyn>:/,/^$/' "$OUT_DIR/falcon_sign_path_objdump.txt" | grep -Eo '\bcmov[a-z]*\b' | wc -l | tr -d ' ')"
WRAP_JCC="$(awk '/<ff_falcon_sign_ct>:/,/^$/' "$OUT_DIR/falcon_wrapper_objdump.txt" | grep -Eo '\bj[a-z]+\b' | wc -l | tr -d ' ')"
WRAP_CMOV="$(awk '/<ff_falcon_sign_ct>:/,/^$/' "$OUT_DIR/falcon_wrapper_objdump.txt" | grep -Eo '\bcmov[a-z]*\b' | wc -l | tr -d ' ')"
SIGN_MSG_T="$(awk -F= '/welch_t=/{print $2}' "$OUT_DIR/falcon_sign_msg_ttest_${SAMPLES}.log" | tail -n 1)"
SIGN_KEY_T="$(awk -F= '/welch_t=/{print $2}' "$OUT_DIR/falcon_sign_keyclass_ttest_${SAMPLES}.log" | tail -n 1)"
VERIFY_T="$(awk -F= '/welch_t=/{print $2}' "$OUT_DIR/falcon_verify_ttest_${SAMPLES}.log" | tail -n 1)"

{
  printf 'samples=%s\n' "$SAMPLES"
  printf 'runtime_path=prepared_signer_only\n'
  printf 'sign_msg_welch_t=%s\n' "${SIGN_MSG_T:-NA}"
  printf 'sign_keyclass_welch_t=%s\n' "${SIGN_KEY_T:-NA}"
  printf 'verify_welch_t=%s\n' "${VERIFY_T:-NA}"
  printf 'sign_dyn_jcc=%s\n' "$SIGN_JCC"
  printf 'sign_dyn_cmov=%s\n' "$SIGN_CMOV"
  printf 'legacy_wrapper_sign_jcc=%s\n' "$WRAP_JCC"
  printf 'legacy_wrapper_sign_cmov=%s\n' "$WRAP_CMOV"
  printf 'falcon_round3_reference_summary=%s\n' \
    "$OUT_DIR/falcon_round3_reference/falcon_round3_reference_summary.txt"
  printf 'legacy_ctgrind_sk_status=%s\n' "$LEGACY_CTGRIND_SK_STATUS"
  printf 'legacy_ctgrind_msg_status=%s\n' "$LEGACY_CTGRIND_MSG_STATUS"
  printf 'prepared_ctgrind_sk_status=%s\n' "$PREPARED_CTGRIND_SK_STATUS"
  printf 'prepared_ctgrind_msg_status=%s\n' "$PREPARED_CTGRIND_MSG_STATUS"
  printf 'legacy_ctgrind_logs=%s,%s\n' \
    "$OUT_DIR/falcon_ctgrind_sk.log" \
    "$OUT_DIR/falcon_ctgrind_msg.log"
  printf 'prepared_ctgrind_logs=%s,%s\n' \
    "$OUT_DIR/falcon_prepared_dyn_ctgrind_sk.log" \
    "$OUT_DIR/falcon_prepared_dyn_ctgrind_msg.log"
} >"$OUT_DIR/falcon_objdump_summary.txt"

printf 'falcon freeze artifacts: %s\n' "$OUT_DIR"

if [ "$PREPARED_CTGRIND_SK_STATUS" -ne 0 ] || [ "$PREPARED_CTGRIND_MSG_STATUS" -ne 0 ]; then
  exit 1
fi
