# privAI System Freeze Draft for PVA Coin

Status: draft roboczy z zatwierdzonymi decyzjami bazowymi.
Canonicality: canonical source of truth for nadrzedny model produktu i privacy rails w canonical spec set. Wszystkie normatywne docs systemu musza mieszkac pod `spec/`.
Owner: privAI protocol and product freeze.
Depends on: none.
Supersedes: czesciowo `PRIVAI_V0_PROTOCOL.md` i `PRIVAI_V0_PAYMENTS_AND_ECONOMICS.md` na poziomie finalnej semantyki produktu.

Cel tego dokumentu:
- zamrozic docelowy model produktu i platnosci dla projektu `privAI` oraz `privAI Coin (PVA)`,
- zatrzymac dalsze rozjazdy miedzy docs a kodem,
- oddzielic rzeczy finalne od rzeczy eksperymentalnych,
- pozwolic rozwijac implementacje bez ciaglego zmieniania pol i semantyki.

Kazdy podobny dokument freeze powinien miec jawnie 3 warstwy:
- `jak ma byc finalnie`,
- `jaki jest stan obecny`,
- `co trzeba zrobic, aby dojsc do stanu finalnego`.

Ten dokument opisuje model docelowy systemu.
Nie oznacza, ze kazdy element jest juz w 100% domkniety w kodzie.
Jesli kod nie zgadza sie z tym dokumentem, trzeba doprowadzic kod do zgodnosci z dokumentem albo jawnie oznaczyc dana sciezke jako `experimental`.

## How Devs And Agents Should Read This Spec Set

Punktem startowym dla kazdego deva i agenta AI pracujacego przy `privAI` jest:
- `spec/PRIVAI_SPEC_INDEX.md`

Ten dokument jest pierwszym dokumentem semantycznym po indexie.

Kolejnosc czytania canonical set:
1. `spec/PRIVAI_SPEC_INDEX.md`
2. ten dokument
3. `spec/PRIVAI_PROTOCOL_CORE.md`
4. `spec/PRIVAI_CANONICAL_FORMATS.md`
5. `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`
6. `spec/PRIVAI_CONSENSUS.md`
7. `spec/PRIVAI_REFERENCE_VECTORS.md`

Zamrozona zasada pracy:
- te dokumenty sa roadmapa systemu i zrodlem prawdy dla architektury,
- wszystkie normatywne source-of-truth docs musza znajdowac sie pod `spec/`,
- zadania implementacyjne maja byc pisane pod ten zestaw dokumentow,
- dev lub agent nie powinien dopowiadac architektury poza tym, co wynika z canonical set,
- jesli jakas rzecz nie jest tu opisana, nalezy ja potraktowac jako `not yet frozen`, a nie jako wolna przestrzen do projektowania.

Dokumenty i pliki poza `spec/`:
- moga byc czytane pomocniczo,
- moga byc przydatne do znalezienia implementacji albo historii decyzji,
- ale nie moga byc normatywnym source of truth dla finalnego systemu.

Jesli kod nie zgadza sie z canonical docs:
- nie wolno po cichu "dostosowywac docs do kodu",
- nalezy albo doprowadzic kod do zgodnosci z docs,
- albo jawnie dopisac `current non-conformity`,
- albo przygotowac jawny freeze update.

Jawnego freeze update wymagaja zawsze:
- zmiana modelu produktu,
- zmiana raili,
- zmiana progow i jednostek,
- zmiana semantyki core objects,
- zmiana wire-format commitments,
- zmiana trust assumptions.

## Interpretation Status Vocabulary

Ponizsze etykiety sa zamrozonym slownikiem interpretacji dla devow i agentow.
Nie wolno dopowiadac sobie dodatkowych "prawie takich samych" statusow.

- `Current canonical`
  - oznacza stan, ktory jest dzis normatywny i ma byc implementowany, testowany i czytany jako biezace source of truth dla danej warstwy.
- `Frozen spec rule`
  - oznacza zamrozona regule docelowa, ktora jest juz zatwierdzona semantycznie; jesli kod jej jeszcze nie spelnia, to kod ma zostac doprowadzony do zgodnosci albo jawnie oznaczony jako gap.
