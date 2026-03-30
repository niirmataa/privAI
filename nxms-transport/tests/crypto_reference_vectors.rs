#![cfg(feature = "crypto")]

use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use nxms_transport::crypto::{
    Keys, PreparedTransportSigner, decrypt, encrypt_with_signer, falcon_verify, kem_decaps,
};
use sha3::{
    Shake256,
    digest::{ExtendableOutput, Update, XofReader},
};

const AAD_PREFIX: &[u8] = b"NXMS-AAD-v1";
const SIG_PREFIX: &[u8] = b"NXMS-SIG-v1";
const CTHASH_PREFIX: &[u8] = b"NXMS-CTHASH-v1";
const KDF_PREFIX: &[u8] = b"NXMS-KDF-v1";
const TAG_PREFIX: &[u8] = b"NXMS-TAG-v1";
const STREAM_PREFIX: &[u8] = b"NXMS-STREAM-v1";
const NXMS_KEM_ID: &[u8] = b"FrodoKEM-640-SHAKE";
const NXMS_SIG_ID: &[u8] = b"Falcon-1024-CT";
const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 32;

fn u32be(value: u32) -> [u8; 4] {
    value.to_be_bytes()
}

fn u64be(value: u64) -> [u8; 8] {
    value.to_be_bytes()
}

fn shake256(parts: &[&[u8]], out_len: usize) -> Vec<u8> {
    let mut hasher = Shake256::default();
    for part in parts {
        hasher.update(part);
    }
    let mut reader = hasher.finalize_xof();
    let mut out = vec![0u8; out_len];
    reader.read(&mut out);
    out
}

fn derive_keys_ref(ss: &[u8], escrow_id: &[u8; 16]) -> ([u8; 32], [u8; 32]) {
    let ss_len = u32be(ss.len() as u32);
    let ke = shake256(&[KDF_PREFIX, &ss_len, ss, escrow_id, b"ms-ke"], 32);
    let km = shake256(&[KDF_PREFIX, &ss_len, ss, escrow_id, b"ms-km"], 32);
    (ke.try_into().expect("ke"), km.try_into().expect("km"))
}

fn ct_hash_ref(kem_ct: &[u8]) -> [u8; 32] {
    shake256(&[CTHASH_PREFIX, kem_ct], 32)
        .try_into()
        .expect("ct hash")
}

fn build_aad_ref(
    sender_id: &str,
    to_id: &str,
    msg_type: &str,
    escrow_id: &[u8; 16],
    seq: u64,
    kem_ct: &[u8],
) -> Vec<u8> {
    let sender = sender_id.as_bytes();
    let to = to_id.as_bytes();
    let msg = msg_type.as_bytes();
    let ct_hash = ct_hash_ref(kem_ct);

    let mut out = Vec::with_capacity(
        AAD_PREFIX.len()
            + 4
            + sender.len()
            + 4
            + to.len()
            + 4
            + NXMS_KEM_ID.len()
            + 4
            + NXMS_SIG_ID.len()
            + 4
            + msg.len()
            + escrow_id.len()
            + 8
            + ct_hash.len(),
    );
    out.extend_from_slice(AAD_PREFIX);
    out.extend_from_slice(&u32be(sender.len() as u32));
    out.extend_from_slice(sender);
    out.extend_from_slice(&u32be(to.len() as u32));
    out.extend_from_slice(to);
    out.extend_from_slice(&u32be(NXMS_KEM_ID.len() as u32));
    out.extend_from_slice(NXMS_KEM_ID);
    out.extend_from_slice(&u32be(NXMS_SIG_ID.len() as u32));
    out.extend_from_slice(NXMS_SIG_ID);
    out.extend_from_slice(&u32be(msg.len() as u32));
    out.extend_from_slice(msg);
    out.extend_from_slice(escrow_id);
    out.extend_from_slice(&u64be(seq));
    out.extend_from_slice(&ct_hash);
    out
}

