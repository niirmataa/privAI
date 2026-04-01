use thiserror::Error;

use std::collections::{HashMap, BTreeSet};

use privai_chain::{
    Block, BlockTemplate, ConsensusReceipt, ExecutionMode, Hash32, ProofCertificate, Transaction,
    Vote, VoteType, QuorumCertificate
};
use privai_ledger::{Ledger, LedgerError, LedgerStore};
use privai_proof::{
    artifact::BlockProofArtifacts,
    store::{MemoryProofArtifactStore, ProofArtifactStore, ProofArtifactStoreError},
    build_execution_bundle_from_transactions, BatchBuildError, BlockArtifactVerifier,
    ProofVerifier, SidecarProofVerifier, StructuralProofVerifier,
};

use crate::config::NodeConfig;
use crate::proposer::select_proposer;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    #[error(transparent)]
    ProofBatch(#[from] BatchBuildError),
    #[error(transparent)]
    ProofArtifacts(#[from] ProofArtifactStoreError),
    #[error(transparent)]
    Artifact(#[from] privai_proof::artifact::ArtifactError),
    #[error(transparent)]
    ArtifactVerify(#[from] privai_proof::ArtifactVerificationError),
    #[error("no validators are configured for proposer selection")]
    NoValidators,
    #[error("current node is not proposer for round {round}")]
    NotProposer { round: u32 },
}

pub struct PrivaiNode<
    S: LedgerStore,
    V: ProofVerifier = StructuralProofVerifier,
    A: ProofArtifactStore = MemoryProofArtifactStore,
    P: BlockArtifactVerifier = SidecarProofVerifier,
> {
    config: NodeConfig,
    ledger: Ledger<S, V>,
    proof_artifacts: A,
    artifact_verifier: P,
    
    // Proste liczniki glosow z pamiecia 1 epoki w PC-BFT do budowy QC
    prevotes: HashMap<Hash32, BTreeSet<Vec<u8>>>,
    precommits: HashMap<Hash32, BTreeSet<Vec<u8>>>,
}

impl<S: LedgerStore> PrivaiNode<S, StructuralProofVerifier, MemoryProofArtifactStore, SidecarProofVerifier> {
    pub fn open(config: NodeConfig, store: S) -> Result<Self, NodeError> {
        Self::open_with_verifier(config, store, StructuralProofVerifier)
    }
}

impl<S: LedgerStore, A: ProofArtifactStore> PrivaiNode<S, StructuralProofVerifier, A, SidecarProofVerifier> {
    pub fn open_with_artifact_store(
        config: NodeConfig,
        store: S,
        artifact_store: A,
    ) -> Result<Self, NodeError> {
        Self::open_with_components(
            config,
            store,
            StructuralProofVerifier,
            artifact_store,
            SidecarProofVerifier::default(),
        )
    }
}

impl<S: LedgerStore, V: ProofVerifier> PrivaiNode<S, V, MemoryProofArtifactStore, SidecarProofVerifier> {
    pub fn open_with_verifier(config: NodeConfig, store: S, verifier: V) -> Result<Self, NodeError> {
        Self::open_with_components(
            config,
            store,
            verifier,
            MemoryProofArtifactStore::new(),
            SidecarProofVerifier::default(),
        )
    }
}

impl<S: LedgerStore, V: ProofVerifier, A: ProofArtifactStore>
    PrivaiNode<S, V, A, SidecarProofVerifier>
{
    pub fn open_with_components_and_store(
        config: NodeConfig,
        store: S,
        verifier: V,
        artifact_store: A,
    ) -> Result<Self, NodeError> {
        Self::open_with_components(
            config,
            store,
            verifier,
            artifact_store,
            SidecarProofVerifier::default(),
        )
    }
}

impl<S: LedgerStore, V: ProofVerifier, A: ProofArtifactStore, P: BlockArtifactVerifier>
    PrivaiNode<S, V, A, P>
{
    pub fn open_with_components(
        config: NodeConfig,
        store: S,
        verifier: V,
        artifact_store: A,
        artifact_verifier: P,
    ) -> Result<Self, NodeError> {
        let ledger = Ledger::open(store, config.chain_id, verifier)?;
        Ok(Self {
            config,
            ledger,
            proof_artifacts: artifact_store,
            artifact_verifier,
            prevotes: HashMap::new(),
            precommits: HashMap::new(),
        })
    }

    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    pub fn ledger(&self) -> &Ledger<S, V> {
        &self.ledger
    }

    pub fn ledger_mut(&mut self) -> &mut Ledger<S, V> {
        &mut self.ledger
    }

    pub fn proof_artifact_store(&self) -> &A {
        &self.proof_artifacts
    }

    pub fn proof_artifact_store_mut(&mut self) -> &mut A {
        &mut self.proof_artifacts
    }

    pub fn artifact_verifier(&self) -> &P {
        &self.artifact_verifier
    }

    pub fn next_proposer(&self, epoch_seed_hash: &Hash32, round: u32) -> Option<Hash32> {
        select_proposer(&self.config.validators, epoch_seed_hash, round)
    }

    pub fn submit_transaction(
        &mut self,
        tx: Transaction,
        received_at_ms: u64,
    ) -> Result<Hash32, NodeError> {
        Ok(self.ledger.submit_transaction(tx, received_at_ms)?)
    }

    pub fn propose_block(
        &mut self,
        epoch: u64,
        round: u32,
        timestamp_ms: u64,
        epoch_seed_hash: Hash32,
        parent_qc_hash: Hash32,
        proof_certificates: Vec<ProofCertificate>,
        extra_receipts: Vec<ConsensusReceipt>,
    ) -> Result<Block, NodeError> {
        let proposer = self
            .next_proposer(&epoch_seed_hash, round)
            .ok_or(NodeError::NoValidators)?;
        if proposer != self.config.node_pk_hash {
            return Err(NodeError::NotProposer { round });
        }

        let txs = self.ledger.candidate_transactions(self.config.max_block_txs);
        let execution_mode = if txs.is_empty() {
            ExecutionMode::Housekeeping
        } else {
            ExecutionMode::FullBatchProof
        };
        let execution_bundle = build_execution_bundle_from_transactions(&txs, execution_mode)?;

        Ok(Block::from_template(BlockTemplate {
            chain_id: self.config.chain_id,
            height: self.ledger.snapshot().height + 1,
            epoch,
            round,
            timestamp_ms,
            prev_block_hash: self.ledger.snapshot().tip_hash,
            proposer_pk_hash: self.config.node_pk_hash,
            epoch_seed_hash,
            parent_qc_hash,
            txs,
            execution_bundle,
            proof_certificates,
            extra_receipts,
        }))
    }

    pub fn import_block(&mut self, block: &Block) -> Result<(), NodeError> {
        self.ledger
            .apply_block(block, self.config.epoch_params.min_proof_coverage)?;
        Ok(())
    }

    pub fn record_block_artifacts(
        &mut self,
        block: &Block,
        artifacts: &BlockProofArtifacts,
    ) -> Result<(), NodeError> {
        self.artifact_verifier.verify_block_artifacts(
            block,
            artifacts,
        )?;
        self.proof_artifacts.save_block(artifacts)?;
        Ok(())
    }

    pub fn load_block_artifacts(
        &self,
        block_hash: &Hash32,
    ) -> Result<Option<BlockProofArtifacts>, NodeError> {
        Ok(self.proof_artifacts.load_block(block_hash)?)
    }

    pub fn import_block_with_artifacts(
        &mut self,
        block: &Block,
        artifacts: Option<&BlockProofArtifacts>,
    ) -> Result<(), NodeError> {
        if let Some(artifacts) = artifacts {
            self.record_block_artifacts(block, artifacts)?;
        }
        self.import_block(block)
    }

    pub fn receive_vote(&mut self, vote: Vote) -> Option<QuorumCertificate> {
        let threshold = (self.config.validators.len() * 2) / 3 + 1; // Proste zalozenie > 2/3 wag lub validatorow

        match vote.vote_type {
            VoteType::Prevote => {
                let entry = self.prevotes.entry(vote.block_hash).or_insert_with(BTreeSet::new);
                entry.insert(vote.validator_pk.clone());
                
                // Gdy przebije threshold, walidator lokalny emituje / buduje QC
                if entry.len() >= threshold {
                    return Some(QuorumCertificate {
                        height: vote.height,
                        round: vote.round,
                        block_hash: vote.block_hash,
                        vote_type: VoteType::Prevote,
                        signers: entry.iter().cloned().collect(),
                        signatures: vec![], // Tu nalezaloby zaggregowac autentyczne sygnatury w warstwie sieci (tu dla v0 skip)
                    });
                }
            }
            VoteType::Precommit => {
                let entry = self.precommits.entry(vote.block_hash).or_insert_with(BTreeSet::new);
                entry.insert(vote.validator_pk.clone());

                if entry.len() >= threshold {
                    return Some(QuorumCertificate {
                        height: vote.height,
                        round: vote.round,
                        block_hash: vote.block_hash,
                        vote_type: VoteType::Precommit,
                        signers: entry.iter().cloned().collect(),
                        signatures: vec![],
                    });
                }
            }
        }
        None
    }

    /// Implementacja sciezki finalizujacej import - gdy uzyskamy QuorumCertificate typu PRECOMMIT, osiagamy pelne Finality.
    pub fn finalize_block_with_qc(
        &mut self,
        block: &Block,
        qc: &QuorumCertificate,
    ) -> Result<(), NodeError> {
        if qc.block_hash != block.hash() {
            return Err(NodeError::NotProposer { round: qc.round }); // Reuse error temporary for hash mismatch logic
        }
        if qc.vote_type != VoteType::Precommit {
            return Err(NodeError::NoValidators); // Tylko QC precommit daje finality
        }

        // Twarda autoryzacja stanu na blockchain, osiagnieto konsensus.
        self.import_block(block)?;

        // Wyczyść stany starych glosowan po zaimportowaniu
        self.prevotes.clear();
        self.precommits.clear();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use privai_chain::{
        merkle_root, Amount14, AuxWitness, CanonicalEncode, InputRef, LweCiphertext, OutputNote,
        RecipientBox, RecipientBoxPlaintext, SpendPolicy, Transaction, TransferNoteTx, TxCore,
        TX_TYPE_TRANSFER_NOTE, PRIVAI_V0,
    };
    use privai_ledger::MemoryStore;
    use privai_proof::{
        artifact::{BatchProofArtifact, BlockProofArtifacts},
        build_execution_bundle_from_transfer_proofs, ArtifactBackendError, BatchProofVerifierBackend,
        TransferInputWitness,
        TransferOutputWitness, TransferProvingData, TransferPublicInputs, TransferStatement,
        TransferWitness,
    };

    use super::*;

    fn sample_note(seed: u8) -> OutputNote {
        OutputNote::new(
            [seed; 32],
            LweCiphertext::default(),
            [seed.wrapping_add(1); 32],
            RecipientBox::new(
                vec![seed],
                [seed; 24],
                vec![seed.wrapping_add(1)],
                [seed; 16],
                [seed; 16],
            ),
        )
    }

    fn sample_transfer_and_proving(seed: u8) -> (Transaction, TransferProvingData) {
        let output = sample_note(seed);
        let statement = TransferStatement {
            input_note_commits: vec![[seed.wrapping_add(30); 32]],
            input_nullifiers: vec![privai_chain::Nullifier([seed.wrapping_add(40); 32])],
            output_note_commits: vec![output.note_commit],
            fee: seed as u64,
        };
        let tx = TransferNoteTx {
            core: TxCore {
                version: PRIVAI_V0,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: vec![InputRef {
                    note_commit: [seed.wrapping_add(30); 32],
                }],
                input_nullifiers: vec![privai_chain::Nullifier([seed.wrapping_add(40); 32])],
                outputs: vec![output.clone()],
                fee: seed as u64,
                statement_commit: statement.commitment(),
                auth: Vec::new(),
            },
        };
        let proving = TransferProvingData::from_tx_and_witness(
            &tx,
            TransferWitness {
                input: TransferInputWitness {
                    amount: Amount14::new(10).expect("amount"),
                    witness_seed: [1; 32],
                    nullifier_key: [2; 32],
                    spend_policy_opening: vec![3],
                    aux_opening: vec![4],
                },
                outputs: vec![TransferOutputWitness {
                    note_commit: output.note_commit,
                    recipient_opening: RecipientBoxPlaintext {
                        version: PRIVAI_V0,
                        bundle_id: output.recipient_box.hint,
                        note_payload_commit: output.payload_commit(),
                        amount: Amount14::new(10).expect("amount"),
                        witness_seed: [5; 32],
                        nullifier_key: [6; 32],
                        spend_policy_opening: SpendPolicy::Single {
                            falcon_pk_hash: [7; 32],
                        }
                        .to_canonical_bytes(),
                        aux_opening: AuxWitness {
                            version: PRIVAI_V0,
                            amount: Amount14::new(10).expect("amount"),
                            witness_seed: [5; 32],
                            noise_class: 1,
                            bundle_id: output.recipient_box.hint,
                        }
                        .to_canonical_bytes(),
                        sender_memo: None,
                    },
                }],
            },
        )
        .expect("proving");

        (Transaction::TransferNote(tx), proving)
    }

    fn sample_block_and_artifacts(seed: u8) -> (Block, BlockProofArtifacts) {
        let (tx, proving) = sample_transfer_and_proving(seed);
        let execution_bundle = build_execution_bundle_from_transfer_proofs(
            std::slice::from_ref(&proving),
            ExecutionMode::FullBatchProof,
        )
        .expect("bundle");
        let artifact = BatchProofArtifact {
            proof_system_id: 1u8,
            statement_root: merkle_root(execution_bundle.statement_commits.iter().copied()),
            public_inputs_root: execution_bundle.public_inputs_root,
            covered_tx_indexes: vec![0],
            proof_bytes: vec![1, 2, 3],
            prover_ids: vec![[8u8; 32]],
            proof_meta_hash: [9u8; 32],
        };
        let block = Block::from_template(BlockTemplate {
            chain_id: 17,
            height: 1,
            epoch: 0,
            round: 0,
            timestamp_ms: 1_000,
            prev_block_hash: [0; 32],
            proposer_pk_hash: [1; 32],
            epoch_seed_hash: [2; 32],
            parent_qc_hash: [3; 32],
            txs: vec![tx],
            execution_bundle: execution_bundle.clone(),
            proof_certificates: vec![artifact.certificate()],
            extra_receipts: Vec::new(),
        });
        let artifacts = BlockProofArtifacts::from_transfer_proofs(
            block.hash(),
            &[proving],
            vec![artifact],
        )
        .expect("artifacts");

        (block, artifacts)
    }

    #[test]
    fn proposer_builds_block_with_derived_public_inputs_root() {
        let config = NodeConfig::example();
        let mut node = PrivaiNode::open(config.clone(), MemoryStore::new()).expect("node");
        let tx = Transaction::TransferNote(TransferNoteTx {
            core: TxCore {
                version: 0,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: Vec::new(),
                input_nullifiers: Vec::new(),
                outputs: vec![sample_note(10)],
                fee: 3,
                statement_commit: [11; 32],
                auth: Vec::new(),
            },
        });

        node.submit_transaction(tx.clone(), 1_000).expect("submit");
        let block = node
            .propose_block(0, 0, 1_000, [2; 32], [3; 32], Vec::new(), Vec::new())
            .expect("propose");

        assert_eq!(block.body.execution_bundle.statement_commits, vec![tx.statement_commit()]);
        assert_eq!(block.body.execution_bundle.covered_tx_indexes, vec![0]);
        assert_eq!(
            block.body.execution_bundle.public_inputs_root,
            merkle_root([TransferPublicInputs::from_tx(match &tx {
                Transaction::TransferNote(tx) => tx,
                _ => unreachable!("transfer tx"),
            })
            .hash()])
        );
    }

    #[test]
    fn node_records_block_artifacts_sidecar() {
        let config = NodeConfig::example();
        let mut node = PrivaiNode::open(config.clone(), MemoryStore::new()).expect("node");
        let (block, artifacts) = sample_block_and_artifacts(10);

        node.record_block_artifacts(&block, &artifacts)
            .expect("record artifacts");
        assert_eq!(
            node.load_block_artifacts(&block.hash()).expect("load"),
            Some(artifacts)
        );
    }

    #[derive(Clone, Copy, Debug, Default)]
    struct RejectingBackend;

    impl BatchProofVerifierBackend for RejectingBackend {
        fn verify_batch_artifact(
            &self,
            _artifact: &BatchProofArtifact,
            _covered_entries: &[privai_proof::artifact::BlockProofEntry],
            _execution_bundle: &privai_chain::ExecutionBundle,
        ) -> Result<(), ArtifactBackendError> {
            Err(ArtifactBackendError::Rejected("forced rejection".into()))
        }
    }

    #[test]
    fn node_rejects_artifacts_when_backend_rejects_them() {
        let config = NodeConfig::example();
        let mut node = PrivaiNode::open_with_components(
            config.clone(),
            MemoryStore::new(),
            StructuralProofVerifier,
            privai_proof::store::MemoryProofArtifactStore::new(),
            SidecarProofVerifier::new(RejectingBackend),
        )
        .expect("node");
        let (block, artifacts) = sample_block_and_artifacts(10);

        assert!(matches!(
            node.record_block_artifacts(&block, &artifacts),
            Err(NodeError::ArtifactVerify(privai_proof::ArtifactVerificationError::Backend { .. }))
        ));
    }

    use crate::config::ValidatorConfig;

    #[test]
    fn node_builds_qc_on_threshold_votes() {
        let config = NodeConfig::example(); // Domyslnie example validator.len() wynosi np. 1. Dopiszemy ich tu wiecej
        let mut config_modified = config.clone();
        config_modified.validators = vec![
            ValidatorConfig { pk_hash: [1u8; 32], weight: 1 },
            ValidatorConfig { pk_hash: [2u8; 32], weight: 1 },
            ValidatorConfig { pk_hash: [3u8; 32], weight: 1 },
            ValidatorConfig { pk_hash: [4u8; 32], weight: 1 },
        ]; // 4 val

        let mut node = PrivaiNode::open(config_modified, MemoryStore::new()).expect("node");

        let hash = [9u8; 32];
        
        let vote1 = Vote { height: 1, round: 1, block_hash: hash, vote_type: VoteType::Precommit, validator_pk: vec![1], falcon_sig: vec![] };
        let vote2 = Vote { height: 1, round: 1, block_hash: hash, vote_type: VoteType::Precommit, validator_pk: vec![2], falcon_sig: vec![] };
        let vote3 = Vote { height: 1, round: 1, block_hash: hash, vote_type: VoteType::Precommit, validator_pk: vec![3], falcon_sig: vec![] };

        // 1szy i 2gi glos nic nie daja na QC
        assert!(node.receive_vote(vote1).is_none());
        assert!(node.receive_vote(vote2).is_none());
        
        // 3ci głos to quorum (3 to próg dla f=1 w n=4). Oddaje zbudowane QC:
        let qc_option = node.receive_vote(vote3);
        assert!(qc_option.is_some());
        
        let qc = qc_option.unwrap();
        assert_eq!(qc.vote_type, VoteType::Precommit);
        assert_eq!(qc.signers.len(), 3);
        
        // Finalizacja pojdzie po qc do importu ledgera!
        let (block, _) = sample_block_and_artifacts(10);
        let mut final_block = block.clone();
        final_block.header.height = 1;
        
        // To powinnosie wypalic ale z bledem NotProposer/Hash, bo hash() sztucznego QC sie nie zgadza z naszym dummy sample_block. 
        // Oznacz to po prostu wyjsciem None-err.
        let result = node.finalize_block_with_qc(&final_block, &qc);
        assert!(result.is_err());
    }
}
