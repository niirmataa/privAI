#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
DURATION="${1:-180}"
PARSER_WORKERS="${2:-2}"
DECRYPT_WORKERS="${3:-4}"
DECRYPT_MUT_WORKERS="${4:-4}"
KEM_WORKERS="${5:-3}"
KEM_MUT_WORKERS="${6:-3}"
FALCON_MUT_WORKERS="${7:-2}"
FALCON_SEED_WORKERS="${8:-2}"
CAMPAIGN_ROOT="$ROOT/fuzz_c/campaign_deep"
PARSER_OUT="$CAMPAIGN_ROOT/parser"
DECRYPT_OUT="$CAMPAIGN_ROOT/decrypt"
DECRYPT_MUT_OUT="$CAMPAIGN_ROOT/decrypt_mut"
KEM_OUT="$CAMPAIGN_ROOT/kem"
KEM_MUT_OUT="$CAMPAIGN_ROOT/kem_mut"
FALCON_MUT_OUT="$CAMPAIGN_ROOT/falcon_mut"
FALCON_SEED_OUT="$CAMPAIGN_ROOT/falcon_seed"
FIXTURE="$ROOT/fuzz_c/corpus/fixture.bin"
FALCON_FIXTURE="$ROOT/fuzz_c/corpus/falcon_fixture.bin"
BASE_PACKET="$ROOT/fuzz_c/corpus/seed_valid.bin"
BASE_KEM="$ROOT/fuzz_c/corpus_kem/kem_ct_valid.bin"

if ! command -v afl-fuzz >/dev/null 2>&1 || ! command -v afl-clang-fast >/dev/null 2>&1; then
    echo "AFL++ not available"
    exit 1
fi

if ! command -v afl-whatsup >/dev/null 2>&1; then
    echo "afl-whatsup not available"
    exit 1
fi

run_single_swarm() {
    KIND="$1"
    OUTDIR="$2"
    INPUT_DIR="$3"
    DICT="$4"
    HARNESS="$5"
    WORKERS="$6"

    rm -rf "$OUTDIR"
    mkdir -p "$OUTDIR"

    SESSION_PIDS=""
    INDEX=0
    while [ "$INDEX" -lt "$WORKERS" ]; do
        if [ "$INDEX" -eq 0 ]; then
            ROLE_ARGS="-M ${KIND}_main"
        else
            ROLE_ARGS="-S ${KIND}_s${INDEX}"
        fi

        (
            cd "$ROOT"
            AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 \
            afl-fuzz -V "$DURATION" $ROLE_ARGS \
                -x "$DICT" \
                -i "$INPUT_DIR" \
                -o "$OUTDIR" \
                -- "$HARNESS" @@
        ) &
        SESSION_PIDS="${SESSION_PIDS} $!"
        INDEX=$((INDEX + 1))
    done

    for PID in $SESSION_PIDS; do
        wait "$PID"
    done
}

run_fixture_stdin_swarm() {
    KIND="$1"
    OUTDIR="$2"
    INPUT_DIR="$3"
    DICT="$4"
    HARNESS="$5"
    WORKERS="$6"
    FIXTURE_ARG="$7"

    rm -rf "$OUTDIR"
    mkdir -p "$OUTDIR"

    SESSION_PIDS=""
    INDEX=0
    while [ "$INDEX" -lt "$WORKERS" ]; do
        if [ "$INDEX" -eq 0 ]; then
            ROLE_ARGS="-M ${KIND}_main"
        else
            ROLE_ARGS="-S ${KIND}_s${INDEX}"
        fi

        (
            cd "$ROOT"
            AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 \
            afl-fuzz -V "$DURATION" $ROLE_ARGS \
                -x "$DICT" \
                -i "$INPUT_DIR" \
                -o "$OUTDIR" \
                -- "$HARNESS" "$FIXTURE_ARG" @@
        ) &
        SESSION_PIDS="${SESSION_PIDS} $!"
        INDEX=$((INDEX + 1))
    done

    for PID in $SESSION_PIDS; do
        wait "$PID"
    done
}

run_fixture_swarm() {
    KIND="$1"
    OUTDIR="$2"
    INPUT_DIR="$3"
    DICT="$4"
    HARNESS="$5"
    WORKERS="$6"
    FIXTURE_ARG="$7"

    rm -rf "$OUTDIR"
    mkdir -p "$OUTDIR"

    SESSION_PIDS=""
    INDEX=0
    while [ "$INDEX" -lt "$WORKERS" ]; do
        if [ "$INDEX" -eq 0 ]; then
            ROLE_ARGS="-M ${KIND}_main"
        else
            ROLE_ARGS="-S ${KIND}_s${INDEX}"
        fi

        (
            cd "$ROOT"
            AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 \
            afl-fuzz -V "$DURATION" $ROLE_ARGS \
                -x "$DICT" \
                -i "$INPUT_DIR" \
                -o "$OUTDIR" \
                -- "$HARNESS" "$FIXTURE_ARG" @@
        ) &
        SESSION_PIDS="${SESSION_PIDS} $!"
        INDEX=$((INDEX + 1))
    done

    for PID in $SESSION_PIDS; do
        wait "$PID"
    done
}

