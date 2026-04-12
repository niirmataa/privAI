# privAI V0: Identity Migration Audit

**Status:** technical audit / identity migration analysis
**Data:** 2026-04-11
**Źródło:** P-T043-XIAOMI
**Zakres:** jak obecny kod używa Falcon identity i jak bezpiecznie migrować do hidden root + role keys

---

## 1. Current Identity Map

| File | Symbol | What Identity Means There | Falcon Role | pk_hash Role | Risk |
|---|---|---|---|---|---|
| `privai-node/src/identity_provider.rs:5-11` | `PQCIdentity` | **Primary node identity.** Ładuje Falcon PK+SK + FrodoKEM PK+SK z vault. | Falcon PK = primary public identity. Falcon SK = signing key dla wszystkiego. | N/A — pk_hash jest derived w node.rs | **HIGH** — Falcon PK jest traktowany jako TOŻSAMOŚĆ, nie jako narzędzie podpisu. V0 mówi: Falcon jest signing tool, nie identity. |
| `privai-node/src/identity_provider.rs:13-19` | Vault TLV tags | Vault format: `T_FALCON_SK=3, T_FALCON_PK=4, T_KEM_SK=5, T_KEM_PK=6` | Falcon SK/PK są głównymi tagami w vault. | N/A | **MEDIUM** — vault format jest fixed. Dodanie nowych tagów (root, epoch keys) jest additive ale wymaga zmiany nexum-cli. |
| `privai-node/src/node.rs:224-244` | `load_identity()` | Ładuje PQCIdentity z vault. Ustawia `node_pk_hash` = `domain_hash("privai:falcon-pk:v0", &[falcon_pk])`. Ustawia `node_sig_sk`, `node_sig_pk`. | Falcon SK → `self.falcon_sk` (Zeroizing). Falcon PK → `self.config.node_sig_pk`. | `domain_hash("privai:falcon-pk:v0", &[falcon_pk])` → `self.config.node_pk_hash` | **HIGH** — `node_pk_hash` jest derived z Falcon PK. To jest primary system identifier. Zmiana = zmiana wszędzie. |
| `privai-node/src/node.rs:95` | `falcon_sk: Option<Zeroizing<Vec<u8>>>` | Secret signing key węzła. Używany do voting, block signing, marketplace batch signing. | **Signing key dla WSZYSTKIEGO.** | N/A | **HIGH** — jeden klucz podpisuje consensus, escrow, marketplace. V0 chce separacji ról. |
| `privai-node/src/node.rs:342-343` | Consensus voting | `falcon_sign_ct_prepared(sk, &block.hash())` — podpis bloku. | Falcon SK podpisuje blok. | N/A | **MEDIUM** — voting jest core. Zmiana signing key = zmiana consensus. Ale klucz może zostać ten sam (semantyczna zmiana: "to jest validator role key"). |
| `privai-node/src/node.rs:403` | Block template | `proposer_pk_hash: self.config.node_pk_hash` — proposer jest identified by pk_hash. | N/A | `node_pk_hash` identyfikuje proposera w block template. | **MEDIUM** — consensus używa pk_hash jako identifier. To jest code-confirmed. |
| `privai-node/src/node.rs:371` | Proposer check | `if proposer != self.config.node_pk_hash` — porównanie proposera z node identity. | N/A | `node_pk_hash` jest porównywany z proposer. | **MEDIUM** — zmiana node_pk_hash = zmiana proposer check. |
| `privai-node/src/node.rs:253-270` | `sign_marketplace_batch()` | Node podpisuje MarketplaceBatchTx kluczem Falcon. | Falcon SK podpisuje settlement batch. | N/A | **HIGH** — to jest marketplace-era. V0 odrzuca. Ale mechanika signing jest reuse'owalna. |
| `privai-node/src/config.rs:85` | `NodeConfig.node_pk_hash: Hash32` | **Primary node identifier.** Hash z Falcon PK. Używany w consensus, block building, identity matching. | N/A | `node_pk_hash` = hash Falcon PK. Jest primary ID w całym systemie. | **HIGH** — to jest najważniejszy punkt. Zmiana node_pk_hash = zmiana wszędzie. |
| `privai-node/src/config.rs:97-100` | `node_sig_pk, node_sig_sk` | Pełny Falcon PK i SK w config. Używane do P2P handshake. | Falcon PK/SK dla P2P transport. | N/A | **MEDIUM** — transport używa Falcon PK jako peer identity. |
| `privai-chain/src/hash.rs:20-26` | `falcon_pk_hash()` | `domain_hash("privai:falcon-pk:v0", &[pk])` — canonical hash Falcon PK. | N/A | **Canonical hasher dla Falcon PK.** Używany w SpendPolicy, escrow, consensus. | **HIGH** — to jest funkcja która tworzy pk_hash. Zmiana domain string = zmiana wszystkich hashes. Ale NIE zmieniać — backward compatibility. |
| `privai-chain/src/escrow.rs:47-54` | `SignerRole` | Buyer=0, Merchant=1, Operator=2. Role derived from policy pk_hashes. | N/A — SignerRole jest index-based, nie key-based. | pk_hashes w SpendPolicy::Escrow2of3 identyfikują role. | **NISKI** — to jest escrow bridge. Buyer/Merchant/Operator zostają. ComputeLeaseEscrow używa nowych nazw. |
| `privai-chain/src/escrow.rs:91-96` | `SpendPolicy::Escrow2of3` | `buyer_pk_hash, merchant_pk_hash, operator_pk_hash` — 3 Falcon PK hashes. | N/A | **3 pk_hashes definiują escrow policy.** To jest core binding. | **NISKI** — Escrow2of3 jest bridge. Nie zmieniać. |
| `privai-ledger/src/escrow.rs:100-110` | Signer identification | `falcon_pk_hash(pk)` → match against buyer/merchant/operator pk_hash. | N/A | Ledger identyfikuje signera przez pk_hash match. | **NISKI** — to jest frozen validation. Nie zmieniać. |
| `privai-chain/src/consensus.rs:256-257` | `Vote` | `validator_pk: Vec<u8>` (pełny Falcon PK) + `falcon_sig: Vec<u8>`. | Falcon PK jest validator identity w vote. Falcon SK podpisuje vote. | N/A — validator_pk jest full key, nie hash. | **MEDIUM** — consensus używa full Falcon PK jako validator identity. Zmiana = zmiana consensus protocol. |
| `privai-chain/src/consensus.rs:119` | `BlockTemplate.proposer_pk_hash` | Hash identyfikujący proposera bloku. | N/A | `proposer_pk_hash` identyfikuje proposera. | **MEDIUM** — consensus core. |
| `nxms-transport/src/peers.rs:6-14` | `Peer` | `{ id, host, port, kem_pk_b64, sig_pk_b64 }` — peer identity w transport. | `sig_pk_b64` = Falcon PK peer. Używany do weryfikacji envelope signatures. | N/A | **MEDIUM** — transport używa Falcon PK jako peer identifier. |
| `privai-node/src/config.rs:97` | `node_sig_pk` | Pełny Falcon PK w NodeConfig. Używany do P2P handshake. | Falcon PK = transport identity. | N/A | **MEDIUM** — transport layer. |
| `privai-wallet/src/escrow_builder.rs` | Escrow builder | Buduje escrow transfer notes. Wymaga Falcon PK hashes dla buyer/merchant/operator. | N/A | pk_hashes są inputem do SpendPolicy::Escrow2of3. | **NISKI** — escrow builder jest bridge. |

