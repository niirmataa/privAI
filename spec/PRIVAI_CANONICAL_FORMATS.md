# privAI Canonical Formats

Status: draft canonical formats doc in migration.
Canonicality: intended canonical source of truth for bytes, field order, hash domains and commitment formulas. Product semantics remain governed by `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`. Protocol semantics remain governed by `spec/PRIVAI_PROTOCOL_CORE.md`. Pliki poza `spec/` nie sa normatywnym source of truth.
Owner: privAI protocol formats.
Depends on: `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`, `spec/PRIVAI_PROTOCOL_CORE.md`.
Supersedes: czesciowo `PRIVAI_V0_FORMATS.md`.

## 1. Cel

Ten dokument opisuje kanoniczny format obiektow `privAI` na poziomie bajtow:
- reguly `CanonicalEncode`,
- kolejnosc pol,
- domain strings,
- formuly commitmentow,
- status formatow finalnych i przejsciowych.

To nie jest dokument polityki produktu.
To nie jest tez dokument wysokopoziomowej semantyki protokolu.

## 1.1. How Devs And Agents Should Use This Doc

Ten dokument jest zrodlem prawdy dla bytes, field order, domain strings i commitment formulas.

Czytaj go razem z:
- `spec/PRIVAI_SPEC_INDEX.md` jako punktem wejscia do calego frozen setu,
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md` dla polityki produktu,
- `spec/PRIVAI_PROTOCOL_CORE.md` dla znaczenia obiektow,
- `spec/PRIVAI_CONSENSUS.md` dla semantyki consensus objects.

Zamrozona zasada pracy:
- jesli task dotyczy serializacji, hash domains, commitments, signed envelopes albo merkle roots, to ten dokument jest punktem wyjscia,
- jesli kod `CanonicalEncode` rozjezdza sie z tym dokumentem, to jest bug albo jawna migration gap, a nie powod do lokalnego "zgadywania" formatu,
- kod i stare docs poza `spec/` moga sluzyc tylko do znalezienia current behavior, a nie do nadpisywania frozen format rules,
- current code format i frozen final target format nie sa tym samym i trzeba je rozrozniac jawnie tam, gdzie migration nie jest jeszcze domknieta.

Jawnego freeze update wymagaja zawsze:
- zmiana kolejnosci pol,
- zmiana domain string,
- zmiana commitment formula,
- zmiana signed-envelope boundaries,
- zmiana final canonical amount encoding,
- zmiana dziwnych legacy traits typu `MarketplaceBatchTx.operator_sig`.

Interpretation rule for this document:
- status labels sa zdefiniowane w `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`,
- jesli canonical detail nie ma jawnej etykiety statusu albo jawnej formuly, nalezy traktowac go jako `unresolved`,
- `unresolved` nie wolno wypelniac przez zgadywanie:
  - field order,
  - enum tags,
  - domain strings,
  - envelope bytes,
  - merkle rules,
  - commitment formulas.

## 2. Zasady ogolne kodowania

Aktualne reguly `CanonicalEncode` sa nastepujace:
- `u8` jest kodowane jako 1 bajt,
- `u16`, `u32`, `u64`, `i64` sa kodowane little-endian,
- stale tablice bajtow sa kodowane bez prefixu dlugosci,
- `bytes` sa kodowane jako `u32_len_le || raw_bytes`,
- `Option<bytes>` jest kodowane jako:
  - `0x00` dla `None`,
  - `0x01 || bytes` dla `Some`,
- `Vec<T: CanonicalEncode>` jest kodowane jako:
  - `u32_count_le`
  - potem kolejne elementy po ich canonical encoding, bez dodatkowego prefixu per element,
- `Vec<Vec<u8>>` jest kodowane jako:
  - `u32_count_le`
  - potem kazdy element jako `u32_len_le || raw_bytes`.

Zamrozona zasada:
- zmiana tych regul wymaga jawnego freeze update,
- nie wolno lokalnie wprowadzac "specjalnych przypadkow" kodowania dla pojedynczych struktur bez update tego dokumentu.

### 2.1. Formalne aliasy pomocnicze

Na potrzeby tego dokumentu:
- `option_bytes = Option<Vec<u8>>`
- `vec_bytes = Vec<Vec<u8>>`

Formalna interpretacja:
- `option_bytes` jest kodowane jako:
  - `0x00` dla `None`
  - `0x01 || u32_len_le || raw_bytes` dla `Some`
- `vec_bytes` jest kodowane jako:
  - `u32_count_le`
  - potem dla kazdego elementu `u32_len_le || raw_bytes`

## 3. Parametry i typy bazowe

Aktualne stale bazowe z kodu:
- `PRIVAI_V0 = 0x00`
- `DEFAULT_CHAIN_ID = 0x50524149`
- `LWE_MODULUS_Q = 4294967291`
- `PLAINTEXT_SPACE_P = 16384`
- `LWE_DIMENSION = 1024`
- `DELTA = 262143`
- `B_MAX = 131071`
- `FRODOKEM_640_SHAKE = 0x01`
- `AEAD_ALG_XCHACHA20_POLY1305 = 0x01`

Typy bazowe:
- `Hash32 = [u8; 32]`
- `BundleId = [u8; 16]`
- `ContextId = [u8; 16]`
- `BlockHeight = u64`
- `Flags8 = u8`

### 3.1. Amount14

Aktualny typ `Amount14`:
- jest kodowany jako `u16_le`,
- musi spelniac `value < 16384`.

Status:
- to jest aktualny kodowany typ amount dla obecnego witness/plaintext layer,
- nie jest to finalna ekonomiczna reprezentacja `PVA + aPVA`,
- finalny integer amount encoding dla systemu pozostaje do domkniecia.

Globalna nota:
- wszystkie obecne amount-bearing encodings pozostaja przejsciowe do czasu jawnego zamrozenia finalnych canonical amount rules dla `PVA + aPVA`.

### 3.2. Nullifier

`Nullifier`:
- jest wrapperem nad `Hash32`,
- jest kodowany jako surowe 32 bajty.

### 3.3. LweCiphertext

`LweCiphertext`:
- `a` musi miec dokladnie 1024 wspolczynniki,
- kazdy wspolczynnik jest kodowany jako `u32_le`,
- potem kodowane jest `b` jako `u32_le`.

Canonical encoding:
- `a[0]`
- `a[1]`
- ...
- `a[1023]`
- `b`

## 4. Rejestr domain strings

### 4.1. Core domains

Aktualne core domain strings:
- `NOTE_DOMAIN = "privai:note:v0"`
- `NOTE_PAYLOAD_DOMAIN = "privai:note-payload:v0"`
- `POLICY_DOMAIN = "privai:policy:v0"`
- `BUNDLE_DOMAIN = "privai:bundle:v0"`
- `NULLIFIER_DOMAIN = "privai:nullifier:v0"`
- `STATEMENT_DOMAIN = "privai:stmt:v0"`
- `AUX_DOMAIN = "privai:aux:v0"`
- `TX_DOMAIN = "privai:tx:v0"`
- `TX_SIGNING_DOMAIN = "privai:tx-signing:v0"`
- `FALCON_PK_DOMAIN = "privai:falcon-pk:v0"`
- `BLOCK_HEADER_DOMAIN = "privai:block-header:v0"`
- `PROOF_CERT_DOMAIN = "privai:proof-cert:v0"`
- `MERKLE_DOMAIN = "privai:merkle:v0"`
- `MERKLE_EMPTY_DOMAIN = "privai:merkle-empty:v0"`
- `EPOCH_SEED_DOMAIN = "privai:epoch-seed:v0"`
- `LITE_NOTE_DOMAIN = "privai:lite-note:v0"`
- `LITE_NOTE_PAYLOAD_DOMAIN = "privai:lite-note-payload:v0"`

Nowe domeny auth/escrow:
- `TX_SIGNING_DOMAIN` — domena dla `tx_signing_hash`, canonical signing message (nie `tx_id`),
- `FALCON_PK_DOMAIN` — domena dla `falcon_pk_hash()`, canonical signer identity hash.

### 4.2. Marketplace domains

Aktualne marketplace domain strings:
- `SERVICE_PAYMENT_POLICY_DOMAIN = "nxms_privai_policy_v0"`
- `SPEND_GRANT_DOMAIN = "nxms_privai_grant_v0"`
- `RECEIPT_DOMAIN = "nxms_privai_receipt_v0"`
- `RECEIPT_ROOT_DOMAIN = "nxms_privai_receipt_root_v0"`
- `SETTLEMENT_ROOT_DOMAIN = "nxms_privai_settlement_root_v0"`
- `OPERATOR_GRANT_SIG_DOMAIN = "nxms_privai_grant_sig_v0"`
- `MERCHANT_RECEIPT_SIG_DOMAIN = "nxms_privai_receipt_sig_v0"`
- `MARKETPLACE_BATCH_SIG_DOMAIN = "nxms_privai_batch_sig_v0"`

Wazna uwaga:
- core `SpendPolicy` i marketplace `ServicePaymentPolicy` nie korzystaja z tego samego domain string,
- nie wolno ich mieszac ani utozsamiac,
- `MARKETPLACE_BATCH_SIG_DOMAIN` jest zarezerwowany dla frozen future target requiring migration,
- nie opisuje jeszcze obecnego current-canonical verifier behavior dla `MarketplaceBatchTx`.

## 5. Regula hashowania

Aktualna funkcja `domain_hash(domain, parts)`:
- tworzy nowy BLAKE3 hasher,
- hashuje `domain.as_bytes()`,
- potem kolejno hashuje wszystkie `parts`,
- nie dodaje ukrytych separatorow miedzy parts poza tym, co zostalo jawnie zakodowane przez wywolujacego.

Zamrozona interpretacja:
- jesli formula commitmentu korzysta z `to_canonical_bytes()`, to wszelka jednoznacznosc wynika z canonical encoding,
- jesli formula commitmentu sklada payload recznie, kolejnosc i typy pol musza byc traktowane jako czesc wire definition.

## 6. Core note objects

### 6.1. ReceiveBundle

Canonical encoding `ReceiveBundle`:
1. `version: u8`
2. `bundle_id: [u8; 16]`
3. `expires_at: u64_le`
4. `flags: u8`
5. `one_time_falcon_pk: bytes`
6. `one_time_frodo_pk: bytes`
7. `route_hint: option_bytes`
8. `nullifier_key: [u8; 32]`

Commitment:
- `bundle_commit = H_bundle(canonical(ReceiveBundle))`

Security note:
- `nullifier_key` jest polem canonical formatu,
- jego semantyczna akceptacja jest regulowana przez protocol-core i wallet verification,
- samo zakodowanie pola nie oznacza automatycznej akceptacji jego wartosci przez wallet.

### 6.2. SpendPolicy

`SpendPolicy::Single`:
1. tag `0x01`
2. `falcon_pk_hash: [u8; 32]`

`SpendPolicy::MarketplaceSettlement`:
1. tag `0x02`
2. `buyer_pk_hash: [u8; 32]`
3. `seller_pk_hash: [u8; 32]`
4. `moderator_pk_hash: [u8; 32]`
5. `timeout_block: u64_le`

`SpendPolicy::Escrow2of3`:
1. tag `0x03`
2. `buyer_pk_hash: [u8; 32]`
3. `merchant_pk_hash: [u8; 32]`
4. `operator_pk_hash: [u8; 32]`
5. `timeout_block: u64_le`

Wazna uwaga:
- `buyer/merchant/operator_pk_hash` sa liczone jako `falcon_pk_hash(raw_falcon_pk)` z domena `FALCON_PK_DOMAIN`,
- canonical signer ordering: Buyer (index 0), Merchant (index 1), Operator (index 2),
- action rules sa implied przez policy_tag `0x03` — nie sa zakodowane w polach policy,
- `timeout_block` determinuje moment dostepnosci recovery mode.

Commitment:
- `spend_policy_commit = H_policy(canonical(SpendPolicy))`

### 6.3. RecipientBox

Canonical encoding `RecipientBox`:
1. `version: u8`
2. `kem_alg: u8`
3. `aead_alg: u8`
4. `kem_ct: bytes`
5. `nonce: [u8; 24]`
6. `ciphertext: bytes`
7. `tag: [u8; 16]`
8. `hint: [u8; 16]`

### 6.4. RecipientBoxPlaintext

Canonical encoding `RecipientBoxPlaintext`:
1. `version: u8`
2. `bundle_id: [u8; 16]`
3. `note_payload_commit: [u8; 32]`
4. `amount: Amount14`
5. `witness_seed: [u8; 32]`
6. `nullifier_key: [u8; 32]`
7. `spend_policy_opening: bytes`
8. `aux_opening: bytes`
9. `sender_memo: option_bytes`

Zamrozona interpretacja:
- aktualnym bindingiem plaintextu jest `note_payload_commit`,
- nie wolno wracac do starszego opisu, w ktorym plaintext byl powiazany tylko ze starym `note_commit`.

### 6.5. AuxWitness

Canonical encoding `AuxWitness`:
1. `version: u8`
2. `amount: Amount14`
3. `witness_seed: [u8; 32]`
4. `noise_class: u8`
5. `bundle_id: [u8; 16]`

Commitment:
- `aux_commit = H_aux(canonical(AuxWitness))`

### 6.6. OutputNote

Canonical encoding `OutputNote`:
1. `version: u8`
2. `note_commit: [u8; 32]`
3. `spend_policy_commit: [u8; 32]`
4. `ct_amt: LweCiphertext`
5. `aux_commit: [u8; 32]`
6. `recipient_box: RecipientBox`

Payload bytes for payload commitment:
1. `version: u8`
2. `spend_policy_commit: [u8; 32]`
3. `ct_amt: LweCiphertext`
4. `aux_commit: [u8; 32]`

Commitments:
- `note_payload_commit = H_note_payload(payload_bytes)`
- `note_commit = H_note(payload_bytes || canonical(RecipientBox))`

### 6.7. Nullifier derivation

Canonical formula:
- `nullifier = H_nullifier(note_commit || nullifier_key)`

`note_commit` i `nullifier_key` sa przekazywane jako surowe 32-bajtowe fragmenty.

## 7. Current lite-path formats

### 7.1. LiteOutputNote

Status:
- current code-defined format,
- nie jest jeszcze automatycznie finalnym formatem zamrozonego `OnChainLite`.

Canonical encoding `LiteOutputNote`:
1. `version: u8`
2. `note_commit: [u8; 32]`
3. `amount: u64_le`
4. `spend_policy_commit: [u8; 32]`
5. `aux_commit: [u8; 32]`
6. `recipient_box: RecipientBox`

Payload bytes for lite payload commitment:
1. `version: u8`
2. `spend_policy_commit: [u8; 32]`
3. `amount: u64_le`
4. `aux_commit: [u8; 32]`

Lite note commitment formula in current code:
- encode `version`
- encode `spend_policy_commit`
- encode `amount`
- encode `aux_commit`
- encode `recipient_box.hint`
- `lite_note_commit = H_lite_note(encoded_payload_with_hint)`

Current lite payload commitment:
- `lite_note_payload_commit = H_lite_note_payload(version || spend_policy_commit || amount || aux_commit)`

Wazna uwaga:
- obecny `LiteOutputNote.note_commit` binduje `hint`, a nie caly `RecipientBox`,
- jednoczesnie sam canonical encoding obiektu nadal przenosi pelny `RecipientBox`.

## 8. Input i tx core formats

### 8.1. InputRef

Canonical encoding:
1. `note_commit: [u8; 32]`

### 8.2. InputAuth

Canonical encoding:
1. `policy_tag: u8`
2. `signer_pks: vec_bytes`
3. `signatures: vec_bytes`
4. `policy_opening: option_bytes`
5. `escrow_action: option_u8`

Nowe pola (escrow v1):
- `policy_opening` — canonical encoding `SpendPolicy`, sluzy do rekonstrukcji policy i weryfikacji binding z `spend_policy_commit`,
- `escrow_action` — deklarowany typ akcji escrow (`0x01` Release, `0x02` Refund, `0x03` RecoveryRelease),
- dla non-escrow auth (np. `Single`): `policy_opening = Some(canonical(Single{...}))`, `escrow_action = None`,
- `option_u8` jest kodowane jako: `0x00` dla `None`, `0x01 || u8_value` dla `Some`.

FullPrivacy v1 mandatory auth rule:
- `policy_opening` jest WYMAGANE dla kazdego auth entry na railu `FullPrivacy`,
- ledger weryfikuje: `H_policy(policy_opening) == input_note.spend_policy_commit`,
- dopiero po tym bindingu ledger derywuje typ policy i wybiera sciezke walidacji,
- `policy_tag` jest tylko routing hint / early dispatch hint,
- twarda regula: `policy_tag` MUSI byc rowne tagowi derywowanemu z `policy_opening`,
- mismatch `policy_tag != derived_policy_tag(policy_opening)` = hard reject.

### 8.3. TxCore

Canonical encoding `TxCore`:
1. `version: u8`
2. `tx_type: u8`
3. `inputs: vec<InputRef>`
4. `input_nullifiers: vec<Nullifier>`
5. `outputs: vec<OutputNote>`
6. `fee: u64_le`
7. `statement_commit: [u8; 32]`
8. `auth: vec<InputAuth>`

### 8.4. LiteTxCore

Canonical encoding `LiteTxCore`:
1. `version: u8`
2. `tx_type: u8`
3. `inputs: vec<InputRef>`
4. `input_nullifiers: vec<Nullifier>`
5. `outputs: vec<LiteOutputNote>`
6. `fee: u64_le`
7. `statement_commit: [u8; 32]`
8. `auth: vec<InputAuth>`

### 8.5. Transaction wrapper

Aktualny enum `Transaction` nie dodaje dodatkowego outer discriminator w canonical encoding.

Zamrozona interpretacja stanu obecnego:
- canonical bytes `Transaction` sa po prostu canonical bytes wewnetrznego wariantu,
- rozroznienie typu transakcji opiera sie na `tx_type` zakodowanym w `TxCore` lub `LiteTxCore`.

Tx hash:
- `tx_id = H_tx(canonical(Transaction))`

Tx signing hash:
- `tx_signing_hash = H_tx_signing(signing_preimage(Transaction))`
- `signing_preimage` to canonical tx body BEZ signature bytes — patrz section 8.6.

### 8.6. tx_signing_hash preimage

`tx_signing_hash` jest canonical signing message, oddzielna od `tx_id`.

Powod:
- `tx_id` obejmuje `auth` (w tym sygnatury),
- sygnatury nie moga byc liczone nad wlasnym wynikiem,
- `tx_signing_hash` rozwiazuje ta cykliczna zaleznosc.

Preimage formula (`signing_preimage`):
- canonical encoding calego tx body, ale dla kazdego `InputAuth`:
  - kodowane sa: `policy_tag`, `signer_pks` (bez sygnatur), `policy_opening`, `escrow_action`,
  - `signatures` sa POMINIETE.

Wazna uwaga:
- skoro `policy_tag` jest czescia `signing_preimage`, jego wartosc musi byc kanonicznie spojna z `policy_opening`,
- inaczej ten sam policy/auth state moglby dawac rozne `tx_signing_hash`,
- dlatego ledger musi hard-rejectowac kazdy auth entry, dla ktorego `policy_tag` nie zgadza sie z typem policy wyprowadzonym z `policy_opening`.

Hash:
- `tx_signing_hash = H_tx_signing(signing_preimage)`
- domena: `TX_SIGNING_DOMAIN = "privai:tx-signing:v0"`

Zamrozona zasada:
- auth artifacts sa ZAWSZE liczone nad `tx_signing_hash`,
- NIE nad `tx_id`,
- patrz `spec/PRIVAI_AUTH_SIGNING_MODEL.md` section 5.

## 9. Payment transaction formats

### 9.1. TransferNoteTx

Canonical encoding:
- identyczne z canonical encoding `TxCore`

Current `tx_type`:
- `0x01`

### 9.2. LiteTransferTx

Canonical encoding:
- identyczne z canonical encoding `LiteTxCore`

Current `tx_type`:
- `0x08`

Status:
- current code-defined format,
- semantycznie pozostaje `experimental` do czasu pelnego freeze `OnChainLite`.

### 9.3. MarketplaceBatchTx

Canonical encoding `MarketplaceBatchTx`:
1. `core: TxCore`
2. `summary: SettlementBatchSummary`
3. `ticket_nullifiers: vec<Nullifier>`
4. `operator_sig: vec_bytes` zakodowane jako wektor o dlugosci 1, zawierajacy jedno `operator_sig`

Wazna uwaga:
- obecny encoding `operator_sig` nie jest surowym `bytes`,
- jest kodowany przez `write_vec_bytes` jako lista jednego elementu,
- ta cecha musi byc jawnie zachowana, dopoki nie zostanie jawnie zmieniona przez freeze update.

Current `tx_type`:
- `0x05`

Zamrozona decyzja migracyjna:
- obecny `operator_sig` pozostaje jawnie opisanym historical quirk formatu,
- nie wolno go po cichu normalizowac do surowego `bytes`,
- ewentualna przyszla normalizacja do `bytes` wymaga jawnego freeze update i wire-format migration.

## 10. Marketplace objects

### 10.1. ServicePaymentPolicy

Finalna zasada semantyczna:
- wszystko, co wplywa na autoryzacje, charging, refund, dispute, timeout, batching i eskalacje do `FullPrivacy`, musi byc bindowane przez `policy_commit`,
- reguly nie powinny byc kodowane jako tekst ani niejawna logika merchanta,
- finalny model uzywa `tag + params`, a nie opisowych stringow.

Current code canonical encoding:
1. `policy_version: u8`
2. `merchant_commit: [u8; 32]`
3. `service_commit_present: u8`
4. `service_commit: [u8; 32]` tylko jesli present
5. `allowed_rail: u8`
6. `pricing_mode: u8`
7. `min_deposit_required: u64_le`
8. `max_spend_per_session: u64_le`
9. `max_spend_per_window: u64_le`
10. `grant_expiry_rule: u32_le`
11. `settlement_window_rule: u32_le`
12. `requires_full_privacy_if: u64_le`

Frozen final target extension after explicit format migration:
1. `policy_version: u8`
2. `merchant_commit: [u8; 32]`
3. `service_commit_present: u8`
4. `service_commit: [u8; 32]` tylko jesli present
5. `allowed_rail: u8`
6. `pricing_mode: u8`
7. `reservation_mode: u8`
8. `min_deposit_required: u64_le`
9. `max_spend_per_session: u64_le`
10. `max_spend_per_window: u64_le`
11. `max_usage_units: u64_le`
12. `grant_expiry_rule_tag: u8`
13. `grant_expiry_rule_param: u32_le`
14. `settlement_window_rule_tag: u8`
15. `settlement_window_rule_param: u32_le`
16. `acceptance_rule_tag: u8`
17. `acceptance_rule_param: u32_le`
18. `refund_rule_tag: u8`
19. `refund_rule_param: u32_le`
20. `dispute_rule_tag: u8`
21. `dispute_rule_param: u32_le`
22. `timeout_rule_tag: u8`
23. `timeout_rule_param: u32_le`
24. `batching_rule_tag: u8`
25. `batching_rule_param: u32_le`
26. `requires_full_privacy_if: u64_le`

Current code status:
- obecny kod implementuje wezszy legacy format policy,
- dopoki migration nie zostanie domkniety, `policy_commit` w kodzie binduje current code encoding,
- po jawnej migracji canonical source of truth przechodzi na rozszerzony final target encoding.

Commitment:
- `policy_commit = H_marketplace_policy(canonical(ServicePaymentPolicy))`

### 10.2. SpendGrant

Canonical encoding:
1. `merchant_commit: [u8; 32]`
2. `service_commit_present: u8`
3. `service_commit: [u8; 32]` tylko jesli present
4. `session_scope: [u8; 32]`
5. `spend_cap: u64_le`
6. `grant_expiry: u64_le`
7. `settlement_window: u64_le`
8. `policy_commit: [u8; 32]`

Wazna uwaga:
- `operator_sig` nie wchodzi do canonical bytes `SpendGrant`,
- `operator_sig` nie wchodzi do `grant_commit`.

Commitment:
- `grant_commit = H_grant(canonical(SpendGrant))`

### 10.3. Receipt

Canonical encoding:
1. `receipt_id: [u8; 32]`
2. `merchant_commit: [u8; 32]`
3. `service_commit_present: u8`
4. `service_commit: [u8; 32]` tylko jesli present
5. `session_commit: [u8; 32]`
6. `grant_commit: [u8; 32]`
7. `purchase_commit: [u8; 32]`
8. `ticket_nullifier: [u8; 32]`
9. `amount: u64_le`
10. `policy_commit: [u8; 32]`
11. `result_commit: [u8; 32]`
12. `issued_at: u64_le`

Wazna uwaga:
- `merchant_sig` nie wchodzi do canonical bytes `Receipt`,
- `merchant_sig` nie wchodzi do `receipt_commit`.

Commitment:
- `receipt_commit = H_receipt(canonical(Receipt))`

### 10.4. SettlementBatchSummary

Canonical encoding:
1. `operator_commit: [u8; 32]`
2. `merchant_commit: [u8; 32]`
3. `grant_commit: [u8; 32]`
4. `settlement_window_start: u64_le`
5. `settlement_window_end: u64_le`
6. `receipt_root: [u8; 32]`
7. `receipt_count: u32_le`
8. `nullifier_count: u32_le`
9. `total_gross_amount: u64_le`
10. `total_fee_amount: u64_le`
11. `total_refund_amount: u64_le`

Commitment:
- `settlement_root = H_settlement_root(canonical(SettlementBatchSummary))`

### 10.5. Signed envelopes

Zamrozona zasada:
- `canonical body`
- `commitment`
- `signed envelope`
sa trzema odrebnymi warstwami i nie wolno ich mieszac.

`SpendGrant`:
- `grant_commit = H_grant(canonical(SpendGrant))`
- `operator_sig = Sign(OPERATOR_GRANT_SIG_DOMAIN || canonical(SpendGrant))`
- `SpendGrantEnvelope = SpendGrant + operator_sig`

`Receipt`:
- `receipt_commit = H_receipt(canonical(Receipt))`
- `merchant_sig = Sign(MERCHANT_RECEIPT_SIG_DOMAIN || canonical(Receipt))`
- `ReceiptEnvelope = Receipt + merchant_sig`

`MarketplaceBatchTx` current canonical rule:
- `settlement_root = H_settlement_root(canonical(SettlementBatchSummary))`
- current verifier message = `settlement_root`
- `operator_sig = Sign(settlement_root)`
- `operator_sig` nie wchodzi do commitmentu `settlement_root`
- w obecnym kodzie `operator_sig` pozostaje czescia canonical encoding `MarketplaceBatchTx`, a wiec historycznie wplywa na `tx_id`

`MarketplaceBatchTx` frozen future target requiring migration:
- `BatchSignedPayload = canonical(core || summary || ticket_nullifiers)`
- `operator_sig = Sign(MARKETPLACE_BATCH_SIG_DOMAIN || BatchSignedPayload)`
- ta regula nie jest jeszcze current canonical verifier rule

Wazna uwaga:
- podpis nie zastepuje commitmentu,
- commitment nie zastepuje podpisu,
- signed envelope nie jest tym samym co canonical body.

## 11. Merkle roots and current gaps

Aktualna pomocnicza funkcja `merkle_root`:
- dla pustej listy zwraca `H_merkle_empty()`,
- dla nieparzystej liczby lisci duplikuje ostatni hash w parze,
- kazdy parent liczony jest jako `H_merkle(left || right)`.

Current gaps do jawnego domkniecia:
- kod ma `RECEIPT_ROOT_DOMAIN`, ale nadal brakuje jednego publicznego helpera w crate, ktory liczy `receipt_root` z listy `receipt_commit`,
- frozen spec rule i referencyjne vectors dla `receipt_root` i bazowych commitmentow sa w `spec/PRIVAI_REFERENCE_VECTORS.md` oraz w testach referencyjnych w `privai-chain/tests/`.

## 12. Status formatow

Za formaty docelowo finalne nalezy uznac po pelnym freeze:
- `ReceiveBundle`
- `SpendPolicy`
- `RecipientBox`
- `RecipientBoxPlaintext`
- `AuxWitness`
- `OutputNote`
- `Nullifier`
- `TransferNoteTx`
- marketplace objects i `MarketplaceBatchTx`

Za formaty przejsciowe / experimental nalezy uznac obecnie:
- `LiteOutputNote`
- `LiteTransferTx`
- wszelkie amount-layer zaleznosci, ktore nadal opieraja sie o `Amount14` jako przejsciowy sufit matematyczny

## 13. Co trzeba domknac przed pelnym freeze

- dopisac golden vectors dla wszystkich finalnych struktur,
- dopisac vectors dla commitment formulas,
- dopisac vectors dla `tx_signing_hash` preimage,
- dopisac vectors dla `SpendPolicy::Escrow2of3` canonical encoding i commitment,
- dopisac vectors dla `InputAuth` z `policy_opening` i `escrow_action`,
- dopisac finalny receipt-root helper w kodzie (opcjonalnie; vectors sa w `spec/PRIVAI_REFERENCE_VECTORS.md`),
- zsynchronizowac finalny integer amount encoding z `PVA + aPVA`,
- potwierdzic, czy obecny encoding lite path zostaje bez zmian czy zostaje zastapiony przez nowy finalny `OnChainLite` format,
- usunac wszelkie rozjazdy miedzy starym `PRIVAI_V0_FORMATS.md`, tym dokumentem i kodem,
- wdrozyc mandatory auth (Option B) w kodzie ledgera i zaktualizowac testy.
