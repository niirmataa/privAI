# 05. Privacy Tiers - Gemini Response

## 1. Recommended v0 design
Wprowadzono twardą macierz z 3 poziomami prywatności: `RecipientPrivacyLite` (lekkie, jawna kwota, public transfer bez operatora), `SmallPaymentsRail` (prywatny depozyt, operator, batching, ukrycie źródłowego portfela w sesji), oraz `FullPrivacy` (z ukrytą kwotą, relacjami i ciężkimi dowodami PQC ZK, dedykowane dla escrow). Wallet jako aktywny klient automatycznie dobiera odpowiedni tier oraz nakłada wektory eskalacji w oparciu o ustaloną `ServicePaymentPolicy`.

## 2. Alternative design
- 5 różnych poziomów prywatności, np. pół-jawny amount (Bucketed) jako osobny tier. Odrzucone - zbyt duże skomplikowanie UI i procesu decyzyjnego dla walleta.
- `RecipientPrivacyLite` używane dla escrow przy tanich przedmiotach. Zdecydowanie odrzucone - ryzyko wycieku metadanych sporów jest zbyt wysokie.

## 3. Privacy tier definitions
1. **RecipientPrivacyLite:** Brak publicznego, globalnego konta płatnika, lecz kwota i odbiorca (np. wprost wskazany) lądują on-chain. Dobre dla małego napiwku.
2. **SmallPaymentsRail:** Używa zaufanego depozytu i operatora marketplace. Operuje na `ticket_nullifier`. Kwoty wewnątrz sesji są ukrywane w batchu, lecz łączna wartość sesji on-chain w settlementcie jest jawna. Chroni wzór zachowań użytkownika.
3. **FullPrivacy:** Najwyższy tier, maskowanie odbiorcy, nadawcy, kwoty, kontekstu transakcji za pomocą not typu "One-time ReceiveBundle" i dowodów ZK. 

## 4. Tier selection matrix
- AI Inference/Usage: -> `SmallPaymentsRail`
- Mały tip dla twórcy: -> `RecipientPrivacyLite`
- Zakup przedmiotu fizycznego / Escrow: -> `FullPrivacy`
- Wypłata wynagrodzeń (Payout): -> `FullPrivacy`
- Flow o wysokiej wrażliwości na ujawnienie wolumenu: -> `FullPrivacy`

## 5. Wallet decision model
Wallet zaczytuje off-chain `ServicePaymentPolicy`. Zaczyna od próby wybrania najtańszego dopuszczalnego przez politykę formatu (np. `SmallPaymentsRail` na marketplace). Weryfikuje kontekst lokalny - jeśli sesja wykracza kwotą poza lokalny `spend_cap` bezpieczeństwa użytkownika lub posiada cechy sporu, bezwarunkowo podejmuje decyzję o odrzuceniu sesji (jeśli merchanta to nie obsługuje) lub wymuszeniu wejścia w `FullPrivacy`.

## 6. Escalation rules
- **Do FullPrivacy:** Występuje ZAWSZE, gdy usługa wymaga escrow, gdy wartość koszyka w `SmallPaymentsRail` przewyższa `requires_full_privacy_if`, lub gdy operator wskaże na rozpoczęcie sformalizowanego sporu.
- Opcji **"downgrade" (z FullPrivacy do Lite)** wbrew `ServicePaymentPolicy` nie ma.
- `RecipientPrivacyLite` i `SmallPaymentsRail` to ortogonalne tory – jeden do transferu p2p (bez operatora), drugi ściśle dedykowany pod Marketplace (z operatorem).

## 7. UX and communication model
UI Walleta jawnie różnicuje tryby graficznie (np. ikona tarczy).
- Dla `RecipientPrivacyLite`: Wallet informuje użytkownika "Kwota będzie widoczna na blockchainie".
- Dla `SmallPaymentsRail`: Wallet raportuje "Transakcja zbiorcza obsługiwana przez Marketplace Operator. Pełna prywatność konta zachowana."
- Użytkownik może wymusić na sobie `FullPrivacy` w ustawieniach zaawansowanych portfela per-transakcja, ale nie może tego zniżyć względem tego co wymaga merchant/policy.

## 8. Frozen decisions
- 3 główne tiery bez wersji "pośrednich".
- `SmallPaymentsRail` funkcjonuje WYŁĄCZNIE w obszarze obwarowanym operatorem.
- Brak osłabienia standardu kryptografii kwantowej (PQ) w ramach jakiegokolwiek tieru.
- Escrow i duże kwoty ZAWSZE lądują w `FullPrivacy`.

## 9. Open questions
- Ostateczny punkt odcięcia (np. w przeliczniku walut) dla pojęcia "mała kwota" będzie obsługiwany i limitowany off-chain przez logikę klienta (wallet UI), jednak sam łańcuch go nie wymusza twardo na konsensusie (jest w `requires_full_privacy_if` polityki usługi).
