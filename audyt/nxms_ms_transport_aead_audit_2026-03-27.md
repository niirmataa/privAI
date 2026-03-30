# Audyt bezpieczenstwa transportu NXMS dla `nxms_ms_transport.c`

Data audytu: 2026-03-27

## 0. Zakres audytu i granice odpowiedzialnosci

Status: `WYKONANE`

In scope:
- plik `nxms-transport/native/nxms_ms_transport.c`
- sciezka wykonania `nxms_ms_encrypt_packet()` oraz `nxms_ms_verify_decrypt()`
- wrapper Rust `nxms-transport/src/crypto.rs` tylko w zakresie marshalingu do tej sciezki C
- wire-format pakietu `kem_ct | nonce | ciphertext | tag | sig`
- zaleznosci kryptograficzne tej sciezki: `FrodoKEM-640-SHAKE`, vendor `Falcon 2020-09-30` z uzyciem `FALCON_SIG_CT`, SHAKE256-based KDF/stream/tag
- build artefaktu z wymuszonym CT backendem Falcona:
  - `FALCON_FPEMU=1`
  - `FALCON_FPNATIVE=0`

Out of scope:
- helpery `xchacha20poly1305_*` w `nxms-transport/src/crypto.rs`
- inne crate w repo
- relay / mailbox bez deszyfrowania
- business logic ponad transportem
- storage replay protection poza polem `seq`
- formalny dowod ProVerif / Tamarin / EasyCrypt

Audited path:
- `nxms-transport/native/nxms_ms_transport.c`

Out-of-scope path(s):
- `nxms-transport/src/crypto.rs` funkcje `xchacha20poly1305_encrypt`, `xchacha20poly1305_decrypt`

Wire version:
- aktualny lokalny wire format `NXMS/1` uzywany przez `SealedPacket`

Build artifact:
- `nxms-transport/build.rs` kompiluje `native/nxms_ms_transport.c`
- aktualny build wymusza:
  - `FALCON_FPEMU=1`
  - `FALCON_FPNATIVE=0`

Commit / snapshot:
- lokalny workspace snapshot z 2026-03-27
- w tym srodowisku katalog nie jest checkoutem git z dostepnym `.git`, wiec nie da sie zapisac konkretnego SHA

## Status koncowy

Werdykt koncowy: `PARTIAL / TOOLING FINDING IN POINT 5`

Najwazniejsze findingi:

1. `[P1]` Coverage-guided fuzzing ujawnil reprodukowalny crash w sciezce `cargo-fuzz/libFuzzer -> Rust -> C`.
   - `cargo +nightly-x86_64-unknown-linux-musl fuzz run --sanitizer none decrypt_fuzz ...`
   - `valgrind` na binarce fuzz target wskazuje invalid jump z dojscia do `ff_kem_decaps()` wywolanego z `nxms_ms_verify_decrypt()` w `nxms-transport/native/nxms_ms_transport.c:746`
   - crash reprodukuje sie na `fuzz/corpus/decrypt_fuzz/seed_valid.json`
   - crash pozostaje nawet po poprawieniu harnessu `decrypt_fuzz.rs`, tak aby ladowal poprawny `fixture.bin` i uzywal zgodnego kontekstu `alice/bob/tx_sign_req/[0x5A;16]/41`
   - po dodaniu zwyklego testu regresyjnego `tests/crypto_fuzz_regression.rs` ten sam `seed_valid.json` **przechodzi poprawnie** zarowno w `cargo test`, jak i `cargo test --release`
   - to przesuwa interpretacje findingu z "bug w samym `nxms_ms_verify_decrypt()`" na "problem w konfiguracji / runtime sciezki `cargo-fuzz` dla tej integracji FFI"

2. `[P2]` Schemat nie jest standardowym AEAD z biblioteki, tylko autorskim transportem kryptograficznym:
   - `SHAKE-stream + SHAKE-based tag + Falcon signature + FrodoKEM`
   - to wymaga mocniejszej argumentacji niz zwykle "gotowy prymityw z biblioteki jest zielony"

3. `[P2]` Formalny model z punktu 7 nadal nie jest zamkniety:
   - brak ProVerif
   - brak Tamarin
   - brak redukcji sformalizowanej w EasyCrypt

4. `[P1]` Trzeba mierzyc dokladnie ten lane Falcona, ktory jest realnie uzywany przez build produkcyjny.
   - ten punkt zostal czesciowo zamkniety:
     - `nxms-transport/build.rs` wymusza teraz:
       - `FALCON_FPEMU=1`
       - `FALCON_FPNATIVE=0`
   - to samo wymuszenie CT zostalo dodane do build skryptow harnessow C
   - pozostaje jednak wymog audytowy:
     - mierzyc nie tylko vendor-core Falcona, ale tez wrapper `ff_falcon_sign_ct()` / `ff_falcon_verify()`
   - ten wymog jest juz pokryty przez osobny harness wrappera opisany w punkcie `5.4`

## 1. Specyfikacja formalna

Status: `PASS`

Na podstawie kodu odtworzono rzeczywista specyfikacje schematu.

### 1.1 Encrypt

```text
Encrypt(sender, to, msg_type, escrow_id, seq, pk_kem, sk_sig, plaintext):
  assert seq > 0
  assert |escrow_id| = 16

  (kem_ct, ss) <- KEM.Encaps(pk_kem, alg="FrodoKEM-640-SHAKE")

  ke <- SHAKE256("NXMS-KDF-v1" || u32(|ss|) || ss || escrow_id || "ms-ke")[0..32)
  km <- SHAKE256("NXMS-KDF-v1" || u32(|ss|) || ss || escrow_id || "ms-km")[0..32)

  ct_hash <- SHAKE256("NXMS-CTHASH-v1" || kem_ct)[0..32)

  aad <- "NXMS-AAD-v1"
         || u32(|sender|) || sender
         || u32(|to|) || to
         || u32(|NXMS_KEM_ID|) || NXMS_KEM_ID
         || u32(|NXMS_SIG_ID|) || NXMS_SIG_ID
         || u32(|msg_type|) || msg_type
         || escrow_id
         || u64(seq)
         || ct_hash

  nonce <- random(24 bytes)
  keystream <- SHAKE256("NXMS-STREAM-v1" || ke || nonce)[0..|plaintext|)
  ciphertext <- plaintext XOR keystream

  tag <- SHAKE256("NXMS-TAG-v1"
                  || km
                  || u32(|aad|) || aad
                  || u32(|nonce|) || nonce
                  || u32(|ciphertext|) || ciphertext)[0..32)

  sig_msg <- "NXMS-SIG-v1"
             || u32(|aad|) || aad
             || u32(|nonce|) || nonce
             || u32(|ciphertext|) || ciphertext
             || u32(32) || tag

  sig <- Falcon.Sign_CT(sk_sig, sig_msg)
  return (kem_ct, nonce, ciphertext, tag, sig)
```

### 1.2 Verify / Decrypt

```text
VerifyDecrypt(sender, to, msg_type, escrow_id, seq,
              kem_ct, nonce, ciphertext, tag, sig,
              sk_kem, pk_sig):
  assert seq > 0
  assert |nonce| = 24
  assert |tag| = 32

  aad <- BuildAAD(sender, to, msg_type, escrow_id, seq, HASH(kem_ct))
  sig_msg <- BuildSigMessage(aad, nonce, ciphertext, tag)

  Verify Falcon signature first.
  If signature invalid: reject.

  ss <- KEM.Decaps(sk_kem, kem_ct)
  (ke, km) <- KDF(ss, escrow_id)

  tag' <- MAC(km, aad || nonce || ciphertext)
  If tag' != tag: reject.

  plaintext <- ciphertext XOR SHAKE256("NXMS-STREAM-v1" || ke || nonce)
  return plaintext
```

Dowody w kodzie:
- `build_aad()` w `nxms-transport/native/nxms_ms_transport.c:190`
- `derive_keys()` w `nxms-transport/native/nxms_ms_transport.c:278`
- `compute_tag()` w `nxms-transport/native/nxms_ms_transport.c:324`
- `build_sig_message()` w `nxms-transport/native/nxms_ms_transport.c:376`
- `nxms_ms_encrypt_packet()` w `nxms-transport/native/nxms_ms_transport.c:427`
- `nxms_ms_verify_decrypt()` w `nxms-transport/native/nxms_ms_transport.c:652`

