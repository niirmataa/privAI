use privai_nxms::{
    ContextId, EscrowApprovalBody, EscrowFundedBody, EscrowFundingDescriptor, EscrowSpendProposal,
    EscrowSpendProposalBody, Hash32,
};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowStageError {
    AlreadyFunded,
    DuplicateApproval,
    ProposalWithoutFunding,
    ApprovalWithoutProposal,
    ApprovalMismatch,
    InvalidProposalHash, // when proposal_hash == tx_signing_hash
    ProposalConflict,
    StagedEscrowNotFound,
}

impl fmt::Display for EscrowStageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyFunded => write!(f, "escrow already funded"),
            Self::DuplicateApproval => write!(f, "duplicate approval"),
            Self::ProposalWithoutFunding => write!(f, "proposal without prior funding"),
            Self::ApprovalWithoutProposal => write!(f, "approval without prior proposal"),
            Self::ApprovalMismatch => write!(f, "approval session mismatch"),
            Self::InvalidProposalHash => {
                write!(f, "proposal_hash must differ from tx_signing_hash")
            }
            Self::ProposalConflict => write!(f, "proposal conflict"),
            Self::StagedEscrowNotFound => write!(f, "staged escrow not found"),
        }
    }
}

impl std::error::Error for EscrowStageError {}

#[derive(Debug, Clone)]
pub struct StagedEscrow {
    pub session_context: ContextId,
    pub escrow_id: Hash32,
    pub funding_tx_ref: Hash32,
    pub descriptor: EscrowFundingDescriptor,
}

#[derive(Debug, Clone)]
pub struct StagedProposal {
    pub session_context: ContextId,
    pub proposal: EscrowSpendProposal,
    pub approvals: HashMap<Hash32, EscrowApprovalBody>,
}

#[derive(Default)]
pub struct EscrowStageStore {
    // map escrow_id -> StagedEscrow
    pub funded_escrows: HashMap<Hash32, StagedEscrow>,
    // map proposal_hash -> StagedProposal
    pub proposals: HashMap<Hash32, StagedProposal>,
}

impl EscrowStageStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ingest_funded(&mut self, body: EscrowFundedBody) -> Result<(), EscrowStageError> {
        let escrow_id = body.descriptor.escrow_id;
        if let Some(existing) = self.funded_escrows.get(&escrow_id) {
            // idempotent check
            if existing.session_context == body.session_context
                && existing.funding_tx_ref == body.funding_tx_ref
                && existing.descriptor == body.descriptor
            {
                return Ok(());
            } else {
                return Err(EscrowStageError::AlreadyFunded); // Conflict
            }
        }

