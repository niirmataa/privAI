# privAI Auth Signing Model

Status: focused support doc for the canonical transaction signing model and authorization boundary in privAI.
Canonicality: supporting auth-semantics document. This document does not override canonical protocol, formats, consensus or product semantics; it records the required signing model, its constraints, and the boundary between auth artifact generation and ledger verification.
Owner: privAI tx/auth, ledger and escrow architecture.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`
- `spec/PRIVAI_EXECUTION_SPINE.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- naprawic i domknac model podpisywania transakcji,
- odciac cykliczna zaleznosc pomiedzy `tx_id` a `auth`,
- zdefiniowac role `tx_signing_hash`,
- zapisac granice:
  - co produkuje warstwa auth,
  - co weryfikuje ledger,
- przygotowac grunt pod:
  - threshold auth,
  - final escrow,
  - poprawne policy-bound signing semantics.

To nie jest dokument o proof.
To nie jest dokument o transport signatures.
To jest dokument o auth signing dla transakcji chainowych.

Scope exclusion:
- `MarketplaceBatchTx` auth pozostaje poza tym modelem threshold auth, chyba ze zostanie jawnie migrowany do niego w przyszlosci.

## 2. Main Problem

Aktualny problem, ktory musi zostac odciety:
- podpis nie moze byc liczony po identyfikatorze transakcji, jesli identyfikator ten sam w sobie obejmuje `auth`.

Taki model jest logicznie zly, bo:
- signing message zalezy od auth artifact,
- auth artifact zalezy od signing message,
- threshold auth i escrow nie maja wtedy stabilnego preimage,
- ledger verification robi sie semantycznie kruche.

Twarda zasada:
- `tx_id` i `tx_signing_hash` nie sa tym samym.

## 3. Final Direction

Finalny model auth signing w `privAI` jest taki:
- `tx_id` pozostaje canonical transaction identifier,
- `tx_signing_hash` jest canonical signing message,
- auth artifacts sa liczone nad `tx_signing_hash`,
- ledger weryfikuje auth package wzgledem `tx_signing_hash`,
- `tx_id` nie moze byc source of truth dla auth signing.

## 4. Definitions

### 4.1. `tx_id`

`tx_id` to:
- canonical transaction identifier,
- hash canonical transaction object,
- identyfikator transakcji dla ledgera, referencji i indeksowania.

`tx_id`:
- moze obejmowac final canonical transaction bytes,
- ale nie powinien byc bezposrednim signing preimage, jesli auth bytes biora udzial w jego liczeniu.

### 4.2. `tx_signing_hash`

`tx_signing_hash` to:
- canonical signing message,
- stabilny preimage dla podpisow auth,
- hash liczony z tx body bez cyklicznej zaleznosci od final auth bytes.

`tx_signing_hash`:
- musi byc niezalezny od finalnego opakowania auth artifacts,
- musi byc jednoznaczny,
- musi byc canonical dla danej tx class i danej action semantics.

Concrete preimage formula:
- final concrete preimage formula musi byc zdefiniowana w canonical formats / auth follow-up doc,
- formula musi obejmowac: domain separator, field ordering i canonical encoding,
- ten dokument zamyka semantyke; formula zamyka bytes.

### 4.3. Auth artifact

Auth artifact to:
- podpis,
- approval,
- signer package,
- threshold auth fragment,
- lub finalny auth bundle,
ktory odnosi sie do `tx_signing_hash`.

## 5. Canonical Rule

Twarda canonical rule:
- podpisy i approvals sa zawsze liczone nad `tx_signing_hash`,
- nie nad `tx_id`.

Ledger rule:
- ledger odtwarza `tx_signing_hash` z transaction body,
- ledger weryfikuje auth package wzgledem odtworzonego `tx_signing_hash`,
- ledger nie ufa zadeklarowanemu z zewnatrz signing hash bez recompute.

## 6. Required Signing Preimage Properties

