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
use privai_chain::{Block, ConsensusMsg, Hash32, VoteType, consensus::PeerInfo};

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
        let kem_sk = self.node.config().node_kem_sk.clone();
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
                kem_sk,
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
            self.node.config().node_kem_sk.clone(),
            self.node.config().node_sig_pk.clone(),
            self.node.config().node_sig_sk.clone(),
        );

        // Timeout checker — co 1 sekundę sprawdzamy czy nie przekroczyliśmy limitu
        let mut timeout_ticker = interval(Duration::from_secs(1));

        eprintln!(
            "[consensus] loop started, timeout={}ms",
            self.node.config().consensus_timeout_ms
        );

        // Na starcie — sprawdź czy jesteśmy proposerem rundy 0
        self.try_propose();

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
                // Lookup pełnego PK z registry (proposer_pk_hash → sig_pk)
                let proposer_pk = match self.node.config().validators.iter()
                    .find(|v| v.pk_hash == block.header.proposer_pk_hash)
                    .map(|v| &v.sig_pk)
                {
                    Some(pk) => pk,
                    None => {
                        eprintln!("[consensus] REJECTED proposal: unknown proposer pk_hash");
                        return;
                    }
                };
                if nxms_transport::crypto::falcon_verify(
                    proposer_pk,
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
                    state_root: block.header.state_root,
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

                // Weryfikacja QC — sprawdź signers, stake, podpisy
                if let Err(e) = self.verify_qc(&qc) {
                    eprintln!("[consensus] REJECTED QC: {}", e);
                    return;
                }

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
                    // Po awansie rundy — sprawdź czy jesteśmy proposerem
                    self.try_propose();
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

            ConsensusMsg::Gossip { tx, sender_pk_hash, hops } => {
                eprintln!(
                    "[consensus] received gossip tx from {:?} hops={}",
                    &sender_pk_hash[..8],
                    hops
                );

                // Weryfikacja podpisu Falcon PRZED dodaniem do mempoola (Zero Trust)
                if !self.node.verify_tx_signatures(&tx) {
                    eprintln!("[consensus] REJECTED gossip tx: invalid Falcon signature");
                    return;
                }

                // Dodaj do mempoola
                if let Err(e) = self.node.submit_transaction(tx.clone(), current_time_ms()) {
                    eprintln!("[consensus] rejected gossip tx: {}", e);
                    return;
                }

                // Propaguj dalej jeśli hops < MAX_GOSSIP_HOPS
                const MAX_GOSSIP_HOPS: u8 = 5;
                if hops < MAX_GOSSIP_HOPS {
                    self.broadcast_msg(ConsensusMsg::Gossip {
                        tx,
                        sender_pk_hash,
                        hops: hops + 1,
                    });
                }
            }

            ConsensusMsg::GetPeers { requester_pk_hash } => {
                eprintln!(
                    "[consensus] GetPeers request from {:?}",
                    &requester_pk_hash[..8]
                );

                // Zbierz informacje o znanych peerach z PeerBook
                let peers: Vec<PeerInfo> = self.peer_book.peers.iter()
                    .filter(|peer| peer.id != self.net_config.my_peer_id)
                    .filter_map(|peer| {
                        use base64::Engine;
                        use base64::engine::general_purpose::STANDARD as B64;
                        if let Ok(falcon_pk) = B64.decode(&peer.sig_pk_b64) {
                            Some(PeerInfo {
                                address: format!("{}:{}", peer.host, peer.port),
                                falcon_pk,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                // Podpisz listę peerów kluczem Falcon (ochrona przed Eclipse attack)
                let peers_bytes = serde_json::to_vec(&peers).unwrap_or_default();
                let falcon_sig = if let Some(sk) = self.node.falcon_sk() {
                    nxms_transport::crypto::falcon_sign_ct_prepared(sk, &peers_bytes).unwrap_or_default()
                } else {
                    Vec::new()
                };

                let response = ConsensusMsg::PeersList {
                    peers,
                    sender_pk_hash: self.node.config().node_pk_hash,
                    falcon_sig,
                };

                // Wyślij odpowiedź bezpośrednio do requestera
                self.broadcast_msg(response);
            }

            ConsensusMsg::PeersList { peers, sender_pk_hash, falcon_sig } => {
                eprintln!(
                    "[consensus] received PeersList from {:?} with {} peers",
                    &sender_pk_hash[..8],
                    peers.len()
                );

                // Weryfikacja podpisu Falcon na liście peerów
                let peers_bytes = serde_json::to_vec(&peers).unwrap_or_default();
                if falcon_sig.is_empty() {
                    eprintln!("[consensus] REJECTED PeersList: missing Falcon signature");
                    return;
                }
                // Lookup pełnego PK z registry (sender_pk_hash → sig_pk)
                let sender_pk = match self.node.config().validators.iter()
                    .find(|v| v.pk_hash == sender_pk_hash)
                    .map(|v| &v.sig_pk)
                {
                    Some(pk) => pk,
                    None => {
                        eprintln!("[consensus] REJECTED PeersList: unknown sender pk_hash");
                        return;
                    }
                };
                if nxms_transport::crypto::falcon_verify(sender_pk, &peers_bytes, &falcon_sig).is_err() {
                    eprintln!("[consensus] REJECTED PeersList: invalid Falcon signature");
                    return;
                }

                // TODO: Dodaj nowych peerów do PeerBook (z weryfikacją)
                eprintln!(
                    "[consensus] PeersList verified — {} peers available",
                    peers.len()
                );
                for peer in &peers {
                    eprintln!("  - peer: {}", peer.address);
                }
            }
        }
    }

    /// Sprawdza czy jesteśmy proposerem dla bieżącej rundy i inicjuje propozycję.
    /// Wywoływane po ViewChange i na starcie rundy.
    fn try_propose(&mut self) {
        let round = self.node.current_round;
        let snapshot = self.node.ledger().snapshot();
        let height = snapshot.height + 1;
        let epoch = snapshot.consensus_safety.current_view as u64; // epoch z current_view
        let epoch_seed_hash = privai_chain::hash::domain_hash(
            "privai:epoch-seed:v0",
            &[&epoch.to_le_bytes(), &self.node.config().chain_id.to_le_bytes()],
        );
        let parent_qc_hash = if height > 1 {
            // Hash ostatniego bloku jako parent
            snapshot.tip_hash
        } else {
            [0u8; 32] // genesis
        };
        let timestamp_ms = current_time_ms();

        // Sprawdź czy jesteśmy proposerem
        let expected_proposer = match self.node.next_proposer(&epoch_seed_hash, round) {
            Some(p) => p,
            None => return, // brak walidatorów
        };

        if expected_proposer != self.node.config().node_pk_hash {
            // Nie jesteśmy proposerem — czekamy
            return;
        }

        eprintln!(
            "[consensus] I am PROPOSER for height={} round={} — building block",
            height, round
        );

        // Zbuduj blok
        let block = match self.node.propose_block(
            epoch,
            round,
            timestamp_ms,
            epoch_seed_hash,
            parent_qc_hash,
            Vec::new(),  // proof_certificates — zbierane później
            Vec::new(),  // extra_receipts
        ) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[consensus] propose_block failed: {}", e);
                return;
            }
        };

        // Podpisz blok kluczem Falcon
        let mut proposer_sig = vec![];
        if let Some(sk) = self.node.falcon_sk() {
            if let Ok(sig) = nxms_transport::crypto::falcon_sign_ct_prepared(sk, &block.hash()) {
                proposer_sig = sig;
            }
        }

        if proposer_sig.is_empty() {
            eprintln!("[consensus] WARNING: no Falcon key — sending unsigned proposal");
        }

        // Cache bloku
        let block_hash = block.hash();
        self.block_cache.insert(block_hash, block.clone());

        eprintln!(
            "[consensus] broadcasting PROPOSAL height={} round={} hash={:?}",
            height,
            round,
            &block_hash[..8]
        );

        self.broadcast_msg(ConsensusMsg::Proposal { block, proposer_sig });
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
            // Po ViewChange — sprawdź czy jesteśmy proposerem nowej rundy
            self.try_propose();
        }
    }

    /// Weryfikuje QuorumCertificate: sprawdza signers, stake, podpisy Falcon.
    /// Używa rayon do równoległej weryfikacji podpisów (Falcon jest CPU-bound).
    fn verify_qc(&self, qc: &privai_chain::QuorumCertificate) -> Result<(), String> {
        use rayon::prelude::*;

        let validators = &self.node.config().validators;
        let total_stake: u64 = validators.iter().map(|v| v.stake_weight).sum();
        let required_stake = (total_stake * 2) / 3 + 1;

        // 1. Sprawdź liczbę signers vs signatures
        if qc.signers.len() != qc.signatures.len() {
            return Err(format!(
                "signers/signatures count mismatch: {} vs {}",
                qc.signers.len(), qc.signatures.len()
            ));
        }

        // 2. Równoległa weryfikacja każdego signera + podpisu
        let results: Vec<Result<u64, String>> = qc.signers.par_iter()
            .zip(qc.signatures.par_iter())
            .map(|(signer_pk, sig)| {
                // 2a. Sprawdź czy signer jest znanym validatorem
                let voter_pk_hash = privai_chain::hash::domain_hash(
                    "privai:falcon-pk:v0",
                    &[signer_pk],
                );
                let voter_stake = validators
                    .iter()
                    .find(|v| v.pk_hash == voter_pk_hash)
                    .map(|v| v.stake_weight)
                    .ok_or_else(|| format!("unknown validator {:?}", &voter_pk_hash[..8]))?;

                // 2b. Weryfikuj podpis Falcon
                if sig.is_empty() {
                    return Err(format!("empty signature for signer {:?}", &voter_pk_hash[..8]));
                }
                if nxms_transport::crypto::falcon_verify(signer_pk, &qc.block_hash, sig).is_err() {
                    return Err(format!("invalid Falcon signature for signer {:?}", &voter_pk_hash[..8]));
                }

                Ok(voter_stake)
            })
            .collect();

        // 3. Sprawdź wyniki i zsumuj stake
        let mut accumulated_stake: u64 = 0;
        for result in results {
            accumulated_stake += result?;
        }

        // 4. Sprawdź próg stake
        if accumulated_stake < required_stake {
            return Err(format!(
                "insufficient stake: {} < {} (required)",
                accumulated_stake, required_stake
            ));
        }

        Ok(())
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
        let kem_sk = self.node.config().node_kem_sk.clone();
        let sig_pk = self.node.config().node_sig_pk.clone();
        let sig_sk = self.node.config().node_sig_sk.clone();

        tokio::spawn(async move {
            let results = pool.broadcast_message(&peer_book, &my_id, &msg, &kem_pk, &kem_sk, &sig_pk, &sig_sk).await;
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