# PrivAI — Simple 4: Jak finalnie wygląda ten system

**Opis możliwości, funkcje, problemy, zależności**

---

## O co chodzi

Przez cały dzień szukaliśmy odpowiedzi na trudne pytania. Okazało się że odpowiedzi są proste. Ten dokument opisuje jak finalnie wygląda system — po naszemu, bez technicznego żargonu.

---

## 1. Chain to co?

**Chain to zegar księgowy prywatności.**

Zapisuje z dowodem krypto że była transakcja. Ale nie pokazuje kwoty (zaszyfrowane). Nie pokazuje między kim. Nie pokazuje co było obliczane. Daje dowód że dwa razy ta sama moneta nie będzie wydana.

Bloki powstają co ~30 sekund. Każdy blok jest numerem (height). Każdy blok ma hash (nieprzewidywalny przed blokiem). Window protocol używa bloków jako zegarka.

---

## 2. Receipt truth — jak sprawdzić czy miner nie kłamie?

**Nie pytamy "czy dostarczył 7 godzin?"**

**Pytamy "czy ma zasób TERAZ? I TERAZ? I TERAZ?"**

Model:
- Co 60 bloków (≈30 minut): Alice dostaje hash bloku → oblicza pytanie → wysyła do Boba
- Bob odpowiada: computation result + czas
- Alice sprawdza: odpowiedział na czas? → PASS. Za wolno? → FAIL.
- Po 1440 oknach: Bob mówi "1368 z 1440 przeszło" + ZK proof że to prawda

Dlaczego to działa:
- Bob nie zna pytania przed blokiem (block_hash jest nieprzewidywalny)
- Bob musi mieć prawdziwy zasób żeby odpowiedzieć na czas
- ZK proof wzmacnia dowód ale nie zastępuje modelu pomiaru — potwierdza spójność danych, nie prawdę o świecie

**To nie jest "proof of delivery." To jest "challenge-sampled proof of resource possession."**

---

## 3. Metering — jak mierzyć?

**Nie mierzymy GPU time per user. To jest niemożliwe na shared GPU.**

**Mierzymy proste rzeczy:**

```
Okno = 60 bloków (≈30 minut)

Każde okno:
  availability: odpowiedział na challenge? → PASS/FAIL
  performance: odpowiedział szybciej niż floor? → PASS/FAIL/N/A

window_pass = availability AND (performance OR performance = N/A)

miner_share = kwota × passed_windows / total_windows
user_share = kwota - miner_share
```

**Dlaczego proste wystarcza:**
- Availability mówi "czy zasób istnieje"
- Performance mówi "czy zasób jest wystarczająco szybki"
- Razem = complete picture
- Nie trzeba GPU time measurement, nie trzeba FLOPS counting

---

## 4. Klasa zasobu — różne precyzje

**Nie każdy zasób da się mierzyć tak samo.**

```
DedicatedGpu (cały GPU dla Ciebie):
  Sprawdzanie co 30 minut. Dokładne. Drogie.

SharedGpu (GPU współdzielony):
  Sprawdzanie co 15 minut (częściej). Mniej dokładne. Tańsze.

DedicatedCpu (cały CPU):
  Sprawdzanie co 30 minut. Łatwe. Dokładne.

Precyzja meteringu zależy od klasy zasobu.
Nie od jednego "magicznego" protokołu.
```

---

## 5. Identity — dwa klucze wystarczą

**Nie potrzeba hidden root hierarchy na Phase 0-5.**

```
Validator: ma klucz (obecny Falcon PK). To jest validator role key.
Compute miner: ma osobny klucz. Generowany niezależnie.

Dwa różne klucze. Zero powiązań.
Validator nie wie kim jest miner.
Miner nie wie kim jest validator.

Falcon to narzędzie podpisu. Nie tożsamość.
```

Hidden root jest Phase 6+ concern. Na Phase 0-5: dwa niezależne klucze.

---

## 6. Discovery — skrzynka wystarczy

**Nie potrzeba DHT/gossip/public registry.**

```
Alice szuka A100.
Alice szyfruje zapytanie i wysyła do NXMS mailbox.
Mailbox przechowuje (nie widzi treści).
Minerowie odbierają zapytania.
Każdy próbuje odszyfrować.
Bob ma A100 → pasuje → odpowiada.
Carol ma T4 → nie pasuje → ignoruje.

Alice dostaje odpowiedzi od pasujących.
Nikt nie widział co Alice szukała.
Nikt nie widział co Bob oferował.
```

NXMS mailbox JUŻ jest discovery transportem. Wystarczy dodać endpoint.

---

## 7. Automated operator — funkcja, nie serwis

**Nie potrzeba osobnego serwisu.**

```
fn validate_settlement(receipts, policy, timeout) → decyzja:
  if receipts valid AND full coverage → Release
  if receipts empty AND timeout → Refund
  if partial → ProRata

Ta funkcja jest deterministyczna.
Te same inputs → te same outputs.
Każdy może ją wywołać.
```

