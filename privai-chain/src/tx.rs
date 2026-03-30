use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::canonical::{
    CanonicalEncode, write_fixed, write_i64, write_u8, write_u64, write_vec, write_vec_bytes,
};
use crate::hash::{STATEMENT_DOMAIN, TX_DOMAIN, domain_hash};
use crate::note::OutputNote;
use crate::primitives::{ContextId, Hash32, Nullifier};

pub const TX_TYPE_TRANSFER_NOTE: u8 = 0x01;
pub const TX_TYPE_SETTLEMENT: u8 = 0x02;
pub const TX_TYPE_MODEL: u8 = 0x03;
pub const TX_TYPE_STAKE: u8 = 0x04;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TxShapeError {
    #[error("inputs and input_nullifiers must have equal length")]
    MismatchedInputsAndNullifiers,
    #[error("auth entries must not exceed inputs")]
    TooManyAuthEntries,
    #[error("transfer transactions must create at least one output")]
    TransferRequiresOutputs,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputRef {
    pub note_commit: Hash32,
}

impl CanonicalEncode for InputRef {
    fn encode(&self, out: &mut Vec<u8>) {
        write_fixed(out, &self.note_commit);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputAuth {
    pub policy_tag: u8,
    pub signer_pks: Vec<Vec<u8>>,
    pub signatures: Vec<Vec<u8>>,
}

impl CanonicalEncode for InputAuth {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.policy_tag);
        write_vec_bytes(out, &self.signer_pks);
        write_vec_bytes(out, &self.signatures);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxCore {
    pub version: u8,
    pub tx_type: u8,
    pub inputs: Vec<InputRef>,
    pub input_nullifiers: Vec<Nullifier>,
    pub outputs: Vec<OutputNote>,
    pub fee: u64,
    pub statement_commit: Hash32,
    pub auth: Vec<InputAuth>,
}

impl TxCore {
    pub fn validate_shape(&self) -> Result<(), TxShapeError> {
        if self.inputs.len() != self.input_nullifiers.len() {
            return Err(TxShapeError::MismatchedInputsAndNullifiers);
        }

        if self.auth.len() > self.inputs.len() {
            return Err(TxShapeError::TooManyAuthEntries);
        }

        Ok(())
    }

    pub fn statement_hash(&self) -> Hash32 {
        domain_hash(STATEMENT_DOMAIN, &[&self.statement_commit])
    }
}

impl CanonicalEncode for TxCore {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.version);
        write_u8(out, self.tx_type);
        write_vec(out, &self.inputs);
        write_vec(out, &self.input_nullifiers);
        write_vec(out, &self.outputs);
        write_u64(out, self.fee);
        write_fixed(out, &self.statement_commit);
        write_vec(out, &self.auth);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferNoteTx {
    pub core: TxCore,
}

impl TransferNoteTx {
    pub fn validate_shape(&self) -> Result<(), TxShapeError> {
        self.core.validate_shape()?;
        if self.core.outputs.is_empty() {
            return Err(TxShapeError::TransferRequiresOutputs);
        }
        Ok(())
    }
}

impl CanonicalEncode for TransferNoteTx {
    fn encode(&self, out: &mut Vec<u8>) {
        self.core.encode(out);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SettlementPhase {
    Open = 0x01,
    Accept = 0x02,
    Release = 0x03,
    Refund = 0x04,
    Dispute = 0x05,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementTx {
    pub core: TxCore,
    pub settlement_id: Hash32,
    pub marketplace_context: ContextId,
    pub phase: SettlementPhase,
    pub payload_commit: Hash32,
}

impl CanonicalEncode for SettlementTx {
    fn encode(&self, out: &mut Vec<u8>) {
        self.core.encode(out);
        write_fixed(out, &self.settlement_id);
        write_fixed(out, &self.marketplace_context);
        write_u8(out, self.phase as u8);
        write_fixed(out, &self.payload_commit);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ModelAction {
    Register = 0x01,
    Update = 0x02,
    Retire = 0x03,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelTx {
    pub core: TxCore,
    pub operator_pk_hash: Hash32,
    pub model_commit: Hash32,
    pub metadata_commit: Hash32,
    pub action: ModelAction,
}

impl CanonicalEncode for ModelTx {
    fn encode(&self, out: &mut Vec<u8>) {
        self.core.encode(out);
        write_fixed(out, &self.operator_pk_hash);
        write_fixed(out, &self.model_commit);
        write_fixed(out, &self.metadata_commit);
        write_u8(out, self.action as u8);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum StakeAction {
    Bond = 0x01,
    Unbond = 0x02,
    Reward = 0x03,
    Slash = 0x04,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StakeTx {
    pub core: TxCore,
    pub validator_pk_hash: Hash32,
    pub action: StakeAction,
    pub amount_delta: i64,
}

impl CanonicalEncode for StakeTx {
    fn encode(&self, out: &mut Vec<u8>) {
        self.core.encode(out);
        write_fixed(out, &self.validator_pk_hash);
        write_u8(out, self.action as u8);
        write_i64(out, self.amount_delta);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transaction {
    TransferNote(TransferNoteTx),
    Settlement(SettlementTx),
    Model(ModelTx),
    Stake(StakeTx),
}

impl Transaction {
    pub fn core(&self) -> &TxCore {
        match self {
            Self::TransferNote(tx) => &tx.core,
            Self::Settlement(tx) => &tx.core,
            Self::Model(tx) => &tx.core,
            Self::Stake(tx) => &tx.core,
        }
    }

    pub fn tx_id(&self) -> Hash32 {
        domain_hash(TX_DOMAIN, &[&self.to_canonical_bytes()])
    }

    pub fn tx_type(&self) -> u8 {
        self.core().tx_type
    }

    pub fn inputs(&self) -> &[InputRef] {
        &self.core().inputs
    }

    pub fn input_nullifiers(&self) -> &[Nullifier] {
        &self.core().input_nullifiers
    }

    pub fn outputs(&self) -> &[OutputNote] {
        &self.core().outputs
    }

    pub fn fee(&self) -> u64 {
        self.core().fee
    }

    pub fn statement_commit(&self) -> Hash32 {
        self.core().statement_commit
    }

    pub fn validate_shape(&self) -> Result<(), TxShapeError> {
        match self {
            Self::TransferNote(tx) => tx.validate_shape(),
            Self::Settlement(tx) => tx.core.validate_shape(),
            Self::Model(tx) => tx.core.validate_shape(),
            Self::Stake(tx) => tx.core.validate_shape(),
        }
    }
}

impl CanonicalEncode for Transaction {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::TransferNote(tx) => tx.encode(out),
            Self::Settlement(tx) => tx.encode(out),
            Self::Model(tx) => tx.encode(out),
            Self::Stake(tx) => tx.encode(out),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_requires_output() {
        let tx = TransferNoteTx {
            core: TxCore {
                version: 0,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: Vec::new(),
                input_nullifiers: Vec::new(),
                outputs: Vec::new(),
                fee: 0,
                statement_commit: [0; 32],
                auth: Vec::new(),
            },
        };

        assert_eq!(tx.validate_shape(), Err(TxShapeError::TransferRequiresOutputs));
    }
}
