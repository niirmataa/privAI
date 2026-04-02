use serde::{Deserialize, Serialize};

use crate::canonical::{CanonicalEncode, write_fixed, write_u8, write_u32, write_u64, write_vec};
use crate::hash::{BLOCK_HEADER_DOMAIN, PROOF_CERT_DOMAIN, domain_hash, merkle_root};
use crate::params::PRIVAI_V0;
use crate::primitives::Hash32;
use crate::tx::Transaction;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpochParams {
    pub epoch_number: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub min_validator_stake: u64,
    pub min_prover_bond: u64,
    pub min_fee: u64,
    pub max_block_bytes: u32,
    pub max_block_statements: u32,
    pub min_proof_coverage: u32,
}

impl CanonicalEncode for EpochParams {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u64(out, self.epoch_number);
        write_u64(out, self.start_height);
        write_u64(out, self.end_height);
        write_u64(out, self.min_validator_stake);
        write_u64(out, self.min_prover_bond);
        write_u64(out, self.min_fee);
        write_u32(out, self.max_block_bytes);
        write_u32(out, self.max_block_statements);
        write_u32(out, self.min_proof_coverage);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ExecutionMode {
    FullBatchProof = 0x01,
    MultiProofBundle = 0x02,
    Housekeeping = 0x03,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionBundle {
    pub statement_commits: Vec<Hash32>,
    pub covered_tx_indexes: Vec<u32>,
    pub public_inputs_root: Hash32,
    pub execution_mode: ExecutionMode,
}

impl CanonicalEncode for ExecutionBundle {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u32(out, self.statement_commits.len() as u32);
        for statement in &self.statement_commits {
            write_fixed(out, statement);
        }
        write_u32(out, self.covered_tx_indexes.len() as u32);
        for index in &self.covered_tx_indexes {
            write_u32(out, *index);
        }
        write_fixed(out, &self.public_inputs_root);
        write_u8(out, self.execution_mode as u8);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofCertificate {
    pub proof_system_id: u8,
    pub statement_root: Hash32,
    pub public_inputs_root: Hash32,
    pub proof_bytes_hash: Hash32,
    pub prover_ids: Vec<Hash32>,
    pub proof_meta_hash: Hash32,
}

impl CanonicalEncode for ProofCertificate {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.proof_system_id);
        write_fixed(out, &self.statement_root);
        write_fixed(out, &self.public_inputs_root);
        write_fixed(out, &self.proof_bytes_hash);
        write_u32(out, self.prover_ids.len() as u32);
        for prover_id in &self.prover_ids {
            write_fixed(out, prover_id);
        }
        write_fixed(out, &self.proof_meta_hash);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusReceipt {
    pub receipt_type: u8,
    pub payload_hash: Hash32,
}

impl CanonicalEncode for ConsensusReceipt {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.receipt_type);
        write_fixed(out, &self.payload_hash);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u8,
    pub chain_id: u32,
    pub height: u64,
    pub epoch: u64,
    pub round: u32,
    pub timestamp_ms: u64,
    pub prev_block_hash: Hash32,
    pub tx_root: Hash32,
    pub note_root: Hash32,
    pub nullifier_root: Hash32,
    pub statement_root: Hash32,
    pub proof_cert_root: Hash32,
    pub proposer_pk_hash: Hash32,
    pub epoch_seed_hash: Hash32,
    pub parent_qc_hash: Hash32,
}

impl BlockHeader {
    pub fn hash(&self) -> Hash32 {
        domain_hash(BLOCK_HEADER_DOMAIN, &[&self.to_canonical_bytes()])
    }
}

impl CanonicalEncode for BlockHeader {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u8(out, self.version);
        write_u32(out, self.chain_id);
        write_u64(out, self.height);
        write_u64(out, self.epoch);
        write_u32(out, self.round);
        write_u64(out, self.timestamp_ms);
        write_fixed(out, &self.prev_block_hash);
        write_fixed(out, &self.tx_root);
        write_fixed(out, &self.note_root);
        write_fixed(out, &self.nullifier_root);
        write_fixed(out, &self.statement_root);
        write_fixed(out, &self.proof_cert_root);
        write_fixed(out, &self.proposer_pk_hash);
        write_fixed(out, &self.epoch_seed_hash);
        write_fixed(out, &self.parent_qc_hash);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockBody {
    pub txs: Vec<Transaction>,
    pub execution_bundle: ExecutionBundle,
    pub proof_certificates: Vec<ProofCertificate>,
    pub extra_receipts: Vec<ConsensusReceipt>,
}

impl CanonicalEncode for BlockBody {
    fn encode(&self, out: &mut Vec<u8>) {
        write_vec(out, &self.txs);
        self.execution_bundle.encode(out);
        write_vec(out, &self.proof_certificates);
        write_vec(out, &self.extra_receipts);
    }
}

impl CanonicalEncode for Block {
    fn encode(&self, out: &mut Vec<u8>) {
        self.header.encode(out);
        self.body.encode(out);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub body: BlockBody,
}

impl Block {
    pub fn from_template(template: BlockTemplate) -> Self {
        let body = BlockBody {
            txs: template.txs,
            execution_bundle: template.execution_bundle,
            proof_certificates: template.proof_certificates,
            extra_receipts: template.extra_receipts,
        };

        let header = BlockHeader {
            version: PRIVAI_V0,
            chain_id: template.chain_id,
            height: template.height,
            epoch: template.epoch,
            round: template.round,
            timestamp_ms: template.timestamp_ms,
            prev_block_hash: template.prev_block_hash,
            tx_root: tx_root(&body.txs),
            note_root: note_root(&body.txs),
            nullifier_root: nullifier_root(&body.txs),
            statement_root: statement_root(&body),
            proof_cert_root: proof_cert_root(&body.proof_certificates),
            proposer_pk_hash: template.proposer_pk_hash,
            epoch_seed_hash: template.epoch_seed_hash,
            parent_qc_hash: template.parent_qc_hash,
        };

        Self { header, body }
    }

    pub fn hash(&self) -> Hash32 {
        self.header.hash()
    }

    pub fn roots_match(&self) -> bool {
        self.header.tx_root == tx_root(&self.body.txs)
            && self.header.note_root == note_root(&self.body.txs)
            && self.header.nullifier_root == nullifier_root(&self.body.txs)
            && self.header.statement_root == statement_root(&self.body)
            && self.header.proof_cert_root == proof_cert_root(&self.body.proof_certificates)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockTemplate {
    pub chain_id: u32,
    pub height: u64,
    pub epoch: u64,
    pub round: u32,
    pub timestamp_ms: u64,
    pub prev_block_hash: Hash32,
    pub proposer_pk_hash: Hash32,
    pub epoch_seed_hash: Hash32,
    pub parent_qc_hash: Hash32,
    pub txs: Vec<Transaction>,
    pub execution_bundle: ExecutionBundle,
    pub proof_certificates: Vec<ProofCertificate>,
    pub extra_receipts: Vec<ConsensusReceipt>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum VoteType {
    Prevote = 0x01,
    Precommit = 0x02,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vote {
    pub height: u64,
    pub round: u32,
    pub block_hash: Hash32,
    pub vote_type: VoteType,
    pub validator_pk: Vec<u8>,
    pub falcon_sig: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewChange {
    pub height: u64,
    pub new_round: u32,
    pub validator_pk: Vec<u8>,
    pub falcon_sig: Vec<u8>,
}

impl CanonicalEncode for ViewChange {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u64(out, self.height);
        write_u32(out, self.new_round);
        crate::canonical::write_bytes(out, &self.validator_pk);
        crate::canonical::write_bytes(out, &self.falcon_sig);
    }
}

impl CanonicalEncode for Vote {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u64(out, self.height);
        write_u32(out, self.round);
        write_fixed(out, &self.block_hash);
        write_u8(out, self.vote_type as u8);
        crate::canonical::write_bytes(out, &self.validator_pk);
        crate::canonical::write_bytes(out, &self.falcon_sig);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuorumCertificate {
    pub height: u64,
    pub round: u32,
    pub block_hash: Hash32,
    pub vote_type: VoteType,
    pub signers: Vec<Vec<u8>>,
    pub signatures: Vec<Vec<u8>>,
}

impl CanonicalEncode for QuorumCertificate {
    fn encode(&self, out: &mut Vec<u8>) {
        write_u64(out, self.height);
        write_u32(out, self.round);
        write_fixed(out, &self.block_hash);
        write_u8(out, self.vote_type as u8);
        crate::canonical::write_vec_bytes(out, &self.signers);
        crate::canonical::write_vec_bytes(out, &self.signatures);
    }
}

pub fn tx_root(txs: &[Transaction]) -> Hash32 {
    merkle_root(txs.iter().map(Transaction::tx_id))
}

pub fn note_root(txs: &[Transaction]) -> Hash32 {
    merkle_root(
        txs.iter()
            .flat_map(|tx| tx.outputs().iter().map(|output| output.note_commit)),
    )
}

pub fn nullifier_root(txs: &[Transaction]) -> Hash32 {
    merkle_root(
        txs.iter()
            .flat_map(|tx| tx.input_nullifiers().iter().map(|nullifier| nullifier.0)),
    )
}

pub fn statement_root(body: &BlockBody) -> Hash32 {
    if body.execution_bundle.statement_commits.is_empty() {
        merkle_root(body.txs.iter().map(Transaction::statement_commit))
    } else {
        merkle_root(body.execution_bundle.statement_commits.iter().copied())
    }
}

pub fn proof_cert_root(proof_certificates: &[ProofCertificate]) -> Hash32 {
    merkle_root(
        proof_certificates
            .iter()
            .map(|certificate| domain_hash(PROOF_CERT_DOMAIN, &[&certificate.to_canonical_bytes()])),
    )
}

/// ConsensusMsg — envelope P2P dla wiadomości konsensusowych wysyłanych między nodami.
/// Każdy variant jest wysyłany broadcastem do validatorów przez Tor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusMsg {
    /// Proposer wysyła blok do walidacji.
    Proposal {
        block: Block,
        proposer_sig: Vec<u8>,
    },

    /// Walidator głosuje prevote (akceptuje blok do dalszej tury).
    Prevote(Vote),

    /// Walidator głosuje precommit (zatwierdza blok).
    Precommit(Vote),

    /// Zbudowany QuorumCertificate — broadcastowany po osiągnięciu thresholdu.
    QuorumCert(QuorumCertificate),

    /// ViewChange — timeout w rundzie, walidator prosi o nową rundę.
    ViewChange(ViewChange),

    /// Ping/keepalive między nodami (opcjonalny, do przyszłego użycia).
    Ping {
        height: u64,
        round: u32,
        sender_pk_hash: Hash32,
    },

    /// Żądanie bloków od peera (state sync).
    SyncRequest {
        from_height: u64,
        to_height: u64,
        requester_pk_hash: Hash32,
    },

    /// Odpowiedź z blokami (state sync).
    SyncResponse {
        blocks: Vec<Block>,
        qcs: Vec<QuorumCertificate>,
        sender_pk_hash: Hash32,
    },
}

impl ConsensusMsg {
    pub fn msg_type(&self) -> &'static str {
        match self {
            Self::Proposal { .. } => "proposal",
            Self::Prevote(_) => "prevote",
            Self::Precommit(_) => "precommit",
            Self::QuorumCert(_) => "quorum_cert",
            Self::ViewChange(_) => "view_change",
            Self::Ping { .. } => "ping",
            Self::SyncRequest { .. } => "sync_request",
            Self::SyncResponse { .. } => "sync_response",
        }
    }

    pub fn height(&self) -> u64 {
        match self {
            Self::Proposal { block, .. } => block.header.height,
            Self::Prevote(v) | Self::Precommit(v) => v.height,
            Self::QuorumCert(qc) => qc.height,
            Self::ViewChange(vc) => vc.height,
            Self::Ping { height, .. } => *height,
            Self::SyncRequest { from_height, .. } => *from_height,
            Self::SyncResponse { blocks, .. } => blocks.first().map(|b| b.header.height).unwrap_or(0),
        }
    }

    pub fn round(&self) -> u32 {
        match self {
            Self::Proposal { block, .. } => block.header.round,
            Self::Prevote(v) | Self::Precommit(v) => v.round,
            Self::QuorumCert(qc) => qc.round,
            Self::ViewChange(vc) => vc.new_round,
            Self::Ping { round, .. } => *round,
            Self::SyncRequest { .. } | Self::SyncResponse { .. } => 0,
        }
    }
}

impl CanonicalEncode for ConsensusMsg {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Proposal { block, proposer_sig } => {
                write_u8(out, 0x01);
                block.encode(out);
                crate::canonical::write_bytes(out, proposer_sig);
            }
            Self::Prevote(vote) => {
                write_u8(out, 0x02);
                vote.encode(out);
            }
            Self::Precommit(vote) => {
                write_u8(out, 0x03);
                vote.encode(out);
            }
            Self::QuorumCert(qc) => {
                write_u8(out, 0x04);
                qc.encode(out);
            }
            Self::ViewChange(vc) => {
                write_u8(out, 0x05);
                vc.encode(out);
            }
            Self::Ping { height, round, sender_pk_hash } => {
                write_u8(out, 0x10);
                write_u64(out, *height);
                write_u32(out, *round);
                write_fixed(out, sender_pk_hash);
            }
            Self::SyncRequest { from_height, to_height, requester_pk_hash } => {
                write_u8(out, 0x20);
                write_u64(out, *from_height);
                write_u64(out, *to_height);
                write_fixed(out, requester_pk_hash);
            }
            Self::SyncResponse { blocks, qcs, sender_pk_hash } => {
                write_u8(out, 0x21);
                write_vec(out, blocks);
                write_vec(out, qcs);
                write_fixed(out, sender_pk_hash);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::{TX_TYPE_TRANSFER_NOTE, TransferNoteTx, TxCore};

    #[test]
    fn consensus_msg_type_and_height_round() {
        let vote = Vote {
            height: 42,
            round: 7,
            block_hash: [0xAA; 32],
            vote_type: VoteType::Prevote,
            validator_pk: vec![1, 2, 3],
            falcon_sig: vec![4, 5, 6],
        };

        let msg = ConsensusMsg::Prevote(vote.clone());
        assert_eq!(msg.msg_type(), "prevote");
        assert_eq!(msg.height(), 42);
        assert_eq!(msg.round(), 7);

        let msg2 = ConsensusMsg::Precommit(vote);
        assert_eq!(msg2.msg_type(), "precommit");
    }

    #[test]
    fn consensus_msg_view_change() {
        let vc = ViewChange {
            height: 10,
            new_round: 5,
            validator_pk: vec![1, 2, 3],
            falcon_sig: vec![],
        };
        let msg = ConsensusMsg::ViewChange(vc);
        assert_eq!(msg.msg_type(), "view_change");
        assert_eq!(msg.height(), 10);
        assert_eq!(msg.round(), 5);
    }

    #[test]
    fn consensus_msg_ping() {
        let msg = ConsensusMsg::Ping {
            height: 100,
            round: 3,
            sender_pk_hash: [0xBB; 32],
        };
        assert_eq!(msg.msg_type(), "ping");
        assert_eq!(msg.height(), 100);
        assert_eq!(msg.round(), 3);
    }

    #[test]
    fn block_roots_validate() {
        let tx = Transaction::TransferNote(TransferNoteTx {
            core: TxCore {
                version: 0,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: Vec::new(),
                input_nullifiers: Vec::new(),
                outputs: Vec::new(),
                fee: 0,
                statement_commit: [11; 32],
                auth: Vec::new(),
            },
        });
        let block = Block::from_template(BlockTemplate {
            chain_id: 7,
            height: 1,
            epoch: 0,
            round: 0,
            timestamp_ms: 1,
            prev_block_hash: [0; 32],
            proposer_pk_hash: [1; 32],
            epoch_seed_hash: [2; 32],
            parent_qc_hash: [3; 32],
            txs: vec![tx],
            execution_bundle: ExecutionBundle {
                statement_commits: vec![[11; 32]],
                covered_tx_indexes: vec![0],
                public_inputs_root: [4; 32],
                execution_mode: ExecutionMode::FullBatchProof,
            },
            proof_certificates: Vec::new(),
            extra_receipts: Vec::new(),
        });

        assert!(block.roots_match());
    }
}
