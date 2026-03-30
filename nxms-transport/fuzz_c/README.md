# C Fuzz Harness

Pliki:
- `nxms_packet_parser_harness.c`: parser-only harness dla formatu `NXMSINP1`
- `build_nxms_packet_parser_harness.sh`: build harnessu parsera
- `nxms_ms_verify_decrypt_harness.c`: harness plikowy dla `nxms_ms_verify_decrypt`
- `build_nxms_ms_verify_decrypt_harness.sh`: build harnessu
- `nxms_ms_verify_decrypt_mut_harness.c`: harness mutacyjny startujacy od poprawnego pakietu i modyfikujacy konkretne pola
- `build_nxms_ms_verify_decrypt_mut_harness.sh`: build harnessu mutacyjnego decrypt
- `pqc_kem_decaps_harness.c`: harness plikowy dla `ff_kem_decaps`
- `build_pqc_kem_decaps_harness.sh`: build harnessu `ff_kem_decaps`
- `pqc_kem_decaps_mut_harness.c`: harness mutacyjny startujacy od poprawnego `kem_ct`
- `build_pqc_kem_decaps_mut_harness.sh`: build harnessu mutacyjnego KEM
- `pqc_falcon_sign_verify_mut_harness.c`: harness mutacyjny dla deterministycznego `Falcon sign/verify`
- `build_pqc_falcon_sign_verify_mut_harness.sh`: build harnessu mutacyjnego Falcona
- `pqc_falcon_wrapper_mut_harness.c`: harness mutacyjny dla wrappera `ff_falcon_sign_ct()` / `ff_falcon_verify()`, czyli sciezki uzywanej w buildzie produkcyjnym
- `build_pqc_falcon_wrapper_mut_harness.sh`: build harnessu wrappera Falcona
- `pqc_falcon_sign_seed_harness.c`: harness dla aktywnej sciezki `sign_dyn`, w ktorym fuzzer kontroluje bezposrednio seed RNG podpisywania
- `build_pqc_falcon_sign_seed_harness.sh`: build harnessu sign-seed
- `pqc_falcon_sign_ttest_harness.c`: timing lane `wrapper sign`: fixed message vs random message
- `build_pqc_falcon_sign_ttest_harness.sh`: build wrapper timing lane `sign msg`
- `pqc_falcon_sign_keyclass_ttest_harness.c`: timing lane `wrapper sign`: key class A vs key class B
- `build_pqc_falcon_sign_keyclass_ttest_harness.sh`: build wrapper timing lane `sign key class`
- `pqc_falcon_verify_ttest_harness.c`: timing lane `wrapper verify`: valid vs invalid relation
- `build_pqc_falcon_verify_ttest_harness.sh`: build wrapper timing lane `verify`
- `corpus/fixture.bin`: deterministyczny zestaw kluczy testowych
- `corpus/falcon_fixture.bin`: deterministyczny zestaw kluczy Falcona + bank poprawnych wiadomosci
- `corpus/seed_valid.bin`: poprawny seed wejscia do harnessu

Przygotowanie fixture i corpus:

```bash
cd /home/nxms-server/privAI/nxms-transport
cargo run --example emit_c_fuzz_fixture --features crypto -- fuzz_c/corpus
```

To polecenie generuje:
- `fuzz_c/corpus/fixture.bin`
- `fuzz_c/corpus/seed_valid.bin`
- `fuzz_c/corpus/seed_mutated.bin` (legacy)
- `fuzz_c/corpus/seed_tampered_kem.bin` (regression-only raw `kem_ct`)
- `fuzz_c/corpus_decrypt/*` (pelne pakiety `NXMSINP1`, takze warianty skracanych pol i header-fail paths)
- `fuzz_c/corpus_decrypt_valid/*` (bank poprawnych pakietow do raw decrypt AFL)
- `fuzz_c/corpus_decrypt_mut/*` (programy mutacji 5-bajtowych rekordow dla harnessu structured decrypt)
- `fuzz_c/corpus_kem/*` (zminimalizowane surowe `kem_ct` pod AFL++)
- `fuzz_c/corpus_kem_valid/*` (bank poprawnych `kem_ct` do raw KEM AFL)
- `fuzz_c/corpus_kem_mut/*` (programy mutacji 4-bajtowych rekordow dla harnessu structured KEM)
- `fuzz_c/corpus_kem_regression/*` (szerszy zestaw regresyjny dla `ff_kem_decaps`, replayowany pod sanitizerami)
- `fuzz_c/corpus_falcon_mut/*` (programy mutacji dla:
  - wyboru wariantu poprawnej wiadomosci
  - wiadomosci po podpisaniu
  - podpisu
  - klucza publicznego)