run_mut_swarm() {
    KIND="$1"
    OUTDIR="$2"
    INPUT_DIR="$3"
    DICT="$4"
    HARNESS="$5"
    WORKERS="$6"
    FIXTURE_ARG="$7"
    BASE_ARG="$8"

    rm -rf "$OUTDIR"
    mkdir -p "$OUTDIR"

    SESSION_PIDS=""
    INDEX=0
    while [ "$INDEX" -lt "$WORKERS" ]; do
        if [ "$INDEX" -eq 0 ]; then
            ROLE_ARGS="-M ${KIND}_main"
        else
            ROLE_ARGS="-S ${KIND}_s${INDEX}"
        fi

        (
            cd "$ROOT"
            AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 \
            afl-fuzz -V "$DURATION" $ROLE_ARGS \
                -x "$DICT" \
                -i "$INPUT_DIR" \
                -o "$OUTDIR" \
                -- "$HARNESS" "$FIXTURE_ARG" "$BASE_ARG" @@
        ) &
        SESSION_PIDS="${SESSION_PIDS} $!"
        INDEX=$((INDEX + 1))
    done

    for PID in $SESSION_PIDS; do
        wait "$PID"
    done
}

echo "[1/9] refresh fixture and corpora"
(cd "$ROOT" && cargo run --example emit_c_fuzz_fixture --features crypto -- fuzz_c/corpus >/dev/null)

echo "[2/9] baseline replay outside libFuzzer"
(cd "$ROOT" && cargo test --features crypto --test crypto_corpus_runner >/dev/null)

echo "[3/9] rebuild AFL++ harnesses"
(cd "$ROOT" && rm -f fuzz_c/fuzz_nxms_parser fuzz_c/fuzz_nxms_decrypt fuzz_c/fuzz_nxms_decrypt_mut fuzz_c/fuzz_pqc_kem_decaps fuzz_c/fuzz_pqc_kem_decaps_mut fuzz_c/fuzz_pqc_falcon_sign_verify_mut fuzz_c/fuzz_pqc_falcon_wrapper_mut fuzz_c/fuzz_pqc_falcon_sign_seed)
(cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_nxms_packet_parser_harness.sh >/dev/null)
(cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_nxms_ms_verify_decrypt_harness.sh >/dev/null)
(cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_nxms_ms_verify_decrypt_mut_harness.sh >/dev/null)
(cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_pqc_kem_decaps_harness.sh >/dev/null)
(cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_pqc_kem_decaps_mut_harness.sh >/dev/null)
(cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_pqc_falcon_sign_verify_mut_harness.sh >/dev/null)
(cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_pqc_falcon_wrapper_mut_harness.sh >/dev/null)
(cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_pqc_falcon_sign_seed_harness.sh >/dev/null)

echo "[4/9] self-tests"
"$ROOT/fuzz_c/fuzz_nxms_parser" --self-test "$BASE_PACKET"
"$ROOT/fuzz_c/fuzz_nxms_decrypt" --self-test "$FIXTURE" "$BASE_PACKET"
"$ROOT/fuzz_c/fuzz_nxms_decrypt_mut" --self-test "$FIXTURE" "$BASE_PACKET" /dev/null
"$ROOT/fuzz_c/fuzz_pqc_kem_decaps" --self-test "$FIXTURE" "$BASE_PACKET"
"$ROOT/fuzz_c/fuzz_pqc_kem_decaps_mut" --self-test "$FIXTURE" "$BASE_KEM" /dev/null
"$ROOT/fuzz_c/fuzz_pqc_falcon_sign_verify_mut" --self-test "$FALCON_FIXTURE" /dev/null
"$ROOT/fuzz_c/fuzz_pqc_falcon_wrapper_mut" --self-test "$FALCON_FIXTURE" /dev/null
"$ROOT/fuzz_c/fuzz_pqc_falcon_sign_seed" --self-test "$FALCON_FIXTURE" /dev/null

echo "[5/9] AFL++ parser campaign (${DURATION}s, workers=${PARSER_WORKERS})"
run_single_swarm \
    "parser" \
    "$PARSER_OUT" \
    "$ROOT/fuzz_c/corpus_decrypt" \
    "$ROOT/fuzz_c/decrypt.dict" \
    "$ROOT/fuzz_c/fuzz_nxms_parser" \
    "$PARSER_WORKERS"

echo "[6/9] AFL++ decrypt campaigns (${DURATION}s, workers=${DECRYPT_WORKERS}/${DECRYPT_MUT_WORKERS})"
run_fixture_stdin_swarm \
    "decrypt" \
    "$DECRYPT_OUT" \
    "$ROOT/fuzz_c/corpus_decrypt_valid" \
    "$ROOT/fuzz_c/decrypt.dict" \
    "$ROOT/fuzz_c/fuzz_nxms_decrypt" \
    "$DECRYPT_WORKERS" \
    "$FIXTURE"
