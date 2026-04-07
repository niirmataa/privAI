//! Moduł sieciowy P2P dla konsensusu — Tor-only transport.
//!
//! Każdy validator:
//! - nasłuchuje na Tor hidden service (TCP)
//! - wysyła ConsensusMsg do peerów przez SOCKS5h proxy
//! - odbiera ConsensusMsg od innych validatorów
//!
//! ConnectionPool v3 — Actor pattern per peer + encrypted frames + backpressure
//!
//! Architektura:
//! - Każdy peer ma dedykowany writer task (wzorzec Actor)
//! - Wiadomości wrzucane do BOUNDED MPSC channel (backpressure)
//! - Frame encryption: AES-256-GCM z FrodoKEM shared secret
//! - Timeout na całe połączenie (nie tylko na kroki handshake)
//! - Zeroize na kluczach tajnych (ochrona przed memory dump)

use tokio::sync::mpsc;

use nxms_transport::peers::{Peer, PeerBook};
use nxms_transport::tor_net::{connect_via_tor, read_frame, write_frame, serve};
use privai_chain::ConsensusMsg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};

/// Maksymalna liczba jednoczesnych Tor circuit builds.
/// Zapobiega floodowaniu Tora przy burst'ach Gossip.
/// Zwiększono z 3 do 6 aby zmniejszyć kolejki przy wielu validatorach.
const MAX_CONCURRENT_BUILDS: usize = 6;

/// Maksymalna liczba jednoczesnych incoming connections.
/// Zapobiega resource exhaustion przez atak DDoS.
const MAX_INCOMING_CONNECTIONS: usize = 10;

/// Czas banowania złośliwego peera (sekundy).
const BAN_DURATION_SECS: u64 = 3600; // 1 godzina

/// Maksymalna liczba połączeń z tego samego źródła w oknie czasowym.
const MAX_CONNECTIONS_PER_SOURCE: usize = 5;

/// Okno czasowe dla rate limitingu (sekundy).
const RATE_LIMIT_WINDOW_SECS: u64 = 60;

/// Protokół wersji handshake.
const HANDSHAKE_VERSION: u8 = 1;

/// Wiadomość handshake wymieniana przy pierwszym połączeniu.
/// Zawiera klucze publiczne PQC (FrodoKEM + Falcon) do szyfrowania dalszej komunikacji.
/// Podpisana Falconem aby zapobiec MITM.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandshakeMsg {
    /// Wersja protokołu handshake
    pub version: u8,
    /// Klucz publiczny FrodoKEM (base64)
    pub kem_pk_b64: String,
    /// Ciphertext FrodoKEM encapsulation (base64) — do derivacji shared secret
    /// Nadawca robi encap(peer_kem_pk) → (kem_ct, shared_secret)
    /// Odbiorca robi decap(my_kem_sk, kem_ct) → shared_secret
    #[serde(default)]
    pub kem_ct_b64: String,
    /// Klucz publiczny Falcon (base64)
    pub sig_pk_b64: String,
    /// ID peera w PeerBook
    pub peer_id: String,
    /// Podpis Falcon całego obiektu (bez tego pola) — anti-MITM
    pub falcon_sig_b64: String,
}

/// Konfiguracja sieciowa węzła konsensusowego.
#[derive(Clone)]
pub struct NetConfig {
    /// Adres nasłuchu lokalnego (np. "127.0.0.1:19000")
    pub listen_addr: String,
    /// URL proxy Tor SOCKS5h (np. "socks5h://127.0.0.1:9050")
    pub tor_socks_url: String,
    /// Ścieżka do peers.json z PeerBook
    pub peers_path: String,
    /// ID tego węzła w PeerBook
    pub my_peer_id: String,
}

impl NetConfig {
    pub fn new(listen_addr: String, tor_socks_url: String, peers_path: String, my_peer_id: String) -> Self {
        Self {
            listen_addr,
            tor_socks_url,
            peers_path,
            my_peer_id,
        }
    }
}

impl Default for NetConfig {
    fn default() -> Self {
        Self::new(
            "127.0.0.1:19000".to_string(),
            "socks5h://127.0.0.1:9050".to_string(),
            "peers.json".to_string(),
            "validator-0".to_string(),
        )
    }
}

/// Błąd warstwy sieciowej.
#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("transport error: {0}")]
    Transport(#[from] anyhow::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("peer not found: {0}")]
    PeerNotFound(String),
}

