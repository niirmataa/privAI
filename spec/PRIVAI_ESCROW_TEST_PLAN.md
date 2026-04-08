# PRIVAI ESCROW TEST PLAN

## Status
DRAFT - Dokument planistyczny; określa specyfikację weryfikacji i docelowe wektory, lecz nie odzwierciedla w pełni istniejącej implementacji w kodzie.

## Canonicality
Jest to dokument wspierający (support/test-plan doc). Nie stanowi ostatecznego źródła prawdy w kontekście semantyki protokołu, lecz jedynie wytycza wektory weryfikacji na podstawie nadrzędnych specyfikacji. Prawdziwa logika działania zawarta jest w plikach modelu docelowego.

## Owner
privAI QA & Architecture Team

## Depends on
- `PRIVAI_ESCROW_FINAL_MODEL.md`
- `PRIVAI_ESCROW_TX_MATRIX.md`
- `PRIVAI_ESCROW_PROOF_INTEGRATION.md`
- `PRIVAI_AUTH_SIGNING_MODEL.md`
- `PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`
- `PRIVAI_EXECUTION_SPINE.md`

## Cel
Przełożenie teoretycznego modelu Escrow na konkretny, iteracyjny plan testów. Celem jest zdefiniowanie, w jaki sposób będziemy weryfikować poprawność rygorystycznych polityk escrow i auth w kodzie on-chain. Dokument ten nie zawiera samego kodu testów, a jedynie nakreśla listę niezbędnych scenariuszy wykonawczych.

## Test philosophy
- **Brak mieszania z marketplace:** Testujemy wyłącznie rdzenną warstwę protokołu sieciowego (escrow note-based), bez ułatwień (convenience rail) i ustępstw obecnych w środowisku marketplace v0.
- **Kategoryzacja stanu:** Każdy scenariusz posiada oznaczenie przynależności do puli:
  - `[stable tests]` - scenariusze gotowe do walidacji na obecnej architekturze i logice polityk v1,
  - `[blocked tests]` - testy zablokowane przez braki w sąsiadujących komponentach (np. full locktime timeout dla recovery, integracja weryfikatorów ZKP PQ).
- **Brak overclaimingu:** Dokument odzwierciedla plany na testy. Nie udajemy, że pełne zabezpieczenia PQ (Post-Quantum) czy timer-locked proofy są domknięte i w 100% otestowane, jeśli w rzeczywistości stanowią blokadę.

## Positive scenarios
Scenariusze weryfikujące poprawne zachowanie dla prawidłowych danych wejściowych (Happy path).
- **valid EscrowFund:** Weryfikacja prawidłowego zasilenia (lock) not w escrow z odpowiednimi politykami i ujęciem ról (Buyer, Merchant, Operator). `[stable test]`
- **valid ReleaseToMerchant:** Uwolnienie środków (Action: Release) do Sprzedawcy na podstawie prawidłowej pary podpisów `Buyer + Operator`. `[stable test]`
- **valid RefundToBuyer:** Zwrot środków (Action: Refund) do Kupującego po skompletowaniu zestawu podpisów `Merchant + Operator`. `[stable test]`

## Negative scenarios
Scenariusze sprawdzające uodpornienie systemu na nieprawidłowe dane, ataki i błędy autoryzacji.
- **reject unsupported policy family/version at funding:** Odrzucenie operacji wpłaty środków, ujętych w nieznaną, zdeprecjonowaną lub nienależącą do dopuszczalnych wariantów `policy_family` lub dla nieobsługiwanej wersji tejże polityki. `[stable test]`
- **reject release with wrong signer set:** Odrzucenie transakcji Release, jeżeli podpisy pochodzą od nieautoryzowanych w tej akcji podmiotów (np. podpis składa sam Merchant i Operator na ReleaseToMerchant) lub w ogóle brakuje sygnatury w Normal mode. `[stable test]`
- **reject refund with wrong signer set:** Odrzucenie transakcji Refund, gdy w jej realizację angażuje się niewłaściwy uczestnik, np. zgadza się sam Buyer z Operatorem, lub gdy pod obostrzenie podpięty jest sygnatariusz bez wymaganych uprawnień do danej akcji. `[stable test]`
- **reject duplicate signer:** Odrzucenie operacji uwalniania środków, gdy dostarczono dwa identyczne podpisy tej samej osoby podszywającej się pod unikalnych sygnatariuszy autoryzacji. `[stable test]`
- **reject wrong action type for signed tx_signing_hash:** Odrzucenie akcji z powodu niezgodności celu w podpisie – np. podjęto próbę użycia podpisu zebranego oryginalnie dla akcji `Refund` do zainicjowania akcji `Release`. `[stable test]`

