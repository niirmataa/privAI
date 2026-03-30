# privAI v0 Concrete Formats

## 1. Cel dokumentu

Ten dokument zamraza konkretne formaty obiektow i payloadow dla `privAI v0`.
Ma byc pomostem miedzy architektura a implementacja.

Zakres:

- canonical primitives,
- `ReceiveBundle`,
- `SpendPolicy`,
- `RecipientBox`,
- `Note`,
- `Nullifier`,
- `TransferNoteTx`,
- `PRIVAI/1` message types nad NXMS.

## 2. Canonical primitives

Wszystkie hashe i bindingi sa liczone przez `BLAKE3`.
Wszystkie liczby calkowite serializujemy little-endian.
Wszystkie ciagi bajtow serializujemy jako `u32_len || bytes`.

Typy bazowe:

```text
Hash32      = [u8; 32]
BundleId    = [u8; 16]
ContextId   = [u8; 16]
BlockHeight = u64
Amount14    = u16   where value < 16384
Flags8      = u8
```

Funkcje domenowe:

```text
H_note(x)      = BLAKE3("privai:note:v0"      || x)
H_policy(x)    = BLAKE3("privai:policy:v0"    || x)
H_bundle(x)    = BLAKE3("privai:bundle:v0"    || x)
H_nullifier(x) = BLAKE3("privai:nullifier:v0" || x)
H_stmt(x)      = BLAKE3("privai:stmt:v0"      || x)
H_aux(x)       = BLAKE3("privai:aux:v0"       || x)
```

## 3. LWE ciphertext encoding

Kwota noty jest kodowana jako swiezy ciphertext LWE.
Dla v0 przyjmujemy:

```text
q = 4294967291
n = 1024
```

Canonical encoding:

```rust
struct LweCiphertext {
    a: [u32; 1024],
    b: u32,
}
```

Zasady:

- kazdy element `a[i]` i `b` jest interpretowany mod `q`,
- serializacja to 1024 razy `u32_le` plus jedno `u32_le`,
- rozmiar canonical = `4100 B`.

## 4. ReceiveBundle

`ReceiveBundle` jest jednorazowym ukrytym adresem odbiorczym.
Jest naturalnym nastepca `prekeys` z `nexum-cli`.

```rust
struct ReceiveBundle {
    version: u8,
    bundle_id: BundleId,
    expires_at: u64,
    flags: u8,
    one_time_falcon_pk: Vec<u8>,
    one_time_frodo_pk: Vec<u8>,
    route_hint: Option<Vec<u8>>,
}
```

Pole `route_hint` jest opcjonalne i moze zawierac:

- mailbox endpoint,
- onion hint,
- katalog marketplace,
- identyfikator warstwy relay.

Commitment bundle:

```text
bundle_commit = H_bundle(canonical_receive_bundle)
```

Zasady v0:

- `version = 0x00`
- `flags`:
  - `0x01` uploaded
  - `0x02` used
  - `0x04` revoked
- bundle jest jednorazowy,
- sender nie powinien uzywac bundle po `expires_at`.

## 5. SpendPolicy

Spend policy opisuje warunek wydania noty.

```rust
enum SpendPolicy {
    Single {
        falcon_pk_hash: Hash32,
    },
    Escrow2of3 {
        buyer_pk_hash: Hash32,
        seller_pk_hash: Hash32,
        arbiter_pk_hash: Hash32,
        timeout_block: u64,
    },
}
```

Canonical `policy_tag`:

```text
0x01 = Single
0x02 = Escrow2of3
```

Commitment polityki:

```text
spend_policy_commit = H_policy(canonical_spend_policy)
```

## 6. RecipientBox

`RecipientBox` zawiera dane potrzebne odbiorcy do rozpoznania i wydania noty.
Nie moze byc jedynym zrodlem prawdy o funduszach, ale musi byc wystarczajacy do odtworzenia witnessa odbiorczego.

### 6.1. On-chain box

```rust
struct RecipientBox {
    version: u8,
    kem_alg: u8,
    aead_alg: u8,
    kem_ct: Vec<u8>,
    nonce: [u8; 24],
    ciphertext: Vec<u8>,
    tag: [u8; 16],
    hint: [u8; 16],
}
```

Wartosci v0:

```text
version  = 0x00
kem_alg  = 0x01  // FrodoKEM-640-SHAKE
 aead_alg = 0x01  // XChaCha20-Poly1305
```

`hint` nie jest sekretne. Moze sluzyc do:

- szybszego skanowania walleta,
- indeksowania lokalnego,
- powiazania z `bundle_id` bez ujawniania calego bundle.

### 6.2. Plaintext box

```rust
struct RecipientBoxPlaintext {
    version: u8,
    bundle_id: BundleId,
    note_commit: Hash32,
    amount: Amount14,
    witness_seed: Hash32,
    nullifier_key: Hash32,
    spend_policy_opening: Vec<u8>,
    aux_opening: Vec<u8>,
    sender_memo: Option<Vec<u8>>,
}
```