fn xor_shake_keystream_ref(buf: &mut [u8], ke: &[u8; 32], nonce: &[u8]) {
    let stream = shake256(&[STREAM_PREFIX, ke, nonce], buf.len());
    for (dst, src) in buf.iter_mut().zip(stream.iter()) {
        *dst ^= *src;
    }
}

fn compute_tag_ref(km: &[u8; 32], aad: &[u8], nonce: &[u8], ciphertext: &[u8]) -> [u8; 32] {
    let aad_len = u32be(aad.len() as u32);
    let nonce_len = u32be(nonce.len() as u32);
    let ct_len = u32be(ciphertext.len() as u32);
    shake256(
        &[
            TAG_PREFIX, km, &aad_len, aad, &nonce_len, nonce, &ct_len, ciphertext,
        ],
        TAG_LEN,
    )
    .try_into()
    .expect("tag")
}

fn build_sig_message_ref(aad: &[u8], nonce: &[u8], ciphertext: &[u8], tag: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(
        SIG_PREFIX.len() + 4 + aad.len() + 4 + nonce.len() + 4 + ciphertext.len() + 4 + tag.len(),
    );
    out.extend_from_slice(SIG_PREFIX);
    out.extend_from_slice(&u32be(aad.len() as u32));
    out.extend_from_slice(aad);
    out.extend_from_slice(&u32be(nonce.len() as u32));
    out.extend_from_slice(nonce);
    out.extend_from_slice(&u32be(ciphertext.len() as u32));
    out.extend_from_slice(ciphertext);
    out.extend_from_slice(&u32be(tag.len() as u32));
    out.extend_from_slice(tag);
    out
}

fn decode_hex(value: &str) -> Vec<u8> {
    hex::decode(value).expect("valid hex")
}

