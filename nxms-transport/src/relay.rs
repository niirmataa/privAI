//! Relay routing for privAI V0.
//!
//! Onion-style routing: each relay sees only prev/next hop.
//! User wraps payload in N layers of encryption (one per relay).
//! Each relay decrypts one layer and forwards to the next hop.
//!
//! This module does NOT replace Tor. It adds privAI-internal relay hops
//! on top of Tor (or without Tor for nxms_only mode).

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

// ── Relay Layer ────────────────────────────────────────────────────────

/// A single layer in the onion-routed message.
///
/// Each relay receives this struct, decrypts it with its own key,
/// and forwards the inner payload to the next hop.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelayLayer {
    /// Next hop identifier (peer id or onion address).
    pub next_hop: String,
    /// Next hop port.
    pub next_hop_port: u16,
    /// Encrypted payload for the next hop (or final destination).
    /// If this is the last relay, this is the actual message.
    pub encrypted_payload: Vec<u8>,
    /// Is this the final hop? (true = deliver, false = forward)
    pub is_final: bool,
}

/// The complete onion-routed message sent to the first relay.
///
/// User builds this by wrapping the payload in N layers.
/// First relay receives this, decrypts its layer, sees the next hop.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnionMessage {
    /// The outermost encrypted layer.
    pub outer_layer: Vec<u8>,
    /// KEM ciphertext for the first relay (to derive shared secret).
    pub first_hop_kem_ct: Vec<u8>,
    /// AEAD nonce for the first layer.
    pub first_hop_nonce: Vec<u8>,
}

/// Route specification for building an onion message.
#[derive(Clone, Debug)]
pub struct RelayRoute {
    /// Ordered list of relay hops (first = entry, last = exit).
    pub hops: Vec<RelayHop>,
    /// Final destination after the last relay.
    pub destination: String,
    /// Final destination port.
    pub destination_port: u16,
}

/// A single hop in the relay route.
#[derive(Clone, Debug)]
pub struct RelayHop {
    /// Peer identifier.
    pub peer_id: String,
    /// Peer address (onion or DNS).
    pub host: String,
    /// Peer port.
    pub port: u16,
    /// Peer FrodoKEM public key (for encryption).
    pub kem_pk: Vec<u8>,
}

/// Instructions for the relay after processing a layer.
#[derive(Clone, Debug)]
pub struct RelayForward {
    /// Next hop identifier.
    pub next_hop: String,
    /// Next hop port.
    pub next_hop_port: u16,
    /// Encrypted payload to forward (relay cannot read this).
    pub payload: Vec<u8>,
    /// KEM ciphertext for the next hop.
    pub next_hop_kem_ct: Vec<u8>,
    /// AEAD nonce for the next layer.
    pub next_hop_nonce: Vec<u8>,
    /// Is this the final hop? (true = deliver to destination)
    pub is_final: bool,
}

/// Per-hop envelope containing KEM ciphertext + AEAD nonce + encrypted payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelayEnvelope {
    /// KEM ciphertext for this hop (used to derive shared secret).
    pub kem_ct: Vec<u8>,
    /// AEAD nonce for this layer.
    pub nonce: Vec<u8>,
    /// The encrypted RelayLayer (after decryption with shared secret).
    pub encrypted_layer: Vec<u8>,
    /// AEAD tag.
    pub tag: Vec<u8>,
}

// ── Route building ─────────────────────────────────────────────────────

impl RelayRoute {
    /// Build a route from a list of peers.
    pub fn from_peers(
        peers: &[super::peers::Peer],
        destination: &str,
        destination_port: u16,
    ) -> Result<Self> {
        if peers.is_empty() {
            return Err(anyhow!("route must have at least one relay"));
        }

        use base64::Engine;
        let hops: Vec<RelayHop> = peers
            .iter()
            .map(|p| {
                let kem_pk = base64::engine::general_purpose::STANDARD
                    .decode(&p.kem_pk_b64)
                    .map_err(|e| anyhow!("base64 decode: {e}"))?;
                Ok(RelayHop {
                    peer_id: p.id.clone(),
                    host: p.host.clone(),
                    port: p.port,
                    kem_pk,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            hops,
            destination: destination.to_string(),
            destination_port,
        })
    }

    /// Number of hops in the route.
    pub fn hop_count(&self) -> usize {
        self.hops.len()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_hop(id: &str) -> RelayHop {
        RelayHop {
            peer_id: id.to_string(),
            host: format!("{id}.onion"),
            port: 9000,
            kem_pk: vec![0u8; 16],
        }
    }

    #[test]
    fn route_hop_count() {
        let route = RelayRoute {
            hops: vec![test_hop("r1"), test_hop("r2"), test_hop("r3")],
            destination: "final.onion".to_string(),
            destination_port: 8080,
        };
        assert_eq!(route.hop_count(), 3);
    }

    #[test]
    fn relay_layer_serializes() {
        let layer = RelayLayer {
            next_hop: "relay2.onion".to_string(),
            next_hop_port: 9001,
            encrypted_payload: vec![0x01, 0x02, 0x03],
            is_final: false,
        };
        let bytes = serde_json::to_vec(&layer).unwrap();
        let deserialized: RelayLayer = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(deserialized.next_hop, "relay2.onion");
        assert!(!deserialized.is_final);
    }

    #[test]
    fn final_layer_is_marked() {
        let layer = RelayLayer {
            next_hop: "destination.onion".to_string(),
            next_hop_port: 8080,
            encrypted_payload: vec![0xFF],
            is_final: true,
        };
        assert!(layer.is_final);
    }

    #[test]
    fn relay_forward_structure() {
        let forward = RelayForward {
            next_hop: "relay3.onion".to_string(),
            next_hop_port: 9002,
            payload: vec![0xAA, 0xBB],
            next_hop_kem_ct: vec![0x01; 32],
            next_hop_nonce: vec![0x02; 24],
            is_final: false,
        };
        assert_eq!(forward.next_hop, "relay3.onion");
        assert!(!forward.is_final);
        assert_eq!(forward.next_hop_kem_ct.len(), 32);
    }

    #[test]
    fn envelope_serializes() {
        let envelope = RelayEnvelope {
            kem_ct: vec![0x01; 32],
            nonce: vec![0x02; 24],
            encrypted_layer: vec![0x03; 64],
            tag: vec![0x04; 16],
        };
        let bytes = serde_json::to_vec(&envelope).unwrap();
        let deserialized: RelayEnvelope = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(deserialized.kem_ct.len(), 32);
        assert_eq!(deserialized.nonce.len(), 24);
    }
}
