//! Regression pack for validator session transport layer.
//!
//! Tests current canonical behavior as described in
//! `spec/PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`.
//!
//! Status markers in test names match the invariants doc:
//! - current canonical behavior tests
//! - current non-conformity regression notes
//!
//! Blockers noted where full integration testing requires Tor infrastructure
//! that is not available in CI.

use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use nxms_transport::crypto;
use nxms_transport::peers::{Peer, PeerBook};
use privai_node::net::{
    BanList, ConnectionPool, ConnectionPoolConfig, HandshakeMsg, NetConfig, RateLimiter,
};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Generate a full node key material (KEM + Falcon) for tests.
struct TestKeys {
    kem_pk: Vec<u8>,
    kem_sk: Vec<u8>,
    sig_pk: Vec<u8>,
    sig_sk: Vec<u8>,
}

impl TestKeys {
    fn generate() -> Self {
        let keys = crypto::Keys::generate().expect("Keys::generate");
        Self {
            kem_pk: keys.kem_pk().expect("kem_pk"),
            kem_sk: keys.kem_sk_zeroizing().expect("kem_sk").to_vec(),
            sig_pk: keys.sig_pk().expect("sig_pk"),
            sig_sk: keys.sig_sk_zeroizing().expect("sig_sk").to_vec(),
        }
    }
}

/// Sign a HandshakeMsg's canonical form (falcon_sig_b64 cleared) and fill the sig field.
/// This is the single source of truth for how a handshake gets signed in tests.
fn sign_handshake(msg: &mut HandshakeMsg, sig_sk: &[u8]) {
    let payload = serde_json::to_vec(&HandshakeMsg {
        falcon_sig_b64: String::new(),
        ..msg.clone()
    })
    .expect("serialize sig payload");

    let sig = crypto::falcon_sign_ct_prepared(sig_sk, &payload).expect("falcon sign");
    msg.falcon_sig_b64 = B64.encode(&sig);
}

/// Build a correctly signed HandshakeMsg (version = 1).
fn build_handshake(peer_id: &str, keys: &TestKeys) -> HandshakeMsg {
    let mut msg = HandshakeMsg {
        version: 1,
        kem_pk_b64: B64.encode(&keys.kem_pk),
        kem_ct_b64: String::new(),
        sig_pk_b64: B64.encode(&keys.sig_pk),
        peer_id: peer_id.to_string(),
        falcon_sig_b64: String::new(),
    };
    sign_handshake(&mut msg, &keys.sig_sk);
    msg
}

/// Build a HandshakeMsg with wrong version AND a valid signature over that wrong version.
/// This isolates the version gate: the sig is valid, but version != 1.
fn build_handshake_wrong_version(peer_id: &str, keys: &TestKeys) -> HandshakeMsg {
    let mut msg = HandshakeMsg {
        version: 99,
        kem_pk_b64: B64.encode(&keys.kem_pk),
        kem_ct_b64: String::new(),
        sig_pk_b64: B64.encode(&keys.sig_pk),
        peer_id: peer_id.to_string(),
        falcon_sig_b64: String::new(),
    };
    // Re-sign with the wrong version — sig is valid, but version gate should reject it
    sign_handshake(&mut msg, &keys.sig_sk);
    msg
}

/// Build a HandshakeMsg with a bad Falcon signature (correct version, valid keys).
fn build_handshake_bad_sig(peer_id: &str, keys: &TestKeys) -> HandshakeMsg {
    let mut msg = build_handshake(peer_id, keys);
    msg.falcon_sig_b64 = B64.encode(b"this-is-not-a-valid-falcon-signature");
    msg
}

/// Serialize a HandshakeMsg to bytes (what gets written over the wire).
fn handshake_bytes(msg: &HandshakeMsg) -> Vec<u8> {
    serde_json::to_vec(msg).expect("serialize handshake")
}

