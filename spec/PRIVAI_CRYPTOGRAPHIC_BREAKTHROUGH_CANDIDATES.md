# privAI Cryptographic Breakthrough Candidates

Status: research-direction doc for potential cryptographic contribution areas.
Canonicality: non-overriding focused research doc. This document does not define final bytes, protocol rules or consensus behavior by itself. It records where `privAI` may produce a real cryptographic contribution and what would have to be true before such a claim is defensible.
Owner: privAI research and protocol architecture.
Depends on:
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_PROOF_BOUNDARIES.md`
- `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zapisac, czy `privAI` idzie w kierunku realnego przelomu kryptograficznego,
- wskazac, ktore obszary maja najwiekszy potencjal na prawdziwy wklad kryptograficzny,
- odroznic wklad systemowo-architektoniczny od wkladu kryptograficznego,
- zablokowac naduzywanie slowa "przelom", zanim bedzie na to techniczna podstawa.

## 2. Uczciwy Status Na Dzis

Na dzis `privAI` jest:
- autorska architektura systemu prywatnych platnosci i escrow,
- autorskim polaczeniem privacy, note/UTXO, rails, marketplace accounting, escrow `2 z 3` i kierunku PQ,
- systemem z realnym potencjalem na wklad kryptograficzny.

Na dzis `privAI` nie jest jeszcze:
- udowodnionym przelomem kryptograficznym,
- nowym formalnie opisanym prymitywem kryptograficznym,
- systemem z opublikowanym i zrecenzowanym nowym mechanizmem kryptograficznym.

## 3. Kiedy Mozna Uczciwie Mowic O Przelomie Kryptograficznym

Mozna tak mowic dopiero wtedy, gdy istnieje co najmniej jedna z ponizszych rzeczy:

1. nowy prymityw kryptograficzny
   - nowy mechanizm, a nie tylko nowy system produktowy,
   - z jasnym opisem bezpieczenstwa i przewagi.

2. nowa konstrukcja z formalna przewaga
   - np. lepsza prywatnosc, mniej zalozen, lepsza efektywnosc albo inny model bezpieczenstwa,
   - pokazana formalnie albo bardzo mocno eksperymentalnie.

3. nowy model prywatnosci / auth / proof composition
   - ktory nie jest redukowalny do prostego sklejenia znanych elementow,
   - i daje realnie nowe wlasciwosci.

Minimalny prog dowodowy:
- wyodrebniony mechanizm,
- threat model,
- security claims,
- proof sketch albo formalny dowod,
- porownanie do stanu techniki,
- review zewnetrzne,
- najlepiej implementacja referencyjna i audyt.

## 4. Czego Nie Wolno Mylic Z Przelomem Kryptograficznym

Ponizsze rzeczy moga byc bardzo wartosciowe, ale same w sobie nie wystarcza:
- ambitna architektura,
- wielowarstwowy system,
- autorski produkt,
- note/UTXO + privacy + escrow + marketplace w jednym stosie,
- PQ signatures jako kierunek,
- spojny canonical spec set.

To moze byc:
- bardzo mocny system autorski,
- bardzo dobra kompozycja,
- bardzo dobra architektura.

Ale nie musi jeszcze byc:
- przelomem kryptograficznym.

## 5. Najmocniejsze Obszary Potencjalnego Wkladu Kryptograficznego

### 5.1. Threshold Authorization Dla Note Spends

To jest obecnie najmocniejszy kandydat.

Cel:
- zbudowac note-spend authorization, ktora:
  - wspiera `2 z 3` i inne quorum,
  - pozostaje spojna z note/UTXO,
  - dobrze integruje sie z privacy modelem,
  - dobrze integruje sie z kierunkiem PQ.

Dlaczego to jest mocne:
- obecne systemy czesto ida albo w klasyczny multisig wallet, albo w account-based auth,
- `privAI` probuje zrobic threshold auth jako warstwe nad spendem noty,
- to otwiera przestrzen na nowy model:
  - ownership oddzielone od approval authority,
  - threshold policy nad spendem,
  - mozliwe przyszle ukrycie auth semantics przez proof layer.

