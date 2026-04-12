# T-059-GEMINI — Final Accepted Decisions Draft (Independent Review)

## 1. Verdict

Wyniki prac analitycznych dostarczają solidnego fundamentu pod dokumentację migracyjną, jednak wymagają stanowczego oczyszczenia z detali implementacyjnych. Paradygmat sieci prywatnych obliczeń (private compute network) w pełni zastępuje model "AI marketplace". Zgadzam się z koncepcją budowania nowych funkcji równolegle do istniejących (add new alongside old) w celu ochrony integralności obecnego systemu. Niemniej, propozycje zawierają przedwczesne próby definiowania formatów bitowych i typów przed ostatecznymi decyzjami Operatora, co musi zostać odrzucone, aby zapobiec powstawaniu technicznego długu architektonicznego. System nie zależy też od zgody fikcyjnego autorytetu (brak "Opus gate").

## 2. Accept Now

- **Zmiana Paradygmatu:** privAI V0 to post-kwantowa sieć prywatnych obliczeń, całkowicie odrzucająca model "marketplace".
- **Strategia Migracji:** Nowe funkcjonalności są dodawane obok starych (add new alongside old). Stary kod działa w niezmienionej formie.
- **Nienaruszalność `Escrow2of3`:** Istniejąca logika walidacji escrow pozostaje nienaruszona i służy jako bezpieczny pomost dla fazy 0/1.
- **Wykorzystanie `Amount14`:** Typ ten (u16) pozostaje ściśle ograniczony wyłącznie do ścieżki dowodów kryptograficznych (proof lane).
- **Kwarantanna Kodu Rynkowego:** Istniejące typy, takie jak `MarketplaceBatchTx` czy `MarketplaceSettlement`, nie będą usuwane natychmiast, lecz otagowane jako przestarzałe (`#[deprecated]`).
- **Baza dla Operatorless:** `RecoveryRelease` stanowi jedyny obecnie działający fundament, który nie wymaga podpisu operatora.

## 3. Accept Directionally Only

- **Nowy Wariant Escrow:** Utworzenie odrębnej polityki wydatków `ComputeLeaseEscrow` zamiast rozszerzania obecnej.
- **Pojęciowe Typy Domenowe:** `NetworkMode`, `SettlementMode`, `PrivacyClass` i `RoleType` na poziomie konceptualnym.
- **Model Rozliczeń:** Kierunkowa zgoda na koncepcję modelu "Pro-rata" dla rozliczeń (ale bez implementacji przed specyfikacją matematyki podziału).
- **Tożsamość Hierarchiczna:** Wprowadzenie "hidden root" oraz kluczy ról/sesyjnych, gdzie obecny klucz Falcon pełni semantycznie rolę `ValidatorRoleKey`.

## 4. Keep Open

- **Model Zaufania dla Mierzenia (Metering):** Ostateczny wybór pomiędzy systemem paragonów opartym na samo-raportowaniu z podpisami (self-reported) a mechanizmem wyzwanie-odpowiedź (challenge/response).
- **Granulacja Zasobów i Prywatności:** Poziom szczegółowości dla ofert obliczeniowych (`ResourceClass`) oraz widoczność środowiska uruchomieniowego dla górników (`Runtime Privacy Classes`).

## 5. Reject For Now

- **Przypisania Tagów Numerycznych:** Odrzucam jakiekolwiek przedwczesne przypisywanie numerów (np. `0x04` dla `ComputeLeaseEscrow` lub `ProRataSplit`).
- **Zamrażanie Typów Bazowych (`LedgerAmount = u64`):** Odrzucam zadeklarowanie tego w kodzie do czasu, aż Operator nie poda ostatecznej, sztywnej maksymalnej podaży (max supply) PVA.
- **Wczesne Struktury Implementacyjne:** Odrzucam definicje struktur takich jak `TargetRecipient::Two` przed powstaniem kompletnej specyfikacji matematycznej podziału środków.

## 6. Operator Decisions Still Needed

- **Maksymalna Podaż (Max Supply) PVA:** Absolutny bloker decydujący o użyciu typu `u64` względem `u128` do rozliczeń ekonomicznych.
- **Los Kodu Rynkowego:** Potwierdzenie dla rozpoczęcia oznaczania flagą `#[deprecated]` starych modułów rynkowych.
- **Kryteria Wyłączenia (Kill Criteria):** Reguły brzegowe (ilość miesięcy, liczba transakcji) dla Fazy 1 zautomatyzowanego operatora.
- **Rozwiązanie Kolizji Nazw:** Zmiana nazewnictwa "Fazy 1", aby uniknąć konfliktów pojęciowych między dokumentami operacyjnymi a etapami wdrożenia escrow.

