# privAI v0 Protocol Skeleton

Status: legacy protocol reference in migration.
Canonicality: non-canonical. Ten dokument nie moze nadpisywac `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`.
Owner: privAI protocol.
Depends on: `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`.
Superseded by: planowany `spec/PRIVAI_PROTOCOL_CORE.md` oraz `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA.md`.

## 1. Cel

`privAI` ma byc privacy blockchainem z natywnym coinem do rozliczen w marketplace lokalnych modeli AI.

System ma laczyc:

- ukryte adresy odbiorcow,
- prywatne kwoty oparte o LWE,
- autoryzacje i escrow oparte o Falcon,
- prywatny transport i store-and-forward przez NXMS,
- threshold operacje sieciowe przez Shamir,
- lookup-based proofs do sprawdzania poprawnosci przejsc stanu.

Projekt nie jest wrapperem na inny chain. Budujemy wlasny stos pod wlasne komponenty.

Ekonomia i produktowy model platnosci dla v0 sa opisane osobno w:

- `PRIVAI_V0_PAYMENTS_AND_ECONOMICS.md`

W szczegolnosci dokument ekonomiczny rozroznia:

- `FullPrivacy` dla pelnych prywatnych settlementow,
- `RecipientPrivacyLite` jako opcjonalny tryb dla drobnych platnosci
  (`ukryty adres + jawna kwota`).

Robocza zasada v0:

- drobnica domyslnie idzie przez depozyt / tab / batch settlement,
- male platnosci on-chain moga uzyc `RecipientPrivacyLite`,
- escrow i settlement o wyzszej wartosci powinny uzywac `FullPrivacy`.

## 2. Zasady architektoniczne

Kazdy komponent ma jedna glowna role:

- `LWE` ukrywa kwoty.
- `ReceiveBundle` ukrywa adres odbiorcy.
- `Falcon` autoryzuje wydanie i realizuje multisig escrow.
- `NXMS` przenosi prywatne wiadomosci, witness updates i komunikacje marketplace.
- `Shamir` obsluguje wybrane operacje threshold na kluczu sieciowym.
- `Lookup proofs` udowadniaja poprawne przejscia stanu bez ujawniania danych.

Zasada nadrzedna:

- nie uzywamy `account-based ledger` jako glownego modelu prywatnosci,
- przechodzimy na `note/UTXO`,
- kazdy output ma swiezy ciphertext LWE,
- nie aktualizujemy w kolko jednego publicznego konta.

## 3. Parametry v0

Parametry bazowe dla kwot prywatnych:

```text
q      = 2^32 - 5 = 4294967291   (prime)
p      = 2^14    = 16384         (plaintext amount space)
sigma  = 8
Delta  = floor(q / p) = 262143
B_max  = floor(Delta / 2) = 131071
n      = 1024
```

Wnioski:

- `p = 2^14` daje sensowny margines dekodowania.
- Kwoty on-chain zyja w `Z_p`.
- Dla wiekszych kwot stosujemy denominacje, np. `1 privAI = 100` lub `1000` jednostek bazowych.

## 4. Model ledgera

privAI v0 uzywa modelu `note/UTXO`.

Kazdy output to prywatna nota:

```rust
struct Note {
    note_commit: [u8; 32],
    spend_policy: SpendPolicyCommit,
    ct_amt: LweCiphertext,
    aux_commit: [u8; 32],
    recipient_box: RecipientBox,
}
```

Znaczenie pol:

- `note_commit`:
  commitment do pelnej noty; sluzy jako glowny identyfikator outputu.
- `spend_policy`:
  commitment do warunku wydania; single-spend albo escrow 2-of-3.
- `ct_amt`:
  `Enc(pk_net, amount)` z fresh noise.
- `aux_commit`:
  commitment do sekretnego witnessa potrzebnego do przyszlego wydania.
- `recipient_box`:
  zaszyfrowany pakiet dla odbiorcy, zawierajacy dane do odczytu i wydania noty.