Co byloby realnym wkladem kryptograficznym:
- jesli powstanie konstrukcja, ktora nie jest tylko "dwa podpisy Falcon i licznik w ledgerze",
- jesli uda sie osiagnac:
  - prywatnosc rol albo quorum,
  - bezpieczne powiazanie z commitmentem noty,
  - PQ threshold-like auth bez wspolnego walleta,
  - formalnie uzasadnione security properties.

Co jeszcze nie wystarcza:
- samo `2 z 3` w ledgerze,
- samo `policy_opening + signer list`,
- sama adaptacja `nexum-core`.

### 5.2. Recipient Privacy / Ownership Binding

Drugi mocny kandydat.

Cel:
- miec model odbioru noty, w ktorym:
  - recipient ownership jest dobrze zwiazane z boxem i payloadem,
  - unlinkability jest mocna,
  - sender nie moze skutecznie podmieniac krytycznych elementow ownership path,
  - model pozostaje kompatybilny z PQ direction.

Dlaczego to jest mocne:
- to jest rdzen privacy payment systemu,
- tu latwo o pozorne rozwiazania, ktore sa praktyczne, ale kryptograficznie przecietne,
- jesli tu powstanie nowa konstrukcja z lepszym modelem wiazania ownership i recipient privacy, to moze to byc realny wklad.

Co byloby realnym wkladem kryptograficznym:
- nowy lub nietrywialny model bindingu:
  - `RecipientBox`
  - `note_payload_commit`
  - ownership verification
  - nullifier-related semantics
- model z jasnym security argumentem przeciw podmianom i relinkingowi,
- lepsza kompozycja privacy i practical receive path niz w typowych systemach.

Co jeszcze nie wystarcza:
- samo istnienie `RecipientBox`,
- samo szyfrowanie recipient payload,
- sama dobra architektura walleta.

### 5.3. Proof Model Dla Wielu Raili I Proof-Carrying Execution

Trzeci najmocniejszy kandydat.

Cel:
- miec proof/execution model, ktory:
  - nie jest jednym ZK mlotkiem do wszystkiego,
  - umie obsluzyc rozne raile bez rozmycia semantyki,
  - dobrze spina privacy, state commitments i consensus.

Dlaczego to jest mocne:
- wiekszosc systemow albo upraszcza wszystko do jednego execution path, albo ma slabe rozdzielenie warstw,
- `privAI` juz rozroznia:
  - `FullPrivacy`
  - `OnChainLite`
  - `MarketplaceSmallPaymentsRail`
- jesli uda sie dla tego zbudowac rzeczywiscie spojny i formalnie uzasadniony model proof coverage / statement coverage / public inputs, to moze to byc cos wiecej niz dobra architektura.

Co byloby realnym wkladem kryptograficznym:
- nowy sposob kompozycji:
  - statement commitments,
  - execution bundle,
  - proof certificates,
  - rail-specific proof semantics,
  - state commitments.
- model, w ktorym:
  - nie ma semantycznego chaosu miedzy railami,
  - privacy path i settlement path maja rozne wymagania,
  - a wszystko nadal sklada sie do jednego dobrze obronionego execution/finality modelu.

Co jeszcze nie wystarcza:
- samo posiadanie `ExecutionBundle` i `ProofCertificate`,
- sama strukturalna proof coverage,
- samo stwierdzenie "proof-carrying execution".

## 6. Obszary O Duzej Wartosci Systemowej, Ale Mniejszym Potencjale Na Samodzielny Przelom Kryptograficzny

### 6.1. Three-Rail Architecture

To jest bardzo mocny wklad systemowy i produktowy.
Sam w sobie nie jest jeszcze nowym prymitywem kryptograficznym.

### 6.2. MarketplaceSmallPaymentsRail

To jest bardzo dobra i uczciwa architektura trust modelu.
Najmocniejsza jest systemowo, niekoniecznie kryptograficznie.