Wniosek:
- punkt 1 jest wykonany,
- schemat zostal jednoznacznie odtworzony z kodu.

## 2. Testy wektorow (Test Vectors)

Status: `PARTIAL`

Wykonane artefakty:
- `audyt/nxms_ms_transport_reference_vectors_v1.json`
- `audyt/nxms_ms_transport_reference_impl.py`
- `nxms-transport/tests/crypto_reference_vectors.rs`

Wykonane kontrole:
- deterministyczne wektory dla `derive_keys`, `ct_hash`, `aad`, `ciphertext`, `tag`, `sig_msg`
- niezalezna referencja Python:

```bash
python3 /home/nxms-server/privAI/audyt/nxms_ms_transport_reference_impl.py \
  /home/nxms-server/privAI/audyt/nxms_ms_transport_reference_vectors_v1.json
```

Wynik:
- `python reference ok`

Ocena:
- wektory dla warstw KDF/AAD/stream/tag/sigmsg istnieja i sa sprawdzone,
- nadal brakuje w pelni deterministycznego end-to-end `Encrypt/Decrypt` z ustalonym `kem_ct` i `nonce` na publicznym API produkcyjnym,
- punkt 2 pozostaje `PARTIAL`, ale z mocna baza referencyjna.

## 3. Wlasciwosci kryptograficzne do udowodnienia

### 3a. Correctness

Status: `PASS`

Uruchomione:

```bash
cd /home/nxms-server/privAI/nxms-transport
cargo test --features crypto \
  --test crypto_negative \
  --test crypto_roundtrip \
  --test crypto_reference_vectors \
  --test crypto_proptest
```

Wynik:
- `crypto_roundtrip_encrypt_decrypt_legacy_escrow`: `PASS`
- `crypto_roundtrip_encrypt_decrypt_v2_context`: `PASS`
- `crypto_roundtrip_generic_kem_and_aead`: `PASS`
- `c_transport_packet_matches_reference_reconstruction`: `PASS`

### 3b. Integrity / tampering

Status: `PASS`

Przeszly m.in.:
- `decrypt_rejects_tampered_tag`
- `decrypt_rejects_tampered_nonce`
- `decrypt_rejects_tampered_signature`
- `decrypt_rejects_tampered_ciphertext`
- `decrypt_rejects_tampered_kem_ciphertext`
- `decrypt_rejects_wrong_sender_key`
- `decrypt_rejects_wrong_sender_id`
- `decrypt_rejects_wrong_recipient_id`
- `decrypt_rejects_wrong_msg_type`
- `decrypt_rejects_wrong_seq`

### 3c. Binding kontekstu

Status: `PASS`

Przeszly m.in.:
- `decrypt_for_context_rejects_wrong_context_id`
- `decrypt_for_context_rejects_wrong_recipient_id`

Wniosek dla punktu 3:
- `correctness`, `integrity` i `binding` sa empirycznie zaliczone dla audytowanej sciezki.

## 4. Property-Based Testing

Status: `PASS`

Wykonane:
- dodano `proptest`
- dodano `nxms-transport/tests/crypto_proptest.rs`

Test generuje arbitralne:
- `plaintext`
- `seq`

i sprawdza round-trip przez produkcyjna sciezke.

Wynik:
- `cargo test --features crypto --test crypto_proptest` przechodzi

## 5. Fuzzing strukturalny (coverage-guided)

Status: `PARTIAL / TOOLING FINDING`

### 5.1 Rust `cargo-fuzz`

Harness:
- `nxms-transport/fuzz/fuzz_targets/decrypt_fuzz.rs`

Stan koncowy harnessu:
- target laduje poprawny `fuzz_c/corpus/fixture.bin`
- uzywa zgodnych parametrow AAD:
  - `sender = "alice"`
  - `to = "bob"`
  - `msg_type = "tx_sign_req"`
  - `escrow_id = [0x5A; 16]`
  - `seq = 41`

Uruchomienie:

```bash
cd /home/nxms-server/privAI/nxms-transport
cargo +nightly-x86_64-unknown-linux-musl fuzz run --sanitizer none \
  decrypt_fuzz fuzz/corpus/decrypt_fuzz -- \
  -runs=100 -max_len=65536 -timeout=10
```

Wynik:
- crash reprodukowalny na `fuzz/corpus/decrypt_fuzz/seed_valid.json`
- artefakt:
  - `nxms-transport/fuzz/artifacts/decrypt_fuzz/crash-a09d633d3a2916429ddbd18b33365aa381bed98b`

Diagnostyka `valgrind` na binarce fuzz target:

```bash
cd /home/nxms-server/privAI/nxms-transport
valgrind --tool=memcheck --error-limit=no \
  fuzz/target/x86_64-unknown-linux-musl/release/decrypt_fuzz \
  -runs=1 fuzz/corpus/decrypt_fuzz/seed_valid.json
```

Najwazniejszy wynik:
- invalid jump
- dojscie:
  - `decrypt_fuzz.rs:104`
  - `src/crypto.rs:573`
  - `nxms_ms_verify_decrypt()` w `nxms-transport/native/nxms_ms_transport.c:746`
  - konkretnie na wywolaniu `ff_kem_decaps(...)`

Interpretacja pierwotna:
- punkt 5 nie mogl byc oznaczony jako `PASS`
- coverage-guided fuzzing ujawnil rzeczywisty crash path
- znalezienie tego crasha bylo sukcesem audytu, ale oznaczalo otwarty finding do naprawy

Aktualizacja po dodaniu regresji poza libFuzzerem:

```bash
cd /home/nxms-server/privAI/nxms-transport
cargo test --features crypto --test crypto_fuzz_regression
cargo test --release --features crypto --test crypto_fuzz_regression
```

Wynik:
- `decrypt(seed_valid.json)` przechodzi poprawnie poza libFuzzerem
- plaintext zgadza sie z fixture / seed

Wniosek z aktualizacji:
- na dzis finding z punktu 5 jest **realny**, ale dotyczy konkretnej sciezki narzedziowej `cargo-fuzz/libFuzzer` na tym runtime
- nie ma juz podstaw, aby klasyfikowac go automatycznie jako "bug funkcjonalny w `nxms_ms_verify_decrypt()`"
- przed finalnym audytem trzeba:
  1. zredukowac roznice build/runtime miedzy `cargo-fuzz` a zwyklym testem,
  2. ustalic, czy crash jest skutkiem instrumentacji libFuzzer, musl/FFI, czy konkretnego modelu linkowania `liboqs`

### 5.1.1 Minimalny reproduktor poza NXMS

Dodany target:
- `nxms-transport/fuzz/fuzz_targets/openssl_malloc_smoke.rs`

Cel:
- sprawdzic, czy crash z `cargo-fuzz` jest zwiazany z sama logika transportu NXMS,
- czy da sie go odtworzyc na minimalnym targetcie, ktory nie dotyka `nxms_ms_transport.c`,
- i nie korzysta z `FrodoKEM` ani `ff_kem_decaps()`.

Uruchomienie:

```bash
cd /home/nxms-server/privAI/nxms-transport
cargo +nightly-x86_64-unknown-linux-musl fuzz run --sanitizer none \
  openssl_malloc_smoke -- -runs=1
```

Implementacja targetu:
- ignoruje wejscie fuzzera,
- wywoluje tylko:
  - `CRYPTO_malloc(32, NULL, 0)`
  - `CRYPTO_free(...)`

Wynik:
- crash reprodukuje sie juz na pustym wejsciu `[]`
- nie ma w tej sciezce:
  - `nxms_ms_verify_decrypt()`
  - `ff_kem_decaps()`
  - `FrodoKEM`
  - rekonstrukcji `SealedPacket`

Dodatkowa diagnostyka:
- przy wariancie z `liboqs` linkowanym dynamicznie crash byl mapowany do okolic wywolania
  `OQS_KEM_frodokem_640_shake_decaps@plt`