---

## 2. V0 Identity Requirements

| Requirement | Description | Code Gap |
|---|---|---|
| **Hidden root credential** | Nigdy exposeowany. Źródło wszystkich derivacji. Post-quantum secure. | Brak w kodzie. `PQCIdentity` nie ma root — ma tylko Falcon PK/SK i KEM PK/SK. |
| **Role keys (per role)** | Validator, ComputeMiner, Relay, Mailbox, ExitNode — osobne klucze derived z root. | Brak. Kod ma JEDEN Falcon SK dla wszystkiego. `falcon_sk` w node podpisuje consensus, escrow, marketplace. |
| **Validator role key** | Obecny Falcon PK staje się validator role key. Semantyczna zmiana. | **Możliwe bez zmian kodu.** Obecny Falcon PK = validator role key. Zmiana jest tylko w docs/naming. |
| **Compute miner role key** | Nowy Falcon PK derived z root. Osobny od validator. | Brak. Nie istnieje. Nowy klucz. |
| **Relay/Mailbox/Exit role keys** | Nowe Falcon PK derived z root. Osobne per rola. | Brak. Nie istnieją. Nowe klucze. |
| **Session keys** | Per-lease, derived z role key. Discarded after session. | Brak. Session management nie istnieje. |
| **Epoch keys** | Rotowane periodicznie. Derived z role key. | Brak. Rotation protocol nie istnieje. |
| **Scoped offering IDs** | Per-offering, derived z compute miner role key. Rotowalny. | Brak. Discovery protocol nie istnieje. |
| **Selective reliability proof** | Miner pokazuje score specific userowi, nie globalnie. | Brak. Scoring formula nie istnieje. |