- `Frozen future target requiring migration`
  - oznacza zatwierdzony target przyszly, ale nie wolno go wdrazac przez domysly; wymaga osobnej jawnej migracji formatu, wektorow i taska implementacyjnego.
- `Current non-conformity`
  - oznacza znany rozjazd miedzy docs a kodem; to nie daje prawa wyboru "ktora wersja bardziej pasuje".
- `Provisional`
  - oznacza byt lub semantyke tymczasowo obecna, ale bez finalnej zamrozonej roli; nie wolno jej rozszerzac bez osobnego canonical doc albo freeze update.
- `Experimental`
  - oznacza sciezke poza finalna gwarancja systemu; nie trafia do finalnych obietnic produktu ani finalnych reference vectors.

## Anti-Hallucination Rule

Zamrozona zasada:
- jesli cos nie ma jawnie przypisanej etykiety statusu albo jawnej formuly, nalezy to traktowac jako `unresolved`,
- `unresolved` nie wolno implementowac przez zgadywanie,
- dev lub agent nie moze sam przypisac:
  - enum tagow,
  - bajtow,
  - root formulas,
  - signed payload formulas,
  - threshold rules,
  - nowych pol,
  - nowych semantyk.

Dozwolone reakcje na `unresolved` sa tylko trzy:
- przygotowac freeze update,
- przygotowac task dopinajacy brakujacy canonical detail,
- oznaczyc luke jako `current non-conformity`, `provisional` albo `experimental`, jesli to rzeczywisty stan systemu.

## Fundamental invariants

Ponizsze elementy sa fundamentem systemu i nie powinny byc rozluzniane ani podmieniane bez nowego jawnego freeze update:

- model `note/UTXO`,
- jednorazowe `ReceiveBundle`,
- `RecipientBox` jako podstawowy mechanizm ukrycia odbiorcy,
- `nullifier`-based anti-double-spend,
- hierarchia `master_seed -> roots -> one-time derived keys`,
- brak oslabiania warstwy PQ dla zadnego raila.

## 1. Nazwa i jednostka

Finalna nazwa coina:
- `privAI Coin`

Finalny ticker:
- `PVA`

Finalny model jednostek ma byc monero-style:
- `PVA` jest glowna jednostka uzytkowa,
- ledger zapisuje kwoty jako liczby calkowite w najmniejszej jednostce atomowej,
- finalna nazwa najmniejszej jednostki: `aPVA`,
- freeze zamraza:
  - `1 PVA = 10^12 aPVA`

Finalne zasady:
- progi produktu, polityki i UX sa opisywane w `PVA`,
- wire format i ledger accounting operuja na integer amounts w `aPVA`,
- system nie powinien opierac polityk prywatnosci o USD ani inna walute zewnetrzna,
- konwersja `PVA <-> aPVA` musi byc deterministyczna i bez ulamkow.

### 1.1. Stan obecny i luka implementacyjna

Stan obecny kodu nie jest jeszcze zgodny z finalnym modelem jednostek.

Obecnie:
- czesc kodu opiera kwoty o waski model `Amount14`,
- obecny limit matematyczny plaintext space jest za maly, zeby naiwnie przeniesc model `1 PVA = 10^12 aPVA`,
- obecna implementacja lite raila nie moze byc traktowana jako finalna reprezentacja kwot dla modelu `PVA + aPVA`.

To nie przekresla kierunku.
To znaczy tylko, ze amount layer musi zostac przebudowany tak, aby zgadzal sie z finalnym modelem produktu.

## 2. Finalne tory platnosci

System ma miec dokladnie 3 kanoniczne tory platnosci:

1. `FullPrivacy`
2. `OnChainLite`
3. `MarketplaceSmallPaymentsRail`

Nie wolno mieszac semantyki tych torow.

`FullPrivacy`:
- to glowny tor prywatny systemu,
- sluzy do wiekszych kwot i flow wrazliwych,
- ma chronic platnika, odbiorce, kwote i graf transakcji w najsilniejszy dostepny sposob.

`OnChainLite`:
- to lekki tor on-chain dla malych platnosci,
- kwota moze byc jawna,
- platnik i odbiorca maja pozostac nielinkowalni na poziomie produktu,
- ten tor nie jest zamiennikiem marketplace raila.

