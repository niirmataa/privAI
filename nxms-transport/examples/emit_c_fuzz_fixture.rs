#![cfg(feature = "crypto")]

use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use nxms_transport::crypto::{Keys, PreparedTransportSigner, encrypt_with_signer};
use std::env;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

const FIXTURE_MAGIC: &[u8; 8] = b"NXMSFIX1";
const INPUT_MAGIC: &[u8; 8] = b"NXMSINP1";
const TARGET_SEQ: u8 = 1;
const TARGET_ESCROW: u8 = 2;
const TARGET_SENDER: u8 = 3;
const TARGET_TO: u8 = 4;
const TARGET_MSG: u8 = 5;
const TARGET_KEM: u8 = 6;
const TARGET_NONCE: u8 = 7;
const TARGET_CIPHERTEXT: u8 = 8;
const TARGET_TAG: u8 = 9;
const TARGET_SIG: u8 = 10;
const TARGET_PUBKEY: u8 = 11;
const OP_XOR: u8 = 0;
const OP_SET: u8 = 1;
const OP_ADD: u8 = 2;
const OP_ZERO: u8 = 3;

fn push_u16_le(out: &mut Vec<u8>, value: usize, label: &str) -> Result<()> {
    if value > u16::MAX as usize {
        bail!("{label} too large for u16");
    }
    out.extend_from_slice(&(value as u16).to_le_bytes());
    Ok(())
}

fn write_fixture(path: &Path, recipient_kem_sk: &[u8], sender_sig_pk: &[u8]) -> Result<()> {
    let mut out = Vec::with_capacity(8 + 4 + recipient_kem_sk.len() + 4 + sender_sig_pk.len());
    out.extend_from_slice(FIXTURE_MAGIC);
    out.extend_from_slice(&(recipient_kem_sk.len() as u32).to_le_bytes());
    out.extend_from_slice(recipient_kem_sk);
    out.extend_from_slice(&(sender_sig_pk.len() as u32).to_le_bytes());
    out.extend_from_slice(sender_sig_pk);
    fs::write(path, out).with_context(|| format!("write fixture {}", path.display()))
}

fn write_falcon_fixture(
    path: &Path,
    sender_sig_sk: &[u8],
    sender_sig_pk: &[u8],
    messages: &[Vec<u8>],
) -> Result<()> {
    let mut out = Vec::new();
    out.extend_from_slice(b"NXMSFAL1");
    out.extend_from_slice(&(sender_sig_sk.len() as u32).to_le_bytes());
    out.extend_from_slice(sender_sig_sk);
    out.extend_from_slice(&(sender_sig_pk.len() as u32).to_le_bytes());
    out.extend_from_slice(sender_sig_pk);
    out.extend_from_slice(&(messages.len() as u32).to_le_bytes());
    for msg in messages {
        out.extend_from_slice(&(msg.len() as u32).to_le_bytes());
        out.extend_from_slice(msg);
    }
    fs::write(path, out).with_context(|| format!("write falcon fixture {}", path.display()))
}

fn write_rust_fuzz_seed(
    path: &Path,
    kem_ct_b64: &str,
    nonce_b64: &str,
    ciphertext_b64: &str,
    tag_b64: &str,
    sig_b64: &str,
) -> Result<()> {
    let json = serde_json::json!({
        "kem_ct_b64": kem_ct_b64,
        "nonce_b64": nonce_b64,
        "ciphertext_b64": ciphertext_b64,
        "tag_b64": tag_b64,
        "sig_b64": sig_b64,
    });
    fs::write(path, serde_json::to_vec_pretty(&json)?)
        .with_context(|| format!("write rust fuzz seed {}", path.display()))
}

struct InputPacket<'a> {
    sender_id: &'a [u8],
    to_id: &'a [u8],
    msg_type: &'a [u8],
    escrow_id: &'a [u8; 16],
    seq: u64,
    kem_ct: &'a [u8],
    nonce: &'a [u8],
    ciphertext: &'a [u8],
    tag: &'a [u8],
    sig: &'a [u8],
}

struct ValidVariant {
    name: String,
    sender_id: String,
    to_id: String,
    msg_type: String,
    escrow_id: [u8; 16],
    seq: u64,
    plaintext: Vec<u8>,
}

struct EncodedInput {
    bytes: Vec<u8>,
    kem_ct_range: Range<usize>,
    ciphertext_range: Range<usize>,
    tag_range: Range<usize>,
}

fn encode_input(packet: &InputPacket<'_>) -> Result<EncodedInput> {
    let mut out = Vec::new();
    out.extend_from_slice(INPUT_MAGIC);
    out.extend_from_slice(&packet.seq.to_le_bytes());
    out.extend_from_slice(packet.escrow_id);
    push_u16_le(&mut out, packet.sender_id.len(), "sender_id")?;
    push_u16_le(&mut out, packet.to_id.len(), "to_id")?;
    push_u16_le(&mut out, packet.msg_type.len(), "msg_type")?;
    push_u16_le(&mut out, packet.kem_ct.len(), "kem_ct")?;
    push_u16_le(&mut out, packet.nonce.len(), "nonce")?;
    push_u16_le(&mut out, packet.ciphertext.len(), "ciphertext")?;
    push_u16_le(&mut out, packet.tag.len(), "tag")?;
    push_u16_le(&mut out, packet.sig.len(), "sig")?;
    out.extend_from_slice(packet.sender_id);
    out.extend_from_slice(packet.to_id);
    out.extend_from_slice(packet.msg_type);

    let kem_ct_start = out.len();
    out.extend_from_slice(packet.kem_ct);
    let kem_ct_end = out.len();

    out.extend_from_slice(packet.nonce);

    let ciphertext_start = out.len();
    out.extend_from_slice(packet.ciphertext);
    let ciphertext_end = out.len();

    let tag_start = out.len();
    out.extend_from_slice(packet.tag);
    let tag_end = out.len();

    out.extend_from_slice(packet.sig);
    Ok(EncodedInput {
        bytes: out,
        kem_ct_range: kem_ct_start..kem_ct_end,
        ciphertext_range: ciphertext_start..ciphertext_end,
        tag_range: tag_start..tag_end,
    })
}

