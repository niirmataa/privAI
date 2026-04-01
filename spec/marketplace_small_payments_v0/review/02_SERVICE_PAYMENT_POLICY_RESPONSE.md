# 02. Service Payment Policy - Gemini Response

## 1. Recommended v0 design
Domyślnym modelem v0 dla usług usage-metered (np. AI marketplace) jest `ReservationThenSettle`. Usługa (operator marketplace) wystawia jawny `SpendGrant` określający `spend_cap` oraz `settlement_window` na podstawie prywatnego depozytu użytkownika. W przypadku prostych płatności jednorazowych dopuszczalny jest opcjonalny model `ExactAmount`. Settlement przeprowadzany jest domyślnie przez marketplace operatora (trusted accounting).

## 2. Alternative design
Alternatywnym modelem jest "Exact per debit" bez rezerwacji dla każdego zakupu (odrzucony jako główny ze względu na słabą ergonomię dla usług usage-metered i dużą ilość mikro-transakcji na łańcuchu). Inny model to otwarty "UsageMetered" bez żadnych capów (odrzucony, zagraża użytkownikowi).

## 3. ServicePaymentPolicy data model
Minimalne i obowiązkowe pola dla ServicePaymentPolicy (jawne off-chain dla walleta):
- `policy_version`: Wersja polityki
- `merchant_commit`: Zobowiązanie merchanta
- `service_commit`: (Opcjonalne) Zobowiązanie usługi
- `allowed_rail`: Dopuszczalne tryby (np. `SmallPaymentsRail`, `FullPrivacy`)
- `pricing_mode`: `ReservationThenSettle` lub `ExactAmount`
- `reservation_mode`: Typ rezerwacji
- `min_deposit_required`: Minimalny depozyt
- `max_spend_per_session`: Maksymalny wydatek na sesję
- `max_spend_per_window`: Maksymalny wydatek na okno
- `grant_expiry_rule`: Czas życia grantu (sekundy)
- `settlement_window_rule`: Okno czasowe na settlement (sekundy)
- `requires_full_privacy_if`: Warunki wymuszające `FullPrivacy` (kwota progu)
- `policy_commit`: Hash kanoniczny polityki (wyliczany).

## 4. SpendGrant data model
Minimalny `SpendGrant` (wystawiany przez operatora po udowodnieniu depozytu, widoczny dla walleta i merchanta):
- `merchant_commit`: Powiązanie z merchantem
- `service_commit`: Powiązanie usługi (opcja)
- `session_scope`: Identyfikator sesji (off-chain)
- `spend_cap`: Twardy limit wydatków w ramach grantu
- `grant_expiry`: Kiedy grant wygasa
- `settlement_window`: Ostateczny czas na publikację batcha
- `policy_commit`: Skrót zaakceptowanej polityki
- `operator_sig`: Podpis marketplace operatora autoryzujący grant.

## 5. Pricing and reservation model
Domyślny tryb to `ReservationThenSettle`. `Amount` na v0 jest dokładny (jawny), aby uprościć audytowalność przez operatora i łańcuch, a prywatność zachowana jest przez agregację w batchach oraz ukrycie źródłowego konta płatnika. Wallet z góry rezerwuje pulę w ramach `SpendGrant`, a settlement rozlicza tylko realne użycie na podstawie przedstawionych receipts. Wallet odrzuca przepływ bez twardych limitów.

## 6. Refund and timeout model
Zwroty są regulowane przez zasady zdefiniowane w polityce. W domyślnym v0, jeśli operator nie opublikuje settlementu przed wygaśnięciem `settlement_window`, depozyt automatycznie odzyskuje uwolnienie po stronie *operator-trusted accounting* (`timeout_refund`). Spory co do realizacji usługi rozliczane są off-chain i uwzględniane przez zwrot przy agregacji batchu (`partial_refund_allowed`).

## 7. Batching and settlement authority model
Settlement jest publikowany **tylko przez marketplace operatora** (v0 default). Merchant tworzy receipt, ale przekazuje go operatorowi. Batching zależy od `merchant_commit` oraz `settlement_window_rule`. Sieć konsensusu polega na podpisie autoryzacyjnym od operatora by spalić `ticket_nullifier`.

## 8. Wallet decision model
Wallet bierze pod uwagę `ServicePaymentPolicy` i sam weryfikuje flow. Jeśli dozwolony rail mówi `SmallPaymentsRail`, ale kwota sesji lub żądanie jest większe od `requires_full_privacy_if`, portfel asertywnie odrzuca żądanie i wymaga przejścia na `FullPrivacy`. Wybór lekkiego raila musi mieścić się w ściśle określonych granicach budżetowych w `max_spend_per_session`.

## 9. Frozen decisions
- Domyślny model to `ReservationThenSettle` dla marketplace drobnicy.
- Konieczny twardy `spend_cap` i `settlement_window` w `SpendGrant`.
- Operator samodzielnie publikuje settlement (`marketplace-operator-trusted accounting`).
- Wymagane są precyzyjne zasady eskalacji na podstawie limitów, w przypadku przekroczenia wallet narzuca pełną ścieżkę prywatności.

## 10. Open questions
Brak. Struktury i schematy serializacji są wystarczające do zbudowania modułów `operator.rs` i `small_payments.rs` w portfelu i łańcuchu.