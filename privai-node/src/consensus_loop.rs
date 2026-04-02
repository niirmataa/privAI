//! Główna pętla konsensusu PC-BFT.
//!
//! Łączy:
//! - Tor listener (odbiera ConsensusMsg)
//! - PrivaiNode (logika konsensusu)
//! - Tor broadcast (wysyła odpowiedzi)
//! - Timeout checker (ViewChange)

use std::collections::HashMap;

use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

use nxms_transport::peers::PeerBook;
use privai_chain::{Block, ConsensusMsg, Hash32, VoteType};

use crate::net::{self, BanList, NetConfig, NetError, RateLimiter};
use crate::node::{NodeError, PrivaiNode};
use privai_ledger::LedgerStore;
use privai_proof::{BlockArtifactVerifier, ProofVerifier};
use privai_proof::store::ProofArtifactStore;

/// Stan pętli konsensusu.
pub struct ConsensusLoop<S: LedgerStore, V, A, P>
where
    V: ProofVerifier,
    A: ProofArtifactStore,
    P: BlockArtifactVerifier,
{
    pub node: PrivaiNode<S, V, A, P>,
    pub net_config: NetConfig,
    pub peer_book: PeerBook,
    pub connection_pool: net::ConnectionPool,
    /// Ban list — blokuje złośliwe peerów.
    pub ban_list: BanList,
    /// Rate limiter — zapobiega floodowi incoming connections.
    pub rate_limiter: RateLimiter,
    /// Cache bloków po hashu — potrzebny do finalizacji po otrzymaniu QC.
    block_cache: HashMap<Hash32, Block>,
    /// Cache QC po block_hash — potrzebny do state sync.
    qc_cache: HashMap<Hash32, privai_chain::QuorumCertificate>,
}

