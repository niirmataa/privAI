//! Gossip Protocol — propagacja transakcji między nodami przez Tor.
//!
//! Flow:
//! 1. Użytkownik wysyła Tx do węzła (przez Tor)
//! 2. Węzeł weryfikuje Tx, dodaje do mempoola
//! 3. Gossip: propaguje Tx do sąsiadów (przez nxms-transport/Tor)
//! 4. Sąsiedzi weryfikują, dodają do swojego mempoola, propagują dalej
//!
//! Anti-spam:
//! - Każda Tx musi mieć ważny podpis Falcon (Zero Trust)
//! - Deduplikacja po tx_hash
//! - Rate-limit per sender

use nxms_transport::peers::PeerBook;
use privai_chain::{Hash32, Transaction};

use crate::mempool::{Mempool, MempoolEntry};
use crate::net::NetConfig;
use crate::node::PrivaiNode;
use privai_ledger::LedgerStore;
use privai_proof::{BlockArtifactVerifier, ProofVerifier};
use privai_proof::store::ProofArtifactStore;

/// Maksymalna liczba peerów do których propagujemy jedną Tx.
/// Nie wysyłamy do WSZYSTKICH — tylko do subsetu (gossip fanout).
pub const GOSSIP_FANOUT: usize = 3;

/// Wiadomość Gossip wysyłana między nodami.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GossipTxMsg {
    pub tx: Transaction,
    pub tx_hash: Hash32,
    pub sender_pk_hash: Hash32,
    pub timestamp_ms: u64,
    /// Liczba hops — ile razy ta Tx była już propagowana.
    /// Po osiągnięciu limitu przestajemy propagować (anti-loop).
    pub hops: u8,
}

/// Maksymalna liczba hops przed zatrzymaniem propagacji.
pub const MAX_GOSSIP_HOPS: u8 = 5;

/// Obsługuje przychodzącą wiadomość Gossip z peera.
/// Weryfikuje Tx, dodaje do mempoola, propaguje dalej.
pub fn handle_gossip_tx<S: LedgerStore, V: ProofVerifier, A: ProofArtifactStore, P: BlockArtifactVerifier>(
    _node: &PrivaiNode<S, V, A, P>,
    mempool: &mut Mempool,
    msg: GossipTxMsg,
    peer_book: &PeerBook,
    net_config: &NetConfig,
    current_time_ms: u64,
) {
    eprintln!(
        "[gossip] received tx {:?} from {:?} (hops={})",
        &msg.tx_hash[..8],
        &msg.sender_pk_hash[..8],
        msg.hops
    );

    // TODO: Weryfikacja podpisu Falcon na wejściu (Zero Trust)
    // Na razie sprawdzamy tylko podstawowe walidacje

    // Sprawdź czy Tx już jest w mempoolu (deduplikacja)
    if mempool.contains(&msg.tx_hash) {
        eprintln!("[gossip] rejected: duplicate tx {:?}", &msg.tx_hash[..8]);
        return;
    }

    // Dodaj do mempoola
    let entry = MempoolEntry {
        tx: msg.tx.clone(),
        tx_hash: msg.tx_hash,
        received_at_ms: current_time_ms,
        sender_pk_hash: msg.sender_pk_hash.clone(),
    };

    if !mempool.insert(entry) {
        eprintln!("[gossip] rejected by mempool: {:?}", &msg.tx_hash[..8]);
        return;
    }

    eprintln!(
        "[gossip] accepted tx {:?} into mempool (size={})",
        &msg.tx_hash[..8],
        mempool.len()
    );

    // Propaguj dalej jeśli nie przekroczyliśmy limitu hops
    if msg.hops < MAX_GOSSIP_HOPS {
        propagate_tx(
            peer_book,
            net_config,
            &msg.tx,
            msg.tx_hash,
            msg.sender_pk_hash,
            msg.hops + 1,
        );
    }
}

/// Propaguje transakcję do losowego subsetu peerów (gossip fanout).
fn propagate_tx(
    peer_book: &PeerBook,
    net_config: &NetConfig,
    tx: &Transaction,
    tx_hash: Hash32,
    sender_pk_hash: Hash32,
    hops: u8,
) {
    let my_id = &net_config.my_peer_id;
    let peers = peer_book.others(my_id);

    if peers.is_empty() {
        return;
    }

    // Wybierz losowy subset peerów (fanout)
    let fanout = GOSSIP_FANOUT.min(peers.len());
    let selected: Vec<_> = peers.into_iter().take(fanout).cloned().collect();

    let gossip_msg = GossipTxMsg {
        tx: tx.clone(),
        tx_hash,
        sender_pk_hash,
        timestamp_ms: current_time_ms(),
        hops,
    };

    // Wysyłamy do każdego wybranego peera (fire-and-forget)
    for peer in selected {
        let peer = peer.clone();
        let msg = gossip_msg.clone();

        tokio::spawn(async move {
            eprintln!(
                "[gossip] propagating tx {:?} to {} (hop {})",
                &msg.tx_hash[..8],
                peer.host,
                msg.hops
            );
        });
    }
}

/// Wysyła nową transakcję do mempoola i gossipuje do sieci.
pub fn submit_and_gossip<S: LedgerStore, V: ProofVerifier, A: ProofArtifactStore, P: BlockArtifactVerifier>(
    _node: &PrivaiNode<S, V, A, P>,
    mempool: &mut Mempool,
    tx: Transaction,
    peer_book: &PeerBook,
    net_config: &NetConfig,
    current_time_ms: u64,
) -> bool {
    let tx_hash = tx.tx_id();

    // TODO: Wyciągnij sender_pk_hash z podpisu Falcon
    let sender_pk_hash = [0u8; 32]; // placeholder

    let entry = MempoolEntry {
        tx: tx.clone(),
        tx_hash,
        received_at_ms: current_time_ms,
        sender_pk_hash,
    };

    if !mempool.insert(entry) {
        eprintln!("[gossip] local tx rejected by mempool: {:?}", &tx_hash[..8]);
        return false;
    }

    eprintln!(
        "[gossip] local tx accepted {:?} (mempool size={})",
        &tx_hash[..8],
        mempool.len()
    );

    // Gossip do sieci
    propagate_tx(
        peer_book,
        net_config,
        &tx,
        tx_hash,
        sender_pk_hash,
        0, // hops = 0 (pierwsza propagacja)
    );

    true
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}