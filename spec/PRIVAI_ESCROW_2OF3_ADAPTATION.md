# privAI Escrow 2-of-3 And nexum-core Adaptation

Status: focused architecture target doc for escrow adaptation and coin-side threshold authorization work.
Canonicality: binding focused direction for escrow architecture, coin-side multisig-style authorization and `nexum-core` adaptation. This document does not override the canonical spec set; it narrows and operationalizes the escrow direction already implied by the freeze, protocol, marketplace and consensus docs.
Owner: privAI protocol and integration architecture.
Depends on:
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_PROOF_BOUNDARIES.md`
- `spec/PRIVAI_DECISION_REGISTER.md`
- `spec/PRIVAI_GAP_REGISTER.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zapisac finalny kierunek escrow `2 z 3` dla `privAI`,
- zapisac, jak `nexum-core` ma byc zaadaptowany,
- zapisac, czego brakuje po stronie coina,
- zablokowac dopowiadanie "multisig jak w Monero" tam, gdzie system nie dziala w ten sposob,
- dac jedno miejsce do powrotu do tematu bez odtwarzania calego kontekstu od zera.

Ten dokument nie jest:
- nowym railem,
- zamiennikiem `PRIVAI_PROTOCOL_CORE.md`,
- zamiennikiem `PRIVAI_CANONICAL_FORMATS.md`,
- opisem Monero multisig.

## 2. Frozen Boundary

### 2.1. Frozen

- Escrow `2 z 3` nalezy do `FullPrivacy`.
- Escrow `2 z 3` nie nalezy do `MarketplaceSmallPaymentsRail`.
- Escrow `2 z 3` nie nalezy do `OnChainLite`.
- `nexum-core` ma byc zaadaptowany jako workflow/control-plane, a nie jako Monero execution engine.
- `privAI` v1 escrow ma byc egzekwowany przez threshold authorization nad spendem noty, a nie przez wspolny Monero-style multisig wallet.

### 2.2. Current Canonical

- Obecnym on-chain nosnikiem polityki `2 z 3` jest `SpendPolicy::MarketplaceSettlement`.
- Obecny proof plane nie egzekwuje jeszcze finalnej semantyki escrow threshold auth.
- Obecny model auth w coinie nie wystarcza jeszcze do prawdziwego `2 z 3`.

### 2.3. Future Target Requiring Migration

- Mozliwa przyszla migracja nazwy `SpendPolicy::MarketplaceSettlement` do bardziej ogolnej nazwy escrow/threshold.
- Mozliwe przyszle ukrycie threshold auth w bardziej zaawansowanym modelu proof-aware spend.
- Mozliwe przyszle rozszerzenie timeout/refund/dispute semantics na bardziej jawne on-chain bindings.

### 2.4. Unresolved

- Finalna nazwa canonical policy typu escrow `2 z 3`.
- Czy timeout semantics pozostanie tylko w `policy_opening + off-chain snapshot`, czy dostanie dodatkowe on-chain wiazanie.
- Czy przyszly model privacy/auth dla escrow bedzie public-auth-threshold only czy proof-aware threshold.

### 2.5. Forbidden Inference

- Nie wolno zakladac, ze escrow `2 z 3` ma dzialac jak Monero multisig, skoro `privAI` jest systemem note-based.
- Nie wolno zakladac, ze `MarketplaceSmallPaymentsRail` nadaje sie do escrow tylko dlatego, ze ma `buyer/seller/moderator`.
- Nie wolno zakladac, ze obecny `tx_id`-based auth jest wystarczajacy do realnego multisiga.
- Nie wolno zakladac, ze brak wspolnego multisig address oznacza brak prawdziwego `2 z 3`.

## 3. Zamrozony Kierunek

Finalny kierunek jest nastepujacy:

- buyer finansuje escrow przez `FullPrivacy` note,
- note ma policy commit, ktory otwiera sie do polityki `2 z 3`,
- `nexum-core` trzyma workflow escrow, snapshot, proposal i approval bundle,
- `privAI coin` waliduje sam spend noty i threshold auth,
- release/refund/dispute/timeout sa modelowane najpierw w control-plane i proposal layer,
- on-chain v1 egzekwuje przede wszystkim:
  - poprawny spend,
  - poprawny threshold auth,
  - nullifier rules,
  - state transition.