Zasady:

- `witness_seed` sluzy do odtworzenia prywatnego witnessa noty,
- `nullifier_key` jest sekretem spendera do wyliczenia `nullifier`,
- `spend_policy_opening` pozwala odbiorcy odtworzyc pelny `SpendPolicy`,
- `aux_opening` sluzy do otwarcia `aux_commit`,
- `sender_memo` jest opcjonalne i nigdy nie trafia jawnie na chain.

## 7. Aux commitment

`aux_commit` w nocie powinien wiazac dane potrzebne do przyszlego proofa wydania.

Przyklad v0:

```rust
struct AuxWitness {
    version: u8,
    amount: Amount14,
    witness_seed: Hash32,
    noise_class: u8,
    bundle_id: BundleId,
}
```

Commitment:

```text
aux_commit = H_aux(canonical_aux_witness)
```

W `RecipientBoxPlaintext` nie trzeba przenosic calego witnessa LWE, jesli mozna go deterministycznie odtworzyc z `witness_seed`.

## 8. Note i note_commit

On-chain output:

```rust
struct OutputNote {
    note_commit: Hash32,
    spend_policy_commit: Hash32,
    ct_amt: LweCiphertext,
    aux_commit: Hash32,
    recipient_box: RecipientBox,
}
```

`note_commit` nie jest dowolne.
Wyliczamy je jako:

```text
note_commit = H_note(
    canonical(spend_policy_commit) ||
    canonical(ct_amt) ||
    canonical(aux_commit) ||
    canonical(recipient_box)
)
```

Zasada:

- `note_commit` zawsze hash'uje caly output bez pola `note_commit`.

## 9. Nullifier

`Nullifier` zuzywa note bez ujawniania adresu odbiorcy.

```rust
struct Nullifier(Hash32);
```

Definicja v0:

```text
nullifier = H_nullifier(note_commit || nullifier_key)
```

Wymagania:

- `nullifier_key` jest znany tylko spenderowi,
- dwa rozne noty nie powinny dawac tego samego nullifiera,
- ten sam note zawsze daje ten sam nullifier.

## 10. Public statement binding

Kazda transakcja musi miec publiczny binding do statementu proofa.

```text
statement_commit = H_stmt(
    tx_version ||
    all_input_note_commits ||
    all_input_nullifiers ||
    all_output_note_commits ||
    fee ||
    tx_type
)
```

Dzieki temu:

- proof moze byc inline albo batch,
- tx i proof sa zwiazane jednoznacznie,
- blok moze zawierac jeden proof dla wielu tx, jesli kazda tx ma poprawny `statement_commit`.

## 11. TransferNoteTx

v0 zamraza taki publiczny format transakcji:

```rust
struct TransferNoteTx {
    version: u8,
    tx_type: u8,
    inputs: Vec<InputRef>,
    input_nullifiers: Vec<Nullifier>,
    outputs: Vec<OutputNote>,
    fee: u64,
    statement_commit: Hash32,
    auth: Vec<InputAuth>,
}

struct InputRef {
    note_commit: Hash32,
}

struct InputAuth {
    policy_tag: u8,
    signer_pks: Vec<Vec<u8>>,
    signatures: Vec<Vec<u8>>,
}
```

Wartosci:

```text
version = 0x00
tx_type = 0x01  // TransferNote
```

Zasady walidacji:

- `inputs.len() == input_nullifiers.len() == auth.len()` dla v0,
- kazdy input musi istniec i nie byc juz zuzyty,
- kazdy `InputAuth` musi pasowac do polityki input note,
- `statement_commit` musi odpowiadac polom transakcji,
- proof zwiazany z `statement_commit` musi byc wazny,
- conservation i range checks sa w proofie,
- `nullifier` uniqueness jest sprawdzana przez ledger.

## 12. Escrow notes

Escrow nie jest osobnym systemem formatow.
To tylko specjalny `SpendPolicy`.

Przykladowe zastosowanie:

- buyer tworzy output note z `Escrow2of3`,
- release wymaga podpisow buyer+seller albo seller+arbiter,
- po `timeout_block` refund path moze byc odblokowany przez odpowiednia polityke i proof logic.

Mozliwe `tx_type` v0:

```text
0x01 = TransferNote
0x02 = EscrowOpen
0x03 = EscrowResolve
0x04 = Rebalance
0x05 = ModelRegister
0x06 = StakeDeposit
0x07 = StakeWithdraw
```

## 13. Canonical hashing of Falcon keys

Aby nie nosic stale calych kluczy w politykach, hashujemy publiczne klucze Falcon tak:

```text
falcon_pk_hash = BLAKE3("privai:falcon-pk:v0" || pk_bytes)
```

Ten hash trafia do `SpendPolicy`.
Pelne klucze publiczne pojawiaja sie w `InputAuth`, gdy trzeba zweryfikowac podpis.

