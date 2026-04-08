use thiserror::Error;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use privai_chain::{
    tx::MarketplaceBatchTx, Block, BlockTemplate, ConsensusReceipt, ExecutionMode, Hash32,
    ProofCertificate, QuorumCertificate, Transaction, ViewChange, Vote, VoteType,
};
use privai_ledger::{compute_state_root, Ledger, LedgerError, LedgerStore};
use privai_nxms::PrivaiBody;
use privai_proof::{
    artifact::BlockProofArtifacts,
    build_execution_bundle_from_transactions,
    store::{MemoryProofArtifactStore, ProofArtifactStore, ProofArtifactStoreError},
    BatchBuildError, BlockArtifactVerifier, ProofVerifier, SidecarProofVerifier,
    StructuralProofVerifier,
};

use crate::config::NodeConfig;
use crate::escrow_stage::{EscrowStageError, EscrowStageStore, StagedEscrow, StagedProposal};
use crate::proposer::select_proposer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowIngestOutcome {
    FundedStored,
    ProposalStored,
    ApprovalStored,
    Ignored,
}

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
    #[error("escrow ingest: {0}")]
    EscrowIngest(#[from] EscrowStageError),
    #[error("no validators are configured for proposer selection")]
    NoValidators,
    #[error("current node is not proposer for round {round}")]
    NotProposer { round: u32 },
    #[error("vote error: {0}")]
    VoteError(String),
    #[error("identity error: {0}")]
    IdentityError(String),
    #[error("QC hash mismatch: expected {expected:?}, got {actual:?}")]
    QcHashMismatch { expected: Hash32, actual: Hash32 },
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

    // Stake-weighted vote tracking: block_hash -> (pk→sig map, accumulated stake)
    // BTreeMap gwarantuje tę samą kolejność dla keys (signers) i values (signatures)
    prevotes: HashMap<Hash32, (BTreeMap<Vec<u8>, Vec<u8>>, u64)>,
    precommits: HashMap<Hash32, (BTreeMap<Vec<u8>, Vec<u8>>, u64)>,
    /// QC already emitted for (block_hash, vote_type) — prevents duplicate broadcasts
    qc_emitted: HashSet<(Hash32, u8)>,

    // PC-BFT Liveness & View Change
    pub current_round: u32,
    pub round_start_time_ms: u64,
    view_changes: HashMap<u32, BTreeSet<Vec<u8>>>,

    // Do generowania odpornych sygnatur (Non-Custodial / Anti-Forging)
    // Zeroizing chroni klucz tajny przed odczytem z dumpów pamięci
    falcon_sk: Option<zeroize::Zeroizing<Vec<u8>>>,

    // Escrow staging: control-plane store for funded/proposal/approval bodies
    escrow_store: EscrowStageStore,
}

impl<S: LedgerStore>
    PrivaiNode<S, StructuralProofVerifier, MemoryProofArtifactStore, SidecarProofVerifier>
{
    pub fn open(config: NodeConfig, store: S) -> Result<Self, NodeError> {
        Self::open_with_verifier(config, store, StructuralProofVerifier)
    }
}