## 5. Ukryte adresy

Ukryte adresy nie beda realizowane przez stale publiczne `AccountID`.
Odbiorca publikuje lub udostepnia pule jednorazowych bundle'i odbiorczych.

```rust
struct ReceiveBundle {
    bundle_id: [u8; 16],
    one_time_falcon_pk: Vec<u8>,
    one_time_falcon_sk_ref: SecretRef,
    one_time_frodo_pk: Vec<u8>,
    one_time_frodo_sk_ref: SecretRef,
    expires_at: u64,
    flags: u8,
}
```

Flow:

1. Wallet odbiorcy generuje wiele `ReceiveBundle`.
2. Bundle moze byc opublikowany w katalogu marketplace, wyslany prywatnie albo pobrany z warstwy relay.
3. Nadawca wybiera nieuzyty bundle.
4. Nadawca tworzy note pod `one_time_falcon_pk`.
5. Nadawca pakuje sekretne dane do `recipient_box`, szyfrowanego pod `one_time_frodo_pk`.
6. Odbiorca rozpoznaje note po mozliwosci otwarcia `recipient_box`.
7. Chain nie ujawnia stalej tozsamosci odbiorcy.

W praktyce:

- `prekeys` z `nexum-cli` sa naturalnym seedem dla `ReceiveBundle`.
- Bundle powinien byc jednorazowy.
- Po uzyciu bundle przechodzi w stan `used`.

## 6. SpendPolicy

Warunek wydania jest jawnie modelowany.

```rust
enum SpendPolicy {
    Single {
        falcon_pk_hash: [u8; 32],
    },
    Escrow2of3 {
        buyer_pk_hash: [u8; 32],
        seller_pk_hash: [u8; 32],
        arbiter_pk_hash: [u8; 32],
        timeout_block: u64,
    },
}
```

Wnioski:

- zwykle transfery korzystaja z `Single`,
- escrow w marketplace korzysta z `Escrow2of3`,
- nie tworzymy osobnego systemu escrow poza ledgerem,
- escrow to tylko specjalny typ polityki wydania noty.

## 7. RecipientBox

Srodki nie moga zalezec od dzialania mailboxa.
Dlatego dane potrzebne do odbioru musza byc zaszyte w samej nocie.

```rust
struct RecipientBox {
    kem_id: String,
    kem_ct: Vec<u8>,
    ciphertext: Vec<u8>,
    tag: Vec<u8>,
    hint: [u8; 16],
}
```

Przykladowa zawartosc plaintextu `recipient_box`:

```rust
struct RecipientBoxPlaintext {
    note_commit: [u8; 32],
    amount_hint_commit: [u8; 32],
    witness_seed: Vec<u8>,
    spend_policy_opening: Vec<u8>,
    sender_memo: Option<Vec<u8>>,
}
```

`RecipientBox` sluzy do:

- odzyskania witnessa odbiorczego,
- powiazania noty z wlasciwa polityka wydania,
- dostarczenia danych do przyszlego dowodu wydania.

NXMS moze wysylac kopie tych danych pomocniczo, ale odbior nie moze od niego zalezec.

## 8. Nullifiers

Aby zapobiec double-spend, kazda wydawana nota produkuje `nullifier`.

```rust
struct Nullifier([u8; 32]);
```

Wymagania:

- `nullifier` musi byc deterministycznie zwiazany z nota i sekretami spendowymi,
- nie moze ujawniac publicznie tozsamosci odbiorcy,
- musi byc sprawdzalny przez full node podczas walidacji wydania.

Przykladowy kierunek:

```text
nullifier = H(note_commit || spend_secret || domain_sep)
```

Ledger trzyma globalny zbior wykorzystanych `nullifier`.

## 9. Typy transakcji v0

Minimalny zestaw:

