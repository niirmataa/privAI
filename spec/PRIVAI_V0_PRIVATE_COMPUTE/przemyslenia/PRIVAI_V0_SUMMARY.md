# privAI V0 — Dokument podsumowujący

**Data:** 2026-04-12  
**Status:** summary of everything we built, decided, and designed

---

## 1. Czym jest privAI

privAI V0 to prywatna sieć wynajmu zasobów compute.

Użytkownik nie kupuje jakości AI. Użytkownik kupuje prywatny dostęp do zasobu compute.

```
Privacy is the product.
Compute is the supply.
PVA is the incentive.
Chain is the settlement.
Transport is the shield.
```

Nie jest marketplace. Nie ma publicznych profili. Nie ma oceny jakości outputu AI. Nie ma moderatora.

---

## 2. Co zrobiliśmy

### Decyzje architektoniczne (podjęte)

```
1. u128 dla LedgerAmount        ✓ (Bitcoin nie mieści się w u64)
2. #[deprecated] na marketplace  ✓ (6 zmian, zero break)
3. Stage/Phase naming            ✓ (Production Stages, Escrow Phases)
4. Skip Phase 1                  ✓ (idź bezpośrednio do operatorless)
5. Window-based metering         ✓ (availability + performance per okno)
6. Receipt = aggregate + ZK proof ✓ (total/passed + consistency proof)
7. 2 niezależne klucze           ✓ (validator + miner, Phase 0-5)
8. NXMS mailbox = discovery      ✓ (encrypted queries)
9. FrodoKEM + XChaCha20Poly1305  ✓ (PQ + fast symmetric)
10. Chain = księgowy prywatności  ✓ (commitments, nie workloads)
```

### Kod (zaimplementowany, testowany)

```
privai-chain/src/compute_lease.rs      — types + settlement (8 tests)
privai-chain/src/compute_escrow.rs     — escrow policy + evaluate (1 test)
privai-chain/src/versioning.rs         — 9 domen, no-downgrade (10 tests)

privai-node/src/metering.rs            — agent + hash chain + receipt (11 tests)
privai-node/src/compute_session.rs     — session loop z traits
privai-node/src/identity.rs            — 2 niezależne klucze

privai-nxms/src/discovery.rs           — DiscoveryQuery + encode (1 test)

nxms-transport/src/relay.rs            — onion routing structs (5 tests)

privai-proof/src/halo2/receipt_circuit.rs — ZK proof circuit (3 tests)

privai-wallet/src/compute_lease_builder.rs — wallet builder (4 tests)

RAZEM: 80 nowych testów, 0 regression
```

### Dokumenty (zapisane)

```
przemyslenia/SIMPLE_1_CZYM_JEST_SYSTEM.md
przemyslenia/SIMPLE_2_JAK_WYGLADA_SYSTEM.md
przemyslenia/SIMPLE_3_JAK_PRZEJSC_Z_KODU_DO_V0.md
przemyslenia/SIMPLE_4_JAK_FINALNIE_WYGLADA_SYSTEM.md (z poprawkami kolegi)
przemyslenia/SIMPLE_5_SYSTEM_CORE_DRAFT.md
przemyslenia/PRIVAI_V0_PRODUCTION_ROADMAP.md
PRIVAI_V0_FORMATS.md (zaktualizowany)
PRIVAI_V0_DIAGRAMS.md (zaktualizowany, 12 diagramów)
PRIVAI_V0_SIMPLE_SYSTEM_DESCRIPTION_PL.md
PRIVAI_V0_WINDOW_PROTOCOL_DESIGN_QUESTIONS.md (36 decyzji)
```

---

## 3. Architektura systemu

### Warstwy

```
User → Discovery → Transport → Session → Metering → Receipt → Settlement → Chain

On-chain (1%):   escrow lock, receipt hash, settlement split
Off-chain (99%): discovery, negotiation, session, metering, receipts
```

### Escrow

