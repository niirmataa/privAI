# privAI Threshold Auth Canonical Rules

Status: focused support doc for threshold auth execution rules in privAI.
Canonicality: supporting auth-execution document. This document does not override canonical protocol, formats, consensus or product semantics; it defines execution-ready rules for threshold auth package validation, building on the signing model from `PRIVAI_AUTH_SIGNING_MODEL.md`.
Owner: privAI tx/auth, ledger and escrow architecture.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_EXECUTION_SPINE.md`
- `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`

## 1. Cel

Zdefiniowanie waskich, wykonywalnych regul (execution-ready rules) dla autoryzacji progowej (threshold auth) w fazie auth/signing (zgodnie z Execution Spine). Dokument precyzuje semantyke walidacji pakietow autoryzacyjnych, bez powielania logiki modelu biznesowego (np. escrow), bez definiowania formatow binarnych i bez mieszania pojec z zakresu dowodow (proof semantics).

Scope exclusion:
- `MarketplaceBatchTx` auth pozostaje poza tym modelem threshold auth, chyba ze zostanie jawnie migrowany do niego w przyszlosci.

## 2. Current Direction

Podejscie rozdziela czysta weryfikacje spelnienia wymogow kryptograficznych tozsamosci od logiki biznesowej transakcji. Pakiet autoryzacyjny (`auth package`) stanowi niezalezna, wyizolowana warstwe kontroli wejsciowej, ewaluowana scisle i deterministycznie.

---

## Canonical Rules

### 1. Canonical Signer Set Semantics
- **Niezmiennosc:** Zbior autoryzowanych sygnatariuszy (Signer Set) dla danej operacji (okreslony przez policy) jest zamkniety i zdefiniowany a priori.
- **Brak rozszerzen:** W trakcie ewaluacji pakietu nie ma mozliwosci dynamicznego dolaczania nowych sygnatariuszy ani modyfikacji ich wag/rol.

### 2. Canonical Signer Identity Binding
- **Scisle powiazanie:** Kazdy podpis musi kryptograficznie wiazac tozsamosc sygnatariusza (klucz publiczny) z `tx_signing_hash` zdefiniowanym w `PRIVAI_AUTH_SIGNING_MODEL.md`.
- **Zasada Expectation-First:** System najpierw identyfikuje podpisy na podstawie zadeklarowanego identyfikatora klucza, a nastepnie weryfikuje ich poprawnosc. Podpisy pochodzace od tozsamosci spoza zbioru (Signer Set) sa odrzucane (status `UnknownSigner`).
- **Twarda zasada:** auth package podpisuje `tx_signing_hash`, nie dowolny payload digest. To jest centralny kontrakt auth modelu.

### 3. Canonical Signer Ordering
- **Porzadek wg canonical signer index in policy:** Pakiety autoryzacyjne musza zawierac podpisy scisle posortowane wedlug canonical signer index zdefiniowanego w zrekonstruowanej policy. Porzadek wynika z semantyki ról i pozycji w policy, nie z surowego sortowania bajtow klucza.
- **Hard Reject:** Jakiekolwiek odchylenie od poprawnego porzadku skutkuje natychmiastowym odrzuceniem pakietu jako `MalformedAuthPackage`. Regula ta chroni przed problemem roznej reprezentacji tego samego stanu i uniemozliwia trywialne ataki malleability.

### 4. Duplicate Signer Rejection
- **Zero tolerancji dla duplikatow:** W pakiecie moze pojawic sie maksymalnie jedna reprezentacja podpisu od danej tozsamosci (klucza publicznego).
- **Hard Reject:** Wykrycie wiecej niz jednego podpisu przypisanego do tego samego sygnatariusza — nawet jesli oba sa kryptograficznie poprawne — powoduje blad `DuplicateSignerError` i odrzucenie calego pakietu.

### 5. Threshold Satisfaction Rule
- **Wymog minimum:** Autoryzacja jest poprawna wylacznie wtedy, gdy liczba udokumentowanych i matematycznie zweryfikowanych podpisow $M$ spelnia warunek $M \ge T$, gdzie $T$ to wymagany prog zdefiniowany przez policy.
- **Brak wartosci nadmiarowej:** Dodatkowe podpisy powiazane z poprawnymi sygnatariuszami w zbiorze, o ile nie lamia reguly duplikacji ani porzadku, sa dopuszczalne i prowadza do spelnienia progu. Autoryzacja ocenia jednak tylko zero-jedynkowe spelnienie warunku $M \ge T$.

### 6. Role-bound Signer Semantics (jesli policy tego wymaga)
- **Podzialy progu (Sub-thresholds):** Jesli policy definiuje specyficzne role operacyjne (np. "wymagany podpis arbitra"), zbior sygnatariuszy dzieli sie na podzbiory na podstawie rol.
- **Ocena logiczna AND:** Zespol Threshold Satisfaction jest oceniany niezaleznie dla kazdego podzbioru. Pakiet jest wazny tylko, jesli progi wszystkich wymaganych rol zostana zaspokojone rownoczesnie. Brak wsparcia dla logiki rozmytej (OR) na poziomie pojedynczego polityki wykonawczej w fazie auth.

### 7. Auth Package Semantics
- **Granice izolacji:** `Auth package` to struktura-koperta (envelope). Zamyka sie na zdefiniowaniu: identyfikatora policy, kanonicznego sortowanego wektora podpisow oraz `tx_signing_hash` jako autoryzowanego digestu.
- **Separacja od Payloadu:** Weryfikator pakietu autoryzacyjnego *nie deserializuje* ladunku ani jego proofow; zajmuje sie wylacznie sprawdzeniem, czy `tx_signing_hash` zostal prawidlowo podpisany zgodnie z regulami progu.

### 7a. Policy Reconstruction and `policy_opening`
- **Obowiazkowa rekonstrukcja:** Ledger rekonstruuje policy z canonical `policy_opening` material dostarczonego wraz z transakcja.
- **Zrodlo threshold i signer set:** Threshold, signer set i role constraints sa sprawdzane wylacznie wzgledem zrekonstruowanej policy — nie wzgledem zadeklarowanych wartosci z zewnatrz.
- **Twarda zasada:** threshold auth package nie istnieje "sam z siebie" — jest walidowany wzgledem reconstructed signer set i threshold rule z policy. Bez poprawnego `policy_opening` pakiet autoryzacyjny nie moze byc zweryfikowany.

### 8. Ledger Verification Rules
- **Pre-kondycja wlaczenia do ksiegi:** Transakcje moga trafic na ledger jedynie wtedy, gdy posiadaja zweryfikowany i matematycznie zatwierdzony w fazie signing `auth package`.
- **Zasada Stateless Auth Eval:** Wezel ledger sprawdza pakiet statycznie bez polegania na zewnetrznym stanie w czasie wykonywania. Ewaluacja podpisu musi dac deterministyczny wynik `Valid`/`Invalid`.

### 9. Nexum-Core vs PrivAI Split
- **PrivAI:** Implementuje cala "inteligentna" logike operacyjna. Odpowiada za: wydobycie rol, liczenie sygnatariuszy (threshold), sprawdzanie canonical signer ordering wg policy index, wychwytywanie duplikatow, rekonstrukcje policy z `policy_opening` i konstrukcje/dekodowanie `auth package`.
- **Nexum-Core:** Funkcjonuje jako "slepy" prymityw matematyczny. Wystawia wylacznie API typu `Verify(PublicKey, Hash, Signature) -> bool`. Nexum-Core nie jest swiadome pojec takich jak progi, role, sortowanie czy polityki transakcyjne.

---

## Checklist
- [ ] Implementacja walidacji policy reconstruction z `policy_opening`.
- [ ] Implementacja threshold evaluation wzgledem zrekonstruowanej policy.
- [ ] Implementacja canonical signer ordering wg signer index in policy.
- [ ] Implementacja deterministycznego odrzucania duplikatow sygnatariuszy (`DuplicateSignerError`).
- [ ] Implementacja hard reject dla blednego porzadku (`MalformedAuthPackage`).
- [ ] Implementacja flat threshold (M-of-N) i role-bound threshold (np. 1ofN + Arbiter).
- [ ] Implementacja izolacji auth package od proof semantics ladunku.
- [ ] Integracja z Nexum-Core przez waskie API `Verify(PublicKey, Hash, Signature) -> bool`.
- [ ] Testy regresyjne potwierdzajace, ze auth package binds to `tx_signing_hash`.

## Exit Criteria
- System testow niezawodnie odrzuca pakiety autoryzacyjne, ktore nie zachowuja canonical signer ordering wg policy index.
- Kazda proba umieszczenia w jednym pakiecie zduplikowanego podpisu od tej samej tozsamosci skutkuje bledem `DuplicateSignerError` i odrzuceniem calego pakietu.
- System poprawnie ewaluuje minimum 1 polityke flat (M-of-N) oraz 1 polityke role-bound, sprawdzajac prog czesciowy dla rol.
- Kod implementujacy logike progowa jest w pelni odizolowany od kryptograficznych procedur Nexum-Core.
- Auth package jest weryfikowany wylacznie wzgledem policy zrekonstruowanej z `policy_opening`, nie wzgledem zadeklarowanych wartosci.