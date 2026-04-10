# Xiaomi Handoff i Szablon Tasków

Data: 2026-04-09
Cel: jeden plik, który zapisuje:
- co już jest naprawdę zrobione,
- jaki jest aktualny task dla Xiaomi,
- jaki jest nasz najlepszy kompaktowy szablon tasków dla agentów.

---

## 1. Co jest już zrobione

To są rzeczy, które traktujemy jako domknięte albo wystarczająco zweryfikowane:

### Stage A / control-plane / docs
- `nexum-core` orchestrator został oczyszczony z fałszywego modelu, w którym Stage A zna finalny canonical `tx_signing_hash`.
- Docs zostały zsynchronizowane:
  - `spec/PRIVAI_ESCROW_OBJECT_MODEL.md`
  - `spec/PRIVAI_ESCROW_TX_MATRIX.md`
  - `spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md`
- `EscrowApprovalBundle` w docs jest już opisany jako Stage A authorization material, a nie finalny ledger-ready auth.

### privai-nxms
- są roundtrip / payload / msg_type / body_hash tests dla escrow bodies,
- `EscrowSpendProposal` nie niesie już `tx_signing_hash`,
- Stage A wire model jest zsynchronizowany z nowym boundary.

### privai-node
- działa staging:
  - funded
  - proposal
  - approvals
  - quorum
- działa payload ingress:
  - `handle_nxms_payload(...)`
- działa persistence Stage A store,
- działa `build_escrow_approval_bundle(...)`,
- approval-layer jest utwardzony:
  - duplicate rejection
  - deterministic ordering
  - count checks
- escrow submit gate jest domknięty:
  - proposal exists
  - quorum exists
  - `policy_opening` exists
  - `policy_opening` decodes
  - policy type is `Escrow2of3`
  - policy fields match staged descriptor
  - `escrow_action` matches proposal
  - signer set matches Stage A bundle
  - signatures verify against `tx_signing_hash`

### privai-wallet / privai-proof
- wallet final assembly bridge jest zrobiony,
- narrowing bugs w amount path zostały poprawione,
- typed Stage B proof handoff jest zrobiony:
  - `EscrowProofReadyHandoff`
  - `EscrowAttachedProof`

### E2E / tests
- validator session regression pack jest zrobiony i przechodzi,
- honest local escrow e2e release flow jest zrobiony i przechodzi,
- `cargo test -p privai-wallet -q` przechodzi,
- `cargo test -p privai-proof -q` przechodzi,
- `cargo test -p privai-node --test validator_session -q` przechodzi,
- `cargo test -p privai-node --test escrow_e2e_release -q` przechodzi,
- `cargo test -p privai-node --test escrow_submit_gate -q` przechodzi.

### Transport/runtime freeze
- powstało memo:
  - `spec/PRIVAI_TRANSPORT_RUNTIME_FREEZE_MEMO.md`
- zamrożenie v1:
  - validators = direct P2P session transport
  - NXMS control-plane = mailbox store-and-forward over Tor

---

## 2. Co NIE jest jeszcze uczciwie domknięte

Najważniejsza otwarta rzecz:

### Mailbox runtime loop w `privai-node`
W worktree są zmiany sugerujące, że task został rozpoczęty:
- `privai-node/src/config.rs`
- `privai-node/src/lib.rs`
- `privai-node/src/mailbox_pull.rs`
- `privai-node/Cargo.toml`
- `Cargo.lock`

Kod wygląda sensownie architektonicznie:
- jest `MailboxPullConfig`,
- jest `MailboxSource`,
- jest `mailbox_ingest_tick(...)`,
- jest `run_mailbox_pull_loop(...)`,
- jest opisana ack policy,
- są eksporty w `privai-node/src/lib.rs`.

Ale ten task NIE jest jeszcze uczciwie zamknięty, bo:
- środowisko WSL bywa niestabilne,
- jako `root` często odpala się złe `cargo 1.83.0` zamiast toolchainu z `rust-toolchain.toml`,
- przez to pełne `cargo test -p privai-node -q` nie jest jeszcze wiarygodnym sygnałem dla tego tasku,
- nie mamy jeszcze twardego, spokojnego review tego konkretnego pathu po testach mailboxowych.

