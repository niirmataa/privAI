# privAI Canonical Spec Index

Status: canonical navigation doc for the frozen spec set.
Canonicality: canonical entrypoint and reading order for all normative `privAI` specs. This document does not override product, protocol, format, marketplace, consensus or vector semantics; it only defines where the source of truth lives and how it should be read.
Owner: privAI spec governance.
Depends on:
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md`
- `spec/PRIVAI_CRYPTOGRAPHIC_BREAKTHROUGH_CANDIDATES.md`
- `spec/PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`
- `spec/PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`
- `spec/PRIVAI_CHAIN_STORAGE_AND_RECOVERY_NEXT_STEPS.md`
- `spec/PRIVAI_CHAIN_STORAGE_INVARIANTS.md`
- `spec/PRIVAI_RECOVERY_AND_RESTART_RULES.md`
- `spec/PRIVAI_REFERENCE_VECTORS.md`
- `spec/PRIVAI_DECISION_REGISTER.md`
- `spec/PRIVAI_GAP_REGISTER.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`
- `spec/PRIVAI_PROOF_BOUNDARIES.md`
Supersedes:
- ad-hoc navigation through old `PRIVAI_V0*` docs
- any assumption that a dev or agent should derive the current architecture from mixed old and new documents

## 1. Cel

Ten dokument istnieje po to, zeby kazdy dev i kazdy agent AI zaczynal w jednym miejscu.

Zamrozona zasada:
- wszystkie normatywne source-of-truth docs dla finalnego systemu `privAI` znajduja sie pod `spec/`,
- dokument spoza `spec/` nie moze byc finalnym zrodlem prawdy dla systemu,
- kod nie jest samodzielnym zrodlem prawdy dla architektury, produktu ani finalnej semantyki,
- stare docs moga byc czytane pomocniczo albo historycznie, ale nie moga nadpisywac canonical spec set.

## 2. Canonical Spec Set

Jedynymi normatywnymi dokumentami systemu sa:

1. `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
   Zakres:
   - nadrzedny model systemu,
   - produkt,
   - rails,
   - jednostki,
   - progi,
   - status vocabulary,
   - anti-hallucination rules.

2. `spec/PRIVAI_PROTOCOL_CORE.md`
   Zakres:
   - semantyka rdzenia `note/UTXO`,
   - core objects,
   - tx classes,
   - lifecycle,
   - odpowiedzialnosci wallet / ledger / consensus.

3. `spec/PRIVAI_CANONICAL_FORMATS.md`
   Zakres:
   - bytes,
   - field order,
   - domain strings,
   - commitments,
   - signed-envelope boundaries,
   - merkle rules.

4. `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`
   Zakres:
   - `MarketplaceSmallPaymentsRail`,
   - policy / grant / receipt / settlement semantics,
   - trust assumptions,
   - operator authority.

5. `spec/PRIVAI_CONSENSUS.md`
   Zakres:
   - block validity,
   - state commitments,
   - rail enforcement,
   - proof coverage,
   - finality,
   - consensus authority model.

6. `spec/PRIVAI_REFERENCE_VECTORS.md`
   Zakres:
   - reference bytes,
   - reference commits,
   - roots,
   - signed payload examples,
   - bit-to-bit comparison pack.

## 2.1. Focused Architecture Targets

Ponizsze dokumenty nie rozszerzaja canonical spec setu o nowy rail ani nowy model source of truth.
Sluza do zapisania jednego zamrozonego kierunku tam, gdzie potrzebny jest bardziej operacyjny plan architektury.

1. `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md`
   Zakres:
   - escrow `2 z 3` na `FullPrivacy`,
   - adaptacja `nexum-core`,
   - coin-side architecture do dobudowania,
   - granica miedzy current-compatible target i future migration.

2. `spec/PRIVAI_CRYPTOGRAPHIC_BREAKTHROUGH_CANDIDATES.md`
   Zakres:
   - gdzie `privAI` ma realny potencjal na wklad kryptograficzny,
   - po czym wolno rozpoznac przelom kryptograficzny,
   - jak nie mylic autorskiej architektury z nowym prymitywem.

