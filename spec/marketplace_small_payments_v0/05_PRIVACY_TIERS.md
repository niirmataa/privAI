# Marketplace Small Payments v0

## Privacy Tiers

## 1. Cel

Ten dokument ma zamknac formalny kontrakt privacy-tierow dla `privAI v0`, tak zeby:

- wallet wiedzial, jaki rail wolno wybrac,
- merchant nie probowal wciskac lekkiego raila tam, gdzie nie wolno,
- marketplace operator wiedzial, kiedy moze uzyc `SmallPaymentsRail`,
- proof i chain layer mialy jasna granice miedzy:
  - `RecipientPrivacyLite`
  - `SmallPaymentsRail`
  - `FullPrivacy`

To nie ma byc opis marketingowy.
To ma byc twarda macierz decyzyjna dla implementacji i review.

## 2. Szybkie podsumowanie dla Gemini

Jesli masz zapamietac tylko jedno:

1. `FullPrivacy` jest wymagane dla:
   - escrow,
   - dispute-sensitive flows,
   - duzych kwot,
   - payoutow wrazliwych,
   - przypadkow, gdzie sama kwota jest wrazliwa.
2. `SmallPaymentsRail` jest tylko dla marketplace:
   - prywatny depozyt
   - `SessionGrant/SpendGrant`
   - `ticket_nullifier`
   - receipts
   - settlement przez operatora
3. `RecipientPrivacyLite` to lekki rail:
   - ukryty odbiorca / brak stalego publicznego konta platnika
   - jawna kwota
   - bez pelnego hidden-amount proof
4. Nie wolno:
   - oslabiac PQ dla drobnicy,
   - uzywac `SmallPaymentsRail` poza marketplace,
   - wciskac `RecipientPrivacyLite` do escrow lub sporow o duzej stawce.

## 3. Co jest juz zamrozone i nie podlega renegocjacji w tym dokumencie

1. `privAI v0` nie jest rail'em do pelnej prywatnej mikropatnosci on-chain per event.
2. Dla marketplace drobnicy domyslny model to:
   - `private deposit -> off-chain service state -> batch settlement`
3. `ticket` jest `strictly one-time`.
4. Marketplace v0 uzywa `marketplace-operator-trusted accounting`.
5. `purchase_commit` pozostaje wymagany dla lekkiego raila marketplace.
6. Nie wolno wprowadzac slabszej warstwy PQ dla drobnicy.
7. Zwykla drobnica poza marketplace nie uzywa marketplace ticketow ani grantow.

## 4. Czego ten dokument ma nie robic

Ten dokument nie ma:

- projektowac nowego raila od zera,
- zastapic `ServicePaymentPolicy`,
- zastapic dokumentu ticketow,
- zastapic dokumentu receipts/settlementu,
- rozmywac juz zamrozonych trust assumptions.

Ten dokument ma zamknac tylko:

- nazwy i semantyke tierow,
- kiedy jaki tier wolno uzyc,
- kto podejmuje decyzje o wyborze tieru,
- kiedy trzeba eskalowac do `FullPrivacy`,
- jak to komunikowac walletowi i polityce uslugi.

## 5. Twardy problem do rozwiazania

Musimy umiec odpowiedziec:

- jaki rail jest domyslny dla danego flow,
- kto moze obnizyc prywatnosc,
- kto moze wymusic wyzszy tier,
- kiedy marketplace moze uzyc lekkiego raila,
- kiedy zwykla platnosc moze uzyc tylko `RecipientPrivacyLite`,
- kiedy system ma powiedziec "nie, tu musi wejsc `FullPrivacy`".

Po tej iteracji ma byc jasne:

- jakie mamy privacy tiery,
- jaka jest ich semantyka,
- jaka jest macierz wyboru,
- jaka jest logika eskalacji.

## 6. RecipientPrivacyLite — Concrete Data Model (v0.1)

### 6.0. Cel tej sekcji

Ta sekcja zamraża konkretny format `RecipientPrivacyLite` jako lekkiego raila on-chain z maksymalną prywatnością stealth address.

### 6.0.1. Zasada nadrzędna

`RecipientPrivacyLite` jest zaprojektowany tak, żeby:
- **stealth address** (RecipientBox) pozostał w pełni prywatny,
- **kwota** była jawna (świadomy kompromis na rzecz lekkości),
- **on-chain payload** był minimalny (~43% mniejszy niż FullPrivacy),
- **proof** był uproszczony (brak LWE range proof).

### 6.0.2. LiteOutputNote

Nowy typ outputu dla RecipientPrivacyLite:

```rust
struct LiteOutputNote {
    version: u8,                    // 0x00
    note_commit: Hash32,            // Commitment do całej noty
    amount: u64,                    // JAWNA kwota (zamiast LWE ciphertext)
    spend_policy_commit: Hash32,    // Commitment do SpendPolicy
    aux_commit: Hash32,             // Commitment do AuxWitness
    recipient_box: RecipientBox,    // UKRYTY odbiorca (stealth address)
}
```

**Co znika wobec FullPrivacy OutputNote:**
- `ct_amt: LweCiphertext` (~4100 B) → `amount: u64` (8 B)
- **Oszczędność: ~4092 B na output**

**Co zostaje (prywatność):**
- `RecipientBox` — ukryty odbiorca, stealth address, pełna prywatność relacji
- `spend_policy_commit` — warunki wydania
- `aux_commit` — witness material
- `note_commit` — commitment do całej struktury

### 6.0.3. Budowa `note_commit` dla Lite

```text
note_commit = H_note(
    version            ||   // 1 bajt
    spend_policy_commit ||   // 32 bajty
    amount             ||   // 8 bajtów (jawna kwota)
    aux_commit         ||   // 32 bajty
    recipient_box_hint      // 16 bajtów hint z RecipientBox (nie cały box!)
)
```

**Ważne:** `note_commit` NIE zawiera całego `RecipientBox` (6000+ B), tylko `hint` (16 B). Pełny box jest przechowywany off-chain/encrypted. Chain widzi hint do szybkiego skanowania, ale nie może odtworzyć stealth address bez klucza odbiorcy.

### 6.0.4. LiteTransferTx

Nowy typ transakcji:

```rust
struct LiteTransferTx {
    core: TxCore,                  // Standardowy TxCore
}
```

`tx_type = 0x08` dla `LiteTransferTx`.

### 6.0.5. Conservation check (jawny)

W odróżnieniu od FullPrivacy (gdzie conservation jest dowodzona w ZK na ukrytych kwotach), tutaj:

```text
sum(input_amounts) == sum(output_amounts) + fee
```

To jest jawna arytmetyka — prosta, tania, nie wymaga LWE range proof.

### 6.0.6. Proof dla RecipientPrivacyLite

Uproszczony proof obejmuje:
1. **Conservation**: jawna arytmetyka
2. **Nullifier correctness**: `nullifier = H_nullifier(note_commit || nullifier_key)`
3. **Policy binding**: output spend_policy_commit jest poprawnie związany
4. **RecipientBox binding**: box jest poprawnie zaszyfrowany
5. **Input authorization**: sygnatury Falcon pasują do SpendPolicy

**Brak w odróżnieniu od FullPrivacy:**
- LWE range proof na ukrytą kwotę
- Noise class check
- Well-formedness proof na ciphertext

### 6.0.7. Rozmiar porównawczy

| Komponent | FullPrivacy | RecipientPrivacyLite | Oszczędność |
|-----------|-------------|---------------------|-------------|
| Output | ~10600 B | ~6500 B | ~4100 B (39%) |
| Proof per tx (2in/2out) | ~5000 B | ~2000 B | ~3000 B (60%) |
| **Razem tx** | **~26200 B** | **~15000 B** | **~11200 B (43%)** |

### 6.0.8. Zamrożone decyzje

1. `RecipientPrivacyLite` jest osobnym typem tx (`tx_type = 0x08`)
2. `amount` jest jawny na chainie
3. `RecipientBox` zostaje w pełni (stealth address)
4. `note_commit` używa hint zamiast całego RecipientBox
5. Proof jest uproszczony (brak LWE)
6. Conservation jest jawna arytmetyka

---

## 7. Definicje robocze

### 6.1. ServicePrivacy

`ServicePrivacy` to prywatnosc relacji uslugowej:

- co user kupuje,
- jak wyglada sesja,
- jakie receipts powstaja,
- jak wyglada usage flow.

### 6.2. RecipientPrivacy

`RecipientPrivacy` to ochrona relacji kto-komu placi.

### 6.3. AmountPrivacy

`AmountPrivacy` to ochrona samej kwoty.

### 6.4. FullPrivacy

`FullPrivacy` to ciezszy rail, ktory ma chronic co najmniej:

- recipient/privacy relation,
- amount privacy,
- wrazliwe settlement semantics.

To jest domyslny tier dla flow wrazliwych i wysokiego ryzyka.

### 6.5. RecipientPrivacyLite

`RecipientPrivacyLite` to lekki rail z:

- ukrytym odbiorca albo ukryta relacja platnicza (RecipientBox / stealth address — pełna prywatność relacji),
- jawna kwota (świadomy kompromis),
- prostszym i tanszym modelem on-chain (brak LWE ciphertext, uproszczony proof).

