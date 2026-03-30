#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
COV_DIR="$ROOT/fuzz_c/coverage"
PARSER_PROF_DIR="$COV_DIR/parser"
DECRYPT_MUT_PROF_DIR="$COV_DIR/decrypt_mut"
FALCON_MUT_PROF_DIR="$COV_DIR/falcon_mut"
FALCON_WRAPPER_PROF_DIR="$COV_DIR/falcon_wrapper"
FALCON_SEED_PROF_DIR="$COV_DIR/falcon_seed"
PARSER_PROFDATA="$COV_DIR/parser.profdata"
DECRYPT_MUT_PROFDATA="$COV_DIR/decrypt_mut.profdata"
FALCON_MUT_PROFDATA="$COV_DIR/falcon_mut.profdata"
FALCON_WRAPPER_PROFDATA="$COV_DIR/falcon_wrapper.profdata"
FALCON_SEED_PROFDATA="$COV_DIR/falcon_seed.profdata"
FIXTURE="$ROOT/fuzz_c/corpus/fixture.bin"
FALCON_FIXTURE="$ROOT/fuzz_c/corpus/falcon_fixture.bin"
BASE_PACKET="$ROOT/fuzz_c/corpus/seed_valid.bin"
FALCON_MUT_QUEUE="${FALCON_MUT_QUEUE:-$ROOT/fuzz_c/findings_falcon_mut/default/queue}"
FALCON_SEED_QUEUE="${FALCON_SEED_QUEUE:-$ROOT/fuzz_c/findings_falcon_seed/default/queue}"