- po przejsciu na rzeczywisty test statycznego `liboqs` crash nie zniknal,
  tylko przesunal sie do okolic `CRYPTO_malloc@plt`

Wniosek:
- obecny crash `cargo-fuzz --sanitizer none` na Alpine/musl **nie moze byc juz traktowany jako dowod na bug funkcjonalny w NXMS**
- jest to silny finding narzedziowo-runtime'owy dla sciezki `libFuzzer + musl + zewnetrzne biblioteki C`
- dopoki ten runtime finding nie zostanie wyjasniony lub obejdziany innym fuzzing flow,
  `cargo-fuzz` w tej konfiguracji nie moze byc jedynym arbitrem poprawnosci transportu

### 5.1.2 Zawazenie przyczyny: `cargo-fuzz` vs `libcrypto` na `musl`

Wykonane kontrole rozdzielajace:

1. `noop_smoke`
- target `nxms-transport/fuzz/fuzz_targets/noop_smoke.rs`
- nie linkuje `libcrypto`
- wynik:
  - `cargo +nightly-x86_64-unknown-linux-musl fuzz run --sanitizer none noop_smoke -- -runs=1`
  - `PASS`

2. `openssl_malloc_smoke`
- target `nxms-transport/fuzz/fuzz_targets/openssl_malloc_smoke.rs`
- bezposrednie wywolanie `CRYPTO_malloc/CRYPTO_free` z Rusta
- wynik:
  - crash na pustym wejsciu `[]`

3. `openssl_malloc_shim_smoke`
- target `nxms-transport/fuzz/fuzz_targets/openssl_malloc_shim_smoke.rs`
- Rust wywoluje tylko lokalny shim C, a dopiero shim C wywoluje `CRYPTO_malloc/CRYPTO_free`
- wynik:
  - crash na pustym wejsciu `[]`

4. Probe poza `cargo-fuzz`
- zwykly program C linkowany z `libcrypto`:
  - `PASS`
- zwykly minimalny program Rust na `x86_64-unknown-linux-musl`:
  - bez `-C target-feature=-crt-static` -> `SIGSEGV`
  - z `-C target-feature=-crt-static` -> `PASS`

6. Runner korpusowy poza `cargo-fuzz`
- test:
  - `nxms-transport/tests/crypto_corpus_runner.rs`
- wynik:
  - `decrypt_json_corpus_cases_outside_libfuzzer` -> `PASS`
  - `kem_decaps_seed_valid_outside_libfuzzer` -> `PASS`
- znaczenie:
  - mamy stabilny workflow do replayu korpusu i artefaktow pod normalnym buildem projektu,
    bez blokowania sie na narzedziowym problemie `cargo-fuzz`

5. Weryfikacja build metadata `cargo-fuzz`
- fingerprint:
  - `fuzz/target/x86_64-unknown-linux-musl/release/.fingerprint/.../bin-openssl_malloc_smoke.json`
  - `.../bin-openssl_malloc_shim_smoke.json`
  - `.../bin-noop_smoke.json`
- we wszystkich targetach `cargo-fuzz` zapisuje wlasne `rustflags`:
  - `-Cpasses=sancov-module`
  - `-Cllvm-args=...sanitizer-coverage...`
  - `-Cdebug-assertions`
  - `-Ccodegen-units=1`
- brak w nich projektu:
  - `-C target-feature=-crt-static`

Interpretacja:
- `cargo-fuzz` na tej konfiguracji nadpisuje / ignoruje rustflags potrzebne nam dla `x86_64-unknown-linux-musl`
- `noop_smoke` przechodzi, bo nie dotyka `libcrypto`
- targety linkujace `libcrypto` padaja, niezaleznie od tego, czy dojscie jest:
  - bezposrednio z Rusta, czy
  - przez lokalny shim C
- to wskazuje na finding klasy:
  - **tooling/build configuration problem dla fuzz targetow linkujacych `libcrypto` na musl**
  - a nie na bezposredni bug funkcjonalny `nxms_ms_verify_decrypt()`

Wniosek:
- punkt 5 pozostaje `PARTIAL / TOOLING BUILD FINDING`
- znaleziono reprodukowalna przyczyne buildowa:
  - fuzz binaries nie dziedzicza wymaganego `-C target-feature=-crt-static`
- jako obejscie robocze audytu istnieje juz normalny runner korpusowy:
  - `tests/crypto_corpus_runner.rs`
- dopoki nie wymusimy poprawnego modelu linkowania albo nie przeniesiemy tej klasy fuzzingu na inny runner/target,
  wyniki `cargo-fuzz` dla targetow dotykajacych `libcrypto` na Alpine/musl nie moga byc interpretowane jako bezposredni finding przeciwko transportowi NXMS

### 5.2 AFL++ dla warstwy C

Harness:
- `nxms-transport/fuzz_c/nxms_ms_verify_decrypt_harness.c`
- `nxms-transport/fuzz_c/pqc_kem_decaps_harness.c`

Uruchomienie:

```bash
cd /home/nxms-server/privAI/nxms-transport
CC=afl-clang-fast sh fuzz_c/build_nxms_ms_verify_decrypt_harness.sh
AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 \
  afl-fuzz -V 20 -i fuzz_c/afl_in -o fuzz_c/findings -- \
  ./fuzz_c/fuzz_nxms_decrypt fuzz_c/corpus/fixture.bin @@

CC=afl-clang-fast sh fuzz_c/build_pqc_kem_decaps_harness.sh
AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 \
  afl-fuzz -V 10 -i fuzz_c/corpus -o fuzz_c/findings_kem -- \
  ./fuzz_c/fuzz_pqc_kem_decaps fuzz_c/corpus/fixture.bin @@
```

Wynik:
- `72 new corpus items found`
- `9.36% coverage achieved`
- `0 crashes`
- `0 timeouts`
- runtime `20 sec`
- dla `ff_kem_decaps`:
  - `6-7 new corpus items found` na krotkim przebiegu
  - `~8.18% coverage achieved`
  - `0 crashes`
  - `0 timeouts`

Dodatkowy workflow:
- `nxms-transport/fuzz_c/run_nxms_fuzz_suite.sh`
- skrypt uruchamia:
  - replay korpusu poza `cargo-fuzz`
  - build obu harnessow C
  - self-test obu harnessow
  - krotki przebieg AFL++ dla obu sciezek

Aktualizacja po podziale korpusow:
- generator fixture:
  - `nxms-transport/examples/emit_c_fuzz_fixture.rs`
- obecny podzial:
  - `fuzz_c/corpus_decrypt/*` zawiera pelne pakiety `NXMSINP1`
  - `fuzz_c/corpus_kem/*` zawiera tylko seed-y startowe o roznych bitmapach pokrycia
  - `fuzz_c/corpus_kem_regression/*` zawiera szerszy zestaw regresyjny dla `ff_kem_decaps`, replayowany pod sanitizerami
  - `fuzz_c/corpus/seed_tampered_kem.bin` jest zachowany jako seed regresyjny, ale nie jest domyslnym seedem AFL++
- wynik po odswiezeniu i uruchomieniu:
  - `corpus_kem` startuje z `2` seedow:
    - `kem_ct_valid.bin`
    - `kem_ct_truncated.bin`
  - warning AFL++ o "useless seeds" dla `corpus_kem` zniknal
  - krotki smoke run:
  - `7 new corpus items found`
  - `7.67% coverage achieved`
  - `0 crashes`
  - `0 timeouts`

Aktualizacja po rozszerzeniu seedow:
- `corpus_decrypt` zawiera teraz `10` seedow:
  - `seed_valid.bin`
  - `seed_tampered_ciphertext.bin`
  - `seed_tampered_tag.bin`
  - `seed_tampered_sig.bin`
  - `seed_tampered_kem.bin`
  - `seed_seq_zero.bin`
  - `seed_short_nonce.bin`
  - `seed_short_tag.bin`
  - `seed_short_kem.bin`
  - `seed_empty_sender.bin`
- `corpus_kem_regression` zawiera `5` seedow:
  - `kem_ct_tampered.bin`
  - `kem_ct_all_zero.bin`
  - `kem_ct_all_ff.bin`
  - `kem_ct_half_zero.bin`
  - `kem_ct_last_byte_tampered.bin`

