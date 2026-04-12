//! Discovery layer for privAI V0.
//!
//! Architecture (Phase 1 / V0 Simple 4):
//! - Discovery is purely off-chain, via NXMS mailbox.
//! - User sends encrypted `DiscoveryQuery`.
//! - Miner decrypts (if capable) and replies with `ComputeOffering`.
//! - No public registries, no profiles.

use serde::{Deserialize, Serialize};

use privai_chain::canonical::{write_fixed, write_u64, write_u8, write_vec_bytes, CanonicalEncode};
use privai_chain::compute_lease::{NetworkMode, PrivacyClass, ResourceClass};
use privai_chain::primitives::Hash32;

/// A private query sent by a User to the NXMS mailbox network.
/// Wrapped in an `NxmsEnvelope` (FrodoKEM) so only active miners can read it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryQuery {
    pub min_resource_class: ResourceClass,
    pub max_price_aPVA_per_unit: u64,
    pub min_duration_units: u64,
    pub preferred_network_mode: NetworkMode,
    pub preferred_privacy_class: PrivacyClass,
    /// Temporary symmetric key or pubkey for the miner's response
    pub response_kem_pk: Vec<u8>,
    /// Deadline for response (Unix timestamp)
    pub expires_at_unix: u64,
}

impl CanonicalEncode for DiscoveryQuery {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.min_resource_class.tag());

        // Exact encoding of ResourceClass matching compute_lease.rs
        match &self.min_resource_class {
            ResourceClass::Gpu { class, vram_mb } => {
                write_u8(out, class.tag());
                write_fixed(out, &vram_mb.to_le_bytes());
            }
            ResourceClass::Cpu { tier, cores } => {
                write_u8(out, tier.tag() as u8);
                write_fixed(out, &cores.to_le_bytes());
            }
            ResourceClass::Memory { ram_mb } => {
                write_fixed(out, &ram_mb.to_le_bytes());
            }
            ResourceClass::Composite {
                gpu,
                cpu,
                ram_mb,
                storage_mb,
            } => {
                match gpu {
                    Some((gc, vram)) => {
                        write_u8(out, 1);
                        write_u8(out, gc.tag());
                        write_fixed(out, &vram.to_le_bytes());
                    }
                    None => write_u8(out, 0),
                }
                match cpu {
                    Some((ct, cores)) => {
                        write_u8(out, 1);
                        write_u8(out, ct.tag() as u8);
                        write_fixed(out, &cores.to_le_bytes());
                    }
                    None => write_u8(out, 0),
                }
                write_fixed(out, &ram_mb.to_le_bytes());
                write_fixed(out, &storage_mb.to_le_bytes());
            }
        }

        write_u64(out, self.max_price_aPVA_per_unit);
        write_u64(out, self.min_duration_units);
        write_u8(out, self.preferred_network_mode as u8);
        write_u8(out, self.preferred_privacy_class as u8);
        write_vec_bytes(out, &vec![self.response_kem_pk.clone()]);
        write_u64(out, self.expires_at_unix);
    }
}

/// NxmsEnvelope: The transport layer wrapper for discovery messages.
/// Ensures the payload is encrypted using FrodoKEM + XChaCha20Poly1305.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NxmsEnvelope {
    pub recipient_mailbox_id: Hash32,
    pub kem_ciphertext: Vec<u8>,
    pub aead_nonce: [u8; 24],
    pub encrypted_payload: Vec<u8>, // Contains encoded DiscoveryQuery or ComputeOffering
    pub auth_tag: [u8; 16],
}

impl CanonicalEncode for NxmsEnvelope {
    fn encode(&self, out: &mut Vec<u8>) {
        write_fixed(out, &self.recipient_mailbox_id);
        write_vec_bytes(out, &vec![self.kem_ciphertext.clone()]);
        write_fixed(out, &self.aead_nonce);
        write_vec_bytes(out, &vec![self.encrypted_payload.clone()]);
        write_fixed(out, &self.auth_tag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use privai_chain::compute_lease::GpuClass;

    #[test]
    fn test_discovery_query_encode() {
        let query = DiscoveryQuery {
            min_resource_class: ResourceClass::Gpu {
                class: GpuClass::A100,
                vram_mb: 40_000,
            },
            max_price_aPVA_per_unit: 10_000_000,
            min_duration_units: 120,
            preferred_network_mode: NetworkMode::Isolated,
            preferred_privacy_class: PrivacyClass::Vm,
            response_kem_pk: vec![0xAA; 32],
            expires_at_unix: 1700000000,
        };

        let mut encoded = Vec::new();
        query.encode(&mut encoded);
        assert!(!encoded.is_empty());
    }
}
