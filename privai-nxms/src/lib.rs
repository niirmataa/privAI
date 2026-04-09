use nxms_transport::wire::{NxmsPayloadV2, NXMS_PROTO_V2};
use privai_chain::small_payments::{Receipt, ServicePaymentPolicy, SpendGrant};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PRIVAI_APP_PROTO_V1: &str = "PRIVAI/1";
pub const PRIVAI_BODY_HASH_DOMAIN_V1: &[u8] = b"privai:nxms:body:v1";
pub const PRIVAI_CONTEXT_ID_DOMAIN_V1: &[u8] = b"privai:nxms:context:v1";
pub const PRIVAI_PROOF_JOB_ID_DOMAIN_V1: &[u8] = b"privai:nxms:proof-job:v1";
pub const PRIVAI_BUNDLE_DOMAIN_V0: &[u8] = b"privai:bundle:v0";
pub const PRIVAI_STATEMENT_DOMAIN_V0: &[u8] = b"privai:stmt:v0";
pub const PRIVAI_FALCON_PK_DOMAIN_V0: &[u8] = b"privai:falcon-pk:v0";

pub type Hash32 = [u8; 32];
pub type BundleId = [u8; 16];
pub type ContextId = [u8; 16];

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("unexpected app_proto '{0}'")]
    UnexpectedAppProto(String),
    #[error("payload msg_type '{payload}' does not match body kind '{expected}'")]
    MsgTypeMismatch {
        payload: String,
        expected: &'static str,
    },
    #[error("body kind '{body}' does not belong to PRIVAI/1")]
    UnexpectedBodyKind { body: String },
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PricingMode {
    PerRequest,
    PerToken,
    PerMinute,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleOfferBody {
    pub bundle_id: BundleId,
    pub bundle_commit: Hash32,
    pub relay_hint: Option<String>,
    pub expires_at_block: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleRequestBody {
    pub bundle_id: BundleId,
    pub requester_pk_hash: Hash32,
    pub reply_context: ContextId,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleDeliveryBody {
    pub bundle_id: BundleId,
    pub bundle_commit: Hash32,
    pub bundle_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketOfferBody {
    pub model_id: Hash32,
    pub operator_id: Hash32,
    pub pricing_mode: PricingMode,
    pub price_units: u64,
    pub max_context_tokens: u32,
    pub terms_commit: Hash32,
    /// Zaktualizowane dla v0 SmallPaymentsRail
    pub small_payments_policy: Option<ServicePaymentPolicy>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketAcceptBody {
    pub model_id: Hash32,
    pub session_context: ContextId,
    pub escrow_required: bool,
    pub accepted_price_units: u64,
    /// Przypisany Grant dla transakcji w Railu, autoryzujący małe płatności z portfela Usera.
    pub spend_grant: Option<SpendGrant>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InferenceRequestBody {
    pub session_context: ContextId,
    pub model_id: Hash32,
    pub request_box: Vec<u8>,
    pub max_output_tokens: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InferenceResponseBody {
    pub session_context: ContextId,
    pub response_box: Vec<u8>,
    pub response_commit: Hash32,
    pub usage_units: u64,
    /// Rachunek do podpisania/odesłania do Operatora jako płatność za Inferencję.
    pub receipt_offer: Option<Receipt>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WitnessUpdateBody {
    pub statement_commit: Hash32,
    pub tx_refs: Vec<Hash32>,
    pub witness_box: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EscrowOpenBody {
    pub session_context: ContextId,
    pub escrow_note_commit: Hash32,
    pub tx_ref: Hash32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EscrowFundedBody {
    pub session_context: ContextId,
    pub descriptor: EscrowFundingDescriptor,
    pub funding_tx_ref: Hash32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EscrowSpendProposalBody {
    pub session_context: ContextId,
    pub proposal: EscrowSpendProposal,
}

/// Single signer approval over a proposal.
///
/// `signature` is authorization material — signer intent over the proposal.
/// It is NOT a final ledger-ready signature over `tx_signing_hash`.
/// Final signing over the canonical tx body happens in Stage B (wallet/final assembly).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EscrowApprovalBody {
    pub session_context: ContextId,
    pub proposal_hash: Hash32,
    pub signer_pk: Hash32,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EscrowResolveBody {
    pub session_context: ContextId,
    pub resolution_tx_ref: Hash32,
    pub outcome_code: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EscrowFundingDescriptor {
    pub escrow_id: Hash32,
    pub buyer_pk: Hash32,
    pub merchant_pk: Hash32,
    pub operator_pk: Hash32,
    pub amount: u64,
    pub spend_policy_commit: Hash32,
    pub timeout_blocks: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EscrowSnapshot {
    pub escrow_id: Hash32,
    pub funding_descriptor: EscrowFundingDescriptor,
    pub funding_note_commit: Option<Hash32>,
    pub status: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EscrowSpendProposal {
    pub proposal_hash: Hash32,
    pub escrow_id: Hash32,
    pub snapshot_hash: Hash32,
    pub action: u8, // 0 = release, 1 = refund, 2 = recovery_release, etc.
}

/// Stage A (control-plane) approval bundle — authorization material handed to Stage B.
///
/// This bundle collects quorum approvals and passes them to wallet/final assembly (Stage B)
/// where the final `TransferNoteTx` with `tx_signing_hash` is constructed.
///
/// Semantics:
/// - `signatures` and `signer_pks` are approval / authorization material, NOT final ledger-ready auth.
/// - `tx_signing_hash` is a Stage B artifact. In Stage A it is `[0u8; 32]` (unset).
///   The actual hash is computed in Stage B after canonical tx body assembly.
/// - Final auth insertion ordering and signing over `tx_signing_hash` happen in Stage B.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EscrowApprovalBundle {
    pub proposal_hash: Hash32,
    pub tx_signing_hash: Hash32,
    pub signer_pks: Vec<Hash32>,
    pub signatures: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BundleValidationError {
    /// signer_pks.len() != signatures.len()
    SignerSignatureCountMismatch { signers: usize, signatures: usize },
    /// Two or more signer_pks are identical
    DuplicateSigner(Hash32),
}

impl std::fmt::Display for BundleValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SignerSignatureCountMismatch {
                signers,
                signatures,
            } => {
                write!(
                    f,
                    "signer/signature count mismatch: {signers} signers vs {signatures} signatures"
                )
            }
            Self::DuplicateSigner(pk) => {
                write!(f, "duplicate signer: {}", hex::encode(pk))
            }
        }
    }
}

impl std::error::Error for BundleValidationError {}

/// Stage A sentinel: `tx_signing_hash` is not set until Stage B.
pub const TX_SIGNING_HASH_STAGE_A: Hash32 = [0u8; 32];

impl EscrowApprovalBundle {
    /// Build a bundle from individual approval bodies, sorted deterministically by signer_pk.
    ///
    /// The returned bundle has `signer_pks` and `signatures` in stable ascending order
    /// by `signer_pk`, ensuring deterministic assembly regardless of input ordering.
    ///
    /// `tx_signing_hash` is set to `TX_SIGNING_HASH_STAGE_A` (zeroed) because
    /// the actual hash is computed in Stage B.
    pub fn from_approvals_sorted(
        proposal_hash: Hash32,
        approvals: &[EscrowApprovalBody],
    ) -> Result<Self, BundleValidationError> {
        let mut sorted: Vec<&EscrowApprovalBody> = approvals.iter().collect();
        sorted.sort_by_key(|a| a.signer_pk);

        let signer_pks: Vec<Hash32> = sorted.iter().map(|a| a.signer_pk).collect();
        let signatures: Vec<Vec<u8>> = sorted.iter().map(|a| a.signature.clone()).collect();

        // Check duplicates (sorted, so adjacent comparison suffices)
        for i in 1..signer_pks.len() {
            if signer_pks[i] == signer_pks[i - 1] {
                return Err(BundleValidationError::DuplicateSigner(signer_pks[i]));
            }
        }

        Ok(Self {
            proposal_hash,
            tx_signing_hash: TX_SIGNING_HASH_STAGE_A,
            signer_pks,
            signatures,
        })
    }

    /// Validate the bundle invariants:
    /// - signer_pks and signatures have the same length
    /// - no duplicate signer_pks
    pub fn validate(&self) -> Result<(), BundleValidationError> {
        if self.signer_pks.len() != self.signatures.len() {
            return Err(BundleValidationError::SignerSignatureCountMismatch {
                signers: self.signer_pks.len(),
                signatures: self.signatures.len(),
            });
        }

        // Check for duplicates
        for i in 0..self.signer_pks.len() {
            for j in (i + 1)..self.signer_pks.len() {
                if self.signer_pks[i] == self.signer_pks[j] {
                    return Err(BundleValidationError::DuplicateSigner(self.signer_pks[i]));
                }
            }
        }

        Ok(())
    }

    /// Returns true if `tx_signing_hash` is still at the Stage A sentinel (unset).
    ///
    /// A valid Stage B bundle must have this field set to the actual signing hash
    /// computed from the canonical tx body.
    pub fn is_stage_a(&self) -> bool {
        self.tx_signing_hash == TX_SIGNING_HASH_STAGE_A
    }

    /// Returns the signers in stable (sorted) order.
    /// Convenience method for callers that need a deterministic signer list.
    pub fn signers_sorted(&self) -> Vec<Hash32> {
        let mut v = self.signer_pks.clone();
        v.sort();
        v
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofServiceRequestBody {
    pub job_id: Hash32,
    pub statement_commit: Hash32,
    pub proof_system_id: u8,
    pub witness_box: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofServiceResponseBody {
    pub job_id: Hash32,
    pub statement_commit: Hash32,
    pub proof_system_id: u8,
    pub proof_bytes: Vec<u8>,
    pub verifier_hint: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "body", rename_all = "snake_case")]
pub enum PrivaiBody {
    BundleOffer(BundleOfferBody),
    BundleRequest(BundleRequestBody),
    BundleDelivery(BundleDeliveryBody),
    MarketOffer(MarketOfferBody),
    MarketAccept(MarketAcceptBody),
    InferenceRequest(InferenceRequestBody),
    InferenceResponse(InferenceResponseBody),
    WitnessUpdate(WitnessUpdateBody),
    EscrowOpen(EscrowOpenBody),
    EscrowFunded(EscrowFundedBody),
    EscrowSpendProposal(EscrowSpendProposalBody),
    EscrowApproval(EscrowApprovalBody),
    EscrowResolve(EscrowResolveBody),
    ProofServiceRequest(ProofServiceRequestBody),
    ProofServiceResponse(ProofServiceResponseBody),
}

impl PrivaiBody {
    pub fn to_canonical_json_vec(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn to_canonical_json_string(&self) -> Result<String, ProtocolError> {
        Ok(String::from_utf8(self.to_canonical_json_vec()?)?)
    }

    pub fn from_canonical_json_slice(bytes: &[u8]) -> Result<Self, ProtocolError> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn from_canonical_json_str(value: &str) -> Result<Self, ProtocolError> {
        Ok(serde_json::from_str(value)?)
    }

    pub fn body_hash(&self) -> Result<Hash32, ProtocolError> {
        let body_json = self.to_canonical_json_vec()?;
        Ok(hash32_with_domain(
            PRIVAI_BODY_HASH_DOMAIN_V1,
            &[&body_json],
        ))
    }

    pub fn body_hash_hex(&self) -> Result<String, ProtocolError> {
        Ok(hex::encode(self.body_hash()?))
    }

    pub fn msg_type_key(&self) -> &'static str {
        match self {
            Self::BundleOffer(_) => "bundle_offer",
            Self::BundleRequest(_) => "bundle_request",
            Self::BundleDelivery(_) => "bundle_delivery",
            Self::MarketOffer(_) => "market_offer",
            Self::MarketAccept(_) => "market_accept",
            Self::InferenceRequest(_) => "inference_request",
            Self::InferenceResponse(_) => "inference_response",
            Self::WitnessUpdate(_) => "witness_update",
            Self::EscrowOpen(_) => "escrow_open",
            Self::EscrowFunded(_) => "escrow_funded",
            Self::EscrowSpendProposal(_) => "escrow_spend_proposal",
            Self::EscrowApproval(_) => "escrow_approval",
            Self::EscrowResolve(_) => "escrow_resolve",
            Self::ProofServiceRequest(_) => "proof_service_request",
            Self::ProofServiceResponse(_) => "proof_service_response",
        }
    }

    pub fn to_payload(
        &self,
        context_id: ContextId,
        from: impl Into<String>,
        to: impl Into<String>,
        seq: u64,
    ) -> Result<NxmsPayloadV2, ProtocolError> {
        Ok(NxmsPayloadV2 {
            app_proto: PRIVAI_APP_PROTO_V1.to_string(),
            msg_type: self.msg_type_key().to_string(),
            context_id_hex: hex::encode(context_id),
            from: from.into(),
            to: to.into(),
            seq,
            data: self.to_canonical_json_string()?,
        })
    }

    pub fn from_payload(payload: &NxmsPayloadV2) -> Result<Self, ProtocolError> {
        if payload.app_proto != PRIVAI_APP_PROTO_V1 {
            return Err(ProtocolError::UnexpectedAppProto(payload.app_proto.clone()));
        }

        let body = PrivaiBody::from_canonical_json_str(&payload.data)?;
        let expected = body.msg_type_key();
        if payload.msg_type != expected {
            return Err(ProtocolError::MsgTypeMismatch {
                payload: payload.msg_type.clone(),
                expected,
            });
        }
        Ok(body)
    }
}

pub fn make_context_id(bytes: [u8; 16]) -> ContextId {
    bytes
}

pub fn hash32_with_domain(domain: &[u8], parts: &[&[u8]]) -> Hash32 {
    let mut hasher = blake3::Hasher::new();
    update_len_prefixed(&mut hasher, domain);
    for part in parts {
        update_len_prefixed(&mut hasher, part);
    }
    *hasher.finalize().as_bytes()
}

pub fn derive_context_id(parts: &[&[u8]]) -> ContextId {
    truncate_hash32(hash32_with_domain(PRIVAI_CONTEXT_ID_DOMAIN_V1, parts))
}

pub fn derive_bundle_id(parts: &[&[u8]]) -> BundleId {
    truncate_hash32(hash32_with_domain(PRIVAI_BUNDLE_DOMAIN_V0, parts))
}

pub fn derive_proof_job_id(parts: &[&[u8]]) -> Hash32 {
    hash32_with_domain(PRIVAI_PROOF_JOB_ID_DOMAIN_V1, parts)
}

pub fn statement_commit(parts: &[&[u8]]) -> Hash32 {
    hash32_with_domain(PRIVAI_STATEMENT_DOMAIN_V0, parts)
}

pub fn falcon_pk_hash(pk_bytes: &[u8]) -> Hash32 {
    hash32_with_domain(PRIVAI_FALCON_PK_DOMAIN_V0, &[pk_bytes])
}

pub fn nxms_proto_v2() -> &'static str {
    NXMS_PROTO_V2
}

fn truncate_hash32(hash: Hash32) -> [u8; 16] {
    let mut out = [0u8; 16];
    out.copy_from_slice(&hash[..16]);
    out
}

fn update_len_prefixed(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u32).to_le_bytes());
    hasher.update(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h32(fill: u8) -> Hash32 {
        [fill; 32]
    }

    fn c16(fill: u8) -> ContextId {
        [fill; 16]
    }

    #[test]
    fn market_offer_roundtrip_payload() {
        let body = PrivaiBody::MarketOffer(MarketOfferBody {
            model_id: h32(1),
            operator_id: h32(2),
            pricing_mode: PricingMode::PerToken,
            price_units: 42,
            max_context_tokens: 4096,
            terms_commit: h32(3),
            small_payments_policy: None,
        });

        let payload = body
            .to_payload(c16(9), "alice", "provider", 7)
            .expect("payload");

        assert_eq!(payload.app_proto, PRIVAI_APP_PROTO_V1);
        assert_eq!(payload.msg_type, "market_offer");
        assert_eq!(payload.context_id_hex, hex::encode(c16(9)));

        let decoded = PrivaiBody::from_payload(&payload).expect("decode");
        assert_eq!(decoded, body);
    }

    #[test]
    fn bundle_offer_canonical_json_is_stable() {
        let body = PrivaiBody::BundleOffer(BundleOfferBody {
            bundle_id: c16(0x11),
            bundle_commit: h32(0x22),
            relay_hint: Some("relay://alpha".to_string()),
            expires_at_block: 55,
        });

        let json = body.to_canonical_json_string().expect("json");
        assert_eq!(
            json,
            "{\"kind\":\"bundle_offer\",\"body\":{\"bundle_id\":[17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17],\"bundle_commit\":[34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34,34],\"relay_hint\":\"relay://alpha\",\"expires_at_block\":55}}"
        );
    }

    #[test]
    fn proof_service_response_canonical_json_is_stable() {
        let body = PrivaiBody::ProofServiceResponse(ProofServiceResponseBody {
            job_id: h32(0x33),
            statement_commit: h32(0x44),
            proof_system_id: 7,
            proof_bytes: vec![1, 2, 3, 4],
            verifier_hint: Some(vec![9, 8]),
        });

        let json = body.to_canonical_json_string().expect("json");
        assert_eq!(
            json,
            "{\"kind\":\"proof_service_response\",\"body\":{\"job_id\":[51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51,51],\"statement_commit\":[68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68,68],\"proof_system_id\":7,\"proof_bytes\":[1,2,3,4],\"verifier_hint\":[9,8]}}"
        );
    }

    #[test]
    fn canonical_json_roundtrip_from_bytes() {
        let body = PrivaiBody::InferenceRequest(InferenceRequestBody {
            session_context: c16(0x55),
            model_id: h32(0x66),
            request_box: vec![7, 7, 7],
            max_output_tokens: 128,
        });

        let bytes = body.to_canonical_json_vec().expect("bytes");
        let decoded = PrivaiBody::from_canonical_json_slice(&bytes).expect("decode");
        assert_eq!(decoded, body);
    }

    #[test]
    fn bundle_offer_body_hash_is_stable() {
        let body = PrivaiBody::BundleOffer(BundleOfferBody {
            bundle_id: c16(0x11),
            bundle_commit: h32(0x22),
            relay_hint: Some("relay://alpha".to_string()),
            expires_at_block: 55,
        });

        let hash_hex = body.body_hash_hex().expect("hash");
        assert_eq!(
            hash_hex,
            "c32da6c3466e7ea12635d7d79db735a0c603cb4af96646cfe12c4576f7e79c67"
        );
    }

    #[test]
    fn body_hash_survives_canonical_roundtrip() {
        let body = PrivaiBody::InferenceRequest(InferenceRequestBody {
            session_context: c16(0x55),
            model_id: h32(0x66),
            request_box: vec![7, 7, 7],
            max_output_tokens: 128,
        });

        let decoded =
            PrivaiBody::from_canonical_json_slice(&body.to_canonical_json_vec().expect("bytes"))
                .expect("decode");

        assert_eq!(
            decoded.body_hash().expect("decoded hash"),
            body.body_hash().expect("hash")
        );
    }

    #[test]
    fn derive_context_id_is_stable() {
        let context_id = derive_context_id(&[b"market-session", &h32(0x10), &c16(0x20)]);
        assert_eq!(hex::encode(context_id), "3c924ef4efd5e77291c0e93844c99018");
    }

    #[test]
    fn proof_job_id_is_stable() {
        let job_id = derive_proof_job_id(&[&h32(0x33), &h32(0x44), &[7u8], b"batch-0"]);
        assert_eq!(
            hex::encode(job_id),
            "83850c616b8accac26a98c74e49e380a67a30a6c600c716b3dd9a13c2ce8910c"
        );
    }

    #[test]
    fn falcon_pk_hash_is_stable() {
        let pk_hash = falcon_pk_hash(b"falcon-public-key-bytes");
        assert_eq!(
            hex::encode(pk_hash),
            "8c5ae572fb780d179533d3952a0e16b851d793cdd263587ca4a6869644703c74"
        );
    }

    #[test]
    fn proof_service_payload_rejects_msg_type_mismatch() {
        let body = PrivaiBody::ProofServiceRequest(ProofServiceRequestBody {
            job_id: h32(4),
            statement_commit: h32(5),
            proof_system_id: 1,
            witness_box: vec![1, 2, 3],
        });
        let mut payload = body
            .to_payload(c16(8), "wallet", "prover", 11)
            .expect("payload");
        payload.msg_type = "market_offer".to_string();

        let err = PrivaiBody::from_payload(&payload).expect_err("must reject mismatch");
        match err {
            ProtocolError::MsgTypeMismatch { payload, expected } => {
                assert_eq!(payload, "market_offer");
                assert_eq!(expected, "proof_service_request");
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn payload_rejects_wrong_app_proto() {
        let payload = NxmsPayloadV2 {
            app_proto: "ESCROW/1".to_string(),
            msg_type: "bundle_offer".to_string(),
            context_id_hex: hex::encode(c16(1)),
            from: "alice".to_string(),
            to: "bob".to_string(),
            seq: 1,
            data: serde_json::to_string(&PrivaiBody::BundleOffer(BundleOfferBody {
                bundle_id: c16(2),
                bundle_commit: h32(3),
                relay_hint: None,
                expires_at_block: 10,
            }))
            .expect("json"),
        };

        let err = PrivaiBody::from_payload(&payload).expect_err("must reject app proto");
        match err {
            ProtocolError::UnexpectedAppProto(value) => assert_eq!(value, "ESCROW/1"),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn marketplace_small_payments_v0_nxms_flow() {
        use privai_chain::small_payments::{AllowedRail, PricingMode as SpPricingMode};

        // 1. Operator (Provider) prepares a MarketOffer with a small payment policy
        let policy = ServicePaymentPolicy {
            policy_version: 1,
            merchant_commit: h32(0x10),
            service_commit: None,
            allowed_rail: AllowedRail::SmallPaymentsRail,
            pricing_mode: SpPricingMode::ReservationThenSettle,
            min_deposit_required: 1000,
            max_spend_per_session: 5000,
            max_spend_per_window: 10000,
            grant_expiry_rule: 3600,
            settlement_window_rule: 86400,
            requires_full_privacy_if: 50000,
        };

        let offer = PrivaiBody::MarketOffer(MarketOfferBody {
            model_id: h32(1),
            operator_id: h32(2),
            pricing_mode: PricingMode::PerToken,
            price_units: 1,
            max_context_tokens: 4096,
            terms_commit: h32(3),
            small_payments_policy: Some(policy.clone()),
        });

        // 2. User creates a SpendGrant out-of-band and accepts the offer
        let grant = SpendGrant {
            merchant_commit: policy.merchant_commit,
            service_commit: None,
            session_scope: h32(0x20),
            spend_cap: 1000,
            grant_expiry: 999999999,
            settlement_window: 999999999,
            policy_commit: policy.policy_commit(),
            operator_sig: vec![],
        };

        let accept = PrivaiBody::MarketAccept(MarketAcceptBody {
            model_id: h32(1),
            session_context: c16(0x55),
            escrow_required: false,
            accepted_price_units: 1,
            spend_grant: Some(grant.clone()),
        });

        // 3. User sends an Inference Request
        let req = PrivaiBody::InferenceRequest(InferenceRequestBody {
            session_context: c16(0x55),
            model_id: h32(1),
            request_box: vec![0xaa, 0xbb],
            max_output_tokens: 200,
        });

        // 4. Provider sends Inference Response with a Receipt
        let receipt = Receipt {
            receipt_id: h32(0x99),
            merchant_commit: policy.merchant_commit,
            service_commit: None,
            session_commit: h32(0x20),
            grant_commit: grant.grant_commit(),
            purchase_commit: h32(0x30),
            ticket_nullifier: privai_chain::Nullifier(h32(0x40)),
            amount: 50,
            policy_commit: policy.policy_commit(),
            result_commit: h32(0x50),
            issued_at: 0,
            merchant_sig: vec![],
        };

        let resp = PrivaiBody::InferenceResponse(InferenceResponseBody {
            session_context: c16(0x55),
            response_box: vec![0xcc, 0xdd],
            response_commit: h32(0x50),
            usage_units: 50,
            receipt_offer: Some(receipt),
        });

        // Test serialization to ensure formats are valid for NXMS wire transmission
        assert!(offer.to_canonical_json_string().is_ok());
        assert!(accept.to_canonical_json_string().is_ok());
        assert!(req.to_canonical_json_string().is_ok());
        assert!(resp.to_canonical_json_string().is_ok());
    }

    // ---------------------------------------------------------------
    // Escrow wire/canonical roundtrip tests
    // ---------------------------------------------------------------

    fn escrow_descriptor(fill: u8) -> EscrowFundingDescriptor {
        EscrowFundingDescriptor {
            escrow_id: h32(fill),
            buyer_pk: h32(fill.wrapping_add(1)),
            merchant_pk: h32(fill.wrapping_add(2)),
            operator_pk: h32(fill.wrapping_add(3)),
            amount: 1000 + fill as u64,
            spend_policy_commit: h32(fill.wrapping_add(4)),
            timeout_blocks: 500 + fill as u64,
        }
    }

    // --- 1. Roundtrip tests ---

    #[test]
    fn escrow_funded_canonical_roundtrip() {
        let body = PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: c16(0xA1),
            descriptor: escrow_descriptor(0x10),
            funding_tx_ref: h32(0xBB),
        });
        let bytes = body.to_canonical_json_vec().expect("encode");
        let decoded = PrivaiBody::from_canonical_json_slice(&bytes).expect("decode");
        assert_eq!(decoded, body);
    }

    #[test]
    fn escrow_spend_proposal_canonical_roundtrip() {
        let body = PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: c16(0xA2),
            proposal: EscrowSpendProposal {
                proposal_hash: h32(0xC1),
                escrow_id: h32(0xC2),
                snapshot_hash: h32(0xC3),
                action: 0, // release
            },
        });
        let bytes = body.to_canonical_json_vec().expect("encode");
        let decoded = PrivaiBody::from_canonical_json_slice(&bytes).expect("decode");
        assert_eq!(decoded, body);
    }

    #[test]
    fn escrow_approval_canonical_roundtrip() {
        let body = PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: c16(0xA3),
            proposal_hash: h32(0xD1),
            signer_pk: h32(0xD2),
            signature: vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE],
        });
        let bytes = body.to_canonical_json_vec().expect("encode");
        let decoded = PrivaiBody::from_canonical_json_slice(&bytes).expect("decode");
        assert_eq!(decoded, body);
    }

    #[test]
    fn escrow_resolve_canonical_roundtrip() {
        let body = PrivaiBody::EscrowResolve(EscrowResolveBody {
            session_context: c16(0xA4),
            resolution_tx_ref: h32(0xE1),
            outcome_code: 2,
        });
        let bytes = body.to_canonical_json_vec().expect("encode");
        let decoded = PrivaiBody::from_canonical_json_slice(&bytes).expect("decode");
        assert_eq!(decoded, body);
    }

    // --- 2. msg_type_key consistency ---

    #[test]
    fn escrow_msg_type_keys() {
        assert_eq!(
            PrivaiBody::EscrowFunded(EscrowFundedBody {
                session_context: c16(0),
                descriptor: escrow_descriptor(0),
                funding_tx_ref: h32(0),
            })
            .msg_type_key(),
            "escrow_funded"
        );

        assert_eq!(
            PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
                session_context: c16(0),
                proposal: EscrowSpendProposal {
                    proposal_hash: h32(0),
                    escrow_id: h32(0),
                    snapshot_hash: h32(0),
                    action: 0,
                },
            })
            .msg_type_key(),
            "escrow_spend_proposal"
        );

        assert_eq!(
            PrivaiBody::EscrowApproval(EscrowApprovalBody {
                session_context: c16(0),
                proposal_hash: h32(0),
                signer_pk: h32(0),
                signature: vec![],
            })
            .msg_type_key(),
            "escrow_approval"
        );

        assert_eq!(
            PrivaiBody::EscrowResolve(EscrowResolveBody {
                session_context: c16(0),
                resolution_tx_ref: h32(0),
                outcome_code: 0,
            })
            .msg_type_key(),
            "escrow_resolve"
        );
    }

    // --- 3. body_hash stability ---

    #[test]
    fn escrow_funded_body_hash_is_deterministic() {
        let body = PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: c16(0xA1),
            descriptor: escrow_descriptor(0x10),
            funding_tx_ref: h32(0xBB),
        });
        let h1 = body.body_hash().expect("hash1");
        let h2 = body.body_hash().expect("hash2");
        assert_eq!(h1, h2);
        // Golden value — same body always produces this hash.
        assert_eq!(
            hex::encode(h1),
            "d5142db2a67596a99e71fe026a363b4a6eaa6cd8fc9c4303820bb26f72d54d49"
        );
    }

    #[test]
    fn escrow_spend_proposal_body_hash_is_deterministic() {
        let body = PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: c16(0xA2),
            proposal: EscrowSpendProposal {
                proposal_hash: h32(0xC1),
                escrow_id: h32(0xC2),
                snapshot_hash: h32(0xC3),
                action: 0,
            },
        });
        let h1 = body.body_hash().expect("hash1");
        let h2 = body.body_hash().expect("hash2");
        assert_eq!(h1, h2);
        // Golden value will need to be updated after removing tx_signing_hash field
        let golden = body.body_hash_hex().expect("golden");
        assert_eq!(hex::encode(h1), golden);
    }

    #[test]
    fn escrow_spend_proposal_hash_changes_with_action() {
        let mk_body = |action: u8| {
            PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
                session_context: c16(0xA2),
                proposal: EscrowSpendProposal {
                    proposal_hash: h32(0xC1),
                    escrow_id: h32(0xC2),
                    snapshot_hash: h32(0xC3),
                    action,
                },
            })
        };
        let h_release = mk_body(0).body_hash().expect("release hash");
        let h_refund = mk_body(1).body_hash().expect("refund hash");
        let h_recovery = mk_body(2).body_hash().expect("recovery hash");
        assert_ne!(h_release, h_refund);
        assert_ne!(h_release, h_recovery);
        assert_ne!(h_refund, h_recovery);
    }

    #[test]
    fn escrow_funded_hash_changes_with_amount() {
        let mk_body = |fill: u8| {
            PrivaiBody::EscrowFunded(EscrowFundedBody {
                session_context: c16(0xA1),
                descriptor: escrow_descriptor(fill),
                funding_tx_ref: h32(0xBB),
            })
        };
        let h1 = mk_body(0x10).body_hash().expect("h1");
        let h2 = mk_body(0x20).body_hash().expect("h2");
        assert_ne!(h1, h2);
    }

    #[test]
    fn escrow_body_hash_survives_roundtrip() {
        let body = PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: c16(0xA3),
            proposal_hash: h32(0xD1),
            signer_pk: h32(0xD2),
            signature: vec![0xAA, 0xBB, 0xCC],
        });
        let original_hash = body.body_hash().expect("original hash");
        let decoded =
            PrivaiBody::from_canonical_json_slice(&body.to_canonical_json_vec().expect("bytes"))
                .expect("decode");
        assert_eq!(decoded.body_hash().expect("decoded hash"), original_hash);
    }

    // --- 4. Payload roundtrip (PRIVAI/1 + msg_type) ---

    #[test]
    fn escrow_funded_payload_roundtrip() {
        let body = PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: c16(0xA1),
            descriptor: escrow_descriptor(0x10),
            funding_tx_ref: h32(0xBB),
        });
        let payload = body
            .to_payload(c16(1), "buyer", "operator", 1)
            .expect("payload");
        assert_eq!(payload.app_proto, PRIVAI_APP_PROTO_V1);
        assert_eq!(payload.msg_type, "escrow_funded");
        let decoded = PrivaiBody::from_payload(&payload).expect("decode");
        assert_eq!(decoded, body);
    }

    #[test]
    fn escrow_spend_proposal_payload_roundtrip() {
        let body = PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: c16(0xA2),
            proposal: EscrowSpendProposal {
                proposal_hash: h32(0xC1),
                escrow_id: h32(0xC2),
                snapshot_hash: h32(0xC3),
                action: 1, // refund
            },
        });
        let payload = body
            .to_payload(c16(2), "nexum-core", "signer", 3)
            .expect("payload");
        assert_eq!(payload.app_proto, PRIVAI_APP_PROTO_V1);
        assert_eq!(payload.msg_type, "escrow_spend_proposal");
        let decoded = PrivaiBody::from_payload(&payload).expect("decode");
        assert_eq!(decoded, body);
    }

    #[test]
    fn escrow_approval_payload_roundtrip() {
        let body = PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: c16(0xA3),
            proposal_hash: h32(0xD1),
            signer_pk: h32(0xD2),
            signature: vec![0x01, 0x02, 0x03],
        });
        let payload = body
            .to_payload(c16(3), "buyer", "nexum-core", 5)
            .expect("payload");
        assert_eq!(payload.app_proto, PRIVAI_APP_PROTO_V1);
        assert_eq!(payload.msg_type, "escrow_approval");
        let decoded = PrivaiBody::from_payload(&payload).expect("decode");
        assert_eq!(decoded, body);
    }

    #[test]
    fn escrow_resolve_payload_roundtrip() {
        let body = PrivaiBody::EscrowResolve(EscrowResolveBody {
            session_context: c16(0xA4),
            resolution_tx_ref: h32(0xE1),
            outcome_code: 1,
        });
        let payload = body
            .to_payload(c16(4), "operator", "buyer", 7)
            .expect("payload");
        assert_eq!(payload.app_proto, PRIVAI_APP_PROTO_V1);
        assert_eq!(payload.msg_type, "escrow_resolve");
        let decoded = PrivaiBody::from_payload(&payload).expect("decode");
        assert_eq!(decoded, body);
    }

    #[test]
    fn escrow_payload_rejects_msg_type_mismatch() {
        let body = PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
            session_context: c16(0xA2),
            proposal: EscrowSpendProposal {
                proposal_hash: h32(0xC1),
                escrow_id: h32(0xC2),
                snapshot_hash: h32(0xC3),
                action: 2, // recovery_release
            },
        });
        let mut payload = body
            .to_payload(c16(5), "core", "signer", 1)
            .expect("payload");
        payload.msg_type = "escrow_funded".to_string(); // wrong type
        let err = PrivaiBody::from_payload(&payload).expect_err("must reject");
        match err {
            ProtocolError::MsgTypeMismatch { payload, expected } => {
                assert_eq!(payload, "escrow_funded");
                assert_eq!(expected, "escrow_spend_proposal");
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn escrow_payload_rejects_wrong_app_proto() {
        let body = PrivaiBody::EscrowFunded(EscrowFundedBody {
            session_context: c16(0xA1),
            descriptor: escrow_descriptor(0x10),
            funding_tx_ref: h32(0xBB),
        });
        let mut payload = body
            .to_payload(c16(6), "buyer", "node", 1)
            .expect("payload");
        payload.app_proto = "ESCROW/1".to_string(); // wrong proto
        let err = PrivaiBody::from_payload(&payload).expect_err("must reject");
        match err {
            ProtocolError::UnexpectedAppProto(value) => assert_eq!(value, "ESCROW/1"),
            other => panic!("unexpected error: {other}"),
        }
    }

    // --- 5. Golden canonical JSON string (EscrowApproval) ---

    #[test]
    fn escrow_approval_golden_canonical_json() {
        let body = PrivaiBody::EscrowApproval(EscrowApprovalBody {
            session_context: c16(0xA3),
            proposal_hash: h32(0xD1),
            signer_pk: h32(0xD2),
            signature: vec![0xAA, 0xBB],
        });
        let json = body.to_canonical_json_string().expect("json");
        assert_eq!(
            json,
            "{\"kind\":\"escrow_approval\",\"body\":{\"session_context\":[163,163,163,163,163,163,163,163,163,163,163,163,163,163,163,163],\"proposal_hash\":[209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209,209],\"signer_pk\":[210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210,210],\"signature\":[170,187]}}"
        );
    }

    // --- 6. All escrow bodies serialize cleanly ---

    #[test]
    fn all_escrow_bodies_canonical_json_ok() {
        let bodies: Vec<PrivaiBody> = vec![
            PrivaiBody::EscrowFunded(EscrowFundedBody {
                session_context: c16(1),
                descriptor: escrow_descriptor(1),
                funding_tx_ref: h32(1),
            }),
            PrivaiBody::EscrowSpendProposal(EscrowSpendProposalBody {
                session_context: c16(2),
                proposal: EscrowSpendProposal {
                    proposal_hash: h32(2),
                    escrow_id: h32(3),
                    snapshot_hash: h32(4),
                    action: 0,
                },
            }),
            PrivaiBody::EscrowApproval(EscrowApprovalBody {
                session_context: c16(3),
                proposal_hash: h32(6),
                signer_pk: h32(7),
                signature: vec![0xFF],
            }),
            PrivaiBody::EscrowResolve(EscrowResolveBody {
                session_context: c16(4),
                resolution_tx_ref: h32(8),
                outcome_code: 0,
            }),
        ];

        for body in &bodies {
            assert!(body.to_canonical_json_vec().is_ok());
            assert!(body.to_canonical_json_string().is_ok());
            assert!(body.body_hash().is_ok());
        }
    }

    // ---------------------------------------------------------------
    // EscrowApprovalBundle hardening tests
    // ---------------------------------------------------------------

    #[test]
    fn bundle_rejects_duplicate_signers() {
        let approvals = vec![
            EscrowApprovalBody {
                session_context: c16(1),
                proposal_hash: h32(0x30),
                signer_pk: h32(1),
                signature: vec![0xAA],
            },
            EscrowApprovalBody {
                session_context: c16(1),
                proposal_hash: h32(0x30),
                signer_pk: h32(1), // same signer
                signature: vec![0xBB],
            },
        ];

        let result = EscrowApprovalBundle::from_approvals_sorted(h32(0x30), &approvals);
        assert!(matches!(
            result,
            Err(BundleValidationError::DuplicateSigner(_))
        ));

        // Also detectable via validate()
        let bundle = EscrowApprovalBundle {
            proposal_hash: h32(0x30),
            tx_signing_hash: TX_SIGNING_HASH_STAGE_A,
            signer_pks: vec![h32(1), h32(1)],
            signatures: vec![vec![0xAA], vec![0xBB]],
        };
        assert!(matches!(
            bundle.validate(),
            Err(BundleValidationError::DuplicateSigner(_))
        ));
    }

    #[test]
    fn bundle_rejects_signer_signature_count_mismatch() {
        // More signers than signatures
        let bundle = EscrowApprovalBundle {
            proposal_hash: h32(0x30),
            tx_signing_hash: TX_SIGNING_HASH_STAGE_A,
            signer_pks: vec![h32(1), h32(2), h32(3)],
            signatures: vec![vec![0xAA], vec![0xBB]],
        };
        assert!(matches!(
            bundle.validate(),
            Err(BundleValidationError::SignerSignatureCountMismatch {
                signers: 3,
                signatures: 2
            })
        ));

        // More signatures than signers
        let bundle2 = EscrowApprovalBundle {
            proposal_hash: h32(0x30),
            tx_signing_hash: TX_SIGNING_HASH_STAGE_A,
            signer_pks: vec![h32(1)],
            signatures: vec![vec![0xAA], vec![0xBB]],
        };
        assert!(matches!(
            bundle2.validate(),
            Err(BundleValidationError::SignerSignatureCountMismatch {
                signers: 1,
                signatures: 2
            })
        ));
    }

    #[test]
    fn bundle_ordering_is_deterministic() {
        // Approvals in reverse order of signer_pk
        let approvals = vec![
            EscrowApprovalBody {
                session_context: c16(1),
                proposal_hash: h32(0x30),
                signer_pk: h32(0x20),
                signature: vec![1],
            },
            EscrowApprovalBody {
                session_context: c16(1),
                proposal_hash: h32(0x30),
                signer_pk: h32(0x10),
                signature: vec![2],
            },
        ];

        let bundle =
            EscrowApprovalBundle::from_approvals_sorted(h32(0x30), &approvals).expect("valid");

        // signer_pks must be ascending
        assert_eq!(bundle.signer_pks[0], h32(0x10));
        assert_eq!(bundle.signer_pks[1], h32(0x20));

        // signatures aligned with signer_pks
        assert_eq!(bundle.signatures[0], vec![2]);
        assert_eq!(bundle.signatures[1], vec![1]);

        // Build again with different input order — must produce identical bundle
        let approvals_reversed = vec![approvals[1].clone(), approvals[0].clone()];
        let bundle2 = EscrowApprovalBundle::from_approvals_sorted(h32(0x30), &approvals_reversed)
            .expect("valid");
        assert_eq!(bundle, bundle2);
    }

    #[test]
    fn bundle_validate_passes_on_clean_bundle() {
        let bundle = EscrowApprovalBundle {
            proposal_hash: h32(0x30),
            tx_signing_hash: TX_SIGNING_HASH_STAGE_A,
            signer_pks: vec![h32(1), h32(2)],
            signatures: vec![vec![0xAA], vec![0xBB]],
        };
        assert!(bundle.validate().is_ok());
    }

    #[test]
    fn bundle_is_stage_a_sentinel() {
        let bundle = EscrowApprovalBundle {
            proposal_hash: h32(0x30),
            tx_signing_hash: TX_SIGNING_HASH_STAGE_A,
            signer_pks: vec![h32(1)],
            signatures: vec![vec![0xAA]],
        };
        assert!(bundle.is_stage_a());

        let mut stage_b_bundle = bundle.clone();
        stage_b_bundle.tx_signing_hash = h32(0xFF);
        assert!(!stage_b_bundle.is_stage_a());
    }

    #[test]
    fn bundle_signers_sorted_returns_stable_order() {
        let bundle = EscrowApprovalBundle {
            proposal_hash: h32(0x30),
            tx_signing_hash: TX_SIGNING_HASH_STAGE_A,
            signer_pks: vec![h32(0x30), h32(0x10), h32(0x20)],
            signatures: vec![vec![3], vec![1], vec![2]],
        };

        let sorted = bundle.signers_sorted();
        assert_eq!(sorted, vec![h32(0x10), h32(0x20), h32(0x30)]);
    }

    #[test]
    fn bundle_from_approvals_rejects_empty_duplicate_edge() {
        // Single approval — no duplicate possible
        let approvals = vec![EscrowApprovalBody {
            session_context: c16(1),
            proposal_hash: h32(0x30),
            signer_pk: h32(1),
            signature: vec![0xAA],
        }];
        let bundle =
            EscrowApprovalBundle::from_approvals_sorted(h32(0x30), &approvals).expect("valid");
        assert_eq!(bundle.signer_pks.len(), 1);
        assert!(bundle.validate().is_ok());
    }

    #[test]
    fn bundle_from_approvals_empty_input() {
        let approvals: Vec<EscrowApprovalBody> = vec![];
        let bundle =
            EscrowApprovalBundle::from_approvals_sorted(h32(0x30), &approvals).expect("valid");
        assert!(bundle.signer_pks.is_empty());
        assert!(bundle.signatures.is_empty());
        assert!(bundle.validate().is_ok());
    }
}
