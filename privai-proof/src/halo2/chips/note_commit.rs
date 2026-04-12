use std::array;

use halo2_gadgets::poseidon::{
    primitives::{self as poseidon, ConstantLength, P128Pow5T3},
    Hash, Pow5Chip, Pow5Config,
};
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Instance},
};

const POSEIDON_WIDTH: usize = 3;
const POSEIDON_RATE: usize = 2;

/// Scaffold config for Poseidon-based note commitment opening.
#[derive(Clone, Debug)]
pub struct NoteCommitConfig {
    pub poseidon: Pow5Config<Fp, POSEIDON_WIDTH, POSEIDON_RATE>,
    pub note_commit: Column<Instance>,
    pub spend_policy_commit: Column<Advice>,
    pub ct_amt_commit: Column<Advice>,
    pub aux_commit: Column<Advice>,
    pub recipient_box_commit: Column<Advice>,
    pub blinding: Column<Advice>,
}

#[derive(Clone, Debug)]
pub struct NoteCommitChip {
    config: NoteCommitConfig,
}

impl NoteCommitChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> NoteCommitConfig {
        let state = array::from_fn(|_| meta.advice_column());
        let partial_sbox = meta.advice_column();
        let rc_a = array::from_fn(|_| meta.fixed_column());
        let rc_b = array::from_fn(|_| meta.fixed_column());
        let note_commit = meta.instance_column();
        let spend_policy_commit = meta.advice_column();
        let ct_amt_commit = meta.advice_column();
        let aux_commit = meta.advice_column();
        let recipient_box_commit = meta.advice_column();
        let blinding = meta.advice_column();

        meta.enable_constant(rc_b[0]);
        meta.enable_equality(note_commit);
        for column in state {
            meta.enable_equality(column);
        }
        meta.enable_equality(spend_policy_commit);
        meta.enable_equality(ct_amt_commit);
        meta.enable_equality(aux_commit);
        meta.enable_equality(recipient_box_commit);
        meta.enable_equality(blinding);

        let poseidon = Pow5Chip::configure::<P128Pow5T3>(meta, state, partial_sbox, rc_a, rc_b);

