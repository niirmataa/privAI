# Zadania 12-14 — split na 3 agentów

Data przygotowania: 2026-04-09
Status: gotowe do odpalenia równolegle

## Zadanie 12 — Xiaomi A
### privai-node mailbox runtime loop

Pracujesz tylko w WSL.

Repo:
`/home/nxms-server/privAI`

Dodatkowo możesz czytać read-only:
`/home/nxms-server/nexum-core`

Nie pracuj na ścieżkach Windows.
Nie używaj `C:\...`.

To jest NOWY TASK.
Nie kontynuacja starego tasku.
Nie dotykasz `nexum-core`.
Nie dotykasz escrow e2e ani validator regression tasków wykonywanych równolegle.

#### Twoja rola
Zbudować brakujący mailbox runtime loop w `privai-node`.

To jest główny runtime blocker z transport freeze memo:
- `handle_nxms_payload(...)` istnieje
- `nxms-mailbox-client` istnieje
- ale node runtime nie ma jeszcze działającego loopa:
  - pull
  - ingest
  - ack

Masz to domknąć.

#### Główny cel
Node ma umieć:
1. pobrać wiadomości z mailbox
2. przepuścić payload przez `handle_nxms_payload(...)`
3. ackować po udanym przetworzeniu
4. nie panicować przy decode/ingest/pull error

To ma być realny runtime integration point, nie tylko helper do testów.

#### Write scope
Możesz zmieniać tylko:
- `/home/nxms-server/privAI/privai-node/src/**`
- `/home/nxms-server/privAI/privai-node/tests/**`
- jeśli naprawdę konieczne:
  - `/home/nxms-server/privAI/privai-node/Cargo.toml`

Read-only:
- `/home/nxms-server/nexum-core/crates/nxms-mailbox-client/**`
- `/home/nxms-server/nexum-core/crates/nxms-mailbox/**`

Nie zmieniaj:
- `/home/nxms-server/privAI/privai-wallet/**`
- `/home/nxms-server/privAI/privai-ledger/**`
- `/home/nxms-server/privAI/privai-chain/**`
- `/home/nxms-server/privAI/privai-nxms/**`
- `/home/nxms-server/nexum-core/**`

#### Source of truth docs
Przeczytaj:
- `/home/nxms-server/privAI/spec/PRIVAI_TRANSPORT_RUNTIME_FREEZE_MEMO.md`
- `/home/nxms-server/privAI/spec/PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`
- `/home/nxms-server/privAI/spec/PRIVAI_EXECUTION_SPINE.md`

#### Code do przeczytania
- `/home/nxms-server/privAI/privai-node/src/node.rs`
- `/home/nxms-server/privAI/privai-node/src/config.rs`
- `/home/nxms-server/privAI/privai-node/src/main.rs`
- `/home/nxms-server/privAI/privai-node/src/lib.rs`

Read-only:
- `/home/nxms-server/nexum-core/crates/nxms-mailbox-client/src/lib.rs`
- `/home/nxms-server/nexum-core/crates/nxms-mailbox/src/api.rs`

#### Required changes
##### 1. Add mailbox runtime config
Dodaj minimalny config potrzebny do mailbox loop, np.:
- enable flag
- base URL
- auth token / mailbox id jeśli potrzebne
- poll interval

Nie projektuj wielkiego config systemu.

##### 2. Add runtime mailbox loop
Dodaj realny loop/runtime entry point, który:
- pull
- decode
- `handle_nxms_payload(...)`
- ack on success

##### 3. Ack policy
Masz jasno ustalić v1 policy:
- co ackujemy
- czego nie ackujemy
- co się dzieje przy malformed payload
- co się dzieje przy ingest error

##### 4. Controlled errors
Brak `unwrap`.
Brak paniców.
Błędy mają być kontrolowane i sensownie logowane / typowane.

##### 5. Tests
Dodaj minimum testy:

