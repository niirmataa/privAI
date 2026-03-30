#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmountCipherParams {
    pub lwe_dimension: usize,
    pub packed_u32_limbs_per_field: usize,
}

impl Default for AmountCipherParams {
    fn default() -> Self {
        Self {
            lwe_dimension: 512,
            packed_u32_limbs_per_field: 7,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Halo2PrivaiParams {
    pub amount_cipher: AmountCipherParams,
}
