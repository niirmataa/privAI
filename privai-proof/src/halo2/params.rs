use privai_chain::params::{
    DELTA as CHAIN_DELTA,
    LWE_DIMENSION as CHAIN_LWE_DIMENSION,
    LWE_MODULUS_Q as CHAIN_LWE_MODULUS_Q,
    PLAINTEXT_SPACE_P as CHAIN_PLAINTEXT_SPACE_P,
};

pub const LWE_DIMENSION_V0: usize = CHAIN_LWE_DIMENSION;
pub const LWE_MODULUS_Q_V0: u64 = CHAIN_LWE_MODULUS_Q as u64;
pub const PLAINTEXT_SPACE_P_V0: u16 = CHAIN_PLAINTEXT_SPACE_P;
pub const DELTA_V0: u32 = CHAIN_DELTA;
pub const MAX_AMOUNT_V0: u16 = PLAINTEXT_SPACE_P_V0 - 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmountCipherParams {
    pub lwe_dimension: usize,
    pub packed_u32_limbs_per_field: usize,
    pub lwe_modulus_q: u64,
    pub plaintext_space_p: u16,
    pub delta: u32,
    pub max_amount: u16,
}

impl Default for AmountCipherParams {
    fn default() -> Self {
        Self {
            lwe_dimension: LWE_DIMENSION_V0,
            packed_u32_limbs_per_field: 7,
            lwe_modulus_q: LWE_MODULUS_Q_V0,
            plaintext_space_p: PLAINTEXT_SPACE_P_V0,
            delta: DELTA_V0,
            max_amount: MAX_AMOUNT_V0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Halo2PrivaiParams {
    pub amount_cipher: AmountCipherParams,
}