        NoteCommitConfig {
            poseidon,
            note_commit,
            spend_policy_commit,
            ct_amt_commit,
            aux_commit,
            recipient_box_commit,
            blinding,
        }
    }

    pub fn new(config: NoteCommitConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &NoteCommitConfig {
        &self.config
    }

    pub fn poseidon_hash(
        spend_policy_commit: Fp,
        ct_amt_commit: Fp,
        aux_commit: Fp,
        recipient_box_commit: Fp,
        blinding: Fp,
    ) -> Fp {
        poseidon::Hash::<_, P128Pow5T3, ConstantLength<5>, POSEIDON_WIDTH, POSEIDON_RATE>::init()
            .hash([
                spend_policy_commit,
                ct_amt_commit,
                aux_commit,
                recipient_box_commit,
                blinding,
            ])
    }

    pub fn assign_with_cells(
        &self,
        mut layouter: impl Layouter<Fp>,
        spend_policy_commit: Fp,
        ct_amt_commit: AssignedCell<Fp, Fp>,
        aux_commit: Fp,
        recipient_box_commit: Fp,
        blinding: Fp,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let (
            spend_policy_commit_cell,
            ct_amt_commit_cell,
            aux_commit_cell,
            recipient_box_commit_cell,
            blinding_cell,
        ) = layouter.assign_region(
            || "load note_commit inputs",
            |mut region| {
                let spend_policy_commit_cell = region.assign_advice(
                    || "spend_policy_commit",
                    self.config.spend_policy_commit,
                    0,
                    || Value::known(spend_policy_commit),
                )?;
                let ct_amt_commit_cell = ct_amt_commit.copy_advice(
                    || "ct_amt_commit",
                    &mut region,
                    self.config.ct_amt_commit,
                    0,
                )?;
                let aux_commit_cell = region.assign_advice(
                    || "aux_commit",
                    self.config.aux_commit,
                    0,
                    || Value::known(aux_commit),
                )?;
                let recipient_box_commit_cell = region.assign_advice(
                    || "recipient_box_commit",
                    self.config.recipient_box_commit,
                    0,
                    || Value::known(recipient_box_commit),
                )?;
                let blinding_cell = region.assign_advice(
                    || "blinding",
                    self.config.blinding,
                    0,
                    || Value::known(blinding),
                )?;

                Ok((
                    spend_policy_commit_cell,
                    ct_amt_commit_cell,
                    aux_commit_cell,
                    recipient_box_commit_cell,
                    blinding_cell,
                ))
            },
        )?;

        let chip = Pow5Chip::construct(self.config.poseidon.clone());
        let hasher =
            Hash::<_, _, P128Pow5T3, ConstantLength<5>, POSEIDON_WIDTH, POSEIDON_RATE>::init(
                chip,
                layouter.namespace(|| "init poseidon"),
            )?;
        let output = hasher.hash(
            layouter.namespace(|| "hash note_commit"),
            [
                spend_policy_commit_cell,
                ct_amt_commit_cell,
                aux_commit_cell,
                recipient_box_commit_cell,
                blinding_cell,
            ],
        )?;

        layouter.constrain_instance(output.cell(), self.config.note_commit, 0)?;
        Ok(output)
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        spend_policy_commit: Fp,
        ct_amt_commit: Fp,
        aux_commit: Fp,
        recipient_box_commit: Fp,
        blinding: Fp,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let ct_amt_commit_cell = layouter.assign_region(
            || "load standalone ct_amt_commit",
            |mut region| {
                region.assign_advice(
                    || "ct_amt_commit",
                    self.config.ct_amt_commit,
                    0,
                    || Value::known(ct_amt_commit),
                )
            },
        )?;

        self.assign_with_cells(
            layouter,
            spend_policy_commit,
            ct_amt_commit_cell,
            aux_commit,
            recipient_box_commit,
            blinding,
        )
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

    use super::{NoteCommitChip, NoteCommitConfig};

    #[derive(Clone, Default)]
    struct NoteCommitCircuit {
        spend_policy_commit: Option<Fp>,
        ct_amt_commit: Option<Fp>,
        aux_commit: Option<Fp>,
        recipient_box_commit: Option<Fp>,
        blinding: Option<Fp>,
    }

    impl Circuit<Fp> for NoteCommitCircuit {
        type Config = NoteCommitConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
            NoteCommitChip::configure(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            layouter: impl Layouter<Fp>,
        ) -> Result<(), Error> {
            let _output = NoteCommitChip::new(config).assign(
                layouter,
                self.spend_policy_commit.ok_or(Error::Synthesis)?,
                self.ct_amt_commit.ok_or(Error::Synthesis)?,
                self.aux_commit.ok_or(Error::Synthesis)?,
                self.recipient_box_commit.ok_or(Error::Synthesis)?,
                self.blinding.ok_or(Error::Synthesis)?,
            )?;
            Ok(())
        }
    }

    #[test]
    fn note_commit_chip_accepts_expected_public_output() {
        let spend_policy_commit = Fp::from(11);
        let ct_amt_commit = Fp::from(22);
        let aux_commit = Fp::from(33);
        let recipient_box_commit = Fp::from(44);
        let blinding = Fp::from(55);
        let expected = NoteCommitChip::poseidon_hash(
            spend_policy_commit,
            ct_amt_commit,
            aux_commit,
            recipient_box_commit,
            blinding,
        );

        let circuit = NoteCommitCircuit {
            spend_policy_commit: Some(spend_policy_commit),
            ct_amt_commit: Some(ct_amt_commit),
            aux_commit: Some(aux_commit),
            recipient_box_commit: Some(recipient_box_commit),
            blinding: Some(blinding),
        };

        let prover = MockProver::run(7, &circuit, vec![vec![expected]]).expect("mock prover");
        prover.assert_satisfied();
    }

    #[test]
    fn note_commit_chip_rejects_wrong_public_output() {
        let circuit = NoteCommitCircuit {
            spend_policy_commit: Some(Fp::from(11)),
            ct_amt_commit: Some(Fp::from(22)),
            aux_commit: Some(Fp::from(33)),
            recipient_box_commit: Some(Fp::from(44)),
            blinding: Some(Fp::from(55)),
        };

        let prover = MockProver::run(7, &circuit, vec![vec![Fp::from(999)]]).expect("mock prover");
        assert!(prover.verify().is_err());
    }
}