To nie jest `FullPrivacy`.

Konkretny format jest opisany w sekcji 6.0 powyżej.

### 6.6. SmallPaymentsRail

`SmallPaymentsRail` to marketplace-only rail:

- prywatny depozyt,
- `SessionGrant/SpendGrant`,
- `ticket_nullifier`,
- merchant/session receipts,
- batch settlement publikowany przez operatora.

To nie jest rail ogolnego przeznaczenia dla calego chaina.

## 7. Domyslne decyzje robocze do przyjecia, jesli nie ma lepszej kontrpropozycji

### D1. Mamy trzy glowne tiery v0

Domyslna decyzja:

- `RecipientPrivacyLite`
- `SmallPaymentsRail`
- `FullPrivacy`

Nie wprowadzamy dodatkowych "polowek" bez bardzo mocnego uzasadnienia.

### D2. `SmallPaymentsRail` jest tylko dla marketplace

Domyslna decyzja:

- poza marketplace nie ma `SessionGrant`, `SpendGrant`, ticketow i operatorskiego settlementu,
- zwykle male platnosci poza marketplace uzywaja co najwyzej `RecipientPrivacyLite`,
- nie kopiujemy ticketowego modelu do publicznego raila bez operatora.

### D3. `RecipientPrivacyLite` nie zastepuje `FullPrivacy`

Domyslna decyzja:

- jawna kwota to swiadomy kompromis,
- jesli amount privacy jest istotna, trzeba wejsc w `FullPrivacy`.

### D4. `FullPrivacy` jest wymagane dla flow wrazliwych

Domyslna decyzja:

`FullPrivacy` jest wymagane co najmniej dla:

- escrow,
- dispute-sensitive flows,
- duzych kwot,
- payoutow wrazliwych,
- flow, gdzie sama kwota jest dana wrazliwa,
- flow, ktore nie mieszcza sie w scoped `SpendGrant`.

### D5. Wallet nie moze byc biernym UI

Domyslna decyzja:

- wallet musi aktywnie egzekwowac privacy tiers,
- wallet musi umiec odmowic lekkiego raila,
- wallet nie moze tylko wyswietlac ostrzezenia.

### D6. Service policy moze zawezac, ale nie rozluznia globalnych wymogow

Domyslna decyzja:

- `ServicePaymentPolicy` moze powiedziec:
  - ta usluga wymaga `FullPrivacy`
  - ta usluga dopuszcza `SmallPaymentsRail`
  - ta usluga dopuszcza `RecipientPrivacyLite`
- ale nie moze powiedziec:
  - escrow jednak poleci lekkim rail'em,
  - wrazliwy payout jednak poleci `RecipientPrivacyLite`.

### D7. Operator marketplace nie moze sam obnizac tieru wzgledem polityki uslugi

Domyslna decyzja:

- operator moze zarzadzac grantami i settlementem,
- ale nie moze samowolnie zmieniac flow wymagajacego `FullPrivacy` w lekki rail.

## 8. Wymagana macierz wyboru tieru

Masz pracowac z nastepujacym szkicem:

```text
Flow characteristics
  -> allowed tier(s)
  -> default tier
  -> escalation trigger
  -> authority deciding final tier
```

Musisz ocenic co najmniej te klasy flow:

### 8.1. Marketplace small usage

Przyklad:

- AI inference session,
- token metering,
- male pakiety retail,
- subskrypcyjny usage u jednego merchanta.

Domyslny tier:

- `SmallPaymentsRail`

### 8.2. Small one-shot on-chain transfer poza marketplace

Przyklad:

- mala platnosc do odbiorcy,
- prosty transfer bez operatora marketplace.

Domyslny tier:

- `RecipientPrivacyLite`

### 8.3. Escrow flow

Domyslny tier:

- `FullPrivacy`

### 8.4. Dispute-heavy service flow

Domyslny tier:

- `FullPrivacy`

### 8.5. Payout / withdrawal-like flow wrazliwy

Domyslny tier:

- `FullPrivacy`

### 8.6. Amount-sensitive flow

Jesli kwota sama jest wrazliwa:

- `FullPrivacy`

## 9. Wymagany model decyzji walleta

Musisz opisac, jak wallet wybiera tier.

Na starcie przyjmij, ze wallet bierze pod uwage co najmniej:

- typ flow,
- `ServicePaymentPolicy`,
- czy flow jest marketplace-only,
- wartosc transakcji,
- czy amount privacy jest wymagana,
- czy jest escrow/dispute sensitivity,
- czy istnieje wazny `SpendGrant`,
- preferencje usera, jesli nie lamia twardej polityki.

Domyslny kierunek v0:

- wallet wybiera najlzejszy tier dozwolony przez polityke i bezpieczny dla flow,
- ale eskaluje do `FullPrivacy`, gdy tylko flow wpada w wrazliwa klase.

## 10. Wymagany model eskalacji

To jest sekcja krytyczna.

Masz odpowiedziec:

1. kiedy `RecipientPrivacyLite` musi eskalowac do `FullPrivacy`,
2. kiedy `SmallPaymentsRail` musi eskalowac do `FullPrivacy`,
3. czy `SmallPaymentsRail` moze fallbackowac do `RecipientPrivacyLite`,
4. czy `RecipientPrivacyLite` moze fallbackowac do `SmallPaymentsRail`,
5. kto moze wymusic eskalacje:
   - wallet
   - `ServicePaymentPolicy`
   - operator
   - merchant

Domyslny v0:

- `SmallPaymentsRail -> FullPrivacy` tak, gdy flow wychodzi poza policy / grant / sensitivity scope
- `RecipientPrivacyLite -> FullPrivacy` tak, gdy amount privacy lub dispute sensitivity staje sie wymagana
- `RecipientPrivacyLite <-> SmallPaymentsRail` nie sa automatycznymi zamiennikami, bo sluza innym swiatom:
  - public small transfer
  - marketplace rail

## 11. Wymagany model komunikacji i UX

Masz odpowiedziec:

1. jak wallet komunikuje userowi, ze jest w lekkim tierze,
2. jak wallet komunikuje jawna kwote,
3. jak wallet komunikuje wejscie w `FullPrivacy`,
4. czy user moze recznie podniesc tier,
5. czy user moze recznie obnizyc tier,
6. jak pokazywac, ze `SmallPaymentsRail` jest marketplace-only.

Domyslny v0:

- user moze recznie prosic o wyzszy tier,
- user nie moze recznie wymusic nizszego tieru, jesli policy lub flow wymagaja `FullPrivacy`.

## 12. Pytania, ktore musza dostac odpowiedz

Nie wolno zostawic tych pytan bez odpowiedzi:

1. Jakie privacy tiery istnieja w `privAI v0`?
2. Ktory tier jest domyslny dla marketplace drobnicy?
3. Ktory tier jest domyslny dla zwyklych malych platnosci poza marketplace?
4. Kiedy `FullPrivacy` jest wymagane bez dyskusji?
5. Czy `RecipientPrivacyLite` moze byc uzyty w escrow?
6. Czy `SmallPaymentsRail` moze byc uzyty poza marketplace?
7. Kto wybiera finalny tier: wallet, policy, operator czy merchant?
8. Kiedy wallet musi eskalowac do `FullPrivacy`?
9. Czy user moze recznie podniesc tier?
10. Czy user moze recznie obnizyc tier?
11. Jak privacy tiers lacza sie z `ServicePaymentPolicy`?
12. Jak privacy tiers lacza sie z `SpendGrant`?
13. Jak komunikowac te tiery w UX bez mylenia usera?

## 13. Czego nie wolno proponowac

Nie proponuj:

- slabszej kryptografii PQ dla drobnicy,
- uzywania `SmallPaymentsRail` poza marketplace,
- uzywania `RecipientPrivacyLite` dla escrow lub wysokowartosciowych sporow,
- modelu, w ktorym operator lub merchant moga obnizyc tier wbrew polityce i walletowi,
- modelu, w ktorym user moze zawsze wymusic tanszy rail mimo ryzyka,
- modelu, w ktorym privacy tiers sa tylko luźna sugestia UI.

## 14. Wymagany format odpowiedzi

Odpowiedz ma miec dokladnie te sekcje:

1. `Recommended v0 design`
2. `Alternative design`
3. `Privacy tier definitions`
4. `Tier selection matrix`
5. `Wallet decision model`
6. `Escalation rules`
7. `UX and communication model`
8. `Frozen decisions`
9. `Open questions`

## 15. Oczekiwany wynik

Po przeczytaniu tego dokumentu i wykonaniu zadania mamy dostac:

- gotowy katalog privacy-tierow `privAI v0`,
- gotowa macierz kiedy wolno uzyc ktorego tieru,
- gotowa odpowiedz, kiedy trzeba wymusic `FullPrivacy`,
- gotowa odpowiedz, jak wallet podejmuje decyzje,
- gotowa odpowiedz, jak policy i granty wchodza w wybor tieru,
- gotowe podsumowanie dla wdrozenia i dla Gemini,
- minimalna liste pytan otwartych, jesli cos naprawde musi zostac otwarte.

Nie chcemy po tej iteracji dostac kolejnego brainstormingu.
Chcemy dostac material, z ktorego da sie robic spec i implementacje.