1. pulled escrow payload reaches `handle_nxms_payload`
2. successful ingest triggers ack
3. malformed payload follows chosen ack policy without panic
4. ingest failure follows chosen ack policy without panic
5. non-escrow payload is handled safely

Jeśli potrzeba, użyj cienkiego adaptera/fake mailbox client w testach.
Ale produkcyjny loop ma pozostać realny.

#### Forbidden shortcuts
Nie wolno:
1. zrobić tego jako helper tylko do testów
2. mieszać validator session transport z mailbox path
3. ackować wszystko w ciemno
4. dotykać `nexum-core`
5. dotykać walleta / proof path

#### Definition of Done
Task jest skończony tylko jeśli:
1. node ma realny mailbox pull loop / integration point
2. loop używa `handle_nxms_payload(...)`
3. successful ingest prowadzi do ack
4. błędy są kontrolowane
5. są testy runtime/helper layer
6. `cargo test -p privai-node -q` przechodzi

#### Minimalne komendy
Uruchom minimum:
- `cd /home/nxms-server/privAI && cargo test -p privai-node -q`

#### Wynik
Na końcu podaj dokładnie:
1. zmienione pliki
2. gdzie jest mailbox runtime loop / integration point
3. jaki config dodałeś
4. jaka jest ack policy
5. jakie testy dodałeś
6. jakie komendy uruchomiłeś
7. czego ten loop dalej jeszcze NIE robi

## Zadanie 13 — Xiaomi B
### validator session regression pack

Pracujesz tylko w WSL.

Repo:
`/home/nxms-server/privAI`

Nie pracuj na ścieżkach Windows.
Nie używaj `C:\...`.

To jest NOWY TASK.
Nie kontynuacja starego tasku.
Nie dotykasz `nexum-core`.
Nie dotykasz mailbox runtime ani escrow e2e tasków wykonywanych równolegle.

#### Twoja rola
Dodać regression test pack dla validator session transport.

To jest task hardening/testowy dla Model A:
- direct validator P2P
- handshake
- encrypted frames
- session transport
- gossip boundary

Nie zmieniasz architektury.
Nie robisz redesignu.
Masz utrwalić najważniejsze invariants.

#### Główny cel
Dodać mocny zestaw testów regresyjnych, który pilnuje, że validator session path:
- nie miesza się z mailbox path
- zachowuje swoje invariants
- nie psuje się po kolejnych refaktorach

#### Write scope
Możesz zmieniać tylko:
- `/home/nxms-server/privAI/privai-node/src/session_transport.rs`
- `/home/nxms-server/privAI/privai-node/src/session_impl.rs`
- `/home/nxms-server/privAI/privai-node/tests/**`
- jeśli naprawdę konieczne:
  - `/home/nxms-server/privAI/privai-node/Cargo.toml`

Nie zmieniaj:
- `/home/nxms-server/privAI/privai-node/src/node.rs`
- `/home/nxms-server/privAI/privai-node/src/config.rs`
- `/home/nxms-server/privAI/privai-wallet/**`
- `/home/nxms-server/privAI/privai-ledger/**`
- `/home/nxms-server/privAI/privai-chain/**`
- `/home/nxms-server/nexum-core/**`

#### Source of truth docs
Przeczytaj:
- `/home/nxms-server/privAI/spec/PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`
- `/home/nxms-server/privAI/spec/PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`
- `/home/nxms-server/privAI/spec/PRIVAI_TRANSPORT_RUNTIME_FREEZE_MEMO.md`

#### Code do przeczytania
- `/home/nxms-server/privAI/privai-node/src/session_transport.rs`
- `/home/nxms-server/privAI/privai-node/src/session_impl.rs`
- `/home/nxms-server/privAI/privai-node/src/gossip.rs`
- `/home/nxms-server/privAI/privai-node/tests/validator_session.rs`

