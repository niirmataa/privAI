#![cfg(feature = "crypto")]

use nxms_transport::crypto::{
    FF_FALCON_TEST_SEED_LEN, falcon_keygen_seeded, falcon_sign_ct_prepared_seeded,
    falcon_sign_ct_seeded, falcon_verify,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FalconWrapperKatCase {
    name: String,
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

fn decode_seed(hex_value: &str) -> [u8; FF_FALCON_TEST_SEED_LEN] {
    let bytes = hex::decode(hex_value).expect("valid seed hex");
    bytes.try_into().expect("correct seed length")
}

#[test]
fn falcon_wrapper_seeded_kat_vectors_are_stable() {
    let cases: Vec<FalconWrapperKatCase> = serde_json::from_str(include_str!(
        "../../audyt/falcon_wrapper_kat_v1.json"
    ))
    .expect("valid kat json");

    for case in cases {
        let keygen_seed = decode_seed(&case.keygen_seed_hex);
        let sign_seed = decode_seed(&case.sign_seed_hex);
        let msg = hex::decode(&case.message_hex).expect("valid message hex");
        let expected_sk = hex::decode(&case.sk_hex).expect("valid sk hex");
        let expected_pk = hex::decode(&case.pk_hex).expect("valid pk hex");
        let expected_sig = hex::decode(&case.sig_hex).expect("valid sig hex");

        let (sk, pk) = falcon_keygen_seeded(&keygen_seed).expect("seeded keygen");
        assert_eq!(sk.len(), case.sk_len, "{}", case.name);
        assert_eq!(pk.len(), case.pk_len, "{}", case.name);
        assert_eq!(sk.as_slice(), expected_sk.as_slice(), "{}", case.name);
        assert_eq!(pk.as_slice(), expected_pk.as_slice(), "{}", case.name);

        let sig = falcon_sign_ct_seeded(&sign_seed, sk.as_slice(), &msg).expect("seeded sign");
        assert_eq!(sig.len(), case.sig_len, "{}", case.name);
        assert_eq!(sig.as_slice(), expected_sig.as_slice(), "{}", case.name);
        falcon_verify(&pk, &msg, &sig).expect("verify");
    }
}

#[test]
fn falcon_prepared_seeded_matches_reference_wrapper() {
    let keygen_seed = [0x11u8; FF_FALCON_TEST_SEED_LEN];
    let sign_seed = [0x22u8; FF_FALCON_TEST_SEED_LEN];
    let msg = hex::decode("00112233445566778899aabbccddeeff").expect("valid hex");

    let (sk, pk) = falcon_keygen_seeded(&keygen_seed).expect("seeded keygen");
    let sig_ref = falcon_sign_ct_seeded(&sign_seed, sk.as_slice(), &msg).expect("seeded sign");
    let sig_prepared =
        falcon_sign_ct_prepared_seeded(&sign_seed, sk.as_slice(), &msg).expect("prepared sign");

    assert_eq!(sig_prepared, sig_ref);
    falcon_verify(&pk, &msg, &sig_prepared).expect("verify prepared");
}
