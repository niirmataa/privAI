use std::array;

use halo2_gadgets::poseidon::{
    Hash, Pow5Chip, Pow5Config,
    primitives::{self as poseidon, ConstantLength, P128Pow5T3},
};
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Instance},
};

const POSEIDON_WIDTH: usize = 3;
const POSEIDON_RATE: usize = 2;

/// Real Poseidon-based nullifier gadget for the v0 proof scaffold.
///
/// The exact privAI domain separation parameters may still evolve, but the
/// circuit now uses an actual Halo2 Poseidon gadget instead of a placeholder
/// arithmetic relation.
#[derive(Clone, Debug)]
pub struct NullifierConfig {
    pub poseidon: Pow5Config<Fp, POSEIDON_WIDTH, POSEIDON_RATE>,
    pub note_commit: Column<Advice>,
    pub nullifier_key: Column<Advice>,
    pub public_inputs: Column<Instance>,
}

#[derive(Clone, Debug)]
pub struct NullifierChip {
    config: NullifierConfig,
}

impl NullifierChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> NullifierConfig {
        let state = array::from_fn(|_| meta.advice_column());
        let partial_sbox = meta.advice_column();
        let rc_a = array::from_fn(|_| meta.fixed_column());
        let rc_b = array::from_fn(|_| meta.fixed_column());
        let note_commit = meta.advice_column();
        let nullifier_key = meta.advice_column();
        let public_inputs = meta.instance_column();

        meta.enable_constant(rc_b[0]);
        meta.enable_equality(public_inputs);
        for column in state {
            meta.enable_equality(column);
        }
        meta.enable_equality(note_commit);
        meta.enable_equality(nullifier_key);

        let poseidon = Pow5Chip::configure::<P128Pow5T3>(
            meta,
            state,
            partial_sbox,
            rc_a,
            rc_b,
        );

        NullifierConfig {
            poseidon,
            note_commit,
            nullifier_key,
            public_inputs,
        }
    }

    pub fn new(config: NullifierConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &NullifierConfig {
        &self.config
    }

    pub fn poseidon_hash(note_commit: Fp, nullifier_key: Fp) -> Fp {
        poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, POSEIDON_WIDTH, POSEIDON_RATE>::init()
            .hash([note_commit, nullifier_key])
    }

    pub fn assign_with_note_commit_cell(
        &self,
        mut layouter: impl Layouter<Fp>,
        note_commit: AssignedCell<Fp, Fp>,
        nullifier_key: Fp,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let (note_commit_cell, nullifier_key_cell) = layouter.assign_region(
            || "load nullifier inputs",
            |mut region| {
                let note_commit_cell = note_commit.copy_advice(
                    || "note_commit",
                    &mut region,
                    self.config.note_commit,
                    0,
                )?;
                let nullifier_key_cell = region.assign_advice(
                    || "nullifier_key",
                    self.config.nullifier_key,
                    0,
                    || Value::known(nullifier_key),
                )?;
                Ok((note_commit_cell, nullifier_key_cell))
            },
        )?;

        let chip = Pow5Chip::construct(self.config.poseidon.clone());
        let hasher =
            Hash::<_, _, P128Pow5T3, ConstantLength<2>, POSEIDON_WIDTH, POSEIDON_RATE>::init(
                chip,
                layouter.namespace(|| "init poseidon"),
            )?;
        let output = hasher.hash(
            layouter.namespace(|| "hash nullifier"),
            [note_commit_cell.clone(), nullifier_key_cell],
        )?;

        layouter.constrain_instance(note_commit_cell.cell(), self.config.public_inputs, 0)?;
        layouter.constrain_instance(output.cell(), self.config.public_inputs, 1)?;
        Ok(output)
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        note_commit: Fp,
        nullifier_key: Fp,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let note_commit_cell = layouter.assign_region(
            || "load standalone note_commit",
            |mut region| {
                region.assign_advice(
                    || "note_commit",
                    self.config.note_commit,
                    0,
                    || Value::known(note_commit),
                )
            },
        )?;

        self.assign_with_note_commit_cell(layouter, note_commit_cell, nullifier_key)
    }
}

#[cfg(test)]
mod tests {
    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
        pasta::Fp,
        plonk::{Circuit, ConstraintSystem, Error},
    };

    use super::{NullifierChip, NullifierConfig};

    #[derive(Clone, Default)]
    struct NullifierCircuit {
        note_commit: Option<Fp>,
        nullifier_key: Option<Fp>,
    }

    impl Circuit<Fp> for NullifierCircuit {
        type Config = NullifierConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
            NullifierChip::configure(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            layouter: impl Layouter<Fp>,
        ) -> Result<(), Error> {
            let note_commit = self.note_commit.ok_or(Error::Synthesis)?;
            let nullifier_key = self.nullifier_key.ok_or(Error::Synthesis)?;
            let _output = NullifierChip::new(config).assign(layouter, note_commit, nullifier_key)?;
            Ok(())
        }
    }

    #[test]
    fn nullifier_chip_accepts_expected_public_output() {
        let note_commit = Fp::from(17);
        let nullifier_key = Fp::from(29);
        let expected = NullifierChip::poseidon_hash(note_commit, nullifier_key);

        let circuit = NullifierCircuit {
            note_commit: Some(note_commit),
            nullifier_key: Some(nullifier_key),
        };

        let prover = MockProver::run(6, &circuit, vec![vec![note_commit, expected]])
            .expect("mock prover");
        prover.assert_satisfied();
    }

    #[test]
    fn nullifier_chip_rejects_wrong_public_output() {
        let note_commit = Fp::from(17);
        let nullifier_key = Fp::from(29);
        let wrong = Fp::from(999);

        let circuit = NullifierCircuit {
            note_commit: Some(note_commit),
            nullifier_key: Some(nullifier_key),
        };

        let prover = MockProver::run(6, &circuit, vec![vec![note_commit, wrong]])
            .expect("mock prover");
        assert!(prover.verify().is_err());
    }
}