`MarketplaceSmallPaymentsRail`:
- to operator/trusted-accounting rail dla usage-metered marketplace,
- sluzy do `SpendGrant -> Receipt -> MarketplaceBatchTx`,
- nie jest tym samym co `OnChainLite`,
- moze miec inny model prywatnosci i inny model zaufania.

### 2.1. Regula nazewnictwa

Zamrozona zasada nazewnictwa:
- `OnChainLite` jest nazwa raila / toru platnosci,
- `RecipientPrivacyLite` jest nazwa obietnicy prywatnosci, a nie nazwa tx type ani osobnego raila,
- `OnChainLite` ma spelniac obietnice `RecipientPrivacyLite`,
- `MarketplaceSmallPaymentsRail` nie jest odmiana `OnChainLite` i nie powinien byc tak opisywany.

## 3. Finalne zasady prywatnosci

### 3.1. FullPrivacy

`FullPrivacy` jest wymagane dla:
- wszystkich platnosci powyzej progu lite,
- escrow,
- dispute-sensitive flows,
- payoutow,
- settlementow wrazliwych,
- wszystkich flow, w ktorych privacy amount jest krytyczna.

`FullPrivacy` jest zawsze dozwolone takze dla malych kwot.

### 3.2. OnChainLite

`OnChainLite` ma spelniac nastepujaca obietnice produktu:
- mala kwota moze byc jawna,
- platnik nie moze miec stalej publicznej tozsamosci,
- odbiorca nie moze miec stalej publicznej tozsamosci,
- kolejne male platnosci nie powinny byc prosto linkowalne do tego samego platnika lub odbiorcy.

To oznacza wprost:
- sama rotacja kluczy nie wystarcza, jesli nadal zostaje jawny graf inputow,
- finalny `OnChainLite` nie moze byc uznany za zamrozony, dopoki nie spelni powyzszej obietnicy.

### 3.2.1. Acceptance criteria for OnChainLite

Przed uznaniem `OnChainLite` za finalny musza byc spelnione wszystkie ponizsze warunki:
- brak stalego publicznego payer/account id,
- brak stalego publicznego receiver/account id,
- brak jawnego input graph, ktory prostolinijnie linkuje kolejne platnosci tego samego usera,
- wallet, ledger i consensus wspieraja ten rail end-to-end,
- docs, implementacja i testy opisuja ten rail w ten sam sposob,
- dopoki powyzsze warunki nie sa spelnione, status pozostaje `experimental`.

### 3.3. MarketplaceSmallPaymentsRail

`MarketplaceSmallPaymentsRail` ma spelniac nastepujaca obietnice produktu:
- sluzy do malych, szybkich i usage-metered rozliczen w marketplace,
- opiera sie o policy, grant, receipt i batch settlement,
- jego model prywatnosci i trust assumptions sa inne niz w `OnChainLite`,
- nie wolno sprzedawac go jako tego samego mechanizmu co lite p2p on-chain.

## 4. Progi kwotowe

Freeze draft wprowadza parametr:
- `MAX_LITE_TX_AMOUNT_PVA = 500 PVA`

Zasady:
- dla kwoty `<= 500 PVA` user moze wybrac `OnChainLite` albo `FullPrivacy`,
- dla kwoty `> 500 PVA` obowiazkowe jest `FullPrivacy`,
- dla flow wrazliwych `FullPrivacy` moze byc wymagane takze ponizej `500 PVA`.

To jest sufit dla lekkiego toru.
To nie jest nakaz uzywania lekkiego toru.

Finalna zasada reprezentacji:
- prog jest definiowany w `PVA`,
- implementacja moze przechowywac jego odpowiednik w `aPVA`,
- sam prog nie moze wymuszac zmiany wire format.

Przyklady:
- `30 PVA` moze isc przez `FullPrivacy`,
- `30 PVA` moze isc przez `OnChainLite`, jesli flow na to pozwala,
- `300 PVA` moze isc przez `FullPrivacy`,
- `300 PVA` moze isc przez `OnChainLite`, jesli flow na to pozwala,
- `700 PVA` nie moze isc przez `OnChainLite`,
- escrow `30 PVA` nadal moze wymagac `FullPrivacy`.

## 5. Zmiennosc progu

`MAX_LITE_TX_AMOUNT_PVA` jest parametrem systemowym i moze zostac zmieniony w przyszlosci.

