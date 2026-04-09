# Kontekst orkiestracji — stan rozmowy i projektu

Data: 2026-04-09
Cel pliku: zachować pełny stan operacyjny, architektoniczny i wykonawczy tak, żeby można było wznowić pracę bez utraty kontekstu.

## 1. Główna oś projektu

Naszym celem jest dowiezienie pierwszego uczciwego escrow v1 end-to-end dla `privAI`:

`funded note -> Stage A approvals -> Stage B assembly -> proof handoff -> node submit gate -> block/import-ready path`

Pracujemy świadomie warstwami i pilnujemy granicy:
- Stage A = control-plane, proposal, approvals, quorum, authorization material
- Stage B = final assembly, final auth, final canonical signing context

## 2. Najważniejsze ustalenia architektoniczne

### 2.1. FullPrivacy i auth
- `FullPrivacy v1` idzie w wariancie Option B.
- Wszystkie `TransferNoteTx` inputy wymagają auth.
- `policy_opening` jest obowiązkowe.
- `policy_tag` to tylko hint i ma być zgodne z `policy_opening`.

### 2.2. Escrow
- Escrow działa jako `Escrow2of3` na `FullPrivacy`.
- Operator to workflow machine, nie trust anchor.
- `nexum-core` to control-plane, nie execution engine.

### 2.3. Transport split
Zamrożony kierunek v1:
- validator / node-to-node = direct P2P session transport
- NXMS control-plane / mailbox over Tor = store-and-forward
- nie mieszamy validator P2P z NXMS control-plane

To zostało później zapisane w memo transportowym.

## 3. Ważne ograniczenia bezpieczeństwa i zakresu

Nie ruszamy bez audytu modułów:
- `crypto/*`
- `contracts/*`
- `keys/*`
- `withdrawals/*`

Nie robimy deploy/migrate bez review człowieka.
Nie wklejamy sekretów i prywatnych danych.
Nie dociągamy magicznych zależności z niezweryfikowanych źródeł.

## 4. Co zostało już dowiezione

### 4.1. nexum-core / orchestrator
Naprawiony został fałszywy model, w którym Stage A twierdziło, że zna finalny canonical `tx_signing_hash`.
W praktyce:
- `AuthMaterial` i `FinalAssemblyInputs` są rozdzielone
- control-plane nie udaje final assembly

Status: domknięte.

### 4.2. Docs anti-drift
Zsynchronizowane zostały:
- `spec/PRIVAI_ESCROW_OBJECT_MODEL.md`
- `spec/PRIVAI_ESCROW_TX_MATRIX.md`
- `spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md`

Potem doszedł mini-fix dla `EscrowApprovalBundle`, żeby Stage A nie udawało podpisów nad hashem, który jeszcze nie istnieje.

Status: domknięte.

### 4.3. privai-nxms escrow wire layer
Dodane i utwardzone:
- roundtrip tests
- payload roundtrip
- msg_type_key consistency
- body_hash stability

Później usunięto stary Stage A ogon:
- `tx_signing_hash` usunięty z `EscrowSpendProposal`

Status: domknięte.

### 4.4. privai-node Stage A store / wire ingress / persistence
Dowiezione:
- staging funded/proposal/approval
- quorum readiness
- payload ingress `handle_nxms_payload(...)`
- persistence Stage A store
- bundle export `build_escrow_approval_bundle(...)`

Status: domknięte.

### 4.5. approval-layer hardening
Dowiezione:
- `EscrowApprovalBundle::from_approvals_sorted(...)`
- `EscrowApprovalBundle::validate()`
- `EscrowApprovalBundle::is_stage_a()`
- `TX_SIGNING_HASH_STAGE_A`
- testy duplicate signer rejection
- deterministic ordering

Status: domknięte.

### 4.6. wallet Stage B assembly
Dowiezione:
- wallet-side final assembly bridge
- binding do `funding_note_commit`
- walidacja action
- walidacja signerów, duplikatów i ról
- przejście przez właściwy wallet builder path

Potem naprawiono dwa consistency bugs:
- bezpieczne narrowing `u64 -> u16` w `escrow_builder.rs`
- analogiczny fix w `builder.rs`

Status: domknięte.

### 4.7. typed proof/submit bridge
Powstał:
- `privai-wallet/src/proof_handoff.rs`
- `EscrowProofReadyHandoff`
- `EscrowAttachedProof`

