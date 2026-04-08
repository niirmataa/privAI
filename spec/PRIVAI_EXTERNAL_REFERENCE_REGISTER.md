# privAI External Reference Register

Status: curated external reference register for architecture, cryptography, proof, transport and storage context.
Canonicality: reference-only document. This file does not override privAI protocol, formats, consensus, proof or product semantics. It exists to record high-value external materials that help explain or justify design choices.
Owner: privAI architecture and research context.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_EXECUTION_SPINE.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- miec jedna kuratorowana liste zewnetrznych referencji,
- nie wrzucac losowych blogpostow i przypadkowych PDF-ow do kontekstu agenta,
- oddzielic:
  - external dependencies,
  - design inspiration,
  - formal target references,
- pomoc w budowie MCP/context layer dla projektu.

To nie jest source of truth dla privAI.
To nie jest dokument o tym, co privAI "musi" implementowac 1:1.
To jest rejestr pomocniczy.

## 2. How To Use This Register

Kazdy wpis ma:
- `Type`
- `Subsystem`
- `Why it matters`
- `Authority level`

### Authority levels

- `Protocol dependency`
  - material bezposrednio zwiazany z technologia uzywana w kodzie
- `Design inspiration`
  - material pomocny semantycznie lub architektonicznie
- `Formal target`
  - material przydatny do uscislenia modelu bezpieczenstwa lub future formalization

Twarda zasada:
- zadna z tych referencji nie nadpisuje lokalnych docs privAI
- agent nie powinien traktowac ich jako canonical privAI semantics

## 3. Priority Starter Set

1. Falcon official specification
2. FrodoKEM official specification
3. The halo2 Book
4. Zcash protocol specification
5. Zero to Monero
6. FROST RFC 9591
7. Tor rendezvous / onion services specs
8. RocksDB docs on column families / write discipline

## 4. PQ Signatures