Zamrozona zasada:
- zmiana progu NIE moze wymagac zmiany wire format transakcji,
- zmiana progu NIE moze wymagac zmiany canonical encoding,
- zmiana progu NIE moze wymagac zmiany formatu noty lub bundla.

To jest parametr polityki/systemu, nie element struktury transakcji.

Stan finalny:
- zmiana `500 -> 300` albo `500 -> 1000` ma byc zmiana parametru systemowego,
- nie moze oznaczac przebudowy modelu danych.

### 5.1. Parameter scope

Ta sekcja istnieje po to, zeby uniknac sytuacji, w ktorej jeden parametr jest traktowany raz jako UX policy, a raz jako element wire format.

| Parametr | Product | Wallet | Consensus | Wire format |
|----------|---------|--------|-----------|-------------|
| `MAX_LITE_TX_AMOUNT_PVA` | tak | tak | tak | nie |
| `1 PVA = 10^12 aPVA` | tak | tak | tak | nie |
| flow-sensitive escalation thresholds | tak | tak | czesciowo | nie |

Zasady interpretacji:
- jesli parametr jest w kolumnie `Wire format = nie`, jego zmiana nie moze zmieniac kolejnosci pol ani canonical encoding,
- `Consensus = tak` oznacza, ze finalny system musi umiec egzekwowac dany parametr przy walidacji,
- `Consensus = czesciowo` oznacza, ze czesc egzekucji nalezy do wallet/policy layer, a czesc do jawnego typu flow lub tx class.

### 5.2. Zamrozona interpretacja progow i jednostek

Zamrozona interpretacja:
- `MAX_LITE_TX_AMOUNT_PVA` jest parametrem walidacji systemowej,
- nie jest polem wire format,
- jego zmiana nie zmienia canonical encoding,
- finalny consensus i wallet musza respektowac ten limit przy wyborze i walidacji raila lite,
- `1 PVA = 10^12 aPVA` jest finalna zasada jednostek systemu, ale nie jest serializowana jako osobne pole wewnatrz struktur transakcyjnych.

## 6. Stan obecny vs finalny model

Ta sekcja istnieje po to, zeby uniknac dalszych nieporozumien.

### 6.1. Amount model

Stan finalny:
- user widzi i ustawia kwoty w `PVA`,
- ledger i wire format operuja na `aPVA`,
- progi prywatnosci sa definiowane w `PVA`,
- implementacja przelicza je deterministycznie do `aPVA`.

Stan obecny:
- obecny kod amountow nie jest jeszcze przygotowany na finalny model `PVA + aPVA`.

### 6.2. FullPrivacy

Stan finalny:
- `FullPrivacy` jest kanonicznym torem dla duzych kwot i flow wrazliwych.

Stan obecny:
- rdzen `note/UTXO`, bundle, recipient box i nullifier logic pozostaja dobra baza do tego toru.

### 6.3. OnChainLite

Stan finalny:
- kwota moze byc jawna,
- platnik i odbiorca maja pozostac nielinkowalni na poziomie produktu.

Stan obecny:
- istnieje zalazek lite raila,
- ale obecny stan nie moze jeszcze byc uznany za spelniajacy finalna obietnice prywatnosci.

### 6.4. MarketplaceSmallPaymentsRail

Stan finalny:
- marketplace rail pozostaje osobnym produktem opartym o policy, grant, receipt i batch settlement.

Stan obecny:
- ten tor ma dobra baze semantyczna,
- ale docs i checks konsensusu nadal wymagaja pelnego doszlifowania.

### 6.5. Tabela statusowa

| Obszar | Target status | Implementation status |
|--------|---------------|-----------------------|
| `FullPrivacy` | frozen target | partially implemented |
| `OnChainLite` | target defined | still experimental |
| `MarketplaceSmallPaymentsRail` | product model frozen | implementation alignment pending |

### 6.6. Current non-conformities

Najwazniejsze obecne niezgodnosci wzgledem modelu finalnego:
- `amount layer`: obecny model amountow nie odpowiada jeszcze finalnemu `PVA + aPVA`,
- `lite rail privacy gap`: obecny lite rail nie spelnia jeszcze finalnej obietnicy nielinkowalnosci,
- `docs/code drift`: czesc docs i czesc implementacji nadal opisuja ten sam obiekt roznymi slowami albo na roznych poziomach szczegolowosci,
- `experimental tx types`: istnieja typy i sciezki implementacyjne, ktore sa juz w kodzie, ale nie powinny byc jeszcze traktowane jako finalne.

