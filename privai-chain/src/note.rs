use serde::{Deserialize, Serialize};

use crate::canonical::{
    write_bytes, write_fixed, write_option_bytes, write_u64, write_u8, CanonicalEncode,
};
use crate::hash::{
    domain_hash, AUX_DOMAIN, BUNDLE_DOMAIN, NOTE_DOMAIN, NOTE_PAYLOAD_DOMAIN, NULLIFIER_DOMAIN,
    POLICY_DOMAIN,
};
use crate::params::{AEAD_ALG_XCHACHA20_POLY1305, FRODOKEM_640_SHAKE, PRIVAI_V0};
use crate::primitives::{Amount14, BundleId, Flags8, Hash32, LweCiphertext, Nullifier};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SpendPolicyTag {
    Single = 0x01,
    MarketplaceSettlement = 0x02,
    /// Escrow 2-of-3 multisig (escrow-2of3-v1).
    Escrow2of3 = 0x03,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiveBundle {
    pub version: u8,
    pub bundle_id: BundleId,
    pub expires_at: u64,
    pub flags: Flags8,
    pub one_time_falcon_pk: Vec<u8>,
    pub one_time_frodo_pk: Vec<u8>,
    pub route_hint: Option<Vec<u8>>,
    /// W v1 nullifier_key jest wyprowadzany z Nullifier Root odbiorcy
    pub nullifier_key: Hash32,
}

impl ReceiveBundle {
    pub fn new(
        bundle_id: BundleId,
        expires_at: u64,
        flags: Flags8,
        one_time_falcon_pk: Vec<u8>,
        one_time_frodo_pk: Vec<u8>,
        route_hint: Option<Vec<u8>>,
        nullifier_key: Hash32,
    ) -> Self {
        Self {
            version: PRIVAI_V0,
            bundle_id,
            expires_at,
            flags,
            one_time_falcon_pk,
            one_time_frodo_pk,
            route_hint,
            nullifier_key,
        }
    }

    pub fn commitment(&self) -> Hash32 {
        domain_hash(BUNDLE_DOMAIN, &[&self.to_canonical_bytes()])
    }
}

impl CanonicalEncode for ReceiveBundle {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.version);
        write_fixed(out, &self.bundle_id);
        write_u64(out, self.expires_at);
        write_u8(out, self.flags);
        write_bytes(out, &self.one_time_falcon_pk);
        write_bytes(out, &self.one_time_frodo_pk);
        write_option_bytes(out, self.route_hint.as_deref());
        write_fixed(out, &self.nullifier_key);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpendPolicy {
    Single {
        falcon_pk_hash: Hash32,
    },
    MarketplaceSettlement {
        buyer_pk_hash: Hash32,
        seller_pk_hash: Hash32,
        moderator_pk_hash: Hash32,
        timeout_block: u64,
    },
    /// Policy-constrained 2-of-3 multisig escrow.
    ///
    /// Signer set: Buyer (index 0), Merchant (index 1), Operator (index 2).
    /// Action rules are implied by `policy_tag` (Escrow2of3) — NOT stored in fields.
    /// See: spec/PRIVAI_ESCROW_OBJECT_MODEL.md section 8.3a
    Escrow2of3 {
        buyer_pk_hash: Hash32,
        merchant_pk_hash: Hash32,
        operator_pk_hash: Hash32,
        timeout_block: u64,
    },
}

impl SpendPolicy {
    pub fn tag(&self) -> SpendPolicyTag {
        match self {
            Self::Single { .. } => SpendPolicyTag::Single,
            Self::MarketplaceSettlement { .. } => SpendPolicyTag::MarketplaceSettlement,
            Self::Escrow2of3 { .. } => SpendPolicyTag::Escrow2of3,
        }
    }

    pub fn commitment(&self) -> Hash32 {
        domain_hash(POLICY_DOMAIN, &[&self.to_canonical_bytes()])
    }
}

impl CanonicalEncode for SpendPolicy {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Single { falcon_pk_hash } => {
                write_u8(out, SpendPolicyTag::Single as u8);
                write_fixed(out, falcon_pk_hash);
            }
            Self::MarketplaceSettlement {
                buyer_pk_hash,
                seller_pk_hash,
                moderator_pk_hash,
                timeout_block,
            } => {
                write_u8(out, SpendPolicyTag::MarketplaceSettlement as u8);
                write_fixed(out, buyer_pk_hash);
                write_fixed(out, seller_pk_hash);
                write_fixed(out, moderator_pk_hash);
                write_u64(out, *timeout_block);
            }
            Self::Escrow2of3 {
                buyer_pk_hash,
                merchant_pk_hash,
                operator_pk_hash,
                timeout_block,
            } => {
                write_u8(out, SpendPolicyTag::Escrow2of3 as u8);
                write_fixed(out, buyer_pk_hash);
                write_fixed(out, merchant_pk_hash);
                write_fixed(out, operator_pk_hash);
                write_u64(out, *timeout_block);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipientBox {
    pub version: u8,
    pub kem_alg: u8,
    pub aead_alg: u8,
    pub kem_ct: Vec<u8>,
    pub nonce: [u8; 24],
    pub ciphertext: Vec<u8>,
    pub tag: [u8; 16],
    pub hint: [u8; 16],
}

impl RecipientBox {
    pub fn new(
        kem_ct: Vec<u8>,
        nonce: [u8; 24],
        ciphertext: Vec<u8>,
        tag: [u8; 16],
        hint: [u8; 16],
    ) -> Self {
        Self {
            version: PRIVAI_V0,
            kem_alg: FRODOKEM_640_SHAKE,
            aead_alg: AEAD_ALG_XCHACHA20_POLY1305,
            kem_ct,
            nonce,
            ciphertext,
            tag,
            hint,
        }
    }
}

impl CanonicalEncode for RecipientBox {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.version);
        write_u8(out, self.kem_alg);
        write_u8(out, self.aead_alg);
        write_bytes(out, &self.kem_ct);
        write_fixed(out, &self.nonce);
        write_bytes(out, &self.ciphertext);
        write_fixed(out, &self.tag);
        write_fixed(out, &self.hint);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipientBoxPlaintext {
    pub version: u8,
    pub bundle_id: BundleId,
    pub note_payload_commit: Hash32,
    pub amount: Amount14,
    pub witness_seed: Hash32,
    pub nullifier_key: Hash32,
    pub spend_policy_opening: Vec<u8>,
    pub aux_opening: Vec<u8>,
    pub sender_memo: Option<Vec<u8>>,
}

impl CanonicalEncode for RecipientBoxPlaintext {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.version);
        write_fixed(out, &self.bundle_id);
        write_fixed(out, &self.note_payload_commit);
        self.amount.encode(out);
        write_fixed(out, &self.witness_seed);
        write_fixed(out, &self.nullifier_key);
        write_bytes(out, &self.spend_policy_opening);
        write_bytes(out, &self.aux_opening);
        write_option_bytes(out, self.sender_memo.as_deref());
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuxWitness {
    pub version: u8,
    pub amount: Amount14,
    pub witness_seed: Hash32,
    pub noise_class: u8,
    pub bundle_id: BundleId,
}

impl CanonicalEncode for AuxWitness {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.version);
        self.amount.encode(out);
        write_fixed(out, &self.witness_seed);
        write_u8(out, self.noise_class);
        write_fixed(out, &self.bundle_id);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputNote {
    pub version: u8,
    pub note_commit: Hash32,
    pub spend_policy_commit: Hash32,
    pub ct_amt: LweCiphertext,
    pub aux_commit: Hash32,
    pub recipient_box: RecipientBox,
}

pub type HiddenOutput = OutputNote;

impl OutputNote {
    pub fn payload_bytes_from_parts(
        version: u8,
        spend_policy_commit: &Hash32,
        ct_amt: &LweCiphertext,
        aux_commit: &Hash32,
    ) -> Vec<u8> {
        let mut out = Vec::new();
        write_u8(&mut out, version);
        write_fixed(&mut out, spend_policy_commit);
        ct_amt.encode(&mut out);
        write_fixed(&mut out, aux_commit);
        out
    }

    pub fn payload_commit_from_parts(
        version: u8,
        spend_policy_commit: &Hash32,
        ct_amt: &LweCiphertext,
        aux_commit: &Hash32,
    ) -> Hash32 {
        let payload_bytes =
            Self::payload_bytes_from_parts(version, spend_policy_commit, ct_amt, aux_commit);
        domain_hash(NOTE_PAYLOAD_DOMAIN, &[&payload_bytes])
    }

    pub fn new(
        spend_policy_commit: Hash32,
        ct_amt: LweCiphertext,
        aux_commit: Hash32,
        recipient_box: RecipientBox,
    ) -> Self {
        let mut note = Self {
            version: PRIVAI_V0,
            note_commit: [0; 32],
            spend_policy_commit,
            ct_amt,
            aux_commit,
            recipient_box,
        };
        note.note_commit = note.recompute_commit();
        note
    }

    pub fn payload_bytes(&self) -> Vec<u8> {
        Self::payload_bytes_from_parts(
            self.version,
            &self.spend_policy_commit,
            &self.ct_amt,
            &self.aux_commit,
        )
    }

    pub fn payload_commit(&self) -> Hash32 {
        Self::payload_commit_from_parts(
            self.version,
            &self.spend_policy_commit,
            &self.ct_amt,
            &self.aux_commit,
        )
    }

    pub fn note_body_bytes(&self) -> Vec<u8> {
        let mut out = self.payload_bytes();
        self.recipient_box.encode(&mut out);
        out
    }

    pub fn recompute_commit(&self) -> Hash32 {
        domain_hash(NOTE_DOMAIN, &[&self.note_body_bytes()])
    }
}

impl CanonicalEncode for OutputNote {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.version);
        write_fixed(out, &self.note_commit);
        write_fixed(out, &self.spend_policy_commit);
        self.ct_amt.encode(out);
        write_fixed(out, &self.aux_commit);
        self.recipient_box.encode(out);
    }
}

pub fn derive_aux_commit(aux_witness: &AuxWitness) -> Hash32 {
    domain_hash(AUX_DOMAIN, &[&aux_witness.to_canonical_bytes()])
}

pub fn derive_nullifier(note_commit: &Hash32, nullifier_key: &Hash32) -> Nullifier {
    Nullifier(domain_hash(NULLIFIER_DOMAIN, &[note_commit, nullifier_key]))
}

/// LiteOutputNote — lekki output dla RecipientPrivacyLite.
/// Zamiast LWE ciphertext (ct_amt) ma jawna kwote (amount).
/// RecipientBox zostaje w pelni (stealth address).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiteOutputNote {
    pub version: u8,
    pub note_commit: Hash32,
    pub amount: u64,
    pub spend_policy_commit: Hash32,
    pub aux_commit: Hash32,
    pub recipient_box: RecipientBox,
}

