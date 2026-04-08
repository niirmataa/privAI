use crate::db::{OrchestratorDb, StepInput};
use crate::flow::WorkflowState;
use anyhow::{Result, anyhow};
use privai_nxms::{
    EscrowApprovalBody, EscrowApprovalBundle, EscrowFundedBody, EscrowSnapshot,
    EscrowSpendProposal, Hash32, hash32_with_domain,
};
use sha3::{Digest, Sha3_256};

/// Pseudo-Ledger interface to demonstrate that operator does not blindly trust the mailbox event.
pub trait LedgerObserver {
    /// Returns true if the note with the given commit exists on-chain and hasn't been spent.
    fn verify_funding_note_exists(&self, note_commit: &Hash32) -> Result<bool>;
}

/// Simulated context showing how an Operator acts as a deterministic state machine
/// rather than a subjective trust anchor.
pub struct OperatorWorkflow<'a, L: LedgerObserver> {
    pub db: &'a OrchestratorDb,
    pub ledger: L,
    pub operator_pk: Hash32,
}

impl<'a, L: LedgerObserver> OperatorWorkflow<'a, L> {
    pub fn new(db: &'a OrchestratorDb, ledger: L, operator_pk: Hash32) -> Self {
        Self { db, ledger, operator_pk }
    }

    /// Handles an incoming EscrowFunded event from the mailbox (from the Buyer).
    pub async fn handle_escrow_funded(
        &self,
        event: EscrowFundedBody,
    ) -> Result<()> {
        let escrow_id_hex = hex::encode(event.descriptor.escrow_id);

        // 1. Operator does not trust the event blindly. Queries the local/trusted ledger view.
        let note_commit = event
            .funding_tx_ref; // W uproszczeniu traktujemy to jako commit/referencję
        let is_valid_on_chain = self.ledger.verify_funding_note_exists(&note_commit)?;

        if !is_valid_on_chain {
            return Err(anyhow!(
                "Escrow funding note {} not found on-chain. Rejecting control-plane event.",
                hex::encode(note_commit)
            ));
        }

        // 2. We could verify descriptor parameters (amount matches on-chain note if visible, policy commit matches).
        // If everything is correct, transition control-plane state to Funded and build snapshot.

        let snapshot = EscrowSnapshot {
            escrow_id: event.descriptor.escrow_id,
            funding_descriptor: event.descriptor.clone(),
            funding_note_commit: Some(note_commit),
            status: 1, // e.g. 1 = Funded
        };

        // Create workflow if it doesn't exist, and immediately transition to Funded
        self.db
            .create_workflow(&escrow_id_hex, &serde_json::to_string(&snapshot)?, &[])
            .await?;
        
        self.db
            .transition_workflow(&escrow_id_hex, WorkflowState::Funded, Some("escrow_funded_event"))
            .await?;

        // 3. We record the step
        self.db
            .record_step(StepInput {
                escrow_id_hex: escrow_id_hex.clone(),
                state: WorkflowState::Funded,
                msg_type: "escrow_funded".to_string(),
                from_id: "buyer".to_string(),
                seq: 1,
                payload_hash_hex: "dummy_hash".to_string(),
            })
            .await?;

        Ok(())
    }

    /// When a release/refund is agreed upon or a timeout triggers recovery.
    /// The Operator deterministically decides the action based on business logic, 
    /// builds a Proposal, and signs it.
    pub async fn initiate_action(
        &self,
        escrow_id_hex: &str,
        action_type: u8,
        snapshot: &EscrowSnapshot,
    ) -> Result<(EscrowSpendProposal, EscrowApprovalBundle)> {
        // 1. Build the Proposal
        let mut hasher = Sha3_256::new();
        hasher.update(serde_json::to_vec(&snapshot)?);
        let mut snapshot_hash = [0u8; 32];
        snapshot_hash.copy_from_slice(&hasher.finalize());

        let tx_signing_hash = self.compute_tx_signing_hash(snapshot, action_type);

        // proposal_hash is the hash of the proposal fields to uniquely identify it in the control plane
        let mut action_bytes = [0u8; 32];
        action_bytes[0] = action_type;
        
        let proposal_hash = hash32_with_domain(b"privai:escrow:proposal", &[
            &snapshot.escrow_id,
            &snapshot_hash,
            &action_bytes,
            &tx_signing_hash,
        ]);

        let proposal = EscrowSpendProposal {
            proposal_hash,
            escrow_id: snapshot.escrow_id,
            snapshot_hash,
            action: action_type,
            tx_signing_hash,
        };

        // Transition state to TxSignPending
        self.db
            .transition_workflow(escrow_id_hex, WorkflowState::TxSignPending, Some("initiate_action"))
            .await?;

        // 2. Operator immediately approves it (signs `tx_signing_hash`) since it initiated the action
        let signature = self.sign_tx_hash(&tx_signing_hash);
        
        let bundle = EscrowApprovalBundle {
            proposal_hash: proposal.proposal_hash,
            tx_signing_hash: proposal.tx_signing_hash,
            signer_pks: vec![self.operator_pk],
            signatures: vec![signature],
        };

        // Transition state to TxSignedQuorum (Assuming we still need 1 more from Buyer/Merchant to complete the 2-of-3)
        // Wait, Operator provides 1 signature. It's a 2-of-3 policy, so we need 1 more.
        // The state remains TxSignPending until another party signs it.
        // For the sake of this mock, we just return it.

        Ok((proposal, bundle))
    }

    /// Process an approval from a Buyer or Merchant
    pub async fn handle_escrow_approval(
        &self,
        escrow_id_hex: &str,
        mut current_bundle: EscrowApprovalBundle,
        approval_event: EscrowApprovalBody,
    ) -> Result<EscrowApprovalBundle> {
        if current_bundle.proposal_hash != approval_event.proposal_hash {
            return Err(anyhow!("Proposal hash mismatch in approval"));
        }

        // Add signature to bundle
        current_bundle.signer_pks.push(approval_event.signer_pk);
        current_bundle.signatures.push(approval_event.signature);

        if current_bundle.signatures.len() >= 2 {
            // Quorum reached! Control plane state machine advances
            self.db
                .transition_workflow(
                    escrow_id_hex,
                    WorkflowState::TxSignedQuorum,
                    Some("quorum_reached"),
                )
                .await?;
        }

        Ok(current_bundle)
    }

    // --- Mock Cryptography & Helpers ---

    fn compute_tx_signing_hash(&self, snapshot: &EscrowSnapshot, action: u8) -> Hash32 {
        // In a real scenario, this hash is generated from the exact TransferNoteTx structure
        // that the Ledger will verify.
        let mut action_bytes = [0u8; 32];
        action_bytes[0] = action;
        
        hash32_with_domain(b"privai:escrow:tx_sign", &[
            &snapshot.escrow_id,
            &action_bytes,
            // inputs, outputs, fee, etc.
        ])
    }

    fn sign_tx_hash(&self, _tx_hash: &Hash32) -> Vec<u8> {
        // Mock Falcon/Ed25519 signature
        vec![0xAA; 64]
    }
}
