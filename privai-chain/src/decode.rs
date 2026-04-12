use thiserror::Error;

use crate::note::{AuxWitness, RecipientBoxPlaintext, SpendPolicy, SpendPolicyTag};
use crate::primitives::{Amount14, AmountError};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DecodeError {
    #[error("unexpected end of canonical buffer")]
    UnexpectedEof,
    #[error("trailing bytes remain after canonical decode")]
    TrailingBytes,
    #[error("invalid spend policy tag {0:#x}")]
    InvalidSpendPolicyTag(u8),
    #[error(transparent)]
    Amount(#[from] AmountError),
}

pub trait CanonicalDecode: Sized {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError>;

    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut input = bytes;
        let value = Self::decode(&mut input)?;
        if !input.is_empty() {
            return Err(DecodeError::TrailingBytes);
        }
        Ok(value)
    }
}

impl CanonicalDecode for Amount14 {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let raw = read_u16(input)?;
        Ok(Amount14::new(raw)?)
    }
}

impl CanonicalDecode for SpendPolicy {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        match read_u8(input)? {
            tag if tag == SpendPolicyTag::Single as u8 => Ok(SpendPolicy::Single {
                falcon_pk_hash: read_fixed(input)?,
            }),
            tag if tag == SpendPolicyTag::MarketplaceSettlement as u8 => {
                Ok(SpendPolicy::MarketplaceSettlement {
                    buyer_pk_hash: read_fixed(input)?,
                    seller_pk_hash: read_fixed(input)?,
                    moderator_pk_hash: read_fixed(input)?,
                    timeout_block: read_u64(input)?,
                })
            }
            tag if tag == SpendPolicyTag::Escrow2of3 as u8 => Ok(SpendPolicy::Escrow2of3 {
                buyer_pk_hash: read_fixed(input)?,
                merchant_pk_hash: read_fixed(input)?,
                operator_pk_hash: read_fixed(input)?,
                timeout_block: read_u64(input)?,
            }),
            tag => Err(DecodeError::InvalidSpendPolicyTag(tag)),
        }
    }
}

impl CanonicalDecode for AuxWitness {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            version: read_u8(input)?,
            amount: Amount14::decode(input)?,
            witness_seed: read_fixed(input)?,
            noise_class: read_u8(input)?,
            bundle_id: read_fixed(input)?,
        })
    }
}

impl CanonicalDecode for RecipientBoxPlaintext {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            version: read_u8(input)?,
            bundle_id: read_fixed(input)?,
            note_payload_commit: read_fixed(input)?,
            amount: Amount14::decode(input)?,
            witness_seed: read_fixed(input)?,
            nullifier_key: read_fixed(input)?,
            spend_policy_opening: read_bytes(input)?,
            aux_opening: read_bytes(input)?,
            sender_memo: read_option_bytes(input)?,
        })
    }
}

fn read_u8(input: &mut &[u8]) -> Result<u8, DecodeError> {
    let bytes = take(input, 1)?;
    Ok(bytes[0])
}

fn read_u16(input: &mut &[u8]) -> Result<u16, DecodeError> {
    let bytes = take(input, 2)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u64(input: &mut &[u8]) -> Result<u64, DecodeError> {
    let bytes = take(input, 8)?;
    Ok(u64::from_le_bytes(bytes.try_into().expect("u64 length")))
}

fn read_fixed<const N: usize>(input: &mut &[u8]) -> Result<[u8; N], DecodeError> {
    let bytes = take(input, N)?;
    let mut out = [0u8; N];
    out.copy_from_slice(bytes);
    Ok(out)
}

fn read_bytes(input: &mut &[u8]) -> Result<Vec<u8>, DecodeError> {
    let len = read_u32(input)? as usize;
    Ok(take(input, len)?.to_vec())
}

fn read_option_bytes(input: &mut &[u8]) -> Result<Option<Vec<u8>>, DecodeError> {
    match read_u8(input)? {
        0 => Ok(None),
        1 => Ok(Some(read_bytes(input)?)),
        _ => Err(DecodeError::UnexpectedEof),
    }
}

fn read_u32(input: &mut &[u8]) -> Result<u32, DecodeError> {
    let bytes = take(input, 4)?;
    Ok(u32::from_le_bytes(bytes.try_into().expect("u32 length")))
}

fn take<'a>(input: &mut &'a [u8], len: usize) -> Result<&'a [u8], DecodeError> {
    if input.len() < len {
        return Err(DecodeError::UnexpectedEof);
    }
    let (head, tail) = input.split_at(len);
    *input = tail;
    Ok(head)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::CanonicalEncode;

    #[test]
    fn spend_policy_roundtrips_from_canonical_bytes() {
        let policy = SpendPolicy::MarketplaceSettlement {
            buyer_pk_hash: [1; 32],
            seller_pk_hash: [2; 32],
            moderator_pk_hash: [3; 32],
            timeout_block: 77,
        };

        let decoded =
            SpendPolicy::from_canonical_bytes(&policy.to_canonical_bytes()).expect("decode");
        assert_eq!(decoded, policy);
    }

    #[test]
    fn aux_witness_roundtrips_from_canonical_bytes() {
        let witness = AuxWitness {
            version: 0,
            amount: Amount14::new(42).expect("amount"),
            witness_seed: [9; 32],
            noise_class: 3,
            bundle_id: [7; 16],
        };

        let decoded =
            AuxWitness::from_canonical_bytes(&witness.to_canonical_bytes()).expect("decode");
        assert_eq!(decoded, witness);
    }
}
