# privAI V0: Architektura Migracji Do Private Compute

**Status:** migration architecture / V0 code strategy
**Data:** 2026-04-11
**Źródło:** P-T039-XIAOMI
**Zakres:** strategia migracji z aktualnego kodu do `privAI V0 private compute`

Ten dokument zapisuje kierunkową strategię migracji.

Nie jest to implementation spec.
Nie definiuje finalnych struktur Rust.
Nie definiuje wire formatów.
Nie upoważnia do zmian w kodzie.
Nie korzysta z legacy docs.

---

## 1. Migration Verdict

Migracja jest realistyczna, ale wymaga strategii:

```text
add new alongside old
```

Nie należy refaktorować starego kodu in-place jako pierwszego kroku.

Najlepsza strategia:

1. dodać nowe V0 primitives obok istniejącego kodu,
2. oznaczyć stare typy jako legacy bridge / deprecated,
3. utrzymać compatibility z istniejącymi escrow notes i testami,
4. stopniowo przenosić ruch na nowe mechaniki V0.

---

## 2. Co Jest Bazą

V0 może budować na:

- escrow `2-of-3`,
- Stage A/B boundary,
- `RecoveryRelease` jako operatorless anchor,
- Falcon signatures,
- Halo2 scaffold,
- NXMS mailbox,
- PC-BFT consensus,
- ledger validation,
- `nxms-escrow-orchestrator` jako kandydat na Phase 1 automated operator.

Te elementy są wartościową bazą mechaniczną.

---

## 3. Co Jest Długiem Technicznym

Najważniejszy dług:

- `MarketplaceBatchTx`,
- `SpendPolicy::MarketplaceSettlement`,
- Falcon public key jako primary identity,
- `Amount14` jako potencjalny blocker dla aPVA,
- operator jako canonical signer dla Release/Refund.

Te elementy nie muszą być natychmiast usuwane.
Muszą dostać jawny status migracyjny.

---

## 4. Co Jest Blockerem

Przed kodem V0 trzeba rozstrzygnąć:

- brak `ComputeLeaseEscrow`,
- brak receipt/metering infrastructure,
- brak hidden-root identity,
- `Amount14` vs `aPVA`,
- los `MarketplaceBatchTx`,
- los `SpendPolicy::MarketplaceSettlement`,
- czy orchestrator jest bazą automated operatora.

---

## 5. Keep / Bridge / Deprecate / Replace / New Primitive Matrix

| Current code element | Current role | V0 role | Classification | Reason | Migration risk | Required doc/spec before code |
|----------------------|--------------|---------|----------------|--------|----------------|-------------------------------|
| `Escrow2of3` | 2-of-3 escrow policy | Phase 0/1 bridge | keep + bridge | solidny i testowany | low | Operatorless Escrow Direction |
| `RecoveryRelease` | timeout recovery | operatorless anchor | keep | już zgodny z V0 | none | none |
| `EscrowApprovalBundle` | Stage A/B handoff | automated operator bridge input | keep + bridge | separuje control/settlement | low | Operatorless Escrow Direction |
| `required_signers()` | frozen signer table | bridge plus future extension | keep + extend | nie łamać existing | medium | Operatorless Escrow Direction |
| `target_recipient()` | one-recipient settlement | bridge only | keep + add new | pro-rata potrzebuje two-recipient logic | high | Pro-rata Note Split Spec |
| `validate_output_target()` | validates one target | bridge only | keep + add new | new validation for pro-rata | high | Pro-rata Note Split Spec |
| `MarketplaceBatchTx` | marketplace batch tx | legacy only | deprecate | V0 nie jest marketplace | medium | operator/Opus decision |
| `SpendPolicy::MarketplaceSettlement` | buyer/seller/moderator policy | legacy only | deprecate | V0 nie ma moderator settlement | medium | operator/Opus decision |
| `Amount14` | encrypted LWE amount | proof/plaintext lane | keep + new primitive | economic amount needs larger type | high | aPVA Denomination Direction |
| `PQCIdentity` | Falcon PK/SK identity | bridge role key | bridge -> new primitive | obecny Falcon może stać się role key | high | Identity Model Direction |
| Falcon key vault | stores Falcon keys | bridge vault | bridge | dodać root/epoch/session later | medium | Identity Model Direction |
| `nxms-escrow-orchestrator` | operator workflow | Phase 1 automated operator base | bridge | state machine already useful | medium | Operatorless Escrow Direction |
| NXMS mailbox | push/pull/ack transport | private discovery transport base | keep + extend | naturalny transport | low | Private Discovery Direction |
| Halo2 scaffold | proof starting point | proof base | keep + extend | scaffold, not full proof | medium | Metering Protocol Direction |
| small payments `Receipt` | service payment receipt | inspiration only | keep + new primitive | compute lease needs separate receipt | low | Metering Receipt Schema |
| consensus validator identity | validator PK hash | validator role key | keep | V0 does not change consensus first | none | Node Roles Direction |
| `node_pk_hash` | node primary id | validator role key hash | bridge | semantic migration first | high | Identity Model Direction |

