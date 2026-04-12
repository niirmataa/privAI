# PrivAI — Simple 2: Jak wygląda system

**Opis możliwości, funkcje, problemy, zależności**

---

## Możliwości

System ma 11 warstw. Każda robi jedną rzeczy. Każda ma jasne granice — nie wchodzi w kompetencje innej.

---

## Warstwy

### 1. Użytkownik (portfel)

**Może:** szukać compute, blokować pieniądze, łączyć się z VM, odbierać dowody, kwestionować rozliczenie.

**Nie może:** widzieć co inni użytkownicy robią. Widzieć co miner robi z zasobem kiedy nie jest na sesji. Decydować o rozliczeniu — formuła jest na chainie.

**Funkcje:** budowanie transakcji escrow, wysyłanie zapytań discovery, odbieranie receipts, składanie roszczeń settlement.

**Problem:** Portfel musi obsługiwać nowy typ SpendPolicy (ComputeLeaseEscrow) — nie istnieje w obecnym kodzie.

**Zależy od:** kluczy użytkownika, chaina, transportu.

---

### 2. Tożsamość (klucze)

**Może:** generować klucze, podpisywać, weryfikować.

**Nie może:** ujawniać root credential. Łączyć ról (validator ≠ miner).

**Funkcje:** validator ma swój klucz (obecny Falcon PK). Miner ma osobny klucz. User ma klucze portfela. Każdy podpisuje swoje rzeczy.

**Problem:** Obecnie Falcon PK jest traktowany jako tożsamość. V0 mówi że to narzędzie podpisu. Semantyczna zmiana — zero zmiany w kodzie ale zmiana myślenia.

**Zależy od:** vault (przechowywanie kluczy).

---

### 3. Discovery (szukanie)

**Może:** user wysyła zapytanie, miner odpowiada, user dostaje listę ofert.

**Nie może:** ujawniać kim jest user. Ujawniać kim jest miner. Mieć publicznego rejestru.

**Funkcje:** zaszyfrowane zapytanie → skrzynka → minerowie próbują odszyfrować → pasujący odpowiada → user dostaje oferty.

**Problem:** Discovery protocol nie istnieje. NXMS mailbox jest transportem ale nie ma endpointu discovery.

**Zależy od:** NXMS mailbox, ComputeOffering struct.

---

### 4. Transport (skrzynka + telefon)

**Może:** wysyłać zaszyfrowane wiadomości (skrzynka). Łączyć się bezpośrednio (telefon przez Tor). Szyfrować FrodoKEM (post-kwantowo) + XChaCha20Poly1305 (szybko).

**Nie może:** widzieć treści wiadomości. Widzieć pełnej trasy (relay — future).

**Funkcje:** skrzynka przechowuje envelope'y i dostarcza. Telefon łączy się P2P przez Tor. FrodoKEM raz (lub per envelope), potem ChaCha.

**Problem:** Brak onion routing (relay). Brak metadata hardening. Ale to jest future — na teraz NXMS + Tor wystarczy.

**Zależy od:** kluczy FrodoKEM, kluczy Falcon, Tor SOCKS5.

---

### 5. Compute Miner (maszyna z GPU)

**Może:** provisionować VM/container, mierzyć zasoby, odpowiadać na challengi, produkować receipts, podpisywać je.

**Nie może:** widzieć workloadów usera (w VM). Osadzać zasobu (jeśli VM). Ujawniać innych userów.

**Funkcje:** odbiera escrow lock → odpala VM → mierzy availability i performance → odpowiada na challengi → tworzy receipt + ZK proof.

**Problem:** Cały compute miner moduł nie istnieje. Runtime provisioning, metering, receipt production — wszystko jest nowe.

**Zależy od:** kluczy minera, transportu, chaina (potwierdzenie escrow lock).

---

### 6. Metering / Okna (sprawdzanie)

**Może:** generować challengi, weryfikować odpowiedzi, liczyć PASS/FAIL, tworzyć agregat.

**Nie może:** ujawniać telemetry minera. Widzieć workloadów. Decydować o rozliczeniu (tylko liczy).

**Funkcje:** co okno: challenge → response → PASS/FAIL. Po sesji: aggregate receipt. W sporze: per-okno ZK proof.

