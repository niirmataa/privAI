# privAI V0: Window-Based Metering Protocol — Detailed Design Questions

**Status:** protocol design — each decision fully described  
**Data:** 2026-04-12  
**Zakres:** każdy element window protocol opisany: co, do czego, po co, dlaczego, jak wykorzystane

---

## Model w jednym zdaniu

Sesja jest podzielona na N okien. Każde okno ma challenge, response, availability check, performance check. Płatność = `passed_windows / total_windows * amount`.

---

## 1. W1 — window_id format

**Co to jest:** Identyfikator pojedynczego okna w sesji. Każde okno w sesji ma unikalny ID.

**Do czego służy:** Pozwala odróżnić okna od siebie. Używany w hash'ach, w receipt, w dispute. Kiedy chain mówi "okno numer 500 failowało" — window_id mówi które.

**Po co jest potrzebne:** Bez window_id nie ma sposobu żeby powiązać challenge z response, receipt z oknami, dispute z konkretnymi momentami w sesji.

**Dlaczego to jest ważne:** Format window_id wpływa na prywatność. Jeśli window_id jest sequential (1, 2, 3...) — jest proste ale może ujawniać kolejność. Jeśli jest hash — jest trudniejsze do correlate ale complexity.

**Jak wykorzystane:** W challenge hash: `hash(session_id || window_id || block_hash)`. W receipt: jako index w liście window_hashes. W dispute: jako referencja do konkretnego okna.

**Decyzja:** Sequential (`u32: 1..N`). Session_id chroni prywatność — window_id nie musi być pseudolosowe.

---

## 2. W2 — window_duration

**Co to jest:** Czas trwania jednego okna. Czas między jednym challenge a następnym.

**Do czego służy:** Definiuje granularity meteringu. Krótsze okna = więcej challengi = lepsza statystyka ale więcej koszt. Dłuższe okna = mniej challengi = gorsza statystyka ale taniej.

**Po co jest potrzebne:** Bez window_duration nie wiadomo kiedy challenge jest generowany. Miner nie wie kiedy się przygotować. User nie wie kiedy sprawdzać.

**Dlaczego to jest ważne:** Za krótkie okna (np. 1s) = overkill, za dużo challengi, za drogo. Za długie okna (np. 1h) = luka między challengami jest duża, miner może oszukiwać przez większość czasu. 60s jest sweet spot — wystarczająco częste dla statystyki, wystarczająco tanie.

**Jak wykorzystane:** Lease policy deklaruje `window_duration` (np. 60 bloków ≈ 60 sekund). Session ma `total_windows = session_duration / window_duration`. Challenge jest generowany co `window_duration`.

**Decyzja:** Lease policy deklaruje. Default = 60 bloków. Stałe dla całej sesji.

---

## 3. W3 — contiguous vs overlapping windows

**Co to jest:** Czy okna następują po sobie (jedno kończy się, drugie zaczyna) czy nachodzą na siebie (część okna 1 jest jednocześnie częścią okna 2).

**Do czego służy:** Contiguous = proste, liniowe, łatwe do policzenia. Overlapping = lepsza coverage bo każdy moment czasu jest w wielu oknach jednocześnie.

**Po co jest potrzebne:** Bez contiguous/overlapping decyzji nie wiadomo jak liczyć passed_windows. Czy okno 1 i okno 2 mogą oba failować w tym samym momencie? Czy fail w jednym oknie wpływa na sąsiednie?

**Dlaczego to jest ważne:** Overlapping jest lepsze statystycznie (więcej samples per time unit) ale complexity jest większa (okna się nakładają, trudniej liczyć). Contiguous jest proste (każda sekunda jest w dokładnie jednym oknie).

**Jak wykorzystane:** Challenge jest generowany na początku każdego okna. Response jest oczekiwana przed końcem okna. W contiguous: okno N kończy się, okno N+1 zaczyna. W overlapping: okno N i N+1 mają wspólną część.

**Decyzja:** Contiguous na Phase 1. Overlapping future.

---

## 4. W4 — fixed-time vs event-driven windows

**Co to jest:** Czy okna zaczynają się co N bloków (fixed-time) czy na żądanie (event-driven).

**Do czego służy:** Fixed-time = deterministyczne, każdy wie kiedy następne okno. Event-driven = elastyczne, challenge jest generowany kiedy user chce.

