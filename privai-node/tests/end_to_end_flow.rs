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
fn three_node_consensus_flow_propose_prevote_precommit_finalize() {
    use privai_chain::{BlockTemplate, VoteType, hash::domain_hash};
    use privai_node::config::ValidatorConfig;
    use nxms_transport::crypto::{falcon_keygen, falcon_sign_ct_prepared};

    // 1. Setup 3 validatorów z kluczami Falcon
    let (sk1, pk1) = falcon_keygen().expect("keygen 1");
    let (sk2, pk2) = falcon_keygen().expect("keygen 2");
    let (sk3, pk3) = falcon_keygen().expect("keygen 3");

    fn pk_hash(pk: &[u8]) -> privai_chain::Hash32 {
        domain_hash("privai:falcon-pk:v0", &[pk])
    }

    let val1 = ValidatorConfig {
        pk_hash: pk_hash(&pk1),
        sig_pk: pk1.clone(),
        stake_weight: 10,
        availability: 1,
        proof_score: 1,
    };
    let val2 = ValidatorConfig {
        pk_hash: pk_hash(&pk2),
        sig_pk: pk2.clone(),
        stake_weight: 10,
        availability: 1,
        proof_score: 1,
    };
    let val3 = ValidatorConfig {
        pk_hash: pk_hash(&pk3),
        sig_pk: pk3.clone(),
        stake_weight: 10,
        availability: 1,
        proof_score: 1,
    };

    let mut config = NodeConfig::example();
    config.validators = vec![val1.clone(), val2.clone(), val3.clone()];
    // Ustawiamy node 1 jako nasz local node
    config.node_pk_hash = val1.pk_hash;
    config.node_sig_pk = pk1.clone();
    
    // Create 3 nodes in memory
    let mut node1 = PrivaiNode::open(config.clone(), MemoryStore::new()).expect("node1");
    node1 = node1.with_falcon_key(sk1.to_vec());
    
    let mut config2 = config.clone();
    config2.node_pk_hash = val2.pk_hash;
    config2.node_sig_pk = pk2.clone();
    let mut node2 = PrivaiNode::open(config2, MemoryStore::new()).expect("node2");
    node2 = node2.with_falcon_key(sk2.to_vec());

    let mut config3 = config.clone();
    config3.node_pk_hash = val3.pk_hash;
    config3.node_sig_pk = pk3.clone();
    let mut node3 = PrivaiNode::open(config3, MemoryStore::new()).expect("node3");
    node3 = node3.with_falcon_key(sk3.to_vec());

    // 2. Node 1 proponuje blok
    let block = node1.propose_block(
        0, 0, 1000, [0; 32], [0; 32], vec![], vec![]
    ).expect("propose");

    let template = BlockTemplate {
        chain_id: block.header.chain_id,
        height: block.header.height,
        epoch: block.header.epoch,
        round: block.header.round,
        timestamp_ms: block.header.timestamp_ms,
        prev_block_hash: block.header.prev_block_hash,
        proposer_pk_hash: block.header.proposer_pk_hash,
        epoch_seed_hash: block.header.epoch_seed_hash,
        parent_qc_hash: block.header.parent_qc_hash,
        state_root: block.header.state_root,
        txs: block.body.txs.clone(),
        execution_bundle: block.body.execution_bundle.clone(),
        proof_certificates: block.body.proof_certificates.clone(),
        extra_receipts: block.body.extra_receipts.clone(),
    };

    // 3. Wszyscy głosują PREVOTE (tworzą głos)
    let vote1_prevote = node1.create_vote_for_proposal(&template).expect("vote1");
    let vote2_prevote = node2.create_vote_for_proposal(&template).expect("vote2");
    let vote3_prevote = node3.create_vote_for_proposal(&template).expect("vote3");

    assert_eq!(vote1_prevote.vote_type, VoteType::Prevote);

    // Node 1 zbiera PREVOTE i powinien wygenerować QC gdy osiągnie threshold (2/3 stake = 21, czyli 3 nody po 10 stake = 30 -> 2.01 nodes, więc 3 nody wymagane bo > 2)
    // Zobaczmy: total stake = 30. (30 * 2) / 3 + 1 = 21. Więc wystarczą 3 głosy po 10 = 30. Ale zaraz, 2 * 10 = 20 < 21. Więc potrzeba 3 głosy.
    assert!(node1.receive_vote(vote1_prevote).is_none());
    assert!(node1.receive_vote(vote2_prevote).is_none());
    
    let prevote_qc = node1.receive_vote(vote3_prevote).expect("should emit Prevote QC");
    assert_eq!(prevote_qc.vote_type, VoteType::Prevote);
    
    // 4. Nody otrzymują Prevote QC, i wysyłają PRECOMMIT
    let vote1_precommit = privai_chain::Vote {
        height: block.header.height,
        round: block.header.round,
        block_hash: block.hash(),
        vote_type: VoteType::Precommit,
        validator_pk: pk1.clone(),
        falcon_sig: falcon_sign_ct_prepared(&sk1, &block.hash()).expect("sign"),
    };
    
    let vote2_precommit = privai_chain::Vote {
        height: block.header.height,
        round: block.header.round,
        block_hash: block.hash(),
        vote_type: VoteType::Precommit,
        validator_pk: pk2.clone(),
        falcon_sig: falcon_sign_ct_prepared(&sk2, &block.hash()).expect("sign"),
    };
    
    let vote3_precommit = privai_chain::Vote {
        height: block.header.height,
        round: block.header.round,
        block_hash: block.hash(),
        vote_type: VoteType::Precommit,
        validator_pk: pk3.clone(),
        falcon_sig: falcon_sign_ct_prepared(&sk3, &block.hash()).expect("sign"),
    };

    // Node 1 zbiera PRECOMMIT i generuje Final QC
    assert!(node1.receive_vote(vote1_precommit).is_none());
    assert!(node1.receive_vote(vote2_precommit).is_none());
    
    let precommit_qc = node1.receive_vote(vote3_precommit).expect("should emit Precommit QC");
    assert_eq!(precommit_qc.vote_type, VoteType::Precommit);

    // 5. Finalizacja bloku
    node1.finalize_block_with_qc(&block, &precommit_qc).expect("finalize");
    node2.finalize_block_with_qc(&block, &precommit_qc).expect("finalize");
    node3.finalize_block_with_qc(&block, &precommit_qc).expect("finalize");

    // Weryfikacja: block height powinien wzrosnąć
    assert_eq!(node1.ledger().snapshot().height, 1);
    assert_eq!(node2.ledger().snapshot().height, 1);
    assert_eq!(node3.ledger().snapshot().height, 1);
    assert_eq!(node1.ledger().snapshot().tip_hash, block.hash());
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
