#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
REPO_ROOT="$(CDPATH= cd -- "$ROOT/.." && pwd)"
ROUND3_EXTRA="${ROUND3_EXTRA:-$REPO_ROOT/falcon-round3/Extra/c}"
VENDOR_DIR="${VENDOR_DIR:-$ROOT/native/vendor/falcon}"
OUT_FILE="${1:-$ROOT/fuzz_c/coverage/falcon_round3_sync.txt}"
CALLER_PWD="$PWD"

case "$OUT_FILE" in
  /*) ;;
  *) OUT_FILE="$CALLER_PWD/$OUT_FILE" ;;
esac

mkdir -p "$(dirname "$OUT_FILE")"

FILES='
README.txt
Makefile
codec.c
common.c
config.h
falcon.c
falcon.h
fft.c
fpr.c
fpr.h
inner.h
keygen.c
rng.c
shake.c
sign.c
speed.c
test_falcon.c
vrfy.c
'

: >"$OUT_FILE"
printf 'round3_extra=%s\n' "$ROUND3_EXTRA" >>"$OUT_FILE"
printf 'vendor_dir=%s\n' "$VENDOR_DIR" >>"$OUT_FILE"

for rel in $FILES; do
  round3_file="$ROUND3_EXTRA/$rel"
  vendor_file="$VENDOR_DIR/$rel"

  if [ ! -f "$round3_file" ]; then
    printf 'missing_round3=%s\n' "$round3_file" >&2
    exit 1
  fi
  if [ ! -f "$vendor_file" ]; then
    printf 'missing_vendor=%s\n' "$vendor_file" >&2
    exit 1
  fi

  cmp -s "$round3_file" "$vendor_file"
  printf '%s=%s\n' "$rel" "$(sha256sum "$vendor_file" | awk '{print $1}')" >>"$OUT_FILE"
done

printf 'status=OK\n' >>"$OUT_FILE"
printf '%s\n' "$OUT_FILE"
