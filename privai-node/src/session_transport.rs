//! Facade for the validator session transport layer.
//!
//! This is the `privai-node` side of the architectural split described in
//! `spec/PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`.
//! `ValidatorSessionTransport` is the Layer C boundary used by higher-level
//! modules (`consensus_loop`, `gossip`, `state_sync`) to move `ConsensusMsg`
//! values over validator sessions without carrying raw session key material
//! through the rest of the node.
//!
//! It wraps the concrete implementation in `session_impl.rs`
//! (connection pool, PQC handshake, encrypted frames) and exposes a simpler
//! API that takes `&NodeConfig` instead of raw cryptographic keys.
//!
//! Responsibilities:
//! - spawn and manage the listener for incoming validator-session traffic
//! - spawn connection-pool maintenance (health checks, reconnection)
//! - send messages to individual peers (`send_message`)
//! - broadcast messages to all known peers (`broadcast_message`)
//!
//! Non-responsibilities:
//! - this is not the escrow / mailbox packet protocol
//! - this is not `NXMS/1`, `NXMS/2`, `NxmsEnvelope*`, or `SealedPacket`
//! - this is not the consensus overlay itself; gossip/sync/vote semantics
//!   stay in higher-level modules

use serde::Serialize;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use nxms_transport::peers::{Peer, PeerBook};
use privai_chain::ConsensusMsg;

use crate::config::NodeConfig;
use crate::net::{self, BanList, ConnectionPool, NetConfig, NetError, RateLimiter};

#[derive(Clone)]
pub struct ValidatorSessionTransport {
    net_config: NetConfig,
    connection_pool: ConnectionPool,
    ban_list: BanList,
    rate_limiter: RateLimiter,
}

impl ValidatorSessionTransport {
    pub fn new(net_config: NetConfig) -> Self {
        let connection_pool = ConnectionPool::new(net_config.tor_socks_url.clone());
        Self {
            net_config,
            connection_pool,
            ban_list: BanList::new(),
            rate_limiter: RateLimiter::new(),
        }
    }

    pub fn my_peer_id(&self) -> &str {
        &self.net_config.my_peer_id
    }

    pub fn spawn_listener(
        &self,
        msg_tx: mpsc::Sender<(String, ConsensusMsg)>,
        node_config: &NodeConfig,
        peer_book: PeerBook,
    ) -> JoinHandle<()> {
        let net_config = self.net_config.clone();
        let kem_pk = node_config.node_kem_pk.clone();
        let kem_sk = node_config.node_kem_sk.clone();
        let sig_pk = node_config.node_sig_pk.clone();
        let sig_sk = node_config.node_sig_sk.clone();
        let peer_id = self.net_config.my_peer_id.clone();
        let ban_list = self.ban_list.clone();
        let rate_limiter = self.rate_limiter.clone();

        tokio::spawn(async move {
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
                eprintln!("[session] listener error: {}", e);
            }
        })
    }

    pub fn spawn_maintenance(&self, peer_book: PeerBook, node_config: &NodeConfig) {
        self.connection_pool.spawn_maintenance(
            peer_book,
            self.net_config.my_peer_id.clone(),
            node_config.node_kem_pk.clone(),
            node_config.node_kem_sk.clone(),
            node_config.node_sig_pk.clone(),
            node_config.node_sig_sk.clone(),
        );
    }

    pub async fn send_message<T: Serialize>(
        &self,
        peer: &Peer,
        msg: &T,
        node_config: &NodeConfig,
    ) -> Result<(), NetError> {
        self.connection_pool
            .send_message(
                peer,
                msg,
                &node_config.node_kem_pk,
                &node_config.node_kem_sk,
                &node_config.node_sig_pk,
                &node_config.node_sig_sk,
                &self.net_config.my_peer_id,
            )
            .await
    }

    pub async fn broadcast_message<T: Serialize + Clone + Send + Sync + 'static>(
        &self,
        peer_book: &PeerBook,
        msg: &T,
        node_config: &NodeConfig,
    ) -> Vec<(String, Result<(), NetError>)> {
        self.connection_pool
            .broadcast_message(
                peer_book,
                &self.net_config.my_peer_id,
                msg,
                &node_config.node_kem_pk,
                &node_config.node_kem_sk,
                &node_config.node_sig_pk,
                &node_config.node_sig_sk,
            )
            .await
    }
}