        self.funded_escrows.insert(
            escrow_id,
            StagedEscrow {
                session_context: body.session_context,
                escrow_id,
                funding_tx_ref: body.funding_tx_ref,
                descriptor: body.descriptor,
            },
        );
        Ok(())
    }

    pub fn ingest_proposal(
        &mut self,
        body: EscrowSpendProposalBody,
    ) -> Result<(), EscrowStageError> {
        let proposal = &body.proposal;

        // C. proposal_hash != tx_signing_hash
        if proposal.proposal_hash == proposal.tx_signing_hash {
            return Err(EscrowStageError::InvalidProposalHash);
        }

        // A. Funding first
        let funded = self
            .funded_escrows
            .get(&proposal.escrow_id)
            .ok_or(EscrowStageError::ProposalWithoutFunding)?;

        // B. Session / escrow consistency
        if funded.session_context != body.session_context {
            return Err(EscrowStageError::ProposalConflict);
        }

        if let Some(existing) = self.proposals.get(&proposal.proposal_hash) {
            // idempotent check
            if existing.session_context == body.session_context && existing.proposal == *proposal {
                return Ok(());
            } else {
                return Err(EscrowStageError::ProposalConflict);
            }
        }

        self.proposals.insert(
            proposal.proposal_hash,
            StagedProposal {
                session_context: body.session_context,
                proposal: proposal.clone(),
                approvals: HashMap::new(),
            },
        );
        Ok(())
    }

    pub fn ingest_approval(&mut self, body: EscrowApprovalBody) -> Result<(), EscrowStageError> {
        // B. Approval must belong to an existing proposal
        let staged_proposal = self
            .proposals
            .get_mut(&body.proposal_hash)
            .ok_or(EscrowStageError::ApprovalWithoutProposal)?;

        // B. Session consistency
        if staged_proposal.session_context != body.session_context {
            return Err(EscrowStageError::ApprovalMismatch);
        }

        // D. No duplicate approvals
        if staged_proposal.approvals.contains_key(&body.signer_pk) {
            return Err(EscrowStageError::DuplicateApproval);
        }

        staged_proposal.approvals.insert(body.signer_pk, body);
        Ok(())
    }

    pub fn is_quorum_ready(&self, proposal_hash: &Hash32) -> bool {
        if let Some(prop) = self.proposals.get(proposal_hash) {
            prop.approvals.len() >= 2
        } else {
            false
        }
    }

    pub fn get_ready_approvals(&self, proposal_hash: &Hash32) -> Option<Vec<EscrowApprovalBody>> {
        if self.is_quorum_ready(proposal_hash) {
            let prop = self.proposals.get(proposal_hash)?;
            // E. Return approvals in stable order (e.g. by signer_pk)
            let mut apps: Vec<_> = prop.approvals.values().cloned().collect();
            apps.sort_by_key(|a| a.signer_pk);
            Some(apps)
        } else {
            None
        }
    }

    pub fn get_staged_proposal(&self, proposal_hash: &Hash32) -> Option<&StagedProposal> {
        self.proposals.get(proposal_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_descriptor(escrow_id: Hash32) -> EscrowFundingDescriptor {
        EscrowFundingDescriptor {
            escrow_id,
            buyer_pk: [1; 32],
            merchant_pk: [2; 32],
            operator_pk: [3; 32],
            amount: 1000,
            spend_policy_commit: [4; 32],
            timeout_blocks: 100,
        }
    }

    #[test]
    fn test_funded_can_be_staged() {
        let mut store = EscrowStageStore::new();
        let body = EscrowFundedBody {
            session_context: [1; 16],
            descriptor: dummy_descriptor([10; 32]),
            funding_tx_ref: [20; 32],
        };

        assert_eq!(store.ingest_funded(body.clone()), Ok(()));
        // Idempotent re-ingest
        assert_eq!(store.ingest_funded(body.clone()), Ok(()));

        assert!(store.funded_escrows.contains_key(&[10; 32]));
    }

    #[test]
    fn test_proposal_before_funding_is_rejected() {
        let mut store = EscrowStageStore::new();
        let proposal_body = EscrowSpendProposalBody {
            session_context: [1; 16],
            proposal: EscrowSpendProposal {
                proposal_hash: [30; 32],
                escrow_id: [10; 32], // Not funded
                snapshot_hash: [40; 32],
                action: 0,
                tx_signing_hash: [50; 32],
            },
        };

        assert_eq!(
            store.ingest_proposal(proposal_body),
            Err(EscrowStageError::ProposalWithoutFunding)
        );
    }

    #[test]
    fn test_proposal_hash_equals_tx_signing_hash_is_rejected() {
        let mut store = EscrowStageStore::new();
        // Even if funded, this should fail early
        let proposal_body = EscrowSpendProposalBody {
            session_context: [1; 16],
            proposal: EscrowSpendProposal {
                proposal_hash: [30; 32],
                escrow_id: [10; 32],
                snapshot_hash: [40; 32],
                action: 0,
                tx_signing_hash: [30; 32], // SAME AS PROPOSAL HASH
            },
        };

        assert_eq!(
            store.ingest_proposal(proposal_body),
            Err(EscrowStageError::InvalidProposalHash)
        );
    }

    #[test]
    fn test_approval_before_proposal_is_rejected() {
        let mut store = EscrowStageStore::new();
        let approval = EscrowApprovalBody {
            session_context: [1; 16],
            proposal_hash: [30; 32],
            signer_pk: [1; 32],
            signature: vec![1, 2, 3],
        };

        assert_eq!(
            store.ingest_approval(approval),
            Err(EscrowStageError::ApprovalWithoutProposal)
        );
    }

    #[test]
    fn test_full_flow_and_duplicate_rejection() {
        let mut store = EscrowStageStore::new();

        let funded_body = EscrowFundedBody {
            session_context: [1; 16],
            descriptor: dummy_descriptor([10; 32]),
            funding_tx_ref: [20; 32],
        };
        store.ingest_funded(funded_body).unwrap();

        let proposal_body = EscrowSpendProposalBody {
            session_context: [1; 16],
            proposal: EscrowSpendProposal {
                proposal_hash: [30; 32],
                escrow_id: [10; 32],
                snapshot_hash: [40; 32],
                action: 0,
                tx_signing_hash: [50; 32],
            },
        };
        store.ingest_proposal(proposal_body.clone()).unwrap();
        // Idempotent re-ingest
        store.ingest_proposal(proposal_body).unwrap();

        assert!(!store.is_quorum_ready(&[30; 32]));

        let approval1 = EscrowApprovalBody {
            session_context: [1; 16],
            proposal_hash: [30; 32],
            signer_pk: [1; 32], // Buyer
            signature: vec![1, 2, 3],
        };
        store.ingest_approval(approval1.clone()).unwrap();

        assert!(!store.is_quorum_ready(&[30; 32]));

        // Duplicate
        assert_eq!(
            store.ingest_approval(approval1),
            Err(EscrowStageError::DuplicateApproval)
        );

        let approval2 = EscrowApprovalBody {
            session_context: [1; 16],
            proposal_hash: [30; 32],
            signer_pk: [2; 32], // Merchant
            signature: vec![4, 5, 6],
        };
        store.ingest_approval(approval2).unwrap();

        assert!(store.is_quorum_ready(&[30; 32]));

        let ready = store.get_ready_approvals(&[30; 32]).unwrap();
        assert_eq!(ready.len(), 2);
    }
}
