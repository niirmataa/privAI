#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
REPO_ROOT="$(CDPATH= cd -- "$ROOT/.." && pwd)"
ROUND3_ROOT="${ROUND3_ROOT:-$REPO_ROOT/falcon-round3}"
OUT_DIR="${1:-$ROOT/fuzz_c/coverage/falcon_round3_reference}"
CALLER_PWD="$PWD"
STAGE_DIR="$OUT_DIR/stage"
SUMMARY_FILE="$OUT_DIR/falcon_round3_reference_summary.txt"
SYNC_SUMMARY="$OUT_DIR/falcon_round3_sync.txt"

require_file() {
  if [ ! -f "$1" ]; then
    printf 'missing required Falcon Round 3 file: %s\n' "$1" >&2
    exit 1
  fi
}

run_round3_kat() {
  family_dir="$1"
  impl_dir="$2"
  runner_name="$3"
  package_prefix="$4"

  BUILD_DIR="$STAGE_DIR/Reference_Implementation/$family_dir/$impl_dir"
  MAKE_LOG="$OUT_DIR/${impl_dir}_make.log"
  RUN_LOG="$OUT_DIR/${impl_dir}_run.log"
  GENERATED_REQ="$BUILD_DIR/PQCsignKAT_${impl_dir}.req"
  GENERATED_RSP="$BUILD_DIR/PQCsignKAT_${impl_dir}.rsp"
  PACKAGED_REQ="$ROUND3_ROOT/KAT/${package_prefix}-KAT.req"
  PACKAGED_RSP="$ROUND3_ROOT/KAT/${package_prefix}-KAT.rsp"
  GENERATED_REQ_COPY="$OUT_DIR/${package_prefix}-KAT.generated.req"
  GENERATED_RSP_COPY="$OUT_DIR/${package_prefix}-KAT.generated.rsp"

  require_file "$PACKAGED_REQ"
  require_file "$PACKAGED_RSP"

  make -C "$BUILD_DIR" clean >"$MAKE_LOG" 2>&1 || true
  make -C "$BUILD_DIR" >>"$MAKE_LOG" 2>&1
  (
    cd "$BUILD_DIR"
    rm -f "$GENERATED_REQ" "$GENERATED_RSP"
    "./build/$runner_name" >"$RUN_LOG" 2>&1
  )

  cmp -s "$GENERATED_REQ" "$PACKAGED_REQ"
  cmp -s "$GENERATED_RSP" "$PACKAGED_RSP"

  cp "$GENERATED_REQ" "$GENERATED_REQ_COPY"
  cp "$GENERATED_RSP" "$GENERATED_RSP_COPY"

  printf '%s_req_sha256=%s\n' "$package_prefix" "$(sha256sum "$GENERATED_REQ" | awk '{print $1}')" >>"$SUMMARY_FILE"
  printf '%s_rsp_sha256=%s\n' "$package_prefix" "$(sha256sum "$GENERATED_RSP" | awk '{print $1}')" >>"$SUMMARY_FILE"
  printf '%s_make_log=%s\n' "$package_prefix" "$MAKE_LOG" >>"$SUMMARY_FILE"
  printf '%s_run_log=%s\n' "$package_prefix" "$RUN_LOG" >>"$SUMMARY_FILE"
  printf '%s_req_copy=%s\n' "$package_prefix" "$GENERATED_REQ_COPY" >>"$SUMMARY_FILE"
  printf '%s_rsp_copy=%s\n' "$package_prefix" "$GENERATED_RSP_COPY" >>"$SUMMARY_FILE"
}

require_file "$ROUND3_ROOT/README.txt"
require_file "$ROUND3_ROOT/KAT/generator/PQCgenKAT_sign.c"

case "$OUT_DIR" in
  /*) ;;
  *) OUT_DIR="$CALLER_PWD/$OUT_DIR" ;;
esac

STAGE_DIR="$OUT_DIR/stage"
SUMMARY_FILE="$OUT_DIR/falcon_round3_reference_summary.txt"
SYNC_SUMMARY="$OUT_DIR/falcon_round3_sync.txt"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR" "$STAGE_DIR/KAT"

cp -R "$ROUND3_ROOT/Reference_Implementation" "$STAGE_DIR/"
cp -R "$ROUND3_ROOT/KAT/generator" "$STAGE_DIR/KAT/"

: >"$SUMMARY_FILE"
printf 'round3_root=%s\n' "$ROUND3_ROOT" >>"$SUMMARY_FILE"
sh "$ROOT/fuzz_c/verify_falcon_round3_sync.sh" "$SYNC_SUMMARY" >/dev/null
printf 'round3_sync_summary=%s\n' "$SYNC_SUMMARY" >>"$SUMMARY_FILE"

run_round3_kat falcon512 falcon512int kat512int falcon512
run_round3_kat falcon1024 falcon1024int kat1024int falcon1024

printf 'falcon_round3_reference_artifacts=%s\n' "$OUT_DIR"
