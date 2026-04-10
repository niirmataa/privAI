# Operator Cheatsheet — jedna strona

## Twoj workflow (4 kroki)

### 1. Nowa sesja z modelem
Powiedz:
> Przeczytaj PRIVAI_START_HERE.md i dokumenty z handoff. Jestes systemowym seniorem privAI.

Model zrobi reszte sam — ma pamiec globalną.

### 2. Model buduje backlog
Czekaj na output. Sprawdz TYLKO:
- czy sekcja "Unchecked assumptions" jest niepusta
- czy nie wrzuca future direction do current
- czy nie mowi "tego nie ma" bez sprawdzenia sasiedniej warstwy

Jesli cos smierdzi, napisz:
> Sprawdz to jeszcze raz. Correction pill.

### 3. Model daje Ci task pill
Sprawdz TYLKO:
- czy "Do not touch" jest wypelnione
- czy "Definition of Done" jest mierzalny (komenda, nie "looks correct")
- czy "What is already there" nie jest puste (znaczyloby ze model nie sprawdzil co juz istnieje)

Jesli OK — wyslij do Gemini / Xiaomi.

### 4. Gemini / Xiaomi zwraca wynik
Opcjonalnie wrzuc do GPT-4:
> Sprawdz czy ten output jest spojny z backlogiem i specami.

## Czerwone flagi (reaguj od razu)

| Widzisz | Zrob |
|---------|------|
| "tego nie ma w systemie" | Zapytaj: "w ktorym module szukales? sprawdziles ledger?" |
| "depends on X" przy kazdym tasku | Zapytaj: "czy to hard dependency czy assumption?" |
| Pusta sekcja "Unchecked" | Napisz: "co jeszcze nie zostalo sprawdzone? nie wierze ze nic." |
| "done" bez nazwy testu | Zapytaj: "jaki test to potwierdza?" |
| Task pill bez "Do not touch" | Napisz: "dodaj Do not touch zanim wysle to do codera." |
| Model kombinuje po bledzie | Napisz: "Correction pill. Co twierdziles, co jest prawda, dowod." |

## Routing — kogo do czego

| Co robisz | Komu daj |
|-----------|----------|
| Nowa sesja, backlog, re-entry | Opus |
| Konkretny task do zakodowania | Gemini |
| Sprawdzenie czy plan ma sens | GPT-4 |
| Prosty bounded task | Xiaomi |

## Dokumenty — nie musisz ich pamietac

Model czyta je sam. Ale jakbys potrzebowal:

| Dokument | Po co |
|----------|-------|
| PRIVAI_PROMPTYZACJA_SYSTEMOWEGO_SENIORA.md | Reguly gry |
| PRIVAI_SZABLON_REGUL_TWORZENIA_PROMPTOW_1_6.md | Sekwencja uczaca |
| PRIVAI_PROMPTY_UTWARDZAJACE_WEJSCIE_W_KOD.md | Zabezpieczenia |
| PRIVAI_MODEL_FIT_AND_ROUTING.md | Kto do czego |
| PRIVAI_OPERATOR_CHEATSHEET.md | Ta kartka |

## Jednozdaniowo

Ty nie musisz pamietac systemu. Ty musisz reagowac na 6 czerwonych flag
i routowac prace do wlasciwego modelu. Reszte pamieta model.
