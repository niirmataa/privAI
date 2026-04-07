use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use privai_chain::{Block, Hash32, Nullifier, OutputNote, DEFAULT_CHAIN_ID};

/// Serde helper module for using `[u8; 32]` as JSON map keys.
///
/// serde_json cannot serialize `BTreeMap<[u8; 32], V>` by default because
/// `[u8; 32]` serializes as a JSON array, but JSON requires string keys.
/// This module provides hex encoding for the key type.
mod hex_key_map {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::BTreeMap;

    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    fn bytes_to_hex(bytes: &[u8]) -> String {
        let mut hex = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            hex.push(HEX_CHARS[(b >> 4) as usize] as char);
            hex.push(HEX_CHARS[(b & 0x0f) as usize] as char);
        }
        hex
    }

    fn hex_to_bytes(hex: &str) -> Result<[u8; 32], String> {
        if hex.len() != 64 {
            return Err(format!("expected 64 hex chars, got {}", hex.len()));
        }
        let mut bytes = [0u8; 32];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let hi = hex_digit(chunk[0])?;
            let lo = hex_digit(chunk[1])?;
            bytes[i] = (hi << 4) | lo;
        }
        Ok(bytes)
    }

    fn hex_digit(c: u8) -> Result<u8, String> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'A'..=b'F' => Ok(c - b'A' + 10),
            _ => Err(format!("invalid hex char: {}", c as char)),
        }
    }

    pub fn serialize<S, V>(map: &BTreeMap<[u8; 32], V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        V: Serialize,
    {
        use serde::ser::SerializeMap;
        let mut ser_map = serializer.serialize_map(Some(map.len()))?;
        for (k, v) in map {
            ser_map.serialize_entry(&bytes_to_hex(k), v)?;
        }
        ser_map.end()
    }

    pub fn deserialize<'de, D, V>(deserializer: D) -> Result<BTreeMap<[u8; 32], V>, D::Error>
    where
        D: Deserializer<'de>,
        V: Deserialize<'de>,
    {
        let string_map: BTreeMap<String, V> = BTreeMap::deserialize(deserializer)?;
        let mut result = BTreeMap::new();
        for (k, v) in string_map {
            let key = hex_to_bytes(&k).map_err(serde::de::Error::custom)?;
            result.insert(key, v);
        }
        Ok(result)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteStatus {
    Unspent,
    Spent {
        nullifier: Nullifier,
        spent_in_block: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteRecord {
    pub note: OutputNote,
    pub created_in_block: Option<u64>,
    pub status: NoteStatus,
}

use privai_chain::QuorumCertificate;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConsensusSafetyState {
    pub current_view: u32,
    pub last_voted_view: u32,
    pub current_round: u32,
    pub locked_qc: Option<QuorumCertificate>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerSnapshot {
    pub chain_id: u32,
    pub height: u64,
    pub tip_hash: Hash32,
    pub blocks: BTreeMap<u64, Block>,
    #[serde(with = "hex_key_map")]
    pub notes: BTreeMap<Hash32, NoteRecord>,
    pub spent_nullifiers: BTreeSet<Nullifier>,
    /// Osobny zbiór ticket nullifiers — unika kolizji z note nullifiers.
    pub spent_ticket_nullifiers: BTreeSet<Nullifier>,
    /// Persystowane QCs po height — potrzebne do state sync.
    pub qcs: BTreeMap<u64, QuorumCertificate>,
    pub consensus_safety: ConsensusSafetyState,
}

impl LedgerSnapshot {
    pub fn genesis(chain_id: u32) -> Self {
        Self {
            chain_id,
            height: 0,
            tip_hash: [0; 32],
            blocks: BTreeMap::new(),
            notes: BTreeMap::new(),
            spent_nullifiers: BTreeSet::new(),
            spent_ticket_nullifiers: BTreeSet::new(),
            qcs: BTreeMap::new(),
            consensus_safety: ConsensusSafetyState::default(),
        }
    }

    pub fn is_ticket_nullifier_spent(&self, nullifier: &Nullifier) -> bool {
        self.spent_ticket_nullifiers.contains(nullifier)
    }

    pub fn mark_ticket_nullifier_spent(&mut self, nullifier: Nullifier) {
        self.spent_ticket_nullifiers.insert(nullifier);
    }
}

impl Default for LedgerSnapshot {
    fn default() -> Self {
        Self::genesis(DEFAULT_CHAIN_ID)
    }
}
