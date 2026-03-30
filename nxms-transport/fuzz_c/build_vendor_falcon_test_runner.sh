#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
OUT="$ROOT/fuzz_c/fuzz_vendor_falcon_test"
CC_BIN="${CC:-cc}"
CFLAGS_EXTRA="${CFLAGS:-}"
LDFLAGS_EXTRA="${LDFLAGS:-}"

$CC_BIN -std=c11 -O2 -g $CFLAGS_EXTRA \
  -DFALCON_FPEMU=1 \
  -DFALCON_FPNATIVE=0 \
  -DFALCON_UNALIGNED=0 \
  -I"$ROOT/native/vendor/falcon" \
  "$ROOT/native/vendor/falcon/test_falcon.c" \
  "$ROOT/native/vendor/falcon/codec.c" \
  "$ROOT/native/vendor/falcon/common.c" \
  "$ROOT/native/vendor/falcon/falcon.c" \
  "$ROOT/native/vendor/falcon/fft.c" \
  "$ROOT/native/vendor/falcon/fpr.c" \
  "$ROOT/native/vendor/falcon/keygen.c" \
  "$ROOT/native/vendor/falcon/rng.c" \
  "$ROOT/native/vendor/falcon/shake.c" \
  "$ROOT/native/vendor/falcon/sign.c" \
  "$ROOT/native/vendor/falcon/vrfy.c" \
  $LDFLAGS_EXTRA \
  -lm \
  -o "$OUT"

printf '%s\n' "$OUT"