---

## 3. Migration Options

### Option A: Keep Falcon PK as Validator Role Key (semantyczna zmiana)

**Opis:** Obecny Falcon PK w vault = "validator role key." Nie zmieniać zachowania, nie zmieniać kodu. Zmienić tylko semantykę w docs: "Falcon PK jest kluczem roli validator, nie tożsamością."

**Benefit:**
- ZERO zmian w kodzie. ZERO zmian w consensus. ZERO zmian w escrow.
- `node_pk_hash` nadal jest hash Falcon PK. Nadal identyfikuje validatora.
- `falcon_sk` nadal podpisuje. Nadal działa.
- Vault format niezmieniony.
- Backward compatibility pełna.

**Risk:**
- To jest "połowa migration." Falcon PK jest nadal używany jako identity w praktyce (consensus, escrow, transport). Zmiana jest tylko semantyczna.
- Nie dodaje hidden root, session keys, epoch keys.

**Code impact:** ZERO.

### Option B: Introduce Hidden Root Alongside Vault

**Opis:** Dodaj nowe TLV tagi do vault format: `T_HIDDEN_ROOT = 0x0010`. Hidden root jest generowany osobno. Obecny Falcon PK jest derived z root (lub jest marked jako "role key under root"). Vault ma oba: root + existing keys.

**Benefit:**
- Root istnieje jako source of truth.
- Existing keys mogą być "linked" do root jako role keys.
- Derivation path jest defined (root → validator key, root → compute miner key, etc.).

**Risk:**
- Zmiana vault format w nexum-cli — nowe TLV tagi.
- `load_identity()` musi ładować root.
- Root jest wrażliwy — musi być Zeroizing, musi być chroniony.

**Code impact:** Średni — nowe TLV tagi, nowe ładowanie, ale existing key usage niezmienione.

### Option C: Derive New Role Keys from Root

**Opis:** Po dodaniu root, derivuj nowe klucze: compute miner, relay, mailbox, exit. Każdy jest osobnym Falcon keypair derived z root.

**Benefit:**
- Separacja ról na poziomie kluczy.
- Compute miner identity jest niezależny od validator identity.

**Risk:**
- Nowe klucze = nowe pk_hashes = nowe SpendPolicy fields.
- Nie wpływa na consensus (validator key jest ten sam).

**Code impact:** Średni — nowe klucze, nowe pk_hashes, ale additive.

### Option D: Keep Consensus Identity Unchanged

**Opis:** Consensus nadal używa `node_pk_hash` = hash obecnego Falcon PK. Nie zmieniać voting, block building, proposer selection.

**Benefit:**
- ZERO ryzyka dla consensus.
- ZERO ryzyka dla block production.
- ZERO ryzyka dla validator set.