## 7. Finalny rdzen noty i odbioru

Finalny rdzen systemu pozostaje oparty o:
- `ReceiveBundle`,
- `RecipientBox`,
- `RecipientBoxPlaintext`,
- `OutputNote`,
- `Nullifier`,
- model `note/UTXO`.

Jednorazowe bundle odbiorcze pozostaja podstawowym mechanizmem ukrycia odbiorcy.

Finalne zasady:
- bundle jest jednorazowy,
- wallet bez lokalnych secret keys nie moze otworzyc cudzego `RecipientBox`,
- `nullifier_key` nie moze byc dowolnie wybierany przez nadawce,
- derivation i walidacja `nullifier_key` pozostaja elementem modelu finalnego.

## 8. Master seed i model kluczy

Model z `master_seed` zostaje i jest uznany za dobry kierunek finalny.

Docelowa hierarchia:
- `master_seed`
- `spend_root`
- `scan_root`
- `nullifier_root`
- `kem_root`

Finalne zasady:
- wallet moze odtworzyc swoj stan z `master_seed`,
- system ma preferowac jednorazowe klucze pochodne zamiast stalej publicznej tozsamosci,
- eksport skanowania moze istniec bez eksportu spend authority,
- klucze jednorazowe maja byc pochodna od `master_seed`, a nie zbiorem recznie zarzadzanych bytow.

## 9. Finalny model tx types

Kanoniczne typy platnosci do utrzymania:
- `TransferNoteTx` dla `FullPrivacy`,
- finalny typ dla `OnChainLite`,
- `MarketplaceBatchTx` dla marketplace raila.

Freeze draft zaklada:
- obecny `TransferNoteTx` jest baza toru `FullPrivacy`,
- `MarketplaceBatchTx` zostaje jako finalny typ settlementowy dla marketplace,
- obecny `LiteTransferTx` nie jest jeszcze automatycznie uznany za finalny tylko dlatego, ze istnieje w kodzie.

Finalna zasada:
- sciezka lite moze zostac uznana za finalna dopiero, gdy spelni produktowa obietnice nielinkowalnosci oraz bedzie domknieta end-to-end w wallet, ledger i consensus.

## 10. Wire format i source of truth

Od chwili zatwierdzenia tego dokumentu obowiazuje zasada:
- architektura i semantyka produktu wynikaja z tego dokumentu,
- canonical bytes i hash commitments musza byc jednoznacznie zdefiniowane,
- zmiana pola w strukturze wymaga najpierw zmiany spec, potem zmiany kodu.

Do chwili pelnego wypelnienia canonical vectors:
- `spec/PRIVAI_CANONICAL_FORMATS.md` pozostaje normatywnym zrodlem prawdy dla bajtow, field order i commitments,
- kod `CanonicalEncode` moze byc uzywany tylko jako referencja implementacyjna do wykrywania current behavior i generowania vectors,
- current behavior z kodu nie moze samodzielnie nadpisac canonical docs.

Po synchronizacji:
- docs i kod musza mowic jednym glosem,
- nie wolno utrzymywac dwoch roznych opisow tego samego obiektu.

### 10.1. Canonical final spec set

Finalny system `privAI` nie moze opierac sie na luznej grupie "waznych" dokumentow.  
Musi miec jedna zamknieta liste kanonicznych plikow specyfikacyjnych.

Canonical entrypoint dla calego zestawu:

0. `spec/PRIVAI_SPEC_INDEX.md`
   Zakres:
   - jeden punkt wejscia dla devow i agentow,
   - reading order,
   - granica miedzy canonical docs a referencjami nienormatywnymi,
   - zasady przydzielania taskow do frozen docs.

Po pelnym freeze za jedyne kanoniczne dokumenty systemu nalezy uznac tylko:

1. `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA.md`  
   Zakres:
   - finalny model produktu,
   - nazwa i jednostki coina,
   - privacy rails,
   - progi i zasady eskalacji,
   - statusy `final/experimental/legacy`,
   - change-control dla calego systemu.