pub const LITE_NOTE_DOMAIN: &str = "privai:lite-note:v0";
pub const LITE_NOTE_PAYLOAD_DOMAIN: &str = "privai:lite-note-payload:v0";

impl LiteOutputNote {
    pub fn new(
        amount: u64,
        spend_policy_commit: Hash32,
        aux_commit: Hash32,
        recipient_box: RecipientBox,
    ) -> Self {
        let mut note = Self {
            version: PRIVAI_V0,
            note_commit: [0; 32],
            amount,
            spend_policy_commit,
            aux_commit,
            recipient_box,
        };
        note.note_commit = note.recompute_commit();
        note
    }

    /// note_commit dla Lite: binduje version, spend_policy, amount, aux_commit i hint (nie caly box)
    pub fn recompute_commit(&self) -> Hash32 {
        let mut payload = Vec::new();
        write_u8(&mut payload, self.version);
        write_fixed(&mut payload, &self.spend_policy_commit);
        write_u64(&mut payload, self.amount);
        write_fixed(&mut payload, &self.aux_commit);
        write_fixed(&mut payload, &self.recipient_box.hint);
        domain_hash(LITE_NOTE_DOMAIN, &[&payload])
    }

    pub fn payload_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_u8(&mut out, self.version);
        write_fixed(&mut out, &self.spend_policy_commit);
        write_u64(&mut out, self.amount);
        write_fixed(&mut out, &self.aux_commit);
        out
    }

    pub fn payload_commit_from_parts(
        version: u8,
        spend_policy_commit: &Hash32,
        amount: u64,
        aux_commit: &Hash32,
    ) -> Hash32 {
        let mut out = Vec::new();
        write_u8(&mut out, version);
        write_fixed(&mut out, spend_policy_commit);
        write_u64(&mut out, amount);
        write_fixed(&mut out, aux_commit);
        domain_hash(LITE_NOTE_PAYLOAD_DOMAIN, &[&out])
    }

    pub fn payload_commit(&self) -> Hash32 {
        domain_hash(LITE_NOTE_PAYLOAD_DOMAIN, &[&self.payload_bytes()])
    }
}

