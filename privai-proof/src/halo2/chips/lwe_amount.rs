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

use crate::halo2::{
    pack_u32_limbs_to_fp, packed_u32_field_len,
    params::AmountCipherParams,
};

pub const LWE_DIMENSION_V0: usize = 512;
pub const LWE_CIPHERTEXT_LIMBS_V0: usize = LWE_DIMENSION_V0 + 1;
pub const PACKED_LWE_CIPHERTEXT_LEN_V0: usize = packed_u32_field_len(LWE_CIPHERTEXT_LIMBS_V0);
pub const PACKED_LWE_PUBLIC_KEY_LEN_V0: usize = packed_u32_field_len(LWE_DIMENSION_V0);
const POSEIDON_WIDTH: usize = 3;
const POSEIDON_RATE: usize = 2;

/// First concrete stage of the future PKE-LWE amount gadget.
///
/// This stage proves only the public Poseidon commitments:
/// - `ct_amt_commit = Poseidon(pack(u, v))`
/// - `t_commit = Poseidon(pack(t))`
///
/// Full well-formedness (`u = A^T r + e1`, `v = t^T r + e2 + Δ·amount`) will
/// be layered on top of this in the next iteration.
#[derive(Clone, Debug)]
pub struct LweAmountConfig {
    pub poseidon: Pow5Config<Fp, POSEIDON_WIDTH, POSEIDON_RATE>,
    pub packed_word: Column<Advice>,
    pub noise_value: Column<Advice>,
    pub ct_amt_commit: Column<Instance>,
    pub t_commit: Column<Instance>,
}

#[derive(Clone, Debug)]
pub struct LweAmountChip {
    config: LweAmountConfig,
    params: AmountCipherParams,
}

#[derive(Clone, Debug)]
pub struct LweAmountOutputs {
    pub ct_amt_commit: AssignedCell<Fp, Fp>,
    pub t_commit: AssignedCell<Fp, Fp>,
}

impl LweAmountChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> LweAmountConfig {
        let state = array::from_fn(|_| meta.advice_column());
        let partial_sbox = meta.advice_column();
        let rc_a = array::from_fn(|_| meta.fixed_column());
        let rc_b = array::from_fn(|_| meta.fixed_column());
        let packed_word = meta.advice_column();
        let noise_value = meta.advice_column();
        let ct_amt_commit = meta.instance_column();
        let t_commit = meta.instance_column();

        meta.enable_constant(rc_b[0]);
        for column in state {
            meta.enable_equality(column);
        }
        meta.enable_equality(packed_word);
        meta.enable_equality(noise_value);
        meta.enable_equality(ct_amt_commit);
        meta.enable_equality(t_commit);

        let poseidon = Pow5Chip::configure::<P128Pow5T3>(
            meta,
            state,
            partial_sbox,
            rc_a,
            rc_b,
        );