```rust
enum Tx {
    RegisterReceiverBundle(RegisterReceiverBundleTx),
    TransferNote(TransferNoteTx),
    SpendNote(SpendNoteTx),
    EscrowOpen(EscrowOpenTx),
    EscrowResolve(EscrowResolveTx),
    Rebalance(RebalanceTx),
    ModelRegister(ModelRegisterTx),
    ModelUpdate(ModelUpdateTx),
    StakeDeposit(StakeDepositTx),
    StakeWithdraw(StakeWithdrawTx),
}
```

Uwagi:

- `RegisterReceiverBundleTx` moze byc opcjonalny, jesli bundle sa dystrybuowane off-chain.
- `TransferNoteTx` tworzy nowe noty.
- `SpendNoteTx` wydaje istniejace noty i produkuje nowe.
- `EscrowOpenTx` oraz `EscrowResolveTx` to specjalizacje nad `SpendPolicy`.

## 10. TransferNoteTx

Podstawowa transakcja platnicza.

```rust
struct TransferNoteTx {
    inputs: Vec<InputRef>,
    input_nullifiers: Vec<Nullifier>,
    outputs: Vec<OutputNote>,
    fee: u64,
    proof_ref: ProofRef,
    auth: TxAuth,
}

struct InputRef {
    note_commit: [u8; 32],
}

struct OutputNote {
    note_commit: [u8; 32],
    spend_policy_commit: [u8; 32],
    ct_amt: LweCiphertext,
    aux_commit: [u8; 32],
    recipient_box: RecipientBox,
}
```

Warunki poprawnosci:

- wszystkie `input_nullifiers` sa nowe,
- input notes istnieja,
- `sum(inputs) = sum(outputs) + fee` w przestrzeni plaintextow,
- wszystkie outputy maja poprawne fresh ciphertexts,
- polityki wydania sa poprawnie zwiazane z outputami,
- sygnatury Falcon odpowiadaja wymaganym politykom wydania inputow.

## 11. Auth warstwa transakcji

privAI nie rozdziela sztucznie autoryzacji i prywatnosci.
Kazdy spend ma:

- jawna czesc identyfikujaca wymagany typ podpisu,
- prywatny proof poprawnosci kwot i witnessa,
- podpis(y) Falcon wymagane przez `SpendPolicy`.

```rust
enum TxAuth {
    Single {
        falcon_pk: Vec<u8>,
        falcon_sig: Vec<u8>,
    },
    Escrow2of3 {
        signers: Vec<Vec<u8>>,
        signatures: Vec<Vec<u8>>,
    },
}
```

## 12. LWE amounts

Kwoty w nowych outputach sa zawsze swieze.

Zamiast modelu:

- `ct_old + delta -> ct_new` przez dlugi czas na tym samym koncie,

przyjmujemy model:

- input note jest konsumowany,
- output note dostaje nowy `ct_amt = Enc(pk_net, amount)` z fresh noise.

To daje:

- prostszy hot path,
- mniejsza presje na rebalancing,
- lepsza zgodnosc z ukrytymi adresami.

Wniosek praktyczny:

- noise budget nadal istnieje,
- ale jest konsumowany glownie podczas tworzenia i wydawania not,
- nie przez nieskonczone aktualizacje tego samego stanu konta.

## 13. Lookup proofs

v0 zaklada lookup-based proofs w warstwie `privai-proof`.

Lookupi sluza do:

- range-checkow kwot,
- range-checkow malych noise terms,
- limb decomposition elementow mod `q`,
- sprawdzania transition rules,
- walidacji relacji miedzy witnessami i publicznymi commitmentami.

Minimalny zestaw tablic:

```text
T1   = {0,1}
T2   = {0..3}
T6   = {0..63}
T8   = {0..255}
T97  = {0..96}
```

Przeznaczenie:

- `T1`: bity
- `T6 + T8`: dekompozycje 14-bit
- `T2 + T8 + T8`: dekompozycje signed 18-bit
- `T97`: mali reprezentanci dla swiezego noise

