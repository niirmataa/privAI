#![cfg(feature = "crypto")]

use nxms_transport::crypto::{
    Keys, PreparedTransportSigner, SealedPacket, XCHACHA20POLY1305_KEY_LEN, decrypt,
    decrypt_for_context, encrypt_for_context_with_signer, encrypt_with_signer, kem_decaps,
    kem_encaps, random_xchacha20poly1305_nonce, suite_kem_id, suite_sig_id,
    xchacha20poly1305_decrypt, xchacha20poly1305_encrypt,
};
use nxms_transport::wire::{
    ESCROW_APP_PROTO_V1, MsgType, NXMS_PROTO_V1, NxmsEnvelope, NxmsPayload, NxmsPayloadV2,
    msg_type_key,
};
use sha3::{
    Shake256,
    digest::{ExtendableOutput, Update, XofReader},
};

fn expand_shared_secret_to_xchacha_key(shared_secret: &[u8]) -> [u8; XCHACHA20POLY1305_KEY_LEN] {
    let mut hasher = Shake256::default();
    hasher.update(b"NXMS-XCHACHA-TEST-v1");
    hasher.update(shared_secret);
    let mut reader = hasher.finalize_xof();
    let mut key = [0u8; XCHACHA20POLY1305_KEY_LEN];
    reader.read(&mut key);
    key
}

#[test]
fn crypto_roundtrip_encrypt_decrypt_legacy_escrow_with_prepared_signer() {
    let sender = Keys::generate().expect("sender keys");
    let recipient = Keys::generate().expect("recipient keys");

    let sender_sig_sk = sender.sig_sk_zeroizing().expect("sender sig sk");
    let sender_sig_pk = sender.sig_pk().expect("sender sig pk");
    let recipient_kem_pk = recipient.kem_pk().expect("recipient kem pk");
    let recipient_kem_sk = recipient.kem_sk_zeroizing().expect("recipient kem sk");
    let signer = PreparedTransportSigner::new(sender_sig_sk.as_slice()).expect("prepared signer");

    let escrow_id = [7u8; 16];
    let escrow_id_hex = hex::encode(escrow_id);
    let seq: u64 = 1;

    let msg_type = MsgType::PrepareInfo;
    let payload = NxmsPayload {
        app_proto: ESCROW_APP_PROTO_V1.to_string(),
        msg_type: msg_type.clone(),
        escrow_id_hex: escrow_id_hex.clone(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        seq,
        data: "hello".to_string(),
    };
    let plaintext = serde_json::to_vec(&payload).expect("payload json");

    let sealed = encrypt_with_signer(
        "alice",
        "bob",
        msg_type_key(&msg_type),
        &escrow_id,
        seq,
        &recipient_kem_pk,
        &signer,
        &plaintext,
    )
    .expect("encrypt");

    let env = NxmsEnvelope {
        proto: NXMS_PROTO_V1.to_string(),
        kem_id: suite_kem_id().to_string(),
        sig_id: suite_sig_id().to_string(),
        msg_type: msg_type.clone(),
        escrow_id_hex: escrow_id_hex.clone(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        seq,
        kem_ct_b64: sealed.kem_ct_b64.clone(),
        nonce_b64: sealed.nonce_b64.clone(),
        ciphertext_b64: sealed.ciphertext_b64.clone(),
        tag_b64: sealed.tag_b64.clone(),
        sig_b64: sealed.sig_b64.clone(),
    };

    let sealed2 = SealedPacket {
        kem_ct_b64: env.kem_ct_b64,
        nonce_b64: env.nonce_b64,
        ciphertext_b64: env.ciphertext_b64,
        tag_b64: env.tag_b64,
        sig_b64: env.sig_b64,
    };

    let out = decrypt(
        "alice",
        "bob",
        msg_type_key(&msg_type),
        &escrow_id,
        seq,
        &sealed2,
        recipient_kem_sk.as_slice(),
        &sender_sig_pk,
    )
    .expect("decrypt");
    let payload2: NxmsPayload = serde_json::from_slice(&out).expect("payload2 json");

    assert_eq!(payload2.data, "hello");
    assert_eq!(payload2.seq, seq);
    assert_eq!(payload2.escrow_id_hex, escrow_id_hex);
    assert_eq!(payload2.from, "alice");
    assert_eq!(payload2.to, "bob");
}

#[test]
fn crypto_roundtrip_encrypt_decrypt_v2_context_with_prepared_signer() {
    let sender = Keys::generate().expect("sender keys");
    let recipient = Keys::generate().expect("recipient keys");

    let sender_sig_sk = sender.sig_sk_zeroizing().expect("sender sig sk");
    let sender_sig_pk = sender.sig_pk().expect("sender sig pk");
    let recipient_kem_pk = recipient.kem_pk().expect("recipient kem pk");
    let recipient_kem_sk = recipient.kem_sk_zeroizing().expect("recipient kem sk");
    let signer = PreparedTransportSigner::new(sender_sig_sk.as_slice()).expect("prepared signer");

    let context_id = [13u8; 16];
    let context_id_hex = hex::encode(context_id);
    let seq: u64 = 5;
    let msg_type = "market_offer";

    let payload = NxmsPayloadV2 {
        app_proto: "PRIVAI/1".to_string(),
        msg_type: msg_type.to_string(),
        context_id_hex: context_id_hex.clone(),
        from: "alice".to_string(),
        to: "bob".to_string(),
        seq,
        data: "{\"offer\":\"prepared-signer\"}".to_string(),
    };
    let plaintext = serde_json::to_vec(&payload).expect("payload json");

    let sealed = encrypt_for_context_with_signer(
        "alice",
        "bob",
        msg_type,
        &context_id,
        seq,
        &recipient_kem_pk,
        &signer,
        &plaintext,
    )
    .expect("encrypt prepared signer");

    let out = decrypt_for_context(
        "alice",
        "bob",
        msg_type,
        &context_id,
        seq,
        &sealed,
        recipient_kem_sk.as_slice(),
        &sender_sig_pk,
    )
    .expect("decrypt");
    let payload2: NxmsPayloadV2 = serde_json::from_slice(&out).expect("payload2 json");

    assert_eq!(payload2.app_proto, "PRIVAI/1");
    assert_eq!(payload2.msg_type, msg_type);
    assert_eq!(payload2.context_id_hex, context_id_hex);
    assert_eq!(payload2.seq, seq);
    assert_eq!(payload2.data, "{\"offer\":\"prepared-signer\"}");
}

#[test]
fn crypto_roundtrip_generic_kem_and_aead() {
    let recipient = Keys::generate().expect("recipient keys");
    let recipient_kem_pk = recipient.kem_pk().expect("recipient kem pk");
    let recipient_kem_sk = recipient.kem_sk_zeroizing().expect("recipient kem sk");

    let (kem_ct, shared_sender) = kem_encaps(&recipient_kem_pk).expect("encaps");
    let shared_recipient = kem_decaps(recipient_kem_sk.as_slice(), &kem_ct).expect("decaps");

    assert_eq!(shared_sender.as_slice(), shared_recipient.as_slice());

    let key = expand_shared_secret_to_xchacha_key(shared_sender.as_slice());

    let nonce = random_xchacha20poly1305_nonce();
    let aad = b"privai-box-test";
    let plaintext = b"local model settlement";
    let (ciphertext, tag) =
        xchacha20poly1305_encrypt(&key, &nonce, plaintext, aad).expect("encrypt");
    let decrypted =
        xchacha20poly1305_decrypt(&key, &nonce, &ciphertext, &tag, aad).expect("decrypt");

    assert_eq!(decrypted, plaintext);
}
