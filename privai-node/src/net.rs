//! Moduł sieciowy P2P dla konsensusu — Tor-only transport.
//!
//! Każdy validator:
//! - nasłuchuje na Tor hidden service (TCP)
//! - wysyła ConsensusMsg do peerów przez SOCKS5h proxy
//! - odbiera ConsensusMsg od innych validatorów

use tokio::sync::mpsc;

use nxms_transport::peers::{Peer, PeerBook};
use nxms_transport::tor_net::{connect_via_tor, read_frame, write_frame, serve};
use privai_chain::ConsensusMsg;
use serde::{Deserialize, Serialize};

/// Konfiguracja sieciowa węzła konsensusowego.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl Default for NetConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:19000".to_string(),
            tor_socks_url: "socks5h://127.0.0.1:9050".to_string(),
            peers_path: "peers.json".to_string(),
            my_peer_id: "validator-0".to_string(),
        }
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
/// Deserializuje ConsensusMsg i wysyła do channel'a do przetworzenia przez node.
pub async fn run_listener(
    config: NetConfig,
    msg_tx: mpsc::UnboundedSender<(String, ConsensusMsg)>,
) -> Result<(), NetError> {
    let listener = serve(&config.listen_addr).await?;
    eprintln!(
        "[net] listening on {} for consensus messages",
        config.listen_addr
    );

    loop {
        let (mut stream, addr) = listener.accept().await?;
        let tx = msg_tx.clone();

        tokio::spawn(async move {
            // Czytaj frame'y w pętli od tego peera
            loop {
                match read_frame(&mut stream, 1024 * 1024).await {
                    Ok(data) => {
                        match serde_json::from_slice::<ConsensusMsg>(&data) {
                            Ok(msg) => {
                                let peer_hint = addr.to_string();
                                if tx.send((peer_hint, msg)).is_err() {
                                    break; // channel zamknięty
                                }
                            }
                            Err(e) => {
                                eprintln!("[net] failed to deserialize ConsensusMsg from {}: {}", addr, e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        // EOF lub timeout — zamykamy połączenie
                        eprintln!("[net] connection from {} closed: {}", addr, e);
                        break;
                    }
                }
            }
        });
    }
}

/// Wysyła ConsensusMsg do konkretnego peera przez Tor.
pub async fn send_to_peer(
    peer: &Peer,
    tor_socks_url: &str,
    msg: &ConsensusMsg,
) -> Result<(), NetError> {
    let data = serde_json::to_vec(msg)?;
    let mut stream = connect_via_tor(tor_socks_url, &peer.host, peer.port)
        .await
        .map_err(NetError::Transport)?;
    write_frame(&mut stream, &data)
        .await
        .map_err(NetError::Transport)?;
    Ok(())
}

/// Broadcastuje ConsensusMsg do wszystkich peerów z PeerBook (oprócz siebie).
pub async fn broadcast(
    peer_book: &PeerBook,
    my_id: &str,
    tor_socks_url: &str,
    msg: &ConsensusMsg,
) -> Vec<(String, Result<(), NetError>)> {
    let peers = peer_book.others(my_id);
    let mut results = Vec::with_capacity(peers.len());

    for peer in peers {
        let result = send_to_peer(peer, tor_socks_url, msg).await;
        results.push((peer.id.clone(), result));
    }

    results
}