fn write_bytes(path: &Path, data: &[u8]) -> Result<()> {
    fs::write(path, data).with_context(|| format!("write {}", path.display()))
}

fn push_offset_u16_le(out: &mut Vec<u8>, value: usize, label: &str) -> Result<()> {
    if value > u16::MAX as usize {
        bail!("{label} offset too large for u16");
    }
    out.extend_from_slice(&(value as u16).to_le_bytes());
    Ok(())
}

fn clear_dir_files(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path).with_context(|| format!("read dir {}", path.display()))? {
        let entry = entry.with_context(|| format!("read entry in {}", path.display()))?;
        let entry_path = entry.path();
        if entry_path.is_file() {
            match fs::remove_file(&entry_path) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => {
                    return Err(err)
                        .with_context(|| format!("remove stale file {}", entry_path.display()));
                }
            }
        }
    }
    Ok(())
}

fn mutate_first_byte(mut data: Vec<u8>, range: Range<usize>) -> Vec<u8> {
    if range.start < range.end && range.end <= data.len() {
        data[range.start] ^= 0x01;
    }
    data
}

fn mutate_last_byte(mut data: Vec<u8>) -> Vec<u8> {
    if let Some(last) = data.last_mut() {
        *last ^= 0x01;
    }
    data
}

fn encode_decrypt_mut_program(records: &[(u8, u8, usize, u8)]) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(records.len() * 5);
    for (target, op, offset, value) in records {
        out.push(*target);
        out.push(*op);
        push_offset_u16_le(&mut out, *offset, "decrypt mutation")?;
        out.push(*value);
    }
    Ok(out)
}

fn encode_kem_mut_program(records: &[(u8, usize, u8)]) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(records.len() * 4);
    for (op, offset, value) in records {
        out.push(*op);
        push_offset_u16_le(&mut out, *offset, "kem mutation")?;
        out.push(*value);
    }
    Ok(out)
}

fn format_afl_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("\\x{b:02X}")).collect::<String>()
}

fn write_structured_dict(path: &Path, entries: &[(&str, Vec<u8>)]) -> Result<()> {
    let mut dict = String::new();
    for (name, bytes) in entries {
        dict.push_str(&format!("{name}=\"{}\"\n", format_afl_bytes(bytes)));
    }
    write_bytes(path, dict.as_bytes())
}

fn valid_variants() -> Vec<ValidVariant> {
    let mut out = Vec::new();
    let base_escrow = [0x5Au8; 16];

    out.push(ValidVariant {
        name: "valid_default".to_string(),
        sender_id: "alice".to_string(),
        to_id: "bob".to_string(),
        msg_type: "tx_sign_req".to_string(),
        escrow_id: base_escrow,
        seq: 41,
        plaintext: br#"{"kind":"tx_sign_req","data":"c-fuzz-seed"}"#.to_vec(),
    });

    out.push(ValidVariant {
        name: "valid_offer".to_string(),
        sender_id: "market-alice".to_string(),
        to_id: "provider-bob".to_string(),
        msg_type: "market_offer".to_string(),
        escrow_id: [0x11; 16],
        seq: 42,
        plaintext: br#"{"kind":"offer","model":"llama-3","price":"17"}"#.to_vec(),
    });

    out.push(ValidVariant {
        name: "valid_witness".to_string(),
        sender_id: "wallet-01".to_string(),
        to_id: "node-01".to_string(),
        msg_type: "witness_update".to_string(),
        escrow_id: [0x22; 16],
        seq: 43,
        plaintext: br#"{"kind":"witness_update","note":"abc123","delta":"fresh"}"#.to_vec(),
    });

    out.push(ValidVariant {
        name: "valid_proof_job".to_string(),
        sender_id: "prover-a".to_string(),
        to_id: "validator-z".to_string(),
        msg_type: "proof_service_request".to_string(),
        escrow_id: [0x33; 16],
        seq: 44,
        plaintext: br#"{"kind":"proof_job","epoch":7,"batch":"feedbeef"}"#.to_vec(),
    });

    out.push(ValidVariant {
        name: "valid_inference_small".to_string(),
        sender_id: "buyer-1".to_string(),
        to_id: "provider-1".to_string(),
        msg_type: "inference_request".to_string(),
        escrow_id: [0x44; 16],
        seq: 45,
        plaintext: br#"{"kind":"infer","prompt":"hello model"}"#.to_vec(),
    });

    out.push(ValidVariant {
        name: "valid_inference_large".to_string(),
        sender_id: "buyer-2".to_string(),
        to_id: "provider-2".to_string(),
        msg_type: "inference_request".to_string(),
        escrow_id: [0x55; 16],
        seq: 46,
        plaintext: format!(
            "{{\"kind\":\"infer\",\"prompt\":\"{}\"}}",
            "local-model ".repeat(96)
        )
        .into_bytes(),
    });

    out.push(ValidVariant {
        name: "valid_binaryish".to_string(),
        sender_id: "bundle-src".to_string(),
        to_id: "bundle-dst".to_string(),
        msg_type: "bundle_delivery".to_string(),
        escrow_id: [0x66; 16],
        seq: 47,
        plaintext: (0..768).map(|i| (i % 251) as u8).collect(),
    });

    out.push(ValidVariant {
        name: "valid_ack".to_string(),
        sender_id: "validator-01".to_string(),
        to_id: "prover-01".to_string(),
        msg_type: "proof_ack".to_string(),
        escrow_id: [0x77; 16],
        seq: 48,
        plaintext: br#"{"kind":"ack","status":"ok","slot":12}"#.to_vec(),
    });

    out
}

