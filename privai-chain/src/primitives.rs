use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::canonical::{CanonicalEncode, write_fixed, write_u16, write_u32};
use crate::params::{LWE_DIMENSION, PLAINTEXT_SPACE_P};

pub type Hash32 = [u8; 32];
pub type BundleId = [u8; 16];
pub type ContextId = [u8; 16];
pub type BlockHeight = u64;
pub type Flags8 = u8;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum AmountError {
    #[error("amount {0} exceeds the v0 plaintext space")]
    OutOfRange(u16),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LweCiphertextError {
    #[error("expected {expected} coefficients, got {actual}")]
    InvalidDimension { expected: usize, actual: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Amount14(u16);

impl Amount14 {
    pub fn new(value: u16) -> Result<Self, AmountError> {
        if value < PLAINTEXT_SPACE_P {
            Ok(Self(value))
        } else {
            Err(AmountError::OutOfRange(value))
        }
    }

    pub fn value(self) -> u16 {
        self.0
    }
}

impl CanonicalEncode for Amount14 {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u16(out, self.0);
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct Nullifier(pub Hash32);

impl CanonicalEncode for Nullifier {
    fn encode(&self, out: &mut Vec<u8>) {
        write_fixed(out, &self.0);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LweCiphertext {
    pub a: Vec<u32>,
    pub b: u32,
}

impl Default for LweCiphertext {
    fn default() -> Self {
        Self {
            a: vec![0; LWE_DIMENSION],
            b: 0,
        }
    }
}

impl LweCiphertext {
    pub fn new(a: Vec<u32>, b: u32) -> Result<Self, LweCiphertextError> {
        if a.len() != LWE_DIMENSION {
            return Err(LweCiphertextError::InvalidDimension {
                expected: LWE_DIMENSION,
                actual: a.len(),
            });
        }

        Ok(Self { a, b })
    }

    pub fn zero() -> Self {
        Self::default()
    }
}

impl CanonicalEncode for LweCiphertext {
    fn encode(&self, out: &mut Vec<u8>) {
        debug_assert_eq!(self.a.len(), LWE_DIMENSION);
        for coeff in &self.a {
            write_u32(out, *coeff);
        }
        write_u32(out, self.b);
    }
}