Walidacja proofa ma potwierdzic:

- poprawnosc input witnesses,
- poprawnosc output witnesses,
- conservation of value,
- poprawnosc nullifierow,
- poprawnosc powiazania `recipient_box` z outputem,
- zgodnosc z `SpendPolicy`.

## 14. Shamir i klucz sieciowy

Shamir nie dotyka zwyklego hot path transakcji.

Jego zakres w v0:

- `DKG` dla `pk_net`,
- `threshold ops` na `sk_net`,
- `proactive reshare`,
- `emergency migration / recovery`,
- opcjonalny `rebalance flow` dla szczegolnych operacji systemowych.

Nie uzywamy Shamira do:

- zwyklego escrow,
- podpisow uzytkownika,
- stalej obslugi wszystkich zwyklych transferow.

## 15. Rebalancing

W modelu note-based rebalancing nie jest glownym mechanizmem codziennej obslugi kont.
Mimo to zostawiamy go jako mechanizm systemowy.

Przyklad zastosowan:

- migracja starych outputow,
- reencrypt / refresh podczas epoki,
- specjalne systemowe operacje naprawcze,
- cold-path dla dlugo zyjacych obiektow specjalnych.

Jesli jest wymagane odswiezenie noty przez siec:

1. Uzytkownik tworzy masked request.
2. Walidatorzy wykonuje threshold operation na `sk_net`.
3. Tworzony jest nowy fresh ciphertext.
4. Uzytkownik otrzymuje nowy sekret wydaniowy.

To jest cold path, nie standardowy flow platnosci.

## 16. NXMS rola w privAI

NXMS zostaje w systemie jako warstwa prywatnej komunikacji.

Zastosowania:

- negocjacja marketplace,
- escrow coordination,
- dispute flow,
- dostarczanie witness updates,
- wymiana prywatnych metadanych transakcyjnych,
- komunikacja wallet-node,
- komunikacja prover-service.

Wymagana generalizacja:

- obecne `ESCROW/1` rozszerzamy do `PRIVAI/1`,
- `msg_type` rozszerzamy poza escrow,
- `escrow_id_hex` zamieniamy logicznie na bardziej ogolny `context_id_hex`.

Przykladowe typy:

```rust
enum PrivAiMsgType {
    MarketOffer,
    InferenceRequest,
    InferenceResponse,
    WitnessUpdate,
    BundleDelivery,
    EscrowOpen,
    EscrowResolve,
    ProofServiceRequest,
    ProofServiceResponse,
    Error,
}
```

## 17. Marketplace local AI models

privAI istnieje po to, zeby obslugiwac rozliczenia i zaufanie w marketplace lokalnych modeli AI.

Model dzialania:

1. Operator publikuje oferte modelu.
2. Klient nawiazuje prywatny kanal przez NXMS.
3. Warunki rozliczenia sa ustalane off-chain.
4. Zabezpieczenie srodkow moze nastapic przez escrow note.
5. Inferencja odbywa sie off-chain.
6. Settlement i dispute sa realizowane on-chain przez `privAI`.

On-chain:

- stake operatora,
- rejestr modelu,
- escrow open/resolve,
- transfery `privAI`,
- reputacyjne lub ekonomiczne konsekwencje naduzyc.

Off-chain:

- prompty,
- odpowiedzi modelu,
- negocjacje warunkow,
- transport duzych danych.

## 18. PoPG jako warstwa uslugowa

PoPG ma sens jako warstwa uslugowa dla proverow i walidatorow, ale nie blokuje v0.

v0:

- user buduje intent,
- prover generuje lookup proof,
- chain weryfikuje proof.

v1+:

- proverzy konkurencyjnie generuja proofy,
- powstaje rynek proof-service,
- walidatorzy moga realizowac czesc tej pracy jako useful work.

Wniosek:

