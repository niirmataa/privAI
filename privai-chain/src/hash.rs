use blake3::Hasher;

use crate::canonical::CanonicalEncode;
use crate::primitives::Hash32;

pub const NOTE_DOMAIN: &str = "privai:note:v0";
pub const NOTE_PAYLOAD_DOMAIN: &str = "privai:note-payload:v0";
pub const POLICY_DOMAIN: &str = "privai:policy:v0";
pub const BUNDLE_DOMAIN: &str = "privai:bundle:v0";
pub const NULLIFIER_DOMAIN: &str = "privai:nullifier:v0";
pub const STATEMENT_DOMAIN: &str = "privai:stmt:v0";
pub const AUX_DOMAIN: &str = "privai:aux:v0";
pub const TX_DOMAIN: &str = "privai:tx:v0";
pub const BLOCK_HEADER_DOMAIN: &str = "privai:block-header:v0";
pub const PROOF_CERT_DOMAIN: &str = "privai:proof-cert:v0";
pub const MERKLE_DOMAIN: &str = "privai:merkle:v0";
pub const MERKLE_EMPTY_DOMAIN: &str = "privai:merkle-empty:v0";
pub const EPOCH_SEED_DOMAIN: &str = "privai:epoch-seed:v0";
pub const TX_SIGNING_DOMAIN: &str = "privai:tx-signing:v0";
pub const FALCON_PK_DOMAIN: &str = "privai:falcon-pk:v0";

/// Compute the canonical hash of a Falcon public key.
/// Used in SpendPolicy pk_hash fields and escrow signer identification.
pub fn falcon_pk_hash(pk: &[u8]) -> Hash32 {
    domain_hash(FALCON_PK_DOMAIN, &[pk])
}

pub fn domain_hash(domain: &str, parts: &[&[u8]]) -> Hash32 {
    let mut hasher = Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(part);
    }
    *hasher.finalize().as_bytes()
}

pub fn hash_encoded<T: CanonicalEncode>(domain: &str, value: &T) -> Hash32 {
    let bytes = value.to_canonical_bytes();
    domain_hash(domain, &[&bytes])
}

pub fn merkle_root<I>(hashes: I) -> Hash32
where
    I: IntoIterator<Item = Hash32>,
{
    let mut layer: Vec<Hash32> = hashes.into_iter().collect();
    if layer.is_empty() {
        return domain_hash(MERKLE_EMPTY_DOMAIN, &[]);
    }

    while layer.len() > 1 {
        let mut next = Vec::with_capacity(layer.len().div_ceil(2));
        for pair in layer.chunks(2) {
            let left = pair[0];
            let right = pair.get(1).copied().unwrap_or(left);
            next.push(domain_hash(MERKLE_DOMAIN, &[&left, &right]));
        }
        layer = next;
    }

    layer[0]
}

pub fn derive_epoch_seed(last_epoch_qc_hash: &Hash32, epoch_number: u64) -> Hash32 {
    domain_hash(
        EPOCH_SEED_DOMAIN,
        &[last_epoch_qc_hash, &epoch_number.to_le_bytes()],
    )
}
