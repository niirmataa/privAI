//! State Sync — mechanizm catch-up dla węzłów które były offline.
//!
//! Gdy węzeł wykryje, że jest z tyłu za siecią:
//! 1. Wysyła SyncRequest do peerów
//! 2. Odbiera SyncResponse z blokami + QC
//! 3. Importuje bloki sekwencyjnie i finalizuje z QC

use nxms_transport::peers::PeerBook;
use privai_chain::{Block, ConsensusMsg, Hash32, QuorumCertificate, VoteType};

use crate::net::{self, NetConfig, NetError};
use crate::node::{NodeError, PrivaiNode};
use privai_ledger::LedgerStore;
use privai_proof::{BlockArtifactVerifier, ProofVerifier};
use privai_proof::store::ProofArtifactStore;

/// Maksymalna liczba bloków w jednej odpowiedzi sync.
pub const MAX_SYNC_BATCH: u64 = 100;

/// Wysyła SyncRequest do losowego peera.
/// Zwraca bloki i QCs do zaimportowania.
pub async fn request_blocks(
    peer_book: &PeerBook,
    my_id: &str,
    tor_socks_url: &str,
    from_height: u64,
    to_height: u64,
    my_pk_hash: Hash32,
) -> Result<Option<(Vec<Block>, Vec<QuorumCertificate>)>, SyncError> {
    // Wybieramy pierwszego dostępnego peera (oprócz siebie)
    let peer = peer_book
        .others(my_id)
        .into_iter()
        .next()
        .ok_or(SyncError::NoPeers)?;

    let msg = ConsensusMsg::SyncRequest {
        from_height,
        to_height,
        requester_pk_hash: my_pk_hash,
    };

    // Wysyłamy request
    net::send_to_peer(peer, tor_socks_url, &msg)
        .await
        .map_err(SyncError::Net)?;

    // W v0 nie czekamy na response (fire-and-forget).
    // Response przyjdzie jako nowa wiadomość w consensus loop.
    // W v1 można dodać request-response correlation.
    Ok(None)
}

/// Obsługuje przychodzący SyncRequest — wysyła bloki do requestera.
pub fn handle_sync_request<S: LedgerStore, V: ProofVerifier, A: ProofArtifactStore, P: BlockArtifactVerifier>(
    node: &PrivaiNode<S, V, A, P>,
    _block_cache: &std::collections::HashMap<Hash32, Block>,
    from_height: u64,
    to_height: u64,
    requester_pk_hash: Hash32,
    net_config: &NetConfig,
    peer_book: &PeerBook,
) {
    let current_height = node.ledger().snapshot().height;
    let from = from_height.min(current_height);
    let to = to_height.min(current_height).min(from + MAX_SYNC_BATCH);

    if from > to {
        eprintln!("[sync] invalid range: from={} to={}", from, to);
        return;
    }

    // Zbieramy bloki z cache (w v0 cache jest w pamięci)
    // W v1 trzeba czytać z persistent storage
    let blocks = Vec::new();
    let qcs = Vec::new();

    // TODO: W v1 czytaj bloki z ledger storage po height
    // Na razie logujemy — full sync wymaga persistent block storage
    eprintln!(
        "[sync] SyncRequest from {:?} for heights {}..{} (current={})",
        &requester_pk_hash[..8],
        from,
        to,
        current_height
    );

    if blocks.is_empty() && from < current_height {
        eprintln!(
            "[sync] cannot serve SyncRequest: no persistent block storage yet (v0 limitation)"
        );
    }

    // Wysyłamy response jeśli mamy bloki
    if !blocks.is_empty() {
        let response = ConsensusMsg::SyncResponse {
            blocks,
            qcs,
            sender_pk_hash: node.config().node_pk_hash,
        };

        // Fire-and-forget broadcast (requester dostanie przez Tor)
        let peer_book = peer_book.clone();
        let my_id = net_config.my_peer_id.clone();
        let tor_url = net_config.tor_socks_url.clone();
        tokio::spawn(async move {
            let _ = net::broadcast(&peer_book, &my_id, &tor_url, &response).await;
        });
    }
}

/// Obsługuje przychodzący SyncResponse — importuje bloki i finalizuje.
pub fn handle_sync_response<S: LedgerStore, V: ProofVerifier, A: ProofArtifactStore, P: BlockArtifactVerifier>(
    node: &mut PrivaiNode<S, V, A, P>,
    blocks: Vec<Block>,
    qcs: Vec<QuorumCertificate>,
    sender_pk_hash: Hash32,
) {
    eprintln!(
        "[sync] received {} blocks and {} QCs from {:?}",
        blocks.len(),
        qcs.len(),
        &sender_pk_hash[..8]
    );

    // Sortujemy bloki po height
    let mut sorted_blocks = blocks;
    sorted_blocks.sort_by_key(|b| b.header.height);

    for block in sorted_blocks {
        // Walidacja roots
        if !block.roots_match() {
            eprintln!("[sync] REJECTED block at height={}: roots mismatch", block.header.height);
            continue;
        }

        // Import bloku
        if let Err(e) = node.import_block(&block) {
            eprintln!(
                "[sync] failed to import block at height={}: {}",
                block.header.height, e
            );
            continue;
        }

        eprintln!("[sync] imported block height={}", block.header.height);

        // Szukamy pasujący QC do finalizacji
        for qc in &qcs {
            if qc.block_hash == block.hash() && qc.vote_type == VoteType::Precommit {
                match node.finalize_block_with_qc(&block, qc) {
                    Ok(()) => {
                        eprintln!(
                            "[sync] FINALIZED block height={} via sync QC",
                            block.header.height
                        );
                    }
                    Err(e) => {
                        eprintln!("[sync] finalization failed: {}", e);
                    }
                }
            }
        }
    }
}

/// Sprawdza czy węzeł potrzebuje sync (jest z tyłu za siecią).
/// Zwraca height do którego trzeba dociągnąć.
pub fn needs_sync(current_height: u64, peer_height_hint: u64) -> Option<(u64, u64)> {
    if peer_height_hint > current_height {
        Some((current_height + 1, peer_height_hint))
    } else {
        None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("no peers available for sync")]
    NoPeers,
    #[error("net error: {0}")]
    Net(#[from] NetError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
}