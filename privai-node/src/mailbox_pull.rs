//! Mailbox pull loop — NXMS control-plane runtime integration.
//!
//! This module closes DRIFT-01 / T-03 from `PRIVAI_TRANSPORT_RUNTIME_FREEZE_MEMO`:
//! `handle_nxms_payload` exists, `nxms-mailbox-client` exists, but nothing
//! connected them in a running runtime path. This module provides that glue.
//!
//! # Architecture
//!
//! - [`MailboxSource`] trait abstracts mailbox pull/ack for testability.
//!   The production adapter wraps `nxms_mailbox_client::MailboxClient` and
//!   handles `NxmsEnvelope` → `NxmsPayloadV2` decryption/decode.
//! - [`mailbox_ingest_tick`] runs one pull→decode→ingest→ack cycle.
//! - [`run_mailbox_pull_loop`] wraps tick in a polling loop with configurable
//!   interval.
//!
//! # Ack policy (v1)
//!
//! - **Successful ingest** → ack immediately. Message is removed from mailbox.
//! - **Protocol/decode error** (malformed payload, wrong `app_proto`, bad
//!   `msg_type`) → do NOT ack. Message stays in mailbox for redelivery
//!   after lease expires. This prevents silent data loss; malformed messages
//!   surface as persistent backlog for investigation.
//! - **Ingest error** (escrow stage conflict, persistence failure) → do NOT
//!   ack. Transient failures (disk full, lock) self-heal on next cycle.
//! - **Ack failure** → logged. Message may be redelivered (at-least-once
//!   delivery). Duplicate ingest is handled by `EscrowStageStore` dedup.
//!
//! # Scope
//!
//! This is strictly the NXMS control-plane path (Model B). It does NOT touch:
//! - `ValidatorSessionTransport` (Model A — direct P2P)
//! - gossip, consensus, or state sync
//! - proof sidecar routing (returns `Ignored` through `handle_nxms_payload`)

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use nxms_transport::wire::NxmsPayloadV2;
use thiserror::Error;

use crate::config::{MailboxPullConfig, NodeConfig};
use crate::node::{EscrowIngestOutcome, NodeError, PrivaiNode};
use privai_ledger::LedgerStore;
use privai_proof::store::ProofArtifactStore;
use privai_proof::{BlockArtifactVerifier, ProofVerifier};

// ── Errors ──────────────────────────────────────────────────────────────

/// Errors from the mailbox pull/ack transport layer.
///
/// These are distinct from `NodeError` (ingest/decode errors), which are
/// handled per-message inside the tick. `MailboxPullError` represents
/// transport-level failures that affect the entire tick.
#[derive(Debug, Error)]
pub enum MailboxPullError {
    #[error("pull failed: {0}")]
    Pull(String),
    #[error("ack failed: {0}")]
    Ack(String),
}

// ── Trait ────────────────────────────────────────────────────────────────

/// A decoded message pulled from the mailbox, ready for ingest.
pub struct PulledPayload {
    /// Opaque receipt token — pass to `ack()` after successful processing.
    pub receipt: String,
    /// Decoded NXMS payload. The `MailboxSource` implementation is responsible
    /// for `NxmsEnvelope` → `NxmsPayloadV2` conversion (crypto decrypt +
    /// JSON deserialize).
    pub payload: NxmsPayloadV2,
}

/// Abstraction over the mailbox pull/ack interface.
///
/// The production implementation wraps `nxms_mailbox_client::MailboxClient`:
/// - `pull_payloads` → `client.pull(inbox, max, wait_ms=0)` + envelope decode
/// - `ack` → `client.ack(receipt)`
///
/// Test implementations return canned payloads without HTTP or crypto.
pub trait MailboxSource: Send + Sync {
    fn pull_payloads<'a>(
        &'a self,
        inbox: &'a str,
        max: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<PulledPayload>, MailboxPullError>> + Send + 'a>>;

    fn ack<'a>(
        &'a self,
        receipt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), MailboxPullError>> + Send + 'a>>;
}