Krótki rerun AFL++ po rozszerzeniu seedow:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_nxms_fuzz_suite.sh 15
```

Wynik:
- decrypt harness:
  - `Loaded a total of 10 seeds`
  - `73 new corpus items found`
  - `9.51% coverage achieved`
  - `0 crashes`
  - `0 timeouts`
- `ff_kem_decaps` harness:
  - `Loaded a total of 2 seeds`
  - `8 new corpus items found`
  - `7.93% coverage achieved`
  - `0 crashes`
  - `0 timeouts`

Wydluzony smoke run AFL++:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_nxms_fuzz_suite.sh 60
```

Wynik:
- decrypt harness:
  - `79 new corpus items found`
  - `9.28% coverage achieved`
  - `0 crashes`
  - `0 timeouts`
- `ff_kem_decaps` harness:
  - `9 new corpus items found`
  - `8.70% coverage achieved`
  - `0 crashes`
  - `0 timeouts`

Agresywniejsza kampania rownolegla AFL++:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_nxms_fuzz_campaign.sh 60 4 3
```

Wynik:
- decrypt swarm (`4` workerow):
  - per-worker wynik koncowy:
    - `76 new corpus items found`, `9.31% coverage achieved`
    - `81 new corpus items found`, `9.40% coverage achieved`
    - `71 new corpus items found`, `9.31% coverage achieved`
    - `77 new corpus items found`, `9.40% coverage achieved`
  - `afl-whatsup` summary:
    - `Coverage reached : 9.40%`
    - `Crashes saved : 0`
    - `Hangs saved : 0`
- `ff_kem_decaps` swarm (`3` workery):
  - per-worker wynik koncowy:
    - `7 new corpus items found`, `8.70% coverage achieved`
    - `9 new corpus items found`, `8.70% coverage achieved`
    - `7 new corpus items found`, `8.70% coverage achieved`
  - `afl-whatsup` summary:
    - `Coverage reached : 8.70%`
    - `Crashes saved : 0`
    - `Hangs saved : 0`
- replay sanitizerowy po kampanii:
  - `decrypt queue: 93 cases`
  - `kem queue: 11 cases`
  - `0` sygnalow `ASan/UBSan`

Rerun kampanii po rozszerzeniu `corpus_decrypt`:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_nxms_fuzz_campaign.sh 60 4 3
```

Wynik:
- decrypt swarm (`4` workery):
  - `94 new corpus items found`, `9.60% coverage achieved`
  - `97 new corpus items found`, `9.60% coverage achieved`
  - `44 new corpus items found`, `9.60% coverage achieved`
  - `46 new corpus items found`, `9.60% coverage achieved`
  - `afl-whatsup` summary:
    - `Coverage reached : 9.60%`
    - `Crashes saved : 0`
    - `Hangs saved : 0`
- `ff_kem_decaps` swarm (`3` workery):
  - `8 new corpus items found`, `7.93% coverage achieved`
  - `7 new corpus items found`, `7.93% coverage achieved`
  - `7 new corpus items found`, `7.93% coverage achieved`
  - `afl-whatsup` summary:
    - `Coverage reached : 7.93%`
    - `Crashes saved : 0`
    - `Hangs saved : 0`
- replay sanitizerowy po kampanii:
  - `decrypt corpus: 10 cases`
  - `decrypt queue: 116 cases`
  - `kem regression corpus: 5 cases`
  - `kem queue: 10 cases`
  - `0` sygnalow `ASan/UBSan`

Nowe harnessy do realnie silowego testowania:
- parser-only:
  - `nxms-transport/fuzz_c/nxms_packet_parser_harness.c`
- decrypt structured mutation:
  - `nxms-transport/fuzz_c/nxms_ms_verify_decrypt_mut_harness.c`
- KEM structured mutation:
  - `nxms-transport/fuzz_c/pqc_kem_decaps_mut_harness.c`
- Falcon sign/verify structured mutation:
  - `nxms-transport/fuzz_c/pqc_falcon_sign_verify_mut_harness.c`

Nowe korpusy i dictionary:
- `fuzz_c/corpus_decrypt_mut/*`
  - `13` seedow programow mutacji dla konkretnych pol:
    - `seq`
    - `escrow`
    - `sender`
    - `to`
    - `msg_type`
    - `kem_ct`
    - `nonce`
    - `ciphertext`
    - `tag`
    - `sig`
- `fuzz_c/corpus_kem_mut/*`
  - `6` seedow programow mutacji dla poprawnego `kem_ct`
- `fuzz_c/corpus_falcon_mut/*`
  - `8` seedow programow mutacji dla:
    - wyboru poprawnej wiadomosci bazowej
    - wiadomości po podpisaniu
    - podpisu
    - klucza publicznego
- `fuzz_c/decrypt_mut.dict`
- `fuzz_c/kem_mut.dict`
- `fuzz_c/falcon_mut.dict`

