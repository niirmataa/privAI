# privAI V0: Zdecentralizowany "Mroczny Las" dla kart graficznych. (Litepaper)

Siema. Siedzisz trochę w krypto, kumasz, jak działają systemy rozproszone i pewnie słyszałeś o DePIN (*Decentralized Physical Infrastructure Networks*). 

Większość projektów w tej branży obiecuje "zdecentralizowane AWS" do trenowania sztucznej inteligencji. W rzeczywistości budują jednak po prostu publiczne Allegro dla kart graficznych (GPU). Zakładasz konto, zbierasz gwiazdki reputacji, a cały świat widzi, kto, komu i za ile wynajmuje sprzęt. Zero prywatności, pełno miejsca na oszustwa i zmowy.

My w **privAI** podeszliśmy do tego zupełnie inaczej. Budujemy system, który jest całkowicie ślepy na to, kim jesteś i co liczysz. Żadnych publicznych profili. Żadnych rankingów. Żadnych moderatorów. Czysta, brutalna kryptografia i matematyka.

Zainteresowany? Zobacz, jak ten "Mroczny Las" działa pod maską.

---

### 1. Discovery, czyli jak dogadać się w ciemności
Wyobraź sobie, że potrzebujesz na 24 godziny potężnej karty Nvidia A100 do potrenowania własnego, prywatnego modelu AI. Nie chcesz, żeby ktokolwiek w internecie (ani nawet sam system) wiedział, że to robisz.

W privAI nie wchodzisz na stronę z listą ofert. Zamiast tego wysyłasz zaszyfrowane zapytanie (używając kryptografii odpornej na komputery kwantowe – `FrodoKEM`) do tzw. Skrzynki Pocztowej (`NXMS Mailbox`). 

To jak wrzucenie zakodowanego listu w butelce do oceanu. Skrzynka pocztowa nie ma pojęcia, co jest w środku. Wiadomość jest tak zaszyfrowana, że tylko sieć węzłów obliczeniowych (Minerów) może próbować ją odczytać. Jeśli jakiś Miner odkoduje wiadomość, ma wolne A100 i pasuje mu Twoja cena, odpowiada Ci bezpośrednio, tworząc bezpieczny, szyfrowany tunel. Ty i on dogadujecie się w totalnej ciemności (P2P przez sieć Tor).

### 2. Blockchain jako Zegar i Głupi Księgowy
W Ethereum każdy węzeł na świecie musi po kolei wykonać Twój kod (Smart Contract), co jest strasznie wolne i potwornie drogie. 

My odrzuciliśmy ten model. W privAI łańcuch bloków (L1) robi tylko dwie, bardzo proste rzeczy:
*   **Działa jako niezawodny metronom (Zegar):** Nowe bloki powstają co około 30 sekund. To nasz niepodważalny, globalny wyznacznik czasu dla całej sieci.
*   **Działa jako Escrow (Depozyt):** Przed startem obliczeń blokujesz na łańcuchu np. 100 tokenów PVA. Łańcuch nie wie, na co idą te pieniądze. Wie tylko: *"Zablokowane na max 24h. Czekam na paragon, żeby je wypłacić"*.

Dzięki temu łańcuch jest lekki, superszybki i dba o pełną anonimowość przepływów.

### 3. Jak udowodnić, że ktoś nie leci w kulki? (Metering & Hash-Chain)
Wynajmujesz od nieznajomego kolesia sprzęt za grube tokeny. Skąd wiesz, że on w tle nie kopie na tym GPU kryptowalut, dając Ci ukradkiem tylko 10% mocy karty?

Zamiast próbować niemożliwego (czyli mierzyć dokładnego zużycia prądu czy cykli procesora na maszynie, której nie kontrolujesz), wymyśliliśmy **Window-Based Metering** (Pomiar okienkowy oparty na wyzwaniach).

1.  Sesja jest podzielona na "Okna" (np. co 60 bloków, czyli ok. 30 minut).
2.  Gdy wybija nowe okno, Ty (User) bierzesz *hash najnowszego bloku* z łańcucha L1 i wysyłasz do Minera jako wyzwanie (Challenge). Hash bloku to losowy ciąg znaków, niemożliwy do odgadnięcia zanim blok nie powstanie.
3.  Na maszynie Minera działa nasz mały, nieugięty Agent (Daemon). Agent błyskawicznie sprawdza dostępność i wydajność GPU (np. puszczając mały, błyskawiczny benchmark), podpisuje wynik cyfrowo kluczem Minera i odsyła Ci go.
4.  **Magia:** Każdy taki paragon zawiera w sobie hash *poprzedniego* paragonu. Tworzy się nierozerwalny łańcuch (`Hash-Chain`). Jeśli Miner spróbuje coś podmienić, oszukać lub zafałszować w przeszłości, cała kryptografia się sypie i od razu to widzisz.

### 4. Rozliczenie za pomocą magii Zero-Knowledge (ZKP)
Koniec sesji. Masz 48 paragonów (po jednym z każdych 30 minut z 24 godzin).

Co jeśli Miner twierdzi, że sprzęt działał idealnie, a Ty twierdzisz, że zrywało połączenie? W tradycyjnym systemie musiałby wkroczyć ludzki moderator. U nas wkracza matematyka.

Miner musi wygenerować i wrzucić na łańcuch dowód **Zero-Knowledge Proof** (używamy zaawansowanej technologii `Halo2`). Ten potężny dowód matematyczny udowadnia węzłom (Validatorom), że ukryty poza łańcuchem *Hash-Chain* faktycznie składa się w prawidłowy wynik, **bez ujawniania światu logów z Twojej pracy czy zawartości telemetrii**.

Łańcuch L1 weryfikuje ten dowód w ułamek sekundy. Jeśli matematyka wykaże, że z 48 okien zdane było 46, deterministyczna funkcja na łańcuchu po prostu przelewa Minerowi zapłatę za 46 okien, a za 2 okna robi Ci z automatu zwrot (Refund). Czysta arytmetyka. Zero ludzkiej ingerencji. Kto kłamał w sporze, traci kaucję.

---

### Dlaczego to jest takie mocne?
Bo budujemy system, który rozwiązuje jeden z największych problemów współczesnej sztucznej inteligencji: **prywatność danych badawczych i korporacyjnych**. 

Jeśli budujesz nową super-aplikację medyczną albo AI dla finansów, nie wyślesz swoich danych na serwery gigantów technologicznych, którzy będą je podglądać i wykorzystywać do trenowania własnych modeli. 

W privAI wynajmujesz czystą, weryfikowalną moc obliczeniową prosto od innych ludzi, w całkowicie zaszyfrowanym tunelu, z gwarancją uczciwego rozliczenia opartą o twardą kryptografię. 

I najlepsze? Cały rdzeń transakcyjny piszemy w RUST, korzystając z podpisów post-kwantowych (Falcon) i najnowszych osiągnięć w dziedzinie ZK (Zero-Knowledge). Tworzymy solidny, inżynierski fundament pod rewolucję prywatnego AI.

Wchodzisz w to?