To jest multisig escrow w sensie:
- srodki nie moga byc wydane bez quorum `2 z 3`,
- chain egzekwuje ten prog,
- ale system nie tworzy wspolnego Monero-style multisig walleta.

## 4. Czym Ten Model Nie Jest

Ten model nie jest:
- Monero multisig z rundami `R1/R2/R3`,
- wspolnym adresem multisig walleta,
- wspolnym `txset_hex`,
- integracja `wallet-rpc`,
- kolejnym railem platnosci.

To jest:
- note spend z threshold authorization,
- workflow-driven escrow,
- off-chain coordination + on-chain validation.

## 5. Rola Raili

### 5.1. `FullPrivacy`

To jest jedyny prawidlowy rail dla escrow `2 z 3`.

Powody:
- rail jest juz przeznaczony do wiekszych i bardziej wrazliwych flow,
- rail jest note-based i pasuje do lockowania srodkow w notach,
- rail nie miesza settlement accounting z escrow authorization.

### 5.2. `MarketplaceSmallPaymentsRail`

Ten rail nie jest do escrow `2 z 3`.

Powody:
- to jest rail operator-trusted accounting,
- sluzy do grant/receipt/batch settlement,
- nie jest modelem blokowania pelnej wartosci escrow note i jej threshold release.

### 5.3. `OnChainLite`

Ten rail nie jest baza dla escrow `2 z 3`.

Powody:
- jest nadal `Experimental`,
- nie ma finalnego proof/state/threshold closure,
- nie powinien byc miejscem dla wrazliwego flow escrow.

## 6. Model Techniczny `2 z 3`

### 6.1. Uczestnicy

Escrow `2 z 3` ma trzy role:
- buyer
- seller
- moderator

Kazda rola ma:
- klucz publiczny Falcon,
- klucz prywatny Falcon.

### 6.2. Policy

Funding note ma `spend_policy_commit`.

Ten commit otwiera sie do polityki zawierajacej co najmniej:
- `buyer_pk_hash`
- `seller_pk_hash`
- `moderator_pk_hash`
- `timeout_block`

Obecnie technicznym nosnikiem tej polityki jest:
- `SpendPolicy::MarketplaceSettlement`

### 6.3. Spend

Aby wydac escrow note:
- budowany jest jeden konkretny `EscrowSpendProposal`,
- co najmniej dwie z trzech rol podpisuja ten sam komunikat podpisu,
- signed bundle trafia do transakcji,
- ledger sprawdza prog `2 z 3`.

### 6.4. Czym jest multisig w tym modelu

W tym modelu multisig oznacza:
- wiele niezaleznych podpisow Falcon,
- quorum sprawdzane przez ledger,
- policy-aware authorization nad spendem noty.

To nie jest:
- jeden zagregowany threshold signature,
- wspolny wallet address,
- wspolny secret key.

## 7. Jak To Nie Jest Robione Jak W Monero

### 7.1. Monero-style model

Monero multisig robi:
- wspolny wallet,
- wspolny adres,
- wymiane multisig danych i rund,
- wspolny txset do podpisania i submitu.

### 7.2. `privAI` model

`privAI` robi:
- normalna note jest ufundowana do escrow targetu,
- note ma polityke wydania `2 z 3`,
- runtime escrow ma zdolnosc odbioru i wykonania spendu,
- buyer/seller/moderator daja approval signatures,
- ledger sprawdza threshold rule.

### 7.3. Wniosek

Nie implementujemy Monero multisig.

Adaptujemy tylko:
- workflow,
- snapshot,
- proposal,
- admission,
- approval bundle,
- replay/idempotency discipline.

## 8. Jak Powstaje Funding Escrow

### 8.1. Nie ma wspolnego multisig address

W `privAI` nie powstaje wspolny adres multisig w stylu Monero.

Zamiast tego powstaje:
- `EscrowFundingDescriptor`

### 8.2. `EscrowFundingDescriptor`

