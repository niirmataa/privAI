# Marketplace Small Payments v0

Status: migration source for final marketplace receipt and settlement semantics.
Canonicality: non-canonical. Ten dokument jest wazny tylko jako material do zlozenia finalnej spec.
Owner: privAI marketplace payments.
Depends on: `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`.
Superseded by: planowany `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`.

## Receipt And Settlement Root

## 1. Cel

Ten dokument ma zamknac kolejny krytyczny filar lekkiego raila marketplace-only:

- czym dokladnie jest `receipt`,
- jak merchant udowadnia operatorowi wykonanie uslugi,
- jak operator agreguje receipty do batcha settlementowego,
- co trafia on-chain,
- co sprawdza konsensus,
- gdzie konczy sie audytowalnosc, a gdzie zaczyna trusted accounting assumption v0.

Ten dokument nie ma byc luznym brainstormem.
Ma wymusic odpowiedz, z ktorej da sie robic spec i implementacje.

## 2. Co jest juz zamrozone i nie podlega renegocjacji w tym dokumencie

1. Rail jest `marketplace-only`.
2. Rail zaczyna sie od prywatnego depozytu.
3. Drobne usage eventy nie robia pelnej noty `FullPrivacy` per event.
4. `RecipientPrivacyLite` jest dopuszczalna dla lekkiego raila.
5. `ticket` jest `strictly one-time`.
6. `ticket` i `tab/session` to dwa rozne byty.
7. `purchase_commit` jest wymagany.
8. Domyslny model v0 to `marketplace-operator-trusted accounting`.
9. Marketplace operator jest domyslna settlement authority v0.
10. Merchant nie jest samodzielna authority od praw z depozytu.

## 3. Czego ten dokument ma nie robic

Ten dokument nie ma:

- przedefiniowac modelu ticketow,
- usuwac `purchase_commit`,
- wracac do modelu, w ktorym merchant sam interpretuje depozyt,
- projektowac `FullPrivacy` batch proof dla receipts,
- projektowac calego UX walleta.

Ten dokument ma zamknac tylko:

- model receiptu,
- model batcha settlementowego,
- role `receipt_root` i `settlement_root`,
- model `SettlementTx`,
- zakres weryfikacji konsensusu.

## 4. Twardy problem do rozwiazania

Musimy umiec odpowiedziec, jak przejsc od:

- sesji usage-metered,
- jednorazowych ticketow,
- merchant-side receipts,

do:

- jednego lub kilku settlement batchy,
- publicznie audytowalnego wyniku,
- globalnej ochrony przed replay przez `ticket_nullifier`,
- przy minimalnej ilosci danych wrzucanych na chain.

Po tej iteracji ma byc jasne:

- co merchant przekazuje operatorowi,
- co operator komituje w batchu,
- co chain widzi,
- czego chain nie widzi,
- co jest tylko trusted accounting assumption.

## 5. Definicje robocze

### 5.1. Receipt

`Receipt` to minimalny obiekt settlementowy potwierdzajacy wykonanie uslugi lub naliczenie naleznosci w ramach lekkiego raila.

To nie jest debug log.
To nie jest tylko telemetry event.
To jest byt, z ktorego ma dawac sie:

- zbudowac batch,
- policzyc naleznosc,
- zrobic refund path,
- zrobic dispute review,
- zrobic audyt settlementu.

### 5.2. ReceiptId

`ReceiptId` to identyfikator konkretnego receiptu.

Rola:

- deduplikacja,
- lokalne i operatorskie przechowywanie,
- powiazanie z sesja i purchase context.

`ReceiptId` nie jest glownym on-chain replay guard.

### 5.3. ReceiptCommit

`ReceiptCommit` to hash/commitment do kanonicznego receiptu.

Rola:

- leaf do `receipt_root`,
- latwy audit anchor,
- stabilny obiekt podpisywalny i serializowalny.

### 5.4. ReceiptRoot

`ReceiptRoot` to commitment do zbioru receiptow settlementowanych w jednym batchu.

Rola:

- audit anchor,
- dispute/refund anchor,
- identyfikator tresci batcha bez wrzucania wszystkich receipts on-chain.

### 5.5. SettlementBatch

`SettlementBatch` to operatorski obiekt rozliczeniowy obejmujacy:

- scope batcha,
- liste lub commitment do zuzytych ticketow,
- commitment do receipts,
- sumaryczne amount/fee/refund semantics,
- podpis lub auth operatora.

### 5.6. SettlementRoot

`SettlementRoot` to commitment do kanonicznego batch summary.

Rola:

- stable reference do konkretnego settlementu,
- wygodne powiazanie metadata batcha z `receipt_root`,
- podstawowy anchor do audytu, storage i ewentualnego challenge path.

### 5.7. SettlementTx

`SettlementTx` to on-chain transakcja publikowana domyslnie przez marketplace operatora.

Jej zadaniem nie jest odtworzyc calej historii usage.
Jej zadaniem jest:

- zuzyc `ticket_nullifier`y,
- zapisac commitment do batcha,
- przesunac srodki zgodnie z wynikiem settlementu,
- zostawic minimalny, publicznie weryfikowalny slady.

## 6. Domyslne decyzje robocze do przyjecia, jesli nie ma lepszej kontrpropozycji

### D1. Receipt jest obiektem settlementowym, nie luznym logiem

Domyslna decyzja:

- receipt ma kanoniczna postac,
- receipt ma commitment,
- receipt nadaje sie do batching i dispute,
- receipt ma byc minimalny, ale formalny.

### D2. Merchant tworzy receipt, operator tworzy batch

Domyslna decyzja:

- merchant wystawia lub wspolwystawia receipt,
- operator agreguje receipty,
- operator publikuje `SettlementTx`,
- merchant nie jest domyslna settlement authority v0.

### D3. V0 publikuje jawna liste `ticket_nullifier`

Domyslna decyzja:

- on-chain v0 publikuje eksplicytna liste `ticket_nullifier` zuzywanych przez batch,
- chain musi umiec bezposrednio oznaczyc te nullifiery jako zuzyte,
- sam `nullifier_root` bez listy nie jest wystarczajacy dla v0.

`nullifier_root` moze istniec jako dodatkowy commitment batcha, ale nie zastapi jawnej listy w podstawowym modelu v0.

### D4. V0 uzywa jednoczesnie `receipt_root` i `settlement_root`

Domyslna decyzja:

- `receipt_root` komituje tresc receipts,
- `settlement_root` komituje kanoniczny batch summary,
- te dwa rooty nie sa tozsame semantycznie.

### D5. Batch jest co najmniej merchant-window scoped

Domyslna decyzja:

- batch settlementowy ma scope co najmniej:
  - `merchant_commit`
  - `settlement_window`
- opcjonalnie:
  - `service_commit`
  - `grant_commit`

Nie chcemy batchy marketplace-global bez scope, jesli nie ma bardzo mocnego uzasadnienia.

### D6. Konsensus sprawdza nullifiery i authority, nie cala semantyke uslugi

Domyslna decyzja:

chain sprawdza twardo:

- poprawny format `SettlementTx`,
- autoryzacje operatora,
- brak duplikatow `ticket_nullifier` wewnatrz batcha,
- brak wczesniejszego zuzycia tych nullifierow globalnie,
- zgodnosc podstawowych sum i pol batch headera,
- syntaktyczna zgodnosc scope batcha.

Chain nie odtwarza w v0:

- calego receipt set,
- calej semantyki uslugi,
- calego usage logu,
- calej logiki grantu off-chain.

To jest jawne ograniczenie v0, nie przeoczenie.

### D7. Refund/dispute musza miec anchor w receipt layer

Domyslna decyzja:

- refund path i dispute path maja sie opierac o receipt layer,
- nie wolno projektowac settlementu, ktory zostawia tylko `amount + nullifier` bez audit anchor.

## 7. Wymagany model receiptu

Masz pracowac z nastepujacym szkicem:

```text
Receipt
  = receipt header
  + debit context
  + purchase context
  + service result context
  + policy context
  + auth/signature material
```

Masz odpowiedziec:

1. jakie pola sa obowiazkowe,
2. ktore pola sa publiczne,
3. ktore pola sa tylko off-chain,
4. kto podpisuje receipt,
5. czy receipt jest jednostronny czy dwustronny,
6. czy payer musi potwierdzac receipt ex post, czy wystarcza wczesniejsza autoryzacja ticketem,
7. jak receipt laczy sie z `ticket_id`, `ticket_nullifier`, `purchase_commit`, `session_commit`, `grant_commit`.

