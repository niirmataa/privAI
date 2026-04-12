# PrivAI — po ludzku

**Co to jest:** Prywatna sieć do wynajmowania mocy obliczeniowej. Nie marketplace. Nie platforma AI. Infrastruktura prywatnego compute.

**Data:** 2026-04-12  
**Źródło:** przepis PRIVAI_V0_DIAGRAMS.md na język ludzki

---

## 1. Jak działa cały system

```
Alice chce odpalić model AI prywatnie.
Alice szuka kogoś kto ma GPU.
Alice znajduje Boba.
Alice blokuje pieniądze na chainie.
Bob odpala VM na swoim GPU.
Alice używa VM przez 24 godziny.
Co pół godziny: Bob dostaje pytanie "czy masz zasób?" i odpowiada.
Po 24h: Bob mówi "1368 z 1440 razy miałem zasób."
Chain dzieli pieniądze: 46 PVA do Boba, 2 PVA do Alice.
```

**Nikt nie widział co Alice robiła na VM.** Nie chain, nie Bob, nie operator.

---

## 2. Kto w tej sieci robi co

**Validator** — pilnuje chaina. Pisze bloki. Dostaje nagrody za pisanie bloków. Nie wie nic o compute.

**Compute miner (Bob)** — ma GPU. Wynajmuje go. Dostaje PVA za to że GPU jest dostępny i szybki.

**Relay** — przekazuje zaszyfrowane wiadomości między węzłami. Widzi tylko poprzedni i następny hop. Nie widzi treści ani pełnej trasy.

**Mailbox** — przechowuje zaszyfrowane listy. Dostarcza je kiedy adresat je odbiera. Nie widzi treści.

**Exit node** — daje dostęp do internetu z sesji. Opcjonalny. Jawny opt-in. Nigdy domyślny.

Jedna maszyna może pełnić wiele ról. Ale protokół traktuje je osobno.

---

## 3. Jak wygląda życie sesji

```
1. Szukanie (off-chain)
   Alice wysyła zaszyfrowane zapytanie przez skrzynkę:
   "Szukam A100, 80GB RAM, prywatna VM, dostęp do internetu, max 2 PVA/h"
   
2. Odpowiedź (off-chain)
   Bob odpowiada przez skrzynkę:
   "Mam A100, 80GB RAM, VM, tor_gated + internet, 2 PVA/h"
   
3. Uzgodnienie (off-chain)
   Alice i Bob uzgadniają warunki przez skrzynkę.
   Warunki: klasa zasobu, cena, czas, prywatność, tryb sieciowy.
   
4. Blokada (on-chain)
   Alice blokuje 48 PVA na chainie.
   Chain zapisuje: "48 PVA zablokowane, warunki: hash(polityki)"
   Alice nie może ruszyć tych pieniędzy.
   Bob ma gwarancję że pieniądze są.
   
5. Uruchomienie (off-chain)
   Bob odpala VM na swoim GPU.
   Bob wysyła Alice dane dostępu (zaszyfrowane).
   Alice łączy się z VM.
   
6. Używanie (off-chain)
   Alice robi co chce na VM.
   Nikt nie widzi co robi.
   
7. Sprawdzanie co 60 bloków (off-chain)
   Każdy blok na chainie to "tik zegara."
   Co 60 bloków (≈30 minut):
     Chain mówi "nowy blok powstał"
     Alice oblicza pytanie: hash(sesja || numer_okna || hash_bloku)
     Alice wysyła pytanie do Boba
     Bob odpowiada (musi mieć prawdziwy GPU żeby odpowiedzieć na czas)
     Alice sprawdza: odpowiedział na czas? → PASS
                     za wolno? → FAIL
   
8. Po 24 godzinach (off-chain → on-chain)
   Alice ma: 1440 wyników (każde okno = PASS lub FAIL)
   Alice liczy: 1368 PASS
   Alice tworzy dowód: "1368 z 1440 okien przeszło"
   Alice wysyła dowód na chain.
   
9. Rozliczenie (on-chain)
   Chain oblicza: 48 × 1368 / 1440 = 46 PVA do Boba
   Chain oblicza: 48 - 46 = 2 PVA do Alice
   Chain dzieli pieniądze.
   Gotowe.
```

---

## 4. Co jest na chainie a co nie

```
NA CHAINIE (1%):
  - blokada pieniędzy (escrow lock)
  - hash polityki (lease_policy_commit)
  - hash dowodu (receipt commitment)
  - podział pieniędzy (settlement)
  - nullifiers (żeby nie wydać dwa razy)

NIE NA CHAINIE (99%):
  - szukanie compute
  - uzgadnianie warunków
  - uruchamianie VM
  - używanie VM
  - challengi i odpowiedzi
  - sprawdzanie dostępności
  - tworzenie dowodu
  - prywatność sesji
```

**Chain widzi tylko commitmenty. Nie widzi treści.**

---

## 5. Jak chain działa

