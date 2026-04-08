use privai_chain::canonical::canonical_bytes;
use privai_chain::consensus::*;
use privai_chain::note::{OutputNote, RecipientBox};
use privai_chain::primitives::{LweCiphertext, Nullifier};
use privai_chain::tx::*;

fn to_hex(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

#[test]
fn test_reference_vectors_consensus() {
    let transfer_output = OutputNote {
        version: 0,
        note_commit: [0xa1; 32],
        spend_policy_commit: [0xa2; 32],
        ct_amt: LweCiphertext::default(),
        aux_commit: [0xa3; 32],
        recipient_box: RecipientBox {
            version: 0,
            kem_alg: 1,
            aead_alg: 1,
            kem_ct: b"c1".to_vec(),
            nonce: [0xa4; 24],
            ciphertext: b"c2".to_vec(),
            tag: [0xa5; 16],
            hint: [0xa6; 16],
        },
    };

    let batch_output = OutputNote {
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

    let transfer_tx = Transaction::TransferNote(TransferNoteTx {
        core: TxCore {
            version: 1,
            tx_type: TX_TYPE_TRANSFER_NOTE,
            inputs: vec![InputRef { note_commit: [0xb1; 32] }],
            input_nullifiers: vec![Nullifier([0xb2; 32])],
            outputs: vec![transfer_output.clone()],
            fee: 5,
            statement_commit: [0xb3; 32],
            auth: vec![InputAuth {
                policy_tag: 1,
                signer_pks: vec![vec![0xb4, 0xb5]],
                signatures: vec![vec![0xb6]],
                policy_opening: None,
                escrow_action: None,
            }],
        },
    });

    let batch_tx = Transaction::MarketplaceBatch(MarketplaceBatchTx {
        core: TxCore {
            version: 1,
            tx_type: TX_TYPE_MARKETPLACE_BATCH,
            inputs: vec![InputRef { note_commit: [0x81; 32] }],
            input_nullifiers: vec![Nullifier([0x82; 32])],
            outputs: vec![batch_output.clone()],
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
        summary: privai_chain::small_payments::SettlementBatchSummary {
            operator_commit: [0x99; 32],
            merchant_commit: [0x11; 32],
            grant_commit: [0x12; 32],
            settlement_window_start: 1_670_000_000,
            settlement_window_end: 1_670_086_400,
            receipt_root: [0x13; 32],
            receipt_count: 3,
            nullifier_count: 2,
            total_gross_amount: 7035,
            total_fee_amount: 135,
            total_refund_amount: 210,
        },
        ticket_nullifiers: vec![Nullifier([0x91; 32]), Nullifier([0x92; 32])],
        operator_sig: vec![0xf0, 0x0d, 0xca, 0xfe],
    });

    let proof_cert = ProofCertificate {
        proof_system_id: 1,
        statement_root: [0xc1; 32],
        public_inputs_root: [0xc2; 32],
        proof_bytes_hash: [0xc3; 32],
        prover_ids: vec![[0xc4; 32]],
        proof_meta_hash: [0xc5; 32],
    };

    let block = Block::from_template(BlockTemplate {
        chain_id: 7,
        height: 42,
        epoch: 2,
        round: 3,
        timestamp_ms: 1_700_000_000_000,
        prev_block_hash: [0xd1; 32],
        proposer_pk_hash: [0xd2; 32],
        epoch_seed_hash: [0xd3; 32],
        parent_qc_hash: [0xd4; 32],
        state_root: [0xd5; 32],
        txs: vec![transfer_tx.clone(), batch_tx.clone()],
        execution_bundle: ExecutionBundle {
            statement_commits: vec![],
            covered_tx_indexes: vec![],
            public_inputs_root: [0u8; 32],
            execution_mode: ExecutionMode::Housekeeping,
        },
        proof_certificates: vec![proof_cert.clone()],
        extra_receipts: vec![],
    });

    let proof_cert_hash = privai_chain::hash::domain_hash(
        privai_chain::hash::PROOF_CERT_DOMAIN,
        &[&canonical_bytes(&proof_cert)],
    );
    assert_eq!(
        to_hex(&transfer_tx.tx_id()),
        "2866e8d39b1077f79a2e3fcc32f7145288672faaa8d3209af6198a0e03e4d45f"
    );
    assert_eq!(
        to_hex(&batch_tx.tx_id()),
        "02e2e1ef0cced7a84213121c59c5be970118a51585f368553389b08dd1d2e42e"
    );
    assert_eq!(
        to_hex(&transfer_output.note_commit),
        "a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1"
    );
    assert_eq!(
        to_hex(&batch_output.note_commit),
        "2121212121212121212121212121212121212121212121212121212121212121"
    );
    assert_eq!(
        to_hex(&Nullifier([0xb2; 32]).0),
        "b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2"
    );
    assert_eq!(
        to_hex(&Nullifier([0x91; 32]).0),
        "9191919191919191919191919191919191919191919191919191919191919191"
    );
    assert_eq!(
        to_hex(&Nullifier([0x92; 32]).0),
        "9292929292929292929292929292929292929292929292929292929292929292"
    );
    assert_eq!(
        to_hex(&proof_cert_hash),
        "5dfa271f95945c4c383c8fd28eb60a070bae74a69d5d9f9b3225d6716c69ae0e"
    );
    assert_eq!(
        to_hex(&block.header.note_root),
        "c86578e27ee77f0bd6e14cb07fa233d2cb59d293b596ac2c80689539b73ecec5"
    );
    assert_eq!(
        to_hex(&block.header.nullifier_root),
        "096d155558f80bc3193f6da17240b84a33812fdd91b9aa4220c6f6996499eb21"
    );
    assert_eq!(
        to_hex(&block.header.proof_cert_root),
        "5dfa271f95945c4c383c8fd28eb60a070bae74a69d5d9f9b3225d6716c69ae0e"
    );
    assert_eq!(
        to_hex(&block.header.statement_root),
        "f17f4be8222d7bacaa9b0e4fce84c2c66f885334891f96c5a24a3fb58c8a5a88"
    );
    assert_eq!(
        to_hex(&block.header.tx_root),
        "f5979146297b1d0786276e0a623dddc00edaeae71accfe53dc9d15c2f853763d"
    );
    assert_eq!(
        to_hex(&canonical_bytes(&block.header)),
        "00070000002a000000000000000200000000000000030000000068e5cf8b010000d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1f5979146297b1d0786276e0a623dddc00edaeae71accfe53dc9d15c2f853763dc86578e27ee77f0bd6e14cb07fa233d2cb59d293b596ac2c80689539b73ecec5096d155558f80bc3193f6da17240b84a33812fdd91b9aa4220c6f6996499eb21f17f4be8222d7bacaa9b0e4fce84c2c66f885334891f96c5a24a3fb58c8a5a885dfa271f95945c4c383c8fd28eb60a070bae74a69d5d9f9b3225d6716c69ae0ed5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4"
    );
    assert_eq!(
        to_hex(&block.hash()),
        "78eaa3778771568e7f12f08d372a852f8ebaaf92cd4c04e18389eae2d9d80594"
    );
}
