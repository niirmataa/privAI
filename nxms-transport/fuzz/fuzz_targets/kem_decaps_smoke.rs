#![no_main]

use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use libfuzzer_sys::fuzz_target;
use nxms_transport::crypto::kem_decaps;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE_MAGIC: &[u8; 8] = b"NXMSFIX1";

#[derive(Deserialize)]
struct FuzzSealedPacket {
    kem_ct_b64: String,
    nonce_b64: String,
    ciphertext_b64: String,
    tag_b64: String,
    sig_b64: String,
}

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel)
}

fn parse_u32_le(data: &[u8], offset: &mut usize) -> usize {
    let end = *offset + 4;
    let bytes = data
        .get(*offset..end)
        .expect("fixture must contain a u32 length");
    *offset = end;
    u32::from_le_bytes(bytes.try_into().expect("u32 length bytes")) as usize
}

fn load_fixture_recipient_kem_sk() -> Vec<u8> {
    let path = repo_path("fuzz_c/corpus/fixture.bin");
    let data = fs::read(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.display()));
    assert!(
        data.len() >= FIXTURE_MAGIC.len() + 8,
        "fixture too short: {}",
        path.display()
    );
    assert_eq!(
        &data[..FIXTURE_MAGIC.len()],
        FIXTURE_MAGIC,
        "invalid fixture magic in {}",
        path.display()
    );

    let mut offset = FIXTURE_MAGIC.len();
    let recipient_kem_sk_len = parse_u32_le(&data, &mut offset);
    let kem_end = offset + recipient_kem_sk_len;
    data.get(offset..kem_end)
        .expect("fixture missing recipient kem sk")
        .to_vec()
}

fn load_seed_kem_ct() -> Vec<u8> {
    let path = repo_path("fuzz/corpus/decrypt_fuzz/seed_valid.json");
    let data = fs::read(&path)
        .unwrap_or_else(|err| panic!("failed to read seed {}: {err}", path.display()));
    let packet: FuzzSealedPacket =
        serde_json::from_slice(&data).expect("seed_valid.json must decode");
    let _ = (
        packet.nonce_b64.as_str(),
        packet.ciphertext_b64.as_str(),
        packet.tag_b64.as_str(),
        packet.sig_b64.as_str(),
    );
    B64.decode(packet.kem_ct_b64).expect("valid kem_ct_b64")
}

fuzz_target!(|_data: &[u8]| {
    let recipient_kem_sk = load_fixture_recipient_kem_sk();
    let kem_ct = load_seed_kem_ct();
    let _ = kem_decaps(recipient_kem_sk.as_slice(), kem_ct.as_slice());
});