```
ComputeLeaseEscrow (tag 0x04)
  user_pk_hash
  miner_pk_hash
  lease_policy_commit
  locked_amount (u128)
  timeout_block

BEZ operatora. Chain jest arbitrem.
```

### Settlement

```
miner_share = amount × (passed + degraded × weight / 1000) / total
user_share = amount - miner_share

Integer arithmetic. Reszta do usera. Zero floatów.
```

### Metering

```
Co 60 bloków (≈30 minut):
  challenge = hash(session_id || window_id || block_hash)
  miner odpowiada (availability + performance)
  agent podpisuje telemetry (hash-chained)
  user weryfikuje PASS/FAIL

Po sesji: aggregate receipt → chain → settlement
```

### Identity

```
Phase 0-5: 2 niezależne klucze
  Validator:   obecny Falcon PK (z vault) — FROZEN
  ComputeMiner: osobny Falcon PK — generowany niezależnie

Phase 6+: hidden root (future)
```

### Transport

```
NXMS mailbox: FrodoKEM + XChaCha20Poly1305 per envelope
P2P direct:   FrodoKEM handshake (raz) → XChaCha20Poly1305 (streaming)
Relay:        onion routing (każdy hop widzi prev/next)
```

### Versioning

```
9 domen (ChainProtocol, TxVersion, EscrowPolicy, ProofSystem,
  NxmsTransport, MailboxProtocol, ComputeLease, MeterProtocol, DiscoveryProtocol)

Hard rule: no silent downgrade from FullPrivacy.
```

---

## 4. Co brakuje do production

```
GOTOWE (80 tests):              BRAKUJE:
  Types                           Ledger integration (validate_compute_lease_escrow_auth)
  Escrow policy                   ComputeSettlementTx (nowy tx type)
  Metering agent                  TimeoutClaimTx (nowy tx type)
  Session loop (traits)           ProRataSplit execution (1→2 outputs)
  Identity (2 keys)               Miner runtime (VM, agent daemon, benchmarks)
  Discovery query                 Mailbox /v1/discover endpoint
  Relay structs                   End-to-end integration test
  Versioning                      Deprecation markers
  ZK circuit scaffold             Hidden sender encryption (z poprawkami)
  Wallet builder                  PoW na envelope (rate limiting)
```

---

## 5. Zagrożenia

```
Znane (mamy rozwiązania):
  - Miner forguje receipt     → ZK proof + dispute
  - User forguje dispute      → dispute fee (loser pays)
  - Nikt nie submituje        → timeout auto-refund
  - Oversubscription          → performance benchmark
  - Double-spend              → nullifier
  - Miner znika               → receipt = 0 → refund
  - User znika                → miner submituje → settlement

Nieznane (na devnet):
  - attack vectors odkryjemy dopiero na devnet
  - timing attacks na challenge/response
  - metadata leakage o której nie pomyśleliśmy
  - economic attacks na incentive model
```

---

## 6. Unikalne cechy

```
1. Post-kwantowe (FrodoKEM + Falcon) — nikt inny w compute rental nie ma
2. Ukryte kwoty (LWE encryption) — na chainie, odporne na "zbieraj dziś, odszyfruj później"
3. Privacy-by-default — nie privacy jako opcja
4. Operatorless settlement — chain jako arbiter, nie człowiek
5. Window-based metering — proste, mierzalne, bez zaufania do minera
```

---

## 7. Decyzje do podjęcia

```
OPEN:
  - Benchmark suite per klasę zasobu (MLPerf, LINPACK, fio)
  - Privacy class granularity (co dokładnie miner widzi)
  - Discovery architecture (mailbox vs encrypted registry)
  - PoW na envelope (rate limiting w ukrytym nadawcy)
```

---

## 8. Najważniejszy wniosek

Rdzeń systemu jest zbudowany. 80 testów. 0 regression.

Prawdziwa praca polega teraz na: integracji (ledger, wallet, miner runtime) i devnet (testowanie unknown unknowns).

Nie potrzeba więcej designu. Potrzeba kodu i testów.