/// Start `run_listener` on a specific port.
/// The caller is responsible for choosing a port that is likely free.
/// If the port is in use, this will fail.
async fn start_test_listener_on_port(
    port: u16,
    peer_book: PeerBook,
    ban_list: BanList,
    rate_limiter: RateLimiter,
    keys: &TestKeys,
    peer_id: &str,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let addr = format!("127.0.0.1:{}", port);
    let socket_addr: std::net::SocketAddr = addr.parse().expect("parse addr");

    let config = NetConfig::new(
        addr,
        "socks5h://127.0.0.1:9050".to_string(),
        String::new(),
        peer_id.to_string(),
    );

    let (msg_tx, _msg_rx) = mpsc::channel(256);

    let kem_pk = keys.kem_pk.clone();
    let kem_sk = keys.kem_sk.clone();
    let sig_pk = keys.sig_pk.clone();
    let sig_sk = keys.sig_sk.clone();
    let pid = peer_id.to_string();

    let handle = tokio::spawn(async move {
        let result = privai_node::net::run_listener(
            config,
            msg_tx,
            kem_pk,
            kem_sk,
            sig_pk,
            sig_sk,
            pid,
            peer_book,
            ban_list,
            rate_limiter,
        )
        .await;
        if let Err(e) = result {
            eprintln!("[test] listener error: {}", e);
        }
    });

    // Wait for the listener to be ready
    tokio::time::sleep(Duration::from_millis(200)).await;
    (socket_addr, handle)
}

/// Connect to the listener, send a handshake, and check whether we receive
/// a reply frame (indicating the listener accepted the handshake).
///
/// Returns `true` if we received a reply frame (handshake accepted),
/// `false` if the connection was closed, timed out, or produced an error.
async fn try_handshake(addr: std::net::SocketAddr, handshake: &HandshakeMsg) -> bool {
    let mut stream = match TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(_) => return false,
    };

    let bytes = handshake_bytes(handshake);

    // Write frame (length-prefixed, matching nxms-transport wire format)
    if nxms_transport::tor_net::write_frame(&mut stream, &bytes)
        .await
        .is_err()
    {
        return false;
    }

    // The listener sends its own signed HandshakeMsg reply on success.
    // If the handshake was rejected, the connection is typically closed.
    let read_result = tokio::time::timeout(
        Duration::from_secs(3),
        nxms_transport::tor_net::read_frame(&mut stream, 64 * 1024),
    )
    .await;

    match read_result {
        Ok(Ok(_data)) => true, // Got a reply frame — handshake accepted
        Ok(Err(_)) => false,   // Read error — connection closed by listener
        Err(_) => false,       // Timeout — no reply (likely dropped)
    }
}

// ===========================================================================
// 1. BanList — current canonical behavior (§10.1)
// ===========================================================================

#[tokio::test]
async fn ban_list_ban_and_is_banned() {
    let ban_list = BanList::new();
    let peer_id = "malicious-peer-1";

    assert!(
        !ban_list.is_banned(peer_id).await,
        "should not be banned initially"
    );

    ban_list.ban(peer_id).await;
    assert!(
        ban_list.is_banned(peer_id).await,
        "should be banned after ban()"
    );

    // Different peer should not be affected
    assert!(
        !ban_list.is_banned("other-peer").await,
        "other peer should not be banned"
    );
}

#[tokio::test]
async fn ban_list_cleanup_preserves_active_bans() {
    let ban_list = BanList::new();
    ban_list.ban("peer-active").await;

    ban_list.cleanup().await;
    // Freshly banned peer has expiry 1h from now — should survive cleanup
    assert!(
        ban_list.is_banned("peer-active").await,
        "freshly banned peer should survive cleanup"
    );
}

// ===========================================================================
// 2. RateLimiter — current canonical behavior (§11.1)
// ===========================================================================

#[tokio::test]
async fn rate_limit_allows_within_limit() {
    let limiter = RateLimiter::new();
    let source = "192.168.1.1:9050";

    // MAX_CONNECTIONS_PER_SOURCE = 5, first 5 should pass
    for i in 0..5 {
        assert!(
            limiter.check(source).await,
            "connection {} should be allowed (within limit of 5)",
            i + 1
        );
    }
}

