use privai_chain::canonical::canonical_bytes;
use privai_chain::hash::{domain_hash, merkle_root};
use privai_chain::note::{OutputNote, RecipientBox};
use privai_chain::primitives::{Hash32, LweCiphertext, Nullifier};
use privai_chain::small_payments::*;
use privai_chain::tx::*;

fn to_hex(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

fn receipt_root_from_commits(commits: &[Hash32]) -> Hash32 {
    domain_hash(RECEIPT_ROOT_DOMAIN, &[&merkle_root(commits.iter().copied())])
}

#[test]
fn test_reference_vectors_marketplace() {
    let policy = ServicePaymentPolicy {
        policy_version: 1,
        merchant_commit: [0x11; 32],
        service_commit: Some([0x22; 32]),
        allowed_rail: AllowedRail::SmallPaymentsRail,
        pricing_mode: PricingMode::ReservationThenSettle,
        min_deposit_required: 1000,
        max_spend_per_session: 5000,
        max_spend_per_window: 10000,
        grant_expiry_rule: 3600,
        settlement_window_rule: 86400,
        requires_full_privacy_if: 50000,
    };

    let grant = SpendGrant {
        merchant_commit: [0x11; 32],
        service_commit: Some([0x22; 32]),
        session_scope: [0x33; 32],
        spend_cap: 4000,
        grant_expiry: 1_670_000_000,
        settlement_window: 1_670_086_400,
        policy_commit: policy.policy_commit(),
        operator_sig: vec![0xaa, 0xbb, 0xcc, 0xdd],
    };

    let receipt1 = Receipt {
        receipt_id: [0x44; 32],
        merchant_commit: [0x11; 32],
        service_commit: Some([0x22; 32]),
        session_commit: [0x55; 32],
        grant_commit: grant.grant_commit(),
        purchase_commit: [0x66; 32],
        ticket_nullifier: Nullifier([0x77; 32]),
        amount: 1234,
        policy_commit: policy.policy_commit(),
        result_commit: [0x88; 32],
        issued_at: 1_670_001_234,
        merchant_sig: vec![0xdd, 0xee, 0xff],
    };

    let receipt2 = Receipt {
        receipt_id: [0x45; 32],
        merchant_commit: [0x11; 32],
        service_commit: Some([0x22; 32]),
        session_commit: [0x56; 32],
        grant_commit: grant.grant_commit(),
        purchase_commit: [0x67; 32],
        ticket_nullifier: Nullifier([0x78; 32]),
        amount: 2345,
        policy_commit: policy.policy_commit(),
        result_commit: [0x89; 32],
        issued_at: 1_670_001_235,
        merchant_sig: vec![0xde, 0xad, 0xbe, 0xef],
    };

    let receipt3 = Receipt {
        receipt_id: [0x46; 32],
        merchant_commit: [0x11; 32],
        service_commit: Some([0x22; 32]),
        session_commit: [0x57; 32],
        grant_commit: grant.grant_commit(),
        purchase_commit: [0x68; 32],
        ticket_nullifier: Nullifier([0x79; 32]),
        amount: 3456,
        policy_commit: policy.policy_commit(),
        result_commit: [0x8a; 32],
        issued_at: 1_670_001_236,
        merchant_sig: vec![0xfa, 0xce],
    };

    let receipt_commits = [
        receipt1.receipt_commit(),
        receipt2.receipt_commit(),
        receipt3.receipt_commit(),
    ];
    let receipt_root = receipt_root_from_commits(&receipt_commits);

    let summary = SettlementBatchSummary {
        operator_commit: [0x99; 32],
        merchant_commit: [0x11; 32],
        grant_commit: grant.grant_commit(),
        settlement_window_start: 1_670_000_000,
        settlement_window_end: 1_670_086_400,
        receipt_root,
        receipt_count: 3,
        nullifier_count: 2,
        total_gross_amount: 7035,
        total_fee_amount: 135,
        total_refund_amount: 210,
    };

    let sample_output = OutputNote {
        version: 0,
        note_commit: [0x21; 32],
        spend_policy_commit: [0x31; 32],
        ct_amt: LweCiphertext::default(),
        aux_commit: [0x41; 32],
        recipient_box: RecipientBox {
            version: 0,
            kem_alg: 1,
            aead_alg: 1,
            kem_ct: b"mkt".to_vec(),
            nonce: [0x51; 24],
            ciphertext: b"ok".to_vec(),
            tag: [0x61; 16],
            hint: [0x71; 16],
        },
    };

    let batch_tx = MarketplaceBatchTx {
        core: TxCore {
            version: 1,
            tx_type: TX_TYPE_MARKETPLACE_BATCH,
            inputs: vec![InputRef { note_commit: [0x81; 32] }],
            input_nullifiers: vec![Nullifier([0x82; 32])],
            outputs: vec![sample_output],
            fee: 77,
            statement_commit: [0x83; 32],
            auth: vec![InputAuth {
                policy_tag: 1,
                signer_pks: vec![vec![0x84, 0x85]],
                signatures: vec![vec![0x86, 0x87, 0x88]],
                policy_opening: None,
                escrow_action: None,
            }],
        },
        summary: summary.clone(),
        ticket_nullifiers: vec![Nullifier([0x91; 32]), Nullifier([0x92; 32])],
        operator_sig: vec![0xf0, 0x0d, 0xca, 0xfe],
    };

    assert_eq!(
        to_hex(&canonical_bytes(&policy)),
        "0111111111111111111111111111111111111111111111111111111111111111110122222222222222222222222222222222222222222222222222222222222222220101e80300000000000088130000000000001027000000000000100e00008051010050c3000000000000"
    );
    assert_eq!(
        to_hex(&policy.policy_commit()),
        "8d70f285a5c53af821d2ca3cb3cbe663ff4ed1be3720d14bddebd592aac7d83a"
    );
    assert_eq!(
        to_hex(&canonical_bytes(&grant)),
        "11111111111111111111111111111111111111111111111111111111111111110122222222222222222222222222222222222222222222222222222222222222223333333333333333333333333333333333333333333333333333333333333333a00f000000000000802d8a6300000000007f8b63000000008d70f285a5c53af821d2ca3cb3cbe663ff4ed1be3720d14bddebd592aac7d83a"
    );
    assert_eq!(
        to_hex(&grant.grant_commit()),
        "d858ecb797c050cf85b990bb2f9de4447cc31f443e3c8ec0da7d403664842d6a"
    );
    assert_eq!(
        to_hex(&[
            b"nxms_privai_grant_sig_v0".as_slice(),
            canonical_bytes(&grant).as_slice()
        ]
        .concat()),
        "6e786d735f7072697661695f6772616e745f7369675f763011111111111111111111111111111111111111111111111111111111111111110122222222222222222222222222222222222222222222222222222222222222223333333333333333333333333333333333333333333333333333333333333333a00f000000000000802d8a6300000000007f8b63000000008d70f285a5c53af821d2ca3cb3cbe663ff4ed1be3720d14bddebd592aac7d83a"
    );
    assert_eq!(to_hex(&grant.operator_sig), "aabbccdd");
    assert_eq!(
        to_hex(&canonical_bytes(&receipt1)),
        "444444444444444444444444444444444444444444444444444444444444444411111111111111111111111111111111111111111111111111111111111111110122222222222222222222222222222222222222222222222222222222222222225555555555555555555555555555555555555555555555555555555555555555d858ecb797c050cf85b990bb2f9de4447cc31f443e3c8ec0da7d403664842d6a66666666666666666666666666666666666666666666666666666666666666667777777777777777777777777777777777777777777777777777777777777777d2040000000000008d70f285a5c53af821d2ca3cb3cbe663ff4ed1be3720d14bddebd592aac7d83a888888888888888888888888888888888888888888888888888888888888888852328a6300000000"
    );
    assert_eq!(
        to_hex(&receipt1.receipt_commit()),
        "4823558d41cbbad092d5b8c007996c1c6b1d628892ae5bf9e80ab4c7d915a9c7"
    );
    assert_eq!(
        to_hex(&receipt2.receipt_commit()),
        "3662f84f16af48aa99e9adff4400de0f0d0b7a37c2ff215066f9ff23f2e80ffa"
    );
    assert_eq!(
        to_hex(&receipt3.receipt_commit()),
        "fd6ed2c3bc29c2a90319f2093194449d558552031d0560b7efab9774012581ad"
    );
    assert_eq!(
        to_hex(&[
            b"nxms_privai_receipt_sig_v0".as_slice(),
            canonical_bytes(&receipt1).as_slice()
        ]
        .concat()),
        "6e786d735f7072697661695f726563656970745f7369675f7630444444444444444444444444444444444444444444444444444444444444444411111111111111111111111111111111111111111111111111111111111111110122222222222222222222222222222222222222222222222222222222222222225555555555555555555555555555555555555555555555555555555555555555d858ecb797c050cf85b990bb2f9de4447cc31f443e3c8ec0da7d403664842d6a66666666666666666666666666666666666666666666666666666666666666667777777777777777777777777777777777777777777777777777777777777777d2040000000000008d70f285a5c53af821d2ca3cb3cbe663ff4ed1be3720d14bddebd592aac7d83a888888888888888888888888888888888888888888888888888888888888888852328a6300000000"
    );
    assert_eq!(to_hex(&receipt1.merchant_sig), "ddeeff");
    assert_eq!(
        to_hex(&canonical_bytes(&summary)),
        "99999999999999999999999999999999999999999999999999999999999999991111111111111111111111111111111111111111111111111111111111111111d858ecb797c050cf85b990bb2f9de4447cc31f443e3c8ec0da7d403664842d6a802d8a6300000000007f8b630000000014643d5c95e246a0c0a76f6a7982503d8ca85445ef4a906038b4caf23680d99f03000000020000007b1b0000000000008700000000000000d200000000000000"
    );
    assert_eq!(
        to_hex(&summary.settlement_root()),
        "c26aff7492fb62456f16075cedb8204ea0885760b643d29aede28816ecfda2ad"
    );
    assert_eq!(
        to_hex(&receipt_root),
        "14643d5c95e246a0c0a76f6a7982503d8ca85445ef4a906038b4caf23680d99f"
    );

    let batch_canonical_hex = to_hex(&canonical_bytes(&batch_tx));
    let expected_batch_canonical = include_str!("reference_vectors_marketplace_batch_hex.txt")
        .lines()
        .collect::<String>();
    assert_eq!(batch_canonical_hex, expected_batch_canonical);

    assert_eq!(
        to_hex(&Transaction::MarketplaceBatch(batch_tx.clone()).tx_id()),
        "6504cc0eee92d71b90691190ba01e3f57fec6f7b083be0ef5a48986e0f200325"
    );
    assert_eq!(
        to_hex(&summary.settlement_root()),
        "c26aff7492fb62456f16075cedb8204ea0885760b643d29aede28816ecfda2ad"
    );
}
