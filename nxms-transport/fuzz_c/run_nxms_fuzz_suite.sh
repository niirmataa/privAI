#!/bin/sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
DURATION="${1:-10}"

echo "[1/6] crypto corpus replay outside libFuzzer"
(cd "$ROOT" && cargo test --features crypto --test crypto_corpus_runner)

echo "[2/6] refresh fixture, split corpora and dictionaries"
(cd "$ROOT" && cargo run --example emit_c_fuzz_fixture --features crypto -- fuzz_c/corpus >/dev/null)

echo "[3/6] build C harnesses"
(cd "$ROOT" && sh fuzz_c/build_nxms_packet_parser_harness.sh >/dev/null)
(cd "$ROOT" && sh fuzz_c/build_nxms_ms_verify_decrypt_harness.sh >/dev/null)
(cd "$ROOT" && sh fuzz_c/build_nxms_ms_verify_decrypt_mut_harness.sh >/dev/null)
(cd "$ROOT" && sh fuzz_c/build_pqc_kem_decaps_harness.sh >/dev/null)
(cd "$ROOT" && sh fuzz_c/build_pqc_kem_decaps_mut_harness.sh >/dev/null)
(cd "$ROOT" && sh fuzz_c/build_pqc_falcon_sign_verify_mut_harness.sh >/dev/null)
(cd "$ROOT" && sh fuzz_c/build_pqc_falcon_wrapper_mut_harness.sh >/dev/null)
(cd "$ROOT" && sh fuzz_c/build_pqc_falcon_sign_seed_harness.sh >/dev/null)

echo "[4/6] self-tests"
"$ROOT/fuzz_c/fuzz_nxms_parser" --self-test "$ROOT/fuzz_c/corpus/seed_valid.bin"
"$ROOT/fuzz_c/fuzz_nxms_decrypt" --self-test "$ROOT/fuzz_c/corpus/fixture.bin" "$ROOT/fuzz_c/corpus/seed_valid.bin"
"$ROOT/fuzz_c/fuzz_nxms_decrypt_mut" --self-test "$ROOT/fuzz_c/corpus/fixture.bin" "$ROOT/fuzz_c/corpus/seed_valid.bin" /dev/null
"$ROOT/fuzz_c/fuzz_pqc_kem_decaps" --self-test "$ROOT/fuzz_c/corpus/fixture.bin" "$ROOT/fuzz_c/corpus/seed_valid.bin"
"$ROOT/fuzz_c/fuzz_pqc_kem_decaps_mut" --self-test "$ROOT/fuzz_c/corpus/fixture.bin" "$ROOT/fuzz_c/corpus_kem/kem_ct_valid.bin" /dev/null
"$ROOT/fuzz_c/fuzz_pqc_falcon_sign_verify_mut" --self-test "$ROOT/fuzz_c/corpus/falcon_fixture.bin" /dev/null
"$ROOT/fuzz_c/fuzz_pqc_falcon_wrapper_mut" --self-test "$ROOT/fuzz_c/corpus/falcon_fixture.bin" /dev/null
"$ROOT/fuzz_c/fuzz_pqc_falcon_sign_seed" --self-test "$ROOT/fuzz_c/corpus/falcon_fixture.bin" /dev/null

if command -v afl-fuzz >/dev/null 2>&1 && command -v afl-clang-fast >/dev/null 2>&1; then
    echo "[5/6] rebuild AFL++ harnesses"
    (cd "$ROOT" && rm -f fuzz_c/fuzz_nxms_parser fuzz_c/fuzz_nxms_decrypt fuzz_c/fuzz_nxms_decrypt_mut fuzz_c/fuzz_pqc_kem_decaps fuzz_c/fuzz_pqc_kem_decaps_mut fuzz_c/fuzz_pqc_falcon_sign_verify_mut fuzz_c/fuzz_pqc_falcon_wrapper_mut fuzz_c/fuzz_pqc_falcon_sign_seed)
    (cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_nxms_packet_parser_harness.sh >/dev/null)
    (cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_nxms_ms_verify_decrypt_harness.sh >/dev/null)
    (cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_nxms_ms_verify_decrypt_mut_harness.sh >/dev/null)
    (cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_pqc_kem_decaps_harness.sh >/dev/null)
    (cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_pqc_kem_decaps_mut_harness.sh >/dev/null)
    (cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_pqc_falcon_sign_verify_mut_harness.sh >/dev/null)
    (cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_pqc_falcon_wrapper_mut_harness.sh >/dev/null)
    (cd "$ROOT" && CC=afl-clang-fast sh fuzz_c/build_pqc_falcon_sign_seed_harness.sh >/dev/null)

    echo "[6/6] AFL++ short run (${DURATION}s each)"
    (cd "$ROOT" && AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 afl-fuzz -V "$DURATION" -x fuzz_c/decrypt.dict -i fuzz_c/corpus_decrypt -o fuzz_c/findings_parser -- ./fuzz_c/fuzz_nxms_parser @@)
    (cd "$ROOT" && AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 afl-fuzz -V "$DURATION" -x fuzz_c/decrypt.dict -i fuzz_c/corpus_decrypt_valid -o fuzz_c/findings -- ./fuzz_c/fuzz_nxms_decrypt fuzz_c/corpus/fixture.bin @@)
    (cd "$ROOT" && AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 afl-fuzz -V "$DURATION" -x fuzz_c/decrypt_mut.dict -i fuzz_c/corpus_decrypt_mut -o fuzz_c/findings_decrypt_mut -- ./fuzz_c/fuzz_nxms_decrypt_mut fuzz_c/corpus/fixture.bin fuzz_c/corpus/seed_valid.bin @@)
    (cd "$ROOT" && AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 afl-fuzz -V "$DURATION" -x fuzz_c/kem.dict -i fuzz_c/corpus_kem_valid -o fuzz_c/findings_kem -- ./fuzz_c/fuzz_pqc_kem_decaps fuzz_c/corpus/fixture.bin @@)
    (cd "$ROOT" && AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 afl-fuzz -V "$DURATION" -x fuzz_c/kem_mut.dict -i fuzz_c/corpus_kem_mut -o fuzz_c/findings_kem_mut -- ./fuzz_c/fuzz_pqc_kem_decaps_mut fuzz_c/corpus/fixture.bin fuzz_c/corpus_kem/kem_ct_valid.bin @@)
    (cd "$ROOT" && AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 afl-fuzz -V "$DURATION" -x fuzz_c/falcon_mut.dict -i fuzz_c/corpus_falcon_mut -o fuzz_c/findings_falcon_mut -- ./fuzz_c/fuzz_pqc_falcon_sign_verify_mut fuzz_c/corpus/falcon_fixture.bin @@)
    (cd "$ROOT" && AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 afl-fuzz -V "$DURATION" -x fuzz_c/falcon_seed.dict -i fuzz_c/corpus_falcon_seed -o fuzz_c/findings_falcon_seed -- ./fuzz_c/fuzz_pqc_falcon_sign_seed fuzz_c/corpus/falcon_fixture.bin @@)
else
    echo "[5/6] AFL++ unavailable, skipped"
    echo "[6/6] AFL++ unavailable, skipped"
fi

echo "nxms fuzz suite ok"