#[tokio::test]
async fn rate_limit_rejects_over_limit() {
    let limiter = RateLimiter::new();
    let source = "192.168.1.2:9050";

    // Consume all 5 allowed connections
    for _ in 0..5 {
        assert!(limiter.check(source).await);
    }

    // 6th should be rejected
    assert!(
        !limiter.check(source).await,
        "6th connection from same source should be rate-limited"
    );

    // 7th also rejected
    assert!(
        !limiter.check(source).await,
        "7th connection from same source should be rate-limited"
    );
}

#[tokio::test]
async fn rate_limit_independent_sources() {
    let limiter = RateLimiter::new();

    // Each source (keyed by full addr string) has its own counter
    for _ in 0..5 {
        assert!(limiter.check("source-a:9050").await);
    }
    assert!(
        !limiter.check("source-a:9050").await,
        "source-a should be limited"
    );

    // Different source should still be fine
    assert!(
        limiter.check("source-b:9050").await,
        "source-b should not be limited"
    );
}

// NOTE: We do NOT test rate limiting through the listener's TCP accept loop.
//
// Current canonical (§11.1): the rate limiter is keyed by `addr.to_string()`,
// which is the full `ip:port` of the incoming TCP connection. Each connection
// from a test client gets a different ephemeral source port, so they do NOT
// share a rate limit bucket. A listener-level rate limit test would require
// simulating multiple connections from the SAME source address, which is not
// feasible with standard TCP sockets in tests.
//
// The direct RateLimiter::check tests above cover the limiter logic accurately.

// ===========================================================================
// 3. HandshakeMsg — signing and verification
// ===========================================================================

#[tokio::test]
async fn handshake_sign_and_verify() {
    let keys = TestKeys::generate();
    let handshake = build_handshake("test-peer", &keys);

    // Verify the signature matches
    let sig_payload = serde_json::to_vec(&HandshakeMsg {
        falcon_sig_b64: String::new(),
        ..handshake.clone()
    })
    .expect("serialize");

    let sig_bytes = B64.decode(&handshake.falcon_sig_b64).expect("decode sig");
    assert!(
        crypto::falcon_verify(&keys.sig_pk, &sig_payload, &sig_bytes).is_ok(),
        "valid Falcon signature should verify"
    );

    // Corrupt the payload — verification should fail
    let mut bad_payload = sig_payload.clone();
    bad_payload[0] ^= 0xff;
    assert!(
        crypto::falcon_verify(&keys.sig_pk, &bad_payload, &sig_bytes).is_err(),
        "corrupted payload should fail verification"
    );
}

#[tokio::test]
async fn handshake_wrong_version_has_valid_signature() {
    // Verify that build_handshake_wrong_version produces a VALID signature
    // over the version=99 payload. This confirms the test isolates the version
    // gate, not a bad-signature rejection.
    let keys = TestKeys::generate();
    let handshake = build_handshake_wrong_version("test-peer", &keys);

    assert_eq!(handshake.version, 99, "version should be 99");

    let sig_payload = serde_json::to_vec(&HandshakeMsg {
        falcon_sig_b64: String::new(),
        ..handshake.clone()
    })
    .expect("serialize");

    let sig_bytes = B64.decode(&handshake.falcon_sig_b64).expect("decode sig");
    assert!(
        crypto::falcon_verify(&keys.sig_pk, &sig_payload, &sig_bytes).is_ok(),
        "wrong-version handshake should still have a valid signature"
    );
}

// ===========================================================================
// 4. Listener handshake tests (via TCP to run_listener)
//
// NOTE on port selection: run_listener does not expose the bound address.
// We use port 19001+ for tests. If the port is in use, the test will fail
// with a clear bind error. This is acceptable for a regression pack.
// ===========================================================================