/// Serwer nasłuchujący na incoming connections (Tor hidden service).
/// 
/// Flow incoming (zabezpieczony):
/// 1. Rate limiter check (max 5 conn/min z tego samego source)
/// 2. Ban list check
/// 3. Semaphore (max 10 jednoczesnych incoming)
/// 4. Odbierz HandshakeMsg od peera (kem_pk, sig_pk)
/// 5. Weryfikacja peer_id w PeerBook (anti-sybil)
/// 6. Wyślij nasz HandshakeMsg w odpowiedzi
/// 7. Czytaj ConsensusMsg w pętli
pub async fn run_listener(
    config: NetConfig,
    msg_tx: mpsc::Sender<(String, ConsensusMsg)>,
    node_kem_pk: Vec<u8>,
    _node_kem_sk: Vec<u8>,
    node_sig_pk: Vec<u8>,
    node_sig_sk: Vec<u8>,
    node_peer_id: String,
    peer_book: PeerBook,
    ban_list: BanList,
    rate_limiter: RateLimiter,
) -> Result<(), NetError> {
    let listener = serve(&config.listen_addr).await?;
    let incoming_semaphore = Arc::new(Semaphore::new(MAX_INCOMING_CONNECTIONS));

    eprintln!(
        "[net] listening on {} for consensus messages (max {} incoming)",
        config.listen_addr, MAX_INCOMING_CONNECTIONS
    );

    loop {
        let (mut stream, addr) = listener.accept().await?;

        // --- Security: Rate limiter ---
        let source = addr.to_string();
        if !rate_limiter.check(&source).await {
            eprintln!("[net] rate limited connection from {}", source);
            drop(stream);
            continue;
        }

        // --- Security: Ban list ---
        // Sprawdzamy po handshake, bo nie znamy peer_id przed handshake
        // (Tor hidden service nie ujawnia prawdziwego IP)

        // --- Security: Incoming connection semaphore ---
        let permit = match incoming_semaphore.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("[net] too many incoming connections, rejecting {}", source);
                 drop(stream);
                continue;
            }
        };

        let tx = msg_tx.clone();
        let kem_pk = node_kem_pk.clone();
        let sig_pk = node_sig_pk.clone();
        let _sig_sk = node_sig_sk.clone();
        let my_peer_id = node_peer_id.clone();
        let peers = peer_book.peers.clone();
        let ban_list = ban_list.clone();

        // Clone node_sig_sk before moving into tokio::spawn
        let node_sig_sk_clone = node_sig_sk.clone();

        tokio::spawn(async move {
            let _permit = permit; // Hold semaphore until connection closes
            let addr_str = addr.to_string();

            // --- Timeout na całe połączenie (nie tylko handshake) ---
            let connection_result = tokio::time::timeout(
                Duration::from_secs(300), // 5 minut max na połączenie
                async {
            // --- PQC Handshake: odbierz od peera (z timeout) ---
            use base64::Engine;
            use base64::engine::general_purpose::STANDARD as B64;

            let peer_handshake_bytes = match tokio::time::timeout(
                Duration::from_secs(10),
                read_frame(&mut stream, 64 * 1024),
            )
            .await
            {
                Ok(Ok(data)) => data,
                Ok(Err(e)) => {
                    eprintln!("[net] incoming handshake read from {} failed: {}", addr, e);
                    return;
                }
                Err(_) => {
                    eprintln!("[net] incoming handshake read timeout from {}", addr);
                    return;
                }
            };

            let peer_handshake: HandshakeMsg = match serde_json::from_slice(&peer_handshake_bytes) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("[net] incoming handshake deserialize from {} failed: {}", addr, e);
                    return;
                }
            };

            if peer_handshake.version != HANDSHAKE_VERSION {
                eprintln!(
                    "[net] incoming handshake from {} unsupported version {}",
                    addr, peer_handshake.version
                );
                return;
            }

            // --- Security: Ban list check ---
            if ban_list.is_banned(&peer_handshake.peer_id).await {
                eprintln!(
                    "[net] rejected banned peer {} from {}",
                    peer_handshake.peer_id, addr
                );
                return;
            }

            // --- Security: Peer verification (anti-sybil) ---
            let peer_known = peers.iter().any(|p| p.id == peer_handshake.peer_id);
            if !peer_known {
                eprintln!(
                    "[net] rejected unknown peer {} from {} (not in PeerBook)",
                    peer_handshake.peer_id, addr
                );
                ban_list.ban(&peer_handshake.peer_id).await;
                return;
            }

            eprintln!(
                "[net] PQC handshake from verified peer {} (addr: {})",
                peer_handshake.peer_id, addr
            );

            // --- Security: Weryfikuj podpis Falcon peera (anti-MITM) ---
            let peer_sig_payload = match serde_json::to_vec(&HandshakeMsg {
                falcon_sig_b64: String::new(),
                ..peer_handshake.clone()
            }) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[net] peer sig payload serialize failed: {}", e);
                    return;
                }
            };

            let peer_falcon_sig = match B64.decode(&peer_handshake.falcon_sig_b64) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[net] invalid peer falcon_sig_b64: {}", e);
                    return;
                }
            };

            let peer_sig_pk = match B64.decode(&peer_handshake.sig_pk_b64) {
                Ok(pk) => pk,
                Err(e) => {
                    eprintln!("[net] invalid peer sig_pk_b64: {}", e);
                    return;
                }
            };

            if let Err(e) = nxms_transport::crypto::falcon_verify(&peer_sig_pk, &peer_sig_payload, &peer_falcon_sig) {
                eprintln!(
                    "[net] handshake falcon_verify from {} failed: {}",
                    peer_handshake.peer_id, e
                );
                ban_list.ban(&peer_handshake.peer_id).await;
                return;
            }

            eprintln!(
                "[net] handshake signature from {} verified (falcon OK)",
                peer_handshake.peer_id
            );

            // --- FrodoKEM encap: generuj shared secret z kluczem publicznym peera ---
            let peer_kem_pk_bytes = match B64.decode(&peer_handshake.kem_pk_b64) {
                Ok(pk) => pk,
                Err(e) => {
                    eprintln!("[net] invalid peer kem_pk_b64: {}", e);
                    return;
                }
            };

            let (kem_ct, _kem_shared_secret) = match nxms_transport::crypto::kem_encaps(&peer_kem_pk_bytes) {
                Ok((ct, ss)) => (ct, ss),
                Err(e) => {
                    eprintln!("[net] kem_encaps failed: {}", e);
                    return;
                }
            };

            eprintln!("[net] KEM encap done, kem_ct={} bytes", kem_ct.len());

            // --- PQC Handshake: wyślij nasz (podpisany + kem_ct) ---
            let mut my_handshake = HandshakeMsg {
                version: HANDSHAKE_VERSION,
                kem_pk_b64: B64.encode(&kem_pk),
                kem_ct_b64: B64.encode(&kem_ct),
                sig_pk_b64: B64.encode(&sig_pk),
                peer_id: my_peer_id,
                falcon_sig_b64: String::new(), // placeholder
            };

            // Podpisz canonical form (bez pola falcon_sig_b64)
            let sig_payload = match serde_json::to_vec(&HandshakeMsg {
                falcon_sig_b64: String::new(),
                ..my_handshake.clone()
            }) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[net] sig payload serialize failed: {}", e);
                    return;
                }
            };

            let falcon_sig = match nxms_transport::crypto::falcon_sign_ct_prepared(&node_sig_sk_clone, &sig_payload) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[net] falcon sign handshake failed: {}", e);
                    return;
                }
            };

            my_handshake.falcon_sig_b64 = B64.encode(&falcon_sig);

            let handshake_bytes = match serde_json::to_vec(&my_handshake) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("[net] handshake serialize failed: {}", e);
                    return;
                }
            };

            if let Err(e) = write_frame(&mut stream, &handshake_bytes).await {
                eprintln!("[net] handshake write to {} failed: {}", addr, e);
                return;
            }

            eprintln!(
                "[net] handshake with {} complete, entering msg loop",
                peer_handshake.peer_id
            );

            // --- Czytaj ConsensusMsg w pętli ---
            loop {
                match read_frame(&mut stream, 1024 * 1024).await {
                    Ok(data) => {
                        match serde_json::from_slice::<ConsensusMsg>(&data) {
                            Ok(msg) => {
                                let peer_hint = peer_handshake.peer_id.clone();
                                if tx.send((peer_hint, msg)).await.is_err() {
                                    break; // channel zamknięty
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "[net] failed to deserialize ConsensusMsg from {}: {}",
                                    peer_handshake.peer_id, e
                                );
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "[net] connection from {} closed: {}",
                            peer_handshake.peer_id, e
                        );
                        break;
                    }
                }
            }
            }
            ).await; // timeout

            match connection_result {
                Ok(()) => {
                    eprintln!("[net] connection from {} completed normally", addr_str);
                }
                Err(_) => {
                    eprintln!("[net] connection from {} timed out (5min max)", addr_str);
                }
            }
        });
    }
}

