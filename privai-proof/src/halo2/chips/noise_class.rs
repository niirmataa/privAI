use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Selector, TableColumn},
    poly::Rotation,
};

pub const LWE_DIMENSION_V0: usize = 512;
pub const ACCEPTED_NOISE_CLASSES_V0: [u8; 3] = [0, 1, 2];
pub const NOISE_CLASS_BOUNDS_V0: [u16; 3] = [16, 32, 64];

#[derive(Clone, Debug)]
pub struct NoiseClassConfig {
    pub q_enable: Selector,
    pub noise_value: Column<Advice>,
    pub noise_abs: Column<Advice>,
    pub noise_sign: Column<Advice>,
    pub noise_class: Column<Advice>,
    pub table_class: TableColumn,
    pub table_abs: TableColumn,
}

#[derive(Clone, Debug)]
pub struct NoiseClassChip {
    config: NoiseClassConfig,
}

#[derive(Clone, Debug)]
pub struct NoiseClassOutputs {
    pub e1_cells: [AssignedCell<Fp, Fp>; LWE_DIMENSION_V0],
    pub e2_cell: AssignedCell<Fp, Fp>,
}

impl NoiseClassChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> NoiseClassConfig {
        let q_enable = meta.complex_selector();
        let noise_value = meta.advice_column();
        let noise_abs = meta.advice_column();
        let noise_sign = meta.advice_column();
        let noise_class = meta.advice_column();
        let table_class = meta.lookup_table_column();
        let table_abs = meta.lookup_table_column();

        meta.enable_equality(noise_value);
        meta.enable_equality(noise_abs);
        meta.enable_equality(noise_sign);
        meta.enable_equality(noise_class);

        meta.create_gate("noise sign bit is boolean", |meta| {
            let q = meta.query_selector(q_enable);
            let sign = meta.query_advice(noise_sign, Rotation::cur());
            vec![q * sign.clone() * (sign - halo2_proofs::plonk::Expression::Constant(Fp::from(1u64)))]
        });

        meta.create_gate("noise value matches abs and sign", |meta| {
            let q = meta.query_selector(q_enable);
            let value = meta.query_advice(noise_value, Rotation::cur());
            let abs = meta.query_advice(noise_abs, Rotation::cur());
            let sign = meta.query_advice(noise_sign, Rotation::cur());
            let one = halo2_proofs::plonk::Expression::Constant(Fp::from(1u64));
            let two = halo2_proofs::plonk::Expression::Constant(Fp::from(2u64));

            vec![q * (value - abs * (one - two * sign))]
        });

        meta.lookup(|meta| {
            let q = meta.query_selector(q_enable);
            let class = meta.query_advice(noise_class, Rotation::cur());
            let abs = meta.query_advice(noise_abs, Rotation::cur());

            // Invariant: (0, 0) must always exist in the lookup table because
            // disabled rows query (0, 0) when q_enable == 0.
            vec![
                (q.clone() * class, table_class),
                (q * abs, table_abs),
            ]
        });