Finalny `tx_signing_hash` musi spelniac nastepujace warunki:

- [ ] Jest deterministyczny.
- [ ] Jest canonical.
- [ ] Nie zalezy cyklicznie od final auth bytes.
- [ ] Wiaze tx class.
- [ ] Wiaze action semantics.
- [ ] Wiaze inputs.
- [ ] Wiaze outputs.
- [ ] Wiaze fee.
- [ ] Wiaze policy-relevant fields.
- [ ] Daje sie odtworzyc przez ledger bez zgadywania.

## 7. What Must Be Bound By `tx_signing_hash`

Minimalny finalny signing preimage musi wiazac co najmniej:

- tx version
- tx class
- action type albo canonical action semantics
- input references / canonical input body
- output commitments / canonical outputs
- fee
- policy-relevant flags
- policy-relevant commits
- threshold / escrow relevant fields, jesli dotyczy

Wazna zasada:
- signer approvals autoryzuja konkretna akcje,
- nie "jakakolwiek transakcje z tym samym inputem".

## 8. What Must Not Be Bound Circularly

`tx_signing_hash` nie moze bezposrednio zalezec od:
- final signature bytes,
- final auth wrapper bytes,
- final threshold bundle encoding, jesli to encoding jest juz wynikiem podpisywania.

To odcina:
- auth recursion,
- unstable signing preimage,
- brittle multisig semantics.

## 9. Signer Identity Binding

Kazdy signer approval musi byc jednoznacznie zwiazany z:
- signer identity,
- signer public key albo canonical key identifier,
- `tx_signing_hash`,
- signer role, jesli policy tego wymaga.

Ledger musi miec jednoznaczna odpowiedz:
- kto podpisal,
- jakim canonical signer identity,
- czy ten signer nalezy do dozwolonego signer set dla tej transakcji.

## 10. Canonical Signer Ordering

Signer ordering musi byc canonical.

Powod:
- threshold auth package nie moze byc semantycznie wieloznaczny,
- rozne kolejnosci tych samych signerow nie powinny tworzyc roznych znaczen,
- vectors i verification musza byc stabilne.

Finalna zasada:
- signer set jest porzadkowany canonicalnie wedlug zdefiniowanej reguly,
- auth package nie zalezy od przypadkowej kolejnosci z walleta / operatora / sieci.

Preferred ordering rule:
- signer ordering follows canonical signer index in policy,
- final encoding rule musi byc zamrozona razem z canonical formats / auth rules doc.

## 11. Duplicate Signer Rejection

Ta sama tozsamosc signerowa:
- nie moze liczyc sie wielokrotnie,
- nie moze byc policzona jako dwa glosy przez rozne reprezentacje tego samego klucza,
- nie moze obejsc threshold semantics.

Ledger rule:
- duplicate signer material jest odrzucane albo redukowane do jednego canonical signer identity,
- ale nie moze zwiekszac signer count.

## 12. Threshold Auth Package

Finalny threshold auth package musi miec jawna semantyke:
- signer set
- signer identities
- signer approvals
- threshold rule
- optional role binding
- optional action binding

To nie znaczy jeszcze, ze ten dokument zamyka final binary format.
Ten dokument zamyka semantyke, ktora format musi przenosic.

Policy reconstruction binding:
- threshold auth package jest zawsze weryfikowany wzgledem policy zrekonstruowanej z canonical policy material / `policy_opening`,
- threshold auth package nie istnieje "sam z siebie" — jest walidowany wzgledem reconstructed signer set i threshold rule z policy.

## 13. `nexum-core` vs `privAI`

### 13.1. `nexum-core`

`nexum-core` ma byc odpowiedzialne za:
- key management / signer coordination,
- approval orchestration,
- building signer approvals over `tx_signing_hash`,
- canonical assembling of signer material before handoff,
- wallet/vault integration.

### 13.2. `privAI`