### 6.3. Escrow Control-Plane Adaptation Z `nexum-core`

To jest bardzo dobra architektura integracyjna.
Samo w sobie to nie jest jeszcze nowa kryptografia.

## 7. Co Musi Sie Wydarzyc, Zeby Kierunek Zamienil Sie W Realny Wklad Kryptograficzny

### 7.1. Formalizacja

Trzeba wyodrebnic konkretne mechanizmy:
- co dokladnie jest nowe,
- co jest tylko adaptacja,
- co jest tylko policy/architecture.

### 7.2. Threat Models

Dla kazdego kandydata trzeba zapisac:
- przed kim bronimy,
- co jest publiczne,
- co jest prywatne,
- jakie sa zalozenia o adversarzu.

### 7.3. Security Claims

Trzeba umiec powiedziec:
- jaka prywatnosc jest gwarantowana,
- jaka unlinkability jest gwarantowana,
- co jest auth guarantee,
- co jest unforgeability boundary,
- co jest binding guarantee.

### 7.4. State Of The Art Comparison

Trzeba porownac kazdy kandydat do:
- Monero-like systems,
- Zcash-like note systems,
- threshold auth systems,
- PQ signature / PQ privacy approaches,
- proof-carrying execution systems.

### 7.5. External Review

Bez review z zewnatrz:
- ryzyko mylenia dobrej architektury z nowa kryptografia jest bardzo wysokie.

## 8. Najbardziej Pragmatyczny Plan Badawczy

### 8.1. Pierwszy kandydat do rozwijania

`Threshold authorization dla note spends`

Powod:
- ma bezposrednia wartosc produktowa,
- jest potrzebny do escrow,
- ma sensowny most miedzy current implementation needs i potencjalnym wkladem kryptograficznym.

### 8.2. Drugi kandydat

`Recipient privacy / ownership binding`

Powod:
- to jest serce privacy quality systemu,
- tu realnie mozna uzyskac przewage jakosciowa.

### 8.3. Trzeci kandydat

`Proof model dla wielu raili`

Powod:
- to jest najtrudniejszy obszar,
- ale tez najbardziej podatny na zbyt wczesne opowiesci bez formalizacji.

## 9. Twarda Zasada Nazewnictwa Na Dzis

Na dzis wolno mowic:
- `autorska architektura`
- `oryginalny system`
- `autorski protocol stack`
- `kandydat na realny wklad kryptograficzny`
- `system z potencjalem na przelom kryptograficzny`

Na dzis nie powinno sie mowic:
- `udowodniony przelom kryptograficzny`
- `nowy prymityw kryptograficzny`, jesli nie ma jeszcze formalnej konstrukcji
- `rewolucja kryptograficzna`
- `pierwszy taki system`, jesli nie ma mocnego porownania do stanu techniki

## 10. Finalny Wniosek

`privAI` idzie w kierunku, ktory moze doprowadzic do realnego przelomu kryptograficznego.

Najmocniejsze kandydaty sa trzy:
- threshold authorization dla note spends,
- recipient privacy / ownership binding,
- proof model dla wielu raili i proof-carrying execution.

Na dzis jest to:
- autorski i ambitny system kryptograficzno-platniczy,
- z realnym potencjalem na nowy wklad kryptograficzny,
- ale jeszcze nie etap, na ktorym uczciwie mozna twierdzic, ze przelom juz zostal osiagniety.

## 11. Current-Compatible Mapping Dla 3 Kandydatow

Ta sekcja istnieje po to, zeby zapisac trzy kandydaty w sposob spojny z tym, co system ma dzisiaj.

Nie wolno w tej sekcji:
- dopisywac nowej architektury nieobecnej w canonical docs,
- udawac, ze current implementation juz realizuje research target,
- mieszac `current canonical`, `future target requiring migration` i `unresolved`.

### 11.1. Kandydat 1: Threshold Authorization Dla Note Spends

#### Co jest juz spojne z obecnym systemem

