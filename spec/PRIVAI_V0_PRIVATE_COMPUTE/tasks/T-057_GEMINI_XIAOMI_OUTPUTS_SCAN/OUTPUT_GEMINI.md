# T-057-GEMINI — Xiaomi Outputs Consistency Scan

## 1. Verdict

Wyniki zadań Xiaomi (T-046 do T-051 oraz T-054) wykazują silne i poprawne zrozumienie kierunku architektonicznego V0 (Private Compute Network zamiast modelu Marketplace). Xiaomi skutecznie izoluje przestarzały kod marketplace i prawidłowo identyfikuje strategię migracji "add new alongside old", w szczególności w zakresie granic między obecnym `Escrow2of3` a nowym `ComputeLeaseEscrow`.

Niemniej jednak, Xiaomi wykazuje zbytnie dążenie do definiowania szczegółów implementacji (too implementation-forward), proponując na przykład konkretne tagi numeryczne (np. `tag 0x04`) oraz zamrażając typy bazowe (np. `LedgerAmount = u64`) przed podjęciem niezbędnych decyzji przez Operatora lub ustaleniem specyfikacji. Ponadto wczesne raporty Xiaomi opierały się na traktowaniu modelu "Opus" jako jedynego autorytetu decyzyjnego, chociaż nowszy raport T-054 słusznie koryguje to na generyczny wymóg "decyzji recenzenta/kierunkowej". Podsumowując, propozycje Xiaomi są solidnym fundamentem dla ostatecznych kanonicznych dokumentów, pod warunkiem odrzucenia przedwczesnych detali implementacyjnych.

## 2. Stable Xiaomi Conclusions

| Conclusion | Appears In | Gemini Status | Notes |
|---|---|---|---|
| privAI V0 to sieć prywatnych obliczeń (private compute network), a nie AI marketplace | T-046, T-047, T-051, T-054 | FROZEN | Prawidłowe zrozumienie głównego modelu V0. |
| Strategia migracji "add new alongside old" dla escrow (`Escrow2of3` jako most, nowy `ComputeLeaseEscrow` obok) | T-046, T-047, T-054 | FROZEN_CANDIDATE | Minimalizuje ryzyko regresji dla obecnego systemu. |
| `Amount14` pozostaje tylko dla ścieżki dowodów (proof lane) | T-046, T-048, T-054 | STRONG_CANDIDATE | Oddziela złożoność dowodów kryptograficznych od prostej ekonomii ledgerowej. |
| Typy `MarketplaceBatchTx` oraz `MarketplaceSettlement` powinny zostać oznaczone jako przestarzałe (deprecated), nie usunięte natychmiast | T-046, T-049, T-054 | BLOCKED_BY_OPERATOR | Wymaga zgody operatora, ale kierunkowo bardzo bezpieczne. |
| Klucz `Falcon PK` reprezentuje roszczenie do roli Validatora (`ValidatorRoleKey`), a nie ostateczną tożsamość | T-046, T-047, T-048, T-054 | STRONG_CANDIDATE | Przejście tożsamości z warstwy podpisów do warstwy zarządzania hierarchicznego. |
| `RecoveryRelease` stanowi jedyny obecny i działający fundament pod operatorless escrow | T-046, T-047, T-054 | FROZEN_CANDIDATE | Weryfikacja w kodzie potwierdza, że nie wymaga podpisu operatora. |

## 3. Contradictions Or Tensions

| Topic | Xiaomi Output A | Xiaomi Output B | Gemini Resolution |
|---|---|---|---|
| Rola "Opusa" jako gatekeepera | **T-046, T-048, T-049:** Używają sformułowań "Opus must decide", "Blocked by Opus". | **T-054:** Koryguje to do "Requires direction/spec/reviewer decision", zaznaczając że Opus nie jest blokerem. | Sformułowania z T-054 są właściwe. System nie może czekać na konkretny model. Należy to traktować jako "missing direction". |
| Natychmiastowe wdrożenie typów koncepcyjnych | **T-046:** Proponuje "Decyzje do dodania teraz (14 typów)" wymuszając pisanie kodu. | **T-048:** Ogranicza listę do "Minimal Types" (10 typów), dzieląc na Fazę 0a i 0b. | Nawet T-048 jest momentami zbyt wybiegające w przyszłość. Typy koncepcyjne (enumy bez numerów, struktury bez pól) mogą być zatwierdzone kierunkowo, ale implementacja kodu musi czekać na specyfikację. |