        LweAmountConfig {
            poseidon,
            packed_word,
            noise_value,
            ct_amt_commit,
            t_commit,
        }
    }

    pub fn new(config: LweAmountConfig, params: AmountCipherParams) -> Self {
        Self { config, params }
    }

    pub fn config(&self) -> &LweAmountConfig {
        &self.config
    }

    pub fn params(&self) -> &AmountCipherParams {
        &self.params
    }

    pub fn poseidon_hash_ct_amt(u: &[u32; LWE_DIMENSION_V0], v: u32) -> Fp {
        let mut limbs = Vec::with_capacity(LWE_CIPHERTEXT_LIMBS_V0);
        limbs.extend_from_slice(u);
        limbs.push(v);
        let packed = pack_u32_limbs_to_fp(&limbs);
        let packed: [Fp; PACKED_LWE_CIPHERTEXT_LEN_V0] = packed
            .try_into()
            .expect("ct_amt packing length must match v0 constant");
        poseidon::Hash::<
            _,
            P128Pow5T3,
            ConstantLength<PACKED_LWE_CIPHERTEXT_LEN_V0>,
            POSEIDON_WIDTH,
            POSEIDON_RATE,
        >::init()
        .hash(packed)
    }

    pub fn poseidon_hash_t(t: &[u32; LWE_DIMENSION_V0]) -> Fp {
        let packed = pack_u32_limbs_to_fp(t);
        let packed: [Fp; PACKED_LWE_PUBLIC_KEY_LEN_V0] = packed
            .try_into()
            .expect("t packing length must match v0 constant");
        poseidon::Hash::<
            _,
            P128Pow5T3,
            ConstantLength<PACKED_LWE_PUBLIC_KEY_LEN_V0>,
            POSEIDON_WIDTH,
            POSEIDON_RATE,
        >::init()
        .hash(packed)
    }

    fn assign_packed_words<const L: usize>(
        &self,
        mut layouter: impl Layouter<Fp>,
        name: &'static str,
        packed: [Fp; L],
    ) -> Result<[AssignedCell<Fp, Fp>; L], Error> {
        layouter.assign_region(
            || name,
            |mut region| {
                let mut assigned = Vec::with_capacity(L);
                for (offset, value) in packed.into_iter().enumerate() {
                    let cell = region.assign_advice(
                        || format!("{name}_{offset}"),
                        self.config.packed_word,
                        offset,
                        || Value::known(value),
                    )?;
                    assigned.push(cell);
                }
                assigned.try_into().map_err(|_| Error::Synthesis)
            },
        )
    }

    fn copy_noise_cells(
        &self,
        mut layouter: impl Layouter<Fp>,
        e1_cells: &[AssignedCell<Fp, Fp>; LWE_DIMENSION_V0],
        e2_cell: &AssignedCell<Fp, Fp>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "copy centered noise into lwe amount",
            |mut region| {
                for (offset, cell) in e1_cells.iter().enumerate() {
                    cell.copy_advice(
                        || format!("wired_e1_{offset}"),
                        &mut region,
                        self.config.noise_value,
                        offset,
                    )?;
                }

                e2_cell.copy_advice(
                    || "wired_e2",
                    &mut region,
                    self.config.noise_value,
                    LWE_DIMENSION_V0,
                )?;
                Ok(())
            },
        )
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        u: &[u32; LWE_DIMENSION_V0],
        v: u32,
        t: &[u32; LWE_DIMENSION_V0],
    ) -> Result<LweAmountOutputs, Error> {
        if self.params.lwe_dimension != LWE_DIMENSION_V0 {
            return Err(Error::Synthesis);
        }

        let mut ct_limbs = Vec::with_capacity(LWE_CIPHERTEXT_LIMBS_V0);
        ct_limbs.extend_from_slice(u);
        ct_limbs.push(v);

        let packed_ct: [Fp; PACKED_LWE_CIPHERTEXT_LEN_V0] = pack_u32_limbs_to_fp(&ct_limbs)
            .try_into()
            .map_err(|_| Error::Synthesis)?;
        let packed_t: [Fp; PACKED_LWE_PUBLIC_KEY_LEN_V0] = pack_u32_limbs_to_fp(t)
            .try_into()
            .map_err(|_| Error::Synthesis)?;

        // TODO(privai-v0): this stage only binds pre-packed field elements to
        // Poseidon commitments. When we add full PKE-LWE well-formedness, we
        // must also constrain the packing relation from raw u32 limbs to these
        // packed Fp words inside the circuit.
        let ct_words = self.assign_packed_words(
            layouter.namespace(|| "assign packed ct_amt"),
            "packed_ct_amt",
            packed_ct,
        )?;
        let t_words = self.assign_packed_words(
            layouter.namespace(|| "assign packed t"),
            "packed_t",
            packed_t,
        )?;

        let ct_chip = Pow5Chip::construct(self.config.poseidon.clone());
        let t_chip = Pow5Chip::construct(self.config.poseidon.clone());

        let ct_hasher = Hash::<
            _,
            _,
            P128Pow5T3,
            ConstantLength<PACKED_LWE_CIPHERTEXT_LEN_V0>,
            POSEIDON_WIDTH,
            POSEIDON_RATE,
        >::init(ct_chip, layouter.namespace(|| "init ct_amt poseidon"))?;
        let ct_output = ct_hasher.hash(
            layouter.namespace(|| "hash ct_amt_commit"),
            ct_words,
        )?;

        let t_hasher = Hash::<
            _,
            _,
            P128Pow5T3,
            ConstantLength<PACKED_LWE_PUBLIC_KEY_LEN_V0>,
            POSEIDON_WIDTH,
            POSEIDON_RATE,
        >::init(t_chip, layouter.namespace(|| "init t poseidon"))?;
        let t_output = t_hasher.hash(
            layouter.namespace(|| "hash t_commit"),
            t_words,
        )?;

        layouter.constrain_instance(ct_output.cell(), self.config.ct_amt_commit, 0)?;
        layouter.constrain_instance(t_output.cell(), self.config.t_commit, 0)?;
        Ok(LweAmountOutputs {
            ct_amt_commit: ct_output,
            t_commit: t_output,
        })
    }

    pub fn assign_with_noise_cells(
        &self,
        mut layouter: impl Layouter<Fp>,
        u: &[u32; LWE_DIMENSION_V0],
        v: u32,
        t: &[u32; LWE_DIMENSION_V0],
        e1_cells: &[AssignedCell<Fp, Fp>; LWE_DIMENSION_V0],
        e2_cell: &AssignedCell<Fp, Fp>,
    ) -> Result<LweAmountOutputs, Error> {
        // Canonical inter-chip representation for fresh encryption noise is
        // centered signed field elements, exactly as enforced by
        // `NoiseClassChip`. Future well-formedness constraints will lift these
        // same cells into the `mod 2^32` arithmetic of `u` and `v` with
        // explicit carry / wrap constraints instead of re-encoding them as
        // standalone `u32` witnesses.
        self.copy_noise_cells(
            layouter.namespace(|| "wire noise cells into lwe amount"),
            e1_cells,
            e2_cell,
        )?;
        self.assign(layouter, u, v, t)
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

    use crate::halo2::params::AmountCipherParams;

    use super::{LWE_DIMENSION_V0, LweAmountChip, LweAmountConfig};

    #[derive(Clone)]
    struct LweAmountCircuit {
        u: [u32; LWE_DIMENSION_V0],
        v: u32,
        t: [u32; LWE_DIMENSION_V0],
    }

    impl Default for LweAmountCircuit {
        fn default() -> Self {
            Self {
                u: [0; LWE_DIMENSION_V0],
                v: 0,
                t: [0; LWE_DIMENSION_V0],
            }
        }
    }

    impl Circuit<Fp> for LweAmountCircuit {
        type Config = LweAmountConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
            LweAmountChip::configure(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            layouter: impl Layouter<Fp>,
        ) -> Result<(), Error> {
            let _outputs = LweAmountChip::new(config, AmountCipherParams::default()).assign(
                layouter,
                &self.u,
                self.v,
                &self.t,
            )?;
            Ok(())
        }
    }

    #[test]
    fn lwe_amount_chip_accepts_expected_public_commitments() {
        let u = array::from_fn(|i| (i as u32).wrapping_mul(17).wrapping_add(3));
        let t = array::from_fn(|i| (i as u32).wrapping_mul(29).wrapping_add(11));
        let v = 0xA5A5_5A5A;

        let ct_commit = LweAmountChip::poseidon_hash_ct_amt(&u, v);
        let t_commit = LweAmountChip::poseidon_hash_t(&t);

        let circuit = LweAmountCircuit { u, v, t };
        let prover =
            MockProver::run(14, &circuit, vec![vec![ct_commit], vec![t_commit]]).expect("mock prover");
        prover.assert_satisfied();
    }

    #[test]
    fn lwe_amount_chip_rejects_wrong_ct_commitment() {
        let u = array::from_fn(|i| (i as u32).wrapping_mul(17).wrapping_add(3));
        let t = array::from_fn(|i| (i as u32).wrapping_mul(29).wrapping_add(11));
        let v = 0xA5A5_5A5A;

        let wrong_ct_commit = Fp::from(123456);
        let t_commit = LweAmountChip::poseidon_hash_t(&t);

        let circuit = LweAmountCircuit { u, v, t };
        let prover = MockProver::run(
            14,
            &circuit,
            vec![vec![wrong_ct_commit], vec![t_commit]],
        )
        .expect("mock prover");
        assert!(prover.verify().is_err());
    }
}