Phase 1: operator key co-signs wynik tej funkcji.
Phase 2: funkcja działa bezpośrednio. Operator key nie jest potrzebny.

---

## 8. Pro-rata — sekwencja wystarczy

**Nie potrzeba nowej mechaniki note split na Phase 1.**

```
Escrow = 48 PVA. Receipts prove 95%.

Phase 1:
  Release(46 PVA) → miner (operator + user sign)
  Refund(2 PVA) → user (operator + miner sign)
  Dwie transakcje. Existing mechanics. Zero zmian.

Phase 2:
  ProRataSplit(48 PVA) → 46 miner + 2 user
  Jedna transakcja. Nowa mechanika.
```

---

## 9. Transport — FrodoKEM raz, potem ChaCha

```
NXMS SKRZYNKA (listy):
  Każdy list ma osobną kopertę FrodoKEM
  Ciężkie ale każde message jest niezależne
  Dobre do: discovery, challengi, receipty

P2P TELEFON (przez Tor):
  Raz FrodoKEM handshake → shared secret
  Potem XChaCha20Poly1305 na wszystko (szybkie)
  Dobre do: VM session, streaming
```

---

## 10. Settlement — formuła jest prosta

```
effective_windows = passed + (degraded × 0.5)

miner_share = amount × effective_windows / total_windows
user_share = amount - miner_share

Reszta zawsze do usera.
Zero ułamków. Całkowite.
```

---

## 11. Dispute — ZK proof pomaga, ale model musi być sensowny

```
Miner mówi: "1368/1440"
User mówi: "moje dane = 1138/1440"
Różnica. Spór.

Chain mówi do minera: "pokaż dowód per okno"
Miner pokazuje: każde okno = PASS/FAIL + ZK proof
Chain weryfikuje.

Kto ma rację?
ZK dowodzi zgodności z modelem pomiaru.
Ale model pomiaru sam w sobie musi być sensowny.
Jeśli challengi są za łatwe albo za rzadkie — model jest słaby.
Wtedy ZK proof "potwierdza" słaby model.

Kto przegrywa? Płaci opłatę za spór.
```

**ZK proof nie zastępuje sensownego modelu pomiaru. Wzmacnia go.**

---

## 12. Co jest odrzucone

```
NIE jest marketplace.
NIE ma publicznych profili.
NIE ma publicznego rejestru.
NIE ma oceny jakości AI.
NIE ma reputacji ludzi.
NIE ma moderatora.
NIE ma internetu jako domyślnego.

PrivAI sprzedaje PRYWATNY DOSTĘP DO COMPUTE.
```

---

## 13. Co jest proste

```
blokada pieniędzy   → jak w banku
podział pieniędzy   → jak w umowie
sprawdzanie         → jak kontrola jakości
dowód krypto        → jak podpis notarialny
skrzynka            → jak email (zaszyfrowany)
telefon przez Tor   → jak VPN (prywatniejszy)
zegar z bloków      → jak metronom
challengi           → jak test na żywo
ZK proof            → jak zaświadczenie bez szczegółów
```

---

## 14. Zależności — co musi być najpierw

```
NAJPIERW (blokuje wszystko):
  1. ComputeLeaseEscrow SpendPolicy (escrow lock nie działa bez tego)
  2. ComputeLeasePolicy struct (lease nie istnieje bez definicji)
  3. ComputeOffering struct (discovery nie działa bez oferty)

POTEM (blokuje sesję):
  4. Discovery protocol (user nie znajdzie compute)
  5. Window metering protocol (settlement nie ma dowodów)

POTEM (blokuje production):
  6. Receipt infrastructure (settlement nie ma evidence)
  7. ProRataSplit action (settlement jest all-or-nothing)
  8. Compute miner runtime (brak compute do wynajęcia)

POTEM (hardening):
  9. Relay chain (lepsza prywatność transportu)
  10. Exit node (dostęp do internetu)
  11. Dispute mechanism (co jeśli receipts się różnią)
```

---

## 15. Najważniejsze insighty

```
1. Window-based metering jest prostsze niż myśleliśmy.
   Nie trzeba mierzyć GPU time. Wystarczy PASS/FAIL per okno.

2. Receipt truth — ZK proof wzmacnia settlement evidence.
   Nie zastępuje modelu pomiaru. Model musi być sensowny sam w sobie.

3. NXMS mailbox już jest discovery transportem.
   Nie trzeba DHT/gossip.

4. Dwa klucze wystarczą na Phase 0-5.
   Nie trzeba hidden root hierarchy.

5. Pro-rata Phase 1 = Release + Refund.
   Nie trzeba nowej mechaniki note split.

6. Automated operator = funkcja.
   Nie trzeba osobnego serwisu.

7. Chain to zegar księgowy prywatności.
   Nie komputer. Nie platforma. Księgowy.

8. Część fundamentów istnieje w kodzie (escrow, konsensus, transport).
   Ale rdzeń compute lease i metering nadal wymaga zaprojektowania.
```

---

**To jest prosty finalny model operacyjny V0. Prosty. Zrozumiały. Do zaprojektowania i zbudowania.**