To jest paczka danych potrzebna do ufundowania escrow note.

Powinien zawierac co najmniej:
- `escrow_id`
- `ReceiveBundle`
- `spend_policy_commit`
- `expiry`
- opcjonalny `context_commit`
- opcjonalny `descriptor_version`

### 8.3. Funding flow

1. `nexum-core` otwiera escrow i ustala role oraz timeout.
2. Escrow runtime generuje jednorazowy `ReceiveBundle`.
3. System tworzy polityke `2 z 3` i liczy `spend_policy_commit`.
4. Z tych danych powstaje `EscrowFundingDescriptor`.
5. Buyer wallet buduje `FullPrivacy OutputNote` do tego descriptora.
6. Escrow runtime wykrywa funding note i od tego momentu note jest lockiem escrow.

## 9. Podzial Odpowiedzialnosci

### 9.1. `privAI coin`

Coin odpowiada za:
- note existence/spend checks,
- nullifier derivation i replay protection,
- spend policy opening verification,
- threshold auth verification,
- state transition.

### 9.2. `nexum-core`

`nexum-core` odpowiada za:
- workflow escrow,
- snapshot kontraktu,
- proposal release/refund/dispute,
- zbieranie approval signatures,
- timeout/dispute orchestration,
- replay/idempotency na poziomie control-plane.

### 9.3. Escrow runtime / executor

Escrow runtime odpowiada za:
- receive capability dla escrow funding note,
- skladanie finalnego spendu po zebraniu approvals,
- broadcast finalnej transakcji.

Wazne:
- receive/execution capability i approval authority sa rozdzielone.

## 10. Obiekty, Ktore Trzeba Dobudowac

### 10.1. `EscrowSnapshot`

Off-chain canonical object opisujacy kontrakt escrow.

Minimalny zakres:
- `escrow_id`
- `buyer_id`
- `seller_id`
- `moderator_id`
- `funding_context`
- `release_rule`
- `refund_rule`
- `timeout_rule`
- `fee_cap`
- `created_at`
- `snapshot_hash`

Cel:
- wszyscy podpisuja proposal odnoszacy sie do tego samego snapshotu.

### 10.2. `EscrowSpendProposal`

Off-chain canonical object opisujacy konkretny spend escrow note.

Minimalny zakres:
- `escrow_id`
- `snapshot_hash`
- `action`
- `input_note_commits`
- `output_plan`
- `fee`
- `height_or_timeout_context`
- `proposal_hash`

`action` powinno co najmniej wspierac:
- `release`
- `refund`

Mozliwe przyszle rozszerzenie:
- `partial_release`
- `dispute_resolution`

### 10.3. `EscrowApprovalBundle`

Off-chain bundle approvals zbierany przez `nexum-core`.

Minimalny zakres:
- `proposal_hash`
- `signer_roles`
- `signer_pks`
- `signatures`
- `created_at`

V1:
- bez kryptograficznej agregacji threshold signatures,
- wystarcza dwa osobne podpisy Falcon.

### 10.4. `EscrowFundingDescriptor`

Funding descriptor dla walleta buyer.

Minimalny zakres:
- `escrow_id`
- `ReceiveBundle`
- `spend_policy_commit`
- `expiry`
- `context_commit`

## 11. Zmiany Wymagane Po Stronie Coina

### 11.1. Naprawa modelu podpisu

Obecny model auth nie jest wystarczajacy, bo podpis jest sprawdzany wzgledem `tx_id`, a `tx_id` obejmuje `auth`.

To tworzy cykliczna zaleznosc.

Trzeba wprowadzic osobny komunikat podpisu:
- `tx_signing_hash`
albo
- `input_auth_message`

Minimalny target:
- hash z `tx_without_auth`
- plus `input_index`
- plus `referenced_note_commit`
- plus `policy_tag`

### 11.2. `policy_opening` w auth

Ledger nie moze egzekwowac `2 z 3`, jesli zna tylko `spend_policy_commit`.

Trzeba dostarczyc `policy_opening` przy spendzie.

