use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    pasta::Fp,
    plonk::{Circuit, ConstraintSystem, Error},
};

use crate::halo2::{
    AmountCipherParams, LweAmountChip, LweAmountConfig, NoiseClassChip, NoiseClassConfig,
    NoteCommitChip, NoteCommitConfig, NullifierChip, NullifierConfig,
};

#[derive(Clone, Debug)]
pub struct PrivaiTxSkeletonConfig {
    pub lwe_amount: LweAmountConfig,
    pub noise_class: NoiseClassConfig,
    pub note_commit: NoteCommitConfig,
    pub nullifier: NullifierConfig,
}

/// Minimal composition circuit for privAI v0.
///
/// This now includes real cell-to-cell wiring from `LweAmountChip` into
/// `NoteCommitChip` through `ct_amt_commit`. Remaining duplicated witnesses
/// (for example consumed-note loading and noise wiring) will be replaced with
/// shared cells in the next iteration.
#[derive(Clone, Debug)]
pub struct PrivaiTxSkeletonCircuit {
    pub output_u: [u32; 512],
    pub output_v: u32,
    pub output_t: [u32; 512],
    pub output_e1: [i16; 512],
    pub output_e2: i16,
    pub output_noise_class: u8,
    pub output_spend_policy_commit: Fp,
    pub output_aux_commit: Fp,
    pub output_recipient_box_commit: Fp,
    pub output_blinding: Fp,
    pub consumed_note_commit: Fp,
    pub consumed_nullifier_key: Fp,
}

impl Default for PrivaiTxSkeletonCircuit {
    fn default() -> Self {
        Self {
            output_u: [0; 512],
            output_v: 0,
            output_t: [0; 512],
            output_e1: [0; 512],
            output_e2: 0,
            output_noise_class: 0,
            output_spend_policy_commit: Fp::from(0u64),
            output_aux_commit: Fp::from(0u64),
            output_recipient_box_commit: Fp::from(0u64),
            output_blinding: Fp::from(0u64),
            consumed_note_commit: Fp::from(0u64),
            consumed_nullifier_key: Fp::from(0u64),
        }
    }
}

impl Circuit<Fp> for PrivaiTxSkeletonCircuit {
    type Config = PrivaiTxSkeletonConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        Self::Config {
            lwe_amount: LweAmountChip::configure(meta),
            noise_class: NoiseClassChip::configure(meta),
            note_commit: NoteCommitChip::configure(meta),
            nullifier: NullifierChip::configure(meta),
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let lwe_amount_chip =
            LweAmountChip::new(config.lwe_amount, AmountCipherParams::default());
        let noise_class_chip = NoiseClassChip::new(config.noise_class);
        let note_commit_chip = NoteCommitChip::new(config.note_commit);
        let nullifier_chip = NullifierChip::new(config.nullifier);

        noise_class_chip.load_lookup_table(layouter.namespace(|| "load noise table"))?;

        let ct_amt_commit = LweAmountChip::poseidon_hash_ct_amt(&self.output_u, self.output_v);
        let output_note_commit = NoteCommitChip::poseidon_hash(
            self.output_spend_policy_commit,
            ct_amt_commit,
            self.output_aux_commit,
            self.output_recipient_box_commit,
            self.output_blinding,
        );

        let amount_outputs = lwe_amount_chip.assign(
            layouter.namespace(|| "assign lwe amount"),
            &self.output_u,
            self.output_v,
            &self.output_t,
        )?;

        noise_class_chip.assign(
            layouter.namespace(|| "assign output noise"),
            &self.output_e1,
            self.output_e2,
            self.output_noise_class,
        )?;

        let _output_note_commit_cell = note_commit_chip.assign_with_cells(
            layouter.namespace(|| "assign output note commit"),
            self.output_spend_policy_commit,
            amount_outputs.ct_amt_commit,
            self.output_aux_commit,
            self.output_recipient_box_commit,
            self.output_blinding,
        )?;

        let consumed_note_commit_cell = layouter.assign_region(
            || "load consumed note commit",
            |mut region| {
                region.assign_advice(
                    || "consumed_note_commit",
                    nullifier_chip.config().note_commit,
                    0,
                    || halo2_proofs::circuit::Value::known(self.consumed_note_commit),
                )
            },
        )?;

        let _consumed_nullifier_cell = nullifier_chip.assign_with_note_commit_cell(
            layouter.namespace(|| "assign consumed nullifier"),
            consumed_note_commit_cell,
            self.consumed_nullifier_key,
        )?;

        let _ = output_note_commit;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::array;

    use halo2_proofs::dev::MockProver;

    use super::*;

    #[test]
    fn tx_skeleton_circuit_composes_existing_halo2_chips() {
        let output_u = array::from_fn(|i| (i as u32).wrapping_mul(7).wrapping_add(5));
        let output_t = array::from_fn(|i| (i as u32).wrapping_mul(13).wrapping_add(9));
        let output_e1 = array::from_fn(|i| match i % 4 {
            0 => 0,
            1 => 4,
            2 => -8,
            _ => 16,
        });
        let output_v = 0x1357_2468;
        let output_noise_class = 0;
        let output_spend_policy_commit = Fp::from(101);
        let output_aux_commit = Fp::from(202);
        let output_recipient_box_commit = Fp::from(303);
        let output_blinding = Fp::from(404);
        let consumed_note_commit = Fp::from(505);
        let consumed_nullifier_key = Fp::from(606);

        let ct_amt_commit = LweAmountChip::poseidon_hash_ct_amt(&output_u, output_v);
        let t_commit = LweAmountChip::poseidon_hash_t(&output_t);
        let output_note_commit = NoteCommitChip::poseidon_hash(
            output_spend_policy_commit,
            ct_amt_commit,
            output_aux_commit,
            output_recipient_box_commit,
            output_blinding,
        );
        let nullifier = NullifierChip::poseidon_hash(consumed_note_commit, consumed_nullifier_key);

        let circuit = PrivaiTxSkeletonCircuit {
            output_u,
            output_v,
            output_t,
            output_e1,
            output_e2: -12,
            output_noise_class,
            output_spend_policy_commit,
            output_aux_commit,
            output_recipient_box_commit,
            output_blinding,
            consumed_note_commit,
            consumed_nullifier_key,
        };

        let prover = MockProver::run(
            15,
            &circuit,
            vec![
                vec![ct_amt_commit],
                vec![t_commit],
                vec![output_note_commit],
                vec![consumed_note_commit, nullifier],
            ],
        )
        .expect("mock prover");
        prover.assert_satisfied();
    }
}