Bridge robi:
- spójny handoff tx + `tx_signing_hash` + `TransferProvingData` + `ProofJob`
- attach proof result do `BatchProofArtifact`
- eksport do `BlockProofArtifacts`

W pewnym momencie po awarii WSL na dysku został placeholder/stub, ale został usunięty i zastąpiony realną implementacją.

Status: domknięte.

### 4.8. escrow-aware submit gate w node
Dowiezione:
- `submit_escrow_transfer_note(...)`
- `validate_escrow_transfer_note(...)`
- `verify_escrow_tx_signatures(...)`

Końcowy stan gate:
- proposal musi istnieć
- quorum musi być gotowe
- tx musi być `TransferNote`
- dokładnie jeden auth entry `Escrow2of3`
- `policy_opening` musi istnieć
- `policy_opening` musi się dekodować
- `policy_opening` musi być `SpendPolicy::Escrow2of3`
- pola policy muszą zgadzać się z `EscrowFundingDescriptor` ze staged context:
  - `buyer_pk_hash`
  - `merchant_pk_hash`
  - `operator_pk_hash`
  - `timeout_block`
- `escrow_action` musi zgadzać się z proposalu
- signer set musi zgadzać się z bundle
- podpisy są weryfikowane względem `tx_signing_hash`

Pozytywne testy seedują ledger realnym input note i kończą się prawdziwym `Ok(...)`.

Status: domknięte.

## 5. Co zostało zapisane w osobnym memo architektonicznym

Powstał plik:
- `spec/PRIVAI_TRANSPORT_RUNTIME_FREEZE_MEMO.md`

Główna rekomendacja v1:
- validators = direct P2P session transport
- NXMS control-plane = mailbox store-and-forward over Tor
- proof sidecar = mailbox, nie validator queue
- gossip = validator session transport

Najważniejsze drifty z memo:
1. brak runtime mailbox-to-node loop
2. README / transport warstwa w `nexum-core` może być źle interpretowana jako obejmująca validator P2P
3. proof sidecar bodies istnieją, ale runtime ich jeszcze nie obsługuje

## 6. Status agentów i modeli

### Xiaomi
Przy dobrze opisanych taskach radzi sobie dobrze także z większymi rzeczami.
Dowiózł m.in.:
- wire ingress
- Stage A cleanup
- approval hardening
- persistence
- submit gate

Wniosek operacyjny:
- warto dawać mu większe, bounded runtime/taski
- ważne są: write scope, explicit DoD, zakazy obejść, konkretne acceptance criteria

### Gemini
Dowiózł dobrze:
- orchestrator boundary cleanup
- później typed proof handoff w wallet/proof warstwie

Wniosek:
- dobrze nadaje się do Stage B i mocniejszych tasków integracyjnych

### Claude
Limit został chwilowo zużyty.
Dowiózł dobre memo architektoniczne.
Na później przygotowany jest task:
- `T-03 mailbox pull loop in privai-node runtime`

### Ampere
To nasz wewnętrzny sub-agent uruchomiony z tego wątku.
Id: `019d6f35-39d3-77b2-a56d-ba5299f73ab7`
Nick: `Ampere`

Jego najważniejszy wkład:
- rekonesans Stage B proof/submit bridge
- wskazanie, że główną luką był typed glue layer między wallet assembly, proving data, proof job i attach step

## 7. Aktualna strategia orkiestracji

Najbardziej sensowny bieżący split na 3 agentów:

### Task 12 — Xiaomi A
`privai-node` mailbox runtime loop

### Task 13 — Xiaomi B
validator session regression pack

### Task 14 — Gemini
full honest local escrow e2e

Opis tych tasków został zapisany osobno w:
- `zadania_12_14_opis.md`

## 8. Co jest teraz największym runtime blockerem

Największy brak po stronie runtime:
- realny mailbox pull loop do `handle_nxms_payload(...)`

To jest dokładnie brakujące spięcie:
- `nxms-mailbox-client`
- runtime node
- `handle_nxms_payload(...)`
- ack policy

## 9. Co jest teraz największym blockerem e2e

Największy brak po stronie pełnego escrow e2e:
- jeden uczciwy local integration flow,
który naprawdę przejdzie:
- Stage A staging
- bundle export
- wallet final assembly
- proof handoff
- submit gate
- import-ready artifact path

Bez udawania sukcesu przez częściowe przejście.

## 10. Ważne szczegóły techniczne, których nie wolno zgubić