#### Required changes
##### 1. Add regression coverage for key invariants
Dodaj testy minimum dla:
- handshake transcript mismatch rejected
- encrypted frame tamper rejected
- seq / nonce misuse rejected
- peer identity mismatch rejected
- stale session rebuild or reconnect behavior
- validator path does not require mailbox components

##### 2. Keep transport split explicit
Jeśli naturalnie pasuje, dodaj mały komentarz/doc comment w testach albo module helperów, który przypomina:
- validator session transport != mailbox/NXMS path

Nie pisz eseju.
Ma być krótko.

##### 3. Add at least one gossip-adjacent regression
Nie musisz robić pełnego gossip integration.
Ale dodaj minimum jeden test, który pilnuje, że validator session-facing message flow nadal spełnia założenia używane przez gossip.

##### 4. Do not redesign runtime
Masz dodać regression pack.
Nie przepisywać transportu.
Nie robić nowego protokołu.

#### Forbidden shortcuts
Nie wolno:
1. mieszać validator tests z mailbox loop
2. robić same komentarze bez realnych testów
3. przepisywać session implementation bez mocnej potrzeby
4. dotykać node submit / wallet / escrow paths

#### Definition of Done
Task jest skończony tylko jeśli:
1. jest realny regression pack dla validator session path
2. testy pokrywają minimum handshake/frame/identity invariants
3. jest co najmniej jeden gossip-adjacent regression
4. `cargo test -p privai-node -q` przechodzi

#### Minimalne komendy
Uruchom minimum:
- `cd /home/nxms-server/privAI && cargo test -p privai-node -q`

Jeśli dodasz osobny test file, uruchom go też jawnie.

#### Wynik
Na końcu podaj dokładnie:
1. zmienione pliki
2. jakie invariants pokrywa regression pack
3. jaki gossip-adjacent test dodałeś
4. jakie komendy uruchomiłeś
5. czego ten pack nadal NIE pokrywa

## Zadanie 14 — Gemini
### full honest local escrow e2e

Pracujesz tylko w WSL.

Repo:
`/home/nxms-server/privAI`

Nie pracuj na ścieżkach Windows.
Nie używaj `C:\...`.

To jest NOWY TASK.
Nie kontynuacja starego tasku.
Nie dotykasz `nexum-core`.
Nie dotykasz mailbox runtime ani validator regression tasków wykonywanych równolegle.

#### Twoja rola
Zbudować pierwszy uczciwy local end-to-end flow dla escrow v1.

Mamy już osobno:
- Stage A staging + approvals
- node bundle export
- wallet final assembly
- typed proof handoff
- escrow-aware submit gate

Teraz trzeba pokazać, że to naprawdę składa się w jeden spójny lokalny flow.

#### Główny cel
Dodać test/integration path, który przechodzi przez:

1. funded escrow staged in node
2. spend proposal staged in node
3. approvals staged in node
4. node builds Stage A bundle
5. wallet builds final escrow tx
6. wallet builds proof-ready handoff
7. proof result is attached
8. node accepts escrow tx submit
9. block / proof artifact path reaches import-ready shape
10. test kończy się uczciwym success condition, nie połowicznym

To NIE ma być fake success.
To NIE ma być "przeszło gate, dalej nie wiadomo".
To ma być pierwszy uczciwy lokalny e2e.

#### Write scope
Możesz zmieniać tylko:
- `/home/nxms-server/privAI/privai-node/tests/**`
- `/home/nxms-server/privAI/privai-wallet/tests/**`
- `/home/nxms-server/privAI/privai-wallet/src/**`
- `/home/nxms-server/privAI/privai-proof/src/**`

Jeśli naprawdę konieczne:
- `/home/nxms-server/privAI/privai-node/Cargo.toml`
- `/home/nxms-server/privAI/privai-wallet/Cargo.toml`

Nie zmieniaj:
- `/home/nxms-server/privAI/privai-node/src/**`
- `/home/nxms-server/privAI/privai-ledger/src/**`
- `/home/nxms-server/privAI/privai-chain/src/**`
- `/home/nxms-server/privAI/privai-nxms/**`
- `/home/nxms-server/nexum-core/**`