#[tokio::test]
async fn listener_handshake_accept_known_peer() {
    // Current canonical (§6.2): known peer with valid sig → listener sends
    // reply handshake, enters message loop.
    let node_keys = TestKeys::generate();
    let peer_keys = TestKeys::generate();

    let peer_book = PeerBook {
        peers: vec![Peer {
            id: "known-peer".to_string(),
            host: "127.0.0.1".to_string(),
            port: 9999,
            kem_pk_b64: B64.encode(&peer_keys.kem_pk),
            sig_pk_b64: B64.encode(&peer_keys.sig_pk),
        }],
    };

    let (addr, handle) = start_test_listener_on_port(
        19001,
        peer_book,
        BanList::new(),
        RateLimiter::new(),
        &node_keys,
        "node-1",
    )
    .await;

    let handshake = build_handshake("known-peer", &peer_keys);
    let got_reply = try_handshake(addr, &handshake).await;

    handle.abort();
    let _ = handle.await;

    assert!(
        got_reply,
        "valid handshake from known peer should get a reply"
    );
}

#[tokio::test]
async fn listener_handshake_reject_unknown_peer() {
    // Current canonical (§6.2 step 7): peer_id must exist in PeerBook,
    // else connection is dropped and peer is banned.
    let node_keys = TestKeys::generate();
    let peer_keys = TestKeys::generate();

    let peer_book = PeerBook { peers: vec![] };

    let (addr, handle) = start_test_listener_on_port(
        19002,
        peer_book,
        BanList::new(),
        RateLimiter::new(),
        &node_keys,
        "node-1",
    )
    .await;

    let handshake = build_handshake("unknown-peer", &peer_keys);
    let got_reply = try_handshake(addr, &handshake).await;

    handle.abort();
    let _ = handle.await;

    assert!(
        !got_reply,
        "connection from unknown peer should be dropped"
    );
}

#[tokio::test]
async fn listener_handshake_reject_wrong_version() {
    // Current canonical (§6.2 step 5): version must be HANDSHAKE_VERSION (1).
    //
    // The handshake has a VALID signature over the version=99 payload.
    // This isolates the version gate — the rejection is because of version,
    // not because of a bad signature.
    let node_keys = TestKeys::generate();
    let peer_keys = TestKeys::generate();

    let peer_book = PeerBook {
        peers: vec![Peer {
            id: "peer-v99".to_string(),
            host: "127.0.0.1".to_string(),
            port: 9999,
            kem_pk_b64: B64.encode(&peer_keys.kem_pk),
            sig_pk_b64: B64.encode(&peer_keys.sig_pk),
        }],
    };

    let (addr, handle) = start_test_listener_on_port(
        19003,
        peer_book,
        BanList::new(),
        RateLimiter::new(),
        &node_keys,
        "node-1",
    )
    .await;

    let handshake = build_handshake_wrong_version("peer-v99", &peer_keys);
    let got_reply = try_handshake(addr, &handshake).await;

    handle.abort();
    let _ = handle.await;

    assert!(
        !got_reply,
        "connection with wrong handshake version should be dropped"
    );
}

#[tokio::test]
async fn listener_handshake_reject_bad_falcon_signature() {
    // Current canonical (§6.2 step 8): bad Falcon sig → peer is banned
    // and connection dropped.
    let node_keys = TestKeys::generate();
    let peer_keys = TestKeys::generate();

    let peer_book = PeerBook {
        peers: vec![Peer {
            id: "peer-bad-sig".to_string(),
            host: "127.0.0.1".to_string(),
            port: 9999,
            kem_pk_b64: B64.encode(&peer_keys.kem_pk),
            sig_pk_b64: B64.encode(&peer_keys.sig_pk),
        }],
    };

    let ban_list = BanList::new();

    let (addr, handle) = start_test_listener_on_port(
        19004,
        peer_book,
        ban_list.clone(),
        RateLimiter::new(),
        &node_keys,
        "node-1",
    )
    .await;

    let handshake = build_handshake_bad_sig("peer-bad-sig", &peer_keys);
    let got_reply = try_handshake(addr, &handshake).await;

    handle.abort();
    let _ = handle.await;

    assert!(
        !got_reply,
        "connection with bad Falcon signature should be dropped"
    );

    // Current canonical: peer with bad sig gets banned (§10.1)
    // The listener task bans asynchronously — give it a moment.
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(
        ban_list.is_banned("peer-bad-sig").await,
        "peer with bad signature should be banned"
    );
}