**Problem:** Window protocol nie istnieje. Challenge generation, response verification, receipt aggregation — wszystko jest nowe. Ale format jest prosty: 6 pół w oknie, 6 pół w receipcie.

**Zależy od:** bloków (zegar), kluczy minera, lease policy.

---

### 7. Polityka Lease (warunki umowy)

**Może:** definiować klasę zasobu, cenę, czas, prywatność, tryb sieciowy, tryb rozliczenia, benchmark floor.

**Nie może:** zmieniać się w trakcie sesji. Być niejawna na chainie (jest hashed).

**Funkcje:** obie strony zgadzają się na warunki. Hash polityki jest na chainie. Settlement korzysta z polityki żeby obliczyć płatność.

**Problem:** ComputeLeasePolicy struct nie istnieje. Pola nie są zdefiniowane (required vs optional).

**Zależy od:** ResourceClass, PrivacyClass, NetworkMode, SettlementMode (wszystkie nowe).

---

### 8. Escrow / Rozliczenie (blokada i podział)

**Może:** blokować pieniądze, walidować podpisy, wykonywać podział (Release, Refund, Recovery, ProRata).

**Nie może:** widzieć workloadów. Widzieć receipts w normalnym flow (tylko hash). Decydować discretionary — formuła jest deterministyczna.

**Funkcje:** lock → session → receipt → settlement. Obecnie: Release (2 podpisy), Refund (2 podpisy), Recovery (2 podpisy, bez operatora). Docelowo: ProRata (1 podpis + protokół).

**Problem:** Nowy SpendPolicy variant (ComputeLeaseEscrow) nie istnieje. ProRataSplit nie istnieje. Ale mechanika execution istnieje — wystarczy dodać nowe variants.

**Zależy od:** polityki lease, receipts, kluczy.

---

### 9. Chain / Konsensus (zegar księgowy)

**Może:** pisać bloki, walidować transakcje, egzekwować nullifiers, dystrybuować nagrody.

**Nie może:** widzieć workloadów, promptów, outputów, modeli, profili providerów. Zmieniać konsensusu (PC-BFT zostaje).

**Funkcje:** blok co ~30 sekund. Transakcje w bloku (lub pusty housekeeping). Chain widzi commitmenty, nie treści.

**Problem:** Nic. Konsensus jest code-confirmed. V0 nie zmienia konsensusu.

**Zależy od:** validatorów, stake, Falcon podpisów.

---

### 10. Incentives (nagrody)

**Może:** płacić validatorom za bloki, minerom za compute, skrzynce za przechowywanie, relay za routowanie.

**Nie może:** płacić za jakość odpowiedzi AI. Płacić za reputację. Płacić za self-reported energy.

**Funkcje:** PVA jako waluta. Każdy role dostaje PVA za swoją pracę. Staking/bond jako zabezpieczenie.

**Problem:** Model incentywów nie jest zdefiniowany (ile PVA za co? jakie staking?). Ale to jest economic design, nie technical.

**Zależy od:** chaina, PVA supply, ról węzłów.

---

### 11. MCP / Kontekst agentów

**Może:** dostarczać V0 docs agentom, egzekwować poprawne odpowiedzi (golden tests), blokować legacy framing.

**Nie może:** edytować kodu. Mieć legacy docs w kontekście. Być source of truth dla implementation.

**Funkcje:** agenci (Xiaomi, Gemini, Codex) czytają z MCP. MCP mówi co jest canonical, co jest deprecated, co jest future. Golden tests wykrywają drift.

**Problem:** MCP nie jest zaimplementowany. Ale direction jest frozen.

**Zależy od:** V0 docs (muszą być kompletne).

---

## Zależności między warstwami

```
Użytkownik → Discovery → Transport → Miner
                ↓              ↓         ↓
             Chain ←——— Escrow ←——— Metering
                ↓              ↓
             Incentives ← Polityka Lease

MCP/Kontekst (niezależny — tylko dostarcza informacje)
```

**Kluczowa zasada: każda warstwa ma jasne granice.**
- Chain nie widzi workloadów
- Transport nie widzi treści
- Metering nie robi settlement
- Discovery nie ujawnia tożsamości
- Identity nie decyduje o zaufaniu