impl<S: LedgerStore, A: ProofArtifactStore>
    PrivaiNode<S, StructuralProofVerifier, A, SidecarProofVerifier>
{
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

impl<S: LedgerStore, V: ProofVerifier>
    PrivaiNode<S, V, MemoryProofArtifactStore, SidecarProofVerifier>
{
    pub fn open_with_verifier(
        config: NodeConfig,
        store: S,
        verifier: V,
    ) -> Result<Self, NodeError> {
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

use crate::identity_provider::PQCIdentity;

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
        let current_round = ledger.snapshot().consensus_safety.current_round;
        Ok(Self {
            config,
            ledger,
            proof_artifacts: artifact_store,
            artifact_verifier,
            prevotes: HashMap::new(),
            precommits: HashMap::new(),
            qc_emitted: HashSet::new(),
            current_round,
            round_start_time_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            view_changes: HashMap::new(),
            falcon_sk: None,
            escrow_store: EscrowStageStore::new(),
        })
    }

    pub fn with_falcon_key(mut self, secret_key: Vec<u8>) -> Self {
        self.falcon_sk = Some(zeroize::Zeroizing::new(secret_key));
        self
    }

    /// Inicjalizacja wezla z uzyciem tozsamosci zewnetrznej C `nexum-cli`
    /// Podpina ona glowny klucz PQC do generowania Consensus Vote.
    pub fn load_identity(&mut self, vault_path: &std::path::Path) -> Result<(), NodeError> {
        let identity = PQCIdentity::load_from_vault(vault_path)?;

        if !identity.falcon_sk.is_empty() {
            self.falcon_sk = Some(identity.falcon_sk.clone());
            self.config.node_sig_sk = identity.falcon_sk.to_vec();
        }

        if !identity.falcon_pk.is_empty() {
            // Gdy mamy zdefiniowane falcon_pk hash klucza w configu zastepujemy wlasnym.
            self.config.node_pk_hash =
                privai_chain::hash::domain_hash("privai:falcon-pk:v0", &[&identity.falcon_pk]);
            self.config.node_sig_pk = identity.falcon_pk.clone();
        }

        if !identity.kem_pk.is_empty() {
            self.config.node_kem_pk = identity.kem_pk.clone();
        }

        if !identity.kem_sk.is_empty() {
            self.config.node_kem_sk = identity.kem_sk.to_vec();
        }

        Ok(())
    }

    /// Implementacja Operator-Consensus Binding
    /// Jako węzeł obsługujący rynek (MarketplaceOperator) ostatecznie podpisujemy
    /// wygenerowany przez portfel batch kluczem Falcon przed puszczeniem go do Mempoola.
    pub fn sign_marketplace_batch(
        &self,
        mut batch: MarketplaceBatchTx,
    ) -> Result<MarketplaceBatchTx, NodeError> {
        if let Some(sk) = &self.falcon_sk {
            // Uzywamy settlement_root jako kanonicznego obiektu reprezentujacego caly ten zbior drobnicy!
            let msg = batch.summary.settlement_root();
            let sig = nxms_transport::crypto::falcon_sign_ct_prepared(sk, &msg)
                .map_err(|e| NodeError::VoteError(e.to_string()))?;

            batch.operator_sig = sig;
            Ok(batch)
        } else {
            Err(NodeError::VoteError(
                "Brak klucza Operatora PQC (Falcon) w wezle!".into(),
            ))
        }
    }

    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    pub fn falcon_sk(&self) -> Option<&[u8]> {
        self.falcon_sk.as_deref().map(|v| &**v)
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

    /// Weryfikuje podpisy Falcon w transakcji (Zero Trust).
    /// Wrapper na Mempool::verify_tx_signatures dla użycia w consensus_loop.
    pub fn verify_tx_signatures(&self, tx: &Transaction) -> bool {
        crate::mempool::Mempool::verify_tx_signatures(tx)
    }

    pub fn create_vote_for_proposal(
        &mut self,
        proposal: &BlockTemplate,
    ) -> Result<Vote, NodeError> {
        let mut safety = self.ledger_mut().snapshot().consensus_safety.clone();

        if proposal.round < safety.current_view || proposal.round < safety.last_voted_view {
            return Err(NodeError::VoteError("Stary widok".into()));
        }

        if proposal.round == safety.last_voted_view && proposal.round != 0 {
            return Err(NodeError::VoteError(
                "Już głosowaliśmy w tej rundzie! Atak double-sign zablokowany.".into(),
            ));
        }

        // KLUCZOWY MOMENT: Najpierw zapis na dysk, potem wysylka do sieci
        safety.last_voted_view = proposal.round;
        // Zapis stanu na dysk atomowo. Ledger flush() od razu rzuci IO Err jesli padnie (zapobiega crash-vulnerability)
        self.ledger_mut().update_consensus_safety(safety)?;

        let block = Block::from_template(proposal.clone());

        let mut falcon_sig = vec![];
        if let Some(sk) = &self.falcon_sk {
            if let Ok(sig) = nxms_transport::crypto::falcon_sign_ct_prepared(sk, &block.hash()) {
                falcon_sig = sig;
            }
        }

        Ok(Vote {
            height: block.header.height,
            round: block.header.round,
            block_hash: block.hash(),
            vote_type: VoteType::Prevote, // Zaczynamy od Prevote w PC-BFT
            validator_pk: self.config.node_sig_pk.clone(), // Pełny Falcon PK — odbiorca może zweryfikować podpis bez lookupu
            falcon_sig,
        })
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

        let txs = self
            .ledger
            .candidate_transactions(self.config.max_block_txs);
        let execution_mode = if txs.is_empty() {
            ExecutionMode::Housekeeping
        } else {
            ExecutionMode::FullBatchProof
        };
        let execution_bundle = build_execution_bundle_from_transactions(&txs, execution_mode)?;

        // Oblicz state_root: symuluj zastosowanie transakcji i oblicz hash wynikowego stanu
        let mut temp_snapshot = self.ledger().snapshot().clone();
        for tx in &txs {
            privai_ledger::apply_transaction_local(
                tx,
                self.ledger.snapshot().height + 1,
                &mut temp_snapshot,
            );
        }
        let state_root = compute_state_root(&temp_snapshot);

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
            state_root,
            txs,
            execution_bundle,
            proof_certificates,
            extra_receipts,
        }))
    }

    pub fn import_block(&mut self, block: &Block) -> Result<(), NodeError> {
        self.ledger.apply_block(
            block,
            self.config.epoch_params.min_proof_coverage,
            &self.config.epoch_params,
        )?;
        Ok(())
    }

    pub fn record_block_artifacts(
        &mut self,
        block: &Block,
        artifacts: &BlockProofArtifacts,
    ) -> Result<(), NodeError> {
        self.artifact_verifier
            .verify_block_artifacts(block, artifacts)?;
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

    // ── Escrow body ingest ──────────────────────────────────────────────

    pub fn handle_privai_body(
        &mut self,
        body: PrivaiBody,
    ) -> Result<EscrowIngestOutcome, NodeError> {
        match body {
            PrivaiBody::EscrowFunded(funded) => {
                self.escrow_store
                    .ingest_funded(funded)
                    .map_err(|e| NodeError::EscrowIngest(e))?;
                Ok(EscrowIngestOutcome::FundedStored)
            }
            PrivaiBody::EscrowSpendProposal(proposal) => {
                self.escrow_store
                    .ingest_proposal(proposal)
                    .map_err(|e| NodeError::EscrowIngest(e))?;
                Ok(EscrowIngestOutcome::ProposalStored)
            }
            PrivaiBody::EscrowApproval(approval) => {
                self.escrow_store
                    .ingest_approval(approval)
                    .map_err(|e| NodeError::EscrowIngest(e))?;
                Ok(EscrowIngestOutcome::ApprovalStored)
            }
            _ => Ok(EscrowIngestOutcome::Ignored),
        }
    }

    pub fn is_escrow_quorum_ready(&self, proposal_hash: &Hash32) -> bool {
        self.escrow_store.is_quorum_ready(proposal_hash)
    }

    pub fn get_escrow_ready_approvals(
        &self,
        proposal_hash: &Hash32,
    ) -> Option<Vec<privai_nxms::EscrowApprovalBody>> {
        self.escrow_store.get_ready_approvals(proposal_hash)
    }

    pub fn get_staged_escrow(&self, escrow_id: &Hash32) -> Option<&StagedEscrow> {
        self.escrow_store.funded_escrows.get(escrow_id)
    }

    pub fn get_staged_proposal(&self, proposal_hash: &Hash32) -> Option<&StagedProposal> {
        self.escrow_store.get_staged_proposal(proposal_hash)
    }

    pub fn escrow_store(&self) -> &EscrowStageStore {
        &self.escrow_store
    }

    pub fn escrow_store_mut(&mut self) -> &mut EscrowStageStore {
        &mut self.escrow_store
    }

    /// PHASE 7 LIVENESS: Sprawdza timeout używając domyślnej wartości z configu (30s dla Tor).
    /// Wygodniejsza wersja do użycia w głównym loopie.
    pub fn check_timeout_default(&mut self, current_time_ms: u64) -> Option<ViewChange> {
        self.check_timeout(current_time_ms, self.config.consensus_timeout_ms)
    }

    /// PHASE 7 LIVENESS: Wywolywana przez wbudowany timer/scheduler.
    /// Gdy przekroczy dozwolony limit - węzeł wysyła swoj glos ViewChange w obieg.
    pub fn check_timeout(
        &mut self,
        current_time_ms: u64,
        timeout_limit_ms: u64,
    ) -> Option<ViewChange> {
        if current_time_ms.saturating_sub(self.round_start_time_ms) > timeout_limit_ms {
            let next_round = self.current_round + 1;

            let mut falcon_sig = vec![];
            if let Some(sk) = &self.falcon_sk {
                let msg = privai_chain::hash::domain_hash(
                    "privai:view-change:v0",
                    &[&next_round.to_le_bytes()],
                );
                if let Ok(sig) = nxms_transport::crypto::falcon_sign_ct_prepared(sk, &msg) {
                    falcon_sig = sig;
                }
            }

            let vc = ViewChange {
                height: self.ledger.snapshot().height + 1,
                new_round: next_round,
                validator_pk: self.config.node_pk_hash.to_vec(),
                falcon_sig,
            };

            // Reset timer — zapobiega spamowi ViewChange co 1s
            self.round_start_time_ms = current_time_ms;

            return Some(vc);
        }
        None
    }

    /// PHASE 7 LIVENESS: Inne wezly przysylaja nam wiadomosc ViewChange.
    /// Jezeli zdobedziemy 2/3 w danej rundzie, Node podbija runde by zapobiec zamrozeniu (wybrac nowego Proposera).
    pub fn receive_view_change(&mut self, vc: ViewChange, current_time_ms: u64) -> bool {
        // 1. Weryfikacja podpisu Falcon na ViewChange
        if vc.falcon_sig.is_empty() || vc.validator_pk.is_empty() {
            eprintln!("[node] rejected ViewChange: missing Falcon signature (mandatory)");
            return false;
        }
        let vc_msg = privai_chain::hash::domain_hash(
            "privai:view-change:v0",
            &[&vc.new_round.to_le_bytes()],
        );
        if nxms_transport::crypto::falcon_verify(&vc.validator_pk, &vc_msg, &vc.falcon_sig).is_err()
        {
            eprintln!("[node] rejected ViewChange: invalid Falcon signature");
            return false;
        }

        // 2. Weryfikacja że głosujący jest znanym validatorem
        let voter_pk_hash =
            privai_chain::hash::domain_hash("privai:falcon-pk:v0", &[&vc.validator_pk]);
        if !self
            .config
            .validators
            .iter()
            .any(|v| v.pk_hash == voter_pk_hash)
        {
            eprintln!("[node] rejected ViewChange from unknown validator");
            return false;
        }

        // Stake-weighted threshold (tak jak w receive_vote)
        let total_stake: u64 = self.config.validators.iter().map(|v| v.stake_weight).sum();
        let required_stake = (total_stake * 2) / 3 + 1;

        let entry = self
            .view_changes
            .entry(vc.new_round)
            .or_insert_with(BTreeSet::new);
        entry.insert(vc.validator_pk.clone());

        // Oblicz accumulated stake dla view_changes
        let accumulated_stake: u64 = entry
            .iter()
            .filter_map(|pk| {
                self.config.validators.iter().find(|v| {
                    v.pk_hash == privai_chain::hash::domain_hash("privai:falcon-pk:v0", &[pk])
                })
            })
            .map(|v| v.stake_weight)
            .sum();

        if accumulated_stake >= required_stake && self.current_round < vc.new_round {
            // Osiagnelismy Quorum na wejscie w nowa runde! Lider "zostal obalony".
            self.current_round = vc.new_round;
            self.round_start_time_ms = current_time_ms; // Reset timera

            // Persystuj current_round na dysk (zapobiega equivocation po restarcie)
            let mut safety = self.ledger.snapshot().consensus_safety.clone();
            safety.current_round = vc.new_round;
            if let Err(e) = self.ledger_mut().update_consensus_safety(safety) {
                eprintln!("[node] WARNING: failed to persist current_round: {}", e);
            }

            // Wyczysc stare stany, gotowi do nowej rundy.
            self.prevotes.clear();
            self.precommits.clear();
            self.qc_emitted.clear();
            // Usuń stare wpisy ViewChange (rundy < nowej) — zapobiega OOM
            self.view_changes.retain(|&round, _| round >= vc.new_round);
            return true;
        }

        false
    }

    pub fn receive_vote(&mut self, vote: Vote) -> Option<QuorumCertificate> {
        // 1. Weryfikuj że głosujący jest znanym validatorem (anti-sybil)
        let voter_pk_hash =
            privai_chain::hash::domain_hash("privai:falcon-pk:v0", &[&vote.validator_pk]);
        let voter_stake = match self
            .config
            .validators
            .iter()
            .find(|v| v.pk_hash == voter_pk_hash)
        {
            Some(v) => v.stake_weight,
            None => {
                eprintln!(
                    "[node] rejected vote from unknown validator {:?}",
                    &voter_pk_hash[..8]
                );
                return None;
            }
        };

        // 2. Obowiązkowa weryfikacja podpisu Falcon (anti-forging)
        if vote.falcon_sig.is_empty() || vote.validator_pk.is_empty() {
            eprintln!("[node] rejected vote: missing Falcon signature (mandatory)");
            return None;
        }
        if nxms_transport::crypto::falcon_verify(
            &vote.validator_pk,
            &vote.block_hash,
            &vote.falcon_sig,
        )
        .is_err()
        {
            eprintln!("[node] rejected vote: invalid Falcon signature");
            return None;
        }

        // 3. Sprawdź czy QC już wyemitowany (deduplikacja)
        let qc_key = (vote.block_hash, vote.vote_type as u8);
        if self.qc_emitted.contains(&qc_key) {
            return None;
        }

        // 4. Stake-weighted threshold: >= 2/3 total stake
        let total_stake: u64 = self.config.validators.iter().map(|v| v.stake_weight).sum();
        let required_stake = (total_stake * 2) / 3 + 1;

        match vote.vote_type {
            VoteType::Prevote => {
                let entry = self
                    .prevotes
                    .entry(vote.block_hash)
                    .or_insert_with(|| (BTreeMap::new(), 0));
                if !entry.0.contains_key(&vote.validator_pk) {
                    entry
                        .0
                        .insert(vote.validator_pk.clone(), vote.falcon_sig.clone());
                    entry.1 += voter_stake;
                }

                if entry.1 >= required_stake {
                    self.qc_emitted.insert(qc_key);
                    return Some(QuorumCertificate {
                        height: vote.height,
                        round: vote.round,
                        block_hash: vote.block_hash,
                        vote_type: VoteType::Prevote,
                        signers: entry.0.keys().cloned().collect(),
                        signatures: entry.0.values().cloned().collect(),
                    });
                }
            }
            VoteType::Precommit => {
                let entry = self
                    .precommits
                    .entry(vote.block_hash)
                    .or_insert_with(|| (BTreeMap::new(), 0));
                if !entry.0.contains_key(&vote.validator_pk) {
                    entry
                        .0
                        .insert(vote.validator_pk.clone(), vote.falcon_sig.clone());
                    entry.1 += voter_stake;
                }

                if entry.1 >= required_stake {
                    self.qc_emitted.insert(qc_key);
                    return Some(QuorumCertificate {
                        height: vote.height,
                        round: vote.round,
                        block_hash: vote.block_hash,
                        vote_type: VoteType::Precommit,
                        signers: entry.0.keys().cloned().collect(),
                        signatures: entry.0.values().cloned().collect(),
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
            return Err(NodeError::QcHashMismatch {
                expected: block.hash(),
                actual: qc.block_hash,
            });
        }
        if qc.vote_type != VoteType::Precommit {
            return Err(NodeError::VoteError(
                "Only Precommit QC gives finality".into(),
            ));
        }

        // Blok powinien być już zaimportowany przy Proposal.
        // Jeśli nie (np. state sync) — importuj teraz.
        if self.ledger.snapshot().tip_hash != block.hash() {
            self.import_block(block)?;
        }

        // Persystuj QC do ledgera (potrzebne do state sync)
        self.ledger_mut()
            .snapshot_mut()
            .qcs
            .insert(block.header.height, qc.clone());
        if let Err(e) = self.ledger_mut().flush() {
            eprintln!("[node] WARNING: failed to persist QC: {}", e);
        }

        // Wyczyść stany starych głosowań
        self.prevotes.clear();
        self.precommits.clear();
        self.qc_emitted.clear();
        // Usuń stare wpisy ViewChange — zapobiega OOM przy długim działaniu
        let current_round = self.current_round;
        self.view_changes.retain(|&round, _| round > current_round);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use privai_chain::{
        merkle_root, Amount14, AuxWitness, CanonicalEncode, InputRef, LweCiphertext, OutputNote,
        RecipientBox, RecipientBoxPlaintext, SpendPolicy, Transaction, TransferNoteTx, TxCore,
        PRIVAI_V0, TX_TYPE_TRANSFER_NOTE,
    };
    use privai_ledger::MemoryStore;
    use privai_proof::{
        artifact::{BatchProofArtifact, BlockProofArtifacts},
        build_execution_bundle_from_transfer_proofs, ArtifactBackendError,
        BatchProofVerifierBackend, TransferInputWitness, TransferOutputWitness,
        TransferProvingData, TransferPublicInputs, TransferStatement, TransferWitness,
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
            state_root: [0; 32],
            txs: vec![tx],
            execution_bundle: execution_bundle.clone(),
            proof_certificates: vec![artifact.certificate()],
            extra_receipts: Vec::new(),
        });
        let artifacts =
            BlockProofArtifacts::from_transfer_proofs(block.hash(), &[proving], vec![artifact])
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
                // W testach v0 uzywamy pustego auth by zignorowac weryfikacje Falcon
                auth: Vec::new(),
            },
        });

        node.submit_transaction(tx.clone(), 1_000).expect("submit");
        let block = node
            .propose_block(0, 0, 1_000, [2; 32], [3; 32], Vec::new(), Vec::new())
            .expect("propose");

        assert_eq!(
            block.body.execution_bundle.statement_commits,
            vec![tx.statement_commit()]
        );
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
            Err(NodeError::ArtifactVerify(
                privai_proof::ArtifactVerificationError::Backend { .. }
            ))
        ));
    }

    use crate::config::ValidatorConfig;

    #[test]
    fn node_triggers_view_change_on_timeout() {
        // Generuj klucze Falcon dla 3 validatorów
        let (sk1, pk1) = nxms_transport::crypto::falcon_keygen().expect("keygen");
        let (sk2, pk2) = nxms_transport::crypto::falcon_keygen().expect("keygen");
        let (sk3, pk3) = nxms_transport::crypto::falcon_keygen().expect("keygen");

        let config = NodeConfig::example();
        let mut config_modified = config.clone();
        config_modified.validators = vec![
            ValidatorConfig {
                pk_hash: test_pk_hash(&pk1),
                sig_pk: pk1.clone(),
                stake_weight: 1,
                availability: 100,
                proof_score: 100,
            },
            ValidatorConfig {
                pk_hash: test_pk_hash(&pk2),
                sig_pk: pk2.clone(),
                stake_weight: 1,
                availability: 100,
                proof_score: 100,
            },
            ValidatorConfig {
                pk_hash: test_pk_hash(&pk3),
                sig_pk: pk3.clone(),
                stake_weight: 1,
                availability: 100,
                proof_score: 100,
            },
        ];
        config_modified.node_pk_hash = test_pk_hash(&pk1);

        let mut node = PrivaiNode::open(config_modified, MemoryStore::new()).expect("node");
        node = node.with_falcon_key(sk1.to_vec());
        node.current_round = 0;
        node.round_start_time_ms = 1000;

        // Nie minął czas — brak ViewChange
        assert!(node.check_timeout(1000 + 4999, 5000).is_none());

        // Przekroczono limit — tworzy ViewChange dla rundy 1
        let vc = node.check_timeout(1000 + 5001, 5000).unwrap();
        assert_eq!(vc.new_round, 1);

        let view_change_msg = privai_chain::hash::domain_hash(
            "privai:view-change:v0",
            &[&vc.new_round.to_le_bytes()],
        );

        // Podpisz każdym kluczem swój własny ViewChange
        // Validator 1: vc już ma podpis od node (validator_pk = node_pk_hash = hash(pk1))
        // Musimy go podpisać raw pk1 żeby verification przeszła
        let mut vc1 = vc.clone();
        vc1.validator_pk = pk1.clone();
        vc1.falcon_sig =
            nxms_transport::crypto::falcon_sign_ct_prepared(&sk1, &view_change_msg).expect("sign");
        assert!(!node.receive_view_change(vc1, 6001)); // 1 głos, za mało

        // Validator 2: podpisuje swoim sk2
        let mut vc2 = vc.clone();
        vc2.validator_pk = pk2.clone();
        vc2.falcon_sig =
            nxms_transport::crypto::falcon_sign_ct_prepared(&sk2, &view_change_msg).expect("sign");
        assert!(!node.receive_view_change(vc2, 6001)); // 2 głosy, za mało

        // Validator 3: podpisuje swoim sk3
        let mut vc3 = vc.clone();
        vc3.validator_pk = pk3.clone();
        vc3.falcon_sig =
            nxms_transport::crypto::falcon_sign_ct_prepared(&sk3, &view_change_msg).expect("sign");
        assert!(node.receive_view_change(vc3, 6001)); // 3 głosy — threshold osiągnięty!

        assert_eq!(node.current_round, 1);
        assert_eq!(node.round_start_time_ms, 6001);
    }

    /// Helper: derive pk_hash from a fake "public key" the same way receive_vote does
    fn test_pk_hash(pk: &[u8]) -> Hash32 {
        privai_chain::hash::domain_hash("privai:falcon-pk:v0", &[pk])
    }

    #[test]
    fn node_builds_qc_on_threshold_votes() {
        // Generuj klucze Falcon dla 4 validatorów
        let (sk1, pk1) = nxms_transport::crypto::falcon_keygen().expect("keygen");
        let (sk2, pk2) = nxms_transport::crypto::falcon_keygen().expect("keygen");
        let (sk3, pk3) = nxms_transport::crypto::falcon_keygen().expect("keygen");
        let (_sk4, pk4) = nxms_transport::crypto::falcon_keygen().expect("keygen");

        let config = NodeConfig::example();
        let mut config_modified = config.clone();
        config_modified.validators = vec![
            ValidatorConfig {
                pk_hash: test_pk_hash(&pk1),
                sig_pk: pk1.clone(),
                stake_weight: 1,
                availability: 100,
                proof_score: 100,
            },
            ValidatorConfig {
                pk_hash: test_pk_hash(&pk2),
                sig_pk: pk2.clone(),
                stake_weight: 1,
                availability: 100,
                proof_score: 100,
            },
            ValidatorConfig {
                pk_hash: test_pk_hash(&pk3),
                sig_pk: pk3.clone(),
                stake_weight: 1,
                availability: 100,
                proof_score: 100,
            },
            ValidatorConfig {
                pk_hash: test_pk_hash(&pk4),
                sig_pk: pk4.clone(),
                stake_weight: 1,
                availability: 100,
                proof_score: 100,
            },
        ]; // 4 val, equal stake — threshold = 4*2/3+1 = 3 stake

        let mut node = PrivaiNode::open(config_modified, MemoryStore::new()).expect("node");

        let hash = [9u8; 32];

        // Podpisz każdy głos swoim kluczem
        let sig1 = nxms_transport::crypto::falcon_sign_ct_prepared(&sk1, &hash).expect("sign");
        let sig2 = nxms_transport::crypto::falcon_sign_ct_prepared(&sk2, &hash).expect("sign");
        let sig3 = nxms_transport::crypto::falcon_sign_ct_prepared(&sk3, &hash).expect("sign");

        let vote1 = Vote {
            height: 1,
            round: 1,
            block_hash: hash,
            vote_type: VoteType::Precommit,
            validator_pk: pk1.clone(),
            falcon_sig: sig1,
        };
        let vote2 = Vote {
            height: 1,
            round: 1,
            block_hash: hash,
            vote_type: VoteType::Precommit,
            validator_pk: pk2.clone(),
            falcon_sig: sig2,
        };
        let vote3 = Vote {
            height: 1,
            round: 1,
            block_hash: hash,
            vote_type: VoteType::Precommit,
            validator_pk: pk3.clone(),
            falcon_sig: sig3,
        };

        // 1szy i 2gi glos nic nie daja na QC (stake 1+1 = 2 < 3)
        assert!(node.receive_vote(vote1).is_none());
        assert!(node.receive_vote(vote2).is_none());

        // 3ci głos: stake = 3 >= required 3. QC!
        let qc_option = node.receive_vote(vote3);
        assert!(qc_option.is_some());

        let qc = qc_option.unwrap();
        assert_eq!(qc.vote_type, VoteType::Precommit);
        assert_eq!(qc.signers.len(), 3);

        // QC dedup: 4ty głos nie buduje drugiego QC
        let sig4 = nxms_transport::crypto::falcon_sign_ct_prepared(&sk1, &hash).expect("sign");
        let vote4 = Vote {
            height: 1,
            round: 1,
            block_hash: hash,
            vote_type: VoteType::Precommit,
            validator_pk: pk4,
            falcon_sig: sig4,
        };
        assert!(node.receive_vote(vote4).is_none());

        // Finalizacja z niepasującym blokiem → QcHashMismatch
        let (block, _) = sample_block_and_artifacts(10);
        let result = node.finalize_block_with_qc(&block, &qc);
        assert!(matches!(result, Err(NodeError::QcHashMismatch { .. })));
    }

    #[test]
    fn qc_signers_and_signatures_are_aligned() {
        use nxms_transport::crypto::{falcon_keygen, falcon_sign_ct_prepared};

        // Utwórz 3 walidatorów z różnymi PK
        let (sk1, pk1) = falcon_keygen().expect("keygen 1");
        let (sk2, pk2) = falcon_keygen().expect("keygen 2");
        let (sk3, pk3) = falcon_keygen().expect("keygen 3");

        fn test_pk_hash(pk: &[u8]) -> Hash32 {
            privai_chain::hash::domain_hash("privai:falcon-pk:v0", &[pk])
        }

        let config = NodeConfig::example();
        let mut config_modified = config.clone();
        config_modified.validators = vec![
            ValidatorConfig {
                pk_hash: test_pk_hash(&pk1),
                sig_pk: pk1.clone(),
                stake_weight: 1,
                availability: 100,
                proof_score: 100,
            },
            ValidatorConfig {
                pk_hash: test_pk_hash(&pk2),
                sig_pk: pk2.clone(),
                stake_weight: 1,
                availability: 100,
                proof_score: 100,
            },
            ValidatorConfig {
                pk_hash: test_pk_hash(&pk3),
                sig_pk: pk3.clone(),
                stake_weight: 1,
                availability: 100,
                proof_score: 100,
            },
        ];

        let mut node = PrivaiNode::open(config_modified, MemoryStore::new()).expect("node");

        let hash = [0xAA; 32];

        // Wyślij głosy w ODWROTNEJ kolejności sortowania PK (pk3, pk1, pk2)
        // BTreeMap posortuje je automatycznie (pk1, pk2, pk3)
        let sig3 = falcon_sign_ct_prepared(&sk3, &hash).expect("sign 3");
        let sig1 = falcon_sign_ct_prepared(&sk1, &hash).expect("sign 1");
        let sig2 = falcon_sign_ct_prepared(&sk2, &hash).expect("sign 2");

        let vote3 = Vote {
            height: 1,
            round: 1,
            block_hash: hash,
            vote_type: VoteType::Prevote,
            validator_pk: pk3.clone(),
            falcon_sig: sig3,
        };
        let vote1 = Vote {
            height: 1,
            round: 1,
            block_hash: hash,
            vote_type: VoteType::Prevote,
            validator_pk: pk1.clone(),
            falcon_sig: sig1,
        };
        let vote2 = Vote {
            height: 1,
            round: 1,
            block_hash: hash,
            vote_type: VoteType::Prevote,
            validator_pk: pk2.clone(),
            falcon_sig: sig2,
        };

        assert!(node.receive_vote(vote3).is_none());
        assert!(node.receive_vote(vote1).is_none());
        let qc = node.receive_vote(vote2).expect("should emit QC");

        // Weryfikacja: signers[i] i signatures[i] muszą pasować do tego samego walidatora
        assert_eq!(qc.signers.len(), 3);
        assert_eq!(qc.signatures.len(), 3);

        for (i, (signer, sig)) in qc.signers.iter().zip(qc.signatures.iter()).enumerate() {
            // Każdy signer musi być prawdziwym PK (nie hash)
            assert!(
                signer.len() > 32,
                "signers[{}] should be full Falcon PK, not hash",
                i
            );

            // Sprawdź czy podpis weryfikuje się z tym właśnie signerem
            let verify_result = nxms_transport::crypto::falcon_verify(signer, &hash, sig);
            assert!(
                verify_result.is_ok(),
                "signers[{}] and signatures[{}] mismatch: sig doesn't verify against signer",
                i,
                i
            );
        }
    }
}