Minimalny target:
- `InputAuthV2`
  - `policy_tag`
  - `policy_opening`
  - `signer_pks`
  - `signatures`

### 11.3. Policy-aware auth validation

Ledger musi przestac sprawdzac tylko "czy podpis Falcon jest poprawny".

Musi zaczac sprawdzac:
- czy `policy_opening` po commitcie daje oczekiwany `spend_policy_commit`,
- czy signer nalezy do polityki,
- czy policy wymaga `1 z 1` czy `2 z 3`,
- czy podpisy pochodza od roznych dozwolonych rol.

Minimalna regula v1:
- `Single` -> dokladnie 1 poprawny signer zgodny z policy
- `MarketplaceSettlement` -> co najmniej 2 poprawne i rozne signery z buyer/seller/moderator

### 11.4. Auth musi byc obowiazkowe

Brak auth nie moze byc dozwolony dla spendu escrow.

Docelowo:
- input-spending tx bez auth = reject

### 11.5. Timeout semantics

`timeout_block` istnieje juz w polityce.

Do ustalenia pozostaje:
- jak duzo timeout semantics ma byc egzekwowane bezposrednio przez chain v1,
- a jak duzo pozostaje w `EscrowSnapshot` i `EscrowSpendProposal`.

Minimalny sensowny target v1:
- chain egzekwuje threshold auth i timeout field availability,
- `nexum-core` egzekwuje pelny business workflow timeout/refund path.

### 11.6. Proof plane nie jest prerequisite dla v1

V1 escrow `2 z 3` nie powinien byc blokowany oczekiwaniem na finalny proof-aware threshold model.

Minimalny target v1:
- ledger-enforced threshold auth,
- poprawny note spend,
- poprawny state transition.

## 12. Zmiany Wymagane Po Stronie `nexum-core`

### 12.1. Co reuse

Do reuse nadaje sie:
- workflow state machine,
- snapshot pattern,
- admission artifact pattern,
- action token pattern,
- approval/proof bundle pattern,
- replay/idempotency discipline,
- inbox/outbox orchestration.

### 12.2. Co odrzucic

Nie przenosimy:
- Monero multisig rounds,
- Monero `wallet-rpc`,
- `multisig_info`,
- `txset_hex`,
- wspolnego Monero multisig address modelu,
- Monero-specific custody bridge.

### 12.3. Jak przepiac execution model

NXMS `txset_hash` nalezy zastapic:
- `proposal_hash`

NXMS signer action nalezy zastapic:
- podpisem Falcon nad `proposal_hash` albo finalnym `input_auth_message`

NXMS submit action nalezy zastapic:
- broadcastem `TransferNoteTx` z auth bundle.

## 13. Przeplyw Techniczny V1

1. Escrow zostaje otwarte w `nexum-core`.
2. Powstaje `EscrowSnapshot`.
3. Powstaje `EscrowFundingDescriptor`.
4. Buyer funduje escrow przez `FullPrivacy` note.
5. Escrow runtime wykrywa funding.
6. Gdy trzeba wykonac akcje, `nexum-core` tworzy `EscrowSpendProposal`.
7. Buyer/seller/moderator podpisuja ten sam `proposal_hash`.
8. `nexum-core` sklada `EscrowApprovalBundle`.
9. Escrow runtime sklada `TransferNoteTx`.
10. `TransferNoteTx` niesie:
    - input refs,
    - nullifier,
    - outputs,
    - auth bundle,
    - `policy_opening`.
11. Ledger sprawdza:
    - input existence,
    - input unspent,
    - policy opening match,
    - Falcon signatures,
    - role membership,
    - quorum `2 z 3`,
    - normalna walidacje spendu.
12. Spend przechodzi i escrow note zostaje wydana.

## 14. Checkpointy Wdrozenia

### CP-00. Freeze architektury

Rezultat:
- ten dokument zostaje zaakceptowany jako focused target,
- escrow `2 z 3` jest formalnie przypisane do `FullPrivacy`.

### CP-01. Signing hash

Do zrobienia:
- dodac osobny `tx_signing_hash` / `input_auth_message`.

Done condition:
- podpis nie zalezy od `auth`,
- nie ma cyklicznej zaleznosci z `tx_id`.