        NoiseClassConfig {
            q_enable,
            noise_value,
            noise_abs,
            noise_sign,
            noise_class,
            table_class,
            table_abs,
        }
    }

    pub fn new(config: NoiseClassConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &NoiseClassConfig {
        &self.config
    }

    pub fn noise_bound_for_class(class: u8) -> Option<u16> {
        ACCEPTED_NOISE_CLASSES_V0
            .iter()
            .position(|accepted| *accepted == class)
            .map(|idx| NOISE_CLASS_BOUNDS_V0[idx])
    }

    fn signed_i16_to_field(value: i16) -> Fp {
        if value >= 0 {
            Fp::from(value as u64)
        } else {
            -Fp::from((-value) as u64)
        }
    }

    pub fn load_lookup_table(&self, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        layouter.assign_table(
            || "noise class lookup table",
            |mut table| {
                let mut offset = 0;
                for (class, bound) in ACCEPTED_NOISE_CLASSES_V0
                    .iter()
                    .zip(NOISE_CLASS_BOUNDS_V0.iter())
                {
                    for abs in 0..=*bound {
                        table.assign_cell(
                            || format!("table class {class} row {offset}"),
                            self.config.table_class,
                            offset,
                            || Value::known(Fp::from(*class as u64)),
                        )?;
                        table.assign_cell(
                            || format!("table abs {abs} row {offset}"),
                            self.config.table_abs,
                            offset,
                            || Value::known(Fp::from(abs as u64)),
                        )?;
                        offset += 1;
                    }
                }
                Ok(())
            },
        )
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        e1: &[i16; LWE_DIMENSION_V0],
        e2: i16,
        noise_class: u8,
    ) -> Result<NoiseClassOutputs, Error> {
        if Self::noise_bound_for_class(noise_class).is_none() {
            return Err(Error::Synthesis);
        }

        // `noise_value` is the canonical inter-chip representation of fresh
        // encryption noise: centered signed field elements. `LweAmountChip`
        // copies these same cells in the next stage, and later well-formedness
        // constraints will connect them to `mod 2^32` ciphertext arithmetic.
        layouter.assign_region(
            || "assign noise witnesses",
            |mut region| {
                let noise_class_cell = region.assign_advice(
                    || "noise_class_0",
                    self.config.noise_class,
                    0,
                    || Value::known(Fp::from(noise_class as u64)),
                )?;
                let mut assigned_values = Vec::with_capacity(LWE_DIMENSION_V0 + 1);
                for (offset, value) in e1
                    .iter()
                    .copied()
                    .chain(std::iter::once(e2))
                    .enumerate()
                {
                    let abs = value.unsigned_abs() as u64;
                    let sign = if value < 0 { 1u64 } else { 0u64 };

                    self.config.q_enable.enable(&mut region, offset)?;

                    let noise_value_cell = region.assign_advice(
                        || format!("noise_value_{offset}"),
                        self.config.noise_value,
                        offset,
                        || Value::known(Self::signed_i16_to_field(value)),
                    )?;
                    assigned_values.push(noise_value_cell);
                    region.assign_advice(
                        || format!("noise_abs_{offset}"),
                        self.config.noise_abs,
                        offset,
                        || Value::known(Fp::from(abs)),
                    )?;
                    region.assign_advice(
                        || format!("noise_sign_{offset}"),
                        self.config.noise_sign,
                        offset,
                        || Value::known(Fp::from(sign)),
                    )?;
                    if offset > 0 {
                        noise_class_cell.copy_advice(
                            || format!("noise_class_copy_{offset}"),
                            &mut region,
                            self.config.noise_class,
                            offset,
                        )?;
                    }
                }

                let e2_cell = assigned_values.pop().ok_or(Error::Synthesis)?;
                let e1_cells = assigned_values.try_into().map_err(|_| Error::Synthesis)?;
                Ok(NoiseClassOutputs { e1_cells, e2_cell })
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use std::array;

    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
        pasta::Fp,
        plonk::{Circuit, ConstraintSystem, Error},
    };

    use super::{LWE_DIMENSION_V0, NoiseClassChip, NoiseClassConfig};

    #[derive(Clone)]
    struct NoiseClassCircuit {
        e1: [i16; LWE_DIMENSION_V0],
        e2: i16,
        noise_class: u8,
    }

    impl Default for NoiseClassCircuit {
        fn default() -> Self {
            Self {
                e1: [0; LWE_DIMENSION_V0],
                e2: 0,
                noise_class: 0,
            }
        }
    }

    impl Circuit<Fp> for NoiseClassCircuit {
        type Config = NoiseClassConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
            NoiseClassChip::configure(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<Fp>,
        ) -> Result<(), Error> {
            let chip = NoiseClassChip::new(config);
            chip.load_lookup_table(layouter.namespace(|| "load noise table"))?;
            let _noise_outputs = chip.assign(
                layouter.namespace(|| "assign noise witnesses"),
                &self.e1,
                self.e2,
                self.noise_class,
            )?;
            Ok(())
        }
    }

    #[test]
    fn noise_class_chip_accepts_valid_linf_noise() {
        let e1 = array::from_fn(|i| match i % 5 {
            0 => 0,
            1 => 3,
            2 => -7,
            3 => 16,
            _ => -16,
        });
        let circuit = NoiseClassCircuit {
            e1,
            e2: -12,
            noise_class: 0,
        };

        let prover = MockProver::run(11, &circuit, vec![]).expect("mock prover");
        prover.assert_satisfied();
    }

    #[test]
    fn noise_class_chip_rejects_noise_over_class_bound() {
        let mut e1 = [0i16; LWE_DIMENSION_V0];
        e1[17] = 17;

        let circuit = NoiseClassCircuit {
            e1,
            e2: 0,
            noise_class: 0,
        };

        let prover = MockProver::run(11, &circuit, vec![]).expect("mock prover");
        assert!(prover.verify().is_err());
    }

    #[test]
    fn noise_class_chip_rejects_negative_noise_over_class_bound() {
        let mut e1 = [0i16; LWE_DIMENSION_V0];
        e1[0] = -17;

        let circuit = NoiseClassCircuit {
            e1,
            e2: 0,
            noise_class: 0,
        };

        let prover = MockProver::run(11, &circuit, vec![]).expect("mock prover");
        assert!(prover.verify().is_err());
    }

    #[test]
    fn noise_class_chip_rejects_e2_over_class_bound() {
        let circuit = NoiseClassCircuit {
            e1: [0i16; LWE_DIMENSION_V0],
            e2: 17,
            noise_class: 0,
        };

        let prover = MockProver::run(11, &circuit, vec![]).expect("mock prover");
        assert!(prover.verify().is_err());
    }

    #[test]
    fn noise_class_chip_rejects_invalid_noise_class() {
        let circuit = NoiseClassCircuit {
            e1: [0i16; LWE_DIMENSION_V0],
            e2: 0,
            noise_class: 9,
        };

        assert!(MockProver::run(11, &circuit, vec![]).is_err());
    }
}