- `fuzz_c/corpus_falcon_seed/*` (seedy dla lane `sign_dyn`, gdzie:
  - pierwszy bajt wybiera wariant wiadomosci bazowej
  - kolejne `48` bajtow to seed RNG podpisywania
  - reszta bajtow moze byc bezposrednia wiadomoscia)
- `fuzz_c/decrypt.dict`
- `fuzz_c/decrypt_mut.dict`
- `fuzz_c/kem.dict`
- `fuzz_c/kem_mut.dict`
- `fuzz_c/falcon_mut.dict`
- `fuzz_c/falcon_seed.dict`

Budowa harnessu:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/build_nxms_packet_parser_harness.sh
sh fuzz_c/build_nxms_ms_verify_decrypt_harness.sh
sh fuzz_c/build_nxms_ms_verify_decrypt_mut_harness.sh
sh fuzz_c/build_pqc_kem_decaps_harness.sh
sh fuzz_c/build_pqc_kem_decaps_mut_harness.sh
sh fuzz_c/build_pqc_falcon_sign_verify_mut_harness.sh
sh fuzz_c/build_pqc_falcon_wrapper_mut_harness.sh
sh fuzz_c/build_pqc_falcon_sign_seed_harness.sh
sh fuzz_c/build_pqc_falcon_sign_ttest_harness.sh
sh fuzz_c/build_pqc_falcon_sign_keyclass_ttest_harness.sh
sh fuzz_c/build_pqc_falcon_verify_ttest_harness.sh
```

Smoke test:

```bash
cd /home/nxms-server/privAI/nxms-transport
./fuzz_c/fuzz_nxms_parser --self-test fuzz_c/corpus/seed_valid.bin
./fuzz_c/fuzz_nxms_decrypt --self-test fuzz_c/corpus/fixture.bin fuzz_c/corpus/seed_valid.bin
./fuzz_c/fuzz_nxms_decrypt_mut --self-test fuzz_c/corpus/fixture.bin fuzz_c/corpus/seed_valid.bin /dev/null
./fuzz_c/fuzz_pqc_kem_decaps --self-test fuzz_c/corpus/fixture.bin fuzz_c/corpus/seed_valid.bin
./fuzz_c/fuzz_pqc_kem_decaps_mut --self-test fuzz_c/corpus/fixture.bin fuzz_c/corpus_kem/kem_ct_valid.bin /dev/null
./fuzz_c/fuzz_pqc_falcon_sign_verify_mut --self-test fuzz_c/corpus/falcon_fixture.bin /dev/null
./fuzz_c/fuzz_pqc_falcon_wrapper_mut --self-test fuzz_c/corpus/falcon_fixture.bin /dev/null
./fuzz_c/fuzz_pqc_falcon_sign_seed --self-test fuzz_c/corpus/falcon_fixture.bin /dev/null
./fuzz_c/fuzz_pqc_falcon_sign_ttest fuzz_c/corpus/falcon_fixture.bin 4096
./fuzz_c/fuzz_pqc_falcon_sign_keyclass_ttest 4096
./fuzz_c/fuzz_pqc_falcon_verify_ttest 4096
```

Tryb fuzz plikowy:

```bash
./fuzz_c/fuzz_nxms_parser fuzz_c/corpus/seed_valid.bin
./fuzz_c/fuzz_nxms_decrypt fuzz_c/corpus/fixture.bin fuzz_c/corpus/seed_valid.bin
./fuzz_c/fuzz_nxms_decrypt_mut fuzz_c/corpus/fixture.bin fuzz_c/corpus/seed_valid.bin fuzz_c/corpus_decrypt_mut/mut_noop.bin
./fuzz_c/fuzz_pqc_kem_decaps fuzz_c/corpus/fixture.bin fuzz_c/corpus/seed_valid.bin
./fuzz_c/fuzz_pqc_kem_decaps_mut fuzz_c/corpus/fixture.bin fuzz_c/corpus_kem/kem_ct_valid.bin fuzz_c/corpus_kem_mut/mut_noop.bin
./fuzz_c/fuzz_pqc_falcon_sign_verify_mut fuzz_c/corpus/falcon_fixture.bin fuzz_c/corpus_falcon_mut/mut_noop.bin
./fuzz_c/fuzz_pqc_falcon_wrapper_mut fuzz_c/corpus/falcon_fixture.bin fuzz_c/corpus_falcon_mut/mut_noop.bin
./fuzz_c/fuzz_pqc_falcon_sign_seed fuzz_c/corpus/falcon_fixture.bin fuzz_c/corpus_falcon_seed/seed_zero.bin
./fuzz_c/fuzz_pqc_falcon_sign_ttest fuzz_c/corpus/falcon_fixture.bin 4096
./fuzz_c/fuzz_pqc_falcon_sign_keyclass_ttest 4096
./fuzz_c/fuzz_pqc_falcon_verify_ttest 4096
```

Uwagi do `fuzz_pqc_kem_decaps`:
- jesli wejscie zaczyna sie od `NXMSINP1`, harness wycina z niego samo pole `kem_ct`
- jesli nie, traktuje caly plik jako surowy `kem_ct`
- dzieki temu mozna uzywac:
  - obecnego `seed_valid.bin`
  - zmutowanych plikow z tego samego korpusu
  - albo dedykowanych plikow zawierajacych tylko ciphertext KEM
- `fuzz_c/corpus_kem` trzyma tylko seed-y startowe o roznych bitmapach pokrycia
- `seed_tampered_kem.bin` zostaje zachowany jako seed regresyjny, ale nie jest domyslnym seedem AFL++, zeby nie rozdymac korpusu bez zysku pokrycia

AFL++ przy dostepnym narzedziu:

```bash
afl-fuzz -x fuzz_c/decrypt.dict -i fuzz_c/corpus_decrypt -o fuzz_c/findings -- ./fuzz_c/fuzz_nxms_decrypt fuzz_c/corpus/fixture.bin @@
afl-fuzz -x fuzz_c/decrypt.dict -i fuzz_c/corpus_decrypt -o fuzz_c/findings_parser -- ./fuzz_c/fuzz_nxms_parser @@
afl-fuzz -x fuzz_c/decrypt_mut.dict -i fuzz_c/corpus_decrypt_mut -o fuzz_c/findings_decrypt_mut -- ./fuzz_c/fuzz_nxms_decrypt_mut fuzz_c/corpus/fixture.bin fuzz_c/corpus/seed_valid.bin @@
afl-fuzz -x fuzz_c/kem.dict -i fuzz_c/corpus_kem_valid -o fuzz_c/findings_kem -- ./fuzz_c/fuzz_pqc_kem_decaps fuzz_c/corpus/fixture.bin @@
afl-fuzz -x fuzz_c/kem_mut.dict -i fuzz_c/corpus_kem_mut -o fuzz_c/findings_kem_mut -- ./fuzz_c/fuzz_pqc_kem_decaps_mut fuzz_c/corpus/fixture.bin fuzz_c/corpus_kem/kem_ct_valid.bin @@
afl-fuzz -x fuzz_c/falcon_mut.dict -i fuzz_c/corpus_falcon_mut -o fuzz_c/findings_falcon_mut -- ./fuzz_c/fuzz_pqc_falcon_sign_verify_mut fuzz_c/corpus/falcon_fixture.bin @@
afl-fuzz -x fuzz_c/falcon_seed.dict -i fuzz_c/corpus_falcon_seed -o fuzz_c/findings_falcon_seed -- ./fuzz_c/fuzz_pqc_falcon_sign_seed fuzz_c/corpus/falcon_fixture.bin @@
```

Skrot do calego smoke workflow:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_nxms_fuzz_suite.sh 10
```