## 4. Corrections From Codex/Gemini Reviews

| Topic | Xiaomi Claim | Review Correction | Status After Correction |
|---|---|---|---|
| Ustalenie `LedgerAmount = u64` | STRONG_CANDIDATE / dodaj teraz (T-046, T-048) | Zablokowane do momentu ustalenia przez Operatora maksymalnej podaży (max supply PVA). Należy oddzielić samą potrzebę posiadania aliasu od wyboru typu bazowego. | BLOCKED_BY_OPERATOR |
| Tagi numeryczne (np. `0x04`) | `SpendPolicyTag::ComputeLeaseEscrow = 0x04` jako zatwierdzone dodanie do kodu (T-048) | Konkretne tagi numeryczne muszą czekać na ujednolicony rejestr protokołu / wersjonowania. Nie zamrażać wartości numerycznych. | CANDIDATE (sam koncept, bez numeru) |
| Typy dla hierarchii tożsamości | Implementacja `HiddenRootCredential` oraz `RoleKey` w fazie 0b. | Koncept jest poprawny, ale należy opóźnić implementację struktury w kodzie do momentu przygotowania pełnego dokumentu `Identity Model Direction`. | CANDIDATE (tylko na poziomie domeny) |
| Wymóg zatwierdzenia przez Opus | Zablokowane przez "Opus" jako decydenta. | Należy traktować jako braki w dokumentacji technicznej, które może wypełnić jakikolwiek certyfikowany recenzent lub zespół. | BLOCKED_BY_DIRECTION_DECISION |

## 5. Freeze-Ready Inputs For Canonical Docs

- V0 to sieć prywatnego obliczeniowego AI (Private compute network), z pełnym odcięciem od modelu "Marketplace".
- `Escrow2of3` zostaje nietknięty jako faza pomostowa (Phase 0/1 bridge).
- `RecoveryRelease` pozostaje głównym istniejącym kotwiczeniem dla wdrożenia docelowego operatorless (brak wymogu zgody operatora).
- Węzły typu exit (Exit node) są opcjonalne (opt-in) i nigdy nie mogą stanowić wartości domyślnej, aby zapobiec wyciekowi prywatności.
- RAG/MCP jest ściśle związany z V0, co oznacza całkowitą kwarantannę dokumentacji legacy.
- `Amount14` nie reprezentuje ekonomii ledgerowej i pozostaje używany tylko dla ścieżki dowodów (proof lane).
- `Falcon PK` jest semantycznie traktowany jako `ValidatorRoleKey`.

## 6. Strong Candidates, Not Frozen

- Nowy wariant polityki wydatków `ComputeLeaseEscrow` (nie jest rozszerzeniem `Escrow2of3`, wymaga oddzielnej ścieżki walidacyjnej i nowego tagu).
- Enum `NetworkMode` z wartościami: Isolated, NxmsOnly, TorGated, InternetExit.
- Enum `SettlementMode` z wartościami: AllOrNothing, ProRata.
- Enum `RoleType` dla określenia ról tożsamości (Validator, ComputeMiner, Relay, Mailbox, ExitNode).
- Oznaczenie `MarketplaceBatchTx` i powiązanych struktur jako przestarzałych (`#[deprecated]`).
- Stworzenie wydzielonego typu `ComputeLeaseReceipt` oddzielonego od istniejącego `Receipt` (small payments).

## 7. Blocked Items

### Blocked by Operator Decision
- Całkowita podaż (max supply) dla PVA (określa u64 vs u128 dla rozliczeń na ledgerze).
- Ostateczny los logiki marketplace (tj. kiedy wdrożyć kroki wycofujące/deprecating).
- Kryteria awaryjne (kill criteria) dla zautomatyzowanego operatora (Phase 1).
- Rozwiązanie problemu nazewnictwa faz ("Phase 1" w dokumentach operacyjnych vs escrow).