impl<S: LedgerStore, V: ProofVerifier, A: ProofArtifactStore, P: BlockArtifactVerifier>
    ConsensusLoop<S, V, A, P>
{
    pub fn new(
        node: PrivaiNode<S, V, A, P>,
        net_config: NetConfig,
        peer_book: PeerBook,
    ) -> Self {
        let connection_pool = net::ConnectionPool::new(net_config.tor_socks_url.clone());
        Self {
            node,
            net_config,
            peer_book,
            connection_pool,
            ban_list: BanList::new(),
            rate_limiter: RateLimiter::new(),
            block_cache: HashMap::new(),
            qc_cache: HashMap::new(),
        }
    }

    /// Uruchamia główną pętlę konsensusu.
    ///
    /// 1. Startuje Tor listener
    /// 2. Uruchamia timeout checker (co 1s)
    /// 3. Dispatchuje przychodzące ConsensusMsg
    pub async fn run(&mut self) -> Result<(), ConsensusLoopError> {
        // Channel na incoming messages — BOUNDED (256) zapobiega OOM przy floodzie
        let (msg_tx, mut msg_rx) = mpsc::channel::<(String, ConsensusMsg)>(256);

        // Start Tor listener w tle (zabezpieczony: rate limiter + ban list + weryfikacja peerów)
        let net_config = self.net_config.clone();
        let kem_pk = self.node.config().node_kem_pk.clone();
        let sig_pk = self.node.config().node_sig_pk.clone();
        let sig_sk = self.node.config().node_sig_sk.clone();
        let peer_id = self.net_config.my_peer_id.clone();
        let peer_book = self.peer_book.clone();
        let ban_list = self.ban_list.clone();
        let rate_limiter = self.rate_limiter.clone();
        let listener_handle = tokio::spawn(async move {
            if let Err(e) = net::run_listener(
                net_config,
                msg_tx,
                kem_pk,
                sig_pk,
                sig_sk,
                peer_id,
                peer_book,
                ban_list,
                rate_limiter,
            )
            .await
            {
                eprintln!("[consensus] listener error: {}", e);
            }
        });

        // Uruchamia pool maintenance — sprawdza health connections co 30s
        self.connection_pool.spawn_maintenance(
            self.peer_book.clone(),
            self.net_config.my_peer_id.clone(),
            self.node.config().node_kem_pk.clone(),
            self.node.config().node_sig_pk.clone(),
            self.node.config().node_sig_sk.clone(),
        );

        // Timeout checker — co 1 sekundę sprawdzamy czy nie przekroczyliśmy limitu
        let mut timeout_ticker = interval(Duration::from_secs(1));

        eprintln!(
            "[consensus] loop started, timeout={}ms",
            self.node.config().consensus_timeout_ms
        );

        loop {
            tokio::select! {
                // Incoming message z Tor listenera
                Some((peer_hint, msg)) = msg_rx.recv() => {
                    self.handle_message(peer_hint, msg);
                }

                // Timeout check
                _ = timeout_ticker.tick() => {
                    self.check_and_handle_timeout();
                }

                else => {
                    eprintln!("[consensus] channel closed, shutting down");
                    break;
                }
            }
        }

        listener_handle.abort();
        Ok(())
    }

    /// Przetwarza przychodzącą wiadomość konsensusową.
    fn handle_message(&mut self, peer_hint: String, msg: ConsensusMsg) {
        eprintln!(
            "[consensus] received {} from {} (height={}, round={})",
            msg.msg_type(),
            peer_hint,
            msg.height(),
            msg.round()
        );

        match msg {
            ConsensusMsg::Proposal { block, proposer_sig } => {
                eprintln!(
                    "[consensus] received proposal height={} round={} hash={:?}",
                    block.header.height,
                    block.header.round,
                    &block.hash()[..8]
                );

                // 1. Weryfikacja proposera — czy jest uprawniony dla (epoch_seed, round)
                let expected_proposer = self.node.next_proposer(
                    &block.header.epoch_seed_hash,
                    block.header.round,
                );
                if expected_proposer != Some(block.header.proposer_pk_hash) {
                    eprintln!(
                        "[consensus] REJECTED proposal: wrong proposer {:?} (expected {:?})",
                        &block.header.proposer_pk_hash[..8],
                        expected_proposer.map(|p| format!("{:?}", &p[..8]))
                    );
                    return;
                }

                // 2. Weryfikacja podpisu Proposera (obowiązkowy)
                if proposer_sig.is_empty() {
                    eprintln!("[consensus] REJECTED proposal: missing proposer signature (mandatory)");
                    return;
                }
                if nxms_transport::crypto::falcon_verify(
                    &block.header.proposer_pk_hash,
                    &block.hash(),
                    &proposer_sig,
                )
                .is_err()
                {
                    eprintln!("[consensus] REJECTED proposal: invalid proposer signature");
                    return;
                }

                // 3. Walidacja timestamp (nie w przyszłości, nie za stary)
                let now = current_time_ms();
                if block.header.timestamp_ms > now + 30_000 {
                    eprintln!(
                        "[consensus] REJECTED proposal: timestamp too far in future ({} > {})",
                        block.header.timestamp_ms,
                        now + 30_000
                    );
                    return;
                }
                if block.header.timestamp_ms < now.saturating_sub(self.node.config().consensus_timeout_ms * 2) {
                    eprintln!(
                        "[consensus] REJECTED proposal: timestamp too old ({} < {})",
                        block.header.timestamp_ms,
                        now.saturating_sub(self.node.config().consensus_timeout_ms * 2)
                    );
                    return;
                }

                // 4. Walidacja height (sekwencyjna)
                let current_height = self.node.ledger().snapshot().height;
                if block.header.height != current_height + 1 {
                    eprintln!(
                        "[consensus] REJECTED proposal: wrong height {} (expected {})",
                        block.header.height,
                        current_height + 1
                    );
                    return;
                }

                // 5. Walidacja prev_block_hash (musi pasować do current tip)
                if block.header.prev_block_hash != self.node.ledger().snapshot().tip_hash {
                    eprintln!(
                        "[consensus] REJECTED proposal: prev_block_hash mismatch"
                    );
                    return;
                }

                // 6. Walidacja: roots muszą się zgadzać
                if !block.roots_match() {
                    eprintln!("[consensus] REJECTED proposal: roots mismatch");
                    return;
                }

                // Cache bloku — potrzebny później do finalizacji po QC
                let block_hash = block.hash();
                self.block_cache.insert(block_hash, block.clone());

                // Import bloku do ledgera
                if let Err(e) = self.node.import_block(&block) {
                    eprintln!("[consensus] REJECTED proposal: import failed: {}", e);
                    return;
                }

                // Generacja i broadcast Prevote
                let template = privai_chain::BlockTemplate {
                    chain_id: block.header.chain_id,
                    height: block.header.height,
                    epoch: block.header.epoch,
                    round: block.header.round,
                    timestamp_ms: block.header.timestamp_ms,
                    prev_block_hash: block.header.prev_block_hash,
                    proposer_pk_hash: block.header.proposer_pk_hash,
                    epoch_seed_hash: block.header.epoch_seed_hash,
                    parent_qc_hash: block.header.parent_qc_hash,
                    txs: block.body.txs.clone(),
                    execution_bundle: block.body.execution_bundle.clone(),
                    proof_certificates: block.body.proof_certificates.clone(),
                    extra_receipts: block.body.extra_receipts.clone(),
                };

                match self.node.create_vote_for_proposal(&template) {
                    Ok(vote) => {
                        eprintln!(
                            "[consensus] sending PREVOTE for block {:?}",
                            &vote.block_hash[..8]
                        );
                        self.broadcast_msg(ConsensusMsg::Prevote(vote));
                    }
                    Err(e) => {
                        eprintln!("[consensus] failed to create prevote: {}", e);
                    }
                }
            }

            ConsensusMsg::Prevote(vote) => {
                if let Some(qc) = self.node.receive_vote(vote.clone()) {
                    eprintln!(
                        "[consensus] PREVOTE QC built for block {:?} at height={}",
                        &qc.block_hash[..8],
                        qc.height
                    );
                    // Broadcast Precommit po osiągnięciu prevote threshold
                    // Używamy Falcon PK z configu, podpisujemy block_hash
                    let validator_pk = self.node.config().node_sig_pk.clone();
                    let mut falcon_sig = vec![];
                    if let Some(sk) = self.node.falcon_sk() {
                        if let Ok(sig) = nxms_transport::crypto::falcon_sign_ct_prepared(sk, &vote.block_hash) {
                            falcon_sig = sig;
                        }
                    }
                    let precommit = privai_chain::Vote {
                        height: vote.height,
                        round: vote.round,
                        block_hash: vote.block_hash,
                        vote_type: VoteType::Precommit,
                        validator_pk,
                        falcon_sig,
                    };
                    self.broadcast_msg(ConsensusMsg::Precommit(precommit));
                }
            }

            ConsensusMsg::Precommit(vote) => {
                if let Some(qc) = self.node.receive_vote(vote) {
                    eprintln!(
                        "[consensus] PRECOMMIT QC built for block {:?} at height={}",
                        &qc.block_hash[..8],
                        qc.height
                    );
                    // Broadcast QC
                    self.broadcast_msg(ConsensusMsg::QuorumCert(qc));
                }
            }

            ConsensusMsg::QuorumCert(qc) => {
                eprintln!(
                    "[consensus] received QC for block {:?} height={} round={} type={:?}",
                    &qc.block_hash[..8],
                    qc.height,
                    qc.round,
                    qc.vote_type
                );

                // Zapisz QC do cache (potrzebny do state sync)
                self.qc_cache.insert(qc.block_hash, qc.clone());

                // Finalizacja tylko dla Precommit QC
                if qc.vote_type == VoteType::Precommit {
                    if let Some(block) = self.block_cache.remove(&qc.block_hash) {
                        match self.node.finalize_block_with_qc(&block, &qc) {
                            Ok(()) => {
                                eprintln!(
                                    "[consensus] FINALIZED block {:?} at height={} round={}",
                                    &qc.block_hash[..8],
                                    qc.height,
                                    qc.round
                                );
                            }
                            Err(e) => {
                                eprintln!("[consensus] finalization failed: {}", e);
                            }
                        }
                    } else {
                        eprintln!(
                            "[consensus] QC for unknown block {:?} — not in cache",
                            &qc.block_hash[..8]
                        );
                    }
                }
            }

            ConsensusMsg::ViewChange(vc) => {
                let advanced = self.node.receive_view_change(vc.clone(), current_time_ms());
                if advanced {
                    eprintln!(
                        "[consensus] VIEW CHANGE to round {} (quorum reached)",
                        self.node.current_round
                    );
                }
            }

            ConsensusMsg::Ping { height, round, sender_pk_hash } => {
                eprintln!(
                    "[consensus] ping from {:?} at height={} round={}",
                    &sender_pk_hash[..8],
                    height,
                    round
                );
            }

            ConsensusMsg::SyncRequest { from_height, to_height, requester_pk_hash } => {
                eprintln!(
                    "[consensus] SyncRequest from {:?} for {}..{}",
                    &requester_pk_hash[..8],
                    from_height,
                    to_height
                );
                crate::state_sync::handle_sync_request(
                    &self.node,
                    &self.block_cache,
                    from_height,
                    to_height,
                    requester_pk_hash,
                    &self.net_config,
                    &self.peer_book,
                    &self.connection_pool,
                    &self.node.config().node_kem_pk,
                    &self.node.config().node_sig_pk,
                    &self.node.config().node_sig_sk,
                    &self.net_config.my_peer_id,
                );
            }

            ConsensusMsg::SyncResponse { blocks, qcs, sender_pk_hash } => {
                crate::state_sync::handle_sync_response(
                    &mut self.node,
                    blocks,
                    qcs,
                    sender_pk_hash,
                );
            }
        }
    }

    /// Sprawdza timeout i wysyła ViewChange jeśli potrzeba.
    fn check_and_handle_timeout(&mut self) {
        let now = current_time_ms();
        if let Some(vc) = self.node.check_timeout_default(now) {
            eprintln!(
                "[consensus] TIMEOUT! Sending ViewChange for round {}",
                vc.new_round
            );
            self.broadcast_msg(ConsensusMsg::ViewChange(vc));
        }
    }

    /// Broadcastuje wiadomość do wszystkich peerów — FIRE AND FORGET.
    /// Nie blokuje pętli konsensusu — spawnowanie w tle.
    ///
    /// Jeśli broadcast do >1/3 peerów zawiódł, loguje ostrzeżenie o możliwej
    /// izolacji sieciowej (Tor circuit failure). Węzeł powinien rozważyć
    /// przejście w tryb ViewChange jeśli sytuacja się powtórzy.
    fn broadcast_msg(&self, msg: ConsensusMsg) {
        let peer_book = self.peer_book.clone();
        let my_id = self.net_config.my_peer_id.clone();
        let pool = self.connection_pool.clone();
        let kem_pk = self.node.config().node_kem_pk.clone();
        let sig_pk = self.node.config().node_sig_pk.clone();
        let sig_sk = self.node.config().node_sig_sk.clone();

        tokio::spawn(async move {
            let results = pool.broadcast_message(&peer_book, &my_id, &msg, &kem_pk, &sig_pk, &sig_sk).await;
            let total = results.len();
            let mut failures = 0usize;

            for (peer_id, result) in &results {
                if let Err(e) = result {
                    failures += 1;
                    eprintln!("[consensus] broadcast to {} failed: {}", peer_id, e);
                }
            }

            // Jeśli >1/3 peerów niedostępnych — możliwe odcięcie od sieci (Tor circuit failure)
            if total > 0 && failures > total / 3 {
                eprintln!(
                    "[consensus] WARNING: {}/{} broadcasts failed — possible network isolation (Tor circuit failure). \
                    Node may need ViewChange if this persists.",
                    failures, total
                );
            }
        });
    }
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug, thiserror::Error)]
pub enum ConsensusLoopError {
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("net error: {0}")]
    Net(#[from] NetError),
    #[error("peer book error: {0}")]
    PeerBook(#[from] anyhow::Error),
}