#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
COV_DIR="$ROOT/fuzz_c/coverage"
REPORT_DIR="$COV_DIR/reports"
EVIDENCE_DIR="$COV_DIR/audit_latest"
CAMPAIGN_ROOT="${CAMPAIGN_ROOT:-$ROOT/fuzz_c/campaign_deep}"

PARSER_QUEUE="$CAMPAIGN_ROOT/parser/parser_main/queue"
DECRYPT_QUEUE="$CAMPAIGN_ROOT/decrypt/decrypt_main/queue"
DECRYPT_MUT_QUEUE="$CAMPAIGN_ROOT/decrypt_mut/decrypt_mut_main/queue"
KEM_QUEUE="$CAMPAIGN_ROOT/kem/kem_main/queue"
KEM_MUT_QUEUE="$CAMPAIGN_ROOT/kem_mut/kem_mut_main/queue"
FALCON_MUT_QUEUE="$CAMPAIGN_ROOT/falcon_mut/falcon_mut_main/queue"
FALCON_SEED_QUEUE="$CAMPAIGN_ROOT/falcon_seed/falcon_seed_main/queue"

count_inputs() {
    TARGET="$1"

    if [ ! -d "$TARGET" ]; then
        echo "0"
        return 0
    fi

    find "$TARGET" -type f ! -name README.txt | wc -l | tr -d ' '
}

first_line() {
    "$@" 2>&1 | sed -n '1p'
}

if pgrep -f "run_nxms_deep_fuzz_campaign.sh" >/dev/null 2>&1; then
    echo "deep fuzz campaign still running; wait for completion before collecting audit evidence" >&2
    exit 1
fi

rm -rf "$EVIDENCE_DIR"
mkdir -p "$EVIDENCE_DIR" "$EVIDENCE_DIR/reports"

{
    echo "date_utc=$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    echo "git_head=$(git -C "$ROOT" rev-parse HEAD)"
    echo "rustc=$(rustc --version)"
    echo "cargo=$(cargo --version)"
    echo "clang=$(clang --version | sed -n '1p')"
    echo "llvm_cov=$(llvm-cov --version | sed -n '1p')"
    echo "llvm_profdata=$(llvm-profdata --version | sed -n '1p')"
    echo "afl=$(first_line afl-fuzz -h)"
    echo "kernel=$(uname -a)"
} > "$EVIDENCE_DIR/toolchain_and_rev.txt"

{
    echo "[parser]"
    echo "queue_files=$(count_inputs "$PARSER_QUEUE")"
    echo "crashes=$(count_inputs "$CAMPAIGN_ROOT/parser/parser_main/crashes")"
    echo "hangs=$(count_inputs "$CAMPAIGN_ROOT/parser/parser_main/hangs")"
    echo
    echo "[decrypt]"
    echo "queue_files=$(count_inputs "$DECRYPT_QUEUE")"
    echo "crashes=$(count_inputs "$CAMPAIGN_ROOT/decrypt/decrypt_main/crashes")"
    echo "hangs=$(count_inputs "$CAMPAIGN_ROOT/decrypt/decrypt_main/hangs")"
    echo
    echo "[decrypt_mut]"
    echo "queue_files=$(count_inputs "$DECRYPT_MUT_QUEUE")"
    echo "crashes=$(count_inputs "$CAMPAIGN_ROOT/decrypt_mut/decrypt_mut_main/crashes")"
    echo "hangs=$(count_inputs "$CAMPAIGN_ROOT/decrypt_mut/decrypt_mut_main/hangs")"
    echo
    echo "[kem]"
    echo "queue_files=$(count_inputs "$KEM_QUEUE")"
    echo "crashes=$(count_inputs "$CAMPAIGN_ROOT/kem/kem_main/crashes")"
    echo "hangs=$(count_inputs "$CAMPAIGN_ROOT/kem/kem_main/hangs")"
    echo
    echo "[kem_mut]"
    echo "queue_files=$(count_inputs "$KEM_MUT_QUEUE")"
    echo "crashes=$(count_inputs "$CAMPAIGN_ROOT/kem_mut/kem_mut_main/crashes")"
    echo "hangs=$(count_inputs "$CAMPAIGN_ROOT/kem_mut/kem_mut_main/hangs")"
    echo
    echo "[falcon_mut]"
    echo "queue_files=$(count_inputs "$FALCON_MUT_QUEUE")"
    echo "crashes=$(count_inputs "$CAMPAIGN_ROOT/falcon_mut/falcon_mut_main/crashes")"
    echo "hangs=$(count_inputs "$CAMPAIGN_ROOT/falcon_mut/falcon_mut_main/hangs")"
    echo
    echo "[falcon_seed]"
    echo "queue_files=$(count_inputs "$FALCON_SEED_QUEUE")"
    echo "crashes=$(count_inputs "$CAMPAIGN_ROOT/falcon_seed/falcon_seed_main/crashes")"
    echo "hangs=$(count_inputs "$CAMPAIGN_ROOT/falcon_seed/falcon_seed_main/hangs")"
} > "$EVIDENCE_DIR/campaign_inventory.txt"

(
    cd "$ROOT"
    PARSER_QUEUE="$PARSER_QUEUE" \
    DECRYPT_QUEUE="$DECRYPT_QUEUE" \
    DECRYPT_MUT_QUEUE="$DECRYPT_MUT_QUEUE" \
    KEM_QUEUE="$KEM_QUEUE" \
    KEM_MUT_QUEUE="$KEM_MUT_QUEUE" \
    FALCON_MUT_QUEUE="$FALCON_MUT_QUEUE" \
    FALCON_SEED_QUEUE="$FALCON_SEED_QUEUE" \
    sh fuzz_c/run_nxms_sanitizer_suite.sh
) | tee "$EVIDENCE_DIR/sanitizer_suite.log"

(
    cd "$ROOT"
    PARSER_QUEUE="$PARSER_QUEUE" \
    DECRYPT_QUEUE="$DECRYPT_QUEUE" \
    DECRYPT_MUT_QUEUE="$DECRYPT_MUT_QUEUE" \
    KEM_QUEUE="$KEM_QUEUE" \
    KEM_MUT_QUEUE="$KEM_MUT_QUEUE" \
    FALCON_MUT_QUEUE="$FALCON_MUT_QUEUE" \
    FALCON_SEED_QUEUE="$FALCON_SEED_QUEUE" \
    sh fuzz_c/run_nxms_coverage_report.sh
) | tee "$EVIDENCE_DIR/coverage_report.log"

cp -f "$REPORT_DIR"/* "$EVIDENCE_DIR/reports/" 2>/dev/null || true

printf '%s\n' "nxms audit evidence ok" > "$EVIDENCE_DIR/status.txt"
echo "audit evidence written to $EVIDENCE_DIR"
