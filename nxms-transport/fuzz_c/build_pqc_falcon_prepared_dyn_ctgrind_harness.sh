#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
OUT="$ROOT/fuzz_c/fuzz_pqc_falcon_prepared_dyn_ctgrind"
FALCON_SRC="$(sh "$ROOT/fuzz_c/prepare_falcon_ctgrind_overlay.sh")"
FALCON_WORK="$(dirname "$FALCON_SRC")"
CC_BIN="${CC:-cc}"
CFLAGS_EXTRA="${CFLAGS:-}"
LDFLAGS_EXTRA="${LDFLAGS:-}"

$CC_BIN -std=c11 -O2 -g $CFLAGS_EXTRA \
  -DFF_FALCON_LOGN=10 \
  -DNXMS_CTGRIND_AUDIT=1 \
  -DFALCON_FPEMU=1 \
  -DFALCON_FPNATIVE=0 \
  -DFALCON_UNALIGNED=0 \
  -I"$ROOT/native" \
  -I"$FALCON_SRC" \
  -I"$ROOT/native/nexum_cli_src" \
  -I/usr/include/valgrind \
  "$ROOT/fuzz_c/pqc_falcon_prepared_dyn_ctgrind_harness.c" \
  "$ROOT/native/nexum_cli_src/pqc_falcon.c" \
  "$FALCON_SRC/codec.c" \
  "$FALCON_SRC/common.c" \
  "$FALCON_SRC/falcon.c" \
  "$FALCON_SRC/fft.c" \
  "$FALCON_SRC/fpr.c" \
  "$FALCON_SRC/keygen.c" \
  "$FALCON_SRC/rng.c" \
  "$FALCON_SRC/shake.c" \
  "$FALCON_SRC/sign.c" \
  "$FALCON_SRC/vrfy.c" \
  $LDFLAGS_EXTRA \
  -lm \
  -o "$OUT"

rm -rf "$FALCON_WORK"

printf '%s\n' "$OUT"