2. `spec/PRIVAI_PROTOCOL_CORE.md`
   Zakres:
   - rdzen `note/UTXO`,
   - `ReceiveBundle`,
   - `RecipientBox`,
   - `RecipientBoxPlaintext`,
   - `Nullifier`,
   - semantyka finalnych tx types,
   - semantyka przejsc stanu na poziomie protokolu.

3. `spec/PRIVAI_CANONICAL_FORMATS.md`
   Zakres:
   - canonical encoding,
   - kolejnosc pol,
   - domain separation,
   - hash commitments,
   - integer encoding,
   - golden vectors,
   - bit-level source of truth.

4. `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`
   Zakres:
   - finalny `MarketplaceSmallPaymentsRail`,
   - `ServicePaymentPolicy`,
   - `SpendGrant`,
   - `ticket_id`,
   - `ticket_nullifier`,
   - `Receipt`,
   - `receipt_root`,
   - `settlement_root`,
   - trust assumptions marketplace operatora.

5. `spec/PRIVAI_CONSENSUS.md`
   Zakres:
   - finality model,
   - role validator/prover/operator,
   - block validity,
   - proof-carrying execution,
   - state commitments,
   - zasady walidacji na poziomie consensus.

6. `spec/PRIVAI_REFERENCE_VECTORS.md`
   Zakres:
   - referencyjne canonical bytes,
   - referencyjne commitments,
   - referencyjne signed payloads,
   - referencyjne merkle roots,
   - bit-level test fixtures dla finalnych obiektow.

Zamrozona zasada:
- jesli pliku nie ma na powyzszej liscie, nie moze byc traktowany jako finalne zrodlo prawdy dla systemu,
- dokument spoza `spec/` nie moze byc finalnym zrodlem prawdy dla systemu,
- pliki spoza tej listy moga byc pomocnicze, historyczne, review-only albo migracyjne,
- pliki spoza tej listy nie moga nadpisywac semantyki ani wire format zdefiniowanych przez canonical final spec set,
- nie wolno mowic o "pol-kanonicznych" lub "waznych, ale nie najwazniejszych" docs.

### 10.2. Regula przejsciowa

Do chwili utworzenia i zatwierdzenia wszystkich plikow z canonical final spec set:
- ten dokument pelni role tymczasowego zrodla prawdy dla modelu produktu i privacy rails,
- stare dokumenty moga byc wykorzystywane tylko pomocniczo,
- kazdy stary dokument musi zostac albo:
  - zsynchronizowany z canonical set,
  - oznaczony jako `legacy`,
  - oznaczony jako `historical`,
  - albo zastapiony nowym dokumentem kanonicznym.

## 11. Rzeczy finalne vs rzeczy eksperymentalne

Za finalne nalezy uznac:
- model `note/UTXO`,
- jednorazowe `ReceiveBundle`,
- `RecipientBox`,
- `OutputNote`,
- `master_seed` hierarchy,
- `MarketplaceSmallPaymentsRail` jako finalny model produktowy i finalny kierunek architektoniczny dla marketplace,
- prog `MAX_LITE_TX_AMOUNT_PVA` jako parametr systemowy.

Za eksperymentalne nalezy uznac do czasu pelnego domkniecia:
- kazdy on-chain lite rail, ktory nie spelnia jeszcze finalnej obietnicy nielinkowalnosci platnika i odbiorcy,
- kazda implementacje lite tx, ktora nie jest jeszcze zgodna z consensus i ledger end-to-end,
- kazda implementacje, ktora zmienia wire format bez uprzedniego freeze spec,
- kazda implementacje marketplace raila, ktora nie jest jeszcze zsynchronizowana z finalnym modelem docs i checks.

## 12. Co trzeba zrobic, aby dojsc do finalnego efektu

Ta sekcja jest obowiazkowa.
Nie opisuje marzen.
Opisuje konkretne klasy prac potrzebnych do osiagniecia modelu finalnego.

### 12.1. Amount layer i jednostki

Do zrobienia:
- oddzielic finalnie `PVA` jako jednostke uzytkowa od `aPVA` jako jednostki ledgerowej,
- usunac zalozenie, ze finalny model amount moze pozostac zwiazany z obecnym waskim `Amount14`,
- zdefiniowac finalny integer amount encoding dla `aPVA`,
- zdefiniowac zasady zaokraglen i konwersji z UX do ledger amounts,
- upewnic sie, ze progi typu `500 PVA` sa przeliczane do `aPVA` bez utraty precyzji.