Deep campaign AFL++:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_nxms_deep_fuzz_campaign.sh 10 1 1 1 1 1
```

Wynik:
- parser-only:
  - `Coverage reached : 32.95%`
  - `Crashes saved : 0`
  - `Hangs saved : 0`
- decrypt raw:
  - `Coverage reached : 9.48%`
  - `Crashes saved : 0`
  - `Hangs saved : 0`
- decrypt structured mutation:
  - `Coverage reached : 10.28%`
  - `Crashes saved : 0`
  - `Hangs saved : 0`
- KEM raw:
  - `Coverage reached : 7.93%`
  - `Crashes saved : 0`
  - `Hangs saved : 0`
- KEM structured mutation:
  - `Coverage reached : 11.56%`
  - `Crashes saved : 0`
  - `Hangs saved : 0`

Sanitizer replay po deep campaign:
- `parser corpus: 10 cases`
- `parser queue: 12 cases`
- `decrypt corpus: 10 cases`
- `decrypt queue: 84 cases`
- `decrypt mut corpus: 13 cases`
- `decrypt mut queue: 86 cases`
- `kem corpus: 2 cases`
- `kem regression corpus: 5 cases`
- `kem queue: 10 cases`
- `kem mut corpus: 6 cases`
- `kem mut queue: 32 cases`
- `0` sygnalow `ASan/UBSan`

Interpretacja:
- parser-only i structured mutation harnessy daja nam realnie glębszy, bardziej ukierunkowany sygnal niz sam surowy corpus packetowy
- najlepszy wzrost widac na:
  - parser-only (`~33%`)
  - KEM structured mutation (`~11.56%` vs `~7.9%` raw)
  - decrypt structured mutation (`~10.28%` vs `~9.5%` raw)
- na aktualnym deep smoke nie znaleziono:
  - crashy
  - hangow
  - naruszen `ASan/UBSan`

Wniosek dla punktu 5:
- fuzzing C harnessu na krotkim przebiegu nie znalazl crasha,
- `cargo-fuzz` dla sciezki Rust->C nadal ujawnia crash w `ff_kem_decaps`,
- zwykla regresja poza libFuzzerem nie reprodukuje problemu,
- minimalny target `openssl_malloc_smoke` pokazuje, ze crash da sie odtworzyc takze poza NXMS,
- `noop_smoke` pokazuje, ze sam `libFuzzer` nie jest martwy,
- `openssl_malloc_shim_smoke` pokazuje, ze problem nie ogranicza sie do bezposredniego wywolania z Rusta,
- fingerprint `cargo-fuzz` pokazuje brak wymaganego `-C target-feature=-crt-static`,
- nowy parser-only i structured mutation fuzzing wzmacnia audyt warstwy C bez polegania na problematycznej sciezce `cargo-fuzz`,
- caly punkt 5 pozostaje `PARTIAL / TOOLING FINDING`, a nie czysty `PASS`.

### 5.3 Dedykowany fuzz `Falcon sign/verify`

Status: `PASS WITH IMPORTANT CLARIFICATION`

Dodane artefakty:
- `nxms-transport/fuzz_c/pqc_falcon_sign_verify_mut_harness.c`
- `nxms-transport/fuzz_c/build_pqc_falcon_sign_verify_mut_harness.sh`
- `nxms-transport/fuzz_c/corpus/falcon_fixture.bin`
- `nxms-transport/fuzz_c/corpus_falcon_mut/*`
- `nxms-transport/fuzz_c/falcon_mut.dict`

Model harnessu:
- laduje poprawny fixture Falcona:
  - `sender_sk_sig`
  - `sender_pk_sig`
  - bank poprawnych wiadomosci bazowych
- dla danego wejscia buduje deterministyczny seed RNG z:
  - indeksu wariantu wiadomosci
  - programu mutacji
- podpisuje wiadomosc deterministycznie przez `falcon_sign_dyn()`
- dopiero potem mutuje:
  - wiadomosc
  - podpis
  - klucz publiczny
- na koncu wywoluje `ff_falcon_verify()`

Wazna poprawka:
- pierwszy wariant harnessu byl niestabilny dla AFL++, bo korzystal z systemowego RNG podczas podpisu
- finalna wersja uzywa deterministycznego seedowania `shake256_init_prng_from_seed()`
- dzieki temu AFL++ widzi roznice wynikajace z wejscia, a nie z szumu RNG

Wazny finding implementacyjny:
- sanitizer poczatkowo zglaszal `UBSan` na:
  - `native/vendor/falcon/inner.h:846`
  - misaligned `uint64_t` load
- dla harnessu fuzzingowego wymuszono:
  - `-DFALCON_UNALIGNED=0`
- to odcina szybki, ale dla `UBSan` problematyczny path unaligned load i daje czystszy sygnal audytowy

Coverage i stabilnosc:
- AFL smoke:
  - `18.62%` coverage
  - `0 crashes`
  - `0 hangs`
  - brak warningu `instability detected` po przejsciu na deterministyczne podpisywanie
- source coverage (`llvm-cov`) dla lane vendor-core:
  - `fuzz_c/pqc_falcon_sign_verify_mut_harness.c`: `71.86%` regions
  - `native/vendor/falcon/sign.c`: `34.52%` regions
  - `native/vendor/falcon/vrfy.c`: `69.12%` regions
  - `native/vendor/falcon/fft.c`: `62.84%` regions
  - `native/vendor/falcon/fpr.c`: `100.00%` regions
  - `native/vendor/falcon/common.c`: `73.21%` regions
  - `native/vendor/falcon/rng.c`: `61.73%` regions
  - `native/vendor/falcon/shake.c`: `95.65%` regions
  - `native/vendor/falcon/codec.c`: `32.97%` regions

Interpretacja:
- lane `5.3` sluzy do glebokiego, coverage-guided fuzzingu Falcon vendor-core przy stabilnym, deterministycznym sygnale
- to nie zastępuje jeszcze pomiaru wrappera produkcyjnego, tylko go uzupelnia

Interpretacja funkcjonalna `sign.c`:
- agregat `sign.c = 34.52%` jest mylacy, bo plik miesza aktywny lane `sign_dyn` z nieuzywanym lane `sign_tree`
- per-function coverage pokazuje, ze aktywna sciezka podpisywania jest realnie dobrze przetestowana:
  - `falcon_inner_gaussian0_sampler`: `100.00%`
  - `falcon_inner_sampler`: `100.00%`
  - `falcon_inner_sign_dyn`: `100.00%`
  - `BerExp`: `100.00%`
  - `do_sign_dyn`: `99.35%`
  - `ffSampling_fft_dyntree`: `100.00%`
- niepokryte regiony siedza glownie w lane, ktore build NXMS nie wykonuje:
  - `falcon_inner_expand_privkey`
  - `falcon_inner_sign_tree`
  - `do_sign_tree`
  - `ffLDL_*`
  - `ffSampling_fft`
  - `skoff_tree`

Wniosek:
- niski agregat `sign.c` nie oznacza juz slabo testowanej aktywnej sciezki CT
- oznacza glownie, ze source file zawiera duzo kodu `sign_tree` / expanded-key martwego dla obecnego builda NXMS

### 5.4 Pomiar wrappera Falcona realnie uzywanego przez build

Status: `PASS`

Dodane artefakty:
- `nxms-transport/fuzz_c/pqc_falcon_wrapper_mut_harness.c`
- `nxms-transport/fuzz_c/build_pqc_falcon_wrapper_mut_harness.sh`

Cel:
- mierzyc dokladnie te API, ktore wchodza do builda produkcyjnego:
  - `ff_falcon_sign_ct()`
  - `ff_falcon_verify()`
- potwierdzic, ze lane wrappera dziala na buildzie z wymuszonym CT:
  - `FALCON_FPEMU=1`
  - `FALCON_FPNATIVE=0`

Uruchomienie:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_nxms_sanitizer_suite.sh
sh fuzz_c/run_nxms_coverage_report.sh
```

Wynik sanitizer replay:
- `falcon wrapper corpus: 12 cases`
- `falcon wrapper queue: 16 cases`
- `0` sygnalow `ASan/UBSan`

Wynik source coverage (`llvm-cov`) dla wrapper lane:
- `fuzz_c/pqc_falcon_wrapper_mut_harness.c`: `67.38%` regions
- `native/nexum_cli_src/pqc_falcon.c`: `73.85%` regions
- `native/vendor/falcon/sign.c`: `34.52%` regions
- `native/vendor/falcon/vrfy.c`: `76.47%` regions
- `native/vendor/falcon/fft.c`: `81.12%` regions
- `native/vendor/falcon/fpr.c`: `100.00%` regions
- `native/vendor/falcon/common.c`: `73.21%` regions
- `native/vendor/falcon/rng.c`: `76.54%` regions
- `native/vendor/falcon/shake.c`: `95.65%` regions
- `native/vendor/falcon/codec.c`: `52.20%` regions

Wynik per-function dla wrapper API:
- `ff_falcon_keygen`: `74.07%` regions
- `ff_falcon_sign_ct`: `81.25%` regions
- `ff_falcon_verify`: `92.31%` regions
- `init_rng`: `75.00%` regions
- jedyna calkowicie niepokryta funkcja to:
  - `ff_shake256_kdf`: `0%`
  - i jest to helper poza audytowana sciezka `nxms_ms_transport.c`

Interpretacja:
- to jest juz pomiar lane produkcyjnego, nie tylko vendor-core obok wrappera
- wrapper lane nie nadaje sie tak dobrze do AFL coverage-guided jak lane `5.3`, bo realnie przechodzi przez losowe podpisywanie systemowym RNG
- mimo to:
  - self-test przechodzi
  - sanitizer replay przechodzi
  - source coverage pokazuje realne wejscie w `pqc_falcon.c` i dalszy CT backend Falcona
- na potrzeby audytu:
  - lane `5.3` traktujemy jako deep fuzz vendor-core
  - lane `5.4` traktujemy jako measurement lane sciezki realnie uzywanej przez build NXMS

### 5.5 Seed-controlled sign lane dla aktywnej sciezki `sign_dyn`

Status: `PASS`

Dodane artefakty:
- `nxms-transport/fuzz_c/pqc_falcon_sign_seed_harness.c`
- `nxms-transport/fuzz_c/build_pqc_falcon_sign_seed_harness.sh`
- `nxms-transport/fuzz_c/corpus_falcon_seed/*`
- `nxms-transport/fuzz_c/falcon_seed.dict`

Cel:
- dac fuzzerowi bezposrednia kontrole nad `48` bajtami seedu RNG podpisywania
- sprawdzic, czy niski agregat `sign.c` wynika z braku kontroli nad RNG, czy z martwego lane `sign_tree`

Wyniki:
- sanitizer replay:
  - `falcon sign-seed corpus: 6 cases`
  - `0` sygnalow `ASan/UBSan`
- AFL smoke `20s`:
  - `131 new corpus items found`
  - `19.50% coverage achieved`
  - `0 crashes`
  - `0 hangs`

Interpretacja:
- lane sign-seed poprawia coverage-guided eksploracje bitmapy AFL dla samego podpisywania
- source coverage `sign.c` nie rosnie juz ponad lane `5.3`, co potwierdza, ze brakujace regiony siedza glownie w nieuzywanym `sign_tree` / expanded-key
- to jest wynik pozytywny: aktywny `sign_dyn` nie jest juz slepym punktem

### 5.6 Dudect-like timing harness dla skompilowanej binarki CT

Status: `PASS`

Uwaga:
- w Alpine nie bylo gotowego `dudect`, dlatego dodano lokalny harness statystyczny:
  - `nxms-transport/fuzz_c/pqc_falcon_sign_ttest_harness.c`
  - `nxms-transport/fuzz_c/build_pqc_falcon_sign_ttest_harness.sh`
- harness mierzy aktywna sciezke `falcon_sign_dyn()` z:
  - tym samym CT buildem (`FALCON_FPEMU=1`, `FALCON_FPNATIVE=0`)
  - tym samym kluczem prywatnym z `falcon_fixture.bin`
  - dwiema klasami wiadomosci o tej samej dlugosci:
    - `fixed`
    - `random`
  - tym samym rozkladzie seedow RNG dla obu klas

Uruchomienie:

```bash
cd /home/nxms-server/privAI/nxms-transport
cargo run --example emit_c_fuzz_fixture --features crypto -- fuzz_c/corpus
sh fuzz_c/build_pqc_falcon_sign_ttest_harness.sh
./fuzz_c/fuzz_pqc_falcon_sign_ttest fuzz_c/corpus/falcon_fixture.bin 2048
./fuzz_c/fuzz_pqc_falcon_sign_ttest fuzz_c/corpus/falcon_fixture.bin 4096
```

Wynik `2048` prob:
- `mean_fixed_ns=9998465.85`
- `mean_random_ns=10023150.25`
- `welch_t=-0.215819`

Wynik `4096` prob:
- `mean_fixed_ns=9681856.74`
- `mean_random_ns=9690587.41`
- `welch_t=-0.145376`

Interpretacja:
- dla aktywnej sciezki `sign_dyn` nie ma obecnie sygnalu rozroznialnosci czasowej miedzy klasa `fixed` i `random`
- to nie jest jeszcze pelny formalny `dudect` report, ale jest sensownym, lokalnym testem statystycznym skompilowanej binarki CT
- wynik jest zgodny z dotychczasowym obrazem:
  - coverage aktywnej sciezki jest wysokie
  - sanitizery sa czyste
  - timing smoke nie pokazuje widocznego rozjazdu klas

Source coverage (`llvm-cov`) dla harnessu Falcona:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_nxms_coverage_report.sh
```

Wynik:
- `fuzz_c/pqc_falcon_sign_verify_mut_harness.c`
  - regions: `71.86%`
  - lines: `70.48%`
- `native/vendor/falcon/sign.c`
  - regions: `50.22%`
  - lines: `42.81%`
- `native/vendor/falcon/vrfy.c`
  - regions: `69.12%`
  - lines: `68.62%`
- `native/vendor/falcon/fft.c`
  - regions: `75.40%`
  - lines: `67.49%`

Wazne doprecyzowanie:
- finalny harness Falcona nie korzysta juz z `ff_falcon_sign_ct()` ani `ff_falcon_verify()`
- celowo mierzy vendorowy rdzen Falcona przez:
  - deterministyczne `falcon_sign_dyn()`
  - bezposrednie `falcon_verify()`
- coverage wrappera `native/nexum_cli_src/pqc_falcon.c` dla tego harnessu wynosi `0%`
- wrapper pozostaje pokryty osobno przez testy wyzszego poziomu, np.:
  - `falcon_sign_and_verify_roundtrip` w `nxms-transport/src/crypto.rs`

Krótki AFL smoke po ustabilizowaniu harnessu:

```bash
cd /home/nxms-server/privAI/nxms-transport
AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_NO_UI=1 \
  afl-fuzz -V 8 -x fuzz_c/falcon_mut.dict \
  -i fuzz_c/corpus_falcon_mut -o fuzz_c/findings_falcon_mut -- \
  ./fuzz_c/fuzz_pqc_falcon_sign_verify_mut fuzz_c/corpus/falcon_fixture.bin @@
```

Wynik:
- `Coverage reached : 18.67%`
- `0 crashes`
- `0 hangs`
- `0` warningow AFL o `instability detected`

Wniosek:
- dedykowany harness Falcona usuwa slepy punkt audytu i daje realny sygnal z `sign/verify/fft`
- ale jednoczesnie ujawnia istotny fakt architektoniczny:
  - obecny build repo nie wymusza `FALCON_FPEMU`
  - więc dzisiejszy stan nalezy opisywac jako:
    - vendor Falcon `2020-09-30`
    - `FALCON_SIG_CT` jako format podpisu
    - osobny otwarty punkt audytowy: czy produkcyjny build ma byc na backendzie emulowanym/CT

## 6. Analiza statyczna

Status: `PARTIAL`

### 6.1 C side

Valgrind na harnessie C:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/build_nxms_ms_verify_decrypt_harness.sh >/dev/null
valgrind --tool=memcheck --leak-check=full --error-exitcode=1 \
  ./fuzz_c/fuzz_nxms_decrypt --self-test \
  fuzz_c/corpus/fixture.bin fuzz_c/corpus/seed_valid.bin
```

Wynik:
- `All heap blocks were freed`
- `ERROR SUMMARY: 0 errors from 0 contexts`

ASan / UBSan na harnessie C:

```bash
cd /home/nxms-server/privAI/nxms-transport
CC=clang \
CFLAGS="-fsanitize=address,undefined -fno-omit-frame-pointer" \
LDFLAGS="-fsanitize=address,undefined" \
sh fuzz_c/build_nxms_ms_verify_decrypt_harness.sh >/dev/null
./fuzz_c/fuzz_nxms_decrypt --self-test \
  fuzz_c/corpus/fixture.bin fuzz_c/corpus/seed_valid.bin
```

Wynik:
- `self-test ok`

Sanitizer suite dla korpusow i kolejek AFL:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_nxms_sanitizer_suite.sh
```

Wynik:
- `decrypt corpus: 10 cases`
- `decrypt queue: 40 cases`
- `kem corpus: 2 cases`
- `kem regression: 1 case`
- `kem regression corpus: 5 cases`
- `kem queue: 9 cases`
- `nxms sanitizer suite ok`

Interpretacja:
- replay calych aktualnych korpusow i kolejek AFL pod `ASan/UBSan` nie ujawnil crasha ani naruszenia pamieci
- daje to drugi, niezalezny od AFL sygnal, ze warstwa C zachowuje sie stabilnie na aktualnie odkrytych wejsciach
- po kampanii rownoleglej liczebnosc replaya wzrosla do:
  - `decrypt queue: 93 cases`
  - `kem queue: 11 cases`
  i nadal pozostala zielona

Sprawdzenie wipe / memzero:

```bash
cd /home/nxms-server/privAI/nxms-transport
nm target/debug/build/nxms-transport-de065ef353fe178e/out/451a93edf38c3e47-nxms_ms_transport.o \
  | grep -e memzero -e secure_free
```

Wynik:
- symbol `memzero` obecny
- symbol `secure_free` obecny

### 6.2 Rust side

Clippy:

```bash
cd /home/nxms-server/privAI/nxms-transport
cargo clippy --features crypto --tests -- -D warnings
```

Wynik:
- `PASS`

Cargo audit:

```bash
cd /home/nxms-server/privAI/nxms-transport
cargo audit
```

Wynik:
- skan `106 crate dependencies`
- brak zgromadzonych CVE w wyniku koncowym

Miri:

```bash
cd /home/nxms-server/privAI/nxms-transport
cargo +nightly-x86_64-unknown-linux-musl miri setup
cargo +nightly-x86_64-unknown-linux-musl miri test \
  --no-default-features --test fuzz_targets_protocol_decode
```

Wynik:
- `2 passed`

Ograniczenia punktu 6:
- `Miri` obejmuje tylko pure-Rust sciezke bez FFI
- `ASan/UBSan` i `Valgrind` zostaly uruchomione na harnessie/self-test, nie na dlugim fuzzingu
- finding z punktu 5 sugeruje, ze mimo zielonych narzedzi nadal istnieje problem do wyjasnienia w sciezce `ff_kem_decaps`

Wniosek:
- punkt 6 jest mocno wykonany praktycznie,
- ale z ostroznosci pozostaje `PARTIAL`, a nie bezwarunkowe `PASS`.

## 7. Weryfikacja wlasciwosci IND-CCA2 / model teoretyczny

Status: `PARTIAL`

Potwierdzone w kodzie:

1. `signature-first`
- `ff_falcon_verify()` jest przed `ff_kem_decaps()`
- patrz `nxms-transport/native/nxms_ms_transport.c:738-746`

2. Tag wiaze caly kontekst
- `aad` obejmuje `sender`, `to`, `msg_type`, `escrow_id`, `seq`, `HASH(kem_ct)`
- `tag` obejmuje `aad`, `nonce`, `ciphertext`

3. Swiezosc kluczy
- `ss` pochodzi z `KEM.Encaps`
- `nonce` jest losowy i ma 24 bajty

Czego nadal nie ma:
- formalnego modelu ProVerif
- formalnego modelu Tamarin
- sformalizowanej redukcji w EasyCrypt

Wniosek:
- jest sensowny argument nieformalny w modelu ROM,
- punkt 7 nie jest domkniety formalnie.

## 8. Porownanie cross-implementacyjne

Status: `PARTIAL`

Wykonane:
- referencja Python:
  - `audyt/nxms_ms_transport_reference_impl.py`
- wektory JSON:
  - `audyt/nxms_ms_transport_reference_vectors_v1.json`
- referencja Rust dla KDF/AAD/tag/sigmsg:
  - `nxms-transport/tests/crypto_reference_vectors.rs`
- test porownujacy pakiet wygenerowany przez C z rekonstrukcja referencyjna po stronie testu:
  - `c_transport_packet_matches_reference_reconstruction`

Wynik:
- Python potwierdza te same wektory dla warstw deterministycznych
- Rust test potwierdza zgodnosc realnego pakietu C z niezalezna rekonstrukcja

Ograniczenie:
- nadal brak pelnego potrojnego `Python == C == Rust` dla w pelni deterministycznego end-to-end `Encrypt/Decrypt`,
  bo produkcyjne API losuje `kem_ct` i `nonce`

Wniosek:
- punkt 8 jest wykonany czesciowo, ale przekonujaco dla warstw deterministycznych.

## 9. Residual risks i punkty do jawnej decyzji

Status: `WYKONANE`

1. To nie jest standardowy AEAD z biblioteki, tylko autorski transport kryptograficzny.

2. `kem_ct` jest zwiazane przez `HASH(kem_ct)`, a nie przez pelne surowe bajty `kem_ct`.
   To wyglada praktycznie sensownie, ale powinno pozostac jawna decyzja projektowa.

3. Replay protection nie jest wymuszana kryptograficznie przez sam modul.
   Pole `seq` jest zwiazane przez AAD, ale egzekwowanie "no replay" zalezy od warstwy wyzszej.

4. Unikalnosc nonce zalezy od RNG.
   Konstrukcja nie wymusza jej deterministycznie.

5. Istnieja inne sciezki crypto poza audytowanym module.
   Helpery `xchacha20poly1305_*` w `src/crypto.rs` sa poza zakresem tego audytu.

6. Properties nie sa sprawdzone formalnie, tylko empirycznie plus nieformalny argument ROM.

7. W punkcie 5 znaleziono crash w coverage-guided fuzzingu.
   Po regresji poza libFuzzerem wyglada on bardziej na problem narzedziowy niz bezposredni bug funkcjonalny transportu,
   a minimalny target `openssl_malloc_smoke` wzmacnia te diagnoze.
   `noop_smoke` i fingerprint `cargo-fuzz` dodatkowo wskazuja, ze jest to problem zawazony do build/runtime fuzz targetow linkujacych `libcrypto` na `musl`.
   Do czasu wyjasnienia nadal nie powinien byc zamiatany pod dywan.

## 10. Decyzja projektowa: `HASH(kem_ct)` zostaje w v1

Status: `WYKONANE / ACCEPTED DESIGN DECISION`

Decyzja projektu na ten etap:
- `NXMS v1` pozostaje przy bindzie `HASH(kem_ct)`, a nie pelnym doslownym bindzie surowych bajtow `kem_ct`

Uzasadnienie techniczne:
- dla `FrodoKEM-640-SHAKE` ciphertext ma rozmiar `9720 B`
- pelny bind nie zmniejsza ani nie zwieksza wire-size pakietu, bo `kem_ct` i tak jest przesylane
- pelny bind zwieksza jednak:
  - rozmiar `AAD`
  - rozmiar `sig_message`
  - liczbe bajtow haszowanych przy tagu
  - liczbe bajtow haszowanych przy podpisie i weryfikacji
  - koszt kopiowania i alokacji w obecnej implementacji

Uzasadnienie systemowe:
- NXMS jest transportem systemowym pod `privAI`, a nie generycznym transportem dla dowolnego srodowiska
- decyzja zostala podjeta pod koszt przetwarzania i stabilnosc transcriptu, nie pod redukcje samego ruchu sieciowego

Warunki obrony tej decyzji:
1. trzeba jawnie zapisac, ze jest to swiadoma decyzja projektowa, a nie przypadkowa cecha konstrukcji
2. trzeba utrzymac mocna funkcje hash z domain separation
3. trzeba wpisac to do claimu bezpieczenstwa jako dodatkowe zalozenie bindujace
4. trzeba utrzymac testy tamperingu `kem_ct`

Wniosek:
- `HASH(kem_ct)` pozostaje zaakceptowanym kompromisem v1
- nie jest to juz "otwarta niezdecydowana kwestia"
- moze zostac ponownie ocenione w `NXMS vNext`, ale nie blokuje dalszej formalizacji obecnej wersji

## Zbiorczy werdykt wg punktow checklisty

0. Zakres audytu: `PASS`
1. Specyfikacja formalna: `PASS`
2. Test vectors: `PARTIAL`
3. Correctness / integrity / binding: `PASS`
4. Property-based testing: `PASS`
5. Coverage-guided fuzzing: `PARTIAL / TOOLING FINDING`
6. Analiza statyczna: `PARTIAL`
7. Model teoretyczny: `PARTIAL`
8. Cross-implementation: `PARTIAL`
9. Residual risks: `PASS`

## Wniosek koncowy

Na dzis:
- audyt wykonuje punkty `0-9` z checklisty,
- implementacja ma sensowna konstrukcje i duzo zielonych testow,
- ale punkt 5 ujawnil rzeczywisty crash path przy fuzzingu coverage-guided,
- po regresji poza libFuzzerem nie wyglada on juz na bezposredni bug w samym `decrypt`,
- dlatego nie mozna jeszcze zamknac tego jako `transport security PASS`, ale tez nie nalezy opisywac tego jako potwierdzonego functional breaka transportu.

Minimalny follow-up po tym audycie:
1. przygotowac poprawny fuzz flow dla targetow linkujacych `libcrypto` na Alpine/musl:
   - albo przez runner, ktory zachowa wymagane rustflags,
   - albo przez inny target / inny model fuzzingu
2. utrzymac regresje `tests/crypto_fuzz_regression.rs` jako staly test "seed_valid poza libFuzzerem"
3. utrzymac minimalny target `fuzz_targets/openssl_malloc_smoke.rs` jako reproduktor problemu narzedziowego
4. utrzymac `fuzz_targets/noop_smoke.rs` jako kontrolke pokazujaca, ze sam libFuzzer zyje
5. utrzymac `tests/crypto_corpus_runner.rs` jako roboczy replay korpusu pod normalnym buildem
6. po poprawce lub re-konfiguracji fuzzingu powtorzyc `cargo-fuzz` i zachowac nowy corpus / artifacts
7. dopiero wtedy ponownie ocenic punkt 5 i status koncowy

## Aneks A. Falcon-1024CT na realnym buildzie NXMS

Status: `STRONG CORRECTNESS / PARTIAL CT FREEZE`

Ten aneks dotyczy wyłącznie backendu podpisu Falcon używanego przez audytowaną ścieżkę transportu.

### A.1 Wymuszenie CT w buildzie

Produkcyjny build `nxms-transport` oraz harnessy C są kompilowane z:
- `FALCON_FPEMU=1`
- `FALCON_FPNATIVE=0`

To jest ważne, bo wszystkie niżej opisane pomiary dotyczą dokładnie lane'u, którego używa `ff_falcon_sign_ct()` / `ff_falcon_verify()`.

### A.2 Correctness backendu i wrappera

Wykonane i zaliczone:

1. Vendor self-test + NIST KAT na buildzie CT
- runner: `nxms-transport/fuzz_c/build_vendor_falcon_test_runner.sh`
- wynik:
  - `Test NIST KAT (512): ... done`
  - `Test NIST KAT (1024): ... done`

2. Seeded KAT dla wrappera NXMS
- wektory: `audyt/falcon_wrapper_kat_v1.json`
- emitter: `nxms-transport/examples/emit_falcon_wrapper_kat.rs`
- test: `nxms-transport/tests/falcon_wrapper_kat.rs`
- wynik:
  - wrapper deterministycznie odtwarza `sk`, `pk` i `sig`
  - `falcon_verify()` przechodzi round-trip

Wniosek:
- backend Falcon na lane CT jest poprawny referencyjnie,
- wrapper NXMS jest stabilny bit-po-bicie dla zamrożonych seedów audytowych.

### A.3 Timing smoke na skompilowanej binarce CT

Wykonane przez:
- `nxms-transport/fuzz_c/pqc_falcon_sign_ttest_harness.c`
- `nxms-transport/fuzz_c/build_pqc_falcon_sign_ttest_harness.sh`

Wyniki:

1. `wrapper sign`: `fixed msg` vs `random msg`
- `4096` pomiarów per klasa:
  - `welch_t=0.190361`

2. `wrapper sign`: `key class A` vs `key class B`
- `4096` pomiarów per klasa:
  - `welch_t=0.031822`

3. `wrapper verify`: `valid relation` vs `invalid relation`
- `4096` pomiarów per klasa:
  - `welch_t=0.178391`

4. wcześniejszy dłuższy smoke dla `sign msg`
- `8192` pomiarów per klasa:
  - `welch_t=0.053861`

Interpretacja:
- lokalne lane'y wrappera nie pokazują rozdziału klas czasowych ani dla treści wiadomości, ani dla doboru klucza, ani dla relacji `valid/invalid` na verify,
- to jest mocny sygnał praktyczny,
- ale nie jest to jeszcze pełny, formalny odpowiednik kompletnego `dudect`.

### A.4 ctgrind / valgrind i ich interpretacja

Harness:
- `nxms-transport/fuzz_c/pqc_falcon_wrapper_ctgrind_harness.c`
- `nxms-transport/fuzz_c/pqc_falcon_prepared_dyn_ctgrind_harness.c`

Stan po rozdzieleniu lane'ow:

1. encoded-key wrapper (`ff_falcon_sign_ct_seeded`)
- przy poisoningu surowego `sk` `ctgrind` raportuje zależności od danych w hotspotach:
  - `trim_i8_decode`
  - `complete_private`
  - historycznie także `BerExp` / `do_sign_dyn` / finalna serializacja podpisu
- interpretacja: to jest problem obecnego sposobu użycia zakodowanego klucza w wrapperze, a nie automatycznie całego rdzenia Falcon-CT.

2. prepared-key dynamic lane (`ff_falcon_prepare_sk` + `ff_falcon_sign_ct_prepared_seeded`)
- zachowuje tę samą referencyjną implementację Falcon-CT i ten sam aktywny rdzeń `sign_dyn`,
- eliminuje powtarzane `trim_i8_decode` / `complete_private` z gorącej ścieżki podpisu,
- wyniki:
  - `msg` lane: `0 errors from 0 contexts`
  - `sk` lane: `0 errors from 0 contexts`

Wniosek:
- referencyjny Falcon-CT da się domknąć do czystego `ctgrind`,
- ale wymaga to poprawienia sposobu użycia w naszym wrapperze: `prepare once -> sign_dyn many`,
- problem nie leży już w samym rdzeniu `sign_dyn`, tylko w dotychczasowym wołaniu podpisu bezpośrednio z encoded `sk`.

### A.5 Stan zamknięcia Falcona

Na dziś:
- `correctness`: `PASS`
- `coverage aktywnej ścieżki`: `PASS`
- `ASan/UBSan`: `PASS`
- `vendor KAT`: `PASS`
- `wrapper KAT`: `PASS`
- `timing smoke (3 wrapper lanes)`: `PASS`
- `ctgrind` dla prepared `sign_dyn`: `PASS`
- `ctgrind` dla encoded-key wrappera: `PARTIAL / migration target`

Dlatego uczciwy stan to:
- Falcon-CT jest bardzo mocno utwardzony i dobrze zbadany na ścieżce używanej przez NXMS,
- referencja Falcona zostaje,
- ścieżka do pełnego `CT freeze` jest już znana i potwierdzona eksperymentalnie: prepared-key `sign_dyn`,
- następnym krokiem nie jest zmiana algorytmu, tylko migracja realnego toru NXMS z encoded `sk` na prepared signing context.

Minimalny dalszy krok dla pełnego freeze:
1. podpiąć prepared-key signing context do realnego toru podpisu NXMS,
2. odpalić ten sam zestaw KAT/timing/ctgrind już na ścieżce produkcyjnej,
3. dopiero wtedy zapisać ostateczny `Falcon Audit Freeze Record`.

## Wykonane polecenia

```bash
cd /home/nxms-server/privAI/nxms-transport
cargo test --features crypto \
  --test crypto_negative \
  --test crypto_roundtrip \
  --test crypto_reference_vectors \
  --test crypto_proptest

cd /home/nxms-server/privAI/nxms-transport
cargo clippy --features crypto --tests -- -D warnings

cd /home/nxms-server/privAI/nxms-transport
cargo audit

python3 /home/nxms-server/privAI/audyt/nxms_ms_transport_reference_impl.py \
  /home/nxms-server/privAI/audyt/nxms_ms_transport_reference_vectors_v1.json

cd /home/nxms-server/privAI/nxms-transport
cargo +nightly-x86_64-unknown-linux-musl miri setup

cd /home/nxms-server/privAI/nxms-transport
cargo +nightly-x86_64-unknown-linux-musl miri test \
  --no-default-features --test fuzz_targets_protocol_decode

cd /home/nxms-server/privAI/nxms-transport
CC=afl-clang-fast sh fuzz_c/build_nxms_ms_verify_decrypt_harness.sh
AFL_SKIP_CPUFREQ=1 AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 \
  afl-fuzz -V 20 -i fuzz_c/afl_in -o fuzz_c/findings -- \
  ./fuzz_c/fuzz_nxms_decrypt fuzz_c/corpus/fixture.bin @@

cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/build_nxms_ms_verify_decrypt_harness.sh >/dev/null
valgrind --tool=memcheck --leak-check=full --error-exitcode=1 \
  ./fuzz_c/fuzz_nxms_decrypt --self-test \
  fuzz_c/corpus/fixture.bin fuzz_c/corpus/seed_valid.bin

cd /home/nxms-server/privAI/nxms-transport
CC=clang \
CFLAGS="-fsanitize=address,undefined -fno-omit-frame-pointer" \
LDFLAGS="-fsanitize=address,undefined" \
sh fuzz_c/build_nxms_ms_verify_decrypt_harness.sh >/dev/null
./fuzz_c/fuzz_nxms_decrypt --self-test \
  fuzz_c/corpus/fixture.bin fuzz_c/corpus/seed_valid.bin

cd /home/nxms-server/privAI/nxms-transport
cargo +nightly-x86_64-unknown-linux-musl fuzz run --sanitizer none \
  decrypt_fuzz fuzz/corpus/decrypt_fuzz -- \
  -runs=100 -max_len=65536 -timeout=10

cd /home/nxms-server/privAI/nxms-transport
valgrind --tool=memcheck --error-limit=no \
  fuzz/target/x86_64-unknown-linux-musl/release/decrypt_fuzz \
  -runs=1 fuzz/corpus/decrypt_fuzz/seed_valid.json
```
