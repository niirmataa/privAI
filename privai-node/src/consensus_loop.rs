//! Główna pętla konsensusu PC-BFT.
//!
//! Łączy:
//! - Tor listener (odbiera ConsensusMsg)
//! - PrivaiNode (logika konsensusu)
//! - Tor broadcast (wysyła odpowiedzi)
//! - Timeout checker (ViewChange)

use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

use nxms_transport::peers::PeerBook;
use privai_chain::{ConsensusMsg, VoteType};

use crate::net::{self, NetConfig, NetError};
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
}

impl<S: LedgerStore, V: ProofVerifier, A: ProofArtifactStore, P: BlockArtifactVerifier>
    ConsensusLoop<S, V, A, P>
{
    /// Uruchamia główną pętlę konsensusu.
    ///
    /// 1. Startuje Tor listener
    /// 2. Uruchamia timeout checker (co 1s)
    /// 3. Dispatchuje przychodzące ConsensusMsg
    pub async fn run(&mut self) -> Result<(), ConsensusLoopError> {
        // Channel na incoming messages
        let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<(String, ConsensusMsg)>();

        // Start Tor listener w tle
        let net_config = self.net_config.clone();
        let listener_handle = tokio::spawn(async move {
            if let Err(e) = net::run_listener(net_config, msg_tx).await {
                eprintln!("[consensus] listener error: {}", e);
            }
        });

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
                    self.handle_message(peer_hint, msg).await;
                }

                // Timeout check
                _ = timeout_ticker.tick() => {
                    self.check_and_handle_timeout().await;
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
    async fn handle_message(&mut self, peer_hint: String, msg: ConsensusMsg) {
        eprintln!(
            "[consensus] received {} from {} (height={}, round={})",
            msg.msg_type(),
            peer_hint,
            msg.height(),
            msg.round()
        );

        match msg {
            ConsensusMsg::Proposal { block, proposer_sig: _ } => {
                // TODO: walidacja bloku, import, generacja Prevote
                eprintln!(
                    "[consensus] received proposal height={} round={} hash={:?}",
                    block.header.height,
                    block.header.round,
                    &block.hash()[..8]
                );
            }

            ConsensusMsg::Prevote(vote) => {
                if let Some(qc) = self.node.receive_vote(vote.clone()) {
                    eprintln!(
                        "[consensus] PREVOTE QC built for block {:?} at height={}",
                        &qc.block_hash[..8],
                        qc.height
                    );
                    // Broadcast Precommit po osiągnięciu prevote threshold
                    let precommit = privai_chain::Vote {
                        height: vote.height,
                        round: vote.round,
                        block_hash: vote.block_hash,
                        vote_type: VoteType::Precommit,
                        validator_pk: self.node.config().node_pk_hash.to_vec(),
                        falcon_sig: vec![],
                    };
                    self.broadcast_msg(ConsensusMsg::Precommit(precommit)).await;
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
                    self.broadcast_msg(ConsensusMsg::QuorumCert(qc)).await;
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
                // TODO: finalize block z tym QC
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
        }
    }

    /// Sprawdza timeout i wysyła ViewChange jeśli potrzeba.
    async fn check_and_handle_timeout(&mut self) {
        let now = current_time_ms();
        if let Some(vc) = self.node.check_timeout_default(now) {
            eprintln!(
                "[consensus] TIMEOUT! Sending ViewChange for round {}",
                vc.new_round
            );
            self.broadcast_msg(ConsensusMsg::ViewChange(vc)).await;
        }
    }

    /// Broadcastuje wiadomość do wszystkich peerów.
    async fn broadcast_msg(&self, msg: ConsensusMsg) {
        let results = net::broadcast(
            &self.peer_book,
            &self.net_config.my_peer_id,
            &self.net_config.tor_socks_url,
            &msg,
        )
        .await;

        for (peer_id, result) in results {
            if let Err(e) = result {
                eprintln!("[consensus] broadcast to {} failed: {}", peer_id, e);
            }
        }
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