#### Source of truth docs
Przeczytaj:
- `/home/nxms-server/privAI/spec/PRIVAI_ESCROW_OBJECT_MODEL.md`
- `/home/nxms-server/privAI/spec/PRIVAI_ESCROW_TX_MATRIX.md`
- `/home/nxms-server/privAI/spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md`
- `/home/nxms-server/privAI/spec/PRIVAI_ESCROW_FULLPRIVACY_BOUNDARY_DECISION_MEMO.md`

#### Code do przeczytania
- `/home/nxms-server/privAI/privai-node/tests/escrow_submit_gate.rs`
- `/home/nxms-server/privAI/privai-node/tests/end_to_end_flow.rs`
- `/home/nxms-server/privAI/privai-wallet/src/escrow_builder.rs`
- `/home/nxms-server/privAI/privai-wallet/src/proof_handoff.rs`
- `/home/nxms-server/privAI/privai-node/src/node.rs` (read-only)
- `/home/nxms-server/privAI/privai-proof/src/artifact.rs`
- `/home/nxms-server/privAI/privai-proof/src/transfer.rs`

#### Required changes
##### 1. Add one honest local escrow e2e test
Dodaj co najmniej jeden pełny test e2e dla:
- `release`
albo
- `refund`

Preferowane:
- najpierw `release`

##### 2. Real staged flow
Test ma przejść przez prawdziwe etapy:
- `EscrowFunded`
- `EscrowSpendProposal`
- approvals
- `build_escrow_approval_bundle(...)`

##### 3. Real wallet path
Test ma użyć:
- wallet-side final assembly
- `EscrowProofReadyHandoff::build(...)`
- attach proof result step

Nie sklejaj ręcznie obiektów, jeśli istnieje właściwe API.

##### 4. Honest success condition
Na końcu test ma sprawdzać coś realnego, np.:
- tx accepted by submit path
- proof artifact shape is valid
- import-ready block proof artifacts match block
- albo równoważny uczciwy success condition

Nie wystarczy:
- “nie było panic”
- “gate passed”
- “artifact się zserializował”

##### 5. Keep proof orchestration honest
Jeśli pełny real prover runner nadal nie istnieje:
- nie udawaj, że istnieje
- użyj already-supported attach step z explicit proof bytes
- ale test ma być uczciwy w tym, co naprawdę weryfikuje

##### 6. Minimal scope discipline
Nie rób nowego frameworka e2e.
Nie przepisuj całych istniejących testów.
Dodaj jeden mocny, czytelny flow i ewentualnie mały helper jeśli naprawdę potrzebny.

#### Forbidden shortcuts
Nie wolno:
1. dotykać `privai-node/src/**`
2. uznawać częściowego przejścia za sukces
3. mockować wszystkiego tak mocno, że test przestaje coś znaczyć
4. ruszać `nexum-core`
5. robić redesignu proof systemu

#### Definition of Done
Task jest skończony tylko jeśli:
1. istnieje co najmniej jeden uczciwy local escrow e2e test
2. test przechodzi przez node Stage A + wallet Stage B + proof handoff
3. test kończy się realnym success condition
4. `cargo test -p privai-node -q` przechodzi
5. `cargo test -p privai-wallet -q` przechodzi
6. `cargo test -p privai-proof -q` przechodzi

#### Minimalne komendy
Uruchom minimum:
- `cd /home/nxms-server/privAI && cargo test -p privai-node -q`
- `cd /home/nxms-server/privAI && cargo test -p privai-wallet -q`
- `cd /home/nxms-server/privAI && cargo test -p privai-proof -q`

#### Wynik
Na końcu podaj dokładnie:
1. zmienione pliki
2. jaki exact flow pokrywa test
3. jaki jest finalny success condition
4. jakie komendy uruchomiłeś
5. czego ten e2e nadal jeszcze NIE dowodzi