Na starcie przyjmij, ze receipt powinien umiec niesc co najmniej:

- `receipt_id`
- `merchant_commit`
- `service_commit`
- `session_commit`
- `grant_commit`
- `purchase_commit`
- `ticket_id` lub rownowazny local debit reference
- `ticket_nullifier`
- `amount`
- `pricing_unit` lub usage unit summary
- `policy_commit`
- `result_commit`
- `issued_at`
- `merchant_sig`
- optional `operator_ack` lub `payer_ack`

Mozesz zaproponowac lepszy minimalny zestaw, ale nie wolno wyjsc ponizej poziomu, ktory psuje refund/dispute semantics.

## 8. Wymagany model batcha i rootow

Masz rozstrzygnac:

### 8.1. Jak budujemy `receipt_root`

Oceń co najmniej:

- Merkle tree nad `ReceiptCommit`
- inne commitment tree / accumulator

Domyslny kierunek v0:

- klasyczne Merkle tree nad kanonicznymi `ReceiptCommit`

Powod:

- prostota implementacyjna,
- latwa interoperacyjnosc,
- czytelny audit path,
- naturalny future-proofing pod challenge path.

### 8.2. Jak budujemy `settlement_root`

Masz zaproponowac batch summary, ktory komituje co najmniej:

- `merchant_commit`
- optional `service_commit`
- `grant_commit` lub rownowazny settlement authority scope
- `settlement_window`
- `receipt_root`
- `receipt_count`
- `nullifier_count`
- `total_gross_amount`
- `total_fee_amount`
- `total_refund_amount`
- `batch_mode`

### 8.3. Flat list vs root-only

Musisz porownac dwa warianty:

#### Wariant A. Flat nullifier list on-chain

- jawna lista `ticket_nullifier`
- `receipt_root`
- `settlement_root`

Zalety:

- najprostsza integracja z globalnym zbiorem nullifierow,
- prosta logika konsensusu,
- dobra audytowalnosc v0.

Wady:

- wiekszy on-chain payload.

#### Wariant B. Root-only / compressed publication

- `nullifier_root`
- `receipt_root`
- `settlement_root`

Zalety:

- mniejszy payload.

Wady:

- chain nie moze bezposrednio oznaczyc poszczegolnych nullifierow jako zuzytych bez dodatkowego mechanizmu,
- wyzsza zlozonosc,
- slabsza ergonomia v0.

Domyslna decyzja v0:

- `Flat nullifier list on-chain + roots jako audit commitments`

## 9. Wymagany model SettlementTx

Masz opisac kanoniczny `SettlementTx`.

Na starcie przyjmij, ze publiczne pola powinny zawierac co najmniej:

- `version`
- `operator_commit` lub operator identity reference
- `merchant_commit`
- optional `service_commit`
- `grant_commit`
- `settlement_window_start`
- `settlement_window_end`
- `receipt_root`
- `settlement_root`
- `receipt_count`
- `nullifier_count`
- `total_gross_amount`
- `total_fee_amount`
- `total_refund_amount`
- jawna liste `ticket_nullifier`
- `operator_sig`

Masz odpowiedziec:

1. czy `purchase_commit` ma byc jawny per receipt on-chain czy tylko pod `receipt_root`,
2. czy `amount` ma byc jawny per debit on-chain czy tylko jako batch aggregate,
3. czy batch ma byc merchant-only, merchant+service, czy grant-scoped,
4. czy settlement moze laczyc wiele sesji,
5. jak rozliczyc refund delta.

## 10. Wymagany model weryfikacji konsensusu

To jest sekcja krytyczna.

Masz odpowiedziec, co dokladnie sprawdza konsensus PC-BFT przy `SettlementTx`.

Musisz rozdzielic:

### 10.1. Twarde sprawdzenia konsensusu

Na starcie przyjmij jako minimum:

- poprawny format tx,
- poprawny podpis operatora,
- operator ma prawo publikowac settlement,
- brak duplikatow `ticket_nullifier` w batchu,
- brak wczesniejszego zuzycia `ticket_nullifier` globalnie,
- zgodnosc `nullifier_count` z lista,
- podstawowa zgodnosc `settlement_root` z headerem,
- podstawowa zgodnosc `receipt_root` z zadeklarowanym batch summary,
- nieujemne i poprawnie zakodowane amount totals.