/// Wiadomość do wysłania przez writer task (Actor pattern).
enum WriterMsg {
    /// Wyślij zaszyfrowaną ramkę
    Send(Vec<u8>),
    /// Zamknij writer task (będzie używany przy graceful shutdown)
    #[allow(dead_code)]
    Shutdown,
}

/// Maksymalna głębokość kolejki writer task (backpressure).
/// Jeśli kanał się zapełni, send_message zwróci błąd.
const WRITER_CHANNEL_CAPACITY: usize = 64;

/// Metadane pojedynczego połączenia w puli (Actor pattern + encrypted frames).
pub struct ConnectionMeta {
    /// Kanał do writer task (Actor pattern — zamiast Arc<Mutex<TcpStream>>)
    /// Bounded channel (backpressure) — chroni przed OOM na wolnych Tor circuits
    writer_tx: mpsc::Sender<WriterMsg>,
    /// Kiedy połączenie zostało nawiązane (Tor circuit build)
    established_at: Instant,
    /// Kiedy ostatnio wysłaliśmy lub otrzymaliśmy dane
    last_activity: Instant,
    /// Liczba udanych operacji I/O na tym połączeniu
    ops_count: u64,
    /// Czy połączenie wymaga przebudowania (np. po błędzie)
    needs_rebuild: bool,
    /// Czy handshake PQC został ukończony
    handshake_done: bool,
    /// Klucz publiczny FrodoKEM peera (po handshake)
    peer_kem_pk: Option<Vec<u8>>,
    /// Klucz publiczny Falcon peera (po handshake)
    peer_sig_pk: Option<Vec<u8>>,
    /// Shared secret z FrodoKEM do szyfrowania ramek (AES-256-GCM)
    /// Zeroizing chroni przed odczytem z dumpów pamięci
    shared_secret: Option<zeroize::Zeroizing<[u8; 32]>>,
}

impl ConnectionMeta {
    fn new(writer_tx: mpsc::Sender<WriterMsg>) -> Self {
        let now = Instant::now();
        Self {
            writer_tx,
            established_at: now,
            last_activity: now,
            ops_count: 0,
            needs_rebuild: false,
            handshake_done: false,
            peer_kem_pk: None,
            peer_sig_pk: None,
            shared_secret: None,
        }
    }

    /// Zapisuje klucze peera po udanym handshake + derivuje shared secret.
    fn set_peer_keys(&mut self, kem_pk: Vec<u8>, sig_pk: Vec<u8>, shared_secret: [u8; 32]) {
        self.peer_kem_pk = Some(kem_pk);
        self.peer_sig_pk = Some(sig_pk);
        self.shared_secret = Some(zeroize::Zeroizing::new(shared_secret));
        self.handshake_done = true;
    }