---

## 6. Amount / aPVA Migration

### 6.1 Czy `Amount14` Może Zostać?

Tak, ale tylko jako representation dla proof/plaintext lane.

`Amount14` jest związane z LWE plaintext space i prywatnym note amount.
Nie powinno być automatycznie traktowane jako pełna ekonomiczna denominacja compute lease.

### 6.2 Czy V0 Potrzebuje Nowego Amount Representation?

Tak.

V0 rozważa:

```text
1 PVA = 10^12 aPVA
```

To wymaga typu większego niż `Amount14`.

### 6.3 Rekomendowana Strategia

Wprowadzić dwuwarstwowy amount model:

```text
LedgerAmount: ekonomiczna kwota escrow w aPVA
Amount14: encrypted note/proof lane amount
```

Direction-level recommendation:

```text
LedgerAmount = u64 albo u128, zależnie od max supply PVA.
Amount14 zostaje dla note-level encryption.
```

### 6.4 Strategie Do Rozważenia

1. `Amount14` tylko dla proof/plaintext lane.
2. Nowy larger ledger amount type.
3. Split wartości na wiele notes/chunks.
4. Multi-note aggregation.

Najczystsza strategia directionally:

```text
Introduce LedgerAmount, keep Amount14 for proof lane.
```

### 6.5 Co Trzeba Sprawdzić

Przed decyzją trzeba sprawdzić:

- gdzie amount jest częścią commitmentów,
- czy ledger porównuje encrypted amount z policy amount,
- czy istnieją już `u64` amount fields w small payments,
- jak note commitments wiążą amount z proof layer.

---

## 7. Escrow / SpendPolicy Migration

### 7.1 `Escrow2of3` Zostaje Jako Bridge

`Escrow2of3` powinien zostać bez zmian dla Phase 0/1.

Nie wolno łamać istniejących escrow notes i testów.

### 7.2 Nowy `ComputeLeaseEscrow`

V0 compute lease powinien dostać nowy policy variant, zamiast dopisywać zbyt wiele do `Escrow2of3`.

Direction-level candidate:

```text
SpendPolicy::ComputeLeaseEscrow
```

Może zawierać directionally:

- user/miner key references,
- lease policy commitment,
- timeout,
- settlement mode,
- receipt requirements reference.

To nie jest finalny Rust struct.

### 7.3 `required_signers()`

Obecne `required_signers()` zostaje dla `Escrow2of3`.

Dla `ComputeLeaseEscrow` trzeba dodać osobne reguły.

### 7.4 Pro-Rata

Pro-rata wymaga nowej logiki względem:

- `target_recipient()`,
- `validate_output_target()`.

Current model:

```text
one output target
```

V0 model:

```text
two output targets: miner share + user remainder
```

### 7.5 RecoveryRelease

`RecoveryRelease` zostaje.

To jest już operatorless anchor.

---

## 8. Marketplace Types Strategy

### 8.1 `MarketplaceBatchTx`

Rekomendacja:

```text
deprecate + isolate as legacy rail
```

Nie usuwać natychmiast.

Powód:

- backward compatibility,
- test stability,
- brak migration planu.

### 8.2 `SpendPolicy::MarketplaceSettlement`

Rekomendacja:

```text
deprecate + isolate as legacy rail
```

V0 nie powinno używać tego wariantu.

### 8.3 Marker W Kodzie