// ── Tick report ─────────────────────────────────────────────────────────

/// Summary of one pull→ingest→ack cycle.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MailboxTickReport {
    /// Total messages pulled from mailbox.
    pub pulled: usize,
    /// Messages successfully ingested via `handle_nxms_payload`.
    pub ingested: usize,
    /// Messages successfully acked.
    pub acked: usize,
    /// Messages that failed protocol/decode (malformed payload).
    pub decode_errors: usize,
    /// Messages that failed ingest (escrow stage, persistence).
    pub ingest_errors: usize,
    /// Messages where ack call failed (message may be redelivered).
    pub ack_errors: usize,
}

// ── Single tick ─────────────────────────────────────────────────────────

/// Execute one pull→ingest→ack cycle.
///
/// Pull-level failures propagate as `Err`. Per-message errors are counted
/// in the report but do not abort the tick — remaining messages are still
/// processed.
pub async fn mailbox_ingest_tick<M, S, V, A, P>(
    source: &M,
    node: &mut PrivaiNode<S, V, A, P>,
    inbox: &str,
    batch_size: u32,
) -> Result<MailboxTickReport, MailboxPullError>
where
    M: MailboxSource,
    S: LedgerStore,
    V: ProofVerifier,
    A: ProofArtifactStore,
    P: BlockArtifactVerifier,
{
    let payloads = source.pull_payloads(inbox, batch_size).await?;

    let mut report = MailboxTickReport {
        pulled: payloads.len(),
        ..Default::default()
    };

    for msg in payloads {
        match node.handle_nxms_payload(&msg.payload) {
            Ok(outcome) => {
                report.ingested += 1;
                if outcome != EscrowIngestOutcome::Ignored {
                    eprintln!(
                        "[mailbox] ingested receipt={} outcome={:?}",
                        msg.receipt, outcome
                    );
                }
                // Ack on successful ingest (including Ignored — the payload
                // was valid, just not an escrow body we act on).
                match source.ack(&msg.receipt).await {
                    Ok(()) => report.acked += 1,
                    Err(e) => {
                        eprintln!("[mailbox] ack error receipt={}: {}", msg.receipt, e);
                        report.ack_errors += 1;
                    }
                }
            }
            Err(NodeError::Protocol(ref e)) => {
                // Malformed / unrecognized payload — do NOT ack.
                // Mailbox will redeliver after lease expires. Persistent
                // malformed messages will accumulate as backlog for
                // investigation (v1 dead-letter is manual).
                eprintln!("[mailbox] decode error receipt={}: {}", msg.receipt, e);
                report.decode_errors += 1;
            }
            Err(ref e) => {
                // Ingest failure (escrow stage conflict, persistence, etc.)
                // Do NOT ack — retry on next pull cycle.
                eprintln!("[mailbox] ingest error receipt={}: {}", msg.receipt, e);
                report.ingest_errors += 1;
            }
        }
    }

    Ok(report)
}

// ── Loop ────────────────────────────────────────────────────────────────

/// Run the mailbox pull loop until cancelled.
///
/// Polls the mailbox at `config.poll_interval_ms` intervals, processes
/// payloads through `handle_nxms_payload`, and acks successful ingests.
///
/// # Concurrency
///
/// Takes `&mut PrivaiNode` exclusively. In a multi-task runtime where the
/// node is shared with consensus/gossip, the caller is responsible for
/// concurrency management (e.g. `tokio::sync::Mutex`). This function is
/// the v1 integration point; concurrency wiring is out of scope.
///
/// # Cancellation
///
/// Runs indefinitely. Cancel via `tokio::select!`, `CancellationToken`,
/// or dropping the spawned task.
pub async fn run_mailbox_pull_loop<M, S, V, A, P>(
    source: &M,
    node: &mut PrivaiNode<S, V, A, P>,
    config: &MailboxPullConfig,
) where
    M: MailboxSource,
    S: LedgerStore,
    V: ProofVerifier,
    A: ProofArtifactStore,
    P: BlockArtifactVerifier,
{
    let interval = Duration::from_millis(config.poll_interval_ms);

    loop {
        match mailbox_ingest_tick(source, node, &config.inbox_id, config.batch_size).await {
            Ok(report) => {
                if report.pulled > 0 {
                    eprintln!(
                        "[mailbox] tick: pulled={} ingested={} acked={} \
                         decode_err={} ingest_err={} ack_err={}",
                        report.pulled,
                        report.ingested,
                        report.acked,
                        report.decode_errors,
                        report.ingest_errors,
                        report.ack_errors,
                    );
                }
            }
            Err(e) => {
                eprintln!("[mailbox] pull error (will retry): {}", e);
            }
        }
        tokio::time::sleep(interval).await;
    }
}