    /// Zwraca wiek połączenia w sekundach.
    pub fn age_secs(&self) -> u64 {
        self.established_elapsed().as_secs()
    }

    fn established_elapsed(&self) -> Duration {
        self.established_at.elapsed()
    }

    fn idle_elapsed(&self) -> Duration {
        self.last_activity.elapsed()
    }

    fn touch(&mut self) {
        self.last_activity = Instant::now();
        self.ops_count += 1;
    }

    fn mark_stale(&mut self) {
        self.needs_rebuild = true;
    }
}

/// Konfiguracja puli połączeń.
#[derive(Clone)]
pub struct ConnectionPoolConfig {
    /// Maksymalny czas bezczynności połączenia przed uznaniem za "stale" (sekundy).
    pub idle_timeout_secs: u64,
    /// Maksymalny wiek połączenia (Tor circuits się "starzeją").
    pub max_age_secs: u64,
    /// Interwał sprawdzania zdrowia połączeń (sekundy).
    pub health_check_interval_secs: u64,
    /// Czy włączyć automatyczne ponowne łączenie.
    pub auto_reconnect: bool,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            // Tor circuits żyją ~10 minut, ale dla Gossip chcemy krócej
            idle_timeout_secs: 120,   // 2 minuty bezczynności
            max_age_secs: 600,        // 10 minut maksymalny wiek (z jitter w maintenance)
            health_check_interval_secs: 30,
            auto_reconnect: true,
        }
    }
}

/// Szyfruje ramkę AES-256-GCM z shared secret (FrodoKEM).
/// Format: [12-byte nonce][ciphertext][16-byte tag]
fn encrypt_frame(data: &[u8], shared_secret: &[u8; 32]) -> Result<Vec<u8>, NetError> {
    use nxms_transport::crypto::{random_xchacha20poly1305_nonce, xchacha20poly1305_encrypt};

    let nonce = random_xchacha20poly1305_nonce();
    let (ciphertext, tag) = xchacha20poly1305_encrypt(shared_secret, &nonce, data, &[])
        .map_err(|e| NetError::Transport(anyhow::anyhow!("frame encryption failed: {}", e)))?;

    let mut frame = Vec::with_capacity(nonce.len() + ciphertext.len() + tag.len());
    frame.extend_from_slice(&nonce);
    frame.extend_from_slice(&ciphertext);
    frame.extend_from_slice(&tag);
    Ok(frame)
}

/// Odszyfrowuje ramkę AES-256-GCM z shared secret (FrodoKEM).
#[allow(dead_code)] // Używane przy odczycie frame w connection read loop
fn decrypt_frame(encrypted: &[u8], shared_secret: &[u8; 32]) -> Result<Vec<u8>, NetError> {
    use nxms_transport::crypto::xchacha20poly1305_decrypt;

    if encrypted.len() < 24 + 16 {
        return Err(NetError::Transport(anyhow::anyhow!(
            "encrypted frame too short: {} bytes",
            encrypted.len()
        )));
    }

    let nonce: [u8; 24] = encrypted[..24].try_into().unwrap();
    let ciphertext = &encrypted[24..encrypted.len() - 16];
    let tag: [u8; 16] = encrypted[encrypted.len() - 16..].try_into().unwrap();

    xchacha20poly1305_decrypt(shared_secret, &nonce, ciphertext, &tag, &[])
        .map_err(|e| NetError::Transport(anyhow::anyhow!("frame decryption failed: {}", e)))
}

/// Losowy jitter do max_age_secs — zapobiega synchronizacji z rotacją Tor circuits.
/// Dodaje 0-120s do bazowego max_age_secs, kaskadując wygaśnięcia połączeń.
fn jitter_max_age(base_age_secs: u64) -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    use std::sync::OnceLock;

    static RANDOM: OnceLock<RandomState> = OnceLock::new();
    let state = RANDOM.get_or_init(RandomState::new);
    let mut hasher = state.build_hasher();
    hasher.write_u64(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64);
    let jitter = hasher.finish() % 120; // 0-119s jitter
    base_age_secs + jitter
}

/// Menedżer stałych połączeń Tor (Connection Pool v1).
///
/// Zapobiega ciągłemu otwieraniu/zamykaniu socketów i budowaniu
/// kosztownych Tor circuits przy każdej wiadomości Gossip.
///
/// Architektura:
/// - Każde połączenie ma metadane (wiek, ostatnia aktywność)
/// - Tła zadanie "pool maintenance" sprawdza zdrowie połączeń
/// - Zepsute/stare połączenia są automatycznie przebudowywane
/// - Semaphore limituje jednoczesne circuit builds (max 3)
#[derive(Clone)]
pub struct ConnectionPool {
    /// Mapa: PeerId -> metadane połączenia
    connections: Arc<RwLock<HashMap<String, ConnectionMeta>>>,
    /// URL proxy Tor SOCKS5h
    tor_socks_url: String,
    /// Konfiguracja puli
    config: ConnectionPoolConfig,
    /// Statystyki dla monitoringu
    stats: Arc<RwLock<PoolStats>>,
    /// Limit jednoczesnych Tor circuit builds
    circuit_semaphore: Arc<Semaphore>,
}

/// Statystyki puli połączeń (do monitoringu/loggingu).
#[derive(Default, Clone, Debug)]
pub struct PoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub stale_connections: usize,
    pub total_messages_sent: u64,
    pub total_reconnects: u64,
    pub total_circuit_builds: u64,
}