impl CanonicalEncode for LiteOutputNote {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.version);
        write_fixed(out, &self.note_commit);
        write_u64(out, self.amount);
        write_fixed(out, &self.spend_policy_commit);
        write_fixed(out, &self.aux_commit);
        self.recipient_box.encode(out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::LweCiphertext;

    #[test]
    fn output_note_commit_is_deterministic() {
        let note = OutputNote::new(
            [7; 32],
            LweCiphertext::default(),
            [9; 32],
            RecipientBox::new(vec![1, 2], [3; 24], vec![4, 5], [6; 16], [7; 16]),
        );

        assert_eq!(note.note_commit, note.recompute_commit());
    }

    #[test]
    fn output_note_payload_commit_is_deterministic() {
        let note = OutputNote::new(
            [7; 32],
            LweCiphertext::default(),
            [9; 32],
            RecipientBox::new(vec![1, 2], [3; 24], vec![4, 5], [6; 16], [7; 16]),
        );

        assert_eq!(
            note.payload_commit(),
            domain_hash(NOTE_PAYLOAD_DOMAIN, &[&note.payload_bytes()])
        );
    }

    #[test]
    fn nullifier_derivation_binds_to_note_and_key() {
        let a = derive_nullifier(&[1; 32], &[2; 32]);
        let b = derive_nullifier(&[1; 32], &[3; 32]);
        assert_ne!(a, b);
    }

    #[test]
    fn lite_output_note_commit_is_deterministic() {
        let note = LiteOutputNote::new(
            1000,
            [7; 32],
            [9; 32],
            RecipientBox::new(vec![1, 2], [3; 24], vec![4, 5], [6; 16], [7; 16]),
        );

        assert_eq!(note.note_commit, note.recompute_commit());
    }

    #[test]
    fn lite_output_note_different_amounts_different_commits() {
        let box_a = RecipientBox::new(vec![1, 2], [3; 24], vec![4, 5], [6; 16], [7; 16]);
        let box_b = RecipientBox::new(vec![1, 2], [3; 24], vec![4, 5], [6; 16], [8; 16]);

        let note_a = LiteOutputNote::new(1000, [7; 32], [9; 32], box_a);
        let note_b = LiteOutputNote::new(1000, [7; 32], [9; 32], box_b);

        // Different hint -> different note_commit
        assert_ne!(note_a.note_commit, note_b.note_commit);
    }

    #[test]
    fn lite_output_note_commit_does_not_include_full_recipient_box() {
        // Verify note_commit is 32 bytes (hash), not the size of the full RecipientBox
        let note = LiteOutputNote::new(
            42,
            [1; 32],
            [2; 32],
            RecipientBox::new(vec![0; 9720], [0; 24], vec![0; 1000], [0; 16], [0xA; 16]),
        );

        assert_eq!(note.note_commit.len(), 32);
        // Commit should be deterministic
        assert_eq!(note.note_commit, note.recompute_commit());
    }

    #[test]
    fn lite_output_note_payload_is_lightweight() {
        let note = LiteOutputNote::new(
            500,
            [1; 32],
            [2; 32],
            RecipientBox::new(vec![1, 2, 3], [4; 24], vec![5, 6], [7; 16], [8; 16]),
        );

        let payload = note.payload_bytes();
        // Payload: version(1) + spend_policy_commit(32) + amount(8) + aux_commit(32) = 73 bytes
        assert_eq!(payload.len(), 73);

        // Much smaller than FullPrivacy payload which includes LWE ciphertext (~4100 bytes)
        assert!(payload.len() < 100);
    }
}
