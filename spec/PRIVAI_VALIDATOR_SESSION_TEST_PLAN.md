# privAI Validator Session Test Plan

Status: active test plan for validator session transport hardening
Canonicality: local plan to drive integration and regression testing for `validator_session.rs`

## 1. Cel Testów

Zapewnienie stabilności i bezpieczeństwa warstwy transportowej po przejściu na handshake v2 (Challenge -> Init -> Response), dodaniu BanList, RateLimiter oraz HandshakeCooldown, a także po wprowadzeniu semantyki sekwencjonowania ramek (Anti-Replay).

## 2. Zakres Testów (Test Matrix)

Poniższe scenariusze powinny być zaimplementowane i utrzymywane w pliku `privai-node/tests/validator_session.rs`.

### 2.1. Handshake i Kryptografia
- **Sukces Handshake v2**: Pełen przepływ `Challenge -> Init -> Response` musi zakończyć się sukcesem dla prawidłowych kluczy.
- **Błędna Wersja Handshake**: Odrzucenie połączenia, jeśli klient wyśle `version != 1`.
- **Niezgodność Nonce (Replay Attack na Handshake)**: Odrzucenie, jeśli klient odeśle inny nonce niż ten wygenerowany przez serwer.
- **Błędny Podpis Falcon**: Odrzucenie połączenia, jeśli podpis klienta w `Init` nie weryfikuje się poprawnie (np. wygenerowany złym kluczem, zmodyfikowany payload).
- **Nieznany Peer**: Odrzucenie połączenia, jeśli `peer_id` klienta nie istnieje w `PeerBook`.
- **Mismatch Kluczy PeerBook**: Odrzucenie, jeśli dostarczone klucze nie zgadzają się z tymi w `PeerBook`.

### 2.2. Polisy Bezpieczeństwa (Anti-Spam / DoS)
- **Rate Limit Reject**: Odpalenie 6 równoległych połączeń z tego samego źródła (IP/Port) w oknie 60 sekund. Szóste połączenie musi zostać zablokowane.
- **Ban List Reject**: Wymuszenie wpisania peera na `BanList` i weryfikacja, że kolejne próby połączenia (zarówno przychodzące jak i wychodzące) kończą się natychmiastowym błędem.
- **Handshake Cooldown**: Symulacja 5 błędnych prób handshake z rzędu z tego samego adresu. Weryfikacja, czy kolejne próby w oknie 5 minut (300s) zostaną natychmiast odrzucone przed wykonaniem kosztownych operacji kryptograficznych.

### 2.3. Semantyka Ramek i Anti-Replay
- **Błędny Nonce w Ramce**: Próba odszyfrowania ramki po modyfikacji nonce musi zakończyć się błędem.
- **Modyfikacja Ciphertext / Tag**: Weryfikacja integralności AEAD.
- **Sequence Number Mismatch**: Próba wysłania duplikatu ramki (ten sam `seq` numer dwukrotnie) musi zostać odrzucona po stronie odbierającej.
- **Missing Shared Secret**: Upewnienie się, że `send_message` przed ukończeniem handshake na pewno zwróci błąd (brak fallbacku do plaintextu).

### 2.4. Lifecycle i Connection Pool
- **Stale Connection Rebuild**: Odczekanie `idle_timeout` lub `max_age` (można zamockować konfigurację) i weryfikacja, czy connection pool automatycznie ustawi flagę `needs_rebuild`.
- **Graceful Timeout**: Sprawdzenie zachowania dla wolnych odczytów w trakcie handshake (np. po 10 sekundach operacja powinna zgłosić timeout).

## 3. Wytyczne do Modyfikacji Testów

- Stare struktury `HandshakeMsg { version, kem_pk_b64 ... }` nie są już poprawne. Należy używać wariantu z enuma, a mianowicie sekwencji czytania i pisania `Challenge`, `Init`, `Response`.
- Dla ścieżek z timeoutami należy używać mockowania zegara lub drastycznie krótkich interwałów w dedykowanym `ConnectionPoolConfig`.

## 4. Raport Zgodności (Tracker)
- [x] Handshake success flow
- [x] Handshake version reject
- [x] Bad signature reject
- [x] Plaintext fallback removal validation
- [ ] Handshake cooldown test coverage
- [ ] Sequence anti-replay test coverage