### 12.2. FullPrivacy core

Do zrobienia:
- zsynchronizowac docs z obecnym poprawnym rdzeniem noty, bundla i recipient box,
- dopisac canonical vectors dla `ReceiveBundle`, `RecipientBox`, `RecipientBoxPlaintext`, `OutputNote`, `Nullifier`,
- sprawdzic, ze finalna semantyka `FullPrivacy` jest zgodna z docs i proof path.

### 12.3. OnChainLite

Do zrobienia:
- zdefiniowac finalny data model lite raila zgodny z docelowa obietnica prywatnosci,
- usunac lub przebudowac miejsca, ktore zostawiaja jawna linkowalnosc platnika lub odbiorcy,
- domknac end-to-end integracje lite raila w wallet, ledger i consensus,
- oznaczyc obecny stan jako `experimental`, dopoki powyzsze nie zostanie wykonane.

### 12.4. MarketplaceSmallPaymentsRail

Do zrobienia:
- domknac finalne checks dla `SpendGrant`, `Receipt`, `receipt_root`, `nullifier_count`, `receipt_count`, `settlement_root`,
- opisac trust assumptions jawnie i bez skrotow myslowych,
- zsynchronizowac marketplace docs z aktualnym stanem implementacji.

### 12.5. Source of truth i freeze hygiene

Do zrobienia:
- wskazac jeden dokument nadrzedny freeze jako zrodlo prawdy dla modelu produktu,
- wskazac zamkniety `canonical final spec set` jako zrodlo prawdy dla pozostalych warstw systemu,
- wskazac kod `CanonicalEncode` plus test vectors jako zrodlo prawdy dla bajtow,
- dopisac do podobnych docs obowiazkowe sekcje:
  - `stan finalny`,
  - `stan obecny`,
  - `co trzeba zrobic`.

### 12.6. Testy i vectors

Do zrobienia:
- dodac golden vectors dla finalnych struktur,
- dodac roundtrip encode/decode tests,
- dodac tests typu bit-flip: zmiana jednego pola zmienia commit,
- dodac tests policy threshold dla `MAX_LITE_TX_AMOUNT_PVA`,
- dodac tests zgodnosci docs examples z kodem.

## 13. Wymagania implementacyjne po zatwierdzeniu

Po zatwierdzeniu tego dokumentu implementacja powinna wykonac tylko nastepujace klasy zmian:
- doprowadzenie kodu do zgodnosci z freeze,
- dopisanie test vectors i canonical encoding tests,
- domkniecie consensus i ledger dla finalnych tx types,
- usuniecie lub oznaczenie rzeczy eksperymentalnych,
- synchronizacje starszych docs z finalnym modelem.

Nie powinno sie juz robic:
- luznych zmian pol w strukturach bez freeze update,
- dodawania nowych raili bez nowego dokumentu decyzji,
- przepychania architektury przez sam kod bez jawnej decyzji produktowej.

## 14. Twarda checklista finalnego stanu

Ponizsza lista ma charakter blokujacy.
Element systemu nie moze byc nazywany `final`, jesli nie spelnia odpowiednich punktow z tej checklisty.

### 14.1. Finalny stan systemu jako calosci

- [ ] istnieje jeden zatwierdzony dokument freeze opisujacy finalny model produktu, prywatnosci i tx types,
- [ ] docs i kod nie opisuja juz tego samego obiektu na dwa rozne sposoby,
- [ ] wszystkie rzeczy poza freeze scope sa jawnie oznaczone jako `planned` albo `experimental`,
- [ ] nie istnieje aktywna implementacja, ktora obiecuje wiecej prywatnosci niz dokument finalny.

### 14.2. Jednostki i progi

- [ ] finalny model `PVA + aPVA` jest zatwierdzony,
- [ ] konwersja `PVA <-> aPVA` jest deterministyczna i bez ulamkow,
- [ ] `MAX_LITE_TX_AMOUNT_PVA` jest parametrem systemowym, a nie elementem wire format,
- [ ] progi prywatnosci sa definiowane w `PVA`, a implementacja przelicza je do `aPVA` bez utraty precyzji,
- [ ] obecny waski model amount nie blokuje finalnej ekonomii systemu.

