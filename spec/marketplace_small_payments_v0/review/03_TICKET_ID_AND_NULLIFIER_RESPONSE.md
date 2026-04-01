# 03. Ticket ID And Nullifier - Gemini Response

## 1. Recommended v0 design
Domyślny design opiera się na `merchant-scoped ticketach`. Łańcuch używa `ticket_nullifier` jako głównego markera zapobiegającego powtórzeniom (replay protection). Model autoryzacji oparty jest na `marketplace-operator-trusted accounting` – łańcuch konsensusu sprawdza unikalność `ticket_nullifier` oraz poprawność publikacji batcha przez operatora, podczas gdy powiązanie (deposit binding) z początkowym depozytem jest egzekwowane off-chain przez zaufaną rolę operatora podczas wystawiania `SpendGrant`.

## 2. Alternative design
- `Marketplace-wide ticket`: Zwiększa linkowalność, odrzucony jako default.
- `Anchor-bound cryptographic settlement`: Wymaga on-chain dowodów ZK łączących depozyt z `ticket_nullifier`. Odrzucony na v0 ze względu na dużą złożoność i koszty dla drobnicy, jednak pożądany jako ścieżka ewolucji (trust-minimized v1/v2).

## 3. Data model
Z lokalnego `rail_seed` (powiązanego z depozytem) wallet generuje zbiór (`ticket_id_i`, `ticket_nullifier_i`, `ticket_auth_i`) z użyciem `merchant_commit` jako kontekstu derywacji.
- `ticket_id_i`: Zależny tylko lokalnie dla walleta i off-chain merchanta.
- `ticket_nullifier_i`: Skrót kryptograficzny publikowany on-chain.
- `ticket_auth_i`: Materiał podpisu autoryzującego, używany do podpisu `purchase_commit`.

## 4. Public vs private fields
- **Publikowane on-chain (wewnątrz SettlementTx):** `ticket_nullifier`, `merchant_commit`, `amount` (jako część batch agregatu lub jawna lista, zależnie od finalnego układu batcha), zobowiązania `service_commit`, `purchase_commit` (jako liście Merkle pod `receipt_root`).
- **Pozostające off-chain (prywatne):** `ticket_id_i`, oryginalny depozyt/konto, `ticket_auth_i` (tylko weryfikowany off-chain przez operatora), historia operacji walleta.

## 5. Deposit binding model
W modelu v0 (`marketplace-operator-trusted accounting`), `ticket_nullifier` nie ma natywnego ZK dowodu do depozytu on-chain. Wallet udowadnia prawo do depozytu **operatorowi** przy prośbie o sesję. Operator wydaje `SpendGrant` off-chain. Chain widzi tylko `ticket_nullifier` w `SettlementTx` i weryfikuje podpis operatora. Zaufanie spoczywa na operatorze (że nie umieści w settlement fałszywych nullifierów bez pokrycia).

## 6. Replay protection model
- **On-chain:** Utrzymywana globalna baza zużytych `ticket_nullifier`. Jeśli nullifier się powtórzy, sieć odrzuca batch.
- **Merchant/Off-chain:** Merchant i operator cache'ują użyte nullifiery/ticket_id, aby zapobiegać reużyciu przed wejściem na łańcuch.
- `purchase_commit` wiąże wydatek z kontekstem, by zablokować użycie ważnego `ticket_nullifier` na rzecz innej usługi/sesji u tego samego merchanta.

## 7. Scope decision
Wybrany zasięg to **Merchant-scoped**. Pule ticketów są generowane ze wskazaniem na konkretny `merchant_commit`, co zapobiega linkowaniu pomiędzy różnymi sprzedawcami na tym samym marketplace i ogranicza promień ataku (blast radius).

## 8. Ticket vs tab separation
- `Ticket` to jednorazowe, zanonimizowane kryptograficznie prawo na pojedyncze odliczenie z konta depozytu. Po wysłaniu i skonsumowaniu w rachunku (receipt) ticket jest spalony.
- `Tab / Session` to długotrwały stan u merchanta (np. otwarta sesja AI). Wiele `ticketów` może autoryzować wpłaty pokrywające rosnący `tab` lub jeden duży z góry sfinansowany ticket z `SpendGrant` obsługuje całą sesję odliczania off-chain.

## 9. Recovery and lifecycle
Wallet potrafi w pełni odtworzyć pule na podstawie `rail_seed` (który jest mnemotechniczną częścią głównego wallet seeda). Niewykorzystane po wygaśnięciu (`expiry`) tickety są logicznie "spalane" off-chain i zapominane. Przy wielu urządzeniach, generowany jest mniejszy sub-seed per-urządzenie, aby unikać konfliktów liczników derywacji na tym samym urządzeniu.

## 10. Threat analysis
- **Zgubiony ticket:** Zablokowanie limitu sesji u operatora (wymaga timeout_refund).
- **Złośliwy merchant:** Może próbować sfabrykować `purchase_commit`, ale brakuje mu `ticket_auth_i` do prawidłowego podpisania zapytania do operatora.
- **Złośliwy operator:** W modelu v0 może zatrzymać batch lub sfabrykować settlement, bo brakuje bezpośredniego bindingu on-chain (jest to akceptowalny w v0 tradeoff za cenę braku drogich ZK-SNARKów).

## 11. Frozen decisions
- On-chain marker to `ticket_nullifier`.
- Ticket posiada merchant-scope.
- Recovery jest deterministyczne z `rail_seed`.
- `purchase_commit` pozostaje niezbędny.
- Decyzja o powiązaniu z depozytem spoczywa u operatora (trusted accounting).

## 12. Open questions
- Zostaje uregulowane implementacyjnie, iż on-chainowy limit czasowy decyduje o przerwaniu rezerwacji z timeout (rozwiązane w `SettlementBatchSummary`).