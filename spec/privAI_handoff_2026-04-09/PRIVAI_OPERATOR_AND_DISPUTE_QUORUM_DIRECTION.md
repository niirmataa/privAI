# privAI Operator And Dispute / Quorum Direction

Date: 2026-04-10
Status: `frozen direction` companion note
Canonicality: derived from the accepted 2026-04-09/2026-04-10 production-direction handoff docs. This note freezes the operator / settlement / recovery / dispute-quorum boundary in one place. It does not redesign escrow, replace code-confirmed Stage A / Stage B behavior, or override protocol-level specs.

Primary source docs:
- `TASK_LOG.md`
- `spec/privAI_handoff_2026-04-09/PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md`
- `spec/privAI_handoff_2026-04-09/PRIVAI_PRODUCTION_SYSTEM_DIAGRAMS.md`
- `spec/privAI_handoff_2026-04-09/PRIVAI_TOR_GATED_NETWORK_DIRECTION.md`
- `spec/privAI_handoff_2026-04-09/PRIVAI_STAGE_A_STAGE_B_CONTRACT_FREEZE.md`
- `spec/privAI_handoff_2026-04-09/OPERATOR_CHEATSHEET.md`

Terminology note:
- marketplace wording often says `provider`
- escrow auth wording often says `merchant`
- in this note, `provider` and `merchant` refer to the same settlement-side role

---

## 1. Purpose

Freeze one honest, canonical statement of:
- who the operator is
- what the operator is not
- how normal settlement works
- what recovery means
- how dispute resolution fits at the system level
- where quorum belongs
- what is frozen now versus what is still future strengthening

This document is intentionally a freeze note, not a redesign note.

---

## 2. Canonicality / status

`frozen direction`
- The accepted production direction is that normal escrow settlement remains operator-signed.
- The accepted production direction is that timeout recovery remains the peer path.
- The accepted production direction is that dispute quorum is a future strengthening path, not the mandatory baseline for every contract.

`current production model`
- Release = buyer + operator
- Refund = merchant + operator
- Recovery = buyer + merchant after timeout, no operator

`code-confirmed alignment`
- The current signer paths above match the Stage A / Stage B freeze and its e2e references.

`not yet frozen`
- quorum sizing
- quorum selection algorithm
- trust weighting
- zk-backed reputation mechanics
- exact binding of future dispute verdicts into the protocol

---

## 3. Operator model

`frozen direction`
- The operator is a system program / rule executor.
- The operator acts under protocol and marketplace rules, not subjective preference.
- The operator role exists to validate hard settlement conditions and co-sign normal settlement actions under the current production model.

`current production model`
- Normal settlement still requires operator participation.
- Release path: buyer signs Release, operator validates rules and co-signs, funds settle to the provider / merchant.
- Refund path: merchant signs Refund, operator validates rules and co-signs, funds settle back to the buyer.

Operator validation is rule-bound. It checks at least the accepted direction already frozen elsewhere:
- contract hash matches the locked contract
- delivery hash was committed by the provider
- the relevant party signed the requested action
- timeout constraints are respected
- signatures are valid

`current production model`
- Phase 0 may still use a centralized dev-team keypair as the operator implementation.

`frozen direction`
- Even when the implementation is centralized today, the role semantics are still "rule executor," not "discretionary admin."
- Operator implementation may evolve into an automated service with published rules without changing the normal settlement invariant.

---

## 4. What the operator is not

`frozen direction`
- The operator is not a discretionary human marketplace moderator.
- The operator is not a centralized support-team judgment call.
- The operator is not a platform admin free to override contract terms.
- The operator is not a substitute for buyer/provider agreement on scope, price, or acceptance criteria.
- The operator is not the timeout recovery signer.

`do not infer`
- Do not describe the operator as "central platform admin."
- Do not describe normal settlement as operator-optional.
- Do not describe operator action as vague quality moderation detached from the contract.

---

## 5. Contract-first settlement model

`frozen direction`
- The contract is primary.
- Contract acceptance happens before escrow lock.
- The locked escrow is bound to that accepted contract.

Settlement and dispute evaluation are contract-first:
- the provider declares contract terms and verification conditions
- the buyer accepts or rejects those terms before lock
- escrow locks only after acceptance
- later settlement is evaluated against that accepted contract, not against vague after-the-fact quality claims

`current production model`
- delivery proof, verification outcomes, acceptance / rejection actions, and timeout state all sit under the already accepted contract boundary

`frozen direction`
- If scope changes materially, the clean path is still refund old escrow, accept a new contract, and lock a new escrow.

---

## 6. Normal settlement paths

`current production model`
- Normal settlement means Release or Refund under the operator-signed escrow model.

### Release

`current production model`
- provider commits `delivery_hash`
- buyer verifies against the accepted contract and verifier conditions
- if accepted, buyer signs Release
- operator validates the hard rules and co-signs
- settlement pays the provider / merchant

### Refund

