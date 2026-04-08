# Audyt bezpieczenstwa autorskiego transportu NXMS - checklista robocza

## 0. Zakres audytu i granice odpowiedzialnosci
Zanim zaczniesz testy, zamroz bardzo jasno **co dokladnie audytujesz**.

Minimalna tabela zakresu:

```text
In scope:
- konkretny plik / modul
- konkretna sciezka wykonania
- konkretna wersja wire formatu
- konkretne zaleznosci kryptograficzne

Out of scope:
- inne implementacje / fallbacki
- inne wersje proto
- relay / mailbox bez deszyfrowania
- warstwy business logic nad transportem
```

To jest szczegolnie wazne, gdy repo ma wiecej niz jedna sciezke kryptograficzna.
Przykladowo:
- `nxms_ms_transport.c` moze byc audytowane osobno jako natywna sciezka C,
- ale to **nie oznacza automatycznie**, ze audyt obejmuje tez inne sciezki w crate, np. wrappery Rust albo alternatywne primitywy transportowe.

W kazdym dokumencie wynikowym dopisz:

```text
Audited path:
Out-of-scope path(s):
Wire version:
Build artifact:
Commit / snapshot:
```

Bez tego pozniej bardzo latwo przecenic zakres audytu.

## 1. Specyfikacja formalna — zanim cokolwiek testujesz
Najpierw spisz precyzyjny dokument opisujący scheme:
Encrypt(sender, to, msg_type, escrow_id, seq, pk_kem, sk_sig, plaintext):
  (ct_kem, ss) ← KEM.Encaps(pk_kem)
  (ke, km)    ← KDF(ss, escrow_id)          // SHAKE256 z domain sep.
  aad         ← BuildAAD(sender, to, ..., HASH(ct_kem))
  nonce       ← random(24 bytes)
  ciphertext  ← plaintext XOR SHAKE256("NXMS-STREAM-v1" || ke || nonce)
  tag         ← MAC(km, aad || nonce || ciphertext)
  sig_msg     ← SIG_PREFIX || aad || nonce || ciphertext || tag
  sig         ← Falcon.Sign(sk_sig, sig_msg)
  return (ct_kem, nonce, ciphertext, tag, sig)
Bez formalnej specyfikacji nie możesz udowodnić że implementacja jest czego poprawna.

2. Testy wektorów (Test Vectors)
To fundament — deterministyczna ścieżka enc/dec z hardkodowanymi wejściami i oczekiwanymi wyjściami.
c// Wygeneruj raz z zaufanej implementacji referencyjnej:
static const uint8_t TEST_SS[]    = { 0x01, 0x02, ... };  // znany shared secret
static const uint8_t TEST_NONCE[] = { 0xAA, ... };        // znany nonce
static const uint8_t TEST_KE[]    = { ... };              // oczekiwane ke
static const uint8_t TEST_KM[]    = { ... };              // oczekiwane km
static const uint8_t TEST_CT[]    = { ... };              // oczekiwany ciphertext
static const uint8_t TEST_TAG[]   = { ... };              // oczekiwany tag

void test_derive_keys_vector(void) {
    uint8_t ke[32], km[32];
    derive_keys(TEST_SS, sizeof(TEST_SS), TEST_ESCROW_ID, ke, km);
    assert(memcmp(ke, TEST_KE, 32) == 0);
    assert(memcmp(km, TEST_KM, 32) == 0);
}
Jak wygenerować wektory? Napisz niezależną implementację referencyjną w Pythonie:
pythonfrom hashlib import shake_256

def derive_keys(ss: bytes, escrow_id: bytes):
    def kdf(label: bytes) -> bytes:
        h = shake_256()
        h.update(b"NXMS-KDF-v1")
        h.update(len(ss).to_bytes(4, 'big'))
        h.update(ss)
        h.update(escrow_id)
        h.update(label)
        return h.digest(32)
    return kdf(b"ms-ke"), kdf(b"ms-km")

def stream_keystream(ke: bytes, nonce: bytes, length: int) -> bytes:
    h = shake_256()
    h.update(b"NXMS-STREAM-v1")
    h.update(ke)
    h.update(nonce)
    return h.digest(length)

def encrypt(plaintext: bytes, ke: bytes, nonce: bytes) -> bytes:
    ks = stream_keystream(ke, nonce, len(plaintext))
    return bytes(a ^ b for a, b in zip(plaintext, ks))
