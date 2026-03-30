use blake3::Hasher;
use privai_chain::{
    Amount14, CanonicalEncode, Hash32, Nullifier, RecipientBoxPlaintext, TransferNoteTx,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const TRANSFER_STATEMENT_DOMAIN_V0: &[u8] = b"privai:proof:transfer-statement:v0";
const TRANSFER_PUBLIC_INPUTS_DOMAIN_V0: &[u8] = b"privai:proof:transfer-public-inputs:v0";
const TRANSFER_WITNESS_DOMAIN_V0: &[u8] = b"privai:proof:transfer-witness:v0";
const PROOF_JOB_ID_DOMAIN_V0: &[u8] = b"privai:proof:job-id:v0";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferStatement {
    pub input_note_commits: Vec<Hash32>,
    pub input_nullifiers: Vec<Nullifier>,
    pub output_note_commits: Vec<Hash32>,
    pub fee: u64,
}

impl TransferStatement {
    pub fn from_tx(tx: &TransferNoteTx) -> Self {
        Self {
            input_note_commits: tx.core.inputs.iter().map(|input| input.note_commit).collect(),
            input_nullifiers: tx.core.input_nullifiers.clone(),
            output_note_commits: tx.core.outputs.iter().map(|output| output.note_commit).collect(),
            fee: tx.core.fee,
        }
    }

    pub fn commitment(&self) -> Hash32 {
        hash_with_domain(TRANSFER_STATEMENT_DOMAIN_V0, &[&self.to_canonical_bytes()])
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_hash_vec(&mut out, &self.input_note_commits);
        write_nullifier_vec(&mut out, &self.input_nullifiers);
        write_hash_vec(&mut out, &self.output_note_commits);
        write_u64(&mut out, self.fee);
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferPublicInputs {
    pub tx_id: Hash32,
    pub statement_commit: Hash32,
    pub input_note_commits: Vec<Hash32>,
    pub input_nullifiers: Vec<Nullifier>,
    pub output_note_commits: Vec<Hash32>,
    pub fee: u64,
}

impl TransferPublicInputs {
    pub fn from_tx(tx: &TransferNoteTx) -> Self {
        Self {
            tx_id: privai_chain::Transaction::TransferNote(tx.clone()).tx_id(),
            statement_commit: tx.core.statement_commit,
            input_note_commits: tx.core.inputs.iter().map(|input| input.note_commit).collect(),
            input_nullifiers: tx.core.input_nullifiers.clone(),
            output_note_commits: tx.core.outputs.iter().map(|output| output.note_commit).collect(),
            fee: tx.core.fee,
        }
    }

    pub fn hash(&self) -> Hash32 {
        hash_with_domain(TRANSFER_PUBLIC_INPUTS_DOMAIN_V0, &[&self.to_canonical_bytes()])
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_fixed(&mut out, &self.tx_id);
        write_fixed(&mut out, &self.statement_commit);
        write_hash_vec(&mut out, &self.input_note_commits);
        write_nullifier_vec(&mut out, &self.input_nullifiers);
        write_hash_vec(&mut out, &self.output_note_commits);
        write_u64(&mut out, self.fee);
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferInputWitness {
    pub amount: Amount14,
    pub witness_seed: Hash32,
    pub nullifier_key: Hash32,
    pub spend_policy_opening: Vec<u8>,
    pub aux_opening: Vec<u8>,
}

impl TransferInputWitness {
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_u16(&mut out, self.amount.value());
        write_fixed(&mut out, &self.witness_seed);
        write_fixed(&mut out, &self.nullifier_key);
        write_bytes(&mut out, &self.spend_policy_opening);
        write_bytes(&mut out, &self.aux_opening);
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferOutputWitness {
    pub note_commit: Hash32,
    pub recipient_opening: RecipientBoxPlaintext,
}

impl TransferOutputWitness {
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_fixed(&mut out, &self.note_commit);
        write_bytes(&mut out, &self.recipient_opening.to_canonical_bytes());
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferWitness {
    pub input: TransferInputWitness,
    pub outputs: Vec<TransferOutputWitness>,
}

impl TransferWitness {
    pub fn commitment(&self) -> Hash32 {
        hash_with_domain(TRANSFER_WITNESS_DOMAIN_V0, &[&self.to_canonical_bytes()])
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_bytes(&mut out, &self.input.to_canonical_bytes());
        write_u32(&mut out, self.outputs.len() as u32);
        for output in &self.outputs {
            write_bytes(&mut out, &output.to_canonical_bytes());
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferProvingData {
    pub statement: TransferStatement,
    pub public_inputs: TransferPublicInputs,
    pub witness: TransferWitness,
}

impl TransferProvingData {
    pub fn from_tx_and_witness(
        tx: &TransferNoteTx,
        witness: TransferWitness,
    ) -> Result<Self, TransferBuildError> {
        let statement = TransferStatement::from_tx(tx);
        let expected_statement_commit = statement.commitment();
        if tx.core.statement_commit != expected_statement_commit {
            return Err(TransferBuildError::StatementCommitMismatch {
                expected: expected_statement_commit,
                actual: tx.core.statement_commit,
            });
        }

        if tx.core.outputs.len() != witness.outputs.len() {
            return Err(TransferBuildError::OutputWitnessCountMismatch {
                expected: tx.core.outputs.len(),
                actual: witness.outputs.len(),
            });
        }

        for (index, (note, output_witness)) in tx.core.outputs.iter().zip(witness.outputs.iter()).enumerate() {
            if note.note_commit != output_witness.note_commit {
                return Err(TransferBuildError::OutputNoteCommitMismatch {
                    index,
                    expected: note.note_commit,
                    actual: output_witness.note_commit,
                });
            }
            if note.payload_commit() != output_witness.recipient_opening.note_payload_commit {
                return Err(TransferBuildError::OutputPayloadCommitMismatch { index });
            }
            if note.recipient_box.hint != output_witness.recipient_opening.bundle_id {
                return Err(TransferBuildError::OutputBundleHintMismatch { index });
            }
        }

        Ok(Self {
            statement,
            public_inputs: TransferPublicInputs::from_tx(tx),
            witness,
        })
    }

    pub fn public_inputs_hash(&self) -> Hash32 {
        self.public_inputs.hash()
    }

    pub fn to_proof_job(
        &self,
        job_fee: u64,
        deadline_height: u64,
        requester_hint: Hash32,
    ) -> super::ProofJob {
        let public_inputs_hash = self.public_inputs_hash();
        let job_id = hash_with_domain(
            PROOF_JOB_ID_DOMAIN_V0,
            &[
                &self.statement.commitment(),
                &public_inputs_hash,
                &deadline_height.to_le_bytes(),
                &requester_hint,
            ],
        );

        super::ProofJob {
            job_id,
            statement_commit: self.statement.commitment(),
            job_fee,
            deadline_height,
            requester_hint,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TransferBuildError {
    #[error("transfer tx statement_commit mismatch: expected {expected:?}, got {actual:?}")]
    StatementCommitMismatch { expected: Hash32, actual: Hash32 },
    #[error("transfer output witness count mismatch: expected {expected}, got {actual}")]
    OutputWitnessCountMismatch { expected: usize, actual: usize },
    #[error("transfer output {index} note_commit mismatch: expected {expected:?}, got {actual:?}")]
    OutputNoteCommitMismatch {
        index: usize,
        expected: Hash32,
        actual: Hash32,
    },
    #[error("transfer output {index} recipient opening payload commit does not match note payload")]
    OutputPayloadCommitMismatch { index: usize },
    #[error("transfer output {index} recipient opening bundle_id does not match note hint")]
    OutputBundleHintMismatch { index: usize },
}

fn hash_with_domain(domain: &[u8], parts: &[&[u8]]) -> Hash32 {
    let mut hasher = Hasher::new();
    write_len_prefixed(&mut hasher, domain);
    for part in parts {
        write_len_prefixed(&mut hasher, part);
    }
    *hasher.finalize().as_bytes()
}

fn write_len_prefixed(hasher: &mut Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u32).to_le_bytes());
    hasher.update(bytes);
}

fn write_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_fixed<const N: usize>(out: &mut Vec<u8>, bytes: &[u8; N]) {
    out.extend_from_slice(bytes);
}

fn write_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    write_u32(out, bytes.len() as u32);
    out.extend_from_slice(bytes);
}

fn write_hash_vec(out: &mut Vec<u8>, hashes: &[Hash32]) {
    write_u32(out, hashes.len() as u32);
    for hash in hashes {
        write_fixed(out, hash);
    }
}

fn write_nullifier_vec(out: &mut Vec<u8>, nullifiers: &[Nullifier]) {
    write_u32(out, nullifiers.len() as u32);
    for nullifier in nullifiers {
        write_fixed(out, &nullifier.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use privai_chain::{
        InputRef, LweCiphertext, OutputNote, RecipientBox, RecipientBoxPlaintext, TransferNoteTx,
        TxCore, TX_TYPE_TRANSFER_NOTE, PRIVAI_V0,
    };

    fn sample_note(seed: u8) -> OutputNote {
        OutputNote::new(
            [seed; 32],
            LweCiphertext::default(),
            [seed.wrapping_add(1); 32],
            RecipientBox::new(vec![seed], [seed; 24], vec![seed.wrapping_add(1)], [seed; 16], [seed; 16]),
        )
    }

    fn sample_tx_and_witness() -> (TransferNoteTx, TransferWitness) {
        let output = sample_note(10);
        let statement = TransferStatement {
            input_note_commits: vec![[9; 32]],
            input_nullifiers: vec![Nullifier([7; 32])],
            output_note_commits: vec![output.note_commit],
            fee: 3,
        };
        let tx = TransferNoteTx {
            core: TxCore {
                version: PRIVAI_V0,
                tx_type: TX_TYPE_TRANSFER_NOTE,
                inputs: vec![InputRef { note_commit: [9; 32] }],
                input_nullifiers: vec![Nullifier([7; 32])],
                outputs: vec![output.clone()],
                fee: 3,
                statement_commit: statement.commitment(),
                auth: Vec::new(),
            },
        };
        let witness = TransferWitness {
            input: TransferInputWitness {
                amount: Amount14::new(42).expect("amount"),
                witness_seed: [1; 32],
                nullifier_key: [2; 32],
                spend_policy_opening: vec![3, 4],
                aux_opening: vec![5, 6],
            },
            outputs: vec![TransferOutputWitness {
                note_commit: output.note_commit,
                recipient_opening: RecipientBoxPlaintext {
                    version: PRIVAI_V0,
                    bundle_id: output.recipient_box.hint,
                    note_payload_commit: output.payload_commit(),
                    amount: Amount14::new(21).expect("amount"),
                    witness_seed: [8; 32],
                    nullifier_key: [9; 32],
                    spend_policy_opening: vec![10],
                    aux_opening: vec![11],
                    sender_memo: None,
                },
            }],
        };
        (tx, witness)
    }

    #[test]
    fn transfer_proving_data_accepts_matching_tx_and_witness() {
        let (tx, witness) = sample_tx_and_witness();
        let proving = TransferProvingData::from_tx_and_witness(&tx, witness).expect("proving");

        assert_eq!(proving.statement.commitment(), tx.core.statement_commit);
        assert_eq!(proving.public_inputs.statement_commit, tx.core.statement_commit);
    }

    #[test]
    fn transfer_proving_data_rejects_statement_commit_mismatch() {
        let (mut tx, witness) = sample_tx_and_witness();
        tx.core.statement_commit = [0xAA; 32];

        assert!(matches!(
            TransferProvingData::from_tx_and_witness(&tx, witness),
            Err(TransferBuildError::StatementCommitMismatch { .. })
        ));
    }

    #[test]
    fn transfer_proving_data_rejects_output_note_commit_mismatch() {
        let (tx, mut witness) = sample_tx_and_witness();
        witness.outputs[0].note_commit = [0xBB; 32];

        assert!(matches!(
            TransferProvingData::from_tx_and_witness(&tx, witness),
            Err(TransferBuildError::OutputNoteCommitMismatch { index: 0, .. })
        ));
    }
}