`current production model`
- if the outcome should return funds and the provider / merchant agrees, the provider / merchant signs Refund
- operator validates the hard rules and co-signs
- settlement returns funds to the buyer

`frozen direction`
- Normal settlement is procedural and rule-bound.
- Normal settlement is not a free-form moderation workflow.
- There is no normal Release or Refund path without operator signature under the currently frozen production model.

`where quorum fits now`
- In the current production model, quorum at settlement time is the escrow authorization threshold for the specific action path.
- For normal settlement, the required signer set includes the operator.
- The current frozen quorum is signer quorum over escrow authorization, not a marketplace jury on every case.
- This is not the same thing as a future dispute jury or reputation-weighted panel.

---

## 7. Timeout recovery path

`frozen direction`
- Recovery is a separate path from normal settlement.
- Recovery exists after timeout.
- Recovery remains the peer path.

`current production model`
- Recovery uses buyer + merchant signatures after timeout.
- Recovery does not require operator signature.

Recovery means:
- normal operator-signed settlement did not complete within the allowed window, or the case remained unresolved until timeout
- the escrow can still close through the peer path defined by the escrow rules

`do not infer`
- Recovery is not the same as normal Release / Refund.
- Recovery is not a discretionary operator override.
- Recovery is not the same as a future dispute quorum mechanism.

---

## 8. Future dispute quorum direction

`future strengthening`
- A future dispute path may use a quorum of independently selected providers.
- Quorum members should evaluate the case against the accepted contract and the recorded evidence, not against vague taste or platform preference.
- Quorum members should sign their decision with Falcon.
- This should be a procedural, rule-bound resolution path.

What that means at the system level:
- dispute resolution should remain contract-first
- the contract and verifier conditions stay primary
- the quorum is for harder unresolved cases, not for every successful settlement
- the mechanism should behave like a bounded protocol process, not like centralized customer support

`not yet frozen`
- exact quorum size
- how quorum members are selected
- whether selection is random, stake-aware, trust-aware, or hybrid
- what evidence packet is mandatory
- how verdicts become settlement-authorizing protocol actions
- appeal or re-open rules

`do not infer`
- Weighted quorum is not current protocol reality.
- zk-backed dispute reputation is not current protocol reality.
- Future dispute quorum is not the current mandatory baseline for every contract.

---

## 9. Trust / reputation boundary

`current production model`
- The system does not depend on a finished reputation system in order to ship the current escrow and settlement model.

`future strengthening`
- trust accumulation
- stake-weighted credibility
- validator participation history
- provider participation history
- dispute-resolution trust
- availability / reliability history
- zk-backed reputation credentials

These future trust inputs may later support:
- stronger dispute resolution
- weighted quorum selection
- better participant selection for sensitive roles

`not yet frozen`
- the scoring model
- the credential format
- the slashing / bonding relation
- how trust changes quorum selection or vote weight

`do not infer`
- Do not turn future trust ideas into present-tense protocol claims.
- Do not claim a finished reputation layer already exists.

---

## 10. What is frozen now

`frozen direction`
- Operator = system program / rule executor.
- Operator is not a discretionary moderator.
- Contract acceptance happens before escrow lock.
- The contract is the primary object for later settlement and dispute evaluation.
- Normal Release / Refund remain operator-signed in the current production model.
- Timeout recovery remains the peer path after timeout.
- Disputes should be judged against contract terms and recorded evidence, not vague quality language.
- Current shipping direction does not require a finished trust or reputation system.

`current production model`
- Release: buyer + operator
- Refund: merchant + operator
- Recovery: buyer + merchant after timeout

`boundary freeze`
- Normal settlement, timeout recovery, and future dispute quorum are three different mechanisms and must not be merged into one description.
- Current escrow quorum means signer quorum on the escrow action, not a standing dispute panel.

---

## 11. What remains intentionally open

`not yet frozen`
- whether future dispute quorum is optional per contract, marketplace-default, or protocol-default
- exact quorum membership rules
- exact quorum selection algorithm
- exact trust / stake weighting model
- exact signed verdict format
- exact on-chain or submit-gate binding for future dispute verdicts
- whether future dispute roles include only providers or a broader class of independent participants
- exact appeals, liveness, and failure handling for quorum-based disputes

These are open by design. They should be frozen later as dedicated dispute/trust decisions, not smuggled into the current production model.

---

## 12. Non-goals / do not infer

This document does not:
- remove the operator from normal settlement
- convert the operator into a platform moderator
- claim that weighted quorum already exists
- claim that zk-backed reputation already exists
- collapse recovery into dispute quorum
- make dispute quorum the baseline requirement for every contract
- redesign pricing, transport topology, marketplace discovery, or UI
- claim that semantic quality is solved automatically outside the accepted contract and verifier boundary

The honest current model is:
- contract-first settlement
- operator-signed normal Release / Refund
- peer timeout recovery
- future procedural dispute quorum as strengthening, not as present-tense protocol reality