Czyli status:
- `validator regression`: zamknięte
- `escrow e2e`: zamknięte
- `mailbox runtime loop`: **pending verification / pending finish**

---

## 3. Aktualny szczegółowy task dla Xiaomi

### Owner
Xiaomi

### Task
Domknąć i zweryfikować `privai-node` mailbox runtime loop.

### Problem
W repo jest już rozpoczęta implementacja runtime mailbox path, ale przez problemy WSL/toolchain nie mamy jeszcze uczciwego zamknięcia tasku.

### Misja
Sprawić, żeby mailbox runtime loop był:
- realnie podpięty do `privai-node`,
- testowalny,
- zweryfikowany,
- i gotowy do commita bez zgadywania, czy środowisko akurat nie kłamie.

### Write scope
Możesz zmieniać tylko:
- `/home/nxms-server/privAI/privai-node/src/**`
- `/home/nxms-server/privAI/privai-node/tests/**`
- jeśli konieczne:
  - `/home/nxms-server/privAI/privai-node/Cargo.toml`
  - `/home/nxms-server/privAI/Cargo.lock`

Read-only:
- `/home/nxms-server/nexum-core/crates/nxms-mailbox-client/**`
- `/home/nxms-server/nexum-core/crates/nxms-mailbox/**`

Nie zmieniaj:
- `/home/nxms-server/privAI/privai-wallet/**`
- `/home/nxms-server/privAI/privai-proof/**`
- `/home/nxms-server/privAI/privai-ledger/**`
- `/home/nxms-server/privAI/privai-chain/**`
- `/home/nxms-server/privAI/privai-nxms/**`
- `/home/nxms-server/nexum-core/**`

### Source of truth
Przeczytaj:
- `/home/nxms-server/privAI/spec/PRIVAI_TRANSPORT_RUNTIME_FREEZE_MEMO.md`
- `/home/nxms-server/privAI/spec/PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`
- `/home/nxms-server/privAI/spec/PRIVAI_EXECUTION_SPINE.md`

### Kod do przeczytania
- `/home/nxms-server/privAI/privai-node/src/config.rs`
- `/home/nxms-server/privAI/privai-node/src/lib.rs`
- `/home/nxms-server/privAI/privai-node/src/node.rs`
- `/home/nxms-server/privAI/privai-node/src/mailbox_pull.rs`
- `/home/nxms-server/privAI/privai-node/src/main.rs`

Read-only:
- `/home/nxms-server/nexum-core/crates/nxms-mailbox-client/src/lib.rs`
- `/home/nxms-server/nexum-core/crates/nxms-mailbox/src/api.rs`

### Required changes
1. Zweryfikuj, że mailbox runtime loop jest naprawdę podpięty do crate’a i dostępny z sensownego API.
2. Zweryfikuj, że ack policy jest jednoznaczna i spójna z kodem:
   - success -> ack
   - protocol/decode error -> no ack
   - ingest error -> no ack
   - ack failure -> log / retry semantics
3. Jeśli brakuje testów dla:
   - successful ingest -> ack
   - malformed payload -> chosen no-ack policy
   - ingest failure -> chosen no-ack policy
   - `Ignored` path for non-escrow
   to je dodaj.
4. Jeśli aktualny loop nie jest jeszcze podpięty do runtime integration point w sensownym miejscu, dopnij go minimalnie, bez redesignu.
5. Nie mieszaj tego z validator session transport.

### Forbidden
Nie wolno:
1. mieszać mailbox path z validator P2P
2. robić test-only helpera bez realnego production integration point
3. ackować wszystkiego w ciemno
4. dotykać `nexum-core`
5. dotykać wallet/proof

### Definition of Done
Task jest skończony tylko jeśli:
1. mailbox runtime loop jest częścią crate’a i ma realny integration point,
2. używa `handle_nxms_payload(...)`,
3. ack policy jest czytelna i egzekwowana,
4. są testy runtime/helper layer,
5. da się uczciwie uruchomić i zweryfikować targeted tests,
6. `cargo test -p privai-node -q` przechodzi w poprawnym toolchainie.

### Uwaga środowiskowa
Przy problemach WSL uruchamiaj z poprawnym toolchainem użytkownika, a nie z systemowego `cargo 1.83.0`.
Jeśli trzeba odpalać jako `root`, użyj:

```sh
HOME=/home/nxms-privAI PATH=/home/nxms-privAI/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin cargo ...
```

### Wynik końcowy
Na końcu podaj dokładnie:
1. zmienione pliki
2. gdzie jest integration point mailbox loop
3. jaka jest ack policy
4. jakie testy dodałeś lub poprawiłeś
5. jakie komendy uruchomiłeś
6. czy task jest naprawdę gotowy do commita

---

## 4. Nasz kompaktowy szablon tasków dla agentów

Poniżej jest szablon, który ma być:
- bardzo zwarty,
- bardzo konkretny,
- dawać maksimum kontekstu i jasnych granic,
- ale nie odbierać modelowi myślenia.

### Szablon

```text
Pracujesz tylko w WSL.

Repo:
`/home/nxms-server/privAI`

Dodatkowe repo read-only, jeśli potrzebne:
`/home/nxms-server/nexum-core`

Nie pracuj na ścieżkach Windows.
Nie używaj `C:\...`.

To jest NOWY TASK.
Nie kontynuacja starego tasku.
Nie dotykasz równoległych tasków poza swoim write scope.

## Rola
Jedno zdanie:
- co masz realnie dowieźć
- na jakiej warstwie systemu
- czego to NIE jest

## Problem
2-6 punktów:
- jaki konkretny brak / drift / bug naprawiasz
- jaki jest obecny stan
- co już istnieje i na czym wolno się oprzeć

## Główny cel
1-5 konkretnych kroków business/architecture outcome, nie implementacyjny esej.

## Write scope
Możesz zmieniać tylko:
- ...
- ...

Read-only:
- ...

Nie zmieniaj:
- ...
- ...

## Source of truth
Przeczytaj:
- 2-5 najważniejszych docs

## Code do przeczytania
- 3-8 konkretnych plików

## Required changes
1. ...
2. ...
3. ...
4. ...

Każdy punkt:
- ma mówić, co musi powstać,
- ale nie rozpisywać implementacji za model.

## Forbidden
Nie wolno:
1. ...
2. ...
3. ...

## Definition of Done
Task jest skończony tylko jeśli:
1. ...
2. ...
3. ...
4. testy/komendy przechodzą

## Minimalne komendy
- `cd /home/nxms-server/privAI && ...`
- `cd /home/nxms-server/privAI && ...`

## Wynik
Na końcu podaj dokładnie:
1. zmienione pliki
2. gdzie jest główny integration point / API
3. jakie invariants lub walidacje są egzekwowane
4. jakie testy dodałeś
5. jakie komendy uruchomiłeś
6. czego to dalej jeszcze NIE robi
```

### Zasady jakości tego szablonu

#### Co dawać zawsze
- twardy write scope,
- twarde forbidden,
- twarde DoD,
- konkretne docs i pliki do czytania,
- dokładny format końcowego raportu.

#### Czego nie robić
- nie pisać całej implementacji za model,
- nie robić za długiego eseju architektonicznego, jeśli task ma być wykonawczy,
- nie zostawiać “zrób coś sensownego” bez kryteriów,
- nie wrzucać modelu na cały kodbase bez granic.

#### Jak dawać dużo kontekstu bez odbierania myślenia
Zasada:
- daj modelowi prawdę o systemie,
- daj granice,
- daj success criteria,
- ale nie dyktuj każdego helpera i każdej funkcji, jeśli nie trzeba.

Najlepiej działa układ:
- `Rola`
- `Problem`
- `Write scope`
- `Required changes`
- `Forbidden`
- `Definition of Done`

To daje:
- dużo informacji,
- mało chaosu,
- i zostawia miejsce na eleganckie rozwiązanie.

---

## 5. Krótkie podsumowanie operacyjne

Na dziś:
- `validator regression pack` = gotowe
- `escrow e2e release` = gotowe
- `wallet proof handoff` = gotowe
- `submit gate` = gotowe
- `mailbox runtime loop` = jeszcze do spokojnej weryfikacji i dokończenia

Jeśli kolejna sesja ma ruszyć dalej bez zgadywania:
1. przeczytaj ten plik,
2. przeczytaj `KONTEKST_ORCHESTRACJI_2026-04-09.md`,
3. przeczytaj `zadania_12_14_opis.md`,
4. dopnij mailbox task,
5. dopiero potem rób kolejny commit runtime.