## 7. Missing Direction / Spec Docs Still Needed

- `Operatorless Escrow Direction` (most między fazami 0/1/2, reguły weryfikacji dla zautomatyzowanego operatora).
- `Metering Protocol Direction` (struktura receiptów, mechanizm heartbeat, model zaufania).
- `Identity Model Direction` (ukryty korzeń, zasady derywacji kluczy, rola kluczy Falcon w nowym modelu, cykl życia epok).
- `Private Discovery Direction` (tryb odkrywania, węzły koordynujące, minimalizacja ujawnianych danych).
- `ComputeLeaseEscrow SpendPolicy Spec` (ostateczne pola, reguły walidacji, specyfikacja dzielenia dla wariantu pro-rata).
- `Protocol Versioning Direction` (podejście do zarządzania i ewolucji tagów numerycznych dla enumów).

## 8. Code Audits Still Needed

- Ocena ryzyka centralizacji w przypadku wykorzystania `nxms-escrow-orchestrator` jako maszyny stanu dla obsługi dowodów dostarczenia obliczeń (receiptów).
- Weryfikacja całkowitego odizolowania starych ścieżek transakcyjnych (marketplace) przed uruchomieniem weryfikacji dla ścieżki `ComputeLeaseEscrow`.

## 9. Where Xiaomi Still Overreaches

Raporty Xiaomi wykazują niebezpieczną tendencję do "implementation-forward design" – definiowania formatów na poziomie bitowym przed stworzeniem odpowiedniej architektury domenowej. Próba zamrażania typów bazowych (LedgerAmount bez wiedzy o max supply), natychmiastowe alokowanie tagów numerycznych dla protokołu (np. tag 0x04) oraz wymuszanie kształtu struktur w kodzie (TargetRecipient::Two) przed zatwierdzeniem matematyki rozliczeń stanowi naruszenie higieny architektonicznej. Takie zachowanie musi zostać całkowicie odrzucone jako tworzące dług technologiczny.

## 10. Minimal Canonical Decision Set

1. privAI V0 jest post-kwantową siecią prywatnych obliczeń; model "AI marketplace" zostaje oficjalnie porzucony.
2. Ewolucja kodu przebiega zgodnie ze strategią "add new alongside old", bez modyfikowania lub łamania istniejących ścieżek walidacji.
3. Walidator `Escrow2of3` jest wdrożony, zamrożony i posłuży jako fundament dla fazy pomostowej 0/1.
4. Ograniczony typ `Amount14` (u16) jest używany wyłącznie na ścieżce dowodów kryptograficznych.
5. Kod powiązany ze starym modelem rynkowym jest poddawany kwarantannie poprzez oznaczanie jako przestarzały (`#[deprecated]`), a nie fizycznie usuwany.
6. Tożsamość w konsensusie pozostaje oparta na kluczach, ale semantycznie klucz Falcon ulega przedefiniowaniu na `ValidatorRoleKey`.

## 11. Final Self-Check

- **Czy edytowałem kod?** Nie.
- **Czy edytowałem kanoniczne dokumenty?** Nie.
- **Czy definiowałem ostateczne formaty (wire formats)?** Nie, wyraźnie to zablokowałem.
- **Czy zamroziłem `u64` vs `u128`?** Nie, oznaczono jako zablokowane przez brak decyzji o max supply.
- **Czy zamroziłem numeryczne tagi enumów?** Nie, oznaczono jako błąd Xiaomi.
- **Czy twierdzę, że operatorless escrow jest zaimplementowane?** Nie.
- **Czy twierdzę, że pro-rata jest zaimplementowane?** Nie.
- **Czy twierdzę, że ukryta tożsamość (hidden-root identity) jest zaimplementowana?** Nie.
- **Czy twierdzę, że prywatne odkrywanie (private discovery) jest zaimplementowane?** Nie.
- **Czy zrzucam trudne decyzje na fikcyjny autorytet w przyszłości?** Nie, zidentyfikowałem konkretne role (Operator) oraz potrzebne dokumenty kierunkowe bez odwoływania się do modelu jako gatekeepera.
- **Czy lista decyzji jest zwięzła?** Tak.