`privAI` ma byc odpowiedzialne za:
- canonical definition of `tx_signing_hash`,
- canonical signer identity rules,
- canonical threshold semantics,
- ledger verification,
- policy validation,
- action validation,
- state transition.

Twarda zasada:
- `nexum-core` moze pomagac budowac auth package,
- ale ledger `privAI` finalnie weryfikuje correctness.

## 14. Ledger Verification Boundary

Ledger musi sprawdzic co najmniej:
- czy `tx_signing_hash` da sie odtworzyc z tx body,
- czy auth artifacts odnosza sie do tego samego `tx_signing_hash`,
- czy signer identities sa poprawne,
- czy signer identities naleza do allowed signer set,
- czy signer count spelnia threshold,
- czy duplicate signer material nie liczy sie wielokrotnie,
- czy action semantics sa zgodne z policy.

To jest finalna granica:
- proof nie zastepuje auth verification,
- wallet/operator/orchestrator nie zastepuje ledger verification.

## 15. Relationship To Escrow

Finalny escrow zalezy bezposrednio od tego dokumentu.

Powod:
- escrow wymaga poprawnego `tx_signing_hash`,
- escrow wymaga canonical signer ordering,
- escrow wymaga policy-bound action semantics,
- escrow wymaga threshold auth package, ktory ledger potrafi jednoznacznie zweryfikowac.

Bez tego:
- `2-z-3`,
- operator-required mode,
- recovery path,
nie maja stabilnego fundamentu.

## 16. Relationship To Proof

Proof layer:
- moze pozniej bindzic pewne policy-relevant fields,
- moze pozniej byc zwiazany z `tx_signing_hash` lub auth-relevant public fields,
- ale proof nie zastepuje samej autoryzacji.

Twarda zasada:
- podpisy i approvals nie sa "zastepowane przez proof".
- proof i auth to osobne warstwy.

## 17. Current Implementation Gap

Ten dokument zaklada, ze obecny projekt wymaga przejscia z:
- signing over `tx_id`
na:
- signing over `tx_signing_hash`

Dopoki to nie jest wykonane:
- auth model pozostaje architektonicznie niedomkniety,
- threshold auth pozostaje ryzykowny,
- final escrow nie powinno byc claimowane jako bezpiecznie zamkniete.

## 18. Checklist

- [ ] Jawnie zdefiniowac `tx_signing_hash`.
- [ ] Rozdzielic `tx_id` od signing preimage.
- [ ] Zdefiniowac canonical fields w signing preimage.
- [ ] Zdefiniowac canonical signer identity binding.
- [ ] Zdefiniowac canonical signer ordering.
- [ ] Zdefiniowac duplicate signer rejection.
- [ ] Zdefiniowac threshold auth package semantics.
- [ ] Zdefiniowac `nexum-core` vs ledger split.
- [ ] Dodac testy regresyjne dla starego bledu cyklicznego podpisu.
- [ ] Dodac vectors dla signing preimage.
- [ ] Zdefiniowac relacje `tx_signing_hash` do istniejacych canonical formats (domain, field order, canonical encoding).

## 19. Exit Criteria

Faza auth/signing model jest domknieta dopiero wtedy, gdy:
- `tx_signing_hash` istnieje i jest canonical,
- `tx_id` nie jest juz auth signing message,
- signer rules sa jednoznaczne,
- threshold auth semantics sa jednoznaczne,
- escrow ma stabilny fundament auth.

## 20. Final Assessment

Finalny model auth w `privAI` ma byc rozumiany tak:

- `tx_id` identyfikuje transakcje,
- `tx_signing_hash` jest canonical signing message,
- auth artifacts odnosza sie do `tx_signing_hash`,
- ledger weryfikuje signer semantics i policy semantics,
- `nexum-core` pomaga wygenerowac auth material,
- ale nie przejmuje roli ledgera.

To jest konieczny fundament przed finalnym escrow i threshold auth.
