use privai_nxms::{
    ContextId, EscrowApprovalBody, EscrowFundedBody, EscrowFundingDescriptor, EscrowSpendProposal,
    EscrowSpendProposalBody, Hash32,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowStageError {
    AlreadyFunded,
    DuplicateApproval,
    ProposalWithoutFunding,
    ApprovalWithoutProposal,
    ApprovalMismatch,
    ProposalConflict,
    StagedEscrowNotFound,
    /// Persistence I/O or serialization error.
    Persistence(String),
}

impl fmt::Display for EscrowStageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyFunded => write!(f, "escrow already funded"),
            Self::DuplicateApproval => write!(f, "duplicate approval"),
            Self::ProposalWithoutFunding => write!(f, "proposal without prior funding"),
            Self::ApprovalWithoutProposal => write!(f, "approval without prior proposal"),
            Self::ApprovalMismatch => write!(f, "approval session mismatch"),
            Self::ProposalConflict => write!(f, "proposal conflict"),
            Self::StagedEscrowNotFound => write!(f, "staged escrow not found"),
            Self::Persistence(msg) => write!(f, "escrow stage persistence: {msg}"),
        }
    }
}

impl std::error::Error for EscrowStageError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedEscrow {
    pub session_context: ContextId,
    pub escrow_id: Hash32,
    pub funding_tx_ref: Hash32,
    pub descriptor: EscrowFundingDescriptor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedProposal {
    pub session_context: ContextId,
    pub proposal: EscrowSpendProposal,
    /// signer_pk -> approval body
    pub approvals: HashMap<Hash32, EscrowApprovalBody>,
}

/// Serializable mirror of `StagedProposal` — uses Vec tuples instead of HashMap
/// for JSON compatibility (`[u8; 32]` cannot be a JSON map key).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotStagedProposal {
    pub session_context: ContextId,
    pub proposal: EscrowSpendProposal,
    /// (hex_signer_pk, approval) tuples
    pub approvals: Vec<(String, EscrowApprovalBody)>,
}

impl SnapshotStagedProposal {
    pub fn from_staged(p: &StagedProposal) -> Self {
        Self {
            session_context: p.session_context,
            proposal: p.proposal.clone(),
            approvals: p
                .approvals
                .iter()
                .map(|(k, v)| (hash32_to_hex(k), v.clone()))
                .collect(),
        }
    }

    pub fn into_staged(
        self,
    ) -> Result<
        (
            EscrowSpendProposal,
            ContextId,
            HashMap<Hash32, EscrowApprovalBody>,
        ),
        EscrowStageError,
    > {
        let approvals: HashMap<Hash32, EscrowApprovalBody> = self
            .approvals
            .into_iter()
            .map(|(k, v)| hex_to_hash32(&k).map(|key| (key, v)))
            .collect::<Result<_, _>>()?;
        Ok((self.proposal, self.session_context, approvals))
    }
}

/// Serializable snapshot of the entire escrow staging state.
///
/// This is the on-disk format written to `escrow_stage.json` inside the node's
/// `data_dir`. It captures funded escrows, proposals and their approvals so that
/// the node can restore Stage A state after a restart.
///
/// Uses Vec of (hex_key, value) tuples because `[u8; 32]` HashMap keys cannot
/// be serialized directly to JSON (JSON requires string keys).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EscrowStageSnapshot {
    pub funded_escrows: Vec<(String, StagedEscrow)>,
    pub proposals: Vec<(String, SnapshotStagedProposal)>,
}

fn hash32_to_hex(h: &Hash32) -> String {
    hex::encode(h)
}

