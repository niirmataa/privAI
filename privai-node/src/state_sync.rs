//! State sync semantics — mechanizm catch-up dla węzłów które były offline.
//!
//! Odpowiada za politykę synchronizacji stanu:
//! 1. Węzeł wykryje, że jest z tyłu za siecią
//! 2. Wysyła SyncRequest do peerów (przez ValidatorSessionTransport)
//! 3. Odbiera SyncResponse z blokami + QC
//! 4. Importuje bloki sekwencyjnie i finalizuje z QC

use nxms_transport::peers::PeerBook;
use privai_chain::{Block, ConsensusMsg, Hash32, QuorumCertificate, VoteType};

use crate::config::NodeConfig;
use crate::node::{NodeError, PrivaiNode};
use crate::session_transport::ValidatorSessionTransport;
use crate::NetError;
use privai_ledger::LedgerStore;
use privai_proof::store::ProofArtifactStore;
use privai_proof::{BlockArtifactVerifier, ProofVerifier};

/// Maksymalna liczba bloków w jednej odpowiedzi sync.
/// Requester wysyła kolejny SyncRequest jeśli potrzebuje więcej.
pub const MAX_SYNC_BATCH: u64 = 10;

/// Wysyła SyncRequest do losowego peera.
/// Zwraca bloki i QCs do zaimportowania.
pub async fn request_blocks(
    peer_book: &PeerBook,
    my_id: &str,
    session_transport: &ValidatorSessionTransport,
    from_height: u64,
    to_height: u64,
    my_pk_hash: Hash32,
    node_config: &NodeConfig,
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
    session_transport
        .send_message(peer, &msg, node_config)
        .await
        .map_err(SyncError::Net)?;

    // W v0 nie czekamy na response (fire-and-forget).
    // Response przyjdzie jako nowa wiadomość w consensus loop.
    // W v1 można dodać request-response correlation.
    Ok(None)
}

/// Obsługuje przychodzący SyncRequest — wysyła bloki do requestera.
pub fn handle_sync_request<
    S: LedgerStore,
    V: ProofVerifier,
    A: ProofArtifactStore,
    P: BlockArtifactVerifier,
>(
    node: &PrivaiNode<S, V, A, P>,
    block_cache: &std::collections::HashMap<Hash32, Block>,
    from_height: u64,
    to_height: u64,
    requester_pk_hash: Hash32,
    peer_book: &PeerBook,
    session_transport: &ValidatorSessionTransport,
    node_config: &NodeConfig,
) {
    let current_height = node.ledger().snapshot().height;
    let from = from_height.min(current_height);
    let to = to_height.min(current_height).min(from + MAX_SYNC_BATCH);

    if from > to {
        eprintln!("[sync] invalid range: from={} to={}", from, to);
        return;
    }

    eprintln!(
        "[sync] SyncRequest from {:?} for heights {}..{} (current={})",
        &requester_pk_hash[..8],
        from,
        to,
        current_height
    );

    // Zbieramy bloki: najpierw z block_cache, potem fallback na ledger.snapshot().blocks
    let mut blocks: Vec<Block> = block_cache
        .values()
        .filter(|b| b.header.height >= from && b.header.height <= to)
        .cloned()
        .collect();

    // Fallback na persistent storage jeśli cache nie ma bloków
    if blocks.is_empty() {
        blocks = node
            .ledger()
            .snapshot()
            .blocks
            .range(from..=to)
            .map(|(_, b)| b.clone())
            .collect();
    }

    blocks.sort_by_key(|b| b.header.height);

    if blocks.is_empty() {
        eprintln!(
            "[sync] no blocks in cache or ledger for range {}..{}",
            from, to
        );
        return;
    }

    eprintln!(
        "[sync] serving {} blocks (heights {}..{})",
        blocks.len(),
        blocks.first().map(|b| b.header.height).unwrap_or(0),
        blocks.last().map(|b| b.header.height).unwrap_or(0),
    );

    // Zbieramy QCs z ledgera dla tych bloków
    let qcs: Vec<QuorumCertificate> = blocks
        .iter()
        .filter_map(|b| node.ledger().snapshot().qcs.get(&b.header.height).cloned())
        .collect();

    let response = ConsensusMsg::SyncResponse {
        blocks,
        qcs,
        sender_pk_hash: node.config().node_pk_hash,
    };

    // Znajdź requestera po pk_hash — porównaj z Falcon pk hash z PeerBook
    let target_peer = find_peer_by_pk_hash(peer_book, &requester_pk_hash);

    if let Some(peer) = target_peer {
        let session_transport = session_transport.clone();
        let node_config = node_config.clone();
        tokio::spawn(async move {
            if let Err(e) = session_transport
                .send_message(&peer, &response, &node_config)
                .await
            {
                eprintln!("[sync] failed to send SyncResponse to requester: {}", e);
            }
        });
    } else {
        eprintln!(
            "[sync] requester {:?} not found in peer_book — cannot send targeted response",
            &requester_pk_hash[..8]
        );
    }
}

/// Znajduje peera w PeerBook po pk_hash (hash Falcon public key).
/// falcon_pk_hash = BLAKE3("privai:falcon-pk:v0" || pk_bytes)
fn find_peer_by_pk_hash(
    peer_book: &PeerBook,
    pk_hash: &Hash32,
) -> Option<nxms_transport::peers::Peer> {
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine;
    use privai_chain::hash::domain_hash;

    const FALCON_PK_DOMAIN: &str = "privai:falcon-pk:v0";

    for peer in &peer_book.peers {
        if let Ok(pk_bytes) = B64.decode(&peer.sig_pk_b64) {
            let hash = domain_hash(FALCON_PK_DOMAIN, &[&pk_bytes]);
            if hash == *pk_hash {
                return Some(peer.clone());
            }
        }
    }
    None
}

/// Obsługuje przychodzący SyncResponse — importuje bloki i finalizuje.
pub fn handle_sync_response<
    S: LedgerStore,
    V: ProofVerifier,
    A: ProofArtifactStore,
    P: BlockArtifactVerifier,
>(
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

    let mut expected_prev_hash = node.ledger().snapshot().tip_hash;

    for block in sorted_blocks {
        // Weryfikacja chain continuity: prev_hash musi pasować do poprzedniego bloku
        if block.header.prev_block_hash != expected_prev_hash {
            eprintln!(
                "[sync] REJECTED block at height={}: prev_hash mismatch (expected {:?}, got {:?})",
                block.header.height,
                &expected_prev_hash[..8],
                &block.header.prev_block_hash[..8]
            );
            break; // prev_hash mismatch — kolejne bloki też będą złe
        }

        // Walidacja roots
        if !block.roots_match() {
            eprintln!(
                "[sync] REJECTED block at height={}: roots mismatch",
                block.header.height
            );
            break; // roots mismatch — zatrzymaj sync
        }

        // Import bloku
        expected_prev_hash = block.hash();
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