### 4.1. Falcon official site
- URL: [https://falcon-sign.info/](https://falcon-sign.info/)
- Type: official project site
- Subsystem: PQ auth, transport auth, tx auth
- Why it matters: glowny punkt odniesienia dla Falcona jako podpisu PQ
- Authority level: `Protocol dependency`

### 4.2. Falcon official specification PDF
- URL: [https://falcon-sign.info/falcon.pdf](https://falcon-sign.info/falcon.pdf)
- Type: specification PDF
- Subsystem: PQ auth, transport auth, tx auth
- Why it matters: najwazniejsza techniczna referencja do semantyki i parametrow Falcona
- Authority level: `Protocol dependency`

### 4.3. Falcon research publication
- URL: [https://research.ibm.com/publications/falcon-fast-fourier-lattice-based-compact-signatures-over-ntru](https://research.ibm.com/publications/falcon-fast-fourier-lattice-based-compact-signatures-over-ntru)
- Type: research paper landing page
- Subsystem: PQ signatures
- Why it matters: background i uzasadnienie konstrukcji Falcona
- Authority level: `Design inspiration`

## 5. PQ Key Exchange

### 5.1. FrodoKEM official site
- URL: [https://frodokem.org/](https://frodokem.org/)
- Type: official project site
- Subsystem: validator session transport, PQ confidentiality
- Why it matters: glowny punkt odniesienia dla FrodoKEM
- Authority level: `Protocol dependency`

### 5.2. FrodoKEM current specification page
- URL: [https://frodokem.org/](https://frodokem.org/)
- Type: specification index
- Subsystem: validator session transport
- Why it matters: prowadzi do aktualnych PDF-ow specyfikacji i papierow
- Authority level: `Protocol dependency`

### 5.3. FrodoKEM Internet-Draft
- URL: [https://datatracker.ietf.org/doc/html/draft-longa-cfrg-frodokem](https://datatracker.ietf.org/doc/html/draft-longa-cfrg-frodokem)
- Type: Internet-Draft
- Subsystem: validator session transport
- Why it matters: aktualniejsza, standaryzowana forma opisu konstrukcji
- Authority level: `Protocol dependency`

## 6. Threshold / Multisig / Auth Semantics

### 6.1. FROST RFC 9591
- URL: [https://www.rfc-editor.org/rfc/rfc9591](https://www.rfc-editor.org/rfc/rfc9591)
- Type: RFC
- Subsystem: threshold auth, signer coordination, rogue-key thinking
- Why it matters: dobry formalny punkt odniesienia dla threshold signing semantics
- Authority level: `Formal target`

### 6.2. FROST Datatracker page
- URL: [https://datatracker.ietf.org/doc/html/rfc9591](https://datatracker.ietf.org/doc/html/rfc9591)
- Type: RFC HTML
- Subsystem: threshold auth
- Why it matters: wygodna wersja przegladarkowa i nawigacyjna
- Authority level: `Formal target`

## 7. Proof / ZK

### 7.1. The halo2 Book
- URL: [https://zcash.github.io/halo2/](https://zcash.github.io/halo2/)
- Type: official documentation
- Subsystem: proof system, circuits, witness/public-input boundary
- Why it matters: glowna dokumentacja halo2
- Authority level: `Protocol dependency`

### 7.2. halo2 GitHub repository
- URL: [https://github.com/zcash/halo2](https://github.com/zcash/halo2)
- Type: official repository
- Subsystem: proof implementation
- Why it matters: kod referencyjny, issues, implementation details
- Authority level: `Protocol dependency`

### 7.3. halo2_proofs crate docs
- URL: [https://docs.rs/halo2_proofs/latest/halo2_proofs/](https://docs.rs/halo2_proofs/latest/halo2_proofs/)
- Type: crate docs
- Subsystem: Rust implementation details
- Why it matters: praktyczne API-level odniesienie do biblioteki
- Authority level: `Protocol dependency`

## 8. Notes / Nullifiers / Privacy System Inspiration

### 8.1. Zcash protocol specification
- URL: [https://zcash.readthedocs.io/en/master/rtd_pages/protocol.html](https://zcash.readthedocs.io/en/master/rtd_pages/protocol.html)
- Type: official protocol docs
- Subsystem: notes, commitments, nullifiers, proof-vs-ledger boundary
- Why it matters: bardzo mocny punkt odniesienia dla note-based privacy systems
- Authority level: `Design inspiration`

### 8.2. CryptoNote whitepaper index
- URL: [https://zenodo.org/records/6496896](https://zenodo.org/records/6496896)
- Type: archive/index
- Subsystem: one-time addresses, privacy transaction model
- Why it matters: klasyczna referencja dla stealth / one-time output intuition
- Authority level: `Design inspiration`

### 8.3. Monero Research Lab index
- URL: [https://www.getmonero.org/resources/research-lab/index.html](https://www.getmonero.org/resources/research-lab/index.html)
- Type: official research index
- Subsystem: privacy transactions, threshold/multisig intuition, note privacy
- Why it matters: uporzadkowany katalog technicznych materialow Monero
- Authority level: `Design inspiration`

### 8.4. Zero to Monero
- URL: [https://www.getmonero.org/library/Zero-to-Monero-2-0-0.pdf](https://www.getmonero.org/library/Zero-to-Monero-2-0-0.pdf)
- Type: technical book PDF
- Subsystem: privacy tx model, outputs, nullifiers/ring intuition
- Why it matters: bardzo praktyczny i konkretny material referencyjny
- Authority level: `Design inspiration`

## 9. Transport / Tor

### 9.1. Tor rendezvous specification v3
- URL: [https://spec.torproject.org/rend-spec/index.html](https://spec.torproject.org/rend-spec/index.html)
- Type: official spec
- Subsystem: onion services, validator transport over Tor
- Why it matters: oficjalna referencja dla onion rendezvous semantics
- Authority level: `Protocol dependency`

### 9.2. Tor onion services technology overview
- URL: [https://onionservices.torproject.org/technology/](https://onionservices.torproject.org/technology/)
- Type: official overview
- Subsystem: onion services
- Why it matters: prostszy kontekst architektoniczny wokol onion services
- Authority level: `Protocol dependency`

### 9.3. Tor onion services support docs
- URL: [https://support.torproject.org/en/onionservices/](https://support.torproject.org/en/onionservices/)
- Type: official support docs
- Subsystem: onion services
- Why it matters: dodatkowe praktyczne wyjasnienia i terminologia
- Authority level: `Protocol dependency`

## 10. AEAD / Session Encryption

### 10.1. libsodium AEAD docs
- URL: [https://libsodium.gitbook.io/doc/secret-key_cryptography/aead/chacha20-poly1305](https://libsodium.gitbook.io/doc/secret-key_cryptography/aead/chacha20-poly1305)
- Type: official docs
- Subsystem: session encryption, AEAD, nonce/AAD handling
- Why it matters: odniesienie do poprawnego uzycia ChaCha20-Poly1305 family
- Authority level: `Protocol dependency`

### 10.2. libsodium XChaCha20 docs
- URL: [https://libsodium.gitbook.io/doc/advanced/stream_ciphers/xchacha20](https://libsodium.gitbook.io/doc/advanced/stream_ciphers/xchacha20)
- Type: official docs
- Subsystem: transport/session confidentiality
- Why it matters: kontekst dla XChaCha20 i extended nonce family
- Authority level: `Protocol dependency`

## 11. Storage / RocksDB

### 11.1. RocksDB getting started
- URL: [https://rocksdb.org/docs/getting-started.html](https://rocksdb.org/docs/getting-started.html)
- Type: official docs
- Subsystem: storage layer, runtime durability
- Why it matters: podstawowa referencja do modelu RocksDB
- Authority level: `Protocol dependency`

### 11.2. RocksDB main site
- URL: [https://rocksdb.org/](https://rocksdb.org/)
- Type: official site
- Subsystem: storage
- Why it matters: glowna strona i wejscie do dokumentacji
- Authority level: `Protocol dependency`

### 11.3. RocksDB column families
- URL: [https://github.com/facebook/rocksdb/wiki/column-families](https://github.com/facebook/rocksdb/wiki/column-families)
- Type: official wiki/docs
- Subsystem: storage layout, durability boundaries
- Why it matters: wazne przy planowaniu incremental durable state
- Authority level: `Protocol dependency`