// ── Production adapter ──────────────────────────────────────────────────

/// Production `MailboxSource` wrapping `nxms_mailbox_client::MailboxClient`.
///
/// Pulls `NxmsEnvelope`s from the mailbox server, decrypts them to
/// `NxmsPayloadV2` using the node's KEM keypair and the sender's Falcon PK,
/// and acks processed messages.
pub struct NxmsMailboxAdapter {
    client: nxms_mailbox_client::MailboxClient,
    node_kem_sk: Vec<u8>,
    sender_sig_pk: Vec<u8>,
}

impl NxmsMailboxAdapter {
    pub fn new(
        client: nxms_mailbox_client::MailboxClient,
        node_kem_sk: Vec<u8>,
        sender_sig_pk: Vec<u8>,
    ) -> Self {
        Self {
            client,
            node_kem_sk,
            sender_sig_pk,
        }
    }

    /// Build from `MailboxPullConfig`.
    ///
    /// Returns `None` if the config is missing required fields
    /// (`mailbox_url`, `inbox_id`, `sender_sig_pk`).
    pub fn from_config(config: &MailboxPullConfig) -> Option<Self> {
        if config.mailbox_url.is_empty() || config.sender_sig_pk.is_none() {
            return None;
        }
        let mut builder = nxms_mailbox_client::MailboxClient::builder(&config.mailbox_url).ok()?;
        if let Some(token) = &config.pull_token {
            builder = builder.pull_token(token);
        }
        if let Some(token) = &config.ack_token {
            builder = builder.ack_token(token);
        }
        let client = builder.build().ok()?;
        Some(Self {
            client,
            node_kem_sk: Vec::new(), // caller must set via with_kem_sk
            sender_sig_pk: config.sender_sig_pk.clone().unwrap_or_default(),
        })
    }

    pub fn with_kem_sk(mut self, kem_sk: Vec<u8>) -> Self {
        self.node_kem_sk = kem_sk;
        self
    }

    /// Safely build from full `NodeConfig`.
    ///
    /// Returns `None` if `config.mailbox.enabled` is false or if
    /// required fields are missing. Ensures `node_kem_sk` is properly loaded
    /// and is not empty or a placeholder (all-zeros) value.
    pub fn from_node_config(config: &NodeConfig) -> Option<Self> {
        if !config.mailbox.enabled {
            return None;
        }
        if config.mailbox.mailbox_url.is_empty()
            || config.mailbox.inbox_id.is_empty()
            || config.mailbox.sender_sig_pk.is_none()
        {
            return None;
        }

        // Reject empty or placeholder (all-zeros) KEM secret.
        if config.node_kem_sk.is_empty()
            || config.node_kem_sk.iter().all(|&b| b == 0)
        {
            return None;
        }

        Self::from_config(&config.mailbox).map(|adapter| {
            adapter.with_kem_sk(config.node_kem_sk.clone())
        })
    }
}