fn write_afl_dict(path: &Path, variants: &[ValidVariant], sig_len: usize) -> Result<()> {
    let sig_len = u16::try_from(sig_len).context("sig length must fit into u16")?;
    let mut dict = String::new();
    dict.push_str("magic=\"NXMSINP1\"\n");
    dict.push_str("version_hint=\"\\x00\\x01\"\n");
    dict.push_str("kem_len=\"\\xF8\\x25\"\n");
    dict.push_str("nonce_len=\"\\x18\\x00\"\n");
    dict.push_str("tag_len=\"\\x20\\x00\"\n");
    dict.push_str(&format!(
        "sig_len=\"\\x{:02X}\\x{:02X}\"\n",
        sig_len as u8,
        (sig_len >> 8) as u8
    ));
    for (idx, variant) in variants.iter().enumerate() {
        dict.push_str(&format!("sender_{idx}=\"{}\"\n", variant.sender_id));
        dict.push_str(&format!("recipient_{idx}=\"{}\"\n", variant.to_id));
        dict.push_str(&format!("msg_type_{idx}=\"{}\"\n", variant.msg_type));
        dict.push_str(&format!(
            "sender_len_{idx}=\"{}\"\n",
            format_afl_bytes(&(variant.sender_id.len() as u16).to_le_bytes())
        ));
        dict.push_str(&format!(
            "recipient_len_{idx}=\"{}\"\n",
            format_afl_bytes(&(variant.to_id.len() as u16).to_le_bytes())
        ));
        dict.push_str(&format!(
            "msg_len_{idx}=\"{}\"\n",
            format_afl_bytes(&(variant.msg_type.len() as u16).to_le_bytes())
        ));
        dict.push_str(&format!(
            "seq_{idx}=\"{}\"\n",
            format_afl_bytes(&variant.seq.to_le_bytes())
        ));
        dict.push_str(&format!(
            "escrow_{idx}=\"{}\"\n",
            format_afl_bytes(&variant.escrow_id)
        ));
    }
    write_bytes(path, dict.as_bytes())
}