### 10.2. Czego konsensus nie sprawdza w v0

Masz jawnie opisac, czego chain w v0 nie udowadnia:

- czy merchant naprawde wykonal usluge,
- czy usage byl "fair",
- czy pricing policy byla biznesowo sluszna,
- czy grant byl wykorzystany optymalnie,
- czy receipt set jest uczciwy poza zakresem operator-trusted accounting.

To musi byc nazwane jako `trust assumption`, nie zamiecione pod dywan.

### 10.3. Gdzie zyje audyt i dispute

Masz odpowiedziec:

- czy dispute opiera sie o ujawnienie konkretnych receipts,
- czy `receipt_root` wystarcza jako audit anchor,
- kto przechowuje receipts,
- jak dlugo receipts musza byc retencjonowane.

## 11. Wymagany model refund, timeout i failure path

Masz opisac co dzieje sie, gdy:

1. ticket zostal wydany, ale usluga nie zostala wykonana,
2. merchant wystawil receipt, ale operator nie settlementuje batcha na czas,
3. operator chce settlementowac tylko czesc receipts,
4. sesja wygasla,
5. grant wygasl,
6. purchase zostal anulowany.

Masz rozstrzygnac:

- czy refund jest osobnym batch path,
- czy refund moze byc netowany w `SettlementTx`,
- czy timeout zamyka mozliwosc pozniejszego claimu,
- jak receipt layer pomaga rozstrzygnac konflikt.

## 12. Pytania, ktore musza dostac odpowiedz

Nie wolno zostawic tych pytan bez odpowiedzi:

1. Jak wyglada minimalny kanoniczny `Receipt`?
2. Kto podpisuje `Receipt`?
3. Czy `Receipt` jest jednostronny czy dwustronny?
4. Czy `ticket_nullifier` wchodzi do receiptu?
5. Jak receipt laczy sie z `purchase_commit`?
6. Jak budujemy `receipt_root`?
7. Jak budujemy `settlement_root`?
8. Czy v0 publikuje jawna liste `ticket_nullifier`, czy tylko root?
9. Co dokladnie trafia do `SettlementTx`?
10. Co dokladnie sprawdza konsensus?
11. Co pozostaje trusted accounting assumption operatora?
12. Kto publikuje settlement w v0?
13. Jak wyglada refund path?
14. Jak wyglada partial settlement?
15. Jak receipts sa przechowywane i retencjonowane?

## 13. Czego nie wolno proponowac

Nie proponuj:

- settlementu bez `ticket_nullifier`,
- settlementu bez `receipt_root` lub rownowaznego audit anchor,
- modelu, w ktorym merchant sam jest domyslna authority od rozliczenia depozytu,
- modelu, w ktorym chain ma widziec cala historie usage eventow per session,
- modelu, w ktorym `receipt` jest tylko nieformalnym JSON logiem,
- modelu, w ktorym `receipt_root` i `settlement_root` sa wrzucone jako ozdobne hashe bez semantyki,
- modelu, w ktorym refund/dispute nie maja zadnego anchoru w receipt layer,
- modelu root-only, ktory nie rozwiazuje problemu globalnej jednorazowosci nullifierow.

## 14. Wymagany format odpowiedzi

Odpowiedz ma miec dokladnie te sekcje:

1. `Recommended v0 design`
2. `Alternative design`
3. `Receipt data model`
4. `Batch and root model`
5. `SettlementTx data model`
6. `Consensus verification model`
7. `Refund and failure model`
8. `Retention and audit model`
9. `Frozen decisions`
10. `Open questions`

## 15. Oczekiwany wynik

Po przeczytaniu tego dokumentu i wykonaniu zadania mamy dostac:

- gotowa definicje receiptu,
- gotowa definicje `receipt_root`,
- gotowa definicje `settlement_root`,
- gotowa definicje `SettlementTx`,
- gotowa odpowiedz, czy v0 publikuje jawna liste nullifierow,
- gotowa odpowiedz, co weryfikuje konsensus,
- gotowa odpowiedz, co pozostaje trusted accounting assumption,
- gotowy szkic refund / timeout / partial settlement path,
- minimalna liste pytan otwartych, jesli cos naprawde musi zostac otwarte.

Nie chcemy po tej iteracji dostac kolejnego brainstormingu.
Chcemy dostac material, z ktorego da sie robic spec i implementacje.
