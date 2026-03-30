#![cfg(feature = "crypto")]

use anyhow::Result;
use nxms_transport::crypto::{
    FF_FALCON_TEST_SEED_LEN, falcon_keygen_seeded, falcon_sign_ct_seeded,
};
use serde::Serialize;

#[derive(Serialize)]
struct FalconWrapperKatCase {
    name: &'static str,
    keygen_seed_hex: String,
    sign_seed_hex: String,
    message_hex: String,
    sk_len: usize,
    pk_len: usize,
    sig_len: usize,
    sk_hex: String,
    pk_hex: String,
    sig_hex: String,
}

fn seed(byte: u8) -> [u8; FF_FALCON_TEST_SEED_LEN] {
    [byte; FF_FALCON_TEST_SEED_LEN]
}

fn hex(data: &[u8]) -> String {
    hex::encode(data)
}

fn main() -> Result<()> {
    let cases = [
        (
            "empty_message",
            seed(0x11),
            seed(0x22),
            Vec::new(),
        ),
        (
            "short_ascii_message",
            seed(0x33),
            seed(0x44),
            b"privai falcon wrapper kat".to_vec(),
        ),
        (
            "long_binary_message",
            seed(0x55),
            seed(0x66),
            (0u16..512).map(|n| (n as u8).wrapping_mul(37)).collect::<Vec<u8>>(),
        ),
    ];

    let mut out = Vec::with_capacity(cases.len());
    for (name, keygen_seed, sign_seed, msg) in cases {
        let (sk, pk) = falcon_keygen_seeded(&keygen_seed)?;
        let sig = falcon_sign_ct_seeded(&sign_seed, sk.as_slice(), &msg)?;
        out.push(FalconWrapperKatCase {
            name,
            keygen_seed_hex: hex(&keygen_seed),
            sign_seed_hex: hex(&sign_seed),
            message_hex: hex(&msg),
            sk_len: sk.len(),
            pk_len: pk.len(),
            sig_len: sig.len(),
            sk_hex: hex(sk.as_slice()),
            pk_hex: hex(&pk),
            sig_hex: hex(&sig),
        });
    }

    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