fn main() -> Result<()> {
    let out_dir = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .context("usage: cargo run --example emit_c_fuzz_fixture --features crypto -- <out-dir>")?;

    fs::create_dir_all(&out_dir)
        .with_context(|| format!("create output dir {}", out_dir.display()))?;

    let sender = Keys::generate().context("generate sender keys")?;
    let recipient = Keys::generate().context("generate recipient keys")?;

    let sender_sig_sk = sender.sig_sk_zeroizing().context("sender sig sk")?;
    let sender_sig_pk = sender.sig_pk().context("sender sig pk")?;
    let recipient_kem_pk = recipient.kem_pk().context("recipient kem pk")?;
    let recipient_kem_sk = recipient.kem_sk_zeroizing().context("recipient kem sk")?;
    let signer = PreparedTransportSigner::new(sender_sig_sk.as_slice()).context("prepared signer")?;

    let variants = valid_variants();
    let base_variant = &variants[0];
    let sealed = encrypt_with_signer(
        &base_variant.sender_id,
        &base_variant.to_id,
        &base_variant.msg_type,
        &base_variant.escrow_id,
        base_variant.seq,
        &recipient_kem_pk,
        &signer,
        &base_variant.plaintext,
    )
    .context("encrypt valid seed packet")?;

    let kem_ct = B64
        .decode(sealed.kem_ct_b64.as_bytes())
        .context("decode kem_ct")?;
    let nonce = B64
        .decode(sealed.nonce_b64.as_bytes())
        .context("decode nonce")?;
    let ciphertext = B64
        .decode(sealed.ciphertext_b64.as_bytes())
        .context("decode ciphertext")?;
    let tag = B64
        .decode(sealed.tag_b64.as_bytes())
        .context("decode tag")?;
    let sig = B64
        .decode(sealed.sig_b64.as_bytes())
        .context("decode sig")?;

    let fixture_path = out_dir.join("fixture.bin");
    write_fixture(&fixture_path, recipient_kem_sk.as_slice(), &sender_sig_pk)?;
    let falcon_fixture_path = out_dir.join("falcon_fixture.bin");
    let falcon_messages: Vec<Vec<u8>> = variants.iter().map(|variant| variant.plaintext.clone()).collect();
    write_falcon_fixture(
        &falcon_fixture_path,
        sender_sig_sk.as_slice(),
        &sender_sig_pk,
        &falcon_messages,
    )?;

    let encoded = encode_input(&InputPacket {
        sender_id: base_variant.sender_id.as_bytes(),
        to_id: base_variant.to_id.as_bytes(),
        msg_type: base_variant.msg_type.as_bytes(),
        escrow_id: &base_variant.escrow_id,
        seq: base_variant.seq,
        kem_ct: &kem_ct,
        nonce: &nonce,
        ciphertext: &ciphertext,
        tag: &tag,
        sig: &sig,
    })?;
    let seed_path = out_dir.join("seed_valid.bin");
    write_bytes(&seed_path, &encoded.bytes)?;

    let mutated_ciphertext = mutate_first_byte(encoded.bytes.clone(), encoded.ciphertext_range.clone());
    let legacy_mutated_seed_path = out_dir.join("seed_mutated.bin");
    write_bytes(&legacy_mutated_seed_path, &mutated_ciphertext)?;

    let rust_fuzz_dir = out_dir
        .parent()
        .map(|p| p.join("../fuzz/corpus/decrypt_fuzz"))
        .unwrap_or_else(|| PathBuf::from("fuzz/corpus/decrypt_fuzz"));
    fs::create_dir_all(&rust_fuzz_dir)
        .with_context(|| format!("create rust fuzz dir {}", rust_fuzz_dir.display()))?;
    let rust_fuzz_seed_path = rust_fuzz_dir.join("seed_valid.json");
    write_rust_fuzz_seed(
        &rust_fuzz_seed_path,
        &sealed.kem_ct_b64,
        &sealed.nonce_b64,
        &sealed.ciphertext_b64,
        &sealed.tag_b64,
        &sealed.sig_b64,
    )?;

    let fuzz_c_root = out_dir
        .parent()
        .context("output dir must have a parent directory")?;
    let decrypt_corpus_dir = fuzz_c_root.join("corpus_decrypt");
    let decrypt_valid_corpus_dir = fuzz_c_root.join("corpus_decrypt_valid");
    let decrypt_mut_corpus_dir = fuzz_c_root.join("corpus_decrypt_mut");
    let kem_corpus_dir = fuzz_c_root.join("corpus_kem");
    let kem_valid_corpus_dir = fuzz_c_root.join("corpus_kem_valid");
    let kem_mut_corpus_dir = fuzz_c_root.join("corpus_kem_mut");
    let kem_regression_dir = fuzz_c_root.join("corpus_kem_regression");
    let falcon_mut_corpus_dir = fuzz_c_root.join("corpus_falcon_mut");
    let falcon_seed_corpus_dir = fuzz_c_root.join("corpus_falcon_seed");
    fs::create_dir_all(&decrypt_corpus_dir)
        .with_context(|| format!("create decrypt corpus dir {}", decrypt_corpus_dir.display()))?;
    fs::create_dir_all(&decrypt_valid_corpus_dir)
        .with_context(|| format!("create decrypt valid corpus dir {}", decrypt_valid_corpus_dir.display()))?;
    fs::create_dir_all(&decrypt_mut_corpus_dir)
        .with_context(|| format!("create decrypt mutation corpus dir {}", decrypt_mut_corpus_dir.display()))?;
    fs::create_dir_all(&kem_corpus_dir)
        .with_context(|| format!("create kem corpus dir {}", kem_corpus_dir.display()))?;
    fs::create_dir_all(&kem_valid_corpus_dir)
        .with_context(|| format!("create kem valid corpus dir {}", kem_valid_corpus_dir.display()))?;
    fs::create_dir_all(&kem_mut_corpus_dir)
        .with_context(|| format!("create kem mutation corpus dir {}", kem_mut_corpus_dir.display()))?;
    fs::create_dir_all(&kem_regression_dir)
        .with_context(|| format!("create kem regression dir {}", kem_regression_dir.display()))?;
    fs::create_dir_all(&falcon_mut_corpus_dir)
        .with_context(|| format!("create falcon mutation corpus dir {}", falcon_mut_corpus_dir.display()))?;
    fs::create_dir_all(&falcon_seed_corpus_dir)
        .with_context(|| format!("create falcon sign-seed corpus dir {}", falcon_seed_corpus_dir.display()))?;
    clear_dir_files(&decrypt_corpus_dir)?;
    clear_dir_files(&decrypt_valid_corpus_dir)?;
    clear_dir_files(&decrypt_mut_corpus_dir)?;
    clear_dir_files(&kem_corpus_dir)?;
    clear_dir_files(&kem_valid_corpus_dir)?;
    clear_dir_files(&kem_mut_corpus_dir)?;
    clear_dir_files(&kem_regression_dir)?;
    clear_dir_files(&falcon_mut_corpus_dir)?;
    clear_dir_files(&falcon_seed_corpus_dir)?;

    write_bytes(&decrypt_corpus_dir.join("seed_valid.bin"), &encoded.bytes)?;
    write_bytes(&decrypt_valid_corpus_dir.join("valid_default.bin"), &encoded.bytes)?;
    write_bytes(
        &decrypt_corpus_dir.join("seed_tampered_ciphertext.bin"),
        &mutated_ciphertext,
    )?;
    let mutated_tag = mutate_first_byte(encoded.bytes.clone(), encoded.tag_range.clone());
    write_bytes(
        &decrypt_corpus_dir.join("seed_tampered_tag.bin"),
        &mutated_tag,
    )?;

    let tampered_sig = mutate_last_byte(sig.clone());
    let encoded_tampered_sig = encode_input(&InputPacket {
        sender_id: base_variant.sender_id.as_bytes(),
        to_id: base_variant.to_id.as_bytes(),
        msg_type: base_variant.msg_type.as_bytes(),
        escrow_id: &base_variant.escrow_id,
        seq: base_variant.seq,
        kem_ct: &kem_ct,
        nonce: &nonce,
        ciphertext: &ciphertext,
        tag: &tag,
        sig: &tampered_sig,
    })?;
    write_bytes(
        &decrypt_corpus_dir.join("seed_tampered_sig.bin"),
        &encoded_tampered_sig.bytes,
    )?;

    let tampered_kem_packet = encode_input(&InputPacket {
        sender_id: base_variant.sender_id.as_bytes(),
        to_id: base_variant.to_id.as_bytes(),
        msg_type: base_variant.msg_type.as_bytes(),
        escrow_id: &base_variant.escrow_id,
        seq: base_variant.seq,
        kem_ct: &mutate_first_byte(kem_ct.clone(), 0..kem_ct.len()),
        nonce: &nonce,
        ciphertext: &ciphertext,
        tag: &tag,
        sig: &sig,
    })?;
    write_bytes(
        &decrypt_corpus_dir.join("seed_tampered_kem.bin"),
        &tampered_kem_packet.bytes,
    )?;

    let seq_zero_packet = encode_input(&InputPacket {
        sender_id: base_variant.sender_id.as_bytes(),
        to_id: base_variant.to_id.as_bytes(),
        msg_type: base_variant.msg_type.as_bytes(),
        escrow_id: &base_variant.escrow_id,
        seq: 0,
        kem_ct: &kem_ct,
        nonce: &nonce,
        ciphertext: &ciphertext,
        tag: &tag,
        sig: &sig,
    })?;
    write_bytes(
        &decrypt_corpus_dir.join("seed_seq_zero.bin"),
        &seq_zero_packet.bytes,
    )?;

    let short_nonce_packet = encode_input(&InputPacket {
        sender_id: base_variant.sender_id.as_bytes(),
        to_id: base_variant.to_id.as_bytes(),
        msg_type: base_variant.msg_type.as_bytes(),
        escrow_id: &base_variant.escrow_id,
        seq: base_variant.seq,
        kem_ct: &kem_ct,
        nonce: &nonce[..nonce.len() - 1],
        ciphertext: &ciphertext,
        tag: &tag,
        sig: &sig,
    })?;
    write_bytes(
        &decrypt_corpus_dir.join("seed_short_nonce.bin"),
        &short_nonce_packet.bytes,
    )?;

    let short_tag_packet = encode_input(&InputPacket {
        sender_id: base_variant.sender_id.as_bytes(),
        to_id: base_variant.to_id.as_bytes(),
        msg_type: base_variant.msg_type.as_bytes(),
        escrow_id: &base_variant.escrow_id,
        seq: base_variant.seq,
        kem_ct: &kem_ct,
        nonce: &nonce,
        ciphertext: &ciphertext,
        tag: &tag[..tag.len() - 1],
        sig: &sig,
    })?;
    write_bytes(
        &decrypt_corpus_dir.join("seed_short_tag.bin"),
        &short_tag_packet.bytes,
    )?;

    let short_kem_packet = encode_input(&InputPacket {
        sender_id: base_variant.sender_id.as_bytes(),
        to_id: base_variant.to_id.as_bytes(),
        msg_type: base_variant.msg_type.as_bytes(),
        escrow_id: &base_variant.escrow_id,
        seq: base_variant.seq,
        kem_ct: &kem_ct[..64],
        nonce: &nonce,
        ciphertext: &ciphertext,
        tag: &tag,
        sig: &sig,
    })?;
    write_bytes(
        &decrypt_corpus_dir.join("seed_short_kem.bin"),
        &short_kem_packet.bytes,
    )?;

    let empty_sender_packet = encode_input(&InputPacket {
        sender_id: b"",
        to_id: base_variant.to_id.as_bytes(),
        msg_type: base_variant.msg_type.as_bytes(),
        escrow_id: &base_variant.escrow_id,
        seq: base_variant.seq,
        kem_ct: &kem_ct,
        nonce: &nonce,
        ciphertext: &ciphertext,
        tag: &tag,
        sig: &sig,
    })?;
    write_bytes(
        &decrypt_corpus_dir.join("seed_empty_sender.bin"),
        &empty_sender_packet.bytes,
    )?;

    let kem_mid = kem_ct.len() / 2;
    let ciphertext_mid = ciphertext.len() / 2;
    let tag_last = tag.len().saturating_sub(1);
    let sig_last = sig.len().saturating_sub(1);
    let decrypt_mut_seeds = [
        ("mut_noop.bin", Vec::new()),
        (
            "mut_kem_first_xor.bin",
            encode_decrypt_mut_program(&[(TARGET_KEM, OP_XOR, 0, 0x01)])?,
        ),
        (
            "mut_kem_mid_xor.bin",
            encode_decrypt_mut_program(&[(TARGET_KEM, OP_XOR, kem_mid, 0x80)])?,
        ),
        (
            "mut_ciphertext_first_xor.bin",
            encode_decrypt_mut_program(&[(TARGET_CIPHERTEXT, OP_XOR, 0, 0x01)])?,
        ),
        (
            "mut_tag_first_zero.bin",
            encode_decrypt_mut_program(&[(TARGET_TAG, OP_ZERO, 0, 0x00)])?,
        ),
        (
            "mut_sig_last_xor.bin",
            encode_decrypt_mut_program(&[(TARGET_SIG, OP_XOR, sig_last, 0x01)])?,
        ),
        (
            "mut_seq_low_zero.bin",
            encode_decrypt_mut_program(&[(TARGET_SEQ, OP_ZERO, 0, 0x00)])?,
        ),
        (
            "mut_sender_first_zero.bin",
            encode_decrypt_mut_program(&[(TARGET_SENDER, OP_ZERO, 0, 0x00)])?,
        ),
        (
            "mut_escrow_first_xor.bin",
            encode_decrypt_mut_program(&[(TARGET_ESCROW, OP_XOR, 0, 0x01)])?,
        ),
        (
            "mut_to_first_zero.bin",
            encode_decrypt_mut_program(&[(TARGET_TO, OP_ZERO, 0, 0x00)])?,
        ),
        (
            "mut_msg_first_zero.bin",
            encode_decrypt_mut_program(&[(TARGET_MSG, OP_ZERO, 0, 0x00)])?,
        ),
        (
            "mut_nonce_first_add.bin",
            encode_decrypt_mut_program(&[(TARGET_NONCE, OP_ADD, 0, 0x01)])?,
        ),
        (
            "mut_multi_mix.bin",
            encode_decrypt_mut_program(&[
                (TARGET_KEM, OP_XOR, kem_mid, 0x01),
                (TARGET_CIPHERTEXT, OP_ADD, ciphertext_mid, 0x11),
                (TARGET_TAG, OP_SET, tag_last, 0xAA),
                (TARGET_SIG, OP_XOR, sig_last, 0x80),
            ])?,
        ),
    ];
    for (name, bytes) in &decrypt_mut_seeds {
        write_bytes(&decrypt_mut_corpus_dir.join(name), bytes)?;
    }

    let kem_ct_valid = encoded.bytes[encoded.kem_ct_range.clone()].to_vec();
    let kem_ct_tampered = mutate_first_byte(
        kem_ct_valid.clone(),
        0..(encoded.kem_ct_range.end - encoded.kem_ct_range.start),
    );
    write_bytes(&kem_corpus_dir.join("kem_ct_valid.bin"), &kem_ct_valid)?;
    write_bytes(&kem_valid_corpus_dir.join("kem_ct_valid_default.bin"), &kem_ct_valid)?;
    write_bytes(
        &kem_corpus_dir.join("kem_ct_truncated.bin"),
        &kem_ct_valid[..64],
    )?;
    write_bytes(
        &out_dir.join("seed_tampered_kem.bin"),
        &kem_ct_tampered,
    )?;
    write_bytes(
        &kem_regression_dir.join("kem_ct_tampered.bin"),
        &kem_ct_tampered,
    )?;
    write_bytes(
        &kem_regression_dir.join("kem_ct_all_zero.bin"),
        &vec![0u8; kem_ct_valid.len()],
    )?;
    write_bytes(
        &kem_regression_dir.join("kem_ct_all_ff.bin"),
        &vec![0xFFu8; kem_ct_valid.len()],
    )?;
    let mut kem_half_zero = kem_ct_valid.clone();
    let kem_half_zero_mid = kem_half_zero.len() / 2;
    for b in &mut kem_half_zero[kem_half_zero_mid..] {
        *b = 0;
    }
    write_bytes(
        &kem_regression_dir.join("kem_ct_half_zero.bin"),
        &kem_half_zero,
    )?;
    write_bytes(
        &kem_regression_dir.join("kem_ct_last_byte_tampered.bin"),
        &mutate_last_byte(kem_ct_valid.clone()),
    )?;

    for variant in variants.iter().skip(1) {
        let sealed_variant = encrypt_with_signer(
            &variant.sender_id,
            &variant.to_id,
            &variant.msg_type,
            &variant.escrow_id,
            variant.seq,
            &recipient_kem_pk,
            &signer,
            &variant.plaintext,
        )
        .with_context(|| format!("encrypt {}", variant.name))?;

        let kem_ct_variant = B64
            .decode(sealed_variant.kem_ct_b64.as_bytes())
            .with_context(|| format!("decode kem_ct {}", variant.name))?;
        let nonce_variant = B64
            .decode(sealed_variant.nonce_b64.as_bytes())
            .with_context(|| format!("decode nonce {}", variant.name))?;
        let ciphertext_variant = B64
            .decode(sealed_variant.ciphertext_b64.as_bytes())
            .with_context(|| format!("decode ciphertext {}", variant.name))?;
        let tag_variant = B64
            .decode(sealed_variant.tag_b64.as_bytes())
            .with_context(|| format!("decode tag {}", variant.name))?;
        let sig_variant = B64
            .decode(sealed_variant.sig_b64.as_bytes())
            .with_context(|| format!("decode sig {}", variant.name))?;

        let encoded_variant = encode_input(&InputPacket {
            sender_id: variant.sender_id.as_bytes(),
            to_id: variant.to_id.as_bytes(),
            msg_type: variant.msg_type.as_bytes(),
            escrow_id: &variant.escrow_id,
            seq: variant.seq,
            kem_ct: &kem_ct_variant,
            nonce: &nonce_variant,
            ciphertext: &ciphertext_variant,
            tag: &tag_variant,
            sig: &sig_variant,
        })?;

        write_bytes(
            &decrypt_valid_corpus_dir.join(format!("{}.bin", variant.name)),
            &encoded_variant.bytes,
        )?;
        write_bytes(
            &kem_valid_corpus_dir.join(format!("{}.bin", variant.name)),
            &kem_ct_variant,
        )?;
    }

    let kem_last = kem_ct_valid.len().saturating_sub(1);
    let kem_mut_seeds = [
        ("mut_noop.bin", Vec::new()),
        (
            "mut_first_xor.bin",
            encode_kem_mut_program(&[(OP_XOR, 0, 0x01)])?,
        ),
        (
            "mut_mid_xor.bin",
            encode_kem_mut_program(&[(OP_XOR, kem_mid, 0x80)])?,
        ),
        (
            "mut_last_xor.bin",
            encode_kem_mut_program(&[(OP_XOR, kem_last, 0x01)])?,
        ),
        (
            "mut_first_zero.bin",
            encode_kem_mut_program(&[(OP_ZERO, 0, 0x00)])?,
        ),
        (
            "mut_mid_ff.bin",
            encode_kem_mut_program(&[(OP_SET, kem_mid, 0xFF)])?,
        ),
    ];
    for (name, bytes) in &kem_mut_seeds {
        write_bytes(&kem_mut_corpus_dir.join(name), bytes)?;
    }

    let falcon_mut_seeds = [
        ("mut_noop.bin", vec![0]),
        ("mut_msg_first_xor.bin", [vec![0], encode_decrypt_mut_program(&[(TARGET_MSG, OP_XOR, 0, 0x01)])?].concat()),
        ("mut_msg_mid_add.bin", [vec![4], encode_decrypt_mut_program(&[(TARGET_MSG, OP_ADD, 8, 0x11)])?].concat()),
        ("mut_sig_first_xor.bin", [vec![0], encode_decrypt_mut_program(&[(TARGET_SIG, OP_XOR, 0, 0x01)])?].concat()),
        ("mut_sig_mid_xor.bin", [vec![0], encode_decrypt_mut_program(&[(TARGET_SIG, OP_XOR, 32, 0x80)])?].concat()),
        ("mut_sig_last_zero.bin", [vec![0], encode_decrypt_mut_program(&[(TARGET_SIG, OP_ZERO, sig_last, 0x00)])?].concat()),
        ("mut_pk_first_xor.bin", [vec![0], encode_decrypt_mut_program(&[(TARGET_PUBKEY, OP_XOR, 0, 0x01)])?].concat()),
        (
            "mut_multi_mix.bin",
            [
                vec![7],
                encode_decrypt_mut_program(&[
                    (TARGET_MSG, OP_XOR, 0, 0x01),
                    (TARGET_SIG, OP_ADD, 64, 0x10),
                    (TARGET_PUBKEY, OP_XOR, 1, 0x80),
                ])?,
            ]
            .concat(),
        ),
        ("mode_keygen.bin", vec![0x20]),
        ("mode_truncated_sk.bin", vec![0x40]),
        ("mode_truncated_pk.bin", vec![0x60]),
        ("mode_truncated_sig.bin", vec![0x80]),
    ];
    for (name, bytes) in &falcon_mut_seeds {
        write_bytes(&falcon_mut_corpus_dir.join(name), bytes)?;
    }

    let falcon_seed_seeds = [
        ("seed_zero.bin", [vec![0], vec![0u8; 48], b"seeded-sign-path".to_vec()].concat()),
        ("seed_ff.bin", [vec![0], vec![0xFFu8; 48], b"seeded-sign-ff".to_vec()].concat()),
        (
            "seed_inc.bin",
            [
                vec![4],
                (0u8..48u8).collect::<Vec<u8>>(),
                b"seeded-sign-inc".to_vec(),
            ]
            .concat(),
        ),
        (
            "seed_alt_variant.bin",
            [
                vec![7],
                (0..48).map(|i| ((i * 3) & 0xFF) as u8).collect::<Vec<u8>>(),
                variants[7].plaintext.clone(),
            ]
            .concat(),
        ),
        ("seed_short.bin", vec![0, 0x11, 0x22, 0x33, 0x44]),
        (
            "seed_msg_large.bin",
            [
                vec![2],
                vec![0xA5u8; 48],
                "falcon-ct-sign ".repeat(64).into_bytes(),
            ]
            .concat(),
        ),
    ];
    for (name, bytes) in &falcon_seed_seeds {
        write_bytes(&falcon_seed_corpus_dir.join(name), bytes)?;
    }

    let decrypt_dict_path = fuzz_c_root.join("decrypt.dict");
    let kem_dict_path = fuzz_c_root.join("kem.dict");
    let decrypt_mut_dict_path = fuzz_c_root.join("decrypt_mut.dict");
    let kem_mut_dict_path = fuzz_c_root.join("kem_mut.dict");
    let falcon_mut_dict_path = fuzz_c_root.join("falcon_mut.dict");
    let falcon_seed_dict_path = fuzz_c_root.join("falcon_seed.dict");
    write_afl_dict(&decrypt_dict_path, &variants, sig.len())?;
    write_afl_dict(&kem_dict_path, &variants, sig.len())?;
    write_structured_dict(
        &decrypt_mut_dict_path,
        &[
            ("seq_zero", encode_decrypt_mut_program(&[(TARGET_SEQ, OP_ZERO, 0, 0x00)])?),
            ("kem_first_xor", encode_decrypt_mut_program(&[(TARGET_KEM, OP_XOR, 0, 0x01)])?),
            ("kem_mid_xor", encode_decrypt_mut_program(&[(TARGET_KEM, OP_XOR, kem_mid, 0x80)])?),
            ("escrow_first_xor", encode_decrypt_mut_program(&[(TARGET_ESCROW, OP_XOR, 0, 0x01)])?),
            ("to_first_zero", encode_decrypt_mut_program(&[(TARGET_TO, OP_ZERO, 0, 0x00)])?),
            ("msg_first_zero", encode_decrypt_mut_program(&[(TARGET_MSG, OP_ZERO, 0, 0x00)])?),
            ("cipher_first_xor", encode_decrypt_mut_program(&[(TARGET_CIPHERTEXT, OP_XOR, 0, 0x01)])?),
            ("tag_first_zero", encode_decrypt_mut_program(&[(TARGET_TAG, OP_ZERO, 0, 0x00)])?),
            ("sig_last_xor", encode_decrypt_mut_program(&[(TARGET_SIG, OP_XOR, sig_last, 0x01)])?),
            ("nonce_first_add", encode_decrypt_mut_program(&[(TARGET_NONCE, OP_ADD, 0, 0x01)])?),
        ],
    )?;
    write_structured_dict(
        &kem_mut_dict_path,
        &[
            ("kem_first_xor", encode_kem_mut_program(&[(OP_XOR, 0, 0x01)])?),
            ("kem_mid_xor", encode_kem_mut_program(&[(OP_XOR, kem_mid, 0x80)])?),
            ("kem_last_xor", encode_kem_mut_program(&[(OP_XOR, kem_last, 0x01)])?),
            ("kem_first_zero", encode_kem_mut_program(&[(OP_ZERO, 0, 0x00)])?),
            ("kem_mid_ff", encode_kem_mut_program(&[(OP_SET, kem_mid, 0xFF)])?),
        ],
    )?;
    write_structured_dict(
        &falcon_mut_dict_path,
        &[
            ("msg_first_xor", [vec![0], encode_decrypt_mut_program(&[(TARGET_MSG, OP_XOR, 0, 0x01)])?].concat()),
            ("msg_mid_add", [vec![4], encode_decrypt_mut_program(&[(TARGET_MSG, OP_ADD, 8, 0x11)])?].concat()),
            ("sig_first_xor", [vec![0], encode_decrypt_mut_program(&[(TARGET_SIG, OP_XOR, 0, 0x01)])?].concat()),
            ("sig_mid_xor", [vec![0], encode_decrypt_mut_program(&[(TARGET_SIG, OP_XOR, 32, 0x80)])?].concat()),
            ("sig_last_zero", [vec![0], encode_decrypt_mut_program(&[(TARGET_SIG, OP_ZERO, sig_last, 0x00)])?].concat()),
            ("pk_first_xor", [vec![0], encode_decrypt_mut_program(&[(TARGET_PUBKEY, OP_XOR, 0, 0x01)])?].concat()),
            ("mode_keygen", vec![0x20]),
            ("mode_truncated_sk", vec![0x40]),
            ("mode_truncated_pk", vec![0x60]),
            ("mode_truncated_sig", vec![0x80]),
        ],
    )?;
    write_structured_dict(
        &falcon_seed_dict_path,
        &[
            ("variant0", vec![0x00]),
            ("variant4", vec![0x04]),
            ("variant7", vec![0x07]),
            ("seed_zero_prefix", [vec![0x00], vec![0u8; 8]].concat()),
            ("seed_ff_prefix", [vec![0x00], vec![0xFFu8; 8]].concat()),
            ("seed_inc_prefix", [vec![0x04], (0u8..8u8).collect::<Vec<u8>>()].concat()),
        ],
    )?;

    println!("{}", fixture_path.display());
    println!("{}", seed_path.display());
    println!("{}", rust_fuzz_seed_path.display());
    println!("{}", decrypt_corpus_dir.display());
    println!("{}", kem_corpus_dir.display());
    println!("{}", decrypt_dict_path.display());
    println!("{}", kem_dict_path.display());
    println!("{}", decrypt_valid_corpus_dir.display());
    println!("{}", kem_valid_corpus_dir.display());
    println!("{}", decrypt_mut_corpus_dir.display());
    println!("{}", kem_mut_corpus_dir.display());
    println!("{}", decrypt_mut_dict_path.display());
    println!("{}", kem_mut_dict_path.display());
    println!("{}", falcon_fixture_path.display());
    println!("{}", falcon_mut_corpus_dir.display());
    println!("{}", falcon_mut_dict_path.display());
    println!("{}", falcon_seed_corpus_dir.display());
    println!("{}", falcon_seed_dict_path.display());
    Ok(())
}