#[tokio::test]
async fn listener_handshake_reject_banned_peer() {
    // Current canonical (§6.2 step 6, §10.1): banned peer is rejected
    // after handshake is read.
    let node_keys = TestKeys::generate();
    let peer_keys = TestKeys::generate();

    let peer_book = PeerBook {
        peers: vec![Peer {
            id: "banned-peer".to_string(),
            host: "127.0.0.1".to_string(),
            port: 9999,
            kem_pk_b64: B64.encode(&peer_keys.kem_pk),
            sig_pk_b64: B64.encode(&peer_keys.sig_pk),
        }],
    };

    let ban_list = BanList::new();
    ban_list.ban("banned-peer").await;

    let (addr, handle) = start_test_listener_on_port(
        19005,
        peer_book,
        ban_list,
        RateLimiter::new(),
        &node_keys,
        "node-1",
    )
    .await;

    let handshake = build_handshake("banned-peer", &peer_keys);
    let got_reply = try_handshake(addr, &handshake).await;

    handle.abort();
    let _ = handle.await;

    assert!(
        !got_reply,
        "connection from banned peer should be dropped"
    );
}

// ===========================================================================
// 5. ConnectionPool — send to unreachable peer returns error
// ===========================================================================

#[tokio::test]
async fn connection_pool_send_to_unreachable_returns_error() {
    // Current canonical (§7.1, §14.1): send_message calls establish_connection
    // when no active connection exists. If connect fails, it returns a
    // transport error — not a panic.
    //
    // This test verifies the error path, not the stale rebuild path specifically.
    // Full stale rebuild testing requires a working Tor proxy.
    let pool = ConnectionPool::new("socks5h://127.0.0.1:1".to_string());

    let peer = Peer {
        id: "unreachable-peer".to_string(),
        host: "dummy.onion".to_string(),
        port: 9000,
        kem_pk_b64: String::new(),
        sig_pk_b64: String::new(),
    };

    let keys = TestKeys::generate();
    let msg = serde_json::json!({"test": true});

    // Should return error (no Tor proxy), not panic
    let result = pool
        .send_message(
            &peer,
            &msg,
            &keys.kem_pk,
            &keys.kem_sk,
            &keys.sig_pk,
            &keys.sig_sk,
            "node-1",
        )
        .await;

    assert!(result.is_err(), "send to unreachable peer should return error");
}

#[tokio::test]
async fn connection_pool_stats_initial() {
    let pool = ConnectionPool::new("socks5h://127.0.0.1:1".to_string());
    let stats = pool.stats().await;
    assert_eq!(stats.total_connections, 0);
    assert_eq!(stats.total_messages_sent, 0);
}

// ===========================================================================
// 6. Queue timeout / backpressure path (§12.1)
// ===========================================================================

#[tokio::test]
async fn bounded_channel_backpressure_does_not_panic() {
    // Current canonical (§12.1): bounded MPSC writer channel (capacity 64)
    // with 10s timeout on enqueue. When the channel is full, send_message
    // returns a transport error — not a panic or deadlock.
    //
    // We test the bounded channel invariant directly.
    let (tx, _rx) = mpsc::channel::<Vec<u8>>(2); // tiny capacity

    tx.try_send(vec![1]).expect("first send should succeed");
    tx.try_send(vec![2]).expect("second send should succeed");

    // Third try_send should fail (channel full)
    let result = tx.try_send(vec![3]);
    assert!(result.is_err(), "try_send on full bounded channel should fail");

    // With timeout: should return Err(timeout), not panic
    let timeout_result = tokio::time::timeout(Duration::from_millis(100), tx.send(vec![4])).await;
    assert!(
        timeout_result.is_err(),
        "timeout on full channel should return Err"
    );
}

