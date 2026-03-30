#![cfg(feature = "crypto")]

use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use nxms_transport::crypto::{
    Keys, SealedPacket, decrypt, decrypt_for_context, encrypt, encrypt_for_context,
};
use nxms_transport::wire::{MsgType, msg_type_key};

fn setup_packet() -> (Vec<u8>, Vec<u8>, [u8; 16], u64, SealedPacket, MsgType) {
    let sender = Keys::generate().expect("sender keys");
    let recipient = Keys::generate().expect("recipient keys");

    let sender_sig_sk = sender.sig_sk_zeroizing().expect("sender sig sk");
    let sender_sig_pk = sender.sig_pk().expect("sender sig pk");
    let recipient_kem_pk = recipient.kem_pk().expect("recipient kem pk");
    let recipient_kem_sk = recipient.kem_sk_zeroizing().expect("recipient kem sk");

    let escrow_id = [9u8; 16];
    let seq: u64 = 11;
    let msg_type = MsgType::TxSignReq;
    let plaintext = br#"{"kind":"tx_sign_req","data":"abcd"}"#.to_vec();

    let sealed = encrypt(
        "alice",
        "bob",
        msg_type_key(&msg_type),
        &escrow_id,
        seq,
        &recipient_kem_pk,
        sender_sig_sk.as_slice(),
        &plaintext,
    )
    .expect("encrypt");

    (
        recipient_kem_sk.as_slice().to_vec(),
        sender_sig_pk,
        escrow_id,
        seq,
        sealed,
        msg_type,
    )
}

fn setup_context_packet() -> (Vec<u8>, Vec<u8>, [u8; 16], u64, SealedPacket, &'static str) {
    let sender = Keys::generate().expect("sender keys");
    let recipient = Keys::generate().expect("recipient keys");

    let sender_sig_sk = sender.sig_sk_zeroizing().expect("sender sig sk");
    let sender_sig_pk = sender.sig_pk().expect("sender sig pk");
    let recipient_kem_pk = recipient.kem_pk().expect("recipient kem pk");
    let recipient_kem_sk = recipient.kem_sk_zeroizing().expect("recipient kem sk");

    let context_id = [0x33u8; 16];
    let seq: u64 = 17;
    let msg_type = "market_offer";
    let plaintext = br#"{"kind":"market_offer","data":"vector"}"#.to_vec();

    let sealed = encrypt_for_context(
        "alice",
        "bob",
        msg_type,
        &context_id,
        seq,
        &recipient_kem_pk,
        sender_sig_sk.as_slice(),
        &plaintext,
    )
    .expect("encrypt");

    (
        recipient_kem_sk.as_slice().to_vec(),
        sender_sig_pk,
        context_id,
        seq,
        sealed,
        msg_type,
    )
}

#[test]
fn decrypt_rejects_tampered_tag() {
    let (recipient_kem_sk, sender_sig_pk, escrow_id, seq, mut sealed, msg_type) = setup_packet();
    let mut tag = B64.decode(sealed.tag_b64.as_bytes()).expect("decode tag");
    tag[0] ^= 0x01;
    sealed.tag_b64 = B64.encode(tag);

    let err = decrypt(
        "alice",
        "bob",
        msg_type_key(&msg_type),
        &escrow_id,
        seq,
        &sealed,
        &recipient_kem_sk,
        &sender_sig_pk,
    )
    .expect_err("tampered tag must fail");
    assert!(err.to_string().contains("failed"));
}

#[test]
fn decrypt_rejects_tampered_nonce() {
    let (recipient_kem_sk, sender_sig_pk, escrow_id, seq, mut sealed, msg_type) = setup_packet();
    let mut nonce = B64
        .decode(sealed.nonce_b64.as_bytes())
        .expect("decode nonce");
    nonce[0] ^= 0x10;
    sealed.nonce_b64 = B64.encode(nonce);

    decrypt(
        "alice",
        "bob",
        msg_type_key(&msg_type),
        &escrow_id,
        seq,
        &sealed,
        &recipient_kem_sk,
        &sender_sig_pk,
    )
    .expect_err("tampered nonce must fail");
}

#[test]
fn decrypt_rejects_tampered_signature() {
    let (recipient_kem_sk, sender_sig_pk, escrow_id, seq, mut sealed, msg_type) = setup_packet();
    let mut sig = B64.decode(sealed.sig_b64.as_bytes()).expect("decode sig");
    sig[0] ^= 0x02;
    sealed.sig_b64 = B64.encode(sig);

    decrypt(
        "alice",
        "bob",
        msg_type_key(&msg_type),
        &escrow_id,
        seq,
        &sealed,
        &recipient_kem_sk,
        &sender_sig_pk,
    )
    .expect_err("tampered signature must fail");
}