#[test]
fn reference_vectors_are_stable() {
    let sender_id = "alice";
    let to_id = "bob";
    let msg_type = "tx_sign_req";
    let seq = 7u64;
    let escrow_id: [u8; 16] = decode_hex("101112131415161718191a1b1c1d1e1f")
        .try_into()
        .expect("escrow");
    let ss = decode_hex("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
    let kem_ct = decode_hex(
        "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f",
    );
    let nonce = decode_hex("a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7");
    let plaintext = b"reference vector plaintext".to_vec();

    let (ke, km) = derive_keys_ref(&ss, &escrow_id);
    assert_eq!(
        hex::encode(ke),
        "592637f9f388d4b42cc65a4756147d851007b857cd6fe6072c1d1528d8a79d26"
    );
    assert_eq!(
        hex::encode(km),
        "ff6646ae73b3795fbe9b3507f46cc4e506298f07fdf8da175773b9ac8116bd7e"
    );

    let ct_hash = ct_hash_ref(&kem_ct);
    assert_eq!(
        hex::encode(ct_hash),
        "51b79e3360e27edb50416229a3eaed3347cb922abe8cb48bcc37702fb9e1e717"
    );

    let aad = build_aad_ref(sender_id, to_id, msg_type, &escrow_id, seq, &kem_ct);
    assert_eq!(
        hex::encode(&aad),
        "4e584d532d4141442d763100000005616c69636500000003626f620000001246726f646f4b454d2d3634302d5348414b450000000e46616c636f6e2d313032342d43540000000b74785f7369676e5f726571101112131415161718191a1b1c1d1e1f000000000000000751b79e3360e27edb50416229a3eaed3347cb922abe8cb48bcc37702fb9e1e717"
    );

    let mut ciphertext = plaintext.clone();
    xor_shake_keystream_ref(&mut ciphertext, &ke, &nonce);
    assert_eq!(
        hex::encode(&ciphertext),
        "835c72f63560017a96f0fb0bfbb6bcd81ad7ae555c24e6733d8b"
    );

    let tag = compute_tag_ref(&km, &aad, &nonce, &ciphertext);
    assert_eq!(
        hex::encode(tag),
        "d9eecb1613d596387ee5b9dfee3a9d69eaff61f3366972c1f8db8f1f30ee788e"
    );

    let sig_msg = build_sig_message_ref(&aad, &nonce, &ciphertext, &tag);
    assert_eq!(
        hex::encode(sig_msg),
        "4e584d532d5349472d76310000008a4e584d532d4141442d763100000005616c69636500000003626f620000001246726f646f4b454d2d3634302d5348414b450000000e46616c636f6e2d313032342d43540000000b74785f7369676e5f726571101112131415161718191a1b1c1d1e1f000000000000000751b79e3360e27edb50416229a3eaed3347cb922abe8cb48bcc37702fb9e1e71700000018a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b70000001a835c72f63560017a96f0fb0bfbb6bcd81ad7ae555c24e6733d8b00000020d9eecb1613d596387ee5b9dfee3a9d69eaff61f3366972c1f8db8f1f30ee788e"
    );
}

#[test]
fn c_transport_packet_matches_reference_reconstruction() {
    let sender = Keys::generate().expect("sender keys");
    let recipient = Keys::generate().expect("recipient keys");

    let sender_sig_sk = sender.sig_sk_zeroizing().expect("sender sig sk");
    let sender_sig_pk = sender.sig_pk().expect("sender sig pk");
    let recipient_kem_pk = recipient.kem_pk().expect("recipient kem pk");
    let recipient_kem_sk = recipient.kem_sk_zeroizing().expect("recipient kem sk");
    let signer = PreparedTransportSigner::new(sender_sig_sk.as_slice()).expect("prepared signer");

    let escrow_id = [0x55u8; 16];
    let seq = 23u64;
    let plaintext = b"cross-check against reference".to_vec();

    let sealed = encrypt_with_signer(
        "alice",
        "bob",
        "tx_sign_req",
        &escrow_id,
        seq,
        &recipient_kem_pk,
        &signer,
        &plaintext,
    )
    .expect("encrypt");

    let kem_ct = B64.decode(sealed.kem_ct_b64.as_bytes()).expect("kem ct");
    let nonce = B64.decode(sealed.nonce_b64.as_bytes()).expect("nonce");
    let ciphertext = B64
        .decode(sealed.ciphertext_b64.as_bytes())
        .expect("ciphertext");
    let tag = B64.decode(sealed.tag_b64.as_bytes()).expect("tag");
    let sig = B64.decode(sealed.sig_b64.as_bytes()).expect("sig");

    assert_eq!(nonce.len(), NONCE_LEN);
    assert_eq!(tag.len(), TAG_LEN);

    let shared_secret = kem_decaps(recipient_kem_sk.as_slice(), &kem_ct).expect("decaps");
    let (ke, km) = derive_keys_ref(shared_secret.as_slice(), &escrow_id);
    let aad = build_aad_ref("alice", "bob", "tx_sign_req", &escrow_id, seq, &kem_ct);
    let expected_tag = compute_tag_ref(&km, &aad, &nonce, &ciphertext);
    assert_eq!(expected_tag.as_slice(), tag.as_slice());

    let sig_msg = build_sig_message_ref(&aad, &nonce, &ciphertext, &tag);
    falcon_verify(&sender_sig_pk, &sig_msg, &sig).expect("reference sig verify");

    let mut recovered = ciphertext.clone();
    xor_shake_keystream_ref(&mut recovered, &ke, &nonce);
    assert_eq!(recovered, plaintext);

    let decrypted = decrypt(
        "alice",
        "bob",
        "tx_sign_req",
        &escrow_id,
        seq,
        &sealed,
        recipient_kem_sk.as_slice(),
        &sender_sig_pk,
    )
    .expect("decrypt");
    assert_eq!(decrypted, plaintext);
}
