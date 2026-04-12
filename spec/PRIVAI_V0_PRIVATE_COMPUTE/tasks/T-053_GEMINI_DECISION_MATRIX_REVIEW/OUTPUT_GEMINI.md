# T-053-GEMINI — Decision Matrix Cross-Review

## 1. Verdict

Decyzja matrix (T-046) przygotowana przez Xiaomi jest solidnym fundamentem i jest bezpieczna do użycia jako materiał wejściowy do kanonicznego dokumentu końcowych decyzji (`PRIVAI_V0_FINAL_DOMAIN_AND_MIGRATION_DECISIONS_PL.md`), o ile zostaną naniesione poprawki korygujące zbyt wczesne zaangażowanie w szczegóły implementacyjne. Matrix prawidłowo oddziela ustaloną architekturę (frozen direction) od kodu legacy oraz dobrze wyznacza fundament pod migrację (Escrow2of3, RecoveryRelease). Głównym błędem Xiaomi jest promowanie konkretnych typów bazowych (np. `u64`) oraz tagów numerycznych (np. `0x04`) przed zatwierdzeniem odpowiednich specyfikacji i braku decyzji Operatora o podaży, co może prowadzić do przedwczesnych zobowiązań implementacyjnych (overclaims). Co więcej, z racji braku Opusa, wszystkie "Opus decisions" muszą zostać przeklasyfikowane jako blokady z powodu braku specyfikacji (missing direction/spec/reviewer decision), które nie mogą trzymać procesu w martwym punkcie.

## 2. Status Corrections Table

Columns:
Decision | Xiaomi Status | Gemini Status | Why

| Decision | Xiaomi Status | Gemini Status | Why |
|---|---|---|---|
| **LedgerAmount = u64** | STRONG_CANDIDATE | BLOCKED_BY_OPERATOR | Typ jest konieczny, ale hardkodowanie `u64` jest zablokowane przez brak decyzji Operatora o maksymalnej podaży (max supply PVA). `u128` może być niezbędne. |
| **ComputeLeaseEscrow tag 0x04** | STRONG_CANDIDATE | STRONG_CANDIDATE (bez tagu) | Sama koncepcja nowego wariantu rozszerzającego jest bezpieczna, ale przypisanie numerycznego tagu `0x04` bez rejestru wersji protokołu jest przedwczesne. |
| **EscrowAction::ProRataSplit 0x04** | CANDIDATE | CANDIDATE (bez tagu) | Podział pro-rata to dobry kierunek, ale zamrażanie struktury lub stałych numerycznych przed specyfikacją jest błędem. |
| **HiddenRootCredential (add now)** | CANDIDATE | CANDIDATE (model only) | Dodanie do kodu jest przedwczesne. Należy zachować pojęcie w modelu (Identity Direction), ale nie implementować bez gotowej specyfikacji tożsamości. |
| **VM as default privacy class** | CANDIDATE | BLOCKED_BY_SPEC | Ustawienie VM jako domyślnej klasy to overclaim bez dokumentu definiującego szczegółowo prywatność i widoczność dla górników (Runtime Privacy Direction). |
| **Opus Decisions (all)** | Blocked by Opus | Blocked by missing spec | Opus nie jest instancją blokującą. Wszystkie jego "zadania" to po prostu brakujące dokumenty kierunkowe, które musi nadrobić zespół lub inny recenzent. |

## 3. Freeze-Ready Decisions

Decyzje gotowe do natychmiastowego zamrożenia (wynikają z niezmienników V0 lub istniejącego, przetestowanego kodu):
1. **V0 product model (private compute, not marketplace)** – potwierdzone przez dokumenty kierunkowe.
2. **Escrow2of3 as bridge (Phase 0/1)** – potwierdzone w kodzie, gotowe jako mechanizm pomostowy.
3. **RecoveryRelease as operatorless anchor** – kod udowadnia, że operacja ta nie wymaga Operatora i może stanowić punkt wyjścia dla operatorless.
4. **Amount14 proof lane only** – rozdzielenie obwodów udowadniających od ekonomii na głównym ledgerze, co potwierdził audyt.
5. **Exit node opt-in only** – wyłączenie domyślnego wyjścia do internetu to wymóg prywatności.
6. **V0-only MCP/RAG context** – izolacja kontekstu AI jest absolutnie krytyczna.
7. **Falcon PK as ValidatorRoleKey** – bezpieczna, wyłącznie semantyczna zmiana (dokumentacja/komentarze).
8. **small payments Receipt not reused directly** – wymusza czystą separację architektoniczną dla paragonów dzierżawy obliczeń (ComputeLeaseReceipt).

