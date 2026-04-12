# PrivAI — Simple 1: Czym jest ten system

**Opis możliwości, funkcje, problemy, zależności**

---

## Możliwości

PrivAI pozwala Ci wynająć prywatny komputer (GPU, CPU, RAM, dysk) od kogoś kto go ma. Płacisz kryptowalutą PVA. Nikt nie widzi co robisz na tym komputerze. Nikt nie wie kim jesteś. Nikt nie widzi ile płacisz ani komu.

### Co możesz robić

- Wynająć GPU do odpalenia modelu AI
- Wynająć CPU do obliczeń
- Wynająć dysk do przechowywania danych
- Wynająć VM z dostępem do internetu
- Wynająć VM bez dostępu do internetu (czysto prywatne)
- Wszystko anonimowo

### Co możesz wybrać

- Klasę zasobu: GPU A100, H100, T4, CPU, RAM
- Prywatność: VM (pełna izolacja), container, sandbox
- Sieć: odcięty od internetu, tylko privAI, przez Tor, z internetem
- Czas: minuta do miesięcy
- Cenę: jaka jest akceptowalna

---

## Funkcje systemu

### Szukanie

Kiedy szukasz compute, wysyłasz zaszyfrowane pytanie: "Szukam A100 za max 2 PVA/h." Pytanie idzie przez skrzynkę pocztową (NXMS mailbox). Skrzynka nie widzi treści. Minerowie odbierają pytania. Kto ma pasujący zasób — odpowiada. Kto nie ma — ignoruje. Nikt nie widział co szukałeś.

### Blokada pieniędzy

Kiedy uzgodnisz warunki z minerem, blokujesz pieniądze na chainie. Chain to zegar księgowy — zapisuje że pieniądze są zablokowane, na jakich warunkach (hash), do kiedy (timeout). Nie zapisuje komu płacisz ani za co. Pieniądze są zamrożone — nie możesz ich ruszyć dopóki sesja się nie skończy.

### Sesja

Miner odpala dla Ciebie prywatny komputer (VM/container). Łączysz się z nim przez zaszyfrowany tunel. Robisz co chcesz. Nikt nie widzi: Twoich promptów, Twoich outputów, Twoich danych, Twojego modelu.

### Sprawdzanie

Co pół godziny (60 bloków), miner dostaje pytanie: "Czy masz zasób?" Musi odpowiedzieć na czas. Jeśli odpowiedział — okno PASS. Jeśli nie — okno FAIL. Dodatkowo raz na jakiś czas: "Jak szybki jesteś?" Jeśli za wolny — FAIL. To jest wykonywane off-chain, między Tobą a minerem. Chain tego nie widzi.

### Dowód (receipt)

Po sesji miner mówi: "Przeszedłem 1368 z 1440 okien." Tworzy dowód kryptograficzny (ZK proof) że to prawda. Ty sprawdzasz czy Twoje dane się zgadzają. Jeśli się zgadzają — akceptujesz. Jeśli nie — spór.

### Rozliczenie

Chain oblicza: kwota × zaliczone_okna / wszystkie_okna. Reszta do Ciebie. Nikt nie decyduje — formuła jest na chainie. Zero ludzkiego osądu.

### Spór

Jeśli nie zgadzasz się z dowodem minera, kwestionujesz. Chain wymaga od minera dowodu per okno. Chain weryfikuje. Kto przegrywa — płaci opłatę za spór.

---

## Problemy

### Problem 1: Jak sprawdzić czy miner nie kłamie?

Miner sam mierzy swoje zasoby. Jak wiedzieć czy nie oszukuje?

**Rozwiązanie:** Challengi. Co okno dostaje pytanie na które tylko prawdziwy zasób może odpowiedzieć. Miner nie zna pytania przed blokiem (bo hash bloku jest nieprzewidywalny). Nie może się przygotować. Albo ma zasób — albo nie. Do tego ZK proof — dowód krypto że jego pomiary są prawdziwe, bez ujawniania szczegółów.

### Problem 2: Jak sprawdzić czy zasób nie jest współdzielony z innymi?

Miner może wynająć ten sam GPU 10 osobom.

**Rozwiązanie:** Jeśli GPU jest overcommitted — jest wolniejszy. Wolniejszy = nie zdaje benchmarku = FAIL. Dodatkowo: statystyka. Jeśli miner oszukuje — więcej okien FAILuje — mniej pieniędzy. Economic incentive żeby nie oszukiwać.

### Problem 3: Jak chronić prywatność minera?

Miner nie chce pokazywać ile ma GPU, ilu userów obsługuje, jakie ma obciążenie.

**Rozwiązanie:** ZK proof. Dowód krypto że pomiary są prawdziwe — bez ujawniania samych pomiarów. Chain widzi PASS/FAIL. Nie widzi szczegółów.

### Problem 4: Jak działa discovery bez publicznego rejestru?

Nie ma publicznej listy minerów. Jak znaleźć compute?

**Rozwiązanie:** Zaszyfrowane zapytania przez skrzynkę. Skrzynka nie widzi treści. Minerowie próbują odszyfrować. Pasujący odpowiada. Niepasujący ignoruje. Nikt nie widział kto szukał ani kto oferował.

### Problem 5: Jak działa tożsamość bez publicznego profilu?

Nie ma profili. Jak powiązać klucze z rolami?

**Rozwiązanie:** Dwa niezależne klucze. Jeden dla validatora (pilnuje chaina). Jeden dla minera (wynajmuje compute). Nie są powiązane. Falcon to narzędzie podpisu — nie tożsamość.

---

## Zależności

```
Sesja potrzebuje:
  ├── escrow lock (chain blokuje pieniądze)
  ├── compute miner (ktoś musi mieć GPU)
  ├── transport (NXMS skrzynka lub P2P)
  └── window protocol (sprawdzanie co pół godziny)

Settlement potrzebuje:
  ├── receipt (dowód od minera)
  ├── ZK proof (dowód krypto)
  ├── lease policy (warunki umowy)
  └── chain (oblicza i dzieli)

Discovery potrzebuje:
  ├── NXMS mailbox (transport)
  ├── ComputeOffering (co miner oferuje)
  └── encrypted query (co user szuka)

Chain potrzebuje:
  ├── konsensus (validatorzy piszą bloki)
  ├── SpendPolicy (reguły escrow)
  └── nullifiers (żeby nie wydać dwa razy)

Window protocol potrzebuje:
  ├── bloki (jako zegar)
  ├── challenge generation (hash z bloku)
  ├── response verification (sprawdzenie odpowiedzi)
  └── receipt aggregation (zliczenie wyników)
```

---

**To jest cały system. Możliwości, funkcje, problemy, zależności. Nic więcej nie trzeba wiedzieć żeby zacząć budować.**