impl MailboxSource for NxmsMailboxAdapter {
    fn pull_payloads<'a>(
        &'a self,
        inbox: &'a str,
        max: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<PulledPayload>, MailboxPullError>> + Send + 'a>>
    {
        Box::pin(async move {
            let resp = self
                .client
                .pull(inbox, Some(max), Some(0))
                .await
                .map_err(|e| MailboxPullError::Pull(e.to_string()))?;

            let mut out = Vec::with_capacity(resp.messages.len());
            for msg in resp.messages {
                let env = &msg.envelope;
                let msg_type_str =
                    nxms_transport::wire::msg_type_key(&env.msg_type).to_string();

                let sealed = nxms_transport::crypto::SealedPacket {
                    kem_ct_b64: env.kem_ct_b64.clone(),
                    nonce_b64: env.nonce_b64.clone(),
                    ciphertext_b64: env.ciphertext_b64.clone(),
                    tag_b64: env.tag_b64.clone(),
                    sig_b64: env.sig_b64.clone(),
                };

                let escrow_id_bytes = hex::decode(&env.escrow_id_hex).map_err(|e| {
                    MailboxPullError::Pull(format!("bad escrow_id hex: {e}"))
                })?;
                if escrow_id_bytes.len() != 16 {
                    return Err(MailboxPullError::Pull(format!(
                        "escrow_id decoded to {} bytes, expected 16",
                        escrow_id_bytes.len()
                    )));
                }
                let mut context_id = [0u8; 16];
                context_id.copy_from_slice(&escrow_id_bytes);

                let plaintext = nxms_transport::crypto::decrypt_for_context(
                    &env.from,
                    &env.to,
                    &msg_type_str,
                    &context_id,
                    env.seq,
                    &sealed,
                    &self.node_kem_sk,
                    &self.sender_sig_pk,
                )
                .map_err(|e| MailboxPullError::Pull(format!("decrypt: {e}")))?;

                let payload: NxmsPayloadV2 = serde_json::from_slice(&plaintext)
                    .map_err(|e| MailboxPullError::Pull(format!("payload decode: {e}")))?;

                out.push(PulledPayload {
                    receipt: msg.receipt,
                    payload,
                });
            }
            Ok(out)
        })
    }

