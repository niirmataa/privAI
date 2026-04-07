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