Jeśli Python i C dają ten sam wynik dla tych samych wejść — masz dowód zgodności.

3. Właściwości kryptograficzne do udowodnienia
3a. Correctness (poprawność round-trip)
rust#[test]
fn encrypt_decrypt_roundtrip() {
    let keys = Keys::generate().unwrap();
    let plaintext = b"hello world, this is a test message";
    
    let sealed = encrypt("alice", "bob", "tx_sign_req",
                         &[0u8; 16], 1,
                         &keys.kem_pk().unwrap(),
                         &keys.sig_sk_zeroizing().unwrap()).unwrap();
    
    let recovered = decrypt("alice", "bob", "tx_sign_req",
                            &[0u8; 16], 1, &sealed,
                            &keys.kem_sk_zeroizing().unwrap(),
                            &keys.sig_pk().unwrap()).unwrap();
    
    assert_eq!(plaintext.as_ref(), recovered.as_slice());
}
3b. Odporność na manipulację (integrity)
Każde pole z osobna musi powodować błąd:
rust#[test]
fn tampered_ciphertext_rejected() {
    let mut sealed = make_sealed_packet();
    // Zmień jeden bit ciphertext
    let mut ct = B64.decode(&sealed.ciphertext_b64).unwrap();
    ct[0] ^= 0x01;
    sealed.ciphertext_b64 = B64.encode(&ct);
    
    assert!(decrypt(..., &sealed, ...).is_err());
}

#[test] fn tampered_tag_rejected()      { /* analogicznie */ }
#[test] fn tampered_nonce_rejected()    { /* analogicznie */ }
#[test] fn tampered_kem_ct_rejected()  { /* analogicznie */ }
#[test] fn tampered_seq_rejected()     { /* analogicznie */ }
#[test] fn wrong_sender_id_rejected()  { /* analogicznie */ }
3c. Binding — AAD faktycznie wiąże kontekst
rust#[test]
fn wrong_context_id_rejected() {
    let sealed = encrypt(..., &context_id_a, ...);
    // Próba odszyfrowania z innym context_id
    assert!(decrypt(..., &context_id_b, ..., &sealed, ...).is_err());
}

4. Property-Based Testing (fuzzing semantyczny)
Użyj proptest lub quickcheck:
rustuse proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_arbitrary_plaintext(
        plaintext in prop::collection::vec(any::<u8>(), 0..=65536),
        seq in 1u64..=u64::MAX,
    ) {
        let keys = Keys::generate().unwrap();
        let sealed = encrypt("a", "b", "t", &[0u8;16], seq,
                             &keys.kem_pk().unwrap(),
                             &keys.sig_sk_zeroizing().unwrap()).unwrap();
        let out = decrypt("a", "b", "t", &[0u8;16], seq, &sealed,
                          &keys.kem_sk_zeroizing().unwrap(),
                          &keys.sig_pk().unwrap()).unwrap();
        prop_assert_eq!(plaintext, out);
    }
}

5. Fuzzing strukturalny (coverage-guided)
cargo-fuzz
rust// fuzz/fuzz_targets/decrypt_fuzz.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Próbuj zdekodować dowolne dane jako SealedPacket
    if let Ok(sealed) = serde_json::from_slice::<SealedPacket>(data) {
        let _ = decrypt("alice", "bob", "tx_sign_req",
                        &[0u8; 16], 1, &sealed,
                        &KNOWN_SK, &KNOWN_PK);
        // Nie może panikować, nie może leakować pamięci
    }
});
bashcargo fuzz run decrypt_fuzz -- -max_len=65536 -timeout=10
AFL++ dla warstwy C
bash# Kompilacja z instrumentacją
CC=afl-gcc cmake -DCMAKE_BUILD_TYPE=Debug ..
afl-fuzz -i corpus/ -o findings/ -- ./fuzz_nxms_decrypt @@

Wazna zasada interpretacji:
```text
Jesli `cargo-fuzz` pokazuje crash w sciezce Rust -> FFI -> C,
to przed zakwalifikowaniem go jako bug transportu trzeba miec
co najmniej jeden drugi kanal potwierdzenia:
- zwykly test regresyjny poza libFuzzerem, albo
- minimalny reproduktor poza audytowanym module, albo
- harness C / ASan / UBSan / Valgrind.
```