- `privAI` jest systemem note/UTXO, a nie account-based.
- Escrow `2 z 3` jest juz przypisane do `FullPrivacy`.
- Obecnym nosnikiem polityki technicznej jest `SpendPolicy::MarketplaceSettlement`.
- `Falcon` jest naturalna warstwa podpisu/auth w systemie.
- `nexum-core` moze byc zaadaptowany jako control-plane dla escrow.

#### Co jeszcze nie jest dowiezione

- coin nie ma jeszcze poprawnego `tx_signing_hash` niezaleznego od `auth`,
- ledger nie dostaje jeszcze jawnego `policy_opening`,
- ledger nie egzekwuje jeszcze realnej semantyki `1 z 1` i `2 z 3`,
- proof plane nie realizuje jeszcze proof-aware threshold auth.

#### Co byloby realnym wkladem kryptograficznym

Realny wklad pojawia sie dopiero wtedy, gdy z current v1:
- `separate Falcon signatures + ledger threshold rule`

przejdziemy do czegos wyraznie mocniejszego, na przyklad:
- prywatnego threshold auth dla spendu noty,
- ukrywania rol albo quorum,
- mocniejszego bindingu miedzy policy, proposalem i spendem,
- PQ-aware threshold auth, ktory nie jest tylko prostym licznikiem podpisow.

#### Czego nie wolno twierdzic dzis

- nie wolno twierdzic, ze samo `2 z 3` w ledgerze jest juz nowym prymitywem kryptograficznym,
- nie wolno twierdzic, ze sama adaptacja `nexum-core` jest nowa kryptografia,
- nie wolno twierdzic, ze obecny auth model jest juz poprawnym modelem multisig.

#### Checkpointy badawcze

1. Dowiezc v1 current-compatible:
   - `tx_signing_hash`
   - `InputAuthV2`
   - `policy_opening`
   - threshold verification w ledgerze
2. Zdefiniowac formalny threat model dla threshold note spend auth.
3. Zdefiniowac, co ma pozostac publiczne, a co docelowo prywatne.
4. Zbadac, czy quorum/role da sie ukryc bez niszczenia praktycznosci systemu.
5. Porownac ten model do:
   - Monero-style multisig
   - account-based multisig
   - threshold signatures
   - note-based spend authorization systems

### 11.2. Kandydat 2: Recipient Privacy / Ownership Binding

#### Co jest juz spojne z obecnym systemem

- system ma `ReceiveBundle`,
- system ma `RecipientBox`,
- system ma `RecipientBoxPlaintext`,
- plaintext jest bindowany przez `note_payload_commit`,
- wallet-side verification dla ownership i `nullifier_key` jest juz istotna semantycznie,
- `note/UTXO + box + payload commit` to juz jest rdzen systemu.

#### Co jeszcze nie jest dowiezione

- nie ma jeszcze formalnego threat modelu dla ownership binding,
- nie ma jeszcze formalnie spisanych security claims dla recipient privacy,
- nie ma jeszcze research-level porownania do innych note systems,
- nie ma jeszcze wyodrebnionego twierdzenia "co tutaj jest kryptograficznie nowe".

#### Co byloby realnym wkladem kryptograficznym

Realny wklad pojawia sie, jesli uda sie pokazac:
- nietrywialny model ownership binding odporny na podmiany i relinking,
- mocny model recipient privacy przy praktycznym receive path,
- sensowne PQ-compatible polaczenie note payload binding, recipient box i ownership verification,
- przewage nad typowym "zaszyfrowany payload + standardowy note commit".

#### Czego nie wolno twierdzic dzis

- nie wolno twierdzic, ze samo istnienie `RecipientBox` jest juz nowa kryptografia,
- nie wolno twierdzic, ze dobra architektura wallet receive path sama w sobie jest przelomem,
- nie wolno twierdzic, ze recipient privacy ma juz formalnie wykazane przewagi nad stanem techniki.

#### Checkpointy badawcze

1. Spisac threat model dla recipient ownership i recipient privacy.
2. Wyodrebnic wszystkie binding points:
   - `ReceiveBundle`
   - `RecipientBox`
   - `RecipientBoxPlaintext`
   - `note_payload_commit`
   - `nullifier_key`
