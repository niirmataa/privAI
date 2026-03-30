#![no_main]

use libfuzzer_sys::fuzz_target;
use nxms_transport::crypto::{SealedPacket, decrypt};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Deserialize)]
struct FuzzSealedPacket {
    kem_ct_b64: String,
    nonce_b64: String,
    ciphertext_b64: String,
    tag_b64: String,
    sig_b64: String,
}

const FIXTURE_MAGIC: &[u8; 8] = b"NXMSFIX1";
const FIXTURE_PATH_ENV: &str = "NXMS_FUZZ_FIXTURE";
const DEFAULT_FIXTURE_PATH: &str = "fuzz_c/corpus/fixture.bin";
const SENDER_ID: &str = "alice";
const TO_ID: &str = "bob";
const MSG_TYPE: &str = "tx_sign_req";
const ESCROW_ID: [u8; 16] = [0x5A; 16];
const SEQ: u64 = 41;

fn fixture_keys() -> &'static (Vec<u8>, Vec<u8>) {
    static FIXTURE: OnceLock<(Vec<u8>, Vec<u8>)> = OnceLock::new();
    FIXTURE.get_or_init(load_fixture_keys)
}

fn fixture_path() -> PathBuf {
    std::env::var_os(FIXTURE_PATH_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_FIXTURE_PATH))
}

fn parse_u32_le(data: &[u8], offset: &mut usize) -> usize {
    let end = *offset + 4;
    let bytes = data
        .get(*offset..end)
        .expect("fixture must contain a u32 length");
    *offset = end;
    u32::from_le_bytes(bytes.try_into().expect("u32 length bytes")) as usize
}

fn load_fixture_keys() -> (Vec<u8>, Vec<u8>) {
    let path = fixture_path();
    let data = fs::read(&path).unwrap_or_else(|err| {
        panic!("failed to read decrypt fuzz fixture {}: {err}", path.display())
    });
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
    let recipient_kem_sk = data
        .get(offset..kem_end)
        .expect("fixture missing recipient kem sk")
        .to_vec();
    offset = kem_end;

    let sender_sig_pk_len = parse_u32_le(&data, &mut offset);
    let sig_end = offset + sender_sig_pk_len;
    let sender_sig_pk = data
        .get(offset..sig_end)
        .expect("fixture missing sender sig pk")
        .to_vec();
    assert_eq!(
        sig_end,
        data.len(),
        "trailing bytes in fixture {}",
        path.display()
    );

    (recipient_kem_sk, sender_sig_pk)
}

fn to_sealed_packet(packet: FuzzSealedPacket) -> SealedPacket {
    SealedPacket {
        kem_ct_b64: packet.kem_ct_b64,
        nonce_b64: packet.nonce_b64,
        ciphertext_b64: packet.ciphertext_b64,
        tag_b64: packet.tag_b64,
        sig_b64: packet.sig_b64,
    }
}

fuzz_target!(|data: &[u8]| {
    if let Ok(packet) = serde_json::from_slice::<FuzzSealedPacket>(data) {
        let sealed = to_sealed_packet(packet);
        let (recipient_kem_sk, sender_sig_pk) = fixture_keys();
        let _ = decrypt(
            SENDER_ID,
            TO_ID,
            MSG_TYPE,
            &ESCROW_ID,
            SEQ,
            &sealed,
            recipient_kem_sk.as_slice(),
            sender_sig_pk.as_slice(),
        );
    }
});
