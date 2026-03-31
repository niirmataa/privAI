use std::env;

use privai_chain::{DELTA, LWE_DIMENSION, LWE_MODULUS_Q, PLAINTEXT_SPACE_P};

const HASH32_BYTES: usize = 32;
const U32_LEN_BYTES: usize = 4;
const U64_BYTES: usize = 8;
const U8_BYTES: usize = 1;

fn parse_arg(args: &[String], index: usize, default: usize) -> usize {
    args.get(index)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn lwe_ciphertext_bytes() -> usize {
    (LWE_DIMENSION * 4) + 4
}

fn recipient_box_bytes(kem_ct_len: usize, ciphertext_len: usize) -> usize {
    U8_BYTES + U8_BYTES + U8_BYTES
        + U32_LEN_BYTES + kem_ct_len
        + 24
        + U32_LEN_BYTES + ciphertext_len
        + 16
        + 16
}

fn output_note_bytes(kem_ct_len: usize, ciphertext_len: usize) -> usize {
    U8_BYTES
        + HASH32_BYTES
        + HASH32_BYTES
        + lwe_ciphertext_bytes()
        + HASH32_BYTES
        + recipient_box_bytes(kem_ct_len, ciphertext_len)
}

fn output_note_bytes_recipient_privacy_lite(kem_ct_len: usize, ciphertext_len: usize) -> usize {
    U8_BYTES
        + HASH32_BYTES
        + HASH32_BYTES
        + 2 // Amount14 in public plaintext space
        + HASH32_BYTES
        + recipient_box_bytes(kem_ct_len, ciphertext_len)
}

fn execution_bundle_bytes(covered_txs: usize) -> usize {
    U32_LEN_BYTES
        + (HASH32_BYTES * covered_txs)
        + U32_LEN_BYTES
        + (U32_LEN_BYTES * covered_txs)
        + HASH32_BYTES
        + U8_BYTES
}

fn proof_certificate_bytes(prover_ids: usize) -> usize {
    U8_BYTES
        + HASH32_BYTES
        + HASH32_BYTES
        + HASH32_BYTES
        + U32_LEN_BYTES
        + (HASH32_BYTES * prover_ids)
        + HASH32_BYTES
}

fn transfer_note_tx_lower_bound_bytes(inputs: usize, outputs: usize, output_note_bytes: usize) -> usize {
    U8_BYTES // version
        + U8_BYTES // tx_type
        + U32_LEN_BYTES + (inputs * HASH32_BYTES) // inputs
        + U32_LEN_BYTES + (inputs * HASH32_BYTES) // nullifiers
        + U32_LEN_BYTES + (outputs * output_note_bytes) // outputs
        + U64_BYTES // fee
        + HASH32_BYTES // statement_commit
        + U32_LEN_BYTES // auth vec len; does not include auth payload
}

fn amortized_total_metadata_bytes_per_purchase(
    tx_lower_bound: usize,
    execution_bundle: usize,
    proof_certificate: usize,
    purchases_per_settlement: usize,
) -> f64 {
    let purchases = purchases_per_settlement.max(1) as f64;
    (tx_lower_bound + execution_bundle + proof_certificate) as f64 / purchases
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let kem_ct_len = parse_arg(&args, 0, 0);
    let box_ct_len = parse_arg(&args, 1, 0);
    let covered_txs = parse_arg(&args, 2, 1);
    let prover_ids = parse_arg(&args, 3, 1);
    let inputs = parse_arg(&args, 4, 2);
    let outputs = parse_arg(&args, 5, 2);

    let lwe_bytes = lwe_ciphertext_bytes();
    let box_bytes = recipient_box_bytes(kem_ct_len, box_ct_len);
    let note_bytes = output_note_bytes(kem_ct_len, box_ct_len);
    let lite_note_bytes = output_note_bytes_recipient_privacy_lite(kem_ct_len, box_ct_len);
    let bundle_bytes = execution_bundle_bytes(covered_txs);
    let cert_bytes = proof_certificate_bytes(prover_ids);
    let proof_meta_per_tx = (bundle_bytes + cert_bytes) as f64 / covered_txs.max(1) as f64;
    let tx_lower_bound = transfer_note_tx_lower_bound_bytes(inputs, outputs, note_bytes);
    let scenario_direct = amortized_total_metadata_bytes_per_purchase(
        tx_lower_bound,
        bundle_bytes,
        cert_bytes,
        1,
    );
    let scenario_deposit = amortized_total_metadata_bytes_per_purchase(
        tx_lower_bound,
        bundle_bytes,
        cert_bytes,
        10,
    );
    let scenario_tab = amortized_total_metadata_bytes_per_purchase(
        tx_lower_bound,
        bundle_bytes,
        cert_bytes,
        50,
    );
    let scenario_escrow_batch = amortized_total_metadata_bytes_per_purchase(
        tx_lower_bound,
        bundle_bytes,
        cert_bytes,
        20,
    );

    println!("privAI v0 economics report");
    println!();
    println!("parameters");
    println!("  q = {}", LWE_MODULUS_Q);
    println!("  p = {}", PLAINTEXT_SPACE_P);
    println!("  delta = {}", DELTA);
    println!("  n = {}", LWE_DIMENSION);
    println!();
    println!("assumptions");
    println!("  recipient_box.kem_ct bytes = {}", kem_ct_len);
    println!("  recipient_box.ciphertext bytes = {}", box_ct_len);
    println!("  covered_txs per proof artifact = {}", covered_txs);
    println!("  prover_ids per certificate = {}", prover_ids);
    println!("  transfer inputs = {}", inputs);
    println!("  transfer outputs = {}", outputs);
    println!();
    println!("sizes");
    println!("  lwe_ciphertext_bytes = {}", lwe_bytes);
    println!("  recipient_box_bytes = {}", box_bytes);
    println!("  output_note_bytes = {}", note_bytes);
    println!(
        "  output_note_bytes_recipient_privacy_lite = {}",
        lite_note_bytes
    );
    println!("  execution_bundle_bytes = {}", bundle_bytes);
    println!("  proof_certificate_bytes = {}", cert_bytes);
    println!("  amortized_proof_metadata_bytes_per_tx = {:.2}", proof_meta_per_tx);
    println!("  transfer_note_tx_lower_bound_bytes = {}", tx_lower_bound);
    println!();
    println!("settlement scenarios");
    println!("  direct_settlement_bytes_per_purchase = {:.2}", scenario_direct);
    println!("  deposit_rail_bytes_per_purchase_at_10_uses = {:.2}", scenario_deposit);
    println!("  merchant_tab_bytes_per_purchase_at_50_uses = {:.2}", scenario_tab);
    println!(
        "  batch_escrow_bytes_per_purchase_at_20_uses = {:.2}",
        scenario_escrow_batch
    );
    println!();
    println!("notes");
    println!("  - transfer_note_tx_lower_bound_bytes excludes auth payload bytes.");
    println!("  - full proof bytes are sidecar/artifact bytes, not block-body payload in the current design.");
    println!("  - use non-zero kem_ct/ciphertext lengths to model realistic RecipientBox costs.");
    println!("  - settlement scenarios show why small packages should prefer deposit/tab/batch rails over direct on-chain settlement.");
    println!("  - recipient_privacy_lite removes the LWE amount ciphertext, but RecipientBox still dominates note size.");
}