3. Spisac security claims:
   - binding
   - unlinkability
   - anti-substitution
   - ownership soundness
4. Porownac model do znanych note systems.
5. Zidentyfikowac, czy nowosc lezy w:
   - samej konstrukcji,
   - kompozycji,
   - czy tylko w dobrej architekturze implementacyjnej.

### 11.3. Kandydat 3: Proof Model Dla Wielu Raili

#### Co jest juz spojne z obecnym systemem

- system ma trzy raile o roznych semantykach,
- `ExecutionBundle` i `ProofCertificate` maja current canonical bytes,
- `FullPrivacy` jest glownym rail proof-sensitive,
- marketplace rail jest jawnie odciety od udawania full-ZK path,
- `OnChainLite` jest uczciwie oznaczone jako `Experimental`.

#### Co jeszcze nie jest dowiezione

- nie ma jeszcze finalnego proof coverage modelu dla `OnChainLite`,
- nie ma jeszcze pelnej frozen multi-rail semantics dla proof plane,
- current proof plane jest bardziej strukturalny niz kryptograficznie wyrozniony,
- nie ma jeszcze formalnego opisu nowej przewagi wzgledem innych modeli execution/proof composition.

#### Co byloby realnym wkladem kryptograficznym

Realny wklad pojawia sie, jesli uda sie zbudowac:
- rail-aware proof model,
- spojnosc miedzy statement commitments, public inputs, state commitments i finality,
- model, ktory nie redukuje wszystkiego do jednego proof path,
- a mimo to ma formalnie uzasadniona calosc execution semantics.

#### Czego nie wolno twierdzic dzis

- nie wolno twierdzic, ze samo istnienie `ExecutionBundle` i `ProofCertificate` oznacza juz nowy proof system,
- nie wolno twierdzic, ze `OnChainLite` ma juz finalny proof model,
- nie wolno twierdzic, ze current structural proof coverage jest juz przelomem kryptograficznym.

#### Checkpointy badawcze

1. Domknac proof boundaries dla wszystkich raili.
2. Zdefiniowac jawnie:
   - proof-covered
   - proof-sensitive
   - proof-optional
   - proof-forbidden
3. Opisac semantics:
   - `statement_root`
   - `public_inputs_root`
   - `proof_cert_root`
   - rail-specific coverage
4. Porownac model do innych proof-carrying execution systems.
5. Dopiero potem oceniac, czy istnieje nowy wklad kryptograficzny, czy tylko dobra architektura execution plane.

## 12. Priorytet Dalszego Rozwoju

Jesli chcemy pozostac spojni z obecnym systemem i jednoczesnie nie zatrzymac research direction, kolejnosc powinna byc taka:

1. `Threshold authorization dla note spends`
   Powod:
   - ma bezposrednia wartosc produktowa,
   - jest potrzebny do escrow,
   - daje najszybszy most miedzy implementation i research.

2. `Recipient privacy / ownership binding`
   Powod:
   - to jest serce privacy quality systemu,
   - tu mozna zrobic mocny research bez psucia obecnej architektury.

3. `Proof model dla wielu raili`
   Powod:
   - to jest najwiekszy obszar,
   - wymaga najwiecej formalizacji,
   - i jest najbardziej podatny na zbyt wczesne tezy bez twardych podstaw.

## 13. Wplyw Istniejacego `nxms-transport`, Audytu I Sieci P2P

Ta sekcja istnieje po to, zeby zapisac wprost:
- co juz dzis wzmacnia wiarygodnosc `privAI` jako systemu kryptograficznego,
- ale czego nadal nie nalezy mylic z samodzielnym przelomem kryptograficznym.

### 13.1. Co juz realnie istnieje

W `D:/privAI` istnieje juz:
- autorski transport `nxms-transport`,
- osobny audyt AEAD / KEM / Falcon lanes,
- zaczatek sieci P2P w `privai-node`,
- handshake PQC oparty o `FrodoKEM + Falcon`,
- szyfrowane frame'y i actor-style connection pool,
- gossip, state sync, QC verification i peer exchange.