## 4. Strong Candidates, Not Frozen

Decyzje będące silnymi kandydatami, ale wymagające dalszych specyfikacji przed zamrożeniem w kodzie:
1. **Add-new-alongside-old strategy** – doskonała zasada, ale to wytyczna dla refaktoryzacji, a nie konkretna struktura do zamrożenia.
2. **LedgerAmount** – wyabstrahowanie typu to konieczność, lecz jego fizyczny rozmiar (bity) nie jest zamrożony.
3. **ComputeLeaseEscrow jako nowy wariant SpendPolicy** – podejście oparte o rozszerzenia jest prawidłowe, jednak kształt wewnętrznych struktur nie jest jeszcze zdefiniowany.
4. **NXMS mailbox jako baza transportowa dla discovery** – logiczny kandydat na warstwę transportową, lecz czeka na finalizację architektury discovery (Private Discovery Direction).

## 5. Blocked Decisions

Separate:
- **Blocked by Operator**
  - **Max supply PVA (u64 vs u128):** Blokuje wielkość typu `LedgerAmount` na głównym ledgerze.
  - **MarketplaceBatchTx & MarketplaceSettlement fate:** Operator musi zdecydować o czasie i formie wdrożenia `#[deprecated]` lub przeniesienia do modułu legacy, by nie zaszkodzić obecnej sieci.
  - **Kill criteria dla Phase 1:** Decyzja o tym, kiedy Phase 1 zostanie zakończona.

- **Blocked by direction/spec/reviewer decision**
  - **Operatorless Escrow Direction:** Blokuje logikę automatycznego zwrotu (TimeoutAutoRefund) i ostateczne odłączenie Operatora.
  - **Identity Model Direction:** Blokuje implementację `SessionKey`, `EpochKey`, `RoleKey`, oraz hierarchii tożsamości z `HiddenRootCredential`.
  - **Metering Protocol Direction:** Blokuje definicję jednostek bilingowych, formę wyzwań i odpowiedzi oraz `HeartbeatStatus`.
  - **Private Discovery Direction:** Blokuje typy danych dotyczące zapytań i ofert (`ScopedOfferingId`, `ComputeOffering`).
  - **Runtime Privacy Classes Direction:** Blokuje ostateczny kształt granulacji środowisk (VM) i obietnic gwarancji prywatności.
  - **ComputeLeaseEscrow & Pro-rata Note Split spec:** Blokuje szczegóły dzielenia not i zawartość specyficznych dla nich dowodów.

- **Blocked by Code Audit**
  - **nxms-escrow-orchestrator jako baza dla automatycznego operatora:** Wymaga dodatkowego audytu, aby udowodnić, że maszyna stanów może obsługiwać nowy mechanizm walidacji paragonów bez wprowadzania niebezpiecznej centralizacji.

## 6. Premature Implementation Commitments

Xiaomi wykazało zbytnie dążenie do definiowania szczegółów implementacji, które muszą zostać cofnięte:
- **14 typów "do dodania teraz":** Próba natychmiastowego dodania do kodu struktur takich jak `ComputeLeasePolicy`, `HiddenRootCredential`, czy wariantu `TargetRecipient::Two` jest przedwczesna. To są obszary koncepcyjne (candidate type surface), które muszą poczekać na zakończenie pisania specyfikacji (direction docs).
- **Zobowiązanie u64:** Narzucenie `u64` jako rozmiaru `LedgerAmount` podczas gdy brakuje zgody Operatora odnośnie całkowitej emisji (supply) waluty.
- **Stałe numeryczne dla enumów:** Ustalanie `0x04` dla `ComputeLeaseEscrow` czy `ProRataSplit` bez uprzedniego stworzenia scentralizowanego rejestru formatów wiadomości / protokołu wersjonowania.
- **Domyślna klasa prywatności (VM):** Brak precyzyjnej definicji, co widzi górnik, czyni ten punkt nadużyciem w gwarantowaniu prywatności (overclaim).

