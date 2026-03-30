#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

build_with_sanitizers() {
    CC=clang \
    CFLAGS="-fsanitize=address,undefined -fno-omit-frame-pointer" \
    LDFLAGS="-fsanitize=address,undefined" \
    sh "$1" >/dev/null
}

run_fixture_input_cases() {
    HARNESS="$1"
    FIXTURE="$2"
    DIR="$3"
    LABEL="$4"

    if [ ! -d "$DIR" ]; then
        return 0
    fi

    COUNT=0
    for INPUT in "$DIR"/*; do
        if [ ! -f "$INPUT" ]; then
            continue
        fi
        "$HARNESS" "$FIXTURE" "$INPUT" >/dev/null
        COUNT=$((COUNT + 1))
    done
    printf '%s: %s cases\n' "$LABEL" "$COUNT"
}

run_single_input_cases() {
    HARNESS="$1"
    DIR="$2"
    LABEL="$3"

    if [ ! -d "$DIR" ]; then
        return 0
    fi

    COUNT=0
    for INPUT in "$DIR"/*; do
        if [ ! -f "$INPUT" ]; then
            continue
        fi
        "$HARNESS" "$INPUT" >/dev/null
        COUNT=$((COUNT + 1))
    done
    printf '%s: %s cases\n' "$LABEL" "$COUNT"
}

run_mutation_cases() {
    HARNESS="$1"
    FIXTURE="$2"
    BASE="$3"
    DIR="$4"
    LABEL="$5"

    if [ ! -d "$DIR" ]; then
        return 0
    fi

    COUNT=0
    for INPUT in "$DIR"/*; do
        if [ ! -f "$INPUT" ]; then
            continue
        fi
        "$HARNESS" "$FIXTURE" "$BASE" "$INPUT" >/dev/null
        COUNT=$((COUNT + 1))
    done
    printf '%s: %s cases\n' "$LABEL" "$COUNT"
}

FIXTURE="$ROOT/fuzz_c/corpus/fixture.bin"
FALCON_FIXTURE="$ROOT/fuzz_c/corpus/falcon_fixture.bin"
BASE_PACKET="$ROOT/fuzz_c/corpus/seed_valid.bin"
BASE_KEM="$ROOT/fuzz_c/corpus_kem/kem_ct_valid.bin"
DECRYPT_VALID_DIR="$ROOT/fuzz_c/corpus_decrypt_valid"
KEM_VALID_DIR="$ROOT/fuzz_c/corpus_kem_valid"
FALCON_MUT_DIR="$ROOT/fuzz_c/corpus_falcon_mut"
FALCON_SEED_DIR="$ROOT/fuzz_c/corpus_falcon_seed"
DECRYPT_QUEUE="${DECRYPT_QUEUE:-$ROOT/fuzz_c/findings/default/queue}"
KEM_QUEUE="${KEM_QUEUE:-$ROOT/fuzz_c/findings_kem/default/queue}"
PARSER_QUEUE="${PARSER_QUEUE:-$ROOT/fuzz_c/findings_parser/default/queue}"
DECRYPT_MUT_QUEUE="${DECRYPT_MUT_QUEUE:-$ROOT/fuzz_c/findings_decrypt_mut/default/queue}"
KEM_MUT_QUEUE="${KEM_MUT_QUEUE:-$ROOT/fuzz_c/findings_kem_mut/default/queue}"
FALCON_MUT_QUEUE="${FALCON_MUT_QUEUE:-$ROOT/fuzz_c/findings_falcon_mut/default/queue}"
FALCON_SEED_QUEUE="${FALCON_SEED_QUEUE:-$ROOT/fuzz_c/findings_falcon_seed/default/queue}"
KEM_REGRESSION_DIR="$ROOT/fuzz_c/corpus_kem_regression"

echo "[1/5] refresh fixture and corpora"
(cd "$ROOT" && cargo run --example emit_c_fuzz_fixture --features crypto -- fuzz_c/corpus >/dev/null)

echo "[2/5] build sanitizer harnesses"
(cd "$ROOT" && build_with_sanitizers fuzz_c/build_nxms_packet_parser_harness.sh)
(cd "$ROOT" && build_with_sanitizers fuzz_c/build_nxms_ms_verify_decrypt_harness.sh)
(cd "$ROOT" && build_with_sanitizers fuzz_c/build_nxms_ms_verify_decrypt_mut_harness.sh)
(cd "$ROOT" && build_with_sanitizers fuzz_c/build_pqc_kem_decaps_harness.sh)
(cd "$ROOT" && build_with_sanitizers fuzz_c/build_pqc_kem_decaps_mut_harness.sh)
(cd "$ROOT" && build_with_sanitizers fuzz_c/build_pqc_falcon_sign_verify_mut_harness.sh)
(cd "$ROOT" && build_with_sanitizers fuzz_c/build_pqc_falcon_wrapper_mut_harness.sh)
(cd "$ROOT" && build_with_sanitizers fuzz_c/build_pqc_falcon_sign_seed_harness.sh)

echo "[3/5] sanitizer self-tests"
"$ROOT/fuzz_c/fuzz_nxms_parser" --self-test "$ROOT/fuzz_c/corpus/seed_valid.bin"
"$ROOT/fuzz_c/fuzz_nxms_decrypt" --self-test "$FIXTURE" "$ROOT/fuzz_c/corpus/seed_valid.bin"
"$ROOT/fuzz_c/fuzz_nxms_decrypt_mut" --self-test "$FIXTURE" "$BASE_PACKET" /dev/null
"$ROOT/fuzz_c/fuzz_pqc_kem_decaps" --self-test "$FIXTURE" "$ROOT/fuzz_c/corpus/seed_valid.bin"
"$ROOT/fuzz_c/fuzz_pqc_kem_decaps_mut" --self-test "$FIXTURE" "$BASE_KEM" /dev/null
"$ROOT/fuzz_c/fuzz_pqc_falcon_sign_verify_mut" --self-test "$FALCON_FIXTURE" /dev/null
"$ROOT/fuzz_c/fuzz_pqc_falcon_wrapper_mut" --self-test "$FALCON_FIXTURE" /dev/null
"$ROOT/fuzz_c/fuzz_pqc_falcon_sign_seed" --self-test "$FALCON_FIXTURE" /dev/null

echo "[4/5] replay decrypt inputs under ASan/UBSan"
run_single_input_cases "$ROOT/fuzz_c/fuzz_nxms_parser" "$ROOT/fuzz_c/corpus_decrypt" "parser corpus"
run_single_input_cases "$ROOT/fuzz_c/fuzz_nxms_parser" "$DECRYPT_VALID_DIR" "parser valid corpus"
run_single_input_cases "$ROOT/fuzz_c/fuzz_nxms_parser" "$PARSER_QUEUE" "parser queue"
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_nxms_decrypt" "$FIXTURE" "$ROOT/fuzz_c/corpus_decrypt" "decrypt corpus"
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_nxms_decrypt" "$FIXTURE" "$DECRYPT_VALID_DIR" "decrypt valid corpus"
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_nxms_decrypt" "$FIXTURE" "$DECRYPT_QUEUE" "decrypt queue"
run_mutation_cases "$ROOT/fuzz_c/fuzz_nxms_decrypt_mut" "$FIXTURE" "$BASE_PACKET" "$ROOT/fuzz_c/corpus_decrypt_mut" "decrypt mut corpus"
run_mutation_cases "$ROOT/fuzz_c/fuzz_nxms_decrypt_mut" "$FIXTURE" "$BASE_PACKET" "$DECRYPT_MUT_QUEUE" "decrypt mut queue"

echo "[5/5] replay kem inputs under ASan/UBSan"
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_pqc_kem_decaps" "$FIXTURE" "$ROOT/fuzz_c/corpus_kem" "kem corpus"
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_pqc_kem_decaps" "$FIXTURE" "$KEM_VALID_DIR" "kem valid corpus"
if [ -f "$ROOT/fuzz_c/corpus/seed_tampered_kem.bin" ]; then
    "$ROOT/fuzz_c/fuzz_pqc_kem_decaps" "$FIXTURE" "$ROOT/fuzz_c/corpus/seed_tampered_kem.bin" >/dev/null
    printf '%s\n' "kem regression: 1 case"
fi
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_pqc_kem_decaps" "$FIXTURE" "$KEM_REGRESSION_DIR" "kem regression corpus"
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_pqc_kem_decaps" "$FIXTURE" "$KEM_QUEUE" "kem queue"
run_mutation_cases "$ROOT/fuzz_c/fuzz_pqc_kem_decaps_mut" "$FIXTURE" "$BASE_KEM" "$ROOT/fuzz_c/corpus_kem_mut" "kem mut corpus"
run_mutation_cases "$ROOT/fuzz_c/fuzz_pqc_kem_decaps_mut" "$FIXTURE" "$BASE_KEM" "$KEM_MUT_QUEUE" "kem mut queue"
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_verify_mut" "$FALCON_FIXTURE" "$FALCON_MUT_DIR" "falcon mut corpus"
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_verify_mut" "$FALCON_FIXTURE" "$FALCON_MUT_QUEUE" "falcon mut queue"
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_pqc_falcon_wrapper_mut" "$FALCON_FIXTURE" "$FALCON_MUT_DIR" "falcon wrapper corpus"
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_pqc_falcon_wrapper_mut" "$FALCON_FIXTURE" "$FALCON_MUT_QUEUE" "falcon wrapper queue"
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_seed" "$FALCON_FIXTURE" "$FALCON_SEED_DIR" "falcon sign-seed corpus"
run_fixture_input_cases "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_seed" "$FALCON_FIXTURE" "$FALCON_SEED_QUEUE" "falcon sign-seed queue"

echo "nxms sanitizer suite ok"