### 14.3. FullPrivacy

- [ ] `FullPrivacy` jest jedynym wymaganym torem dla kwot powyzej progu lite,
- [ ] `FullPrivacy` jest jedynym wymaganym torem dla flow wrazliwych,
- [ ] `FullPrivacy` pozostaje zawsze dostepne takze dla malych kwot,
- [ ] docs, wallet, proof path i consensus zgadzaja sie co do semantyki `FullPrivacy`.

### 14.4. OnChainLite

- [ ] finalny lite rail ma jasno zdefiniowany model danych,
- [ ] mala kwota moze byc jawna,
- [ ] platnik nie ma stalej publicznej tozsamosci,
- [ ] odbiorca nie ma stalej publicznej tozsamosci,
- [ ] kolejne male platnosci nie sa prosto linkowalne do tego samego platnika lub odbiorcy,
- [ ] lite rail jest domkniety end-to-end w wallet, ledger i consensus,
- [ ] dopoki powyzsze warunki nie sa spelnione, lite rail pozostaje `experimental`.

### 14.5. MarketplaceSmallPaymentsRail

- [ ] marketplace rail jest jawnie opisany jako osobny produkt, a nie odmiana lite p2p,
- [ ] `SpendGrant`, `Receipt`, `receipt_root`, `settlement_root` i liczniki batcha maja finalne checks,
- [ ] trust assumptions operatora sa jawnie opisane w docs,
- [ ] docs marketplace sa zsynchronizowane z implementacja.

### 14.6. Note model, bundle i klucze

- [ ] `ReceiveBundle`, `RecipientBox`, `RecipientBoxPlaintext`, `OutputNote` i `Nullifier` maja finalna semantyke,
- [ ] bundle pozostaje jednorazowy,
- [ ] `nullifier_key` nie moze byc dowolnie wybierany przez nadawce,
- [ ] wallet moze odtworzyc stan z `master_seed`,
- [ ] model `master_seed -> roots -> one-time derived keys` pozostaje finalnym kierunkiem systemu.

### 14.7. Wire format i matematyka

- [ ] canonical encoding finalnych struktur jest jednoznacznie zamrozony,
- [ ] kolejnosc pol, domain separation i hash commitments sa opisane i testowalne,
- [ ] istnieja golden vectors dla finalnych struktur i commitow,
- [ ] istnieja roundtrip encode/decode tests,
- [ ] istnieja tests typu bit-flip potwierdzajace, ze zmiana pola zmienia commit,
- [ ] docs examples i wyniki kodu zgadzaja sie bitowo.

### 14.8. Higiena pracy nad systemem

- [ ] nikt nie zmienia pol struktur bez uprzedniej zmiany freeze docs,
- [ ] nikt nie dodaje nowego raila bez nowego dokumentu decyzji,
- [ ] nowe implementacje nie zmieniaja obietnicy prywatnosci bez jawnej decyzji produktowej,
- [ ] kazdy podobny dokument zawiera sekcje `stan finalny`, `stan obecny`, `co trzeba zrobic` i `twarda checklista finalnego stanu`.

Ta sekcja ma zatrzymac dokladnie ten problem, ktory juz wystapil:
- brak wspolnej definicji celu,
- kod wybiegajacy przed decyzjami systemowymi,
- lokalne pomysly zamieniane w ukryte zmiany architektury.

## 15. Zatwierdzone decyzje bazowe

Na dzien freeze review te 4 decyzje zostaly zatwierdzone:

1. `PVA` jest finalnym tickerem.
2. Finalny model jednostek to `PVA + aPVA`, gdzie `1 PVA = 10^12 aPVA`.
3. `MAX_LITE_TX_AMOUNT_PVA = 500 PVA` jako startowy limit.
4. `FullPrivacy` jest zawsze dozwolone ponizej progu lite, a obowiazkowe powyzej progu lub dla flow wrazliwych.

To oznacza, ze dalsze prace nad systemem nie moga juz podwazac tych 4 decyzji bez nowego jawnego freeze update.

Reszta dokumentu pozostaje draftem roboczym do pelnego zatwierdzenia jako calosc, ale powyzsze decyzje nalezy traktowac jako zamrozone.