Kierunkowo typy powinny mieć marker w stylu:

```text
V0 legacy: marketplace settlement is not part of private compute network.
```

Nie jest to jeszcze zgoda na patch.

---

## 9. Identity Migration

### 9.1 Obecny Stan

Aktualnie:

```text
Falcon PK = identity
```

V0 chce:

```text
hidden root -> role keys -> epoch keys -> session keys
```

### 9.2 Strategia Migracji

Faza 1:

```text
obecny Falcon PK = validator role key
```

Bez zmiany consensus.

Faza 2:

```text
dodanie hidden root credential do vault
```

Faza 3:

```text
nowe compute miner role key, session keys, epoch keys
```

### 9.3 `node_pk_hash`

`node_pk_hash` zostaje na początku.

Semantycznie staje się:

```text
validator role key hash
```

Nie jest już "primary human/system identity".

---

## 10. Receipt / Metering Migration

### 10.1 Small Payments Receipt

`small_payments::Receipt` nie powinien być reuse'owany bezpośrednio.

Może być inspiracją:

- signed receipt,
- session binding,
- policy binding,
- amount field,
- result commitment.

Ale compute lease receipt to osobny typ i osobny namespace.

### 10.2 Compute Lease Receipt

Potrzebny nowy obiekt:

```text
ComputeLeaseReceipt
```

Direction-level fields mogą obejmować:

- session reference,
- resource class,
- delivered duration,
- heartbeat status,
- meter version,
- lease policy commitment,
- miner signature.

To nie jest wire format.

### 10.3 Phase 1 Validation

Receipt validation powinna żyć w `nxms-escrow-orchestrator` jako Phase 1 bridge.

Orchestrator powinien deterministycznie wyliczać:

- full release,
- full refund,
- partial/pro-rata candidate.

### 10.4 Challenge/Response

Powinno powstać jako osobny metering layer/module, nie jako dopisek do escrow validation.

---

## 11. Orchestrator / Automated Operator Migration

`nxms-escrow-orchestrator` jest najlepszą bazą dla Phase 1 automated operator.

Ma już:

- state machine,
- ledger observation,
- proposal building,
- approval handling,
- DB persistence.

Musi dostać:

- receipt ingestion,
- deterministic receipt validation,
- lease policy binding,
- decision log,
- audit trail,
- kill criteria.

Nie może:

- podejmować discretionary decyzji,
- widzieć workloadów,
- stać się permanentnym central point,
- ukrywać decyzji przed audit.

---

## 12. Transport / Discovery Migration

NXMS mailbox jest naturalnym base dla private discovery.

Kierunek:

```text
Phase M8: mailbox-based encrypted discovery queries
Phase M8+: encrypted registry hybrid if latency requires
Later: gossip/DHT only after network-size evidence
```

Nie implementować zbyt wcześnie:

- DHT,
- gossip,
- public registry,
- final metadata hardening.

Braki w mailbox:

- encrypted envelope format policy,
- multi-hop / onion routing,
- padding,
- timing obfuscation.

---

## 13. Proof / Halo2 Migration

### 13.1 Co Jest Dzisiaj

Halo2 scaffold daje:

- LWE amount chip,
- nullifier chip,
- note commit chip,
- noise class chip,
- transaction skeleton circuit.

### 13.2 Czego Nie Dowodzi

Nie dowodzi jeszcze:

- full transfer privacy,
- balance conservation,
- consumed-note opening,
- receipt correctness,
- lease policy correctness.

### 13.3 Kierunek

Proof scope może ewoluować później w stronę:

- receipt commitment proof,
- lease policy binding proof,
- stronger transfer proof.

Nie wolno claimować, że to już działa.

---

## 14. Fazy Migracji

### Phase M0: No-Code Docs/Spec Freeze

**Goal:** zamrozić direction baseline.
**Entry:** V0 master, settlement direction, diagrams, docs tree, context plan.
**Exit:** required direction docs complete.
**Must not break:** nic.
**Must not overclaim:** V0 is direction, not implementation.

### Phase M1: Compatibility Cleanup / Markers

**Goal:** oznaczyć legacy types, nie usuwać.
**Entry:** decision about marketplace types.
**Exit:** markers/deprecation comments, tests unchanged.
**Must not break:** existing tests/escrow/consensus.
**Must not overclaim:** marketplace not removed yet.