**Po co jest potrzebne:** Bez tej decyzji nie wiadomo kiedy challenge jest generowany. Fixed-time jest predictable (miner może się przygotować). Event-driven jest unpredictable (miner nie wie kiedy nadejdzie).

**Dlaczego to jest ważne:** Fixed-time jest proste ale miner może "przygotować" zasób na moment challenga i oversubscribe resztę czasu. Event-driven jest lepsze bo nieprzewidywalne ale wymaga aktywnego uczestnictwa usera.

**Jak wykorzystane:** W fixed-time: `start_height_i = session_start + i * window_duration_blocks`. W event-driven: challenge jest generowany kiedy user wysyła request. Deterministic vs unpredictable.

**Decyzja:** Fixed-time na Phase 1 (deterministyczne, proste). Event-driven jako opcjonalny upgrade (user może dodawać extra challengi).

---

## 5. W5 — minimum windows

**Co to jest:** Minimalna liczba okien w sesji. Niżej nie można iść.

**Do czego służy:** Gwarantuje że statystyka ma sens. 1 okno = binary (all pass albo all fail). 100 okien = przyzwoita statystyka. 1440 = dobra statystyka.

**Po co jest potrzebne:** Bez minimum, user mógłby wynająć na 1 okno = jeden challenge = binary pass/fail = zero statystycznej wartości. Miner mógłby "przejść" jeden challenge i dostać pełną płatność.

**Dlaczego to jest ważne:** Statystyka wymaga wielu samples. Im więcej okien, tym dokładniejszy pomiar. Ale więcej okien = więcej koszt (challengi, storage, verification).

**Jak wykorzystane:** Lease policy deklaruje `total_windows`. Protocol sprawdza czy `total_windows >= minimum`. Minimum jest defined w protocol spec (np. 100 okien).

**Decyzja:** Minimum = 100 okien (statystycznie meaningful). Lease policy może być więcej.

---

## 6. C1 — challenge hash formula

**Co to jest:** Deterministyczna formuła która generuje hash challenga dla każdego okna.

**Do czego służy:** Każdy (user, miner, chain, third party) może odtworzyć ten sam hash i sprawdzić czy challenge jest prawidłowy. Hash jest inputem do computation challenge.

**Po co jest potrzebne:** Bez deterministycznego hasha, nie ma sposobu na weryfikację. Jeśli challenge jest random — nikt nie może sprawdzić czy challenge jest prawidłowy. Jeśli jest deterministyczny — każdy może odtworzyć.

**Dlaczego to jest ważne:** Hash musi zawierać nieprzewidywalny element (block_hash) żeby miner nie mógł pre-compute odpowiedzi przed blokiem. Ale musi być odtwarzalny po bloku bo chain musi weryfikować.

**Jak wykorzystane:** `challenge_hash_i = blake3("privai:window:v1" || session_id || i || block_hash(start_height_i))`. Miner oblicza hash po tym jak blok jest znany. Odpowiada na challenge. Chain odtwarza hash i weryfikuje response.

**Decyzja:** `blake3("privai:window:v1" || session_id || window_id || block_hash(start_height))`. Domain separator zapobiega collision z innymi hashami.

---

## 7. C2 — domain separator string

**Co to jest:** Stały string ("privai:window:v1") używany w hash'u challenga.

**Do czego służy:** Zapobiega collision między hashami window challenges a innymi hashami w systemie (note commitment, tx hash, itp.).

**Po co jest potrzebne:** Bez domain separator, `blake3(session_id || window_id || block_hash)` mógłby przypadkowo matchować hash z innego kontekstu. Domain separator gwarantuje że hash jest unique dla window protocol.

**Dlaczego to jest ważne:** Cryptographic best practice. Każda domena hashowania ma własny separator. V0 master używa "privai:policy:v0", "privai:note:v0", etc.

**Jak wykorzystane:** Jako pierwszy element w hash'u challenga. Każdy hash w systemie ma własny domain separator.

**Decyzja:** `"privai:window:v1"`. Versioned (v1) żeby móc zmienić formułę w przyszłości.

---

## 8. C3 — timeout_blocks

**Co to jest:** Maksymalna liczba bloków między startem okna a odpowiedzią minera. Jeśli miner nie odpowie w timeout — okno FAIL.