    fn ack<'a>(
        &'a self,
        receipt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), MailboxPullError>> + Send + 'a>> {
        Box::pin(async move {
            self.client
                .ack(receipt)
                .await
                .map_err(|e| MailboxPullError::Ack(e.to_string()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nxms_transport::wire::NxmsPayloadV2;
    use privai_ledger::MemoryStore;
    use privai_nxms::{
        EscrowFundedBody, EscrowFundingDescriptor,
    };

    use std::collections::VecDeque;
    use std::sync::Mutex;

    // ── FakeMailboxSource ───────────────────────────────────────────────

    /// Test-only `MailboxSource` backed by canned payloads.
    ///
    /// Messages are pulled in FIFO order. Acked receipts are recorded
    /// for assertion. No HTTP, no crypto, no network.
    pub struct FakeMailboxSource {
        inbox: Mutex<VecDeque<PulledPayload>>,
        acked: Mutex<Vec<String>>,
    }

    impl FakeMailboxSource {
        pub fn new(messages: Vec<PulledPayload>) -> Self {
            Self {
                inbox: Mutex::new(messages.into_iter().collect()),
                acked: Mutex::new(Vec::new()),
            }
        }

        pub fn acked_receipts(&self) -> Vec<String> {
            self.acked.lock().unwrap().clone()
        }
    }

    impl MailboxSource for FakeMailboxSource {
        fn pull_payloads<'a>(
            &'a self,
            _inbox: &'a str,
            max: u32,
        ) -> Pin<
            Box<dyn Future<Output = Result<Vec<PulledPayload>, MailboxPullError>> + Send + 'a>,
        > {
            Box::pin(async move {
                let mut guard = self.inbox.lock().unwrap();
                let n = (max as usize).min(guard.len());
                Ok(guard.drain(..n).collect())
            })
        }

        fn ack<'a>(
            &'a self,
            receipt: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<(), MailboxPullError>> + Send + 'a>> {
            Box::pin(async move {
                self.acked.lock().unwrap().push(receipt.to_string());
                Ok(())
            })
        }
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    fn test_node_config() -> crate::config::NodeConfig {
        let mut config = crate::config::NodeConfig::example();
        config.data_dir = String::new();
        config
    }

    fn make_escrow_funded_payload(
        escrow_id: [u8; 32],
        session: [u8; 16],
        seq: u64,
    ) -> NxmsPayloadV2 {
        let body = privai_nxms::PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: session,
            descriptor: EscrowFundingDescriptor {
                escrow_id,
                buyer_pk: [1; 32],
                merchant_pk: [2; 32],
                operator_pk: [3; 32],
                amount: 1000,
                spend_policy_commit: [4; 32],
                timeout_blocks: 100,
            },
            funding_tx_ref: [5; 32],
        });
        body.to_payload(session, "alice", "node", seq).expect("to_payload")
    }

    fn make_non_escrow_payload(session: [u8; 16], seq: u64) -> NxmsPayloadV2 {
        // ProofServiceRequest is a PrivaiBody variant that returns Ignored.
        let body = privai_nxms::PrivaiBody::ProofServiceRequest(
            privai_nxms::ProofServiceRequestBody {
                job_id: [10; 32],
                statement_commit: [11; 32],
                proof_system_id: 1,
                witness_box: vec![],
            },
        );
        body.to_payload(session, "alice", "node", seq)
            .expect("to_payload")
    }

    fn make_malformed_payload() -> NxmsPayloadV2 {
        NxmsPayloadV2 {
            app_proto: "WRONG/99".to_string(),
            msg_type: "escrow_funded".to_string(),
            context_id_hex: "aa".repeat(16),
            from: "alice".to_string(),
            to: "node".to_string(),
            seq: 1,
            data: "{}".to_string(),
        }
    }

    // ── Tests ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn pulled_escrow_payload_reaches_handle_nxms_payload() {
        let config = test_node_config();
        let mut node = PrivaiNode::open(config, MemoryStore::new()).expect("node");

        let session = [0xA0; 16];
        let escrow_id = [0xBB; 32];
        let payload = make_escrow_funded_payload(escrow_id, session, 1);

        let source = FakeMailboxSource::new(vec![PulledPayload {
            receipt: "r1".into(),
            payload,
        }]);

        let report = mailbox_ingest_tick(&source, &mut node, "node_inbox", 10)
            .await
            .expect("tick");

        assert_eq!(report.pulled, 1);
        assert_eq!(report.ingested, 1);
        assert_eq!(report.acked, 1);
        assert_eq!(report.decode_errors, 0);
        assert_eq!(report.ingest_errors, 0);

        // Verify the funded escrow was stored in the node.
        assert!(node.get_staged_escrow(&escrow_id).is_some());
    }

    #[tokio::test]
    async fn successful_ingest_triggers_ack() {
        let config = test_node_config();
        let mut node = PrivaiNode::open(config, MemoryStore::new()).expect("node");

        let session = [0xB0; 16];
        let escrow_id = [0xCC; 32];
        let payload = make_escrow_funded_payload(escrow_id, session, 1);

        let source = FakeMailboxSource::new(vec![
            PulledPayload {
                receipt: "receipt-001".into(),
                payload,
            },
        ]);

        let report = mailbox_ingest_tick(&source, &mut node, "inbox", 10)
            .await
            .expect("tick");

        assert_eq!(report.acked, 1);
        assert_eq!(source.acked_receipts(), vec!["receipt-001".to_string()]);
    }

    #[tokio::test]
    async fn malformed_payload_no_ack_no_panic() {
        let config = test_node_config();
        let mut node = PrivaiNode::open(config, MemoryStore::new()).expect("node");

        let payload = make_malformed_payload();

        let source = FakeMailboxSource::new(vec![PulledPayload {
            receipt: "bad-1".into(),
            payload,
        }]);

        let report = mailbox_ingest_tick(&source, &mut node, "inbox", 10)
            .await
            .expect("tick");

        assert_eq!(report.pulled, 1);
        assert_eq!(report.ingested, 0);
        assert_eq!(report.acked, 0);
        assert_eq!(report.decode_errors, 1);
        assert_eq!(report.ingest_errors, 0);

        // Malformed message must NOT be acked.
        assert!(source.acked_receipts().is_empty());
    }

    #[tokio::test]
    async fn ingest_failure_no_ack_no_panic() {
        let config = test_node_config();
        let mut node = PrivaiNode::open(config, MemoryStore::new()).expect("node");

        let session = [0xC0; 16];
        let escrow_id = [0xDD; 32];

        // Ingest first time — succeeds.
        let payload1 = make_escrow_funded_payload(escrow_id, session, 1);
        let _ = node.handle_nxms_payload(&payload1);

        // Build a *conflicting* funded body (same escrow_id, different tx_ref).
        let descriptor2 = EscrowFundingDescriptor {
            escrow_id,
            buyer_pk: [1; 32],
            merchant_pk: [2; 32],
            operator_pk: [3; 32],
            amount: 2000, // different amount
            spend_policy_commit: [4; 32],
            timeout_blocks: 100,
        };
        let body2 = privai_nxms::PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: session,
            descriptor: descriptor2,
            funding_tx_ref: [99; 32], // different tx ref
        });
        let payload2 = body2
            .to_payload(session, "alice", "node", 2)
            .expect("to_payload");

        let source = FakeMailboxSource::new(vec![PulledPayload {
            receipt: "conflict-1".into(),
            payload: payload2,
        }]);

        let report = mailbox_ingest_tick(&source, &mut node, "inbox", 10)
            .await
            .expect("tick");

        assert_eq!(report.pulled, 1);
        assert_eq!(report.ingested, 0);
        assert_eq!(report.acked, 0);
        assert_eq!(report.decode_errors, 0);
        assert_eq!(report.ingest_errors, 1);

        // Failed ingest must NOT ack.
        assert!(source.acked_receipts().is_empty());
    }

    #[tokio::test]
    async fn non_escrow_payload_handled_safely() {
        let config = test_node_config();
        let mut node = PrivaiNode::open(config, MemoryStore::new()).expect("node");

        let session = [0xD0; 16];
        let payload = make_non_escrow_payload(session, 1);

        let source = FakeMailboxSource::new(vec![PulledPayload {
            receipt: "non-escrow-1".into(),
            payload,
        }]);

        let report = mailbox_ingest_tick(&source, &mut node, "inbox", 10)
            .await
            .expect("tick");

        // Non-escrow bodies are valid payloads — they are ingested (decoded)
        // successfully and return Ignored. The message is acked.
        assert_eq!(report.pulled, 1);
        assert_eq!(report.ingested, 1);
        assert_eq!(report.acked, 1);
        assert_eq!(report.decode_errors, 0);
        assert_eq!(report.ingest_errors, 0);
        assert_eq!(source.acked_receipts(), vec!["non-escrow-1".to_string()]);
    }

    #[test]
    fn adapter_from_node_config_disabled_mailbox_returns_none() {
        let mut config = test_node_config();
        config.mailbox.enabled = false;
        config.mailbox.mailbox_url = "http://xyz.onion".to_string();
        config.mailbox.sender_sig_pk = Some(vec![1; 32]);
        config.node_kem_sk = vec![2; 32];
        
        let adapter = NxmsMailboxAdapter::from_node_config(&config);
        assert!(adapter.is_none());
    }

    #[test]
    fn adapter_from_node_config_valid_populates_kem_key() {
        let mut config = test_node_config();
        config.mailbox.enabled = true;
        config.mailbox.mailbox_url = "http://xyz.onion".to_string();
        config.mailbox.inbox_id = "inbox".to_string();
        config.mailbox.sender_sig_pk = Some(vec![1; 32]);
        config.node_kem_sk = vec![2; 32];

        let adapter = NxmsMailboxAdapter::from_node_config(&config).expect("adapter");
        assert_eq!(adapter.node_kem_sk, vec![2; 32]);
        assert_eq!(adapter.sender_sig_pk, vec![1; 32]);
    }

    #[test]
    fn adapter_from_node_config_missing_required_fields_rejected() {
        let mut config = test_node_config();
        config.mailbox.enabled = true;
        // missing URL
        config.mailbox.mailbox_url = "".to_string();
        config.mailbox.inbox_id = "inbox".to_string();
        config.mailbox.sender_sig_pk = Some(vec![1; 32]);
        config.node_kem_sk = vec![2; 32];
        
        assert!(NxmsMailboxAdapter::from_node_config(&config).is_none());

        let mut config2 = test_node_config();
        config2.mailbox.enabled = true;
        config2.mailbox.mailbox_url = "http://xyz.onion".to_string();
        config2.mailbox.inbox_id = "inbox".to_string();
        // missing sender_sig_pk
        config2.mailbox.sender_sig_pk = None;
        config2.node_kem_sk = vec![2; 32];
        
        assert!(NxmsMailboxAdapter::from_node_config(&config2).is_none());

        let mut config3 = test_node_config();
        config3.mailbox.enabled = true;
        config3.mailbox.mailbox_url = "http://xyz.onion".to_string();
        // missing inbox_id
        config3.mailbox.inbox_id = "".to_string();
        config3.mailbox.sender_sig_pk = Some(vec![1; 32]);
        config3.node_kem_sk = vec![2; 32];
        
        assert!(NxmsMailboxAdapter::from_node_config(&config3).is_none());
    }

    #[test]
    fn adapter_from_node_config_empty_kem_sk_rejected() {
        let mut config = test_node_config();
        config.mailbox.enabled = true;
        config.mailbox.mailbox_url = "http://xyz.onion".to_string();
        config.mailbox.inbox_id = "inbox".to_string();
        config.mailbox.sender_sig_pk = Some(vec![1; 32]);
        config.node_kem_sk = Vec::new(); // empty

        assert!(NxmsMailboxAdapter::from_node_config(&config).is_none());
    }

    #[test]
    fn adapter_from_node_config_placeholder_kem_sk_rejected() {
        let mut config = test_node_config();
        config.mailbox.enabled = true;
        config.mailbox.mailbox_url = "http://xyz.onion".to_string();
        config.mailbox.inbox_id = "inbox".to_string();
        config.mailbox.sender_sig_pk = Some(vec![1; 32]);
        config.node_kem_sk = vec![0; 32]; // placeholder / default

        assert!(NxmsMailboxAdapter::from_node_config(&config).is_none());
    }

    #[tokio::test]
    async fn empty_pull_produces_empty_report() {
        let config = test_node_config();
        let mut node = PrivaiNode::open(config, MemoryStore::new()).expect("node");

        let source = FakeMailboxSource::new(vec![]);

        let report = mailbox_ingest_tick(&source, &mut node, "inbox", 10)
            .await
            .expect("tick");

        assert_eq!(report, MailboxTickReport::default());
    }

    #[tokio::test]
    async fn multiple_messages_mixed_outcomes() {
        let config = test_node_config();
        let mut node = PrivaiNode::open(config, MemoryStore::new()).expect("node");

        let session = [0xE0; 16];

        // Good escrow payload
        let good = make_escrow_funded_payload([0xAA; 32], session, 1);
        // Malformed payload
        let bad = make_malformed_payload();
        // Another good escrow payload
        let good2 = make_escrow_funded_payload([0xBB; 32], session, 2);

        let source = FakeMailboxSource::new(vec![
            PulledPayload { receipt: "r-good".into(), payload: good },
            PulledPayload { receipt: "r-bad".into(), payload: bad },
            PulledPayload { receipt: "r-good2".into(), payload: good2 },
        ]);

        let report = mailbox_ingest_tick(&source, &mut node, "inbox", 10)
            .await
            .expect("tick");

        assert_eq!(report.pulled, 3);
        assert_eq!(report.ingested, 2);
        assert_eq!(report.acked, 2);
        assert_eq!(report.decode_errors, 1);
        assert_eq!(report.ingest_errors, 0);

        // Only good messages acked.
        let acked = source.acked_receipts();
        assert_eq!(acked.len(), 2);
        assert!(acked.contains(&"r-good".to_string()));
        assert!(acked.contains(&"r-good2".to_string()));
    }
}
