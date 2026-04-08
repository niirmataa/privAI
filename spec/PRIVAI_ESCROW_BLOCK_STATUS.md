# privAI Escrow Block Status

Status: checkpoint doc for the escrow documentation block.
Canonicality: support status document. This file does not override protocol, formats, consensus, proof or product semantics. It exists to record what is already closed in the escrow documentation set, what remains unresolved, and which follow-up work is still required before implementation can be called complete.
Owner: privAI escrow, auth, proof and ledger architecture.
Depends on:
- `spec/PRIVAI_EXECUTION_SPINE.md`
- `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
- `spec/PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`
- `spec/PRIVAI_ESCROW_FINAL_MODEL.md`
- `spec/PRIVAI_ESCROW_TX_MATRIX.md`
- `spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md`
- `spec/PRIVAI_GAP_REGISTER.md`
- `spec/PRIVAI_DECISION_REGISTER.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zapisac aktualny status calego bloku escrow docs,
- odciac zgadywanie, czy escrow jest juz domkniete koncepcyjnie czy jeszcze nie,
- wskazac active source of truth dla escrow,
- nazwac, co jest juz zamrozone,
- nazwac, co nadal pozostaje follow-upem implementacyjnym lub researchowym.

## 2. Escrow Docs In Scope

Aktualny blok escrow docs obejmuje:

- `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
- `spec/PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`
- `spec/PRIVAI_ESCROW_FINAL_MODEL.md`
- `spec/PRIVAI_ESCROW_TX_MATRIX.md`
- `spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md`
- `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md` as context/support material

## 3. Active Source Of Truth For Escrow

Na teraz aktywnym source of truth dla escrow sa:

- `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
- `spec/PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`
- `spec/PRIVAI_ESCROW_FINAL_MODEL.md`
- `spec/PRIVAI_ESCROW_TX_MATRIX.md`
- `spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md`

Rola `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md`:
- context / adaptation / historical design support
- nie powinien nadpisywac nowszych docs high-level i execution-level

## 4. What Is Closed Now

Na poziomie architektury i semantyki zamkniete sa nastepujace rzeczy:

- Escrow jest note-based, nie account-based.
- Escrow nalezy do raila `FullPrivacy`.
- Escrow nie jest tym samym co `MarketplaceSmallPaymentsRail`.
- Finalny model escrow jest policy-constrained `2-of-3`.
- Role escrow sa zamrozone:
  - Buyer
  - Merchant
  - Operator
- Normal mode jest operator-required.
- Recovery mode jest buyer+merchant fallback.
- `tx_signing_hash` jest canonical signing message dla auth.
- Threshold auth semantics sa oddzielone od proof semantics.
- `policy_opening` jest canonical source dla policy reconstruction.
- Escrow actions sa action-bound, nie arbitrary-spend.
- High-level signer/action intent jest zamrozony:
  - `Buyer + Operator` for release to Merchant
  - `Merchant + Operator` for refund to Buyer
  - `Buyer + Merchant` for recovery after timeout
- Escrow v1 jest mixed:
  - proof-covered na note level
  - ledger/auth enforced na policy/action/threshold level
- `ExecutionBundle` i `ProofCertificate` obejmuja escrow spend standardowa sciezka jako current runtime/canonical today path.
- Full PQ privacy dla escrow nie jest obecnie claimowane.

## 5. What Is Not Closed Yet

Nadal nie sa domkniete nastepujace rzeczy:

- exact binary formats escrow objects
- exact canonical field layouts
- exact canonical encoding dla escrow action tags
- exact `policy_opening` field set per policy type
- exact object model formats:
  - `EscrowFundingDescriptor`
  - `EscrowSnapshot`
  - `EscrowSpendProposal`
  - `EscrowApprovalBundle`
- exact proof-aware escrow model
- ukrywanie signer identities przed publicznym obserwatorem
- threshold auth inside proof circuit
- action-type binding inside proof statement/public inputs
- final end-to-end PQ privacy claims dla escrow
- exact dispute semantics beyond timeout recovery
- full implementation alignment between docs and code

## 6. Current v1 Interpretation

V1 escrow nalezy rozumiec tak:

- note/spend correctness korzysta z istniejacego proof path `TransferNoteTx`
- threshold auth, signer membership, operator presence, timeout i action semantics sa egzekwowane przez ledger
- v1 escrow nie czeka na future proof-aware threshold auth
- v1 escrow nie obiecuje jeszcze ukrycia signer identities ani policy semantics przed publicznym obserwatorem

## 7. What Must Not Be Inferred

Nie wolno z obecnego stanu docs inferowac, ze:

- escrow jest juz w pelni zaimplementowane on-chain
- escrow ma juz final binary object model
- escrow ma juz final proof-aware privacy
- proof zastapil threshold auth
- threshold auth zostalo przeniesione do circuit
- `MarketplaceSmallPaymentsRail` jest rownowazny final escrow
- `RecoveryRelease` jest dostepny bez timeout gate
- split settlement jest czescia escrow v1

## 8. Relationship To Other Blocks

Escrow zalezy od:
- auth/signing block
- threshold auth block
- proof boundary block
- chain/ledger correctness block

Escrow nie powinno wyprzedzac:
- final code alignment dla auth
- final code alignment dla threshold verification
- ledger implementation of policy reconstruction
- note/state correctness guarantees

## 9. Next Required Follow-Ups

Najblizsze logiczne follow-upy po bloku escrow docs to:

- code reality check vs escrow docs
- auth implementation alignment with `tx_signing_hash`
- threshold auth implementation alignment with canonical rules
- ledger-side policy reconstruction implementation
- escrow object model doc or canonical formats extension
- escrow regression test plan
- future proof-aware escrow research/design task

## 10. Recommended Implementation Order After Docs

1. Zweryfikowac code reality dla auth modelu.
2. Zweryfikowac code reality dla threshold auth semantics.
3. Zmapowac, co w escrow jest juz implementable, a co jeszcze nie.
4. Domknac canonical escrow object model.
5. Dodac escrow-focused regression scenarios.
6. Dopiero potem wracac do future proof-aware escrow.

## 11. Exit Criteria For Escrow Docs Block

Blok escrow docs mozna uznac za koncepcyjnie spiety, gdy:

- high-level model jest zamrozony
- tx matrix jest jednoznaczna
- proof-vs-ledger split jest jednoznaczny
- auth and threshold dependencies sa jawne
- non-claims sa jawne
- follow-up work jest nazwany i nie jest ukrywany

Ten warunek jest obecnie spelniony.

## 12. Final Assessment

Na teraz escrow docs block jest:
- conceptually closed
- execution-aware
- honest about unresolved items
- ready to serve as context for implementation follow-up

Na teraz escrow docs block nie jest:
- final implementation proof
- final binary format definition
- final privacy claim document