**Risk:**
- Consensus nie korzysta z nowej identity hierarchy.
- Ale to jest OK na teraz — V0 mówi że consensus nie zmienia się dla V0.

**Code impact:** ZERO.

### Option E: Full Identity Rewrite

**Opis:** Zmień wszystko: nowy root, nowe klucze, nowy consensus identity, nowy escrow identity, nowy transport identity.

**Benefit:**
- Czysty start.

**Risk:**
- **KATASTROFALNE.** Zmiana consensus identity = zmiana voting, block building, proposer selection, validator set. Zmiana escrow identity = zmiana SpendPolicy fields, existing notes nieważne. Zmiana transport identity = zmiana peer book, handshake, envelope verification.
- **To jest rewrite systemu, nie migration.**

**Code impact:** **KATASTROFALNY.**

---

## 4. Recommendation

### Rekomendacja: Sekwencyjna — A teraz, B później, C w fazach

**Krok 1 (teraz — Phase M7.0): Semantyczna zmiana**
- Obecny Falcon PK = "ValidatorRoleKey" (w docs, nie w kodzie)
- `node_pk_hash` = hash ValidatorRoleKey (niezmienione)
- `falcon_sk` = signing key dla roli validator (niezmienione)
- Dodaj komentarz w `identity_provider.rs`: "PQCIdentity loads validator role keys from vault. V0: Falcon PK is a role key under hidden root, not the root identity."
- Dodaj komentarz w `config.rs`: "node_pk_hash is the hash of the validator role key (Falcon PK)."
- **ZERO zmian w zachowaniu. ZERO zmian w kodzie poza komentarzami.**

**Krok 2 (po Phase M7.0 — Identity Model Direction doc): Hidden root w vault**
- Dodaj nowy TLV tag `T_HIDDEN_ROOT = 0x0010` do vault format
- `HiddenRootCredential` struct z `root_seed: Zeroizing<[u8; 64]>`
- `load_identity()` ładuje root jeśli istnieje
- Root jest optional na start — jeśli nie ma root, system działa jak dziś (backward compatible)
- Obecny Falcon PK jest "linked" do root jako validator role key

**Krok 3 (po Phase M7.0 — ComputeMiner moduł): Compute miner role key**
- Derive nowy Falcon keypair z root
- `ComputeMinerRoleKey` = nowy klucz, nowy pk_hash
- Używany tylko przez compute miner moduł
- Nie wpływa na consensus, nie wpływa na escrow

**Krok 4 (po Phase M7.0 — future): Relay/Mailbox/Exit role keys**
- Derive nowe klucze z root
- Każdy jest osobny
- Dodawane w fazach gdy te role są implementowane

**Krok 5 (po Identity Model Direction doc): Session keys**
- Per-lease key derivation
- Dodane gdy metering/lease infrastructure istnieje

**Krok 6 (po Identity Model Direction doc): Epoch keys**
- Rotated periodically
- Dodane gdy rotation protocol jest zdefiniowany

---

## 5. What Must Not Change Yet

| Element | Dlaczego nie zmieniać | Kiedy można zmienić |
|---|---|---|
| **Consensus identity (`node_pk_hash`)** | Core system identifier. Zmiana = zmiana voting, block building, proposer selection, validator set. Katastrofalne. | **Nigdy** w ramach V0. Consensus identity = obecny Falcon PK hash. Nowe role mają osobne klucze. |
| **`falcon_pk_hash()` function** | Canonical hasher używany w SpendPolicy, escrow, consensus. Zmiana domain string = zmiana wszystkich hashes. | **Nigdy.** Ta funkcja jest frozen. Nowe klucze używają tej samej funkcji. |
| **Escrow auth (`SpendPolicy::Escrow2of3` pk_hashes)** | Existing escrow notes mają embedded pk_hashes. Zmiana = existing notes nieważne. | **Nigdy** dla Escrow2of3. ComputeLeaseEscrow używa nowych pól. |
| **Vault TLV format (existing tags)** | `T_FALCON_SK=3, T_FALCON_PK=4, T_KEM_SK=5, T_KEM_PK=6` — existing vaults używają tych tagów. Zmiana = existing vaults nieczytelne. | **Nigdy** dla existing tags. Nowe tagi (root, epoch keys) są additive. |
| **`Vote.validator_pk` field** | Consensus protocol używa full Falcon PK jako validator identity. Zmiana = zmiana consensus protocol. | **Nigdy** w ramach V0. Validator PK = obecny Falcon PK. |
| **Transport peer identity (`Peer.sig_pk_b64`)** | P2P handshake używa Falcon PK jako peer identity. Zmiana = zmiana peer book format. | **Nigdy** dla existing peers. Nowi peers (compute miner, relay) mają nowe klucze. |
| **`node_sig_pk`, `node_sig_sk` w NodeConfig** | P2P transport używa tych pól. Zmiana = zmiana handshake. | **Nigdy** dla validator node. Compute miner node ma osobny config. |