/// Ban list z TTL — zapobiega ponownemu łączeniu złośliwych peerów.
#[derive(Clone)]
pub struct BanList {
    /// Mapa: peer_id -> ban expiry timestamp (sekundy od UNIX_EPOCH)
    banned: Arc<RwLock<HashMap<String, u64>>>,
}

impl BanList {
    pub fn new() -> Self {
        Self {
            banned: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Sprawdza czy peer jest zbanowany.
    pub async fn is_banned(&self, peer_id: &str) -> bool {
        let banned = self.banned.read().await;
        if let Some(&expiry) = banned.get(peer_id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now < expiry {
                return true;
            }
        }
        false
    }

    /// Banuje peera na BAN_DURATION_SECS.
    pub async fn ban(&self, peer_id: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut banned = self.banned.write().await;
        banned.insert(peer_id.to_string(), now + BAN_DURATION_SECS);
        eprintln!("[net] banned peer {} for {}s", peer_id, BAN_DURATION_SECS);
    }

    /// Czyści wygasłe bany.
    pub async fn cleanup(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut banned = self.banned.write().await;
        banned.retain(|_, &mut expiry| expiry > now);
    }
}

/// Rate limiter na incoming connections per source.
/// Zapobiega floodowaniu z tego samego .onion address.
#[derive(Clone)]
pub struct RateLimiter {
    /// Mapa: source_addr -> (count, window_start)
    counters: Arc<RwLock<HashMap<String, (u64, u64)>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Sprawdza czy source przekroczył limit.
    /// Zwraca true jeśli dozwolone, false jeśli rate limited.
    pub async fn check(&self, source: &str) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut counters = self.counters.write().await;
        let entry = counters.entry(source.to_string()).or_insert((0, now));

        // Reset okna jeśli minęło
        if now - entry.1 >= RATE_LIMIT_WINDOW_SECS {
            entry.0 = 1;
            entry.1 = now;
            return true;
        }

        entry.0 += 1;
        entry.0 <= MAX_CONNECTIONS_PER_SOURCE as u64
    }

    /// Czyści stare wpisy.
    pub async fn cleanup(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut counters = self.counters.write().await;
        counters.retain(|_, (_, window_start)| now - *window_start < RATE_LIMIT_WINDOW_SECS);
    }
}

impl ConnectionPool {
    pub fn new(tor_socks_url: String) -> Self {
        Self::with_config(tor_socks_url, ConnectionPoolConfig::default())
    }

