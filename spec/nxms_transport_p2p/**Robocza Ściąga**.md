**Robocza Ściąga**
To nie jest source of truth. To jest nasza prywatna mapa pamięciowa, żeby po formacie nie zgubić toru.

**1. Jak teraz pracujemy**
- Jedna aktywna faza naraz.
- Teraz aktywna faza to `transport / p2p`.
- Aktywne źródło wykonania to:
- `spec/nxms_transport_p2p/TRANSPORT_P2P_MASTER_TASKS.md`
- `spec/nxms_transport_p2p/TRANSPORT_P2P_FIX_TRACKER.md`
- Globalna mapa kolejności to:
- `spec/PRIVAI_EXECUTION_SPINE.md`
- Reszta docsów to memory anchors na później, nie aktywne źródło implementacji.

**2. Główna architektura systemu**
- `nxms-transport` to shared primitives, Tor, framing, crypto helpers, peer metadata.
- `privai-node` to realna validator session transport, consensus runtime, gossip, sync.
- `privai-ledger` to state transition i durable state semantics.
- `privai-chain` to canonical objects, tx types, block-related semantics.
- `privai-proof` to statement/public inputs/witness/batch/certificate/proof verification.
- `nexum-core` ma być warstwą auth / vault / threshold coordination, nie ledgerem i nie consensusem.
- Marketplace/orchestrator nie może być source of truth dla wydania środków.

**3. Transport jak działa teraz w skrócie**
- `session_transport.rs` to fasada dla reszty noda.
- `session_impl.rs` to konkretna implementacja validator session layer.
- `net.rs` ma zostać cienką fasadą.
- Incoming listener robi `Challenge -> Init -> Response`.
- Incoming path sprawdza freshness i transcript binding.
- Incoming path sprawdza `PeerBook`.
- Incoming path sprawdza podpis Falcon.
- Incoming path robi `kem_encaps`.
- Incoming path potem ma czytać zaszyfrowane frame’y i dopiero potem deserializować `ConsensusMsg`.
- Outgoing `ConnectionPool` utrzymuje persistent connections.
- Outgoing path robi handshake v2 i weryfikuje odpowiedź serwera po oczekiwanym peerze.
- Writer side działa przez actor pattern i bounded MPSC queue.

**4. Co chcieliśmy osiągnąć finalnie w transporcie**
- Zero plaintext po handshake.
- Zero startu na placeholder transport keys.
- Challenge-response zamiast statycznego handshaku.
- Honest listener pressure semantics zamiast udawanego per-peer rate limitera.
- Outgoing symmetry z ban policy.
- Tanie odcięcie floodu przed drogą ścieżką.
- Jasne rozdzielenie:
- pressure guard
- cooldown
- ban list
- peer verification
- encrypted session

**5. Co wygląda na zrobione w transporcie**
- Incoming decrypt gap był naprawiany.
- Handshake v2 był wdrażany.
- Server-side write timeouts były dodawane.
- Shared frame defaults były obcinane.
- Placeholder/zero transport key fail-fast był dodawany.
- Outgoing ban-list symmetry była dodawana.
- Plaintext fallback w `send_message()` wygląda na usunięty.
- `RateLimiter` był uczciwie przemianowywany na coś w stylu `ListenerPressureGuard`.

**6. Co nadal trzeba mieć z tyłu głowy przy transporcie**
- Nie wierzyć ślepo trackerowi bez sanity-checku kodu.
- `ban poisoning` musi być naprawdę usunięty na incoming path, nie tylko opisany.
- `HandshakeCooldown` musi być naprawdę wpięty w failure paths, nie tylko istnieć jako struct.
- Frame-level replay/order semantics trzeba potwierdzić w kodzie, bo raport tasku i plik nie muszą się zgadzać.
- Session key z KEM nie powinien finalnie być robiony przez pad/truncate; docelowo lepiej KDF do stałych 32 bajtów.
- Pressure guard/cooldown pod Torem to bardziej local listener protection niż prawdziwe per-peer identity protection.

**7. Auth model jak ma działać finalnie**
- Obecny duży temat to rozdzielenie `tx_id` od `tx_signing_hash`.
- Podpis nie może być sprawdzany względem czegoś, co samo zawiera auth.
- Finalnie każdy auth path ma mieć jawny, canonical signing preimage.
- Signer ordering ma być canonical.
- Duplicate signer material nie może liczyć się dwa razy.
- Identity binding signer -> auth artifact ma być jednoznaczny.
- Threshold auth package ma być sprawdzane przez ledger.
- `nexum-core` ma pomóc z kluczami i składaniem approvals.
- `privAI` ledger ma być finalnym weryfikatorem poprawności auth package.

**8. Chain / ledger / node jak ma działać finalnie**
- Storage contract ma być jawny.
- Recovery/restart rules mają być jawne.
- Source of truth ma być jawne dla:
- `tip/head`
- `finalized head`
- `last persisted but not fully indexed block`
- RocksDB ma być current implementation choice, nie protocol requirement.
- Durable state ma być oddzielone od rebuildable indexes i ephemeral state.
- Node nie powinien wiedzieć za dużo o low-level storage layout.
- Ledger ma być source of truth dla state transition i correctness.

**9. Proof jak rozumiemy teraz**
- Nie wymyślamy proof od nowa.
- Proof już istnieje i trzeba go opisać oraz domknąć.
- `TransferNoteTx` to główny current proof-covered rail.
- `ExecutionBundle` i `ProofCertificate` mają current bytes/runtime role, ale semantyka musi być jasno dokończona.
- `MarketplaceBatchTx` nie jest proof-equivalent do `TransferNoteTx`.
- `OnChainLite` nie może udawać finalności; ma status `experimental` albo wymaga jawnego domknięcia.
- Najważniejsze boundary:
- co proof sprawdza
- co ledger sprawdza poza proof
- jak liczy się `statement_commit`
- jak liczy się `public_inputs_hash`
- jak to wiąże się z `ExecutionBundle` i `ProofCertificate`

**10. Escrow jak ma działać finalnie**
- Escrow ma być note-based, nie operator-balance-based.
- Środki mają siedzieć w escrow note z policy lockiem.
- Intuicyjnie to jest `2-z-3 multisig`, ale policy-constrained.
- Role to:
- `Buyer`
- `Merchant`
- `Operator`
- Normalny tryb ma wymagać operatora jako jednego z podpisujących.
- Recovery path ma dopuszczać `Buyer + Merchant`, ale tylko jako ścieżkę awaryjną.
- To nie ma być “dowolne dwa podpisy robią cokolwiek”.
- Podpisy mają autoryzować konkretną akcję zakodowaną w `tx_signing_hash`.
- Escrow note ma wypuszczać nowe note’y na jednorazowe / stealth-like outputy.
- `nexum-core` ma produkować auth artifacts i koordinować signers.
- Ledger ma sprawdzać threshold, action semantics, nullifiers, outputs i state transition.
- Proof nie musi w v1 przejąć całego escrow auth; może sprawdzać note correctness, a ledger threshold auth.

**11. Najważniejsze akcje escrow**
- `ReleaseToMerchant`
- `RefundToBuyer`
- `RecoveryRelease`
- Każda akcja musi mieć jawny `action_type`.
- Każda akcja musi być związana z:
- escrow id lub escrow commit
- escrow policy commit
- input references
- output commitments
- amount
- fee
- `tx_signing_hash`
- Signerzy nie podpisują “wydania środków ogólnie”.
- Signerzy podpisują konkretny canonical action payload.

**12. Co może być autorskie / mocne w systemie**
- Nie samo `2-z-3`.
- Mocne jest połączenie:
- note-based ledger
- policy-locked escrow note
- one-time outputs
- threshold auth nad action semantics
- operator-mediated normal mode
- bilateral recovery path
- privacy/proof layer
- To może być bardzo oryginalne jako system design.
- Trzeba uważać z claimami “PQ full privacy”, jeśli proof layer nie jest sama w sobie PQ-secure.

**13. Czego nie mieszać**
- Nie mieszać current marketplace v0 z final escrow.
- Nie mieszać auth semantics z proof semantics.
- Nie mieszać transport claims z ledger correctness.
- Nie mieszać “implemented today” z “target final”.
- Nie mieszać memory docs z aktywnym source of truth.

**14. Kolejność dalszych faz**
- Najpierw kończymy transport naprawdę uczciwie.
- Potem `auth / signing model`.
- Potem `chain / ledger / node baseline`.
- Potem `proof completion`.
- Potem `escrow final model`.
- Na końcu `marketplace / operator trust model`.

**15. Po czym poznać, że możemy przejść dalej**
- Transport:
- handshake działa
- brak plaintext po handshake
- brak startu na placeholder keys
- brak prostego ban poisoning
- tracker nie overclaimuje
- Auth:
- `tx_signing_hash` istnieje
- signer rules są canonical
- threshold auth ma jawne verification rules
- Proof:
- `TransferNoteTx` ma zamkniętą semantykę
- `ExecutionBundle` i `ProofCertificate` są opisane uczciwie
- Escrow:
- role, trust model, action model i proof/ledger split są jawne

**16. Minimalna reguła, żeby po formacie się nie pogubić**
- Aktywna faza: tylko jedna.
- Aktywne docs: tylko te od tej fazy.
- Reszta docs: pamięć kierunku.
- Po każdym tasku pytamy:
- czy fix jest realny
- czy tracker mówi prawdę
- czy temat jest naprawdę zamknięty

**17. Najkrótszy obraz całości**
- `nxms-transport` daje klocki.
- `privai-node` robi bezpieczny validator transport i runtime.
- `privai-ledger` egzekwuje correctness i state transition.
- `privai-proof` daje privacy/correctness proofs tam, gdzie już jest domknięte.
- `nexum-core` robi auth/threshold coordination.
- Escrow finalnie to note-based, policy-constrained, multisig escrow z operatorem w normal mode i recovery path dla stron.

Jak będziesz chciał po formacie, możemy od tego dokładnie ruszyć i z tej ściągi odbudować aktywny plan bez wracania do całego chaosu naraz.