Skrypt:
- odpala replay korpusu poza `cargo-fuzz`
- odswieza fixture, split korpusy i dictionary
- buduje parser, oba harnessy plikowe i oba harnessy mutacyjne
- buduje tez wszystkie lane'y Falcona:
  - vendor-core deterministyczny do glebokiego fuzzingu
  - wrapper `ff_falcon_sign_ct/verify`, czyli sciezke realnie uzywana przez `nxms-transport`
  - sign-seed lane dla aktywnego `sign_dyn`
- robi self-test wszystkich harnessow
- jesli `afl-fuzz` jest dostepny, uruchamia siedem krotkich fuzzingow:
  - parser
  - decrypt
  - decrypt structured mutation
  - kem
  - kem structured mutation
  - Falcon sign/verify structured mutation
  - Falcon sign-seed

Agresywniejsza kampania AFL++:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_nxms_fuzz_campaign.sh 60 4 3
```

Argumenty:
- `60` = czas kampanii w sekundach per worker
- `4` = liczba workerow dla decrypt
- `3` = liczba workerow dla `ff_kem_decaps`

Skrypt:
- odswieza fixture i replay poza `cargo-fuzz`
- buduje harnessy pod AFL++
- uruchamia roje `-M/-S` dla obu targetow
- replayuje kolejki kampanii pod `ASan/UBSan`
- zbiera summary przez `afl-whatsup`

Deep campaign dla realnego dociskania parsera i sciezek strukturalnych:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_nxms_deep_fuzz_campaign.sh 180 2 4 4 3 3 2 2
```

