# T-056-GEMINI — Vertex RAG Bridge Design

## 1. Verdict

Projekt zakłada wdrożenie Vertex RAG jako przyszłego backendu dla `privai-context-mcp` (Sprint 4+), a nie jako wymagania dla Sprintu 1. Architektura serwera MCP musi opierać się na abstrakcji `KnowledgeStore`, która pozwala na bezinwazyjną wymianę lokalnego `FileStore` ze Sprintu 1 na `VertexRetrieverStore` w przyszłości. Integracja z Vertex RAG ma na celu skalowanie wyszukiwania w warstwie kontekstowej V0, bez naruszania struktury narzędzi, polityki jednego źródła prawdy (Single Source of Truth) ani barier dostępu do danych (Guardrails).

## 2. Storage Boundary

Granica przechowywania (Storage Boundary) musi być zdefiniowana przez cechę (trait) `KnowledgeStore` w kodzie serwera MCP (Rust). Narzędzia MCP (np. `privai_v0_lookup_direction`, `privai_v0_prepare_task_context`) nie mogą zależeć od bezpośrednich odczytów z systemu plików ani bezpośrednich wywołań API Vertex. Zamiast tego komunikują się wyłącznie przez metody zdefiniowane w `KnowledgeStore`, które zwracają wystandaryzowane obiekty z odpowiednimi metadanymi. Dzięki temu wstrzyknięcie Vertex RAG ograniczy się do dostarczenia nowej implementacji `KnowledgeStore`, podczas gdy warstwa narzędzi (Tool Layer) pozostanie nietknięta.

## 3. FileStore vs VertexRetrieverStore vs HybridStore

- **FileStore (Sprint 1):** Odczytuje wygenerowane pliki indeksów JSON z dysku (np. `v0_master.json`, `v0_direction.json`). Prosty, deterministyczny, oparty wyłącznie na plikach.
- **VertexRetrieverStore (Przyszłość):** Implementuje `KnowledgeStore` wywołując API Vertex AI. Pobiera fragmenty dokumentów V0 opierając się na wyszukiwaniu semantycznym (embeddings), rzutując wyniki na ten sam format wyjściowy zgodny z MCP.
- **HybridStore (Opcjonalnie):** Może używać `FileStore` do precyzyjnych odczytów (np. master, drzewo dokumentów) oraz `VertexRetrieverStore` do szerszych zapytań tematycznych w warstwach `direction`.

## 4. Source Scope Enforcement

Zarówno na etapie indeksowania w Vertex, jak i w trakcie zapytań (retrieval), system musi kategorycznie egzekwować filtry w metadanych:
- `source_scope = "v0_only"`
- `legacy_allowed = false`
W zapytaniach do Vertex RAG te wartości muszą być wysyłane jako bezwzględne filtry (hard filters). Jeśli fragment dokumentu w indeksie nie posiada zatwierdzonych tagów V0, zapytanie nie może go w ogóle zwrócić.

## 5. What Vertex May Index

Vertex RAG może indeksować wyłącznie pliki z katalogu `spec/PRIVAI_V0_PRIVATE_COMPUTE/`:
- Zaakceptowane dokumenty kierunkowe V0 (np. reset prywatnej sieci obliczeniowej).
- Logi zadań i promptów V0 (`PRIVAI_V0_TASK_LOG.md`, `PRIVAI_V0_PROMPT_LOG.md`).
- Drzewo dokumentów (`PRIVAI_V0_DOCS_TREE.md`).
- Plany kontekstowe i polityki.
- Przyszłe, zaakceptowane specyfikacje protokołów V0 (po ich weryfikacji).

## 6. What Vertex Must Never Index

Vertex RAG bezwzględnie NIE MOŻE indeksować:
- Przestrzeni roboczej zadań: katalogów pod `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/**`.
- Surowych wyników pracy modeli i notatek recenzentów (np. promptów, dokumentów OUTPUT_*).
- Dokumentów legacy (starych założeń marketplace, starych logów z korzenia repozytorium).
- Dokumentów typu handoff (np. `TASK_031_OPUS_WORKING_CONTEXT.md`).
- Treści wprowadzających założenia "AI marketplace", "public discovery" czy "subjective AI quality settlement".

## 7. Tool Contract Stability