### CP-02. `InputAuthV2`

Do zrobienia:
- dodac `policy_opening`,
- ustalic exact field layout dla auth witness.

Done condition:
- ledger potrafi odtworzyc i sprawdzic polityke inputu.

### CP-03. Threshold verification

Do zrobienia:
- dodac policy-aware auth verification,
- dodac `1 z 1` i `2 z 3` rules.

Done condition:
- escrow spend przechodzi tylko przy poprawnym quorum.

### CP-04. Mandatory auth

Do zrobienia:
- brak auth dla spend inputu musi byc odrzucany.

Done condition:
- escrow note nie da sie wydac bez auth.

### CP-05. Escrow object model

Do zrobienia:
- zdefiniowac `EscrowSnapshot`,
- zdefiniowac `EscrowSpendProposal`,
- zdefiniowac `EscrowApprovalBundle`,
- zdefiniowac `EscrowFundingDescriptor`.

Done condition:
- `nexum-core` ma canonical control-plane objects dla `privAI`.

### CP-06. `nexum-core` adaptation

Do zrobienia:
- przepiac workflow z `txset_hash` na `proposal_hash`,
- odciac Monero-specific execution path.

Done condition:
- `nexum-core` orchestruje escrow `privAI` bez Monero runtime assumptions.

### CP-07. End-to-end escrow test

Minimalne scenariusze:
- buyer + seller -> release OK
- buyer + moderator -> refund OK
- seller + moderator -> OK zgodnie z policy
- buyer alone -> reject
- seller alone -> reject
- moderator alone -> reject
- signer spoza policy -> reject
- wrong `policy_opening` -> reject
- replay nullifier -> reject

Done condition:
- coin i orchestrator przechodza pelny flow `2 z 3`.

## 15. Architektura, Ktora Trzeba Dobudowac, Udoskonalic I Poprawic

### 15.1. Do dobudowania

- `EscrowFundingDescriptor`
- `EscrowSnapshot`
- `EscrowSpendProposal`
- `EscrowApprovalBundle`
- coin-side `tx_signing_hash`
- coin-side `InputAuthV2`

### 15.2. Do udoskonalenia

- ledger auth validation
- role-aware threshold verification
- timeout/refund/release orchestration boundaries
- mapping `nexum-core` action tokens do `privAI` proposal modelu

### 15.3. Do poprawienia

- obecny auth model oparty o `tx_id`
- optional auth path dla spendow
- brak jawnego `policy_opening` w spend auth
- brak current escrow-specific canonical object modelu

## 16. Rzeczy Zabronione

- nie wolno robic nowego escrow raila tylko po to, zeby uruchomic `2 z 3`,
- nie wolno wciskac escrow do `MarketplaceSmallPaymentsRail`,
- nie wolno opierac escrow v1 o `OnChainLite`,
- nie wolno implementowac Monero multisig rounds w `privAI`,
- nie wolno udawac, ze obecny `tx_id` auth jest poprawnym finalnym modelem multisig,
- nie wolno czekac na finalny proof-aware threshold system, jesli ledger-enforced v1 jest wystarczajace do uruchomienia escrow.

## 17. Otwarte Follow-upy

- Czy obecna nazwa `SpendPolicy::MarketplaceSettlement` powinna zostac pozniej zmigrowana do bardziej ogolnej nazwy escrow/threshold.
- Jak dokladnie zwiazac timeout semantics miedzy chainem i `nexum-core`.
- Czy przyszly private/auth model ma ukrywac roles/quorum przez proof layer.
- Czy release/refund outputs maja miec dodatkowy canonical `escrow_context_commit`.

## 18. Finalna Odpowiedz: Czy To Jest Multisig Jak Monero

Nie.

To jest:
- prawdziwy `2 z 3` enforced przez chain,
- ale realizowany jako threshold authorization nad spendem noty,
- a nie jako wspolny Monero-style multisig wallet/address/txset runtime.

To oznacza:
- podobienstwo biznesowe do Monero multisig: tak,
- zgodnosc techniczna z Monero multisig: nie.