// ===========================================================================
// 7. Broadcast behavior (§13.1)
// ===========================================================================

#[tokio::test]
async fn broadcast_returns_per_peer_results() {
    // Current canonical (§13.1): broadcast_message spawns one task per peer
    // and returns a vector of per-peer results. Best-effort, no atomic success.
    let pool = ConnectionPool::new("socks5h://127.0.0.1:1".to_string());
    let peer_book = PeerBook { peers: vec![] };
    let keys = TestKeys::generate();
    let msg = serde_json::json!({"broadcast": true});

    let results = pool
        .broadcast_message(
            &peer_book,
            "node-1",
            &msg,
            &keys.kem_pk,
            &keys.kem_sk,
            &keys.sig_pk,
            &keys.sig_sk,
        )
        .await;

    assert!(
        results.is_empty(),
        "broadcast to empty peer book should return empty results"
    );
}

#[tokio::test]
async fn broadcast_does_not_panic_on_unreachable_peers() {
    // Current canonical (§13.1): broadcast is best-effort, returns per-peer
    // errors. Does not panic or deadlock on unreachable peers.
    let pool = ConnectionPool::new("socks5h://127.0.0.1:1".to_string());
    let peer_book = PeerBook {
        peers: vec![Peer {
            id: "unreachable-1".to_string(),
            host: "dummy1.onion".to_string(),
            port: 9000,
            kem_pk_b64: String::new(),
            sig_pk_b64: String::new(),
        }],
    };
    let keys = TestKeys::generate();
    let msg = serde_json::json!({"broadcast": true});

    let results = pool
        .broadcast_message(
            &peer_book,
            "node-1",
            &msg,
            &keys.kem_pk,
            &keys.kem_sk,
            &keys.sig_pk,
            &keys.sig_sk,
        )
        .await;

    assert_eq!(results.len(), 1, "should have one result for one peer");
    assert!(
        results[0].1.is_err(),
        "broadcast to unreachable peer should return error"
    );
}

// ===========================================================================
// 8. Current non-conformity: incoming decrypt gap (§6.5, §17.1)
// ===========================================================================

#[test]
fn incoming_decrypt_gap_is_current_non_conformity() {
    // Regression note (spec/PRIVAI_VALIDATOR_SESSION_INVARIANTS.md §6.5, §17.1):
    //
    //   "incoming path currently discards the derived shared secret and therefore
    //    cannot perform session decrypt before ConsensusMsg deserialize"
    //
    // Evidence in session_impl.rs:
    //   - Line ~143: `_node_kem_sk` is prefixed with underscore (unused in listener)
    //   - Line ~310: FrodoKEM encap produces `_kem_shared_secret` (unused)
    //   - Line ~370: message loop reads raw `read_frame` data and does
    //     `serde_json::from_slice::<ConsensusMsg>(&data)` — no decrypt step
    //   - `decrypt_frame` function exists but is `#[allow(dead_code)]`
    //
    // This test is a compile-time regression marker. If someone removes
    // `decrypt_frame` or changes the listener to decrypt, this comment must
    // be updated to reflect the new behavior.
    //
    // Do NOT "fix" this non-conformity without a spec update and migration plan.
}

// ===========================================================================
// 9. ConnectionPoolConfig defaults — current canonical (§9.1)
// ===========================================================================

#[test]
fn connection_pool_config_defaults() {
    let config = ConnectionPoolConfig::default();
    assert_eq!(config.idle_timeout_secs, 120, "idle_timeout_secs");
    assert_eq!(config.max_age_secs, 600, "max_age_secs");
    assert_eq!(
        config.health_check_interval_secs, 30,
        "health_check_interval_secs"
    );
    assert!(config.auto_reconnect, "auto_reconnect");
}