mkdir -p "$PARSER_PROF_DIR" "$DECRYPT_MUT_PROF_DIR" "$FALCON_MUT_PROF_DIR" "$FALCON_WRAPPER_PROF_DIR" "$FALCON_SEED_PROF_DIR"
rm -f "$PARSER_PROF_DIR"/*.profraw "$DECRYPT_MUT_PROF_DIR"/*.profraw "$FALCON_MUT_PROF_DIR"/*.profraw "$FALCON_WRAPPER_PROF_DIR"/*.profraw "$FALCON_SEED_PROF_DIR"/*.profraw "$PARSER_PROFDATA" "$DECRYPT_MUT_PROFDATA" "$FALCON_MUT_PROFDATA" "$FALCON_WRAPPER_PROFDATA" "$FALCON_SEED_PROFDATA"

echo "[1/5] refresh fixture and corpora"
(cd "$ROOT" && cargo run --example emit_c_fuzz_fixture --features crypto -- fuzz_c/corpus >/dev/null)

echo "[2/5] build coverage binaries"
(
    cd "$ROOT"
    CC=clang \
    CFLAGS="-fprofile-instr-generate -fcoverage-mapping -O0 -g" \
    LDFLAGS="-fprofile-instr-generate -fcoverage-mapping" \
    sh fuzz_c/build_nxms_packet_parser_harness.sh >/dev/null
)
(
    cd "$ROOT"
    CC=clang \
    CFLAGS="-fprofile-instr-generate -fcoverage-mapping -O0 -g" \
    LDFLAGS="-fprofile-instr-generate -fcoverage-mapping" \
    sh fuzz_c/build_nxms_ms_verify_decrypt_mut_harness.sh >/dev/null
)
(
    cd "$ROOT"
    CC=clang \
    CFLAGS="-fprofile-instr-generate -fcoverage-mapping -O0 -g" \
    LDFLAGS="-fprofile-instr-generate -fcoverage-mapping" \
    sh fuzz_c/build_pqc_falcon_sign_verify_mut_harness.sh >/dev/null
)
(
    cd "$ROOT"
    CC=clang \
    CFLAGS="-fprofile-instr-generate -fcoverage-mapping -O0 -g" \
    LDFLAGS="-fprofile-instr-generate -fcoverage-mapping" \
    sh fuzz_c/build_pqc_falcon_wrapper_mut_harness.sh >/dev/null
)
(
    cd "$ROOT"
    CC=clang \
    CFLAGS="-fprofile-instr-generate -fcoverage-mapping -O0 -g" \
    LDFLAGS="-fprofile-instr-generate -fcoverage-mapping" \
    sh fuzz_c/build_pqc_falcon_sign_seed_harness.sh >/dev/null
)

echo "[3/5] replay parser corpora under source coverage"
for INPUT in "$ROOT"/fuzz_c/corpus_decrypt/* "$ROOT"/fuzz_c/corpus_decrypt_valid/*; do
    if [ ! -f "$INPUT" ]; then
        continue
    fi
    LLVM_PROFILE_FILE="$PARSER_PROF_DIR/%p-%m.profraw" \
        "$ROOT/fuzz_c/fuzz_nxms_parser" "$INPUT" >/dev/null
done

echo "[4/5] replay decrypt_mut corpora under source coverage"
for INPUT in "$ROOT"/fuzz_c/corpus_decrypt_mut/*; do
    if [ ! -f "$INPUT" ]; then
        continue
    fi
    LLVM_PROFILE_FILE="$DECRYPT_MUT_PROF_DIR/%p-%m.profraw" \
        "$ROOT/fuzz_c/fuzz_nxms_decrypt_mut" "$FIXTURE" "$BASE_PACKET" "$INPUT" >/dev/null
done

echo "[5/6] replay falcon_mut corpora under source coverage"
for INPUT in "$ROOT"/fuzz_c/corpus_falcon_mut/*; do
    if [ ! -f "$INPUT" ]; then
        continue
    fi
    LLVM_PROFILE_FILE="$FALCON_MUT_PROF_DIR/%p-%m.profraw" \
        "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_verify_mut" "$FALCON_FIXTURE" "$INPUT" >/dev/null
done
for INPUT in "$FALCON_MUT_QUEUE"/*; do
    if [ ! -f "$INPUT" ]; then
        continue
    fi
    LLVM_PROFILE_FILE="$FALCON_MUT_PROF_DIR/%p-%m.profraw" \
        "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_verify_mut" "$FALCON_FIXTURE" "$INPUT" >/dev/null
done

echo "[5b/6] replay falcon_wrapper corpora under source coverage"
for INPUT in "$ROOT"/fuzz_c/corpus_falcon_mut/*; do
    if [ ! -f "$INPUT" ]; then
        continue
    fi
    LLVM_PROFILE_FILE="$FALCON_WRAPPER_PROF_DIR/%p-%m.profraw" \
        "$ROOT/fuzz_c/fuzz_pqc_falcon_wrapper_mut" "$FALCON_FIXTURE" "$INPUT" >/dev/null
done
for INPUT in "$FALCON_MUT_QUEUE"/*; do
    if [ ! -f "$INPUT" ]; then
        continue
    fi
    LLVM_PROFILE_FILE="$FALCON_WRAPPER_PROF_DIR/%p-%m.profraw" \
        "$ROOT/fuzz_c/fuzz_pqc_falcon_wrapper_mut" "$FALCON_FIXTURE" "$INPUT" >/dev/null
done

echo "[5c/6] replay falcon_sign_seed corpora under source coverage"
for INPUT in "$ROOT"/fuzz_c/corpus_falcon_seed/*; do
    if [ ! -f "$INPUT" ]; then
        continue
    fi
    LLVM_PROFILE_FILE="$FALCON_SEED_PROF_DIR/%p-%m.profraw" \
        "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_seed" "$FALCON_FIXTURE" "$INPUT" >/dev/null
done
for INPUT in "$FALCON_SEED_QUEUE"/*; do
    if [ ! -f "$INPUT" ]; then
        continue
    fi
    LLVM_PROFILE_FILE="$FALCON_SEED_PROF_DIR/%p-%m.profraw" \
        "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_seed" "$FALCON_FIXTURE" "$INPUT" >/dev/null
done

echo "[6/6] merge and report"
llvm-profdata merge -sparse "$PARSER_PROF_DIR"/*.profraw -o "$PARSER_PROFDATA"
llvm-profdata merge -sparse "$DECRYPT_MUT_PROF_DIR"/*.profraw -o "$DECRYPT_MUT_PROFDATA"
llvm-profdata merge -sparse "$FALCON_MUT_PROF_DIR"/*.profraw -o "$FALCON_MUT_PROFDATA"
llvm-profdata merge -sparse "$FALCON_WRAPPER_PROF_DIR"/*.profraw -o "$FALCON_WRAPPER_PROFDATA"
llvm-profdata merge -sparse "$FALCON_SEED_PROF_DIR"/*.profraw -o "$FALCON_SEED_PROFDATA"

echo "[parser report]"
llvm-cov report "$ROOT/fuzz_c/fuzz_nxms_parser" \
    -instr-profile="$PARSER_PROFDATA" \
    "$ROOT/fuzz_c/nxms_packet_parser_harness.c"

echo "[decrypt_mut report]"
llvm-cov report "$ROOT/fuzz_c/fuzz_nxms_decrypt_mut" \
    -instr-profile="$DECRYPT_MUT_PROFDATA" \
    "$ROOT/fuzz_c/nxms_ms_verify_decrypt_mut_harness.c" \
    "$ROOT/native/nxms_ms_transport.c" \
    "$ROOT/native/nexum_cli_src/pqc_kem.c" \
    "$ROOT/native/nexum_cli_src/pqc_falcon.c" \
    "$ROOT/native/nexum_cli_src/util.c"

echo "[falcon_mut report]"
llvm-cov report "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_verify_mut" \
    -instr-profile="$FALCON_MUT_PROFDATA" \
    "$ROOT/fuzz_c/pqc_falcon_sign_verify_mut_harness.c" \
    "$ROOT/native/nexum_cli_src/pqc_falcon.c" \
    "$ROOT/native/vendor/falcon/falcon.c" \
    "$ROOT/native/vendor/falcon/sign.c" \
    "$ROOT/native/vendor/falcon/vrfy.c" \
    "$ROOT/native/vendor/falcon/fft.c" \
    "$ROOT/native/vendor/falcon/fpr.c" \
    "$ROOT/native/vendor/falcon/common.c" \
    "$ROOT/native/vendor/falcon/rng.c" \
    "$ROOT/native/vendor/falcon/shake.c" \
    "$ROOT/native/vendor/falcon/codec.c"

echo "[falcon_wrapper report]"
llvm-cov report "$ROOT/fuzz_c/fuzz_pqc_falcon_wrapper_mut" \
    -instr-profile="$FALCON_WRAPPER_PROFDATA" \
    "$ROOT/fuzz_c/pqc_falcon_wrapper_mut_harness.c" \
    "$ROOT/native/nexum_cli_src/pqc_falcon.c" \
    "$ROOT/native/vendor/falcon/falcon.c" \
    "$ROOT/native/vendor/falcon/sign.c" \
    "$ROOT/native/vendor/falcon/vrfy.c" \
    "$ROOT/native/vendor/falcon/fft.c" \
    "$ROOT/native/vendor/falcon/fpr.c" \
    "$ROOT/native/vendor/falcon/common.c" \
    "$ROOT/native/vendor/falcon/rng.c" \
    "$ROOT/native/vendor/falcon/shake.c" \
    "$ROOT/native/vendor/falcon/codec.c"

echo "[falcon_sign_seed report]"
llvm-cov report "$ROOT/fuzz_c/fuzz_pqc_falcon_sign_seed" \
    -instr-profile="$FALCON_SEED_PROFDATA" \
    "$ROOT/fuzz_c/pqc_falcon_sign_seed_harness.c" \
    "$ROOT/native/vendor/falcon/falcon.c" \
    "$ROOT/native/vendor/falcon/sign.c" \
    "$ROOT/native/vendor/falcon/vrfy.c" \
    "$ROOT/native/vendor/falcon/fft.c" \
    "$ROOT/native/vendor/falcon/fpr.c" \
    "$ROOT/native/vendor/falcon/common.c" \
    "$ROOT/native/vendor/falcon/rng.c" \
    "$ROOT/native/vendor/falcon/shake.c" \
    "$ROOT/native/vendor/falcon/codec.c"