```
Chain to zegar księgowy prywatności.

Zapisuje z dowodem krypto że była transakcja.
Ale nie pokazuje kwoty (zaszyfrowane).
Nie pokazuje między kim.
Nie pokazuje co było obliczane.
Daje dowód że dwa razy ta sama moneta nie będzie wydana.

Bloki powstają co ~30 sekund (konsensus).
Każdy blok zawiera transakcje (jeśli są) albo jest pusty (housekeeping).
Każdy blok ma numer (height).
Każdy blok ma hash (nieprzewidywalny przed blokiem).

Window protocol używa bloków jako zegarka:
  co 60 bloków = nowe okno
  hash bloku = nieprzewidywalny element w challengu
```

---

## 6. Jak transport działa

```
Dwa tryby:

TRYB 1: SKRZYNKA (listy)
  Każdy list jest osobno zaszyfrowany.
  FrodoKEM (ciężki, post-kwantowy) — raz na list.
  XChaCha20Poly1305 (szybki) — szyfruje treść.
  Falcon (post-kwantowy) — podpisuje list.
  Skrzynka nie widzi treści.
  
  Dobre do: discovery, negocjacja, challengi, receipty.

TRYB 2: BEZPOŚREDNIE POŁĄCZENIE (telefon)
  Alice łączy się z Bobem przez Tor.
  Raz FrodoKEM handshake → shared secret.
  Potem XChaCha20Poly1305 na wszystko (szybkie).
  Połączenie jest dwukierunkowe.
  
  Dobre do: VM session, streaming, terminal.
```

---

## 7. Jak działa rozliczenie

```
DedicatedGpu (cały GPU dla Ciebie):
  Co 30 minut: pytanie "czy masz GPU?" → odpowiedź
  Łatwe do zmierzenia. Dokładne.

SharedGpu (GPU współdzielony):
  Co 15 minut: pytanie → odpowiedź (częściej bo współdzielony)
  Trudniej zmierzyć. Mniej dokładne. Tańsze.

DedicatedCpu (cały CPU dla Ciebie):
  Co 30 minut: pytanie → odpowiedź
  Łatwe. Dokładne.

Formuła:
  miner_dostaje = kwota × zaliczone_okna / wszystkie_okna
  user_dostaje = kwota - miner_dostaje
  
  Reszta zawsze do usera.
  Zero ułamków. Zero floatów. Całkowite.
```

---

## 8. Jak działa dowód (receipt)

```
Bob tworzy dowód:
  "Przeszedłem 1368 z 1440 okien"
  + ZK proof (dowód krypto że to prawda, bez ujawniania szczegółów)
  + podpis Boba

Alice sprawdza:
  "Moje dane pokazują 1368/1440 — zgadza się" → akceptuję
  "Moje dane pokazują 1138/1440 — nie zgadza się" → spór

Spór:
  Alice mówi "sprawdźmy okno po oknie"
  Chain wymaga od Boba dowodu per okno
  Chain weryfikuje
  Kto przegrywa — płaci opłatę za spór
```

---

## 9. Jak działa tożsamość

```
Validator: ma klucz (obecny Falcon PK). To wystarczy.
Compute miner: ma osobny klucz. Inny niż validator.
User: ma klucze portfela.

Dwa różne klucze. Zero powiązań.
Validator nie wie kim jest miner.
Miner nie wie kim jest validator.

Falcon to NARZĘDZIE PODPISU. Nie tożsamość.
```

---

## 10. Jak działa szukanie compute

```
Alice wpisuje w portfelu:
  "Szukam A100, max 2 PVA/h, VM, internet"

Portfel szyfruje zapytanie i wysyła do skrzynki.
Skrzynka przechowuje (nie widzi treści).
Minerowie odbierają zapytania z skrzynki.
Każdy próbuje odszyfrować.
Bob ma A100 → pasuje → odpowiada.
Carol ma T4 → nie pasuje → ignoruje.
Dave ma A100 ale nie oferuje internetu → nie pasuje.

Alice dostaje odpowiedzi od pasujących minerów.
Alice wybiera.

Nikt nie widział co Alice szukała.
Nikt nie widział co Bob oferował innym.
```

---

## 11. Co jest odrzucone

```
NIE jest marketplace. NIE sprzedajemy modeli AI.
NIE ma publicznych profili providerów.
NIE ma publicznego rejestru usług.
NIE ma oceny jakości odpowiedzi AI.
NIE ma reputacji ludzi.
NIE ma moderatora spórów.
NIE ma internetu jako domyślnego trybu.

PrivAI sprzedaje PRYWATNY DOSTĘP DO COMPUTE.
Nic więcej. Nic mniej.
```

---

## 12. Co jest proste

```
- blokada pieniędzy → jak w banku
- podział pieniędzy → jak w umowie
- sprawdzanie co pół godziny → jak kontrola jakości
- dowód krypto → jak podpis notarialny
- skrzynka pocztowa → jak email (tylko zaszyfrowany)
- telefon przez Tor → jak VPN (tylko prywatniejszy)
- zegar z bloków → jak metronom
```

---

**To jest cały system. Nie ma nic strasznego.**