To jest szczegolnie wazne na konfiguracjach typu:
- `musl + libFuzzer + zewnetrzne biblioteki C`
- dynamiczne lub mieszane linkowanie bibliotek kryptograficznych

Dodatkowy krok diagnostyczny:
```text
Po kazdym nietypowym crashu z `cargo-fuzz` sprawdz fingerprint targetu:
- `fuzz/target/.../.fingerprint/.../bin-<target>.json`
- i potwierdz, jakie `rustflags` naprawde zostaly uzyte.
Nie zakladaj, ze `cargo-fuzz` odziedziczy wszystkie flagi z glownego projektu.
```

Praktyczny fallback dla audytu:
```text
Jesli `cargo-fuzz` jest narzedziowo niestabilny dla danej klasy targetow,
utrzymuj rownolegle zwykly runner korpusowy pod `cargo test` albo `cargo run`,
zeby nadal replayowac corpus i artifacts poza problematyczna sciezka build/runtime.
```

6. Analiza statyczna
Dla C
bash# Valgrind — wycieki i use-after-free
valgrind --tool=memcheck --leak-check=full \
         --error-exitcode=1 ./test_suite

# AddressSanitizer + UBSan
cmake -DCMAKE_C_FLAGS="-fsanitize=address,undefined -g" ..
./test_suite

# Sprawdzenie że memzero nie jest eliminowane
objdump -d libnxms.so | grep -A5 "memzero"
# Lub użyj: -fno-builtin-memset przy kompilacji
Dla Rust
bashcargo clippy -- -D warnings
cargo audit          # znane CVE w zależnościach
MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test  # undefined behavior
```

---

## 7. Weryfikacja właściwości IND-CCA2 (model teoretyczny)

To najtrudniejsza część. Nie możesz udowodnić IND-CCA2 bez redukcji do twardego problemu matematycznego. Możesz jednak udowodnić słabszą właściwość:

**Twierdzenie (nieformalne):** Jeśli KEM jest IND-CCA2-secure, SHAKE256 zachowuje się jak ROM, i Falcon jest EUF-CMA-secure, to Encrypt/Decrypt spełnia AE (Authenticated Encryption) w modelu ROM.

Dowód szkieletowy (do formalizacji np. w EasyCrypt lub ProVerif):
```
1. Sig-then-Encrypt: weryfikacja Falcon przed KEM.Decaps
   → atakujący nie może spreparować (ct_kem, nonce, ct, tag, sig) bez sk_sig
   → bo sig_msg zawiera aad || nonce || ct || tag
   → bo aad zawiera HASH(ct_kem)
   
2. Tag wiąże cały kontekst:
   → aad = f(sender, to, msg_type, escrow_id, seq, hash(ct_kem))
   → tag = SHAKE256("NXMS-TAG-v1" || km || aad || nonce || ct)
   → zmiana dowolnego pola → zmiana aad lub ct → tag nie pasuje
   
3. Klucze są świeże per wiadomość:
   → (ke, km) = KDF(ss) gdzie ss ← KEM.Encaps(pk) fresh
   → nonce jest losowy
   → nie ma reuse key+nonce
Narzędzie: ProVerif lub Tamarin do automatycznej weryfikacji protokołu.

8. Porównanie cross-implementacyjne
Najbardziej przekonujący dowód dla recenzentów:
python# Python (referencja) vs C vs Rust — wszystkie muszą dawać identyczne wyniki
# dla tych samych kluczy, nonce i plaintext

test_vectors = generate_vectors_python()
results_c    = run_c_impl(test_vectors)
results_rust = run_rust_impl(test_vectors)

assert test_vectors == results_c    == results_rust
```

## 9. Residual risks i punkty do jawnej decyzji
Po zakonczeniu testow nie koncz audytu samym "PASS/FAIL". Trzeba jeszcze spisac,
jakie ryzyka **pozostaja mimo zielonych testow**.

Minimalna lista:

```text
1. Czy to jest standardowy prymityw, czy autorska konstrukcja transportowa?
2. Czy `kem_ct` jest zwiazane wprost, czy tylko przez hash/commitment?
3. Czy replay protection zalezy tylko od warstwy wyzej (`seq`)?
4. Czy nonce uniqueness zalezy od RNG, czy jest wymuszana konstrukcyjnie?
5. Czy sa osobne sciezki crypto poza audytowanym module?
6. Czy implementacja ma properties sprawdzone formalnie, czy tylko empirycznie?
```

Przyklad waznego residual risk:

```text
`kem_ct` zwiazane przez HASH(kem_ct), a nie przez pelne bajty `kem_ct`.
To moze byc akceptowalne praktycznie przy 32-byte SHAKE256,
ale powinno byc jawnie zapisane jako swiadoma decyzja projektowa,
a nie przypadkowa wlasciwosc konstrukcji.
```

Kazdy audit result powinien konczyc sie tabela:

```text
Accepted residual risks:
- ...

