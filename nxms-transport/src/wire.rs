use serde::{Deserialize, Serialize};

pub const NXMS_PROTO_V1: &str = "NXMS/1";
pub const NXMS_PROTO_V2: &str = "NXMS/2";
pub const ESCROW_APP_PROTO_V1: &str = "ESCROW/1";
pub const NXMS_CONTEXT_ID_HEX_LEN: usize = 32; // 16 bytes
pub const NXMS_ESCROW_ID_HEX_LEN: usize = NXMS_CONTEXT_ID_HEX_LEN;
pub const NXMS_MAX_PEER_ID_LEN: usize = 64;
pub const NXMS_MAX_PROTO_TOKEN_LEN: usize = 64;
pub const NXMS_WIRE_KEM_ID_V1: &str = "FrodoKEM-640-SHAKE";
pub const NXMS_WIRE_SIG_ID_V1: &str = "Falcon-1024-CT";
const NXMS_WIRE_MAX_PAYLOAD: usize = 16 * 1024 * 1024;
const NXMS_WIRE_MAX_KEM_CT_LEN: usize = 32768;
const NXMS_WIRE_NONCE_LEN: usize = 24;
const NXMS_WIRE_TAG_LEN: usize = 32;
const NXMS_WIRE_MAX_SIG_LEN: usize = 4096;

/// Application-level message types exchanged over Tor P2P.
/// The payload is always protected by NXMS (FrodoKEM + Falcon) unless noted.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MsgType {
    PrepareInfo,
    MakeInfo,
    ExchangeRound1,
    ExchangeRound2,

    ExportInfoReq,
    ExportInfoResp,

    TxSignReq,
    TxSignResp,

    /// Generic error response (unencrypted text for debugging is avoided in production;
    /// we still wrap it as NXMS payload by default).
    Error,
}

/// Legacy NXMS wire envelope for the escrow flow. This remains for backward compatibility.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NxmsEnvelope {
    pub proto: String,  // "NXMS/1"
    pub kem_id: String, // "FrodoKEM-640-SHAKE"
    pub sig_id: String, // "Falcon-1024-CT"
    pub msg_type: MsgType,
    pub escrow_id_hex: String, // 32 hex chars
    pub from: String,
    pub to: String,

    /// Monotonic sequence number per (escrow_id, from).
    /// Used for idempotency/replay protection (receiver tracks last accepted seq).
    pub seq: u64,

    pub kem_ct_b64: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
    pub tag_b64: String,
    pub sig_b64: String,
}

/// Legacy inner plaintext for ESCROW/1. New protocols should prefer `NxmsPayloadV2`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NxmsPayload {
    #[serde(default = "default_app_proto")]
    pub app_proto: String,
    pub msg_type: MsgType,
    pub escrow_id_hex: String,
    pub from: String,
    pub to: String,
    pub seq: u64,

    /// Arbitrary string payload (Monero multisig blobs, txset hex, etc.)
    pub data: String,
}

/// Generic NXMS wire envelope for the rest of the system.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NxmsEnvelopeV2 {
    pub proto: String,  // "NXMS/2"
    pub kem_id: String, // "FrodoKEM-640-SHAKE"
    pub sig_id: String, // "Falcon-1024-CT"
    pub msg_type: String,
    pub context_id_hex: String, // 32 hex chars / 16 raw bytes
    pub from: String,
    pub to: String,

    /// Monotonic sequence number per (context_id, from).
    pub seq: u64,

    pub kem_ct_b64: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
    pub tag_b64: String,
    pub sig_b64: String,
}

/// Generic NXMS inner payload for application protocols such as PRIVAI/1 or MARKET/1.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NxmsPayloadV2 {
    pub app_proto: String,
    pub msg_type: String,
    pub context_id_hex: String,
    pub from: String,
    pub to: String,
    pub seq: u64,
    pub data: String,
}

fn default_app_proto() -> String {
    ESCROW_APP_PROTO_V1.to_string()
}