### Blocked by Direction/Spec/Reviewer Decision
- `Operatorless Escrow Direction` (zasady weryfikacji dla automated operatora, phase bridge).
- `Metering Protocol Direction` (struktura receipt, bicie serca / heartbeat, model zaufania).
- `Identity Model Direction` (klucze ról, klucze sesji, tożsamość hidden root, epoki).
- `Private Discovery Direction` (model zapytań, scoped offering IDs, koordynator).
- Granulacja zasobów dla `ResourceClass` (np. warstwy GPU/CPU).
- Granulacja środowisk dla `Runtime Privacy Classes`.
- Specyfikacja dzielenia not dla wariantu `Pro-rata note split`.
- Kierunek zarządzania rejestrem i wersjonowaniem (Protocol Versioning) dla tagów numerycznych.

### Blocked by Code Audit
- Potwierdzenie, czy `nxms-escrow-orchestrator` może faktycznie posłużyć jako state machine dla weryfikacji receiptów dla zautomatyzowanego operatora, bez wprowadzania ryzyka centralizacji.

## 8. Too Implementation-Forward

- Uznanie `LedgerAmount = u64` jako faktu gotowego do kodu, zanim padła decyzja operatora o maksymalnej podaży waluty (max supply).
- Definiowanie konkretnych wartości numerycznych dla nowych wariantów enum (np. `SpendPolicyTag::ComputeLeaseEscrow = 0x04`, `EscrowAction::ProRataSplit = 0x04`) bez scentralizowanego rejestru formatów wiadomości / wersjonowania.
- Definiowanie na szybko struktur koncepcyjnych takich jak `TargetRecipient::Two` bez wiedzy jak dokładnie ma przebiegać walidacja splitu dla modelu pro-rata.
- Wymuszanie wdrażania struktury `HiddenRootCredential` podczas braku całościowego modelu tożsamości.

## 9. Obsolete Or Superseded Language

- **Opus-as-gatekeeper phrasing:** Raporty T-046 do T-050 stale wykorzystują określenia pokroju "Opus must decide", "Opus (spec)" czy "Blocked by Opus". W ujęciu docelowym, to sformułowanie jest zdezaktualizowane na rzecz "Missing direction/spec/reviewer decision", co zostało naprostowane w T-054.
- **References to non-existent tasks:** Wczesne raporty (T-046) odnosiły się do "T-032", "T-033", "T-035" jako zadań przypisanych dla Opusa, chociaż w rzeczywistości są to braki w dokumentach kierunkowych.
- **References to superseded outputs:** Propozycja natychmiastowego dodania 14 typów z T-046, która została wycofana w T-048 (10 typów) i ostatecznie zredukowana przez przegląd Codex (do samych koncepcji, z opóźnieniem dodawania kodu).

## 10. Legacy Marketplace Drift Check

Xiaomi przeszło testy dryfu ("drift check") z zadowalającym wynikiem. Poprawnie odseparowano i wyizolowano typy powiązane z modelem rynkowym (`MarketplaceBatchTx`, `MarketplaceSettlement`). Raport T-047 jasno formułuje zasady brzegowe odrzucające jakiekolwiek powiązanie logiki konsensusu czy escrow z rynkową analizą jakości zasobów (AI quality). Raport T-051 wprowadza serię zapytań bezpieczeństwa dla MCP/RAG (Golden Questions), które skutecznie ograniczają próby przywrócenia fraz typu "AI marketplace".

## 11. Recommended Codex Review Queue

1. `T-047_DOMAIN_BOUNDARIES_FREEZE/OUTPUT_XIAOMI.md` – Przegląd mający na celu zamrożenie granic architektonicznych między starymi a nowymi systemami.
2. `T-048_MINIMAL_TYPES_FREEZE/OUTPUT_XIAOMI.md` – Oczyszczenie z tagów numerycznych oraz wczesnych implementacji przed ostatecznym zatwierdzeniem na warstwie domenowej.
3. `T-049_IMPLEMENTATION_BLOCKERS/OUTPUT_XIAOMI.md` – Zatwierdzenie kluczowej ścieżki i priorytetyzacja tworzenia dokumentów kierunkowych.
4. Projektowanie kanonicznego dokumentu `PRIVAI_V0_FINAL_DOMAIN_AND_MIGRATION_DECISIONS_PL.md` na bazie zamrożonych koncepcji.

