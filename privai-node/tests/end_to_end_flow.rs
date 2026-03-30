use privai_chain::{
    derive_aux_commit, Amount14, AuxWitness, CanonicalEncode, LweCiphertext, OutputNote,
    RecipientBoxPlaintext, SpendPolicy, Transaction,
};
use privai_ledger::{LedgerSnapshot, LedgerStore, MemoryStore, NoteRecord, NoteStatus};
use privai_node::{NodeConfig, PrivaiNode};
use privai_proof::artifact::{BatchProofArtifact, BlockProofArtifacts};
use privai_wallet::{MemoryWalletStore, PrivaiWallet, TransferOutputPlan};

fn make_funding_note(
    bundle: &privai_chain::ReceiveBundle,
    amount: Amount14,
    witness_seed: [u8; 32],
    nullifier_key: [u8; 32],
    spend_policy: SpendPolicy,
) -> (OutputNote, RecipientBoxPlaintext) {
    let aux_witness = AuxWitness {
        version: privai_chain::PRIVAI_V0,
        amount,
        witness_seed,
        noise_class: 1,
        bundle_id: bundle.bundle_id,
    };
    let aux_commit = derive_aux_commit(&aux_witness);
    let note_payload_commit = OutputNote::payload_commit_from_parts(
        privai_chain::PRIVAI_V0,
        &spend_policy.commitment(),
        &LweCiphertext::default(),
        &aux_commit,
    );
    let opened = RecipientBoxPlaintext {
        version: privai_chain::PRIVAI_V0,
        bundle_id: bundle.bundle_id,
        note_payload_commit,
        amount,
        witness_seed,
        nullifier_key,
        spend_policy_opening: spend_policy.to_canonical_bytes(),
        aux_opening: aux_witness.to_canonical_bytes(),
        sender_memo: Some(b"funding".to_vec()),
    };
    let recipient_box =
        PrivaiWallet::<MemoryWalletStore>::seal_recipient_box(bundle, &opened).expect("seal");
    let note = OutputNote::new(
        spend_policy.commitment(),
        LweCiphertext::default(),
        aux_commit,
        recipient_box,
    );

    (note, opened)
}

#[test]
fn wallet_transfer_proof_and_block_flow_roundtrip() {
    let config = NodeConfig::example();

    let mut sender_wallet = PrivaiWallet::open(MemoryWalletStore::new()).expect("sender wallet");
    let sender_bundle = sender_wallet
        .create_local_bundle(10_000, 0, Some(b"sender".to_vec()))
        .expect("sender bundle");

    let funding_amount = Amount14::new(77).expect("amount");
    let sender_spend_policy = SpendPolicy::Single {
        falcon_pk_hash: [0x31; 32],
    };
    let (funding_note, _funding_opened_direct) = make_funding_note(
        &sender_bundle,
        funding_amount,
        [0x21; 32],
        [0x22; 32],
        sender_spend_policy.clone(),
    );
    let funding_opened = sender_wallet
        .open_recipient_box(&funding_note)
        .expect("open funding note");
    sender_wallet
        .record_opened_note(funding_note.clone(), funding_opened)
        .expect("record funding note");
    let spend = sender_wallet
        .spend_material(&funding_note.note_commit)
        .expect("spend material");

    let mut recipient_wallet =
        PrivaiWallet::open(MemoryWalletStore::new()).expect("recipient wallet");
    let recipient_bundle = recipient_wallet
        .create_local_bundle(20_000, 0, Some(b"recipient".to_vec()))
        .expect("recipient bundle");

    let built = sender_wallet
        .build_transfer_note(
            &spend,
            vec![TransferOutputPlan {
                bundle: recipient_bundle.clone(),
                amount: Amount14::new(74).expect("recipient amount"),
                ct_amt: LweCiphertext::default(),
                witness_seed: [0x41; 32],
                nullifier_key: [0x42; 32],
                spend_policy: SpendPolicy::Single {
                    falcon_pk_hash: [0x51; 32],
                },
                noise_class: 1,
                sender_memo: Some(b"hello recipient".to_vec()),
            }],
            3,
            Vec::new(),
        )
        .expect("build transfer");

    let mut store = MemoryStore::new();
    let mut snapshot = LedgerSnapshot::genesis(config.chain_id);
    snapshot.notes.insert(
        funding_note.note_commit,
        NoteRecord {
            note: funding_note.clone(),
            created_in_block: Some(0),
            status: NoteStatus::Unspent,
        },
    );
    store.save(&snapshot).expect("seed ledger");

    let mut node = PrivaiNode::open(config.clone(), store).expect("node");
    node.submit_transaction(Transaction::TransferNote(built.tx.clone()), 1_000)
        .expect("submit transfer");

    let artifact = BatchProofArtifact {
        proof_system_id: 1,
        statement_root: privai_chain::merkle_root([built.proof.statement.commitment()]),
        public_inputs_root: privai_chain::merkle_root([built.proof.public_inputs_hash()]),
        covered_tx_indexes: vec![0],
        proof_bytes: vec![0xAA, 0xBB, 0xCC],
        prover_ids: vec![[0x91; 32]],
        proof_meta_hash: [0x81; 32],
    };

    let block = node
        .propose_block(
            0,
            0,
            1_000,
            [2; 32],
            [3; 32],
            vec![artifact.certificate()],
            Vec::new(),
        )
        .expect("propose block");
    let artifacts = BlockProofArtifacts::from_transfer_proofs(
        block.hash(),
        std::slice::from_ref(&built.proof),
        vec![artifact],
    )
    .expect("build sidecar artifacts");

    node.import_block_with_artifacts(&block, Some(&artifacts))
        .expect("import block");

    let ledger = node.ledger();
    let spent_record = ledger
        .snapshot()
        .notes
        .get(&funding_note.note_commit)
        .expect("spent funding note");
    assert!(matches!(spent_record.status, NoteStatus::Spent { .. }));
    assert_eq!(ledger.snapshot().height, 1);

    let received_note = built.tx.core.outputs[0].clone();
    let opened_received = recipient_wallet
        .open_recipient_box(&received_note)
        .expect("open received note");
    recipient_wallet
        .record_opened_note(received_note.clone(), opened_received)
        .expect("record received note");

    let recipient_record = recipient_wallet
        .snapshot()
        .owned_notes
        .get(&received_note.note_commit)
        .expect("recipient note");
    assert_eq!(recipient_record.opened.amount.value(), 74);

    let stored_artifacts = node
        .load_block_artifacts(&block.hash())
        .expect("load artifacts")
        .expect("stored artifacts");
    assert_eq!(stored_artifacts.execution_bundle, block.body.execution_bundle);
}