impl NxmsPayload {
    pub fn validate_basic(&self) -> Result<(), String> {
        if self.app_proto != ESCROW_APP_PROTO_V1 {
            return Err(format!(
                "unsupported app_proto '{}' (expected {})",
                self.app_proto, ESCROW_APP_PROTO_V1
            ));
        }
        validate_peer_id(&self.from).map_err(|e| format!("invalid from: {e}"))?;
        validate_peer_id(&self.to).map_err(|e| format!("invalid to: {e}"))?;
        validate_hex(&self.escrow_id_hex, NXMS_ESCROW_ID_HEX_LEN)
            .map_err(|e| format!("invalid escrow_id_hex: {e}"))?;
        if self.seq == 0 {
            return Err("seq must be > 0".to_string());
        }
        if self.data.trim().is_empty() {
            return Err("payload data must not be empty".to_string());
        }
        Ok(())
    }

    pub fn validate_matches_envelope(&self, env: &NxmsEnvelope) -> Result<(), String> {
        self.validate_basic()?;
        env.validate_basic()?;
        if self.escrow_id_hex != env.escrow_id_hex {
            return Err("payload/envelope escrow_id mismatch".to_string());
        }
        if self.from != env.from {
            return Err("payload/envelope from mismatch".to_string());
        }
        if self.to != env.to {
            return Err("payload/envelope to mismatch".to_string());
        }
        if self.seq != env.seq {
            return Err("payload/envelope seq mismatch".to_string());
        }
        if msg_type_key(&self.msg_type) != msg_type_key(&env.msg_type) {
            return Err("payload/envelope msg_type mismatch".to_string());
        }
        Ok(())
    }
}

impl NxmsPayloadV2 {
    pub fn validate_basic(&self) -> Result<(), String> {
        validate_proto_token(&self.app_proto, "app_proto")?;
        validate_msg_label(&self.msg_type)?;
        validate_peer_id(&self.from).map_err(|e| format!("invalid from: {e}"))?;
        validate_peer_id(&self.to).map_err(|e| format!("invalid to: {e}"))?;
        validate_context_id_hex(&self.context_id_hex)
            .map_err(|e| format!("invalid context_id_hex: {e}"))?;
        if self.seq == 0 {
            return Err("seq must be > 0".to_string());
        }
        if self.data.trim().is_empty() {
            return Err("payload data must not be empty".to_string());
        }
        Ok(())
    }