**Do czego służy:** Definiuje deadline na odpowiedź. Chroni przed "ghost miner" który nigdy nie odpowiada. Chroni przed wolnym łączem (jeśli timeout jest wystarczająco szeroki).

**Po co jest potrzebne:** Bez timeout, sesja mogłaby wisieć w nieskończoność czekając na odpowiedź minera. Timeout enforce'uje liveness.

**Dlaczego to jest ważne:** Za krótki timeout = miner na Tor nie zdąży odpowiedzieć (Tor ma 200-500ms na hop, 3+ hops = 1-3s round trip). Za długi timeout = luka między challengami jest duża, miner może oszukiwać.

**Jak wykorzystane:** Chain sprawdza: `response_time <= start_height + timeout_blocks`. Jeśli response jest po timeout — FAIL. Lease policy deklaruje timeout.

**Decyzja:** Lease policy deklaruje. Default zależy od klasy zasobu. DedicatedGpu: 10 bloków (≈10s). SharedGpu: 20 bloków (≈20s, więcej tolerance bo shared).

---

## 9. R1 — computation_result format

**Co to jest:** Wynik computation challenge który miner odsyła jako odpowiedź.

**Do czego służy:** Chain (lub user) weryfikuje czy wynik jest correct. Jeśli wynik jest correct — availability PASS. Jeśli wynik jest incorrect lub brak — FAIL.

**Po co jest potrzebne:** Bez computation_result, nie ma sposobu na weryfikację odpowiedzi. Sam podpis minera nie wystarczy — miner mógłby podpisać cokolwiek.

**Dlaczego to jest ważne:** Computation musi być deterministyczna — ten sam input (challenge_hash) → ten sam output. Każdy może obliczyć expected output i porównać. Jeśli output się nie zgadza — miner kłamie albo nie ma zasobu.

**Jak wykorzystane:** Miner oblicza `result = f(challenge_hash)` gdzie `f` jest deterministyczną funkcją (np. hash preimage search, computation puzzle). Chain odtwarza `expected = f(challenge_hash)` i porównuje `result == expected`.

**Decyzja:** Deterministyczna computation. Dokładna funkcja `f` jest defined w spec per klasę zasobu (GPU computation, CPU computation, memory access, etc.).

---

## 10. R2 — response_time measurement

**Co to jest:** Moment kiedy odpowiedź minera jest zarejestrowana. Używany do sprawdzenia czy odpowiedź była w timeout.

**Do czego służy:** Chain porównuje `response_time` z `start_height + timeout_blocks`. Jeśli response jest poza timeout — FAIL.

**Po co jest potrzebne:** Bez response_time, nie ma sposobu na sprawdzenie czy miner odpowiedział na czas. Sama obecność odpowiedzi nie wystarczy — odpowiedź mogła nadejść po timeout.