## 14. PRIVAI/1 over NXMS

`nxms-transport` ma byc rozszerzony z `ESCROW/1` do `PRIVAI/1`.

Zmiany logiczne:

- `app_proto = "PRIVAI/1"`
- `escrow_id_hex` staje sie ogolnym `context_id_hex`
- `msg_type` rozszerzamy poza escrow

### 14.1. Envelope rules

W v0 zachowujemy obecny model NXMS:

- `from`
- `to`
- `seq`
- PQ encryption/authentication
- anti-replay po `(context_id, from, seq)`

### 14.2. Message types

```rust
enum PrivAiMsgType {
    BundleOffer,
    BundleRequest,
    BundleDelivery,
    WitnessUpdate,
    MarketOffer,
    MarketAccept,
    InferenceRequest,
    InferenceResponse,
    EscrowOpen,
    EscrowResolve,
    ProofServiceRequest,
    ProofServiceResponse,
    Error,
}
```

## 15. PRIVAI/1 payload bodies

### 15.1. BundleOffer

```rust
struct BundleOfferBody {
    bundle_id: BundleId,
    bundle_commit: Hash32,
    expires_at: u64,
    route_hint: Option<Vec<u8>>,
}
```

### 15.2. BundleRequest

```rust
struct BundleRequestBody {
    requested_count: u16,
    min_expiry: u64,
}
```

### 15.3. BundleDelivery

```rust
struct BundleDeliveryBody {
    bundle_id: BundleId,
    receive_bundle_bytes: Vec<u8>,
}
```

### 15.4. WitnessUpdate

`WitnessUpdate` sluzy do pomocniczej synchronizacji odbiorcy po transferze.
Fundusze nie zaleza od tej wiadomosci, ale wallet moze szybciej zaktualizowac lokalny stan.

```rust
struct WitnessUpdateBody {
    note_commit: Hash32,
    bundle_id: BundleId,
    recipient_box_mirror: Vec<u8>,
    sender_hint: Option<Vec<u8>>,
}
```

### 15.5. MarketOffer

```rust
struct MarketOfferBody {
    model_id: Hash32,
    operator_id: Hash32,
    price_model: Vec<u8>,
    settlement_policy: u8,
    metadata_box: Option<Vec<u8>>,
}
```

### 15.6. MarketAccept

```rust
struct MarketAcceptBody {
    model_id: Hash32,
    session_context: ContextId,
    payment_bundle_id: BundleId,
    escrow_required: bool,
}
```

### 15.7. InferenceRequest

```rust
struct InferenceRequestBody {
    session_context: ContextId,
    model_id: Hash32,
    prompt_box: Vec<u8>,
    payment_note_commit: Option<Hash32>,
}
```

### 15.8. InferenceResponse

```rust
struct InferenceResponseBody {
    session_context: ContextId,
    response_box: Vec<u8>,
    settle_hint: Option<Vec<u8>>,
}
```

### 15.9. ProofServiceRequest

```rust
struct ProofServiceRequestBody {
    statement_commit: Hash32,
    tx_refs: Vec<Hash32>,
    witness_box: Vec<u8>,
}
```

### 15.10. ProofServiceResponse

```rust
struct ProofServiceResponseBody {
    statement_commit: Hash32,
    proof_bytes: Vec<u8>,
    verifier_hint: Option<Vec<u8>>,
}
```

### 15.11. Error

```rust
struct PrivAiErrorBody {
    context_id: ContextId,
    code: String,
    reason: String,
}
```

## 16. What is on-chain vs off-chain

On-chain:

- `OutputNote`
- `Nullifier`
- `TransferNoteTx`
- stake/model registration data
- escrow settlement tx

Off-chain przez NXMS:

- bundle exchange
- witness updates
- prompt/response payloads
- proof service witness transport
- marketplace negotiation

On-chain encrypted but public carrier:

- `RecipientBox`

## 17. First implementation targets

Najblizszy krok implementacyjny po tym specu:

1. `privai-crypto`: typy `Hash32`, `LweCiphertext`, hash helpers
2. `privai-wallet`: `ReceiveBundle`, `RecipientBox`, `Nullifier` derivation
3. `privai-ledger`: `OutputNote`, `TransferNoteTx`, canonical serialization
4. `privai-nxms`: `PRIVAI/1` payload enum i `context_id`
5. `privai-proof`: `statement_commit` binding i witness schema

## 18. Freeze points

Na potrzeby v0 zamrazamy juz teraz:

- `q = 2^32 - 5`
- `p = 2^14`
- note-based ledger
- `ReceiveBundle` jako hidden address primitive
- `Nullifier = H_nullifier(note_commit || nullifier_key)`
- `RecipientBox` jako on-chain encrypted delivery
- `statement_commit` jako binding tx-proof
- `PRIVAI/1` jako generalizacja NXMS

To wystarczy, zeby przejsc do scaffoldingu crates i pierwszych struktur kodu.