# Szablon Correction Pill

## Cel

Ten dokument opisuje, jak korygowac model, ktory wszedl na zla trajektorie
rozumienia systemu, backlogu albo repo reality.

Correction pill nie sluzy do:
- ponownego tlumaczenia calego systemu,
- robienia nowego onboardingowego promptu,
- ani do karcenia modelu.

Sluzy do:
- szybkiego zatrzymania blednej inferencji,
- podania konkretnego dowodu,
- przebudowy backlogu lub reading path,
- i powrotu na poprawna trajektorie bez restartu sesji.

## Kiedy uzywac

Uruchom correction pill, gdy model:
- oglosil brak bez sprawdzenia sasiednich warstw,
- pomylil `partial` z `absent`,
- pomylil `future direction` z `current backlog`,
- zbudowal falszywe dependency,
- albo zrobil reading path oparty o bledne zalozenie.

## Obowiazkowy format

```text
KOREKTA:
- Twierdziles: [co model powiedzial]
- Prawda: [co jest faktycznie]
- Dowod: [plik / test / dokument]
- Co sie zmienia: [status, backlog, scope albo reading path]
- Kontynuuj od: [najblizszy nastepny krok]
```

## Zasady dobrej korekty

### 1. Koryguj tylko jedna trajektorie naraz
Nie wrzucaj 5 nowych tematow.
Correction pill ma wyprostowac jedna zla os, nie robic nowy onboarding.

### 2. Zawsze dawaj dowod
Najlepiej:
- plik,
- test,
- albo dokument z jednoznacznym statusem.

### 3. Zawsze powiedz, co zmienia sie praktycznie
Model musi wiedziec, czy:
- task wypada z backlogu,
- zmienia status,
- wymaga tylko hardeningu,
- albo reading path ma isc gdzie indziej.

### 4. Nie rozwlekaj
Correction pill ma byc krotki.
Jesli robi sie z niego pol nowego promptu, to znaczy, ze trzeba raczej dac nowy etap.

## Przyklad

```text
KOREKTA:
- Twierdziles: timeout enforcement nie istnieje
- Prawda: timeout enforcement istnieje w ledgerze i jest testowane
- Dowod: privai-ledger/src/escrow.rs + test reject_recovery_before_timeout
- Co sie zmienia: recovery_release przestaje byc greenfield enforcement taskiem; zostaje e2e closure task
- Kontynuuj od: sprawdz full-path behavior i backlog po tej korekcie
```

## Czego correction pill nie powinien robic

- nie powinien przepisywac calego backlogu od zera, jesli wystarczy zmienic 2 statusy
- nie powinien byc ogolnym "badz ostrozniejszy"
- nie powinien zostawiac modelu bez jasnego nastepnego kroku