To jest wazne, bo pokazuje, ze:
- system nie jest juz tylko papierowa architektura,
- istnieje realna warstwa transportu, auth i networking,
- istnieje juz praktyczna baza pod dalsza formalizacje.

### 13.2. Co wynika z audytu `nxms-transport`

Po stronie transportu masz juz:
- signature-first verification przed KEM decapsulation,
- exact-length checks po stronie wrappera KEM,
- silne domain separation,
- reference vectors i cross-checking,
- property tests,
- fuzzing i sanitizer workflow,
- prepared Falcon path jako mocniejszy tor produkcyjny,
- timing smoke i rozsadna baza pod dalsze CT claims.

To jest mocny sygnal systemowy i kryptoinzynierski.

To nie wystarcza jeszcze do twierdzenia:
- "mamy nowy prymityw transportu kryptograficznego",
- "mamy juz przelom kryptograficzny",
- "mamy formalnie zamkniety nowy model secure transport".

### 13.3. Co wynika z obecnej sieci P2P

Po stronie `privai-node` masz juz:
- Tor-only transport assumptions,
- `PeerBook` / allowlist model,
- Falcon-signed handshake,
- FrodoKEM-derived shared secret do szyfrowania ramek,
- bounded queues i backpressure,
- rate limiting i ban list,
- gossip propagation,
- state sync,
- QC verification,
- stake-aware consensus loop.

To wzmacnia kandydat 3, bo pokazuje, ze:
- execution, transport i consensus sa juz myslane razem,
- research nie zaczyna sie od pustej kartki,
- jest juz implementacyjny szkielet pod formalne rozdzielenie warstw.

### 13.4. Czy transport i P2P to osobny kandydat na przelom

Na dzis:
- nie jako glowny kandydat,
- tak jako bardzo mocna warstwa wspierajaca.

Powod:
- obecny transport i P2P sa bardzo istotne,
- ale same w sobie wygladaja bardziej jak:
  - autorska, dobrze utwardzana infrastruktura kryptograficzna,
  - niz juz wyodrebniony nowy prymityw albo nowa formalna konstrukcja.

Mozna z tego zrobic silniejszy research topic dopiero wtedy, gdy powstanie:
- wyrazny model formalny dla transportu,
- porownanie do innych PQ transport compositions,
- argument, ze ta kompozycja daje nowe wlasciwosci, a nie tylko dobra inzynierie.

### 13.5. Jak to zmienia ocene 3 kandydatow

Po uwzglednieniu transportu i P2P:

- kandydat 1 staje sie mocniejszy praktycznie,
  bo ma juz wokol siebie realny podpisowy i transportowy runtime
- kandydat 2 pozostaje glownie privacy-core i najmniej zalezy od transportu
- kandydat 3 staje sie bardziej wiarygodny,
  bo masz juz zalazek realnej kompozycji:
  - transport
  - gossip
  - state sync
  - QC
  - proof-aware consensus direction

Wniosek:
- `nxms-transport` i P2P nie przesuwaja projektu automatycznie do statusu "przelom kryptograficzny",
- ale bardzo istotnie przesuwaja go z poziomu:
  - "interesujacy pomysl"
  do poziomu:
  - "realny system z badawczo sensowna baza".

## 14. Finalna Ocena

Jesli zapisujemy to spojnie z obecnym stanem systemu, to:

- kandydat 1 jest najbardziej praktyczny i najblizszy wdrozenia,
- kandydat 2 jest najbardziej "privacy-core",
- kandydat 3 ma najwiekszy potencjal architektoniczno-kryptograficzny, ale tez najwieksze ryzyko przeszacowania.

To oznacza, ze `privAI` nie tylko "moze kiedys zrobic cos ciekawego", ale ma juz trzy sensowne, technicznie uzasadnione kierunki, ktore mozna rozwijac bez rozwalania obecnego systemu.
