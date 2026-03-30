pub const PRIVAI_V0: u8 = 0x00;
pub const DEFAULT_CHAIN_ID: u32 = 0x5052_4149; // "PRAI"

pub const LWE_MODULUS_Q: u32 = 4_294_967_291;
pub const PLAINTEXT_SPACE_P: u16 = 16_384;
pub const LWE_DIMENSION: usize = 1024;
pub const DELTA: u32 = 262_143;
pub const B_MAX: u32 = 131_071;

pub const FRODOKEM_640_SHAKE: u8 = 0x01;
pub const AEAD_ALG_XCHACHA20_POLY1305: u8 = 0x01;

pub const BUNDLE_FLAG_UPLOADED: u8 = 0x01;
pub const BUNDLE_FLAG_USED: u8 = 0x02;
pub const BUNDLE_FLAG_REVOKED: u8 = 0x04;