## Recovery scenarios
Scenariusze przewidziane na wypadek awarii, wykluczenia lub braku responsywności Operatora. Timeout implementowany jest ledger-side w docelowym modelu v1 (mixed escrow model).
- **valid RecoveryRelease after timeout:** Pomyślne zwolnienie środków (lub zwrot) po upływie zadanego pułapu czasu (on-chain timeout verification), przy ominięciu Operatora – akcję zatwierdzają `Buyer + Merchant`. `[blocked test]` (do momentu spięcia pełnej egzekucji timeoutów on-chain ledger w ramach weryfikatora polityki noty)
- **reject recovery before timeout:** Odrzucenie próby zignorowania i ominięcia Operatora poprzez rzekomo awaryjne rozwiązanie (z autoryzacją `Buyer + Merchant`) przed formalnym upływem wymaganego okresu karencji. `[blocked test]`

## Proof/auth split scenarios
Rozdzielenie i niezależna walidacja matematycznego dowodu wiedzy zerowej od weryfikacji artefaktów autoryzacyjnych. Model v1 stabilizuje tę separację na weryfikatorze on-chain.
- **reject correct proof with bad auth:** Weryfikator ledger-side odrzuca transakcję posiadającą wprawdzie prawidłowo wyliczony matematyczny dowód ZKP, jeśli towarzyszą jej błędne lub zmanipulowane artefakty auth (np. odrzucane podpisy, fałszywe lub brakujące sygnatury kluczy autoryzujących złą akcję). `[stable test]`
- **reject correct auth with bad proof:** Weryfikator ledger-side odrzuca transakcję posiadającą poprawne i nienaruszone podpisy wszystkich kluczowych uczestników, z powodu przedstawienia matematycznie wadliwego lub naruszającego reguły wariantowe dowodu zerowej wiedzy. `[stable test]`

## Policy reconstruction scenarios
Weryfikacja poprawności mechanizmów odsłaniania warunków polityki transakcyjnej dla zablokowanej noty.
- **reject wrong policy_opening:** Odrzucenie transakcji uwalniającej środki w sytuacji, gdy strona dostarczająca sygnał wykonawczy przedstawia błędne, zmodyfikowane lub niezgodne z publicznym hash-commitmentem parametry odsłaniania skryptu transakcji. `[stable test]`

## Timeout scenarios
Sprawdzenie mechaniki opóźnień czasowych na styku polityk escrow z zegarem ledgera.
- Odrzucenie żądań wymuszających Recovery Mode z ominięciem Operatora przed ostatecznym osiągnięciem wymaganej wysokości bloków lub określonego znacznika locktime on-chain dla zwolnienia awaryjnego. `[blocked test]` (stan zablokowany do wdrożenia locktime clock validation)

## Known blocked areas
Obszary tymczasowo wykluczone z głównego obiegu testów akceptacyjnych ze względu na uwarunkowania deweloperskie poza samym kodem autoryzacji:
- **On-chain locktime enforcement:** Precyzyjne odmierzanie timeoutu w trybie recovery musi opierać się o parametry blokowe ledgera (wysokość/timestamp), których pełne zablokowanie (wsparcie locktime on-chain) nie zostało zapięte w warstwie walidacji dowodowej.
- **Full PQ Privacy verification:** Pełne gwarancje izolacji kwantowej wokół dowodów i operacji na notach escrow zostają wstrzymane dla planów długoterminowych. Czekamy na certyfikację oraz odblokowanie weryfikatorów KEM.

## Checklist
Zadania wykonawcze, czyli mapowanie poszczególnych zaplanowanych pul testów na pliki weryfikacyjne w codebase:
- [ ] Utworzenie dedykowanych helperów i stubów e2e reprezentujących perspektywy ról `Buyer`, `Merchant`, `Operator`.
- [ ] Zaimplementowanie testów dla warstwy wpłat z rygorystycznym sprawdzaniem obostrzeń na `policy_family` (Negative scenario: reject unsupported policy at funding).
- [ ] Zaimplementowanie testów uwalniania z logiką puli `Positive scenarios` (valid Release dla Buyer+Operator; valid Refund dla Merchant+Operator).
- [ ] Zaimplementowanie pełnej weryfikacji podmiany podpisów w `Negative scenarios` (w tym łamania `tx_signing_hash`).
- [ ] Zaprojektowanie i wdrożenie testów weryfikujących poprawność izolacji dla `Proof/auth split` (skuteczne i niezależne odrzucanie złego proof przy dobrym auth oraz na odwrót).
- [ ] Opracowanie tymczasowych mocków opóźnień (block-height bypass) przed odblokowaniem zadania z wdrożeniem logiki `Recovery scenarios`.

## Exit criteria
Kryteria zamknięcia, decydujące o wejściu mechanizmu Escrow do wdrożenia protokolarnego:
- Wszystkie wektory testowe posiadające flagę `[stable test]` w dokumencie muszą zostać skatalogowane, napisane, dodane do puli Continuous Integration (CI) z wynikiem pass.
- Testy pokrywają oryginalną, bezpośrednią logikę silnika transakcyjnego privAI ledger (zero zapożyczeń lub ułatwień pochodzących z webowego mocka z environmentu marketplace v0).
- Poszczególne etapy wykreślone z zakresu testowego i oznaczone jako `[blocked test]` otrzymują wygenerowane follow-upy integracyjne/taskowe w trackerze w celu ich ostatecznego dopięcia do testów on-chain.