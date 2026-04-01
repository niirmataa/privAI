# 04. Receipt And Settlement Root - Gemini Response

## 1. Recommended v0 design
Projekt opiera się na dwuwarstwowej agregacji. Merchant i wallet wspólnie zgadzają się na `Receipt` (dokumentując wykonanie i autoryzację). Operator zbiera Receipts i tworzy `receipt_root` (klasyczne drzewo Merkle) oraz `settlement_root` (podsumowanie batcha). Ze względu na uproszczoną weryfikację na łańcuchu (v0), wprowadzono jawną, płaską listę `ticket_nullifier` wewnątrz on-chain `SettlementTx`, aby konsensus mógł wykluczyć double-spendy bez obciążeń dowodami kryptograficznymi na membership.

## 2. Alternative design
Podejście "Root-only / compressed publication" – umieszczanie on-chain jedynie `nullifier_root` zamiast płaskiej listy – zostało odrzucone. Minimalizuje to payload, ale wymaga on-chain SNARK/STARK weryfikującego, że żaden nullifier w drzewie nie powtarza się w historii, co drastycznie podnosi próg skomplikowania na etapie v0.

## 3. Receipt data model
`Receipt` to kanoniczny JSON/bajty podpisany głównie przez merchanta, i opcjonalnie akceptowany przez payload walleta:
- `receipt_id`: do deduplikacji
- `merchant_commit`, `service_commit`, `session_commit`, `grant_commit`, `purchase_commit`
- `ticket_nullifier`
- `amount` (i ew. pricing unit)
- `policy_commit`
- `result_commit`
- `issued_at`
- `merchant_sig` (obowiązkowy dowód dla operatora).

## 4. Batch and root model
- `receipt_root`: Merkle Root wszystkich `ReceiptCommit` (skrót kanonicznej formy Receiptu) w danym batchu. Pusta pamięć off-chain pozwala na audyt.
- `settlement_root`: Skrót obejmujący: `merchant_commit`, `grant_commit`, okno czasowe, `receipt_root`, liczbę receiptów i nullifierów oraz zsumowane `amount`, `fee`, `refund`.

## 5. SettlementTx data model
On-chain trafia `SettlementTx` zawierający:
- `version`
- `operator_commit`, `merchant_commit`, `grant_commit`
- `settlement_window_start`, `settlement_window_end`
- `receipt_root`, `settlement_root`
- `receipt_count`, `nullifier_count`
- `total_gross_amount`, `total_fee_amount`, `total_refund_amount`
- Jawna, spłaszczona lista `[ticket_nullifier_1, ..., ticket_nullifier_N]`
- `operator_sig`.

## 6. Consensus verification model
**Konsensus PC-BFT twardo weryfikuje:**
- Zgodność matematyczną nagłówków z wartościami batcha.
- Podpis zaufanego `operator_commit` mającego prawa.
- **Krytyczne:** Brak jakiegokolwiek z jawnie wymienionych `ticket_nullifier` w bazie globalnej nullifierów (ochrona przed double-spend).
**Czego NIE weryfikuje w v0:**
- Zgodności `amount` i powiązań `purchase_commit` konkretnego receiptu z początkowym depozytem (to zaufanie operatora off-chain).
- Dowodu poprawnego działania usługi.

## 7. Refund and failure model
Refund ścieżka opiera się na receiptach ujętych w batchu (jako offsety `total_refund_amount`). Jeśli dojdzie do timeoutu (merchant nie dostarczył usługi / brakuje wystarczających Receipts po upływie `settlement_window`), operator automatycznie obniża zsumowany `total_gross_amount` w `SettlementTx`, oddając uwięzione na rezerwie fundusze użytkownikowi, bez dodatkowych "RefundTx" on-chain.

## 8. Retention and audit model
Audyt opiera się na dostarczaniu danych przez operatora (off-chain storage). W przypadku sporu (dispute) operator musi móc wyeksportować i udowodnić konkretne `Receipt`y pasujące do zatwierdzonego `receipt_root`. Wymagany czas retencji dla Receipts po stronie operatora to minimum 30 do 90 dni (zależnie od ServicePaymentPolicy).

## 9. Frozen decisions
- Kanoniczne Merkle tree dla `receipt_root`.
- Płaska (jawna) lista `ticket_nullifier` ląduje w on-chain `SettlementTx` (a dokładniej MarketplaceBatchTx).
- Tylko autoryzowany operator publikuje settlement.
- Istnieją `receipt_root` i `settlement_root` z wyraźnym rozdziałem logiki.

## 10. Open questions
Brak. Implementacja została zweryfikowana z wielkością płaskiej listy na łańcuchu.