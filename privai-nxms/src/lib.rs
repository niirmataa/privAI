use nxms_transport::wire::{NXMS_PROTO_V2, NxmsPayloadV2};
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
    MsgTypeMismatch { payload: String, expected: &'static str },
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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketAcceptBody {
    pub model_id: Hash32,
    pub session_context: ContextId,
    pub escrow_required: bool,
    pub accepted_price_units: u64,
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
pub struct EscrowResolveBody {
    pub session_context: ContextId,
    pub resolution_tx_ref: Hash32,
    pub outcome_code: u8,
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
        Ok(hash32_with_domain(PRIVAI_BODY_HASH_DOMAIN_V1, &[&body_json]))
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

        let decoded = PrivaiBody::from_canonical_json_slice(
            &body.to_canonical_json_vec().expect("bytes"),
        )
        .expect("decode");

        assert_eq!(decoded.body_hash().expect("decoded hash"), body.body_hash().expect("hash"));
    }

    #[test]
    fn derive_context_id_is_stable() {
        let context_id = derive_context_id(&[b"market-session", &h32(0x10), &c16(0x20)]);
        assert_eq!(
            hex::encode(context_id),
            "3c924ef4efd5e77291c0e93844c99018"
        );
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
}
