#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
OUT="$ROOT/fuzz_c/fuzz_nxms_parser"
CC_BIN="${CC:-cc}"
CFLAGS_EXTRA="${CFLAGS:-}"
LDFLAGS_EXTRA="${LDFLAGS:-}"

$CC_BIN -std=c11 -O1 -g $CFLAGS_EXTRA \
  -I"$ROOT/native" \
  "$ROOT/fuzz_c/nxms_packet_parser_harness.c" \
  $LDFLAGS_EXTRA \
  -o "$OUT"

printf '%s\n' "$OUT"
