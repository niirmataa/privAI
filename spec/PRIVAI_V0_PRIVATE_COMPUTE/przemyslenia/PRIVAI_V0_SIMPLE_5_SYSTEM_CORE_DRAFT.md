# privAI V0: System Core Draft

**Status:** rdzen systemu ustalony, finalne formaty do dopisania

Cel tego dokumentu:

- zapisac prosty rdzen systemu,
- oddzielic to, co juz jest ustalone, od tego, co jeszcze trzeba dopisac,
- nie mieszac finalnego modelu operacyjnego z pobocznymi rozwazaniami.

Ten dokument nie definiuje jeszcze finalnych wire formatow.
Ten dokument definiuje rdzen logiki systemu.

---

## 1. Czym jest system

privAI V0 to prywatny system wynajmu zasobow compute.

User nie kupuje "jakosci AI".
User kupuje prywatny dostep do zasobu compute.

System sklada sie z:

- prywatnego discovery,
- lease policy,
- escrow,
- sesji compute,
- meteringu okienkowego,
- settlement,
- prywatnego transportu,
- prostego modelu tozsamosci dla Phase 0-5.

Nie jest to marketplace.
Nie ma publicznych profili.
Nie ma moderatora.
Nie ma oceny jakosci outputu AI.

---

## 2. Najprostszy flow systemu

1. User szuka zasobu przez prywatne discovery.
2. Miner odpowiada oferta compute.
3. User akceptuje lease policy.
4. User lockuje escrow.
5. Startuje sesja compute.
6. Sesja jest dzielona na okna.
7. W kazdym oknie sprawdzana jest dostepnosc i opcjonalnie performance.
8. Po sesji powstaje aggregate receipt.
9. Settlement liczy udzial minera i usera.
10. Chain wykonuje rozliczenie.

---

## 3. Chain

Chain jest zegarem i ksiegowym prywatnosci.

Chain:

- zapisuje transakcje,
- pilnuje nullifierow,
- pilnuje ze te same srodki nie sa wydane dwa razy,
- sluzy jako zegar przez block height,
- egzekwuje escrow i settlement.

Chain nie powinien widziec:

- workloadu,
- outputu,
- pelnej telemetrii sesji,
- publicznych profili stron.

Chain powinien widziec tylko minimum:

- commitmenty,
- akcje escrow,
- timeouty,
- wynik settlement,
- dowody potrzebne do rozstrzygniecia sporu.

---

## 4. Discovery

Discovery w V0 powinno byc prywatne.

Minimalny model:

- user wysyla zaszyfrowane zapytanie,
- NXMS mailbox sluzy jako transport,
- minerzy odbieraja zapytania,
- pasujacy miner odpowiada oferta.

To oznacza:

- brak publicznego rejestru jako default,
- brak publicznych profili,
- brak publicznej historii activity.

### Finalne formaty do dopisania

- format `ComputeOffering`
- format `DiscoveryQuery`
- zasady routingu zapytan
- minimalne pola oferty

---

## 5. Lease

Lease jest umowa o prywatny dostep do zasobu.

Lease musi wiazac:

- klase zasobu,
- czas trwania,
- cene,
- tryb sieci,
- klase prywatnosci,
- minimalny performance floor,
- zasady settlement.

Lease musi byc zwiazany z escrow.

### Finalne formaty do dopisania

- finalny `ComputeLeasePolicy`
- finalny `ComputeLeaseEscrow`
- pole commitmentu polityki

---

## 6. Session

Sesja nie jest liczona jako jedna nieprzerwana "godzina prawdy".
Sesja jest dzielona na okna.

Kazde okno ma:

- identyfikator okna,
- challenge albo checkpoint dostepnosci,
- opcjonalny benchmark performance,
- wynik PASS/FAIL albo PASS/FAIL/N/A.

To jest najprostszy natywny model V0.

Nie mierzymy magicznego "dokladnego GPU time per user".
Mierzymy:

- czy zasob byl dostepny,
- czy odpowiedzial,
- czy utrzymal minimalny floor.

### Finalne formaty do dopisania

- rozmiar okna
- harmonogram okien
- sposob generowania checkpointu / challenge
- reguly PASS/FAIL/N/A

---

## 7. Metering

Metering opiera sie na oknach.

Najprostszy model:

- `availability = checkpoint odpowiedzial / nie odpowiedzial`
- `performance = benchmark powyzej floor / ponizej floor / N/A`

Wynik okna:

- `window_pass = availability AND performance`
  albo
- `window_pass = availability AND (performance OR performance = N/A)`

Precyzja meteringu zalezy od klasy zasobu.

Przyklad:

- `DedicatedGpu` -> mocna mierzalnosc
- `MigInstance` -> dobra mierzalnosc
- `SharedGpu` -> slabiej, bardziej statystycznie
- `DedicatedCpu` -> latwo mierzalne

### Finalne formaty do dopisania

- klasy zasobow
- benchmark suite per klasa zasobu
- floor per klasa zasobu
- jak liczymy degraded windows

---

## 8. Telemetry

Najwazniejsza zasada:

telemetria jest mierzalna, ale prywatna.

Prywatna telemetria nalezy glownie do:

- usera,
- minera.

Nie do:

- chaina,
- publicznego rejestru,
- innych uczestnikow,
- operatora jako pelny stream.

Podzial danych:

- `private telemetry`
  tylko user i miner

- `settlement evidence`
  minimalny agregat i dowody potrzebne do rozliczenia

- `public chain data`
  tylko commitmenty i wynik

---

## 9. Agent i ciaglosc srodowiska

Miner powinien uruchamiac agenta jako daemon.

Agent nie jest zrodlem prawdy o calym swiecie.
Agent sluzy do potwierdzania:

- ze daemon dziala,
- ze srodowisko od uruchomienia nie zostalo zmienione,
- ze nie wykryto anomalii w czasie sesji.

Przed uruchomieniem:

- benchmark,
- stress test,
- fingerprint srodowiska,
- hash binarki,
- hash configu,
- podpisany manifest startowy.

W trakcie:

- signed records per window,
- hash-chain rekordow,
- ciaglosc telemetry.

Server nie jest arbitrem prawdy.
Server jest tylko narzedziem potwierdzajacym ciaglosc daemona i srodowiska.

### Finalne formaty do dopisania

- startup manifest
- window telemetry JSON
- continuity proof
- anomaly markers

---

## 10. Receipt

Receipt nie powinien byc pelnym strumieniem telemetry.
Receipt powinien byc agregatem sesji.

Minimalnie:

- `total_windows`
- `passed_windows`
- opcjonalnie `degraded_windows`
- commitment do szczegolow okien
- podpis

Receipt nie powinien udawac "prawdy absolutnej".
Ma byc syntetycznym dowodem, z ktorego wynikalo settlement.

### Finalne formaty do dopisania

- aggregate receipt shape
- evidence root
- per-window proof relation

---

## 11. Settlement

Settlement powinien byc prosty.

Minimalny model:

- `miner_share = amount * passed_windows / total_windows`
- `user_share = amount - miner_share`

Opcjonalnie:

- `effective_windows = passed + degraded * weight`

Zasada:

- reszta zawsze idzie do usera
- integer arithmetic only

Phase 1:

- pro-rata moze dzialac jako sekwencja `Release + Refund`

Phase 2:

- proper `ProRataSplit`

### Finalne formaty do dopisania

- exact settlement formula
- rounding rules
- degraded window weights
- dispute fee rules

---

## 12. Dispute

Spor nie powinien byc oparty o "slowo przeciw slowu".

Spor powinien byc oparty o:

- signed evidence,
- continuity records,
- per-window records,
- proof spójnosci i poprawnego wyliczenia.

ZK proof nie ma dowodzic wszystkiego.
ZK proof ma dowodzic tylko tego minimum, ktore jest potrzebne do potwierdzenia poprawnosci strony w danym sporze.

Przyklad:

- czy aggregate receipt rzeczywiscie wynika z per-window danych
- czy settlement formula zostala policzona poprawnie
- czy ciag telemetry nie zostal przerwany

### Finalne formaty do dopisania

- dispute trigger
- dispute evidence package
- zakres ZK proof

---

## 13. Identity

Na Phase 0-5 wystarczy prostszy model:

- validator key
- compute miner key

Dwa niezalezne klucze.
Bez koniecznosci wdrazania calej hidden-root hierarchy od razu.

Falcon jest narzedziem podpisu.
Nie finalna tozsamoscia systemu.

Pelniejszy model hidden root moze byc warstwa pozniejsza.

### Finalne formaty do dopisania

- role-key semantics
- miner key registration path
- future hidden-root migration path

---

## 14. Transport

V0 transport:

- NXMS mailbox
- Tor SOCKS5

Rozdzial:

- NXMS dla discovery, receipts, checkpoint messages
- P2P / Tor dla sesji runtime i dluzszych strumieni

### Finalne formaty do dopisania

- envelope shape
- session transport rules
- routing constraints

---

## 15. Co jest juz ustalone

- system nie jest marketplace
- settlement nie dotyczy jakosci AI
- chain jest zegarem i ksiegowym prywatnosci
- sessions sa dzielone na okna
- metering jest window-based
- telemetry jest prywatna
- settlement jest pro-rata z passed/total
- NXMS jest discovery transportem
- validator i miner moga byc oparci na dwoch niezaleznych kluczach
- operatorless target ma byc budowany warstwowo

---

## 16. Co jeszcze trzeba dopisac

Nie brakuje juz rdzenia.
Brakuje finalnych formatow i decyzji granicznych.

Najwazniejsze otwarte rzeczy:

- finalny shape `ComputeLeasePolicy`
- finalny shape `ComputeOffering`
- finalny shape `ComputeLeaseEscrow`
- finalny shape aggregate receipt
- benchmark suite per klasa zasobu
- exact settlement formula
- exact dispute package
- exact continuity / manifest evidence format

---

## 17. Najwazniejszy wniosek

Rdzen systemu jest juz ustalony.

Prawdziwa praca nie polega juz na wymyslaniu czym jest privAI.
Prawdziwa praca polega teraz na dopisaniu:

- formatow,
- proof boundaries,
- benchmark rules,
- settlement rules,
- i code landing path.

To jest punkt, w ktorym projekt przestaje byc zbiorem ogolnych wizji,
a zaczyna byc systemem do zbudowania.