## 7. Missing Decisions Or Missing Blockers

- **Rejestr Wersjonowania / Namespace:** Matrix nie porusza kwestii tego, kto i w jaki sposób zarządza globalną pulą tagów numerycznych (dla typów akcji i polityk). Brakuje zablokowania tagów do czasu stworzenia rejestru.
- **Opus Fallback Mechanism:** Xiaomi potraktowało Opusa jako niezbędny punkt autoryzujący. System musi jednoznacznie określić, że "brak Opusa" oznacza po prostu "brakującą specyfikację", którą musi uzupełnić bieżący zespół projektowy/inny recenzent, aby projekt mógł iść do przodu.

## 8. Legacy / Marketplace Drift Check

- **Czy nastąpił powrót logiki Marketplace?** Nie. Matrix słusznie uznaje typy `MarketplaceBatchTx` i `MarketplaceSettlement` za przestarzałe, wymagające izolacji i wyklucza je z nowego modelu. V0 jest jednoznacznie systemem **Private Compute**, a nie **Marketplace**. Pomyślnie uniknięto przenikania nazewnictwa w nowo proponowanych elementach.

## 9. Recommended Next Task Order

Dla zapewnienia bezpiecznego przejścia od matrixa do kanonicznej dokumentacji (zadania dla Codexa):
1. **T-047 Domain Boundaries Freeze** – ostateczne zdefiniowanie barier izolujących nowy projekt od modułów oznaczonych jako legacy.
2. **T-048 Minimal Types Freeze** – zatwierdzenie absolutnego minimum typów i usunięcie z propozycji tych, które stanowią "premature implementation", włączając w to zablokowanie tagów numerycznych.
3. **T-049 Implementation Blockers** – formalne wylistowanie brakujących specyfikacji, całkowicie eliminując "Opusa" jako podmiot blokujący, a zastępując go wykazem brakujących dokumentów kierunkowych.
4. **Draft `PRIVAI_V0_FINAL_DOMAIN_AND_MIGRATION_DECISIONS_PL.md`** – na podstawie powyższych stworzenie dokumentu końcowego.

## 10. Final Self-Check

- **Czy edytowałem kod?** Nie.
- **Czy edytowałem kanoniczne dokumenty V0?** Nie.
- **Czy definiowałem konkretne formaty (wire formats)?** Nie, stanowczo odrzuciłem przedwczesne przypisywanie tagów numerycznych.
- **Czy traktuję wynik Xiaomi jako kanoniczny?** Nie, Matrix poddano silnej rewizji i obniżono status dla poszczególnych elementów.
- **Czy uznałem Opusa za bloker w autoryzacji?** Nie, wszystkie "Opus decisions" przekwalifikowano na "missing direction/spec".
- **Czy opierałem się na legacy docs?** Nie.
- **Czy twierdzę, że operatorless escrow jest wdrożony?** Nie, RecoveryRelease to jedynie mechanizm pozwalający w przyszłości na operatorless, a sam protokół to pieśń przyszłości (`future`).
- **Czy twierdzę, że pro-rata jest wdrożony?** Nie, wyraźnie zablokowano to brakiem specyfikacji.
- **Czy poparłem `u64`?** Odrzuciłem `u64` aż do decyzji Operatora odnośnie max supply.
- **Czy zatwierdziłem enum tags?** Odmówiłem zatwierdzenia numerów (np. `0x04`) do czasu powstania specyfikacji wersjonowania.
- **Czy poziom odpowiedzi to review, a nie instrukcja kodu?** Tak, dostarczono przegląd architektoniczny z wytycznymi.