3. `spec/PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`
   Zakres:
   - rozdzial `nxms-transport`, validator session transport i `privai-node`,
   - mapowanie aktualnego kodu na docelowe warstwy,
   - granice API pod refactor P2P.

4. `spec/PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`
   Zakres:
   - current canonical behavior validator session layer,
   - handshake / session / reconnect / ban / limiter invariants,
   - current non-conformities and unresolved gaps,
   - anty-hallucination boundary for session transport behavior.
   Status roli:
   - support doc anti-drift,
   - nie tworzy nowego rownorzednego source of truth,
   - rozwija current behavior wynikajacy z transport split i aktualnego kodu.

5. `spec/PRIVAI_CHAIN_STORAGE_AND_RECOVERY_NEXT_STEPS.md`
   Zakres:
   - chain/storage follow-up plan,
   - persistence / restart / recovery checklist,
   - runtime durability contract to freeze next.
   Status roli:
   - support doc anti-drift,
   - nie tworzy nowego rownorzednego source of truth,
   - porzadkuje current storage/recovery work przed wejsciem w nowe code changes.

## 3. Reading Order

Kazdy dev i kazdy agent powinien czytac dokumenty w tej kolejnosci:

1. ten dokument
2. `spec/PRIVAI_REENTRY_GUIDE.md`
3. `spec/PRIVAI_DECISION_REGISTER.md`
4. `spec/PRIVAI_GAP_REGISTER.md`
5. `spec/PRIVAI_EXECUTION_SPINE.md`
6. `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
7. `spec/PRIVAI_PROTOCOL_CORE.md`
8. `spec/PRIVAI_CANONICAL_FORMATS.md`
9. `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`
10. `spec/PRIVAI_CONSENSUS.md`
11. `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md`
12. `spec/PRIVAI_CRYPTOGRAPHIC_BREAKTHROUGH_CANDIDATES.md`
13. `spec/PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`
14. `spec/PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`
15. `spec/PRIVAI_PROOF_BOUNDARIES.md`
16. `spec/PRIVAI_REFERENCE_VECTORS.md`
17. `spec/PRIVAI_CHAIN_STORAGE_AND_RECOVERY_NEXT_STEPS.md`
18. `spec/PRIVAI_CHAIN_STORAGE_INVARIANTS.md`
19. `spec/PRIVAI_RECOVERY_AND_RESTART_RULES.md`
20. `spec/PRIVAI_TRANSFER_NOTE_PROOF_SEMANTICS.md`

## 3.1. Anti-Drift Support Docs

Ponizsze pliki nie zastepuja canonical setu, ale sa obowiazkowymi dokumentami pomocniczymi do pracy bez dopowiadania:

1. `spec/PRIVAI_REENTRY_GUIDE.md`
   Zakres:
   - szybki powrot do kontekstu,
   - co jest frozen, otwarte i zabronione do zgadywania.

2. `spec/PRIVAI_DECISION_REGISTER.md`
   Zakres:
   - zatwierdzone decyzje,
   - current canonical vs future target,
   - jawne rozroznienie migration-required.

3. `spec/PRIVAI_GAP_REGISTER.md`
   Zakres:
   - otwarte luki,
   - blocker status,
   - forbidden inference,
   - next action.

4. `spec/PRIVAI_EXECUTION_SPINE.md`
   Zakres:
   - master execution order across transport, auth, chain, proof, escrow and marketplace work,
   - phase-level dependencies,
   - phase checklists and exit criteria,
   - execution-control spine for future narrow docs.

5. `spec/PRIVAI_PROOF_BOUNDARIES.md`
   Zakres:
   - proof status boundaries,
   - czego nie wolno zakladac dla `OnChainLite`,
   - jak czytac current bytes `ExecutionBundle` i `ProofCertificate`.

6. `spec/PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`
   Zakres:
   - current validator session behavior,
   - handshake / reconnect / ban / limiter invariants,
   - anti-drift guard for test and implementation tasks.

7. `spec/PRIVAI_CHAIN_STORAGE_AND_RECOVERY_NEXT_STEPS.md`
   Zakres:
   - storage / restart / recovery work map,
   - next docs to freeze,
   - anti-drift guard before storage implementation tasks.

8. `spec/PRIVAI_CHAIN_STORAGE_INVARIANTS.md`
   Zakres:
   - current storage behavior mapping (durable / rebuildable / ephemeral),
   - source of truth dla note status, nullifiers, blocks, tip, QC, safety state,
   - atomicity groups, current non-conformities, unresolved gaps.

9. `spec/PRIVAI_RECOVERY_AND_RESTART_RULES.md`
   Zakres:
   - current restart/recovery behavior mapping,
   - startup sequence, tip recovery, safety recovery,
   - failure model, current non-conformities, unresolved gaps.

10. `spec/PRIVAI_TRANSFER_NOTE_PROOF_SEMANTICS.md`
   Zakres:
   - current proof-bearing semantics for `TransferNoteTx`,
   - current statement / public inputs / witness relation,
   - ledger-vs-proof boundary for the main proof-covered tx class,
   - anti-drift guard before further proof vectors and proof tests.

## 4. What Is Not Source Of Truth

Ponizsze rzeczy nie sa normatywnym source of truth:
- `PRIVAI_V0_PROTOCOL.md`
- `PRIVAI_V0_FORMATS.md`
- `PRIVAI_V0_PAYMENTS_AND_ECONOMICS.md`
- `PRIVAI_CONSENSUS_V0.md`
- `spec/marketplace_small_payments_v0/*`
- review responses
- handoff docs
- roadmap docs

Kod zrodlowy:
- jest referencja implementacyjna,
- jest potrzebny do wyciagania obecnych bytes i hash outputs,
- ale nie moze sam nadpisac canonical docs.

## 5. How Devs And Agents Must Use This Index

Zamrozona zasada pracy:
- jesli zadanie dotyczy architektury lub semantyki, agent zaczyna od tego indexu i canonical set,
- jesli zadanie dotyczy implementacji, agent najpierw ustala odpowiedni canonical doc, a dopiero potem schodzi do kodu,
- jesli kod i canonical docs sa rozjechane, agent nie wybiera sobie wygodniejszej wersji,
- agent ma:
  - zglosic `current non-conformity`,
  - albo wykonac task doprowadzajacy kod do zgodnosci,
  - albo zaproponowac jawny freeze update.

Niedozwolone jest:
- projektowanie "pomiedzy dokumentami",
- traktowanie starych docs jako rownorzednego zrodla prawdy,
- dopelnianie brakujacej semantyki przez zgadywanie.

## 6. External References

Jesli canonical docs odnosza sie do:
- plikow kodu,
- testow,
- starych docs,
- review artifacts,

to sa to odniesienia pomocnicze.

Zamrozona zasada:
- odniesienie poza `spec/` nie tworzy nowego source of truth,
- odniesienie poza `spec/` sluzy tylko do:
  - znalezienia implementacji,
  - znalezienia obecnego behavior,
  - znalezienia historycznego kontekstu.

## 7. Before Assigning A Task To An Agent

Przed nadaniem taska implementacyjnego:
- wskaz odpowiedni canonical doc,
- wskaz status rzeczy, ktore dotyczy task,
- zaznacz, czy task dotyczy:
  - `Current canonical`,
  - `Frozen spec rule`,
  - `Frozen future target requiring migration`,
  - `Current non-conformity`,
  - `Provisional`,
  - `Experimental`.

Jesli task dotyczy vectors:
- agent nie moze sam uzupelniac placeholderow z `PRIVAI_REFERENCE_VECTORS.md`,
- moze wpisac tylko wartosci wynikajace z:
  - aktualnego `CanonicalEncode`,
  - zatwierdzonej frozen formula,
  - jawnie zatwierdzonej migracji.

## 8. Current Freeze-Vector Scope

Na dzis:
- `FullPrivacy` core moze byc mrozone bit-to-bit,
- obecny marketplace rail moze byc mrozony bit-to-bit w zakresie obecnej implementacji i frozen spec rules,
- `ExecutionBundle` i `ProofCertificate` maja current canonical bytes i vectors, ale ich pelna multi-rail semantyka pozostaje nie w pelni domknieta,
- `OnChainLite` pozostaje `Experimental`.

To oznacza:
- finalne vectors mozna domykac dla korpusu systemu juz teraz,
- ale nie wolno udawac, ze `OnChainLite` albo provisional consensus objects sa juz finalnym frozen body.
