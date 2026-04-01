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
/// 10 bloków × ~1MB = ~10MB max payload. Realistyczne dla Tor circuit.
/// Requester wysyła kolejny SyncRequest jeśli potrzebuje więcej.
pub const MAX_SYNC_BATCH: u64 = 10;

/// Wysyła SyncRequest do losowego peera.
/// Zwraca bloki i QCs do zaimportowania.
pub async fn request_blocks(
    peer_book: &PeerBook,
    my_id: &str,
    connection_pool: &net::ConnectionPool,
    from_height: u64,
    to_height: u64,
    my_pk_hash: Hash32,
    node_kem_pk: &[u8],
    node_sig_pk: &[u8],
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

    // Wysyłamy request przez pulę połączeń
    connection_pool.send_message(peer, &msg, node_kem_pk, node_sig_pk, my_id)
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
    _net_config: &NetConfig,
    peer_book: &PeerBook,
    connection_pool: &net::ConnectionPool,
    node_kem_pk: &[u8],
    node_sig_pk: &[u8],
    node_peer_id: &str,
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

        // Znajdź requestera w peer_book i wyślij TYLKO do niego
        // (nie broadcast — SyncResponse może być duży ~10MB)
        if let Some(peer) = peer_book.peers.iter().find(|_p| {
            // TODO: w v1 peer_book powinien mapować pk_hash → peer
            // Na razie wysyłamy do pierwszego dostępnego peera
            true
        }) {
            let peer = peer.clone();
            let pool = connection_pool.clone();
            let kem_pk = node_kem_pk.to_vec();
            let sig_pk = node_sig_pk.to_vec();
            let peer_id = node_peer_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = pool.send_message(&peer, &response, &kem_pk, &sig_pk, &peer_id).await {
                    eprintln!("[sync] failed to send SyncResponse to requester: {}", e);
                }
            });
        } else {
            eprintln!("[sync] requester peer not found in peer_book");
        }
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