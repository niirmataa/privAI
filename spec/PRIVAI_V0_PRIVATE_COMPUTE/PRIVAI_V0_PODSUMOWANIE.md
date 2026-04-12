# privAI V0 — Podsumowanie Systemu

**Data:** 2026-04-12
**Status:** Dokument podsumowujacy na podstawie kodu i dokumentacji z folderu `PRIVAI_V0_PRIVATE_COMPUTE`
**Zrodla:** PRIVAI_V0_ARCHITECTURE_SPEC.md, PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md, PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md, PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md, PRIVAI_V0_DIAGRAMS.md, PRIVAI_V0_DOCS_TREE.md, PRIVAI_V0_SIMPLE_SYSTEM_DESCRIPTION_PL.md, PRIVAI_V0_WINDOW_PROTOCOL_DESIGN_QUESTIONS.md, przemyslenia/*

---

## 1. Czym jest privAI

privAI to **post-kwantowa siec prywatnego obliczeniowego AI**. Nie jest marketplace'em modeli AI. Nie jest platforma handlu artefaktami. Jest infrastruktura prywatnego dostepu do compute.

```
Prywatnosc jest produktem.
Compute jest podaza.
PVA jest motywacja.
Chain jest rozliczeniem.
Transport jest tarcza.
```

Uzytkownik prywatnie wynajmuje izolowany runtime (VM, container, sandbox, GPU slice) od compute minera i placi w PVA przez FullPrivacy escrow na chainie. Nikt — ani chain, ani miner, ani operator — nie widzi co uzytkownik oblicza.

---

## 2. Role w sieci

System wymusza scisla separacje odpowiedzialnosci. Jedna maszyna moze pelnic wiele rol, ale protokol traktuje je osobno:

| Rola | Funkcja | Nagroda |
|------|---------|---------|
| **Validator** | Zabezpiecza chain, pisze bloki, przetwarza transakcje | Nagrody blokowe + fee |
| **Compute Miner** | Dostarcza izolowane zasoby obliczeniowe (GPU/CPU/RAM), produkuje receipty | PVA za dostarczony compute |
| **Mailbox (NXMS)** | Przechowuje zaszyfrowane koperty, dostarcza je asynchronicznie | PVA za storage |
| **Relay** | Przekierowuje zaszyfrowany ruch — widzi tylko poprzedni/nastepny hop | PVA za routing (future) |
| **Exit Node** | Opcjonalny dostep do internetu — jawny opt-in, nigdy domyslny | PVA za egress (future, wyzsze ryzyko) |

### Tozsamosc (Phase 0-5)

Tozsamosc opiera sie na **dwoch niezaleznych kluczach Falcon**:
- **Validator Key:** z vault, frozen jako `node_pk_hash`, uzywany do konsensusu
- **Compute Miner Key:** generowany osobno, uzywany do podpisywania receiptow i lease claims

Falcon jest narzedzie podpisu, nie publiczna tozsamosc. Zadne polaczenie miedzy kluczami na poziomie protokolu. Hidden root credential jest odlozone do Phase 6+.

---

## 3. Cykl zycia sesji compute

```
1. Discovery (off-chain)
   Uzytkownik wysyla zaszyfrowane zapytanie przez NXMS mailbox:
   "Szukam A100, 80GB VRAM, max 2 PVA/h"
   Skrzynka nie widzi tresci. Minerowie probuja odszyfrowac.
   Pasujacy odpowiadaja ComputeOffering.

2. Negocjacja (off-chain)
   Uzytkownik i miner uzgadniaja warunki: klasa zasobu, cena, czas, prywatnosc, tryb sieciowy.
   Warunki sa zapisane w ComputeLeasePolicy.

3. Blokada (on-chain)
   Uzytkownik blokuje PVA na chainie w ComputeLeaseEscrow (tag 0x04).
   Chain zapisuje: kwota + hash polityki + timeout.
   Pieniadze sa zamrozone do konca sesji.

4. Uruchomienie (off-chain)
   Miner odpala VM/container/sandbox.
   Wysyla zaszyfrowane dane dostepu.

5. Sesja (off-chain)
   Uzytkownik laczy sie przez Tor (FrodoKEM handshake → XChaCha20Poly1305 streaming).
   Nikt nie widzi promptow, outputow ani danych.

6. Metering — Window Protocol (off-chain)
   Co 60 blokow (~30 minut):
     - Chain generuje blok (nieprzewidywalny block_hash)
     - Uzytkownik tworzy challenge: hash(session_id || window_id || block_hash)
     - Miner odpowiada (musi miec prawdziwy zasob zeby zdazyc)
     - Uzytkownik sprawdza: odpowiedzial na czas? → PASS, za wolno? → FAIL
   Dodatkowo co N okien: benchmark performance (czy zasob jest wystarczajaco szybki).

7. Receipt (off-chain → on-chain)
   Miner tworzy agregat: {total_windows, passed_windows, degraded_windows}
   + ZK proof (dowod krypto ze telemetry jest konsystentna z receiptem)
   + podpis minera

8. Rozliczenie (on-chain)
   Chain oblicza:
     effective = passed + (degraded * weight / 1000)
     miner_share = amount * effective / total
     user_share = amount - miner_share
   Reszta zawsze do uzytkownika. Zero floatow. Arytmetyka calkowita.

9. Spór (on-chain, opcjonalny)
   Jesli uzytkownik kwestionuje receipt:
     - Miner musi przedstawic ZK proof per okno
     - Chain weryfikuje
     - Kto przegrywa — placi oplate za spor
```

---

## 4. Window-Based Metering — protokol pomiarowy

Miare dokladnego czasu GPU (FLOPS) na wspoldzielonym/zirtualizowanym GPU jest technicznie niewykonalne. V0 rozwiazuje to przez **Challenge-Sampled Proof of Resource Possession**.

### Kluczowe decyzje (36 decyzji protokolu):

| Kategoria | Decyzja |
|-----------|---------|
| **Window ID** | Sekwencyjny `u32` (1..N). Session_id chroni prywatnosc. |
| **Czas trwania okna** | 60 blokow (~30 minut). Stale dla calej sesji. |
| **Okna** | Contiguous (nachodzace Phase 1, overlapping future). |
| **Typ okien** | Fixed-time (deterministyczne). Event-driven jako opcjonalny upgrade. |
| **Minimum okien** | 100 (statystycznie znaczace). |
| **Hash challenga** | `blake3("privai:window:v1" || session_id || window_id || block_hash)` |
| **Timeout** | Lease policy deklaruje. DedicatedGpu: 10 blokow, SharedGpu: 20 blokow. |
| **Availability** | Binary PASS/FAIL. Definicja = odpowiedz na challenge w timeout. |
| **Performance** | Benchmark per klase zasobu. Lease policy deklaruje benchmark_floor. |
| **Receipt** | Unilateral (miner tworzy + ZK proof). User ma swoje dane ale nie tworzy formalnego receiptu. |
| **ZK proof na receipcie** | Agregat (suma passed jest prawdziwa). Per-window tylko w sporze. |
| **ZK proof na chainie** | Off-chain. Hash na chainie. Pelen proof na zadanie. |
| **Settlement formula** | `miner_share = amount * (passed + degraded * weight) / total`. Integer division. Reszta do usera. |
| **Degraded weight** | Default 50% (0.5). Lease policy moze deklarowac inna. |

---

## 5. Escrow i Settlement

### Obecny stan kodu

Kod implementuje `Escrow2of3` (tag `0x03`):
- **Release:** Buyer + Operator → Merchant (all-or-nothing)
- **Refund:** Merchant + Operator → Buyer (all-or-nothing)
- **RecoveryRelease:** Buyer + Merchant → (split, **bez operatora**)

To jest bridge — nie jest to kanoniczny model V0.

### Docelowy model V0

Nowa polityka: `ComputeLeaseEscrow` (tag `0x04`).
- Brak `operator_pk_hash` w polityce
- Settlement walidowany przez protokol z receiptow + lease policy + timeout
- Natywny ProRataSplit: 1 input escrow → 2 output UTXOs (miner share + user share)

### Fazy przejscia

| Faza | Opis | Status |
|------|------|--------|
| **Phase 0** (obecny) | Operator co-signs Release/Refund. All-or-nothing. | `zaimplementowane` |
| ~~Phase 1~~ | ~~Automated operator~~ | **POMINIETY** — idziemy bezposrednio do operatorless |
| **Phase 2** (cel V0) | Operatorless: protokol waliduje receipty bez operatora. ProRataSplit jako jedna transakcja. | `kierunek` |

### Wzor rozliczenia

```
effective_windows = passed_windows + (degraded_windows * degraded_weight_permille / 1000)
miner_share = total_locked_amount * effective_windows / total_windows
user_share = total_locked_amount - miner_share
```

Reszta z dzielenia calkowitego zawsze do uzytkownika. Zero floatow.

### Opcje rozliczenia

| Przypadek | Wynik |
|-----------|-------|
| Sesja nie rozpoczela sie | Pelny zwrot do uzytkownika |
| Sesja zakonczona w pelni | Pelna kwota do minera |
| Sesja czesciowo zakonczona | Pro-rata split wedlug receiptow |
| Miner zawiodl wczesnie | Zwrot niewykorzystanej kwoty + opcjonalna kara |
| Timeout | RecoveryRelease (user + miner, bez operatora) |

---

## 6. Receipt Truth Architecture

Najwieksze ryzyko systemu: prawdziwosc receiptu.

```
Podpis minera dowodzi: miner twierdzi X.
Podpis minera NIE dowodzi: miner faktycznie dostarczyl X.
```

### Warstwy dowodzenia:

1. **Signed Receipts** — baseline evidence, wystarcza jako laboratorium Phase 1
2. **User Acknowledgment** — opcjonalne potwierdzenie, wzmacnia evidence
3. **Challenge/Response** — finalny model: deterministyczne challengi weryfikujace posiadanie zasobu w czasie rzeczywistym

### ZK Proof

- **Aggregate proof** na receipcie (zawsze): potwierdza ze suma passed_windows jest prawdziwa
- **Per-window proof** w sporze (na zadanie): potwierdza kazde okno PASS/FAIL
- **Co ukrywa:** dokladne pomiary, obciazenie GPU, innych uzytkownikow, telemetrie systemowa

---

## 7. Transport i prywatnosc

### Dwa tryby transportu:

| Tryb | Uzycie | Szyfrowanie |
|------|--------|-------------|
| **NXMS Mailbox** | Discovery, negocjacja, challengi, receipty | Kazda koperta: FrodoKEM (post-kwantowy) + XChaCha20Poly1305 (szybki) + Falcon (podpis) |
| **P2P direct (Tor)** | Sesja VM, streaming, terminal | FrodoKEM handshake raz → XChaCha20Poly1305 na wszystko |

### Tryby sieciowe:

| Tryb | Opis |
|------|------|
| `isolated` | Brak sieci zewnetrznej |
| `nxms_only` | Tylko prywatny transport privAI/NXMS |
| `tor_gated` | Przez Tor / relay topology |
| `internet_exit` | Publiczny internet. Jawny opt-in. Nigdy domyslny. |

---

## 8. Granica on-chain / off-chain

```
NA CHAINIE (~1%):
  - blokada pieniedzy (escrow lock)
  - hash polityki (lease_policy_commit)
  - hash dowodu (receipt commitment)
  - podzial pieniedzy (settlement)
  - nullifiers (brak podwojnego wydania)
  - podpisy autoryzujace
  - wysokosc bloku (zegar)

NIE NA CHAINIE (~99%):
  - szukanie compute
  - uzgadnianie warunkow
  - uruchamianie VM
  - uzywanie VM (prompti, outputy, dane)
  - challengi i odpowiedzi
  - telemetria
  - receipty (pelne)
  - profile minerow (brak publicznych profili)
```

**Chain widzi tylko commitmenty. Nie widzi tresci.**

---

## 9. To co jest odrzucone

```
NIE jest marketplace. NIE sprzedajemy modeli AI.
NIE ma publicznych profili providerow.
NIE ma publicznego rejestru uslug.
NIE ma oceny jakosci odpowiedzi AI.
NIE ma reputacji ludzi.
NIE ma moderatora sporow.
NIE ma internetu jako domyslnego trybu.
NIE ma cichego downgrade z FullPrivacy.
```

---

## 10. Struktura kodu i stan implementacji

Workspace Rust zawiera nastepujace crate'y:

| Crate | Odpowiedzialnosc |
|-------|-----------------|
| `privai-chain` | Chain, SpendPolicy, escrow rules, tx validation, compute lease types |
| `privai-ledger` | Ledger state, escrow persistence, note management |
| `privai-node` | Node runtime, consensus, block validation, metering, session, identity |
| `privai-nxms` | NXMS transport, Stage A/B boundary, envelope handling, discovery |
| `privai-proof` | Halo2 proof scaffold (LWE amount, nullifier, note commit, receipt circuit) |
| `privai-wallet` | Wallet keys, transaction construction, compute lease builder |
| `nxms-escrow-orchestrator` | Escrow orchestration |
| `nxms-mailbox` | Mailbox storage and delivery |
| `nxms-mailbox-client` | Mailbox client API |
| `nxms-signer` | Transaction signing |
| `nxms-transport` | Low-level transport layer, relay structs |

### Zaimplementowane i przetestowane (80 testow, 0 regression):

| Plik | Opis | Testy |
|------|------|------|
| `privai-chain/src/compute_lease.rs` | Types + settlement formula | 8 |
| `privai-chain/src/compute_escrow.rs` | Escrow policy + evaluate | 1 |
| `privai-chain/src/versioning.rs` | 9 domen, no-downgrade rule | 10 |
| `privai-node/src/metering.rs` | Agent + hash chain + receipt | 11 |
| `privai-node/src/compute_session.rs` | Session loop z traits | — |
| `privai-node/src/identity.rs` | 2 niezalezne klucze | — |
| `privai-nxms/src/discovery.rs` | DiscoveryQuery + encode | 1 |
| `nxms-transport/src/relay.rs` | Onion routing structs | 5 |
| `privai-proof/src/halo2/receipt_circuit.rs` | ZK proof circuit | 3 |
| `privai-wallet/src/compute_lease_builder.rs` | Wallet builder | 4 |

Plus istniejacy kod (code-confirmed):
- Escrow2of3 z Release/Refund/RecoveryRelease
- Stage A / Stage B boundary z prawdziwymi podpisami Falcon
- Timeout enforcement w ledger
- Persistence across node restarts
- Halo2 proof scaffold (1-in/1-out, Poseidon commitments, range checks)
- Dual-track amount: OutputNote (encrypted, Amount14) + LiteOutputNote (plain, u64)

### Kluczowe decyzje architektoniczne (podjete):

```
1. u128 dla LedgerAmount          (Bitcoin nie miesci sie w u64)
2. #[deprecated] na marketplace   (6 zmian, zero break)
3. Stage/Phase naming             (Production Stages, Escrow Phases)
4. Skip Phase 1                   (idz bezposrednio do operatorless)
5. Window-based metering          (availability + performance per okno)
6. Receipt = aggregate + ZK proof (total/passed + consistency proof)
7. 2 niezalezne klucze            (validator + miner, Phase 0-5)
8. NXMS mailbox = discovery       (encrypted queries)
9. FrodoKEM + XChaCha20Poly1305   (PQ + fast symmetric)
10. Chain = ksiegowy prywatnosci   (commitments, nie workloads)
```

### Co brakuje do production:

```
- Ledger integration (validate_compute_lease_escrow_auth)
- ComputeSettlementTx (nowy tx type)
- TimeoutClaimTx (nowy tx type)
- ProRataSplit execution (1→2 outputs)
- Miner runtime (VM, agent daemon, benchmarks)
- Mailbox /v1/discover endpoint
- End-to-end integration test
- Hidden sender encryption (z poprawkami: PoW na envelope)
```

---

## 11. Hierarchia dokumentacji (Tiers)

| Tier | Cel | Status |
|------|-----|--------|
| **Tier 0** | Kontrola / nawigacja (task log, prompt log, docs tree) | Istniejace |
| **Tier 1** | Kanoniczna direkcja (master direction, diagrams, settlement) | Istniejace |
| **Tier 2** | Wymagane dokumenty kierunkowe (operatorless escrow, identity, metering, discovery, itd.) | Czesciowo napisane, wiekszosc `planned` |
| **Tier 3** | Przyszle specyfikacje protokolu (wire formaty, receipt schema, pro-rata spec) | `future protocol spec` |
| **Tier 4** | Legacy quarantine / migracja | Stare dokumenty wylaczone z V0 |
| **Tier 5** | Planowanie implementacji (landing zones, test matrix, devnet) | `blocked` — wymaga Tier 2 + Tier 3 |

---

## 12. Fazy produkcji

| Faza | Cel | Status |
|------|-----|--------|
| **Phase 0: V0 Direction Freeze** | Zamrozenie kierunku, stworzenie dokumentow kierunkowych | `in progress` |
| **Phase 1: Compatibility Bridge** | Automated operator jako bridge, testowanie receipt validation | `future` |
| **Phase 2: Receipt-Aware Escrow** | Definicja lease policy, receipt validation, pro-rata mechanics | `future` |
| **Phase 3: Operatorless Settlement** | Protokol waliduje receipty bez operatora | `future` |
| **Phase 4: Private Discovery / Identity / Transport** | Hidden root, private discovery, NXMS hardening | `future` |
| **Phase 5: Production Readiness** | Brak cichego downgrade, pelna zgodnosc docs/code | `future` |

---

## 13. Wersjonowanie protokolu

12 niezaleznych domen wersji:

| Domena | Aktywacja |
|--------|-----------|
| `chain_protocol_version` | Wysokosc bloku / epoch |
| `tx_version` | Wysokosc bloku / epoch |
| `escrow_policy_version` | Wysokosc bloku / epoch |
| `compute_lease_protocol_version` | Deklarowana w ofercie/kontrakcie |
| `meter_protocol_version` | Deklarowana w receiptach |
| `proof_system_id` | Deklarowana w proof/statement |
| `credential_schema_version` | Deklarowana w proof/credentials |
| `discovery_protocol_version` | Negocjowana |
| `nxms_transport_version` | Negocjowana w handshake |
| `mailbox_protocol_version` | Negocjowana w handshake |

**Twarda regula: Zadna cicha degradacja z FullPrivacy.**

---

## 14. Zagrozenia

```
ZNANE (mamy rozwiazania):
  - Miner forguje receipt       → ZK proof + dispute
  - User forguje dispute        → dispute fee (loser pays)
  - Nikt nie submituje          → timeout auto-refund
  - Oversubscription            → performance benchmark
  - Double-spend                → nullifier
  - Miner znika                 → receipt = 0 → refund
  - User znika                  → miner submituje → settlement

NIEZNANE (na devnet):
  - attack vectors odkryjemy dopiero na devnet
  - timing attacks na challenge/response
  - metadata leakage o ktorej nie pomyslelismy
  - economic attacks na incentive model
```

---

## 15. Unikalne cechy

```
1. Post-kwantowe (FrodoKEM + Falcon) — nikt inny w compute rental nie ma
2. Ukryte kwoty (LWE encryption) — na chainie, odporne na "zbieraj dzis, odszyfruj pozniej"
3. Privacy-by-default — nie privacy jako opcja
4. Operatorless settlement — chain jako arbiter, nie czlowiek
5. Window-based metering — proste, mierzalne, bez zaufania do minera
```

---

## 16. Decyzje do podjecia

```
OPEN:
  - Benchmark suite per klase zasobu (MLPerf, LINPACK, fio)
  - Privacy class granularity (co dokladnie miner widzi)
  - Discovery architecture (mailbox vs encrypted registry)
  - PoW na envelope (rate limiting w ukrytym nadawcy)
```

---

## 17. Ukryte rozwiazania (insights z analizy)

Wiele problemow V0 ma rozwiazania ktore juz istnieja w kodzie:

| Problem | Widoczne rozwiazanie | Ukryte rozwiazanie |
|---------|---------------------|-------------------|
| Receipt truth | Challenge/response protocol | Challenge-sampled proof of resource possession — trustless, bez zaufania do nikogo |
| System kwot | Nowy wiekszy typ | Dual-track (Amount14 + u64) juz dziala w kodzie |
| Tozsamosc | Pelna hierarchia hidden root | Dwa niezalezne klucze (Phase 0-5 wystarczy) |
| Operatorless | Nowy protokol | RecoveryRelease juz jest template (user + miner, bez operatora) |
| Discovery | Nowy protokol (DHT/gossip) | NXMS mailbox juz jest transportem |
| Automated operator | Osobny serwis | Funkcja w ledger — deterministyczna |
| Pro-rata | Nowa mechanika note split | Sekwencja Release+Refund (Phase 1, zero zmian w note split) |
| Governance | Centralne cialo | 12 niezaleznych domen wersji |

---

## 18. Blokujace pytania

1. Jaki jest maksymalny supply PVA? (blokuje `u64` vs `u128` dla aPVA)
2. Czy Phase 2 wymaga challenge/response dla receipt truth? (blokuje operatorless bridge)
3. Czy production phases i escrow transition phases trzeba przemianowac? (blokuje jasna komunikacje)

---

## 19. Finalne stwierdzenie

> privAI to post-kwantowa FullPrivacy siec prywatnego compute, w ktorej uzytkownicy prywatnie wynajmuja izolowane zasoby obliczeniowe od compute minerow.
>
> Chain zapewnia rozliczenie PVA/aPVA przez FullPrivacy compute lease escrow.
>
> Validatorzy zabezpieczaja chain.
>
> Compute minerzy zarabiaja PVA za dostarczone zasoby.
>
> Relay/mailbox zarabiaja PVA za prywatny transport.
>
> Exit node to opcjonalne, jawne role.
>
> Discovery jest prywatne i resource-based — nie publicznym marketplace'em AI.
>
> Scoring mierzy niezawodnosc maszyny i dostepnosc — nie subiektywna jakosc odpowiedzi AI.
>
> Wersje protokolu musza byc jawne dla chain, escrow, proof, transport, metering, credentials, discovery, relay, mailbox i exit.
>
> **Zadna cicha degradacja z FullPrivacy.**

---

*Wersja dokumentu: 2026-04-12. Podsumowanie na podstawie kodu i dokumentacji z PRIVAI_V0_PRIVATE_COMPUTE. Kanoniczne zrodla: PRIVAI_V0_ARCHITECTURE_SPEC.md, PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md, PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md.*