**Dlaczego to jest ważne:** Block_height jest deterministyczne (każdy node widzi tę samą wysokość). Timestamp nie jest (różne node'y mogą mieć różne zegary). Block_height jest lepsze.

**Jak wykorzystane:** Response zawiera `response_block_height`. Chain sprawdza `response_block_height <= start_height + timeout_blocks`.

**Decyzja:** Block_height (nie timestamp). Deterministyczne, verifiable, bez zaufania do zegara.

---

## 11. R3 — miner signature on response

**Co to jest:** Falcon podpis minera na response. Potwierdza że to miner wysłał odpowiedź, nie ktoś inny.

**Do czego służy:** Authenticity. Bez podpisu, attacker mógłby wysłać fałszywą odpowiedź w imieniu minera. Podpis gwarantuje że odpowiedź jest od minera.

**Po co jest potrzebne:** Bez podpisu, nie ma sposobu na powiązanie odpowiedzi z minerem. Ktokolwiek mógłby wysłać correct response i twierdzić że to od minera.

**Dlaczego to jest ważne:** Miner jest stroną kontraktu. Tylko miner może "dowodzić" że ma zasób. Inni nie mogą w jego imieniu.

**Jak wykorzystane:** Miner podpisuje `response_data` swoim Falcon SK. Chain weryfikuje podpis używając miner's Falcon PK (hash matchuje `miner_pk_hash` w SpendPolicy).

**Decyzja:** Falcon signature. Tak jak wszystko w V0 — post-kwantowe podpisy.

---

## 12. A1 — availability definition

**Co to jest:** Definicja "dostępny." Okno jest PASS (availability) jeśli miner odpowiedział na challenge w timeout.

**Do czego służy:** Binary decyzja: zasób jest dostępny albo nie. Nie ma pośredniego stanu.

**Po co jest potrzebne:** Bez definicji, nie wiadomo co znaczy "dostępny." Czy "dostępny" oznacza "odpowiedział na ping"? "VM działa"? "GPU jest wolny"? Musi być precyzyjna definicja.

**Dlaczego to jest ważne:** Definicja musi być mierzalna. "Odpowiedział na challenge w timeout" jest mierzalne. "GPU jest dostępny" nie jest mierzalne bez challenge.

**Jak wykorzystane:** `availability(window_i) = PASS IF response EXISTS AND response_time <= start_height + timeout_blocks`. Inaczej FAIL.

**Decyzja:** Binary (PASS/FAIL). Definicja = odpowiedź na challenge w timeout.

---

## 13. A2 — binary vs graded availability

**Co to jest:** Czy availability jest binary (PASS/FAIL) czy graded (np. 100%, 80%, 50%, 0%).

**Do czego służy:** Binary jest proste — albo jest, albo nie ma. Graded jest bardziej szczegółowe — "częściowo dostępny."

**Po co jest potrzebne:** W realnym świecie, zasób może być "częściowo dostępny" (np. GPU jest dostępny ale z opóźnieniem). Binary nie łapie tego.

**Dlaczego to jest ważne:** Graded jest bardziej fair ale complexity jest większa (ile poziomów? jak mierzyć?). Binary jest wystarczające na Phase 1.

**Jak wykorzystane:** W binary: `PASS` lub `FAIL`. W graded: `100%`, `75%`, `50%`, `25%`, `0%` na podstawie response time relative do timeout.

**Decyzja:** Binary na Phase 1. Graded future (jeśli potrzebne).

---

## 14. P1 — benchmark definition

**Co to jest:** Standardowa computation używana do mierzenia performance. Miner musi ją wykonać. Czas wykonania jest porównywany z benchmark_floor.

**Do czego służy:** Mierzy "jak szybki jest zasób." Nie "czy jest dostępny" (to jest availability) tylko "jak szybko działa."

**Po co jest potrzebne:** Availability mówi "GPU odpowiada." Performance mówi "GPU odpowiada szybko." Bez performance, miner mógłby oferować A100 ale używać T4 — oba "odpowiadają" ale T4 jest 10x wolniejszy.

**Dlaczego to jest ważne:** Benchmark musi być standardowy (ten sam dla wszystkich miners) żeby wyniki były porównywalne. Musi być relevant dla klasy zasobu (GPU benchmark dla GPU, CPU benchmark dla CPU).

**Jak wykorzystane:** Miner dostaje challenge + benchmark spec. Wykonuje computation. Response zawiera computation time. Chain/user porównuje time z benchmark_floor.

**Decyzja:** Standardowy benchmark per klasę zasobu. Definiowany w spec, nie w lease policy (spójność między miners).

---

## 15. P2 — benchmark_floor per class

**Co to jest:** Maksymalny dopuszczalny czas na benchmark computation. Jeśli computation_time > floor — performance FAIL.

**Do czego służy:** Definiuje "wystarczająco szybki." Floor jest threshold między PASS i FAIL.

**Po co jest potrzebne:** Bez floor, nie ma sposobu na ocenę performance. "Szybki" jest subiektywne. Floor jest obiektywny.

**Dlaczego to jest ważne:** Floor musi być dostosowany do klasy zasobu. A100 floor jest inny niż T4 floor. Za niski floor = false FAIL na dobrym GPU. Za wysoki floor = false PASS na wolnym GPU.

**Jak wykorzystane:** Lease policy deklaruje `benchmark_floor`. Chain/user porównuje: `computation_time <= benchmark_floor → PASS`.

**Decyzja:** Lease policy deklaruje per klasę zasobu. Protocol definiuje default floors.

---

## 16. P3 — benchmark frequency

**Co to jest:** Jak często benchmark jest wykonywany. Co okno? Co 10 okien? Co 100 okien?

**Do czego służy:** Benchmark jest droższy niż availability check. Frequency kontroluje koszt vs precision.

**Po co jest potrzebne:** Benchmark w każdym oknie = drogo (każde okno wymaga computation). Benchmark co 100 okien = tanio ale miner może oszukiwać między benchmarkami.

**Dlaczego to jest ważone:** Trade-off: więcej benchmarków = lepsza statystyka ale więcej koszt. Mniej benchmarków = tańsze ale gorsza statystyka.

**Jak wykorzystane:** Lease policy deklaruje `benchmark_interval` (np. co 10 okien). Okna bez benchmarku mają `performance = N/A`. N/A jest traktowane jak PASS.

**Decyzja:** Lease policy deklaruje. Default = co 10 okien.

---

## 17. X1 — receipt bilateral vs unilateral

**Co to jest:** Kto tworzy receipt — tylko miner (unilateral) czy oboje (bilateral).

**Do czego służy:** Unilateral: miner tworzy receipt + ZK proof. User weryfikuje. Bilateral: oboje tworzą i podpisują.

**Po co jest potrzebne:** Unilateral jest prostsze (jeden receipt, jeden ZK proof). Bilateral jest bardziej robust (dwa receipts, trudniej oszukiwać).

**Dlaczego to jest ważne:** W unilateral, user musi zaufać że miner's receipt jest prawdziwy (ZK proof to potwierdza). W bilateral, user ma własny receipt — dispute jest prostszy.

**Jak wykorzystane:** Unilateral: miner → receipt + ZK proof → user verify → settlement. Bilateral: miner → receipt + ZK proof, user → receipt → compare → settlement lub dispute.

**Decyzja:** Unilateral (miner tworzy + ZK proof) na Phase 1. User ma swoje dane ale nie tworzy formalnego receipt. Bilateral future.

---

## 18. X2 — degraded_windows weight

**Co to jest:** Waga okien które przeszły availability ale nie przeszły performance. Np. weight = 0.5 oznacza że degraded okno liczy się jako 50% passed.

**Do czego służy:** Rozróżnia "dostępny ale wolny" od "niedostępny." Degraded nie jest FAIL ale nie jest PASS.

**Po co jest potrzebne:** Bez degraded, każde okno jest binary (PASS/FAIL). Jeśli miner jest dostępny ale wolny — to nie jest FAIL ale nie jest pełny PASS. Degraded łapie ten stan.

**Dlaczego to jest ważne:** Za niska waga (0%) = degraded jest traktowane jak FAIL = nie fair dla minera (bo był dostępny). Za wysoka waga (100%) = degraded jest traktowane jak PASS = nie fair dla usera (bo performance była słaba). 50% jest rozsądnym default.

**Jak wykorzystane:** `effective_windows = passed + (degraded * weight)`. Lease policy deklaruje weight. Default = 0.5.

**Decyzja:** Lease policy deklaruje. Default = 0.5 (50%).

---

## 19. X3 — merkle root vs full hashes

**Co to jest:** Czy receipt zawiera full listę hash'ów każdego okna czy tylko merkle root z tych hash'ów.

**Do czego służy:** Full hashes = każdy hash jest w receipcie (duży rozmiar). Merkle root = tylko root jest w receipcie, full hashes są off-chain (mały rozmiar, ale potrzeba proof dla dispute).

**Po co jest potrzebne:** Receipt musi zawierać dowód że window hashes są konsystentne. Full hashes są proste ale duże. Merkle root jest małe ale wymaga proof w dispute.

**Dlaczego to jest ważne:** 1440 okien × 32 bajty/hash = 46KB w receipcie. Merkle root = 32 bajty. Merkle jest 1400x mniejsze. Ale dispute wymaga merkle proof.

**Jak wykorzystane:** Receipt zawiera merkle_root. W dispute, miner dostarcza merkle proof per okno. Chain weryfikuje proof against root.

**Decyzja:** Merkle root w receipcie. Full hashes off-chain. Merkle proof w dispute.

---

## 20. X4 — ZK proof scope

**Co to jest:** Co ZK proof dowodzi — aggregate (suma passed jest prawdziwa) czy per-window (każde okno jest PASS/FAIL jak twierdzę).

**Do czego służy:** Aggregate proof jest mniejszy (jeden proof na sesję). Per-window proof jest większy (N proofs na sesję) ale bardziej detailed.

**Po co jest potrzebne:** Bez ZK proof, miner mógłby sfabrykować receipt. ZK proof potwierdza że receipt jest konsystentny z telemetry.

**Dlaczego to jest ważne:** Aggregate proof wystarcza do normalnego settlement (user nie kwestionuje). Per-window proof jest needed w dispute (user kwestionuje konretne okna).

**Jak wykorzystane:** Na receipcie: aggregate ZK proof (potwierdza sumę). W dispute: per-window ZK proof (potwierdza każde okno). Dwa różne proofy, dwa różne użycia.

**Decyzja:** Aggregate proof na receipcie (zawsze). Per-window proof w dispute (na żądanie).

---

## 21. S1 — settlement formula

**Co to jest:** Formuła obliczająca płatność na podstawie passed windows.

**Do czego służy:** Mapuje "ile okien przeszło" na "ile PVA dostaje miner."

**Po co jest potrzebne:** Bez formuły, nie wiadomo jak dzielić pieniądze. All-or-nothing jest za gruboziarniste. Pro-rata jest fair.

**Dlaczego to jest ważne:** Integer division jest required (bez floatów). Remainder idzie do usera. Degraded windows mają wagę.

**Jak wykorzystane:** `effective = passed + (degraded * weight)`. `miner_share = amount * effective / total`. `user_share = amount - miner_share`.

**Decyzja:** `miner_share = amount * (passed + degraded * weight) / total`. Integer division. Remainder to user.

---

## 22. S2 — degraded_weight default

**Co to jest:** Domyślna waga dla degraded windows jeśli lease policy nie deklaruje innej.

**Do czego służy:** Fallback. Jeśli lease policy nie specifies wagę — default jest używany.

**Po co jest potrzebne:** Lease policy może nie deklarować wagi. Default musi być fair.

**Dlaczego to jest ważne:** Default musi być rozsądny. 0% = degraded jak FAIL. 100% = degraded jak PASS. 50% = balanced.

**Jak wykorzystane:** Jeśli lease policy.degraded_weight jest None — użyj default (0.5).

**Decyzja:** Default = 0.5 (50%).

---

## 23. S3 — remainder policy

**Co to jest:** Co się dzieje z resztą z integer division. Jeśli `amount * effective / total` nie dzieli się równo — reszta idzie do kogo?

**Do czego służy:** Gwarantuje że `miner_share + user_share = amount`. Reszta nie jest tracona.

**Po co jest potrzebne:** Bez remainder policy, `miner_share + user_share` mógłby nie być równy `amount` (błąd zaokrąglenia).

**Dlaczego to jest ważne:** Reszta jest zawsze < 1 PVA (bo integer division). Ale musi gdzieś iść. User jest naturalnym beneficjentem bo user płaci — reszta wraca do płacącego.

**Jak wykorzystane:** `user_share = amount - miner_share`. Reszta jest implicitnie w user_share.

**Decyzja:** Reszta zawsze do usera.

---

## 24. Z1 — ZK proof what it proves

**Co to jest:** Konkretna rzecz którą ZK proof potwierdza.

**Do czego służy:** Chain weryfikuje proof. Jeśli proof jest valid — chain wie że miner nie kłamie w receipcie.

**Po co jest potrzebne:** Bez ZK proof, miner mógłby sfabrykować receipt ("twierdzę że 1368 passed"). ZK proof potwierdza "moja telemetry jest konsystentna z 1368 passed."

**Dlaczego to jest ważne:** ZK proof musi być precyzyjny. "Telemetry jest konsystentna z receipt" — nie więcej, nie mniej.

**Jak wykorzystane:** Chain sprawdza ZK proof. Jeśli valid — receipt jest accepted. Jeśli invalid — receipt jest rejected.

**Decyzja:** ZK proof dowodzi: "moje prywatne pomiary telemetry są konsystentne z twierdzeniami w receipcie (total, passed, degraded, failed)."

---

## 25. Z2 — ZK proof what it hides

**Co to jest:** Co ZK proof NIE ujawnia.

**Do czego służy:** Chroni prywatność minera. Chain i user widzą tylko PASS/FAIL — nie dokładne pomiary.

**Po co jest potrzebne:** Bez ZK, chain musiałby widzieć pełną telemetry żeby zweryfikować receipt. To ujawnia prywatne dane minera (obciążenie, inni userzy, system metrics).

**Dlaczego to jest ważne:** Prywatność minera jest core value V0. Miner nie chce ujawniać jak ma zajęty GPU, ilu userów obsługuje, jakie ma obciążenie.

**Jak wykorzystane:** ZK proof nie ujawnia: dokładnych pomiarów, obciążenia GPU, innych userów, telemetrii systemowej, memory usage, network traffic.

**Decyzja:** ZK proof ukrywa: dokładne pomiary, obciążenie, inni userzy, telemetria systemowa.

---

## 26. Z3 — ZK proof when required

**Co to jest:** Kiedy ZK proof jest wymagany — zawsze czy tylko w dispute.

**Do czego służy:** Aggregate proof na receipcie = zawsze. Per-window proof = tylko w dispute.

**Po co jest potrzebne:** Aggregate proof na receipcie gwarantuje że receipt nie jest sfabrykowany. Per-window proof w dispute gwarantuje że konkretne okna są prawdziwe.

**Dlaczego to jest ważne:** Aggregate proof jest tani (jeden proof na sesję). Per-window proof jest drogi (N proofs). Per-window tylko w dispute = koszt jest ponoszony tylko kiedy jest potrzebny.

**Jak wykorzystane:** Receipt: aggregate ZK proof (always). Dispute: per-window ZK proof (on demand).

**Decyzja:** Aggregate proof na receipcie (zawsze). Per-window proof w dispute (na żądanie).

---

## 27. Z4 — ZK proof on-chain vs off-chain

**Co to jest:** Czy ZK proof jest na chainie czy off-chain z commitment.

**Do czego służy:** On-chain = chain weryfikuje bezpośrednio (drogie ale trustless). Off-chain = hash na chainie, full proof na żądanie (tańsze).

**Po co jest potrzebne:** ZK proof verification na chainie jest kosztowne (computation). Jeśli jest na chainie — każdy blok z receipt jest droższy. Jeśli jest off-chain — verification jest tylko w dispute.

**Dlaczego to jest ważne:** Cost. On-chain verification = higher tx fees. Off-chain = lower fees but requires availability of proof data.

**Jak wykorzystane:** Receipt na chainie zawiera hash ZK proof. Full proof jest off-chain. W dispute, proof jest submitted on-chain i weryfikowany.

**Decyzja:** Off-chain. Hash na chainie. Full proof na żądanie (w dispute).

---

## 28. D1 — who can initiate dispute

**Co to jest:** Kto może kwestionować receipt — user, miner, czy obaj.

**Do czego służy:** User mówi "za mało passed." Miner mówi "za mało zapłacone." Oboje mogą mieć rację.

**Po co jest potrzebne:** Bez dispute, receipt jest ostateczny nawet jeśli jest nieprawdziwy. Dispute jest safety valve.

**Dlaczego to jest ważne:** User może chcieć kwestionować bo jego dane pokazują mniej passed. Miner może chcieć kwestionować bo jego ZK proof pokazuje więcej passed niż user zaakceptował.

**Jak wykorzystane:** User submituje dispute z powodem. Albo miner submituje dispute z counter-receipt. Chain rozstrzyga.

**Decyzja:** Obaj mogą zainicjować. User kwestionuje receipt. Miner kwestionuje user's settlement claim.

---

## 29. D2 — dispute fee

**Co to jest:** Koszt zainicjowania dispute. Loser płaci.

**Do czego służy:** Disincentive do fałszywych disputes. Bez fee, każdy mógłby kwestionować każdy receipt za darmo.

**Po co jest potrzebne:** Bez fee, user mógłby kwestionować każdy receipt "na próbę" — jeśli nie ma racji, nic nie traci. Fee gwarantuje że dispute jest poważny.

**Dlaczego to jest ważne:** Za niski fee = za dużo disputes. Za wysoki fee = prawdziwe disputes są zniechęcone. Musi być proporcjonalny do kwoty lease.

**Jak wykorzystane:** Dispute fee jest locked na chainie przy dispute initiation. Loser traci fee. Winner odzyskuje fee.

**Decyzja:** Stały na Phase 1 (np. 0.1 PVA). Proporcjonalny future (np. 1% lease amount).

---

## 30. D3 — dispute timeout

**Co to jest:** Ile bloków ma user/miner na zainicjowanie dispute po otrzymaniu receipt.

**Do czego służy:** Gwarantuje że settlement nie wisi w nieskończoność. Po timeout — receipt jest automatycznie akceptowany.

**Po co jest potrzebne:** Bez timeout, user mógłby "wstrzymywać" settlement kwestionując receipt na zawsze. Timeout enforce'uje finalność.

**Dlaczego to jest ważne:** Za krótki timeout = user nie ma czasu na analizę. Za długi timeout = settlement jest delayed.

**Jak wykorzystane:** Chain sprawdza: `current_block <= receipt_block + dispute_timeout`. Jeśli minął — receipt jest accepted automatycznie.

**Decyzja:** Lease policy deklaruje. Default = 100 bloków (≈100 sekund).

---

## 31. D4 — loser pays

**Co to jest:** Reguła że przegrywający dispute płaci fee.

**Do czego służy:** Economic incentive do uczciwych disputes. Jeśli masz rację — nie płacisz. Jeśli nie masz — płacisz.

**Po co jest potrzebne:** Bez tej reguły, fałszywe disputes nie mają kosztu.

**Dlaczego to jest ważne:** To jest standardowy mechanism w systemach prawnych (kto przegrywa, płaci koszty). Disincentive do frivolous litigation.

**Jak wykorzystane:** Chain porównuje receipt z window-by-window data. Jeśli receipt jest prawdziwy → user (disputant) płaci fee. Jeśli receipt jest fałszywy → miner płaci fee.

**Decyzja:** Loser pays. Tak.

---

## 32. E1 — N = 0 (session never started)

**Co to jest:** Edge case: sesja miała N okien ale żadno się nie zaczęło (miner nigdy nie odpowiedział na pierwszy challenge).

**Jak rozwiązać:** Full refund. Timeout po lease policy timeout_blocks. Jeśli po timeout zero okien przeszło → full refund do usera.

**Decyzja:** Full refund po timeout jeśli total_windows_completed = 0.

---

## 33. E2 — all FAIL

**Co to jest:** Edge case: wszystkie okna failowały. Miner nigdy nie dostarczył zasobu.

**Jak rozwiązać:** Full refund. Zero passed = zero payment.

**Decyzja:** Full refund.

---

## 34. E3 — no response to dispute

**Co to jest:** Edge case: user kwestionuje receipt ale miner nie odpowiada na dispute.

**Jak rozwiązać:** Po dispute timeout — chain akceptuje user's data (jeśli user dostarczył) albo defaultuje do refund.

**Decyzja:** Dispute timeout → user's data wins jeśli miner nie odpowiada.

---

## 35. E4 — no response to receipt

**Co to jest:** Edge case: miner dostarcza receipt ale user nie odpowiada (ani accept, ani dispute).

**Jak rozwiązać:** Po dispute timeout — chain automatycznie akceptuje receipt.

**Decyzja:** Timeout → receipt auto-accepted.

---

## 36. E5 — chain reorg

**Co to jest:** Edge case: blok znika (reorg). Challenge zależał od block_hash który zniknął.

**Jak rozwiązać:** Challenge jest deterministyczny z nowym block_hash po reorg. Miner musi odpowiedzieć na nowy challenge. Jeśli nie zdąży — FAIL.

**Decyzja:** Reorg = nowy challenge. Miner musi odpowiedzieć na nowy.

---

## Podsumowanie: 36 decyzji do podjęcia

```
WINDOW STRUCTURE:     W1, W2, W3, W4, W5         (5)
CHALLENGE:            C1, C2, C3                   (3)
RESPONSE:             R1, R2, R3                   (3)
AVAILABILITY:         A1, A2                        (2)
PERFORMANCE:          P1, P2, P3                    (3)
RECEIPT:              X1, X2, X3, X4               (4)
SETTLEMENT:           S1, S2, S3                    (3)
ZK PROOF:             Z1, Z2, Z3, Z4               (4)
DISPUTE:              D1, D2, D3, D4                (4)
EDGE CASES:           E1, E2, E3, E4, E5           (5)
---
RAZEM:                                               36
```

Każda decyzja jest opisana: co, do czego, po co, dlaczego, jak wykorzystane. To jest scope window protocol spec.

---

**Czy edytowano pliki:** NIE (tylko ten plik)  
**Czy czytano legacy docs:** NIE  
**Czy zdefiniowano wire formaty:** NIE