    pub fn validate_matches_envelope(&self, env: &NxmsEnvelopeV2) -> Result<(), String> {
        self.validate_basic()?;
        env.validate_basic()?;
        if self.msg_type != env.msg_type {
            return Err("payload/envelope msg_type mismatch".to_string());
        }
        if self.context_id_hex != env.context_id_hex {
            return Err("payload/envelope context_id mismatch".to_string());
        }
        if self.from != env.from {
            return Err("payload/envelope from mismatch".to_string());
        }
        if self.to != env.to {
            return Err("payload/envelope to mismatch".to_string());
        }
        if self.seq != env.seq {
            return Err("payload/envelope seq mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EscrowAction {
    Release,
    Refund,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractPropose {
    pub escrow_id_hex: String,
    pub snapshot_hash_hex: String,
    pub snapshot_json_b64: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractSig {
    pub escrow_id_hex: String,
    pub snapshot_hash_hex: String,
    pub signer_id: String,
    pub sig_pk_b64: String,
    pub sig_b64: String,
    pub alg: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxSignReqBody {
    pub escrow_id_hex: String,
    pub action: EscrowAction,
    pub multisig_txset_hex: String,
    pub snapshot_hash_hex: String,
    pub human_hint: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxSignRespBody {
    pub escrow_id_hex: String,
    pub approved: bool,
    pub signed_tx_data_hex: Option<String>,
    pub tx_hash_list: Vec<String>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EscrowErrBody {
    pub escrow_id_hex: String,
    pub code: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "body", rename_all = "snake_case")]
pub enum EscrowBody {
    ContractPropose(ContractPropose),
    ContractSig(ContractSig),
    TxSignReq(TxSignReqBody),
    TxSignResp(TxSignRespBody),
    Err(EscrowErrBody),
}

impl EscrowBody {
    pub fn expected_msg_type(&self) -> MsgType {
        match self {
            Self::ContractPropose(_) => MsgType::PrepareInfo,
            Self::ContractSig(_) => MsgType::MakeInfo,
            Self::TxSignReq(_) => MsgType::TxSignReq,
            Self::TxSignResp(_) => MsgType::TxSignResp,
            Self::Err(_) => MsgType::Error,
        }
    }
}

impl NxmsEnvelope {
    pub fn validate_basic(&self) -> Result<(), String> {
        if self.proto != NXMS_PROTO_V1 {
            return Err(format!("invalid proto '{}'", self.proto));
        }
        if self.kem_id != NXMS_WIRE_KEM_ID_V1 {
            return Err(format!("unsupported kem_id '{}'", self.kem_id));
        }
        if self.sig_id != NXMS_WIRE_SIG_ID_V1 {
            return Err(format!("unsupported sig_id '{}'", self.sig_id));
        }
        validate_peer_id(&self.from).map_err(|e| format!("invalid from: {e}"))?;
        validate_peer_id(&self.to).map_err(|e| format!("invalid to: {e}"))?;
        validate_hex(&self.escrow_id_hex, NXMS_ESCROW_ID_HEX_LEN)
            .map_err(|e| format!("invalid escrow_id_hex: {e}"))?;
        if self.seq == 0 {
            return Err("seq must be > 0".to_string());
        }
        if self.kem_ct_b64.trim().is_empty()
            || self.nonce_b64.trim().is_empty()
            || self.ciphertext_b64.trim().is_empty()
            || self.tag_b64.trim().is_empty()
            || self.sig_b64.trim().is_empty()
        {
            return Err("missing crypto fields".to_string());
        }
        validate_b64_field_len("kem_ct_b64", &self.kem_ct_b64, NXMS_WIRE_MAX_KEM_CT_LEN)?;
        validate_b64_field_len("nonce_b64", &self.nonce_b64, NXMS_WIRE_NONCE_LEN)?;
        validate_b64_field_len(
            "ciphertext_b64",
            &self.ciphertext_b64,
            NXMS_WIRE_MAX_PAYLOAD,
        )?;
        validate_b64_field_len("tag_b64", &self.tag_b64, NXMS_WIRE_TAG_LEN)?;
        validate_b64_field_len("sig_b64", &self.sig_b64, NXMS_WIRE_MAX_SIG_LEN)?;
        Ok(())
    }
}

impl NxmsEnvelopeV2 {
    pub fn validate_basic(&self) -> Result<(), String> {
        if self.proto != NXMS_PROTO_V2 {
            return Err(format!("invalid proto '{}'", self.proto));
        }
        if self.kem_id != NXMS_WIRE_KEM_ID_V1 {
            return Err(format!("unsupported kem_id '{}'", self.kem_id));
        }
        if self.sig_id != NXMS_WIRE_SIG_ID_V1 {
            return Err(format!("unsupported sig_id '{}'", self.sig_id));
        }
        validate_msg_label(&self.msg_type)?;
        validate_peer_id(&self.from).map_err(|e| format!("invalid from: {e}"))?;
        validate_peer_id(&self.to).map_err(|e| format!("invalid to: {e}"))?;
        validate_context_id_hex(&self.context_id_hex)
            .map_err(|e| format!("invalid context_id_hex: {e}"))?;
        if self.seq == 0 {
            return Err("seq must be > 0".to_string());
        }
        if self.kem_ct_b64.trim().is_empty()
            || self.nonce_b64.trim().is_empty()
            || self.ciphertext_b64.trim().is_empty()
            || self.tag_b64.trim().is_empty()
            || self.sig_b64.trim().is_empty()
        {
            return Err("missing crypto fields".to_string());
        }
        validate_b64_field_len("kem_ct_b64", &self.kem_ct_b64, NXMS_WIRE_MAX_KEM_CT_LEN)?;
        validate_b64_field_len("nonce_b64", &self.nonce_b64, NXMS_WIRE_NONCE_LEN)?;
        validate_b64_field_len(
            "ciphertext_b64",
            &self.ciphertext_b64,
            NXMS_WIRE_MAX_PAYLOAD,
        )?;
        validate_b64_field_len("tag_b64", &self.tag_b64, NXMS_WIRE_TAG_LEN)?;
        validate_b64_field_len("sig_b64", &self.sig_b64, NXMS_WIRE_MAX_SIG_LEN)?;
        Ok(())
    }
}

impl From<&NxmsEnvelope> for NxmsEnvelopeV2 {
    fn from(value: &NxmsEnvelope) -> Self {
        Self {
            proto: NXMS_PROTO_V2.to_string(),
            kem_id: value.kem_id.clone(),
            sig_id: value.sig_id.clone(),
            msg_type: msg_type_key(&value.msg_type).to_string(),
            context_id_hex: value.escrow_id_hex.clone(),
            from: value.from.clone(),
            to: value.to.clone(),
            seq: value.seq,
            kem_ct_b64: value.kem_ct_b64.clone(),
            nonce_b64: value.nonce_b64.clone(),
            ciphertext_b64: value.ciphertext_b64.clone(),
            tag_b64: value.tag_b64.clone(),
            sig_b64: value.sig_b64.clone(),
        }
    }
}

impl From<&NxmsPayload> for NxmsPayloadV2 {
    fn from(value: &NxmsPayload) -> Self {
        Self {
            app_proto: value.app_proto.clone(),
            msg_type: msg_type_key(&value.msg_type).to_string(),
            context_id_hex: value.escrow_id_hex.clone(),
            from: value.from.clone(),
            to: value.to.clone(),
            seq: value.seq,
            data: value.data.clone(),
        }
    }
}

pub fn msg_type_key(t: &MsgType) -> &'static str {
    match t {
        MsgType::PrepareInfo => "prepare_info",
        MsgType::MakeInfo => "make_info",
        MsgType::ExchangeRound1 => "exchange_round1",
        MsgType::ExchangeRound2 => "exchange_round2",
        MsgType::ExportInfoReq => "export_info_req",
        MsgType::ExportInfoResp => "export_info_resp",
        MsgType::TxSignReq => "tx_sign_req",
        MsgType::TxSignResp => "tx_sign_resp",
        MsgType::Error => "error",
    }
}

pub fn validate_peer_id(value: &str) -> Result<(), &'static str> {
    let v = value.trim();
    if v.is_empty() {
        return Err("empty");
    }
    if v.len() > NXMS_MAX_PEER_ID_LEN {
        return Err("too long");
    }
    if !v
        .bytes()
        .all(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-'))
    {
        return Err("invalid characters (allowed: A-Z a-z 0-9 _ -)");
    }
    Ok(())
}

pub fn validate_context_id_hex(value: &str) -> Result<(), &'static str> {
    validate_hex(value, NXMS_CONTEXT_ID_HEX_LEN)
}

pub fn validate_hex(value: &str, expected_len: usize) -> Result<(), &'static str> {
    let v = value.trim();
    if v.len() != expected_len {
        return Err("invalid length");
    }
    if !v.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("non-hex characters");
    }
    Ok(())
}

pub fn validate_proto_token(value: &str, label: &str) -> Result<(), String> {
    validate_token_like(value, label, true)
}

pub fn validate_msg_label(value: &str) -> Result<(), String> {
    validate_token_like(value, "msg_type", false)
}

fn validate_token_like(value: &str, label: &str, allow_slash: bool) -> Result<(), String> {
    let v = value.trim();
    if v.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    if v.len() > NXMS_MAX_PROTO_TOKEN_LEN {
        return Err(format!(
            "{label} too long: {} > {}",
            v.len(),
            NXMS_MAX_PROTO_TOKEN_LEN
        ));
    }
    let valid = v.bytes().all(|b| match b {
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'.' => true,
        b'/' if allow_slash => true,
        _ => false,
    });
    if !valid {
        let suffix = if allow_slash {
            " A-Z a-z 0-9 _ - . /"
        } else {
            " A-Z a-z 0-9 _ - ."
        };
        return Err(format!("{label} has invalid characters (allowed:{suffix})"));
    }
    Ok(())
}

fn validate_b64_field_len(label: &str, value: &str, max_decoded_len: usize) -> Result<(), String> {
    let max_b64_len = max_b64_len_for_decoded(max_decoded_len)
        .ok_or_else(|| format!("{label} limit overflow"))?;
    if value.len() > max_b64_len {
        return Err(format!(
            "{label} too long: {} > {}",
            value.len(),
            max_b64_len
        ));
    }
    Ok(())
}

fn max_b64_len_for_decoded(max_decoded_len: usize) -> Option<usize> {
    let quads = max_decoded_len.checked_add(2)? / 3;
    quads.checked_mul(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_env() -> NxmsEnvelope {
        NxmsEnvelope {
            proto: NXMS_PROTO_V1.to_string(),
            kem_id: NXMS_WIRE_KEM_ID_V1.to_string(),
            sig_id: NXMS_WIRE_SIG_ID_V1.to_string(),
            msg_type: MsgType::TxSignReq,
            escrow_id_hex: "a".repeat(32),
            from: "alice".to_string(),
            to: "bob".to_string(),
            seq: 7,
            kem_ct_b64: "x".to_string(),
            nonce_b64: "x".to_string(),
            ciphertext_b64: "x".to_string(),
            tag_b64: "x".to_string(),
            sig_b64: "x".to_string(),
        }
    }

    fn sample_env_v2() -> NxmsEnvelopeV2 {
        NxmsEnvelopeV2 {
            proto: NXMS_PROTO_V2.to_string(),
            kem_id: NXMS_WIRE_KEM_ID_V1.to_string(),
            sig_id: NXMS_WIRE_SIG_ID_V1.to_string(),
            msg_type: "market_offer".to_string(),
            context_id_hex: "b".repeat(32),
            from: "alice".to_string(),
            to: "bob".to_string(),
            seq: 9,
            kem_ct_b64: "x".to_string(),
            nonce_b64: "x".to_string(),
            ciphertext_b64: "x".to_string(),
            tag_b64: "x".to_string(),
            sig_b64: "x".to_string(),
        }
    }

    #[test]
    fn payload_matches_envelope() {
        let env = sample_env();
        let payload = NxmsPayload {
            app_proto: ESCROW_APP_PROTO_V1.to_string(),
            msg_type: MsgType::TxSignReq,
            escrow_id_hex: env.escrow_id_hex.clone(),
            from: env.from.clone(),
            to: env.to.clone(),
            seq: env.seq,
            data: "{\"kind\":\"tx_sign_req\"}".to_string(),
        };
        payload
            .validate_matches_envelope(&env)
            .expect("payload should match");
    }

    #[test]
    fn payload_v2_matches_envelope_v2() {
        let env = sample_env_v2();
        let payload = NxmsPayloadV2 {
            app_proto: "PRIVAI/1".to_string(),
            msg_type: env.msg_type.clone(),
            context_id_hex: env.context_id_hex.clone(),
            from: env.from.clone(),
            to: env.to.clone(),
            seq: env.seq,
            data: "{\"kind\":\"offer\"}".to_string(),
        };
        payload
            .validate_matches_envelope(&env)
            .expect("payload v2 should match");
    }

    #[test]
    fn payload_rejects_envelope_mismatch() {
        let mut env = sample_env();
        env.seq = 8;
        let payload = NxmsPayload {
            app_proto: ESCROW_APP_PROTO_V1.to_string(),
            msg_type: MsgType::TxSignReq,
            escrow_id_hex: env.escrow_id_hex.clone(),
            from: env.from.clone(),
            to: env.to.clone(),
            seq: 7,
            data: "{\"kind\":\"tx_sign_req\"}".to_string(),
        };
        let err = payload
            .validate_matches_envelope(&env)
            .expect_err("must reject mismatch");
        assert!(err.contains("seq mismatch"));
    }

    #[test]
    fn payload_v2_rejects_context_mismatch() {
        let mut env = sample_env_v2();
        env.context_id_hex = "c".repeat(32);
        let payload = NxmsPayloadV2 {
            app_proto: "MARKET/1".to_string(),
            msg_type: "market_offer".to_string(),
            context_id_hex: "b".repeat(32),
            from: env.from.clone(),
            to: env.to.clone(),
            seq: env.seq,
            data: "hello".to_string(),
        };
        let err = payload
            .validate_matches_envelope(&env)
            .expect_err("must reject mismatch");
        assert!(err.contains("context_id mismatch"));
    }

    #[test]
    fn payload_defaults_app_proto_for_backward_compat() {
        let raw = serde_json::json!({
            "msg_type": "tx_sign_req",
            "escrow_id_hex": "b".repeat(32),
            "from": "alice",
            "to": "bob",
            "seq": 1,
            "data": "{\"kind\":\"tx_sign_req\"}"
        });
        let payload: NxmsPayload = serde_json::from_value(raw).expect("deserialize");
        assert_eq!(payload.app_proto, ESCROW_APP_PROTO_V1);
    }

    #[test]
    fn msg_type_key_is_stable_snake_case() {
        assert_eq!(msg_type_key(&MsgType::PrepareInfo), "prepare_info");
        assert_eq!(msg_type_key(&MsgType::TxSignReq), "tx_sign_req");
        assert_eq!(msg_type_key(&MsgType::TxSignResp), "tx_sign_resp");
        assert_eq!(msg_type_key(&MsgType::Error), "error");
    }

    #[test]
    fn envelope_rejects_wrong_kem_id() {
        let mut env = sample_env();
        env.kem_id = "FrodoKEM-999".to_string();
        let err = env
            .validate_basic()
            .expect_err("must reject unsupported kem");
        assert!(err.contains("unsupported kem_id"));
    }

    #[test]
    fn envelope_v2_rejects_invalid_msg_label() {
        let mut env = sample_env_v2();
        env.msg_type = "market offer".to_string();
        let err = env
            .validate_basic()
            .expect_err("must reject invalid msg label");
        assert!(err.contains("invalid characters"));
    }

    #[test]
    fn envelope_rejects_wrong_sig_id() {
        let mut env = sample_env();
        env.sig_id = "Falcon-512".to_string();
        let err = env
            .validate_basic()
            .expect_err("must reject unsupported sig");
        assert!(err.contains("unsupported sig_id"));
    }

    #[test]
    fn envelope_rejects_oversized_ciphertext_b64() {
        let mut env = sample_env();
        let cap = max_b64_len_for_decoded(NXMS_WIRE_MAX_PAYLOAD).expect("cap");
        env.ciphertext_b64 = "A".repeat(cap + 1);
        let err = env.validate_basic().expect_err("must reject oversized b64");
        assert!(err.contains("ciphertext_b64 too long"));
    }

    #[test]
    fn legacy_conversion_to_v2_preserves_binding_fields() {
        let env = sample_env();
        let payload = NxmsPayload {
            app_proto: ESCROW_APP_PROTO_V1.to_string(),
            msg_type: MsgType::TxSignReq,
            escrow_id_hex: env.escrow_id_hex.clone(),
            from: env.from.clone(),
            to: env.to.clone(),
            seq: env.seq,
            data: "hello".to_string(),
        };
        let env_v2 = NxmsEnvelopeV2::from(&env);
        let payload_v2 = NxmsPayloadV2::from(&payload);

        assert_eq!(env_v2.context_id_hex, env.escrow_id_hex);
        assert_eq!(payload_v2.context_id_hex, payload.escrow_id_hex);
        assert_eq!(env_v2.msg_type, "tx_sign_req");
        assert_eq!(payload_v2.msg_type, "tx_sign_req");
    }
}