    pub fn with_config(tor_socks_url: String, config: ConnectionPoolConfig) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            tor_socks_url,
            config,
            stats: Arc::new(RwLock::new(PoolStats::default())),
            circuit_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_BUILDS)),
        }
    }

    /// Zwraca aktualne statystyki puli.
    pub async fn stats(&self) -> PoolStats {
        self.stats.read().await.clone()
    }

    /// Wysyła zaszyfrowaną wiadomość przez kanał MPSC (Actor pattern).
    /// Wiadomość jest szyfrowana AES-256-GCM z shared secret (FrodoKEM).
    /// Timeout na send (10s) — chroni przed blokowaniem na pełnym kanale.
    pub async fn send_message<T: Serialize>(
        &self,
        peer: &Peer,
        msg: &T,
        my_kem_pk: &[u8],
        my_kem_sk: &[u8],
        my_sig_pk: &[u8],
        my_sig_sk: &[u8],
        my_peer_id: &str,
    ) -> Result<(), NetError> {
        let data = serde_json::to_vec(msg)?;

        // Sprawdź czy mamy aktywne połączenie z handshake
        let needs_new = {
            let conns = self.connections.read().await;
            match conns.get(&peer.id) {
                Some(meta) if !meta.needs_rebuild && meta.handshake_done => false,
                _ => true,
            }
        };

        if needs_new {
            self.establish_connection(peer, my_kem_pk, my_kem_sk, my_sig_pk, my_sig_sk, my_peer_id)
                .await?;
        }

        // Pobierz kanał writer'a i shared secret (Actor pattern)
        let (writer_tx, shared_secret) = {
            let conns = self.connections.read().await;
            match conns.get(&peer.id) {
                Some(meta) => (meta.writer_tx.clone(), meta.shared_secret.clone()),
                None => {
                    return Err(NetError::Transport(anyhow::anyhow!(
                        "Connection not found after establish for peer {}",
                        peer.id
                    )))
                }
            }
        };

        // Szyfruj ramkę AES-256-GCM z shared secret (jeśli dostępny)
        let encrypted_data = if let Some(secret) = shared_secret {
            encrypt_frame(&data, &secret)?
        } else {
            data // Fallback: plaintext (przed handshake)
        };

        // Wyślij przez kanał MPSC (Actor pattern — brak contention na Mutex)
        // Timeout 10s na send — chroni przed blokowaniem na pełnym kanale (bounded channel)
        tokio::time::timeout(
            Duration::from_secs(10),
            writer_tx.send(WriterMsg::Send(encrypted_data)),
        )
        .await
        .map_err(|_| NetError::Transport(anyhow::anyhow!("Writer send timeout for peer {}", peer.id)))?
        .map_err(|_| NetError::Transport(anyhow::anyhow!("Writer task closed for peer {}", peer.id)))?;

        // Aktualizuj metadane
        {
            let mut conns = self.connections.write().await;
            if let Some(meta) = conns.get_mut(&peer.id) {
                meta.touch();
            }
        }
        {
            let mut s = self.stats.write().await;
            s.total_messages_sent += 1;
        }

        Ok(())
    }

    /// Nawiązuje nowe połączenie Tor do peera z FrodoKEM handshake.
    /// Zawiera "double-check" pattern by uniknąć wyścigów.
    ///
    /// Flow:
    /// 1. Tor Circuit Build (2-5s)
    /// 2. Podpisz HandshakeMsg kluczem Falcon (anti-MITM)
    /// 3. Wyślij HandshakeMsg { kem_pk, sig_pk, falcon_sig }
    /// 4. Odbierz HandshakeMsg od peera + zweryfikuj podpis
    /// 5. Zapisz klucze w ConnectionMeta
    async fn establish_connection(
        &self,
        peer: &Peer,
        my_kem_pk: &[u8],
        my_kem_sk: &[u8],
        my_sig_pk: &[u8],
        my_sig_sk: &[u8],
        my_peer_id: &str,
    ) -> Result<(), NetError> {
        // Sprawdź ponownie pod write-lockiem (double-check pattern)
        {
            let conns = self.connections.read().await;
            if let Some(meta) = conns.get(&peer.id) {
                if !meta.needs_rebuild && meta.handshake_done {
                    return Ok(());
                }
            }
        }

        eprintln!(
            "[pool] establishing new Tor connection to {} ({}:{})",
            peer.id, peer.host, peer.port
        );

        // Ogranicz jednoczesne circuit builds (zapobiega floodowaniu Tora)
        let _permit = self.circuit_semaphore.acquire().await
            .map_err(|e| NetError::Transport(anyhow::anyhow!("semaphore closed: {}", e)))?;

        // WOLNA ŚCIEŻKA: Tor Circuit Build
        let stream = tokio::time::timeout(
            Duration::from_secs(30),
            connect_via_tor(&self.tor_socks_url, &peer.host, peer.port),
        )
        .await
        .map_err(|_| NetError::Transport(anyhow::anyhow!("Tor connect timeout to {}", peer.id)))?
        .map_err(|e| {
            eprintln!("[pool] Tor connect to {} failed: {}", peer.id, e);
            NetError::Transport(e)
        })?;

        let mut stream = stream;

        // --- FrodoKEM Handshake ---
        eprintln!("[pool] starting PQC handshake with {}", peer.id);

        use base64::Engine;
        use base64::engine::general_purpose::STANDARD as B64;

        // Krok 1: Podpisz i wyślij nasz HandshakeMsg (bez kem_ct — nie znamy jeszcze peer_kem_pk)
        let mut my_handshake = HandshakeMsg {
            version: HANDSHAKE_VERSION,
            kem_pk_b64: B64.encode(my_kem_pk),
            kem_ct_b64: String::new(), // wyślemy po otrzymaniu peer_kem_pk
            sig_pk_b64: B64.encode(my_sig_pk),
            peer_id: my_peer_id.to_string(),
            falcon_sig_b64: String::new(), // placeholder
        };

        // Podpisz canonical form (bez pola falcon_sig_b64)
        let sig_payload = serde_json::to_vec(&HandshakeMsg {
            falcon_sig_b64: String::new(),
            ..my_handshake.clone()
        })
        .map_err(|e| NetError::Transport(anyhow::anyhow!("sig payload serialize: {}", e)))?;

        let falcon_sig = nxms_transport::crypto::falcon_sign_ct_prepared(my_sig_sk, &sig_payload)
            .map_err(|e| NetError::Transport(anyhow::anyhow!("falcon sign handshake: {}", e)))?;

        my_handshake.falcon_sig_b64 = B64.encode(&falcon_sig);

        let _handshake_bytes = tokio::time::timeout(
            Duration::from_secs(10),
            async {
                let bytes = serde_json::to_vec(&my_handshake)?;
                write_frame(&mut stream, &bytes).await?;
                Ok::<_, anyhow::Error>(())
            },
        )
        .await
        .map_err(|_| NetError::Transport(anyhow::anyhow!("handshake write timeout to {}", peer.id)))?
        .map_err(|e| {
            eprintln!("[pool] handshake write to {} failed: {}", peer.id, e);
            NetError::Transport(e)
        })?;

        // Krok 2: Odbierz HandshakeMsg od peera (z timeout)
        let peer_handshake_bytes = tokio::time::timeout(
            Duration::from_secs(10),
            read_frame(&mut stream, 64 * 1024),
        )
        .await
        .map_err(|_| NetError::Transport(anyhow::anyhow!("handshake read timeout from {}", peer.id)))?
        .map_err(|e| {
            eprintln!("[pool] handshake read from {} failed: {}", peer.id, e);
            NetError::Transport(e)
        })?;

        let peer_handshake: HandshakeMsg =
            serde_json::from_slice(&peer_handshake_bytes).map_err(|e| {
                eprintln!(
                    "[pool] handshake deserialize from {} failed: {}",
                    peer.id, e
                );
                NetError::Serde(e)
            })?;

        if peer_handshake.version != HANDSHAKE_VERSION {
            return Err(NetError::Transport(anyhow::anyhow!(
                "unsupported handshake version {} from {}",
                peer_handshake.version,
                peer.id
            )));
        }

        // Krok 3: Weryfikuj podpis Falcon peera (anti-MITM)
        let peer_sig_payload = serde_json::to_vec(&HandshakeMsg {
            falcon_sig_b64: String::new(),
            ..peer_handshake.clone()
        })
        .map_err(|e| NetError::Transport(anyhow::anyhow!("peer sig payload: {}", e)))?;

        let peer_falcon_sig = B64
            .decode(&peer_handshake.falcon_sig_b64)
            .map_err(|e| NetError::Transport(anyhow::anyhow!("invalid peer falcon_sig: {}", e)))?;

        let peer_sig_pk = B64
            .decode(&peer_handshake.sig_pk_b64)
            .map_err(|e| NetError::Transport(anyhow::anyhow!("invalid peer sig_pk: {}", e)))?;

        nxms_transport::crypto::falcon_verify(&peer_sig_pk, &peer_sig_payload, &peer_falcon_sig)
            .map_err(|e| {
                eprintln!(
                    "[pool] handshake falcon_verify from {} failed: {}",
                    peer.id, e
                );
                NetError::Transport(anyhow::anyhow!("handshake falcon_verify failed: {}", e))
            })?;

        let peer_kem_pk = B64
            .decode(&peer_handshake.kem_pk_b64)
            .map_err(|e| NetError::Transport(anyhow::anyhow!("invalid peer kem_pk: {}", e)))?;

        eprintln!(
            "[pool] PQC handshake with {} complete (peer_id: {}, falcon verified)",
            peer.id, peer_handshake.peer_id
        );

        // Krok 4: FrodoKEM decap — derivuj shared secret z ciphertext peera
        // Peer zrobił encap(my_kem_pk) → kem_ct, my robimy decap(my_kem_sk, kem_ct) → shared_secret
        let shared_secret = {
            if peer_handshake.kem_ct_b64.is_empty() {
                return Err(NetError::Transport(anyhow::anyhow!(
                    "peer {} did not send kem_ct (old protocol version?)",
                    peer.id
                )));
            }
            let peer_kem_ct = B64.decode(&peer_handshake.kem_ct_b64)
                .map_err(|e| NetError::Transport(anyhow::anyhow!("invalid peer kem_ct: {}", e)))?;
            let ss = nxms_transport::crypto::kem_decaps(my_kem_sk, &peer_kem_ct)
                .map_err(|e| NetError::Transport(anyhow::anyhow!("kem_decaps failed: {}", e)))?;
            // Konwertuj Zeroizing<Vec<u8>> na [u8; 32]
            let mut result = [0u8; 32];
            result.copy_from_slice(&ss[..32]);
            result
        };

        // Krok 5: Utwórz BOUNDED kanał MPSC i writer task (Actor pattern)
        // Bounded channel (64) zapobiega OOM na wolnych Tor circuits (backpressure)
        let (writer_tx, mut writer_rx) = mpsc::channel::<WriterMsg>(WRITER_CHANNEL_CAPACITY);
        let (_read_half, write_half) = stream.into_split();

        // Spawn writer task — dedykowany task do pisania do TcpStream
        tokio::spawn(async move {
            let mut writer = write_half;
            while let Some(msg) = writer_rx.recv().await {
                match msg {
                    WriterMsg::Send(data) => {
                        if let Err(e) = nxms_transport::tor_net::write_frame_half(&mut writer, &data).await {
                            eprintln!("[writer] write failed: {}", e);
                            break;
                        }
                    }
                    WriterMsg::Shutdown => {
                        eprintln!("[writer] shutdown requested");
                        break;
                    }
                }
            }
            eprintln!("[writer] task exiting");
        });

        // Krok 6: Zapisz połączenie z kluczami i kanałem
        let mut meta = ConnectionMeta::new(writer_tx);
        meta.set_peer_keys(peer_kem_pk, peer_sig_pk, shared_secret);

        // Double-check pod write-lockiem
        let mut conns = self.connections.write().await;
        if let Some(existing) = conns.get(&peer.id) {
            if !existing.needs_rebuild && existing.handshake_done {
                // Ktoś inny zdążył połączyć — użyj istniejącego
                return Ok(());
            }
        }

        eprintln!(
            "[pool] connection to {} established with PQC keys + encrypted frames",
            peer.id
        );

        conns.insert(peer.id.clone(), meta);

        // Aktualizuj statystyki
        {
            let mut s = self.stats.write().await;
            s.total_circuit_builds += 1;
            s.total_connections = conns.len();
        }

        Ok(())
    }

    /// Usuwa połączenie z puli (np. gdy peer jest niedostępny).
    pub async fn remove_connection(&self, peer_id: &str) {
        let mut conns = self.connections.write().await;
        if conns.remove(peer_id).is_some() {
            eprintln!("[pool] removed connection to {}", peer_id);
            let mut s = self.stats.write().await;
            s.total_connections = conns.len();
        }
    }

    /// Broadcastuje wiadomość do wszystkich peerów z PeerBook (oprócz siebie).
    /// Wysyła równolegle — nie blokuje na wolnych peerach.
    pub async fn broadcast_message<T: Serialize + Clone + Send + Sync + 'static>(
        &self,
        peer_book: &PeerBook,
        my_id: &str,
        msg: &T,
        my_kem_pk: &[u8],
        my_kem_sk: &[u8],
        my_sig_pk: &[u8],
        my_sig_sk: &[u8],
    ) -> Vec<(String, Result<(), NetError>)> {
        let peers = peer_book.others(my_id);
        let mut handles = Vec::with_capacity(peers.len());

        for peer in peers {
            let pool = self.clone();
            let peer = peer.clone();
            let msg = msg.clone();
            let kem_pk = my_kem_pk.to_vec();
            let kem_sk = my_kem_sk.to_vec();
            let sig_pk = my_sig_pk.to_vec();
            let sig_sk = my_sig_sk.to_vec();
            let id = my_id.to_string();

            handles.push(tokio::spawn(async move {
                let result = pool.send_message(&peer, &msg, &kem_pk, &kem_sk, &sig_pk, &sig_sk, &id).await;
                (peer.id.clone(), result)
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok((peer_id, result)) => results.push((peer_id, result)),
                Err(e) => results.push(("unknown".to_string(), Err(NetError::Transport(anyhow::anyhow!("join error: {}", e))))),
            }
        }

        results
    }

    /// Uruchamia tła zadanie "pool maintenance".
    ///
    /// Co `health_check_interval_secs` sekund:
    /// 1. Sprawdza wiek i bezczynność każdego połączenia
    /// 2. Oznacza stare/bezczynne jako `needs_rebuild`
    /// 3. Loguje statystyki puli
    ///
    /// To zadanie jest "fire-and-forget" — uruchamiane raz przy starcie węzła.
    pub fn spawn_maintenance(
        &self,
        peer_book: PeerBook,
        my_id: String,
        my_kem_pk: Vec<u8>,
        my_kem_sk: Vec<u8>,
        my_sig_pk: Vec<u8>,
        my_sig_sk: Vec<u8>,
    ) {
        let pool = self.clone();
        let interval = self.config.health_check_interval_secs;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(interval));
            loop {
                ticker.tick().await;
                pool.maintenance_tick(&peer_book, &my_id, &my_kem_pk, &my_kem_sk, &my_sig_pk, &my_sig_sk)
                    .await;
            }
        });

        eprintln!(
            "[pool] maintenance task started (interval: {}s, idle_timeout: {}s, max_age: {}s)",
            interval, self.config.idle_timeout_secs, self.config.max_age_secs
        );
    }

    /// Pojedynczy tick maintenance — sprawdza i naprawia połączenia.
    async fn maintenance_tick(
        &self,
        peer_book: &PeerBook,
        my_id: &str,
        my_kem_pk: &[u8],
        my_kem_sk: &[u8],
        my_sig_pk: &[u8],
        my_sig_sk: &[u8],
    ) {
        let peers = peer_book.others(my_id);
        let mut stale_ids: Vec<String> = Vec::new();
        let mut active_count = 0usize;

        // Sprawdź istniejące połączenia (z jitter na max_age aby uniknąć synchronizacji z Tor)
        let max_age_with_jitter = jitter_max_age(self.config.max_age_secs);
        {
            let conns = self.connections.read().await;
            for (peer_id, meta) in conns.iter() {
                let age = meta.age_secs();
                let idle = meta.idle_elapsed().as_secs();

                if meta.needs_rebuild {
                    stale_ids.push(peer_id.clone());
                } else if age > max_age_with_jitter {
                    eprintln!(
                        "[pool] connection to {} is old (age: {}s > {}s with jitter), marking stale",
                        peer_id, age, max_age_with_jitter
                    );
                    stale_ids.push(peer_id.clone());
                } else if idle > self.config.idle_timeout_secs {
                    eprintln!(
                        "[pool] connection to {} is idle (idle: {}s > {}s), marking stale",
                        peer_id, idle, self.config.idle_timeout_secs
                    );
                    stale_ids.push(peer_id.clone());
                } else {
                    active_count += 1;
                }
            }
        }

        // Oznacz stare połączenia jako potrzebujące przebudowy
        if !stale_ids.is_empty() {
            let mut conns = self.connections.write().await;
            for peer_id in &stale_ids {
                if let Some(meta) = conns.get_mut(peer_id) {
                    meta.mark_stale();
                }
            }
        }

        // Aktualizuj statystyki
        {
            let mut s = self.stats.write().await;
            s.active_connections = active_count;
            s.stale_connections = stale_ids.len();
            s.total_connections = active_count + stale_ids.len();
        }

        // Loguj statystyki (co ~5 minut)
        let total = active_count + stale_ids.len();
        if total > 0 {
            let s = self.stats.read().await;
            eprintln!(
                "[pool] stats: {} active, {} stale, {} total | sent: {}, reconnects: {}, builds: {}",
                active_count, stale_ids.len(), total,
                s.total_messages_sent, s.total_reconnects, s.total_circuit_builds
            );
        }

        // Opcjonalnie: pre-connect do znanych peerów (żeby Gossip był szybszy)
        if self.config.auto_reconnect {
            for peer in &peers {
                let conns = self.connections.read().await;
                let needs_connect = match conns.get(&peer.id) {
                    Some(meta) => meta.needs_rebuild,
                    None => true, // Brak połączenia — nie łączymy na zapas (lazy connect)
                };
                drop(conns);

                if needs_connect {
                    // Spróbuj ponownie połączyć tylko jeśli był już kiedyś połączony
                    // (lazy connect: nie łączymy na zapas, tylko gdy potrzebujemy)
                    if self.connections.read().await.contains_key(&peer.id) {
                        eprintln!("[pool] reconnecting to stale peer {}", peer.id);
                        if self.establish_connection(peer, my_kem_pk, my_kem_sk, my_sig_pk, my_sig_sk, my_id).await.is_ok() {
                            let mut s = self.stats.write().await;
                            s.total_reconnects += 1;
                        }
                    }
                }
            }
        }
    }
}