## 12. Recommended Next 5 Task Prompts

| Task ID suggestion | Target model | Purpose | Required input files | Expected output path |
|---|---|---|---|---|
| `T-058_OPERATOR_DECISIONS_COLLECTION` | Operator / Any Reviewer | Zebranie decyzji operatora w zakresie max supply dla PVA, kwarantanny marketplace oraz kill criteria dla Phase 1. | `T-049_IMPLEMENTATION_BLOCKERS/OUTPUT_XIAOMI.md`, `T-054_FINAL_REVIEWER_BRIEF/OUTPUT_XIAOMI.md` | `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-058_OPERATOR_DECISIONS_COLLECTION/OUTPUT.md` |
| `T-059_DRAFT_FINAL_MIGRATION_DECISIONS` | Codex | Opracowanie ostatecznego i kanonicznego dokumentu zatwierdzonych decyzji korygujących dla migracji na V0. | `OUTPUT_XIAOMI.md` (z zadań T-047, T-048, T-049, T-054) | `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_FINAL_DOMAIN_AND_MIGRATION_DECISIONS_PL.md` |
| `T-060_METERING_PROTOCOL_DIRECTION_DRAFT` | Gemini / Claude | Zaprojektowanie dokumentu kierunkowego dotyczącego obsługi receiptów, bicia serca (heartbeat) i modelu zaufania. | `PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md` | `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-060_METERING_PROTOCOL_DIRECTION_DRAFT/OUTPUT.md` |
| `T-061_IDENTITY_MODEL_DIRECTION_DRAFT` | Gemini / Claude | Opracowanie dokumentu kierunkowego dla modelu tożsamości (hidden root, klucze sesji/epok, role). | `PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md` | `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-061_IDENTITY_MODEL_DIRECTION_DRAFT/OUTPUT.md` |
| `T-062_OPERATORLESS_ESCROW_DIRECTION_DRAFT` | Gemini / Claude | Określenie mostu między Phase 0/1/2 oraz zautomatyzowanych reguł walidacyjnych dla operatorless escrow. | `PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md` | `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-062_OPERATORLESS_ESCROW_DIRECTION_DRAFT/OUTPUT.md` |

## 13. Final Self-Check

- **Czy edytowałem kod?** Nie.
- **Czy edytowałem kanoniczne dokumenty V0?** Nie.
- **Czy definiowałem konkretne formaty (wire formats)?** Nie, jednoznacznie odrzuciłem przedwczesne przypisywanie tagów numerycznych i typów w kodzie bez specyfikacji.
- **Czy traktuję wynik Xiaomi jako kanoniczny?** Nie, dokumenty Xiaomi posłużyły wyłącznie jako punkt wejściowy do syntezy i krytyki.
- **Czy traktuję wynik Gemini jako kanoniczny?** Nie, jest to dokument z przeglądu (review).
- **Czy traktuję Opus/Claude jako blokującego autorytetu?** Nie, wyraźnie oznaczono ten język z wcześniejszych raportów jako zdezaktualizowany.
- **Czy opierałem się na legacy docs?** Nie.
- **Czy zaglądałem do folderu zadania Vertex?** Nie.
- **Czy twierdzę, że operatorless escrow jest wdrożony?** Nie, stwierdzono wręcz coś przeciwnego.
- **Czy twierdzę, że pro-rata jest wdrożony?** Nie.
- **Czy twierdzę, że receipt truth jest rozwiązane?** Nie, jest to oznaczony bloker.
- **Czy twierdzę, że ukryta tożsamość (hidden root) jest wdrożona?** Nie.
- **Czy twierdzę, że private discovery jest wdrożone?** Nie.
- **Czy zatwierdziłem `u64` kontra `u128`?** Zamrożono tę decyzję jako wymagającą podania wprost przez Operatora.
- **Czy zamroziłem tagi numeryczne dla enumów?** Nie, zablokowano to w oczekiwaniu na specyfikację i mechanikę wersjonowania protokołu.
- **Czy odpowiedź znajduje się na poziomie architektonicznego przeglądu?** Tak.