### Phase M2: Amount / aPVA Decision

**Goal:** rozstrzygnąć `Amount14` vs `LedgerAmount`.
**Entry:** aPVA direction + max supply decision.
**Exit:** accepted amount strategy.
**Must not break:** LWE/proof lane.
**Must not overclaim:** aPVA frozen before spec.

### Phase M3: Compute Lease Policy Type

**Goal:** zaprojektować `ComputeLeaseEscrow`.
**Entry:** operatorless direction + compute lease object spec.
**Exit:** policy design ready.
**Must not break:** `Escrow2of3`.
**Must not overclaim:** type exists as full implementation.

### Phase M4: Receipt / Metering Prototype

**Goal:** direction/spec dla receipt i metering prototype.
**Entry:** metering direction + receipt schema.
**Exit:** receipt validation path designed.
**Must not break:** small payments receipts.
**Must not overclaim:** receipt truth solved.

### Phase M5: Automated Operator Bridge

**Goal:** orchestrator jako receipt-checking bridge.
**Entry:** operatorless bridge spec.
**Exit:** deterministic automated operator behavior specified/testable.
**Must not break:** current escrow flow.
**Must not overclaim:** operatorless implemented.

### Phase M6: Pro-Rata Note Split

**Goal:** 1 input -> 2 outputs.
**Entry:** pro-rata spec + aPVA freeze.
**Exit:** pro-rata mechanics designed/testable.
**Must not break:** all-or-nothing Release/Refund.
**Must not overclaim:** production-ready pro-rata.

### Phase M7: Identity Scoped Keys

**Goal:** hidden root + scoped role/session/epoch keys.
**Entry:** identity direction + credential schema.
**Exit:** migration plan from Falcon identity to role keys.
**Must not break:** consensus identity.
**Must not overclaim:** hidden root fully deployed.

### Phase M8: Private Discovery

**Goal:** mailbox-based private discovery.
**Entry:** discovery direction + protocol spec.
**Exit:** bootstrap discovery design.
**Must not break:** NXMS mailbox.
**Must not overclaim:** full private discovery solved.

### Phase M9: Devnet Validation

**Goal:** end-to-end private compute lease validation.
**Entry:** M0-M8 accepted.
**Exit:** devnet evidence.
**Must not break:** existing chain/consensus.
**Must not overclaim:** production-ready.

---

## 15. Top 10 Blockers Before Code

1. `Amount14` vs `aPVA`.
2. `MarketplaceBatchTx` fate.
3. `SpendPolicy::ComputeLeaseEscrow` vs extending `Escrow2of3`.
4. Max supply PVA.
5. Operatorless Escrow Direction.
6. Identity Model Direction.
7. Metering Protocol Direction.
8. Private Discovery Direction.
9. Phase 1 kill criteria.
10. Receipt availability model.

---

## 16. Co Można Robić Bezpiecznie Teraz

Bez kodu można:

- pisać Tier 2 direction docs,
- przygotowywać prompt packs,
- robić code audits,
- mapować migration touchpoints,
- robić test inventory,
- rozwijać MCP/RAG direction,
- prowadzić Xiaomi read-only discussions,
- utrzymywać V0 task/prompt logs.

---

## 17. Red Lines

Zatrzymać pracę, jeśli ktoś:

- używa legacy docs jako source of truth,
- traktuje marketplace jako product direction,
- zmienia escrow code przed policy decision,
- zmienia amount type przed `aPVA/Amount14` decision,
- usuwa marketplace types bez compatibility planu,
- twierdzi, że hidden root identity już istnieje,
- twierdzi, że receipt truth jest solved,
- twierdzi, że pro-rata jest implemented,
- traktuje orchestrator jako final operator,
- implementuje RAG/MCP z legacy ingest,
- definiuje wire formats przed protocol specs,
- proponuje kod przed direction docs.

---

## 18. Self-Check

```text
Czy czytano legacy docs: NIE
Czy czytano kod: TAK, według raportu Xiaomi
Czy edytowano kod: NIE
Czy zdefiniowano wire formaty: NIE
Poziom: migration architecture
Status: strategy doc, not implementation spec
```