Argumenty:
- `180` = czas kampanii w sekundach per worker
- `2` = liczba workerow parser-only
- `4` = liczba workerow decrypt raw
- `4` = liczba workerow decrypt structured mutation
- `3` = liczba workerow KEM raw
- `3` = liczba workerow KEM structured mutation
- `2` = liczba workerow Falcon sign/verify mutation
- `2` = liczba workerow Falcon sign-seed

Skrypt:
- odswieza fixture i korpusy
- replayuje baseline poza `cargo-fuzz`
- buduje wszystkie harnessy pod AFL++
- buduje wszystkie lane'y Falcona:
  - vendor-core sign/verify
  - wrapper measurement lane
  - sign-seed lane
- uruchamia roje dla:
  - parser-only
  - decrypt raw packet
  - decrypt structured mutation
  - KEM raw ciphertext
  - KEM structured mutation
  - Falcon sign/verify structured mutation
  - Falcon sign-seed
- replayuje wszystkie kolejki pod `ASan/UBSan`
- daje summary z `afl-whatsup`

Sanitizer suite:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_nxms_sanitizer_suite.sh
```

Skrypt:
- odswieza fixture i split korpusy
- buduje parser, oba harnessy plikowe i oba harnessy mutacyjne z `ASan/UBSan`
- buduje tez dedykowany harness Falcon sign/verify z `ASan/UBSan`
- buduje tez harness wrappera Falcona z `ASan/UBSan`
- buduje tez harness Falcon sign-seed z `ASan/UBSan`
- replayuje:
  - `corpus_decrypt`
  - `corpus_decrypt_valid`
  - `corpus_decrypt_mut`
  - `corpus_kem`
  - `corpus_kem_valid`
  - `corpus_kem_mut`
  - `corpus_kem_regression`
  - `corpus_falcon_mut`
  - `corpus_falcon_seed`
  - kolejki AFL, jesli istnieja
- uruchamia tez regresyjny `seed_tampered_kem.bin`

Wazna uwaga dla lane'ow Falcona:
- `fuzz_pqc_falcon_sign_verify_mut` mierzy gleboki vendor-core Falcona na deterministycznym podpisywaniu seedowanym do fuzzingu
- `fuzz_pqc_falcon_wrapper_mut` mierzy wrapper `ff_falcon_sign_ct()` / `ff_falcon_verify()`, czyli dokladnie te API, ktore sa uzywane przez transport
- `fuzz_pqc_falcon_sign_seed` mierzy aktywna sciezke `sign_dyn` z bezposrednia kontrola seedu RNG, zeby odroznic aktywne podpisywanie od martwego lane `sign_tree`
- `fuzz_pqc_falcon_sign_ttest` mierzy wrapper `ff_falcon_sign_ct_seeded()` na klasach `fixed msg` vs `random msg`
- `fuzz_pqc_falcon_sign_keyclass_ttest` mierzy wrapper `ff_falcon_sign_ct_seeded()` na klasach `key A` vs `key B`
- `fuzz_pqc_falcon_verify_ttest` mierzy wrapper `ff_falcon_verify()` na klasach `valid relation` vs `invalid relation`
- oba lane'y budowane sa z wymuszonym:
  - `FALCON_FPEMU=1`
  - `FALCON_FPNATIVE=0`
  - oraz w harnessach C z `FALCON_UNALIGNED=0`, zeby nie zaszumiec `UBSan` fast-pathem vendorowym

Aktualny rozszerzony zestaw seedow:
- `corpus_decrypt`:
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
- `corpus_kem_regression`:
  - `kem_ct_tampered.bin`
  - `kem_ct_all_zero.bin`
  - `kem_ct_all_ff.bin`
  - `kem_ct_half_zero.bin`
  - `kem_ct_last_byte_tampered.bin`
- `corpus_decrypt_mut`:
  - `mut_noop.bin`
  - `mut_kem_first_xor.bin`
  - `mut_kem_mid_xor.bin`
  - `mut_ciphertext_first_xor.bin`
  - `mut_tag_first_zero.bin`
  - `mut_sig_last_xor.bin`
  - `mut_seq_low_zero.bin`
  - `mut_sender_first_zero.bin`
  - `mut_escrow_first_xor.bin`
  - `mut_to_first_zero.bin`
  - `mut_msg_first_zero.bin`
  - `mut_nonce_first_add.bin`
  - `mut_multi_mix.bin`
- `corpus_kem_mut`:
  - `mut_noop.bin`
  - `mut_first_xor.bin`
  - `mut_mid_xor.bin`
  - `mut_last_xor.bin`
  - `mut_first_zero.bin`
  - `mut_mid_ff.bin`
- `corpus_falcon_mut`:
  - `mut_noop.bin`
  - `mut_msg_first_xor.bin`
  - `mut_msg_mid_add.bin`
  - `mut_sig_first_xor.bin`
  - `mut_sig_mid_xor.bin`
  - `mut_sig_last_zero.bin`
  - `mut_pk_first_xor.bin`
  - `mut_multi_mix.bin`

Wazna uwaga dla Falcona:
- harness Falcona podpisuje deterministycznie przez seedowane `falcon_sign_dyn()`
- to jest celowe: AFL nie moze dostawac losowego podpisu dla tego samego wejscia, bo wtedy kampania robi sie niestabilna
- dla fuzzingu wymuszono tez `-DFALCON_UNALIGNED=0`, zeby `UBSan` nie byl zalewany przez vendorowy fast-path unaligned load

## Falcon CT verification lane

Dla Falcona uzywanego przez `nxms-transport` mamy osobny lane correctness + CT review.

Pliki:
- `run_falcon_round3_reference_lane.sh`: reprodukcja `falcon-round3/KAT/*.req/.rsp` z canonical paczki Round 3
- `verify_falcon_round3_sync.sh`: sprawdza, ze `native/vendor/falcon` pozostaje byte-to-byte zgodny z `falcon-round3/Extra/c`
- `prepare_falcon_ctgrind_overlay.sh`: audit-only overlay dla ctgrind aplikowany na kopie w tempie, bez ruszania vendora
- `falcon_ctgrind_overlay.patch`: jawny patchset overlay dla `sign.c` i `falcon.c`
- `pqc_falcon_wrapper_ctgrind_harness.c`: wrapper sign-path pod `valgrind/memcheck`
- `build_pqc_falcon_wrapper_ctgrind_harness.sh`: build harnessu ctgrind
- `pqc_falcon_prepared_dyn_ctgrind_harness.c`: prepared-key `sign_dyn` pod `valgrind/memcheck`
- `build_pqc_falcon_prepared_dyn_ctgrind_harness.sh`: build harnessu prepared `sign_dyn`
- `run_falcon_ct_verification.sh`: zbiera correctness, timing smoke, ctgrind i objdump artifacts

Wazne:
- ten lane jest budowany z wymuszonym:
  - `FALCON_FPEMU=1`
  - `FALCON_FPNATIVE=0`
- czyli mierzymy dokladnie ten backend, ktorego uzywa realny build `nxms-transport`

Uruchomienie:

```bash
cd /home/nxms-server/privAI/nxms-transport
sh fuzz_c/run_falcon_ct_verification.sh 4096
```

Co robi skrypt:
1. odpala wrapper seeded KAT:
   - `cargo test --no-default-features --features crypto,falcon-audit-raw-api --test falcon_wrapper_kat`
2. sprawdza, ze `native/vendor/falcon` pozostaje byte-to-byte zgodny z `falcon-round3/Extra/c`
3. odtwarza canonical `falcon-round3/KAT/*.req/.rsp` z paczki Round 3 i porownuje wynik byte-to-byte z pakowanymi plikami
3. odpala lokalny timing smoke fixed-vs-random na skompilowanej binarce:
   - `fuzz_pqc_falcon_sign_ttest`
   - `fuzz_pqc_falcon_sign_keyclass_ttest`
   - `fuzz_pqc_falcon_verify_ttest`
4. odpala `valgrind/memcheck` dla audit-only wrappera z encoded `sk`
5. odpala `valgrind/memcheck` dla prepared-key `sign_dyn`
6. zbiera `objdump` artifacts dla wrappera i lane `sign_dyn`

Artefakty laduja do:

```bash
nxms-transport/fuzz_c/coverage/falcon_freeze
```

Aktualna interpretacja:
- `wrapper seeded KAT`: zielone
- `Round 3 sync + packaged KAT reproduction`: zielone
- `timing smoke`: zielone
  - aktualne wartosci trzeba czytac z `coverage/falcon_freeze/falcon_objdump_summary.txt`
- `ctgrind` dla audit-only encoded `sk` wrappera: nadal `PARTIAL`
- `ctgrind` dla prepared-key `sign_dyn`: czyste `PASS` i to on jest runtime gate

Wniosek z rozdzielenia lane'ow:
- problem nie siedzi w samym aktywnym rdzeniu `Falcon-CT sign_dyn`
- problem siedzial w dotychczasowym uzyciu zakodowanego `sk`, gdzie kazde podpisanie przechodzilo przez:
  - `trim_i8_decode`
  - `complete_private`
- po przygotowaniu klucza raz i podpisywaniu przez prepared `sign_dyn` na tej samej referencji dostajemy czysty `ctgrind`

Czyli:
- referencyjny Falcon-CT zostaje,
- ale kanoniczna sciezka do domkniecia CT w signerze powinna isc przez prepared key, a nie przez surowy encoded `sk` przy kazdym podpisie,
- dlatego `ctgrind` pozostaje bardzo cennym narzedziem diagnostycznym,
- a wynik clean `PASS` mamy juz dla tej poprawionej, nadal referencyjnej sciezki.

Aktualny stan po deep smoke:
- `sh fuzz_c/run_nxms_sanitizer_suite.sh`
  - `parser corpus: 10 cases`
  - `decrypt corpus: 10 cases`
  - `decrypt queue: 83 cases`
  - `decrypt mut corpus: 13 cases`
  - `kem corpus: 2 cases`
  - `kem regression corpus: 5 cases`
  - `kem queue: 10 cases`
  - `kem mut corpus: 6 cases`
  - `0` sygnalow `ASan/UBSan`
- `sh fuzz_c/run_nxms_deep_fuzz_campaign.sh 10 1 1 1 1 1`
  - parser: `Coverage reached : 32.95%`
  - decrypt: `Coverage reached : 9.48%`
  - decrypt structured mutation: `Coverage reached : 10.28%`
  - kem: `Coverage reached : 7.93%`
  - kem structured mutation: `Coverage reached : 11.56%`
  - `0` crashy
  - `0` hangow
