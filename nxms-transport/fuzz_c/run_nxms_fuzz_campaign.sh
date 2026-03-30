#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
DURATION="${1:-60}"
DECRYPT_WORKERS="${2:-4}"
KEM_WORKERS="${3:-3}"
CAMPAIGN_ROOT="$ROOT/fuzz_c/campaign"
DECRYPT_OUT="$CAMPAIGN_ROOT/decrypt"
KEM_OUT="$CAMPAIGN_ROOT/kem"
FIXTURE="$ROOT/fuzz_c/corpus/fixture.bin"

if ! command -v afl-fuzz >/dev/null 2>&1 || ! command -v afl-clang-fast >/dev/null 2>&1; then
    echo "AFL++ not available"
    exit 1
fi

if ! command -v afl-whatsup >/dev/null 2>&1; then
    echo "afl-whatsup not available"
    exit 1
fi

run_swarm() {
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
                -- "$HARNESS" "$FIXTURE" @@
        ) &
        PID=$!
        SESSION_PIDS="${SESSION_PIDS} ${PID}"
        INDEX=$((INDEX + 1))
    done

    for PID in $SESSION_PIDS; do
        wait "$PID"
    done
}

echo "[1/7] refresh fixture and corpora"
(cd "$ROOT" && cargo run --example emit_c_fuzz_fixture --features crypto -- fuzz_c/corpus >/dev/null)

echo "[2/7] baseline replay outside libFuzzer"
(cd "$ROOT" && cargo test --features crypto --test crypto_corpus_runner >/dev/null)

echo "[3/7] rebuild AFL++ harnesses"
(cd "$ROOT" && rm -f fuzz_c/fuzz_nxms_decrypt fuzz_c/fuzz_pqc_kem_decaps)
(cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_nxms_ms_verify_decrypt_harness.sh >/dev/null)
(cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_pqc_kem_decaps_harness.sh >/dev/null)

echo "[4/7] self-tests"
"$ROOT/fuzz_c/fuzz_nxms_decrypt" --self-test "$FIXTURE" "$ROOT/fuzz_c/corpus/seed_valid.bin"
"$ROOT/fuzz_c/fuzz_pqc_kem_decaps" --self-test "$FIXTURE" "$ROOT/fuzz_c/corpus/seed_valid.bin"

echo "[5/7] AFL++ decrypt campaign (${DURATION}s, workers=${DECRYPT_WORKERS})"
run_swarm \
    "decrypt" \
    "$DECRYPT_OUT" \
    "$ROOT/fuzz_c/corpus_decrypt" \
    "$ROOT/fuzz_c/decrypt.dict" \
    "$ROOT/fuzz_c/fuzz_nxms_decrypt" \
    "$DECRYPT_WORKERS"

echo "[6/7] AFL++ kem campaign (${DURATION}s, workers=${KEM_WORKERS})"
run_swarm \
    "kem" \
    "$KEM_OUT" \
    "$ROOT/fuzz_c/corpus_kem" \
    "$ROOT/fuzz_c/kem.dict" \
    "$ROOT/fuzz_c/fuzz_pqc_kem_decaps" \
    "$KEM_WORKERS"

echo "[7/7] sanitizer replay of campaign queues"
DECRYPT_QUEUE="$DECRYPT_OUT/decrypt_main/queue"
KEM_QUEUE="$KEM_OUT/kem_main/queue"

(
    cd "$ROOT"
    DECRYPT_QUEUE="$DECRYPT_QUEUE" KEM_QUEUE="$KEM_QUEUE" \
    sh fuzz_c/run_nxms_sanitizer_suite.sh
)

echo "[summary] decrypt"
if command -v tput >/dev/null 2>&1; then
    afl-whatsup -s "$DECRYPT_OUT" || true
else
    afl-whatsup -s "$DECRYPT_OUT" 2>/dev/null || true
fi

echo "[summary] kem"
if command -v tput >/dev/null 2>&1; then
    afl-whatsup -s "$KEM_OUT" || true
else
    afl-whatsup -s "$KEM_OUT" 2>/dev/null || true
fi

echo "nxms fuzz campaign ok"