Open questions:
- ...

Required before production:
- ...
```

## 10. Spójnosc dokumentacji z implementacja
Poza samym kodem trzeba porownac:
- implementacje,
- header/API,
- README,
- test vectors,
- testy negatywne,
- fuzz harness.

To jest osobny krok, bo bardzo czesto problem siedzi nie w algorytmie,
tylko w tym, ze dokumentacja opisuje juz inny protocol order niz kod.

Minimalna checklista:

```text
[ ] README opisuje aktualna kolejnosc verify/decrypt
[ ] header/API opisuje aktualna semantyke
[ ] wektory testowe odpowiadaja aktualnej implementacji
[ ] testy negatywne pokrywaja pola zwiazane w AAD
[ ] fuzz harness uzywa aktualnego wire formatu
```

## 11. Checklista obrony przed kryptografem
To jest sekcja "go / no-go" przed tym, zanim zaczniemy twierdzic, ze konstrukcja
jest gotowa do obrony jako powazny projekt kryptograficzny.

Minimalna checklista:

```text
[ ] Konstrukcja jest opisana bajt-po-bajcie i bez luk semantycznych
[ ] Zakres audytu jest jawny: co obejmuje, czego nie obejmuje
[ ] Wszystkie pola zwiazane kryptograficznie maja testy negatywne
[ ] Istnieja deterministyczne wektory referencyjne
[ ] Istnieje przynajmniej jedna niezalezna referencja do porownania
[ ] Jest uruchomiony coverage-guided fuzzing na aktualnym wire formacie
[ ] Kazdy crash fuzzingowy zostal potwierdzony lub odrzucony drugim kanalem diagnostycznym
[ ] Dla fuzz targetow sprawdzono rzeczywiste `rustflags` / fingerprint builda
[ ] Jest wykonana analiza statyczna i sanitizer run
[ ] Jest spisana lista residual risks
[ ] Jest spisany claim bezpieczenstwa (nawet jesli jeszcze nie formalny proof)
[ ] Jest jawna decyzja: co akceptujemy przed produkcja, a co jeszcze blokuje release
```

Co zwykle musi byc domkniete, zeby kryptograf potraktowal projekt serio:

```text
1. Dokladna specyfikacja i injective encoding
2. Domain separation dla wszystkich rol kluczy i transcriptow
3. Pelny binding kontekstu i pakietu
4. Test vectors + negatywne testy + property tests
5. Fuzzing i analiza pamieci
6. Jasny model bezpieczenstwa
7. Residual risks zapisane wprost, bez pudrowania
```

Szczegolnie wazne pytania kontrolne:

```text
- Czy bronimy standardowej instancji, czy naszej wlasnej konstrukcji?
- Jesli naszej: jaki jest dokladny claim bezpieczenstwa?
- Ktore elementy sa uzasadnione formalnie, a ktore tylko empirycznie?
- Co musimy jeszcze zmienic, zeby konstrukcja byla "do obrony", a nie tylko "dzialajaca"?
```

--- 

## Praktyczna ścieżka dla Twojego projektu
```
Tydzień 1:  Specyfikacja formalna + wektory Python
Tydzień 2:  Testy round-trip + testy tamperingu (wszystkie pola)
Tydzień 3:  Proptest + cargo-fuzz (min. 24h fuzzing)
Tydzień 4:  ASan/Valgrind + miri + cargo-audit
Tydzień 5+: ProVerif / zewnętrzny audyt
Najczęstszy błąd w takich projektach: tag nie wiąże ct_kem. W Twoim kodzie tag jest oparty na AAD który zawiera HASH(ct_kem) — to dobre, ale właśnie tę właściwość najważniej przetestować (test: zmiana ct_kem przy zachowaniu reszty musi odrzucić pakiet).

Na koniec:
- najpierw zaznacz scope audytu,
- potem dopiero wystawiaj werdykt,
- i zawsze odnotuj residual risks, nawet gdy wszystkie testy sa zielone.