fn hex_to_hash32(s: &str) -> Result<Hash32, EscrowStageError> {
    let bytes = hex::decode(s)
        .map_err(|e| EscrowStageError::Persistence(format!("invalid hex key '{s}': {e}")))?;
    if bytes.len() != 32 {
        return Err(EscrowStageError::Persistence(format!(
            "hex key '{s}' decoded to {} bytes, expected 32",
            bytes.len()
        )));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
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

    /// Returns approvals in deterministic (ascending signer_pk) order once quorum is met.
    ///
    /// The returned list is always sorted by signer_pk regardless of ingestion order,
    /// ensuring that node-side assembly never leaks HashMap ordering.
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

    /// Build a serializable snapshot of the current in-memory state.
    pub fn to_snapshot(&self) -> EscrowStageSnapshot {
        EscrowStageSnapshot {
            funded_escrows: self
                .funded_escrows
                .iter()
                .map(|(k, v)| (hash32_to_hex(k), v.clone()))
                .collect(),
            proposals: self
                .proposals
                .iter()
                .map(|(k, v)| (hash32_to_hex(k), SnapshotStagedProposal::from_staged(v)))
                .collect(),
        }
    }

    /// Restore state from a snapshot (overwrites current in-memory state).
    pub fn apply_snapshot(
        &mut self,
        snapshot: EscrowStageSnapshot,
    ) -> Result<(), EscrowStageError> {
        self.funded_escrows = snapshot
            .funded_escrows
            .into_iter()
            .map(|(k, v)| hex_to_hash32(&k).map(|key| (key, v)))
            .collect::<Result<_, _>>()?;
        self.proposals = snapshot
            .proposals
            .into_iter()
            .map(|(k, snap_prop)| {
                let key = hex_to_hash32(&k)?;
                let (proposal, session_context, approvals) = snap_prop.into_staged()?;
                Ok((
                    key,
                    StagedProposal {
                        session_context,
                        proposal,
                        approvals,
                    },
                ))
            })
            .collect::<Result<_, EscrowStageError>>()?;
        Ok(())
    }

    /// Load state from a JSON file on disk.
    ///
    /// - Missing file → empty store, no error.
    /// - Corrupt / unreadable file → `EscrowStageError::Persistence`.
    pub fn load_from_path(path: &Path) -> Result<Self, EscrowStageError> {
        match std::fs::read_to_string(path) {
            Ok(contents) => {
                let snapshot: EscrowStageSnapshot =
                    serde_json::from_str(&contents).map_err(|e| {
                        EscrowStageError::Persistence(format!(
                            "corrupt snapshot at {}: {e}",
                            path.display()
                        ))
                    })?;
                let mut store = Self::new();
                store.apply_snapshot(snapshot)?;
                Ok(store)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::new()),
            Err(e) => Err(EscrowStageError::Persistence(format!(
                "failed to read {}: {e}",
                path.display()
            ))),
        }
    }

    /// Persist the current state to a JSON file on disk using a temp-file + rename pattern.
    ///
    /// This ensures the target path never holds a half-written file on the success path.
    pub fn save_to_path(&self, path: &Path) -> Result<(), EscrowStageError> {
        let snapshot = self.to_snapshot();
        let json = serde_json::to_string(&snapshot)
            .map_err(|e| EscrowStageError::Persistence(format!("serialize snapshot: {e}")))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                EscrowStageError::Persistence(format!("create dir {}: {e}", parent.display()))
            })?;
        }

        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, json.as_bytes()).map_err(|e| {
            EscrowStageError::Persistence(format!("write temp file {}: {e}", tmp_path.display()))
        })?;

        std::fs::rename(&tmp_path, path).map_err(|e| {
            EscrowStageError::Persistence(format!(
                "rename {} -> {}: {e}",
                tmp_path.display(),
                path.display()
            ))
        })?;

        Ok(())
    }

    /// Returns the canonical filename for the escrow stage snapshot.
    pub const SNAPSHOT_FILENAME: &'static str = "escrow_stage.json";
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
            },
        };

        assert_eq!(
            store.ingest_proposal(proposal_body),
            Err(EscrowStageError::ProposalWithoutFunding)
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