---

## 6. Red Lines

1. **Nie zmieniać `node_pk_hash` derivation.** `domain_hash("privai:falcon-pk:v0", &[falcon_pk])` jest frozen. Nowe klucze używają tej samej funkcji.

2. **Nie zmieniać `falcon_pk_hash()` domain string.** `"privai:falcon-pk:v0"` jest frozen. Zmiana = zmiana wszystkich hashes w systemie.

3. **Nie usuwać Falcon PK z vault.** Obecny Falcon PK jest validator role key. Musi pozostać.

4. **Nie zmieniać consensus identity.** `Vote.validator_pk` = Falcon PK. `proposer_pk_hash` = hash Falcon PK. To jest frozen.

5. **Nie zmieniać Escrow2of3 pk_hashes.** Existing notes mają embedded hashes. Zmiana = existing notes nieważne.

6. **Nie dodawać hidden root jako required na start.** Root jest optional — backward compatibility. System działa bez root (obecny model).

7. **Nie twierdzić że hidden root istnieje w kodzie.** Nie istnieje. Jest direction-level.

8. **Nie projektować credential wire formatu.** Format jest future protocol spec.

9. **Nie zmieniać transport peer identity na start.** `Peer.sig_pk_b64` = Falcon PK. To jest frozen dla existing peers.

10. **Nie merge'ować validator identity i compute miner identity.** Dwa różne klucze, dwa różne pk_hashes, nigdy nie powiązane chyba że przez root.

---

## Summary

| Migration Step | When | Code Impact | Risk |
|---|---|---|---|
| A: Semantyczna zmiana (Falcon PK = role key) | TERAZ | Komentarze tylko | ZERO |
| B: Hidden root w vault | Po Identity Model Direction doc | Nowe TLV tagi, opcjonalne ładowanie | NISKI (optional) |
| C: Compute miner role key | Po ComputeMiner moduł | Nowy klucz, nowy pk_hash | NISKI (additive) |
| D: Relay/Mailbox/Exit keys | Future | Nowe klucze | NISKI (additive) |
| E: Session keys | Po metering/lease spec | Nowe klucze per session | ŚREDNI (new infra) |
| F: Epoch keys | Po rotation protocol | Rotated klucze | ŚREDNI (new infra) |
| G: Full rewrite | **NIGDY** | Katastrofalny | KATASTROFALNY |

---

**Czy edytowano pliki:** NIE (poza zapisem tego pliku)
**Czy czytano kod:** TAK — `privai-node/src/identity_provider.rs` (pełny), `privai-node/src/config.rs` (pełny), `privai-node/src/node.rs` (identity/consensus/escrow sections), `privai-chain/src/hash.rs` (falcon_pk_hash), `privai-chain/src/escrow.rs` (SignerRole, SpendPolicy), `privai-chain/src/consensus.rs` (Vote, ViewChange), `nxms-transport/src/peers.rs` (Peer)
**Czy czytano legacy docs:** NIE
**Czy zdefiniowano wire formaty:** NIE
**Czy odpowiedź jest migration audit:** TAK