### 10.1. `EscrowApprovalBundle`
- Stage A artifact
- authorization material, nie final ledger-ready auth
- `TX_SIGNING_HASH_STAGE_A` to sentinel dla Stage A

### 10.2. `EscrowSpendProposal`
- nie niesie już `tx_signing_hash`
- Stage A nie zna finalnego canonical signing hash

### 10.3. `verify_tx_signatures` vs escrow signing
W escrow submit gate trzeba było rozróżnić:
- klasyczne weryfikowanie po `tx_id`
- escrow auth verification po `tx_signing_hash`

To było ważne i zostało poprawione.

### 10.4. signer comparison
W submit gate signer PK porównywane są po hashach PK, nie po pełnych bajtach Falcon PK.

### 10.5. policy_opening
Nie wystarczy `Some(policy_opening)`.
Trzeba:
- dekodować
- wymagać `Escrow2of3`
- porównać do staged funding descriptor

## 11. Commity wykonane wcześniej

W `privAI` były już wcześniej wykonane commity:
- `06f2f32` `build(workspace): centralize rust version metadata`
- `8011197` `feat(escrow-stage): harden stage-a wire and node flow`
- `e73e1ce` `feat(wallet): add escrow assembly builder`
- `54ab473` `build(lockfile): refresh escrow dependency graph`

W `nexum-core`:
- `0f6ea09` `refactor(escrow-orchestrator): split stage-a from final assembly`

Później zostały wykonane kolejne commity w `privAI`:
- `e0d5931` `feat(node): add escrow submit gate`
- `1d1ef40` `feat(wallet): add escrow proof handoff`
- `fef827f` `docs(spec): add transport runtime freeze memo`

## 12. Co świadomie zostawiamy na później

Jeszcze nie domknięte globalnie:
- pełny prover runner
- pełny honest local escrow e2e
- mailbox runtime loop
- validator session regression pack
- decyzje i implementacja proof sidecar runtime path

Także nie robimy teraz:
- redesignu escrow od zera
- mieszania validator P2P z NXMS control-plane
- ruszania `crypto/*`, `contracts/*`, `keys/*`, `withdrawals/*`

## 13. Aktualny stan worktree, którego nie należy przypadkiem wciągnąć

W momencie tworzenia tego pliku w worktree były też inne zmiany, których nie należy mieszać z tym commitem koordynacyjnym:
- lokalny szum w `.gitignore`
- trwające lub świeże zmiany agentów w:
  - `privai-node/src/config.rs`
  - `privai-node/src/net.rs`
  - `privai-node/src/session_impl.rs`
  - `privai-node/tests/validator_session.rs`
  - `privai-node/src/mailbox_pull.rs`
  - `privai-node/tests/escrow_e2e_release.rs`
  - `.kilocode/`

To jest ważne: commity koordynacyjne mają objąć tylko pliki opisowe, bez przypadkowego stage’owania robót agentów.

## 14. Jak rozmawiać z agentami, żeby dawało dobre wyniki

Najlepiej działają taski z:
- jasnym ownerem
- statusem `NOWY TASK` / `TASK NAPRAWCZY`
- bardzo czytelnym write scope
- mocnym `Forbidden`
- mocnym `Definition of Done`
- minimalnymi komendami do odpalenia
- dokładnie określonym formatem wyniku końcowego

To szczególnie dobrze działało z Xiaomi.

## 15. Co jest ważne dla kolejnego operatora / kolejnej sesji

Jeśli trzeba wznowić pracę po przerwie:
1. sprawdź `git status` i nie stage’uj przypadkiem zmian agentów
2. trzymaj rozdział:
   - validator P2P
   - NXMS mailbox control-plane
   - Stage A escrow
   - Stage B wallet/proof
3. korzystaj z `zadania_12_14_opis.md` jako gotowej paczki promptów
4. nie cofaj granicy Stage A / Stage B
5. pamiętaj, że `proof_handoff.rs` był chwilowo stubem po awarii, ale stan docelowy jest już poprawny i testowany

## 16. Krótkie podsumowanie stanu na teraz

To, co jest naprawdę mocne na dziś:
- Stage A escrow model jest uporządkowany
- node submit gate jest uczciwie zamknięty
- wallet Stage B glue layer istnieje
- typed proof handoff istnieje
- transport freeze dla v1 jest spisany

To, co najbardziej opłaca się robić dalej:
- mailbox runtime loop
- validator regression pack
- pełny honest local escrow e2e