run_mut_swarm \
    "decrypt_mut" \
    "$DECRYPT_MUT_OUT" \
    "$ROOT/fuzz_c/corpus_decrypt_mut" \
    "$ROOT/fuzz_c/decrypt_mut.dict" \
    "$ROOT/fuzz_c/fuzz_nxms_decrypt_mut" \
    "$DECRYPT_MUT_WORKERS" \
    "$FIXTURE" \
    "$BASE_PACKET"

echo "[7/9] AFL++ kem campaigns (${DURATION}s, workers=${KEM_WORKERS}/${KEM_MUT_WORKERS})"
run_fixture_stdin_swarm \
    "kem" \
    "$KEM_OUT" \
    "$ROOT/fuzz_c/corpus_kem_valid" \
    "$ROOT/fuzz_c/kem.dict" \
    "$ROOT/fuzz_c/fuzz_pqc_kem_decaps" \
    "$KEM_WORKERS" \
    "$FIXTURE"
run_mut_swarm \
    "kem_mut" \
    "$KEM_MUT_OUT" \
    "$ROOT/fuzz_c/corpus_kem_mut" \
    "$ROOT/fuzz_c/kem_mut.dict" \
    "$ROOT/fuzz_c/fuzz_pqc_kem_decaps_mut" \
    "$KEM_MUT_WORKERS" \
    "$FIXTURE" \
    "$BASE_KEM"

echo "[7b/9] AFL++ falcon campaign (${DURATION}s, workers=${FALCON_MUT_WORKERS})"
run_fixture_swarm \
    "falcon_mut" \
    "$FALCON_MUT_OUT" \
    "$ROOT/fuzz_c/corpus_falcon_mut" \
    "$ROOT/fuzz_c/falcon_mut.dict" \
    "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_verify_mut" \
    "$FALCON_MUT_WORKERS" \
    "$FALCON_FIXTURE"

echo "[7c/9] AFL++ falcon sign-seed campaign (${DURATION}s, workers=${FALCON_SEED_WORKERS})"
run_fixture_swarm \
    "falcon_seed" \
    "$FALCON_SEED_OUT" \
    "$ROOT/fuzz_c/corpus_falcon_seed" \
    "$ROOT/fuzz_c/falcon_seed.dict" \
    "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_seed" \
    "$FALCON_SEED_WORKERS" \
    "$FALCON_FIXTURE"

echo "[8/9] sanitizer replay of campaign queues"
(
    cd "$ROOT"
    PARSER_QUEUE="$PARSER_OUT/parser_main/queue" \
    DECRYPT_QUEUE="$DECRYPT_OUT/decrypt_main/queue" \
    DECRYPT_MUT_QUEUE="$DECRYPT_MUT_OUT/decrypt_mut_main/queue" \
    KEM_QUEUE="$KEM_OUT/kem_main/queue" \
    KEM_MUT_QUEUE="$KEM_MUT_OUT/kem_mut_main/queue" \
    FALCON_MUT_QUEUE="$FALCON_MUT_OUT/falcon_mut_main/queue" \
    FALCON_SEED_QUEUE="$FALCON_SEED_OUT/falcon_seed_main/queue" \
    sh fuzz_c/run_nxms_sanitizer_suite.sh
)

echo "[9/9] summaries"
echo "[summary] parser"
if command -v tput >/dev/null 2>&1; then
    afl-whatsup -s "$PARSER_OUT" || true
else
    afl-whatsup -s "$PARSER_OUT" 2>/dev/null || true
fi
echo "[summary] decrypt"
if command -v tput >/dev/null 2>&1; then
    afl-whatsup -s "$DECRYPT_OUT" || true
else
    afl-whatsup -s "$DECRYPT_OUT" 2>/dev/null || true
fi
echo "[summary] decrypt_mut"
if command -v tput >/dev/null 2>&1; then
    afl-whatsup -s "$DECRYPT_MUT_OUT" || true
else
    afl-whatsup -s "$DECRYPT_MUT_OUT" 2>/dev/null || true
fi
echo "[summary] kem"
if command -v tput >/dev/null 2>&1; then
    afl-whatsup -s "$KEM_OUT" || true
else
    afl-whatsup -s "$KEM_OUT" 2>/dev/null || true
fi
echo "[summary] kem_mut"
if command -v tput >/dev/null 2>&1; then
    afl-whatsup -s "$KEM_MUT_OUT" || true
else
    afl-whatsup -s "$KEM_MUT_OUT" 2>/dev/null || true
fi
echo "[summary] falcon_mut"
if command -v tput >/dev/null 2>&1; then
    afl-whatsup -s "$FALCON_MUT_OUT" || true
else
    afl-whatsup -s "$FALCON_MUT_OUT" 2>/dev/null || true
fi
echo "[summary] falcon_seed"
if command -v tput >/dev/null 2>&1; then
    afl-whatsup -s "$FALCON_SEED_OUT" || true
else
    afl-whatsup -s "$FALCON_SEED_OUT" 2>/dev/null || true
fi

echo "nxms deep fuzz campaign ok"