Wprowadzenie Vertex RAG nie może spowodować zmiany nazw ani kontraktów 8 narzędzi MCP ze Sprintu 1 (np. `privai_v0_route_question`, `privai_v0_prepare_task_context`).
Każde z narzędzi musi wciąż przyjmować te same parametry wejściowe i zwracać obiekty JSON w tej samej strukturze. System nie może dodawać narzędzia do czytania całego kodu ani narzędzia do wyszukiwania ogólnego. Jeśli odpowiedź z Vertex RAG dostarcza kierunku, ale brak pewności co do stanu implementacji, narzędzie musi wymusić na modelu przyjęcie wariantu `repo_unverified`.

## 8. Sync / Ingestion Pipeline

Proces synchronizacji i dodawania danych do indeksu Vertex (Ingestion) musi działać poza serwerem MCP (Out-of-band). Serwer MCP zajmuje się wyłącznie odczytem.
Zewnętrzny skrypt indeksujący musi mieć wbudowane filtry odrzucające całą gałąź `tasks/` oraz inne pliki wykluczone. Pipeline nałoży na wpuszczane do bazy dokumenty twarde metadane warstw (`layer`), zakazu korzystania z legacy i wymogu trzymania się V0, tak aby nie było żadnego ryzyka zaindeksowania niepotwierdzonych brudnopisów.

## 9. Golden Tests For Vertex RAG

Przed użyciem przez agentów, implementacja połączona z Vertex RAG musi przechodzić te same testy Golden co `FileStore`:
1. Vertex nie może opisywać `privAI` jako giełdy modeli (AI marketplace).
2. Vertex musi jednoznacznie odrzucać istnienie publicznych profili dostawców i public discovery jako wartości domyślnej.
3. Vertex musi odpowiadać, że rozliczenie pro-rata i bezoperatorowe escrow to plany docelowe, a nie obecna implementacja.
4. Vertex nie może odnaleźć starych dokumentów legacy ani udostępniać nieprzejrzanych wyników z folderu `tasks/`.
5. Vertex musi utrzymywać i eksponować w wynikach twarde flagi `source_scope = v0_only` oraz `legacy_allowed = false`.

## 10. Rollout Plan

Zgodnie z planem serwera MCP:
- **Sprint 1:** Wdrożenie lokalnego `FileStore` z narzędziami, bez dostępu do chmury. Oparcie indeksu o przygotowane pliki JSON.
- **Sprint 2:** Testy guardrails i upewnienie się, że testy Golden przechodzą na z góry określonym zbiorze.
- **Sprint 3:** Konfiguracja modeli agentowych ze `stdio` pod wpięty serwer MCP.
- **Sprint 4:** Opcjonalne uruchomienie Vertex RAG. Nowy storage wymienia stary na poziomie API, zachowując bezwzględnie ten sam zestaw kontraktów, a pipeline synchronizacji pilnuje braku brudnopisów w indeksie.

## 11. Open Risks

- **Rozmycie Semantyczne (Semantic Drift):** Wyszukiwanie wektorowe Vertexa może błędnie uznawać zapytania o stary model "marketplace" jako "wystarczająco podobne" do nowej "sieci prywatnych obliczeń" (private compute network), zwracając wyniki naruszające zasady.
- **Wyciek Indeksu:** Błąd na etapie pipeline'u synchronizacji może przesłać nieprzetworzone wyniki z folderu `tasks/` do chmury i naruszyć Single Source of Truth.
- **Halucynacje Implementacyjne:** Brak jasnej wiedzy o aktualnym kodzie w chmurowym indeksie może prowokować agenty do wnioskowania faktu implementacji tylko z dokumentu planistycznego. Narzędzia zmuszone są do zachowania ostrożności z metadanymi typu `repo_unverified`.

## 12. Final Self-Check

- Vertex RAG must be future backend only, not Sprint 1 requirement: YES
- MCP tools must keep the same names and output contracts: YES
- Vertex must not ingest legacy docs: YES
- Vertex must not ingest `tasks/` raw outputs: YES
- Vertex must not ingest handoff docs: YES
- Vertex must not become `search_everything`: YES
- Vertex must preserve `source_scope = v0_only`: YES
- Vertex must preserve `legacy_allowed = false`: YES
- If implementation truth is needed, answer `repo_unverified`, not inferred: YES
