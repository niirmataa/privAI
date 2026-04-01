# Nexum CLI & privAI Integration Proposal

Dokument ten jest technicznym pomostem między istniejącym frontendowym narzędziem C (`nexum-cli`) a backendowym jądrem napisanym w języku Rust (`privAI` / `privai-wallet` / `privai-chain`). Stanowi zarys integracji, która połączy potężne procesy offline z `nexum-cli` z siecią NXMS i modelem UTXO oraz Small Payments Rail.

---

## 1. Rola narzędzia `nexum-cli`
Z mojej analizy wynika, że narzędzie to jest pierwszą linią frontu użytkownika (User Client) do operowania z siecią, a nie tylko narzędziem pomocniczym. Jest to niezwykle skuteczna aplikacja, która odciąża serwer ze skomplikowanych i zasobochłonnych operacji:
- Rozwiązywanie układów Proof-of-Work (`pow.c`).
- Zarządzanie kluczami i kryptografią w bezpiecznym magazynie (`vault.c`).
- Operowanie szyfrowaniem PQC na własnej maszynie przez Falcon/FrodoKEM (`pqc_falcon.c`, `pqc_kem.c`).

### Aktualny zakres
Zgodnie z Twoją intencją, w `nexum-cli` **server służy jako marketplace**, w którym challenge oparty na PoW jest realizowany przy rejestracji oraz logowaniu, aby zapewnić, że tylko uczciwy uczestnik, który spali energię (i odstraszy tym ataki typu bot-spam) może podłączyć się do rynku, lub uderzyć w API serwerowe.

## 2. Plan Integracji (C-FFI -> Rust Wallet)

Kiedy wrócimy do tematu łączenia obu światów (C + Rust), proponuję by nie dublować logiki po obu stronach (np. od nowa pisać pow.c w Rust dla użytkowników), ale by wystawić tzw. `C-ABI / FFI Wrapper` z `privai-wallet` dla programu w C, oraz by program w C działał jak "Master Process" sterujący silnikiem Rustowym.

### 2.1. Rejestracja, Logowanie i Serwer (PoW / auth.c)
Proof-of-Work to naturalne narzędzie Rate-Limitingu w środowisku P2P oraz klient/serwer:
- Przy operacjach krytycznych (takich jak generacja setek `ReceiveBundle` i wysyłka ich do `nxms-mailbox`), serwer `privAI` może żądać dodatkowego pakietu Proof-of-Work w ramach tokenu autoryzacyjnego. W ten sposób zablokujemy napastnika, który chciałby przepełnić nam bazę danych setkami tysięcy wpisów. To jest natywnie wspierane przez Twoje `pow.c`.

### 2.2. Kryptografia PQC (vault.c, pqc_falcon.c, pqc_kem.c)
Narzędzie w języku C trzyma klucze. Zamiast budować nowe magazyny kluczy po stronie Rusta i zmuszać użytkownika do migracji:
1. `nexum-cli` otwiera swoją skrytkę z kluczami (`T_FALCON_SK`, `T_FALCON_PK`).
2. Przy autoryzacji transakcji on-chain (takich jak wydanie `OutputNote` / transferu z wykorzystaniem UTXO w warstwie Rusta), silnik `privai-wallet` generuje *hash transakcji (statement_commit)*.
3. Silnik Rusta pyta `nexum-cli`: "Podpisz ten komunikat".
4. Kod z `pqc_falcon.c` składa post-kwantowy podpis i odsyła do przestrzeni Rusta.
W ten sposób Rust jest odcięty od konieczności "znajomości" kluczy dewelopera/użytkownika (tzw. air-gapping).

### 2.3. Prekeys a ReceiveBundles (prekeys.c)
Plik `prekeys.c` to gotowy mechanizm na generowanie pakietów odbiorczych. Użytkownik wykorzysta go do dystrybuowania publicznego wektora kluczy `ReceiveBundle` (w oparciu o FrodoKEM) przez `NXMS/1` albo do umieszczenia go prosto w Marketplace.

### 2.4. Integracja z rynkiem Small Payments Rail (Operator)
Twój serwer orkiestracji (Marketplace Operator) będzie weryfikował zgłoszenia i w pełni współpracował z logowaniem za pomocą challengów (które de facto dają operatorowi dowód "Kto jest kim" przed wystawieniem `SpendGrant`u na depozycie). Klient w C upewni się, że każda iteracja zgłaszania mikropłatności / ticket_nullifier jest poświadczona, albo loguje się poprzez żądanie z podpisanym tokenem Challenge, eliminując luki po stronie serwera API.

---

## 3. Kolejne zadania (gdy do tego wrócimy)

Gdy wrócimy do implementacji Frontu (CLI), rekomendowane kroki to:
- [ ] Utworzyć interfejs w C (nagłówek `.h` oraz build FFI w Rust Cargo) do komunikowania się ze stanem i pamięcią RAM załadowanego portfela `privai-wallet`.
- [ ] Dopiąć polecenie `nexum-cli pay` wywołujące generację `MarketplaceBatchTx` (lub `LocalTicket`) ze `small_payments_rail.rs`.
- [ ] Połączyć autoryzację z rejestracji i login challenge w pełny protokół zapytania o model poprzez nowo utworzone `PRIVAI/1` (InferenceRequest).
- [ ] Zintegrować podpisywanie kluczem (Signature Operations) dla szerokiego zakresu przyszłych decyzji (głosowanie on-chain, zmiany statusów modelu w rejestrze) za pomocą istniejącego challenge/response na podstawie podpisu.