#[test]
fn decrypt_rejects_tampered_ciphertext() {
    let (recipient_kem_sk, sender_sig_pk, escrow_id, seq, mut sealed, msg_type) = setup_packet();
    let mut ct = B64
        .decode(sealed.ciphertext_b64.as_bytes())
        .expect("decode ciphertext");
    ct[0] ^= 0x04;
    sealed.ciphertext_b64 = B64.encode(ct);

    decrypt(
        "alice",
        "bob",
        msg_type_key(&msg_type),
        &escrow_id,
        seq,
        &sealed,
        &recipient_kem_sk,
        &sender_sig_pk,
    )
    .expect_err("tampered ciphertext must fail");
}

#[test]
fn decrypt_rejects_tampered_kem_ciphertext() {
    let (recipient_kem_sk, sender_sig_pk, escrow_id, seq, mut sealed, msg_type) = setup_packet();
    let mut kem_ct = B64
        .decode(sealed.kem_ct_b64.as_bytes())
        .expect("decode kem ct");
    kem_ct[0] ^= 0x08;
    sealed.kem_ct_b64 = B64.encode(kem_ct);

    decrypt(
        "alice",
        "bob",
        msg_type_key(&msg_type),
        &escrow_id,
        seq,
        &sealed,
        &recipient_kem_sk,
        &sender_sig_pk,
    )
    .expect_err("tampered kem ciphertext must fail");
}

#[test]
fn decrypt_rejects_wrong_sender_key() {
    let (recipient_kem_sk, _sender_sig_pk, escrow_id, seq, sealed, msg_type) = setup_packet();
    let wrong_sender = Keys::generate().expect("wrong sender keys");
    let wrong_sender_pk = wrong_sender.sig_pk().expect("wrong sender pk");

    decrypt(
        "alice",
        "bob",
        msg_type_key(&msg_type),
        &escrow_id,
        seq,
        &sealed,
        &recipient_kem_sk,
        &wrong_sender_pk,
    )
    .expect_err("wrong sender key must fail");
}

#[test]
fn decrypt_rejects_wrong_sender_id() {
    let (recipient_kem_sk, sender_sig_pk, escrow_id, seq, sealed, msg_type) = setup_packet();

    decrypt(
        "mallory",
        "bob",
        msg_type_key(&msg_type),
        &escrow_id,
        seq,
        &sealed,
        &recipient_kem_sk,
        &sender_sig_pk,
    )
    .expect_err("wrong sender id must fail");
}

#[test]
fn decrypt_rejects_wrong_msg_type() {
    let (recipient_kem_sk, sender_sig_pk, escrow_id, seq, sealed, _msg_type) = setup_packet();

    decrypt(
        "alice",
        "bob",
        msg_type_key(&MsgType::TxSignResp),
        &escrow_id,
        seq,
        &sealed,
        &recipient_kem_sk,
        &sender_sig_pk,
    )
    .expect_err("wrong msg type must fail");
}

#[test]
fn decrypt_rejects_wrong_recipient_id() {
    let (recipient_kem_sk, sender_sig_pk, escrow_id, seq, sealed, msg_type) = setup_packet();

    decrypt(
        "alice",
        "carol",
        msg_type_key(&msg_type),
        &escrow_id,
        seq,
        &sealed,
        &recipient_kem_sk,
        &sender_sig_pk,
    )
    .expect_err("wrong recipient id must fail");
}

#[test]
fn decrypt_rejects_wrong_seq() {
    let (recipient_kem_sk, sender_sig_pk, escrow_id, seq, sealed, msg_type) = setup_packet();

    decrypt(
        "alice",
        "bob",
        msg_type_key(&msg_type),
        &escrow_id,
        seq + 1,
        &sealed,
        &recipient_kem_sk,
        &sender_sig_pk,
    )
    .expect_err("wrong seq must fail");
}

#[test]
fn decrypt_for_context_rejects_wrong_context_id() {
    let (recipient_kem_sk, sender_sig_pk, context_id, seq, sealed, msg_type) =
        setup_context_packet();
    let wrong_context_id = [0x44u8; 16];
    assert_ne!(wrong_context_id, context_id);

    decrypt_for_context(
        "alice",
        "bob",
        msg_type,
        &wrong_context_id,
        seq,
        &sealed,
        &recipient_kem_sk,
        &sender_sig_pk,
    )
    .expect_err("wrong context id must fail");
}

#[test]
fn decrypt_for_context_rejects_wrong_recipient_id() {
    let (recipient_kem_sk, sender_sig_pk, context_id, seq, sealed, msg_type) =
        setup_context_packet();

    decrypt_for_context(
        "alice",
        "carol",
        msg_type,
        &context_id,
        seq,
        &sealed,
        &recipient_kem_sk,
        &sender_sig_pk,
    )
    .expect_err("wrong recipient id in context packet must fail");
}
