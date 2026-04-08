# privAI Escrow Final Model

Status: finalny high-level model escrow dla privAI. Model wykonawczy (tx matrix, binary formats, proof integration) bedzie domykany w osobnych dokumentach.
Canonicality: binding high-level escrow architecture doc. This document does not override canonical protocol, formats, consensus or product semantics; it records the final escrow model, trust assumptions, role semantics, mode semantics and architectural boundaries. Detailed tx matrix, object formats and proof integration are out of scope and belong in follow-up docs.
Owner: privAI protocol, escrow and integration architecture.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_EXECUTION_SPINE.md`
- `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
- `spec/PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md`
- `spec/PRIVAI_DECISION_REGISTER.md`
- `spec/PRIVAI_GAP_REGISTER.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zamknac finalny high-level model escrow dla privAI,
- zdefiniowac role, trust model, tryby dzialania i granice architektoniczne,
- dac jeden stabilny punkt odniesienia dla dalszych dokumentow wykonawczych,
- odciac overclaiming i niedopowiedzenia.

Ten dokument nie jest:
- tx matrix (to bedzie `PRIVAI_ESCROW_TX_MATRIX.md`),
- object format spec (to bedzie czescia canonical formats / escrow object doc),
- proof integration doc (to bedzie `PRIVAI_ESCROW_PROOF_INTEGRATION.md`),
- implementacja.

## 2. High-level Direction

Escrow w privAI jest mechanizmem note-based, opartym na policy-constrained 2-of-3 multisig. Srodki sa blokowane w nocie z polityke wydania, a nie na wspoldzielonym koncie. Podpisy autoryzuja konkretne akcje, nie dowolne wydatki.

System opiera sie na:
- threshold authorization nad spendem noty,
- policy reconstruction z `policy_opening`,
- action-bound signing przez `tx_signing_hash`,
- ledger-enforced validation.

## 3. Rail Assignment

Escrow 2-of-3 nalezy do `FullPrivacy`.

Nie nalezy do:
- `MarketplaceSmallPaymentsRail` — to jest operator-trusted accounting, nie escrow,
- `OnChainLite` — to jest experimental i nie ma finalnego proof/threshold closure.

To jest zamrozona decyzja (DEC-018).

## 4. Roles

Escrow angazuje trzy role:

- **Buyer:** inicjator transakcji, deponujacy noty jako zabezpieczenie.
- **Merchant:** odbiorca docelowy zabezpieczonych srodkow.
- **Operator:** arbiter/platforma — rozwiazuje spory w standardowym przebiegu, nadzoruje realizacje polityk.

Kazda rola ma:
- klucz publiczny Falcon,
- canonical signer identity,
- canonical signer index w policy.

## 5. Trust Model

- **Brak zaufania do pojedynczej strony:** zadna pojedyncza rola nie moze jednostronnie uwolnic srodkow.
- **Ograniczone zaufanie do Operatora:** Operator moze koordynowac uwolnienie srodkow do Buyera lub Merchanta, ale nie moze przywlaszczyc ich dla siebie — polityka akcji ogranicza dozwolone cele przelewu.
- **Weryfikacja on-chain:** bezpieczenstwo zalezy od poprawnej egzekucji zasad w warstwie ledger, nie od uczciwosci Operatora ani nexum-core.
- **Policy-constrained destinations:** srodki moga trafic tylko do celow zdefiniowanych w polityce akcji — nie do dowolnego adresu.

## 6. Policy-Constrained 2-of-3 Semantics

To nie jest klasyczny multisig, gdzie dwoch posiadaczy kluczy wydaje srodki na dowolny adres.

To jest policy-constrained multisig:
- nota ma `spend_policy_commit`,
- `spend_policy_commit` otwiera sie do polityki 2-of-3 przez `policy_opening`,
- ledger rekonstruuje polityke z `policy_opening` i weryfikuje:
  - czy signer nalezy do policy signer set,
  - czy quorum jest spelnione,
  - czy akcja jest dozwolona przez polityke,
- srodki moga trafic tylko do celow ujętych w zdefiniowanych akcjach.

To odcina wektory ataku z wewnetrznej zmowy.

## 7. Normal Mode

W standardowym scenariuszu **Operator musi byc jednym z sygnatariuszy**.

Dozwolone kombinacje:
- `Buyer + Operator` — release srodkow do Merchanta (Buyer potwierdza wykonanie uslugi/dostawe),
- `Merchant + Operator` — refund srodkow do Buyera (Merchant akceptuje reklamacje / Operator mediuje).

Logika rol:
- Buyer autoryzuje release, bo potwierdza, ze ustalenia zostaly spelnione,
- Merchant autoryzuje refund, bo godzi sie na zwrot,
- Operator jest wymagany w obu przypadkach jako arbiter i anti-fraud guard.

Operator nie moze:
- sam inicjowac release ani refund bez drugiej strony,
- przekierowac srodkow do siebie.

High-level signer/action intent opisany powyzej jest zamrozony w tym dokumencie.
Follow-up `PRIVAI_ESCROW_TX_MATRIX.md` doprecyzuje execution-level semantics, required fields i ledger checks, ale nie odwraca tego intent.

## 8. Recovery Mode

Jesli Operator stanie sie niedostepny lub odmowi wspolpracy:
- **Buyer + Merchant** moga wspolnie (2-of-3) podpisac transakcje uwalniajaca srodki, omijajac Operatora.

To jest ostateczna sciezka ratunkowa:
- eliminuje ryzyko vendor lock-in po stronie Operatora,
- wymusza polubowne rozstrzygniecie miedzy stronami transakcji,
- zapobiega trwalemu zablokowaniu not.

Exact recovery conditions (timeout, dispute semantics) sa jeszcze do domkniecia w follow-up docs.

## 9. Action Model

Podpisy w escrow nie autoryzuja swobodnego wydatku (arbitrary spend).

Kazda proba wydania noty escrow deklaruje konkretna akcje:
- `release` — uwolnienie srodkow do Merchanta,
- `refund` — zwrot srodkow do Buyera.

Mozliwe przyszle rozszerzenia:
- `partial_release`,
- `dispute_resolution`.

Podpis sygnatariusza jest wazny tylko w kontekscie konkretnej zadeklarowanej akcji:
- podpis zlozony na `refund` nie moze byc uzyty do `release`,
- replay protection jest wbudowana w action binding.

## 10. Relationship to `tx_signing_hash`

Zgodnie z `PRIVAI_AUTH_SIGNING_MODEL.md`:
- uczestnicy podpisuja `tx_signing_hash`,
- `tx_signing_hash` wiaze: action type, input references, output commitments, fee, policy-relevant fields,
- nie ma mozliwosci przechwycenia podpisu z jednej akcji i uzycia go do innej,
- ledger odtwarza `tx_signing_hash` z tx body i weryfikuje auth package wzgledem niego.

## 11. Relationship to `policy_opening`

Zgodnie z `PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`:
- threshold auth package jest weryfikowany wzgledem policy zrekonstruowanej z `policy_opening`,
- ledger sprawdza, czy `policy_opening` po commitcie daje oczekiwany `spend_policy_commit`,
- signer set, threshold i role constraints pochodza z zrekonstruowanej policy,
- bez poprawnego `policy_opening` escrow spend nie moze byc zweryfikowany.

## 12. What nexum-core Is Used For

`nexum-core` jest koordynatorem, nie weryfikatorem:
- workflow escrow (open, fund, propose, approve, execute),
- snapshot kontraktu escrow,
- proposal release/refund,
- zbieranie approval signatures od stron,
- timeout/dispute orchestration,
- replay/idempotency na poziomie control-plane.

`nexum-core` nie:
- podejmuje samodzielnych decyzji o przesunięciu srodkow,
- pelni funkcji zaufanego weryfikatora,
- zastepuje ledger validation.

## 13. Ledger Verification Boundary

Ledger jest finalnym weryfikatorem. Sprawdza:
- input existence i unspent status,
- `policy_opening` match z `spend_policy_commit`,
- Falcon signatures poprawnosc,
- signer membership w policy signer set,
- quorum 2-of-3,
- action semantics zgodnosc z policy,
- canonical signer ordering wg signer index in policy,
- duplicate signer rejection,
- nullifier / replay protection,
- normalna walidacje spendu i state transition.

## 14. Relationship to Proof

Proof layer:
- moze docelowo ukrywac role/quorum w publicznym systemie zachowujac przejrzystosc w warstwie weryfikatora,
- ale proof nie zastepuje auth verification.

Twarda zasada:
- v1 escrow nie jest blokowany oczekiwaniem na finalny proof-aware threshold model,
- ledger-enforced threshold auth jest wystarczajacy do uruchomienia escrow v1,
- proof integration details beda zdefiniowane w `PRIVAI_ESCROW_PROOF_INTEGRATION.md`.

Na tym etapie nie zawieramy twierdzien o full PQ privacy dla escrow. Ta wlasciwosc podlega odrebnemu audytowi.

## 15. Relationship to Marketplace Rail

Escrow jest oddzielnym mechanizmem od `v0 marketplace convenience rail`.

Marketplace v0:
- jest operator-trusted accounting (grant/receipt/batch settlement),
- nie jest modelem blokowania pelnej wartosci escrow note,
- nie jest finalnym escrow.

Finalny escrow:
- jest rdzennym mechanizmem sieci privAI,
- celowo odcina "sciezki na skroty" z MVP.

## 16. Escrow Object Model — Scope Pointer

GAP-013 wymaga zdefiniowania canonical escrow object model:
- `EscrowFundingDescriptor`
- `EscrowSnapshot`
- `EscrowSpendProposal`
- `EscrowApprovalBundle`

High-level semantyka tych obiektow jest opisana w `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md` (sekcje 8, 10).

Exact binary formats, field layouts i canonical encoding sa poza scope tego dokumentu i beda domykane w follow-up docs.

## 17. What This Document Does Not Define

Ten dokument definiuje high-level signer/action intent (sekcja 7, 8, 9), ale jawnie nie definiuje:
- execution-level action matrix, required fields i ledger checks (to bedzie `PRIVAI_ESCROW_TX_MATRIX.md`),
- exact binary formats escrow objects,
- exact proof integration dla escrow,
- exact timeout/dispute on-chain enforcement semantics,
- exact recovery conditions beyond "Buyer + Merchant can jointly sign".

Te tematy sa otwarte i beda domykane w kolejnych fazach. Follow-up docs doprecyzuja execution-level details, ale nie odwracaja high-level intent zamrozonego tutaj.

## 18. Checklist

- [ ] Potwierdzic, ze AUTH_SIGNING_MODEL i THRESHOLD_AUTH sa zsynchronizowane z tym modelem.
- [ ] Doprecyzowac execution-level action matrix, required fields i ledger checks w `PRIVAI_ESCROW_TX_MATRIX.md`.
- [ ] Zdefiniowac exact escrow object formats.
- [ ] Zdefiniowac exact proof integration w `PRIVAI_ESCROW_PROOF_INTEGRATION.md`.
- [ ] Zdefiniowac exact recovery conditions (timeout, dispute).
- [ ] End-to-end escrow test (scenarios z CP-07 w ESCROW_2OF3_ADAPTATION).

## 19. Exit Criteria

Faza high-level escrow model jest domknieta, gdy:
- role, trust model i tryby dzialania sa jednoznaczne i zamrozone,
- rail assignment jest zamrozony (FullPrivacy),
- relacja do tx_signing_hash, policy_opening i threshold auth jest jawna,
- nexum-core vs ledger split jest jawny,
- follow-up docs (tx matrix, object formats, proof integration) maja jawny scope i sa przypisane do execution spine.

Ten dokument jest finalny na poziomie modelu.
Nie jest jeszcze finalny na poziomie kazdego obiektu i kazdego pola.
