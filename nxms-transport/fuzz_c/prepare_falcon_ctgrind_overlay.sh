#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
WORK_DIR="$ROOT/fuzz_c/.build/falcon_ctgrind_overlay"
PATCH_FILE="$ROOT/fuzz_c/falcon_ctgrind_overlay.patch"
OVERLAY_DIR="$WORK_DIR/falcon"

rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR"
cp -R "$ROOT/native/vendor/falcon" "$OVERLAY_DIR"
patch -d "$WORK_DIR" -p1 <"$PATCH_FILE" >/dev/null

printf '%s\n' "$OVERLAY_DIR"
