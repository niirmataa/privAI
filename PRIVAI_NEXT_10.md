# privAI - najblizsze 10 duzych punktow

Ten dokument jest naszym najblizszym planem wykonawczym po domknieciu:
- wallet hidden notes
- transfer builder + proving data
- batch execution bundle
- sidecar proof artifacts
- sidecar verifier interface
- end-to-end flow `wallet -> tx -> proof input -> block`

Kazdy punkt ma checkbox, opis celu i konkretne kryterium zamkniecia.
Po zakonczeniu zadania odhaczamy je tutaj i dopisujemy krotki wynik prac.

## 1. [ ] Podpiac realny backend ZK do `BatchProofVerifierBackend`
Opis:
Obecny `ProofEnvelopeVerifier` sprawdza envelope i spojnosci sidecara, ale nie weryfikuje jeszcze prawdziwego proofa ZK. Trzeba podpiac pierwszy konkretny backend verifiera pod `privai-proof`.

Done:
- wybrany i zamrozony proof system dla v0
- `proof_bytes` maja jawny format binarny
- `BatchProofVerifierBackend` robi prawdziwe verify, nie tylko envelope checks
- testy pozytywne i negatywne przechodza w Alpine

## 2. [ ] Zamrozic format `ProofArtifact` i `proof_meta_hash`
Opis:
Trzeba jednoznacznie opisac, co siedzi w `proof_bytes`, co w `proof_meta_hash`, jak wersjonujemy proof system i jak wyglada canonical hash dla artefaktow batch proof.

Done:
- spisany spec formatu
- canonical encoding i hash helpers w kodzie
- kompatybilne testy roundtrip i mismatch
- dokumentacja z przykladem artefaktu

## 3. [ ] Postawic `privai-prover` i lifecycle proof joba
Opis:
Potrzebujemy osobnego procesu/uslugi, ktora bierze `ProofJob`, pobiera witness/public inputs, generuje artefakt i oddaje go do node/proposer flow.

Done:
- nowy crate `privai-prover`
- intake `ProofJob`
- storage jobow i wynikow
- flow `requested -> running -> completed -> rejected`
- test integracyjny z `privai-nxms`

## 4. [ ] Rozszerzyc proof layer poza `TransferNoteTx`
Opis:
Dzisiaj proof flow jest sensownie zrobiony dla transfer note. Trzeba dodac support dla kolejnych typow tx, ktore blockchain i marketplace beda realnie potrzebowaly.

Zakres minimalny:
- settlement tx
- marketplace/model tx
- stake tx

Done:
- public inputs builder dla nowych tx
- statement builder dla nowych tx
- batch bundle obsluguje te typy bez placeholder errors
- testy dla mieszanego bloku

## 5. [ ] Dodac state commitments i membership proofs dla note setu
Opis:
Ledger trzyma teraz stan wygodnie do developmentu, ale do prawdziwych proofow potrzebujemy commitment do stanu i membership/non-membership path dla note/nullifier state.

Done:
- wybrana struktura commitmentu stanu
- note tree / accumulator
- nullifier set commitment
- witness path builder dla wallet/prover
- testy update po bloku

## 6. [ ] Dopiac wallet persistence i recovery flow
Opis:
Wallet dziala, ale trzeba domknac twarde rzeczy produkcyjne: lepszy store, recovery, rotacja bundle, lokalna higiena kluczy i bezpieczne odtwarzanie stanu po restarcie.

Done:
- filesystem wallet store jest stabilny
- recovery z plikow/snapshotu dziala
- bundle rotation i revocation maja testy
- note scanning po restarcie odtwarza stan poprawnie

## 7. [ ] Dopic consensus path do poziomu dzialajacego PC-BFT skeleton
Opis:
Node ma proposer scaffold, ale brakuje pelnego flow quorum/finality. Trzeba domknac minimalny autorski consensus skeleton, zeby blok mial nie tylko proposer path, ale tez realny commit path.

Done:
- prevote / precommit / commit messages
- QC / receipt flow
- import i finality path
- testy wielo-node na Alpine

## 8. [x] Zrobic marketplace v0 nad `PRIVAI/1`
Opis:
Skoro marketplace jest sercem projektu, trzeba postawic pierwszy kompletny flow biznesowy: oferta modelu, request, response, escrow intent i settlement.

Done:
- `MarketOffer`, `MarketAccept`, `InferenceRequest`, `InferenceResponse` dodane nowe struktury z SmallPayments.
- escrow intent/spend policy flow (ServicePaymentPolicy i SpendGrant przesylany miedzy nodami)
- prosty lifecycle sesji user <-> provider z wplata poczatkowa i odsyłaniem "Receiptów" po usłudze
- test integracyjny end-to-end na typach w module `privai-nxms/src/lib.rs` (przeszedl pomyslnie)

## 9. [ ] Wprowadzic threshold/LWE cold path
Opis:
Shamir i operacje na network key nie sa hot path, ale trzeba przygotowac architekture i pierwszy kod pod rebalancing / key refresh / DKG-side operations.

Done:
- spec cold path
- crate/module boundary dla threshold ops
- request/response flow dla rebalancing
- test harness dla partial decrypt / reshare API

## 10. [ ] Zrobic hardening, benchmarki i checklisty do wewnetrznego testnetu
Opis:
Zanim wejdziemy dalej, potrzebujemy twardych metryk i checklist. Tu wchodzi performance, storage, rozmiary proofow, limity i lista rzeczy do audytu.

Done:
- benchmarki wallet/proof/node
- rozmiary blokow i artefaktow zebrane w jednym miejscu
- checklisty audytowe dla proof/node/wallet
- dokument "ready for internal testnet"

## Zasady pracy przy odhaczaniu
- Odhaczamy dopiero wtedy, gdy kod + testy + krotki opis wyniku sa gotowe.
- Po kazdym zamknietym punkcie dopisujemy pod nim 2-5 zdan: co zrobione, jakie pliki ruszone, jakie testy przeszly.
- Jezeli punkt rozbijamy na podetapy, robimy to w osobnym dokumencie roboczym, ale ten plik zostaje glowna lista postepu.
