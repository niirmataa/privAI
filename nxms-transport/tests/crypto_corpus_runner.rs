#![cfg(feature = "crypto")]

use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use nxms_transport::crypto::{SealedPacket, decrypt, kem_decaps};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE_MAGIC: &[u8; 8] = b"NXMSFIX1";
const SENDER_ID: &str = "alice";
const TO_ID: &str = "bob";
const MSG_TYPE: &str = "tx_sign_req";
const ESCROW_ID: [u8; 16] = [0x5A; 16];
const SEQ: u64 = 41;

#[derive(Debug, Deserialize)]
struct FuzzSealedPacket {
    kem_ct_b64: String,
    nonce_b64: String,
    ciphertext_b64: String,
    tag_b64: String,
    sig_b64: String,
}

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
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

fn packet_from_file(path: &Path) -> Option<SealedPacket> {
    let data = fs::read(path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let packet: FuzzSealedPacket = serde_json::from_slice(&data).ok()?;
    Some(SealedPacket {
        kem_ct_b64: packet.kem_ct_b64,
        nonce_b64: packet.nonce_b64,
        ciphertext_b64: packet.ciphertext_b64,
        tag_b64: packet.tag_b64,
        sig_b64: packet.sig_b64,
    })
}

fn json_files_in(rel: &str) -> Vec<PathBuf> {
    let dir = repo_path(rel);
    let mut out: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("failed to read dir {}: {err}", dir.display()))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file())
        .collect();
    out.sort();
    out
}

fn load_seed_valid_kem_ct() -> Vec<u8> {
    let path = repo_path("fuzz/corpus/decrypt_fuzz/seed_valid.json");
    let data = fs::read(&path)
        .unwrap_or_else(|err| panic!("failed to read seed {}: {err}", path.display()));
    let packet: FuzzSealedPacket =
        serde_json::from_slice(&data).expect("seed_valid.json must decode");
    B64.decode(packet.kem_ct_b64)
        .expect("seed_valid kem_ct must be valid base64")
}

#[test]
fn decrypt_json_corpus_cases_outside_libfuzzer() {
    let (recipient_kem_sk, sender_sig_pk) = load_fixture_keys();
    let mut exercised = 0usize;

    for rel in ["fuzz/corpus/decrypt_fuzz", "fuzz/artifacts/decrypt_fuzz"] {
        for path in json_files_in(rel) {
            let Some(sealed) = packet_from_file(&path) else {
                continue;
            };
            exercised += 1;
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
    }

    assert!(exercised > 0, "expected at least one JSON corpus case");
}

#[test]
fn kem_decaps_seed_valid_outside_libfuzzer() {
    let (recipient_kem_sk, _) = load_fixture_keys();
    let kem_ct = load_seed_valid_kem_ct();
    let shared_secret = kem_decaps(recipient_kem_sk.as_slice(), kem_ct.as_slice())
        .expect("seed_valid kem_ct should decapsulate outside libFuzzer");

    assert!(
        !shared_secret.is_empty(),
        "decapsulation should return a non-empty shared secret"
    );
}
