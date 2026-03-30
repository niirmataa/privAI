#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
OUT="$ROOT/fuzz_c/fuzz_pqc_kem_decaps_mut"
CC_BIN="${CC:-cc}"
CFLAGS_EXTRA="${CFLAGS:-}"
LDFLAGS_EXTRA="${LDFLAGS:-}"

$CC_BIN -std=c11 -O1 -g $CFLAGS_EXTRA \
  -I"$ROOT/native" \
  -I"$ROOT/native/vendor/falcon" \
  -I"$ROOT/native/nexum_cli_src" \
  "$ROOT/fuzz_c/pqc_kem_decaps_mut_harness.c" \
  "$ROOT/native/nexum_cli_src/pqc_kem.c" \
  "$ROOT/native/nexum_cli_src/util.c" \
  $LDFLAGS_EXTRA \
  -loqs -lsodium -lcrypto \
  -o "$OUT"

printf '%s\n' "$OUT"