- `PoPG` to naturalna ewolucja,
- ale nie musi byc warunkiem uruchomienia pierwszego testnetu.

## 19. Mapa reuse z istniejacych komponentow

### `nexum-cli`

Do reuse:

- `auth.*` jako baza challenge/authentication
- `pqc_falcon.*` jako podpisy Falcon
- `pqc_kem.*` jako wrapper FrodoKEM
- `vault.*` jako bezpieczne przechowywanie kluczy
- `prekeys.*` jako seed dla `ReceiveBundle`
- `pow.*` jako baza anty-spam / validator admission

Do odciecia od rdzenia:

- logika specyficzna dla poprzedniego use-case,
- wszystko, co bylo klejem do zewnetrznego multisig workflow.

### `nxms-transport`

Do reuse:

- wire format,
- PQ envelope,
- seq anti-replay,
- transport helpers,
- peer abstractions.

Wymagane zmiany:

- rozszerzenie `MsgType`,
- nowy `app_proto = PRIVAI/1`,
- bardziej ogolny `context_id`.

### `nxms-mailbox`

Do reuse:

- store-and-forward,
- leased pull/ack,
- inbox scoping,
- relay przez onion service.

Rola w privAI:

- delivery prywatnych wiadomosci,
- nie storage of funds,
- nie zrodlo prawdy dla ledgera.

## 20. Proponowany podzial crates

```text
privai-crypto/
  falcon wrappers
  frodo wrappers
  hash / kdf helpers

privai-wallet/
  vault
  receive bundles
  note scanning
  recipient box open
  tx builder

privai-nxms/
  PRIVAI/1 wire
  mailbox client integration
  peer session logic

privai-ledger/
  notes
  nullifiers
  transactions
  blocks
  storage

privai-proof/
  lookup tables
  proof circuits
  witness formats
  verifier integration

privai-threshold/
  dkg
  shamir shares
  threshold ops
  resharing

privai-market/
  model registry
  offer metadata
  settlement policies
  escrow templates

privai-node/
  mempool
  validation
  block production
  p2p sync
```

## 21. Minimalny milestone v0

Pierwszy dzialajacy milestone nie musi miec wszystkiego.

Zakres minimalny:

1. generowanie `ReceiveBundle`
2. tworzenie note outputow z `recipient_box`
3. skanowanie chaina przez wallet
4. wydawanie not przez Falcon auth
5. nullifier set
6. lookup proof dla conservation i range checks
7. prosty escrow 2-of-3 jako `SpendPolicy`
8. NXMS do negocjacji i pomocniczego witness sync

To juz daje:

- prywatny coin,
- ukrytych odbiorcow,
- prywatne kwoty,
- escrow dla marketplace,
- naturalna baze pod dalszy rozwoj.

## 22. Otwarte decyzje do zamrozenia

Najblizsze decyzje projektowe:

1. Czy `ReceiveBundle` sa publikowane on-chain, przez relay, czy hybrydowo.
2. Jak dokladnie definiujemy `nullifier`.
3. Czy proof jest per-tx czy per-block batch.
4. Jak wyglada canonical encoding dla `recipient_box`.
5. Jak wyglada model oplat i denominacja `privAI`.
6. Jakie metadane modelu AI trafiaja on-chain, a jakie zostaja w NXMS.

## 23. Konkluzja

privAI v0 powinno byc zbudowane jako:

- note-based privacy coin,
- z hidden addresses przez `ReceiveBundle`,
- z prywatnymi kwotami przez LWE,
- z autoryzacja i escrow przez Falcon,
- z transportem i prywatna komunikacja przez NXMS,
- z operacjami sieciowymi przez Shamir,
- z lookup-based proof system dla walidacji przejsc stanu.

To jest spojna architektura, zgodna z tym, co juz istnieje w workspace, i daje realna droge od komponentow `nexum-cli` do wlasnego blockchaina dla marketplace lokalnych modeli AI.
