use std::array;

use halo2_gadgets::poseidon::{
    Hash, Pow5Chip, Pow5Config,
    primitives::{self as poseidon, ConstantLength, P128Pow5T3},
};
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Instance, Selector, TableColumn},
    poly::Rotation,
};

use crate::halo2::{
    U32_LIMBS_PER_FIELD, pack_u32_limbs_to_fp, packed_u32_field_len,
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
/// This stage proves:
/// - canonical `u32 -> Fp` packing for `(u, v)` and `t`
/// - `u32` range checks via 16-bit decomposition lookups
/// - `ct_amt_commit = Poseidon(pack(u, v))`
/// - `t_commit = Poseidon(pack(t))`
///
/// Full well-formedness (`u = A^T r + e1`, `v = t^T r + e2 + Δ·amount`) will
/// be layered on top of this in the next iteration.
#[derive(Clone, Debug)]
pub struct LweAmountConfig {
    pub poseidon: Pow5Config<Fp, POSEIDON_WIDTH, POSEIDON_RATE>,
    pub q_pack: Selector,
    pub q_limb: Selector,
    pub limb_value: Column<Advice>,
    pub limb_lo: Column<Advice>,
    pub limb_hi: Column<Advice>,
    pub packed_word: Column<Advice>,
    pub noise_value: Column<Advice>,
    pub table_u16: TableColumn,
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
    pub u_cells: Vec<AssignedCell<Fp, Fp>>,
    pub v_cell: AssignedCell<Fp, Fp>,
    pub t_cells: Vec<AssignedCell<Fp, Fp>>,
    pub r_cells: Vec<AssignedCell<Fp, Fp>>,
}

impl LweAmountChip {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> LweAmountConfig {
        let state = array::from_fn(|_| meta.advice_column());
        let partial_sbox = meta.advice_column();
        let rc_a = array::from_fn(|_| meta.fixed_column());
        let rc_b = array::from_fn(|_| meta.fixed_column());
        let q_pack = meta.selector();
        let q_limb = meta.complex_selector();
        let limb_value = meta.advice_column();
        let limb_lo = meta.advice_column();
        let limb_hi = meta.advice_column();
        let packed_word = meta.advice_column();
        let noise_value = meta.advice_column();
        let table_u16 = meta.lookup_table_column();
        let ct_amt_commit = meta.instance_column();
        let t_commit = meta.instance_column();

        meta.enable_constant(rc_b[0]);
        for column in state {
            meta.enable_equality(column);
        }
        meta.enable_equality(limb_value);
        meta.enable_equality(packed_word);
        meta.enable_equality(noise_value);
        meta.enable_equality(ct_amt_commit);
        meta.enable_equality(t_commit);

        meta.create_gate("u32 limb decomposes into 16-bit halves", |meta| {
            let q = meta.query_selector(q_limb);
            let limb = meta.query_advice(limb_value, Rotation::cur());
            let lo = meta.query_advice(limb_lo, Rotation::cur());
            let hi = meta.query_advice(limb_hi, Rotation::cur());
            let two_pow_16 = Expression::Constant(Fp::from(1u64 << 16));

            vec![q * (limb - (lo + hi * two_pow_16))]
        });

        meta.lookup(|meta| {
            let q = meta.query_selector(q_limb);
            let lo = meta.query_advice(limb_lo, Rotation::cur());

            // Invariant: 0 must always exist in the `u16` lookup table because
            // disabled rows query 0 when `q_limb == 0`.
            vec![(q * lo, table_u16)]
        });

        meta.lookup(|meta| {
            let q = meta.query_selector(q_limb);
            let hi = meta.query_advice(limb_hi, Rotation::cur());

            // Invariant: 0 must always exist in the `u16` lookup table because
            // disabled rows query 0 when `q_limb == 0`.
            vec![(q * hi, table_u16)]
        });

        meta.create_gate("pack seven u32 limbs into one field element", |meta| {
            let q = meta.query_selector(q_pack);
            let packed = meta.query_advice(packed_word, Rotation::cur());
            let radix = Expression::Constant(Fp::from(1u64 << 32));
            let mut coeff = Expression::Constant(Fp::from(1u64));
            let mut acc = Expression::Constant(Fp::from(0u64));

            for rotation in 0..U32_LIMBS_PER_FIELD {
                let limb = meta.query_advice(limb_value, Rotation(rotation as i32));
                acc = acc + limb * coeff.clone();
                coeff = coeff * radix.clone();
            }

            vec![q * (packed - acc)]
        });

        let poseidon = Pow5Chip::configure::<P128Pow5T3>(
            meta,
            state,
            partial_sbox,
            rc_a,
            rc_b,
        );

        LweAmountConfig {
            poseidon,
            q_pack,
            q_limb,
            limb_value,
            limb_lo,
            limb_hi,
            packed_word,
            noise_value,
            table_u16,
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

    pub fn load_u16_table(&self, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        layouter.assign_table(
            || "u16 range table",
            |mut table| {
                for value in 0..=u16::MAX {
                    table.assign_cell(
                        || format!("u16_{value}"),
                        self.config.table_u16,
                        value as usize,
                        || Value::known(Fp::from(value as u64)),
                    )?;
                }
                Ok(())
            },
        )
    }

    fn assign_canonical_limb_cells(
        &self,
        mut layouter: impl Layouter<Fp>,
        name: &'static str,
        limbs: &[u32],
    ) -> Result<Vec<AssignedCell<Fp, Fp>>, Error> {
        layouter.assign_region(
            || name,
            |mut region| {
                let mut assigned = Vec::with_capacity(limbs.len());
                for (offset, limb) in limbs.iter().copied().enumerate() {
                    let lo = limb & 0xFFFF;
                    let hi = limb >> 16;

                    self.config.q_limb.enable(&mut region, offset)?;
                    let cell = region.assign_advice(
                        || format!("{name}_limb_{offset}"),
                        self.config.limb_value,
                        offset,
                        || Value::known(Fp::from(limb as u64)),
                    )?;
                    region.assign_advice(
                        || format!("{name}_limb_lo_{offset}"),
                        self.config.limb_lo,
                        offset,
                        || Value::known(Fp::from(lo as u64)),
                    )?;
                    region.assign_advice(
                        || format!("{name}_limb_hi_{offset}"),
                        self.config.limb_hi,
                        offset,
                        || Value::known(Fp::from(hi as u64)),
                    )?;
                    assigned.push(cell);
                }
                Ok(assigned)
            },
        )
    }

    fn assign_packed_from_cells<const PACKED: usize>(
        &self,
        mut layouter: impl Layouter<Fp>,
        name: &'static str,
        limb_cells: &[AssignedCell<Fp, Fp>],
        packed: [Fp; PACKED],
    ) -> Result<[AssignedCell<Fp, Fp>; PACKED], Error> {
        layouter.assign_region(
            || name,
            |mut region| {
                let mut assigned = Vec::with_capacity(PACKED);
                for (chunk_idx, value) in packed.into_iter().enumerate() {
                    let base = chunk_idx * U32_LIMBS_PER_FIELD;
                    self.config.q_pack.enable(&mut region, base)?;

                    let cell = region.assign_advice(
                        || format!("{name}_packed_{chunk_idx}"),
                        self.config.packed_word,
                        base,
                        || Value::known(value),
                    )?;
                    assigned.push(cell);

                    for limb_offset in 0..U32_LIMBS_PER_FIELD {
                        let offset = base + limb_offset;
                        if let Some(cell) = limb_cells.get(offset) {
                            cell.copy_advice(
                                || format!("{name}_limb_{offset}"),
                                &mut region,
                                self.config.limb_value,
                                offset,
                            )?;
                        } else {
                            region.assign_advice(
                                || format!("{name}_limb_pad_{offset}"),
                                self.config.limb_value,
                                offset,
                                || Value::known(Fp::from(0u64)),
                            )?;
                        }
                    }
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
        r: &[u32; LWE_DIMENSION_V0],
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

        let u_cells = self.assign_canonical_limb_cells(
            layouter.namespace(|| "assign canonical u"),
            "canonical_u",
            u,
        )?;
        let v_cells = self.assign_canonical_limb_cells(
            layouter.namespace(|| "assign canonical v"),
            "canonical_v",
            &[v],
        )?;
        let v_cell = v_cells.into_iter().next().ok_or(Error::Synthesis)?;
        let t_cells = self.assign_canonical_limb_cells(
            layouter.namespace(|| "assign canonical t"),
            "canonical_t",
            t,
        )?;
        let r_cells = self.assign_canonical_limb_cells(
            layouter.namespace(|| "assign canonical r"),
            "canonical_r",
            r,
        )?;

        let mut ct_limb_cells = u_cells.clone();
        ct_limb_cells.push(v_cell.clone());
        let ct_words = self.assign_packed_from_cells(
            layouter.namespace(|| "assign packed ct_amt"),
            "packed_ct_amt",
            &ct_limb_cells,
            packed_ct,
        )?;
        let t_words = self.assign_packed_from_cells(
            layouter.namespace(|| "assign packed t"),
            "packed_t",
            &t_cells,
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
            u_cells,
            v_cell,
            t_cells,
            r_cells,
        })
    }

    pub fn assign_with_noise_cells(
        &self,
        mut layouter: impl Layouter<Fp>,
        u: &[u32; LWE_DIMENSION_V0],
        v: u32,
        t: &[u32; LWE_DIMENSION_V0],
        r: &[u32; LWE_DIMENSION_V0],
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
        self.assign(layouter, u, v, t, r)
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
        r: [u32; LWE_DIMENSION_V0],
    }

    impl Default for LweAmountCircuit {
        fn default() -> Self {
            Self {
                u: [0; LWE_DIMENSION_V0],
                v: 0,
                t: [0; LWE_DIMENSION_V0],
                r: [0; LWE_DIMENSION_V0],
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
            mut layouter: impl Layouter<Fp>,
        ) -> Result<(), Error> {
            let chip = LweAmountChip::new(config, AmountCipherParams::default());
            chip.load_u16_table(layouter.namespace(|| "load u16 table"))?;
            let _outputs = chip.assign(
                layouter,
                &self.u,
                self.v,
                &self.t,
                &self.r,
            )?;
            Ok(())
        }
    }

    #[test]
    fn lwe_amount_chip_accepts_expected_public_commitments() {
        let u = array::from_fn(|i| (i as u32).wrapping_mul(17).wrapping_add(3));
        let t = array::from_fn(|i| (i as u32).wrapping_mul(29).wrapping_add(11));
        let r = array::from_fn(|i| (i as u32).wrapping_mul(31).wrapping_add(19));
        let v = 0xA5A5_5A5A;

        let ct_commit = LweAmountChip::poseidon_hash_ct_amt(&u, v);
        let t_commit = LweAmountChip::poseidon_hash_t(&t);

        let circuit = LweAmountCircuit { u, v, t, r };
        let prover =
            MockProver::run(17, &circuit, vec![vec![ct_commit], vec![t_commit]]).expect("mock prover");
        prover.assert_satisfied();
    }

    #[test]
    fn lwe_amount_chip_rejects_wrong_ct_commitment() {
        let u = array::from_fn(|i| (i as u32).wrapping_mul(17).wrapping_add(3));
        let t = array::from_fn(|i| (i as u32).wrapping_mul(29).wrapping_add(11));
        let r = array::from_fn(|i| (i as u32).wrapping_mul(31).wrapping_add(19));
        let v = 0xA5A5_5A5A;

        let wrong_ct_commit = Fp::from(123456);
        let t_commit = LweAmountChip::poseidon_hash_t(&t);

        let circuit = LweAmountCircuit { u, v, t, r };
        let prover = MockProver::run(
            17,
            &circuit,
            vec![vec![wrong_ct_commit], vec![t_commit]],
        )
        .expect("mock prover");
        assert!(prover.verify().is_err());
    }

    #[test]
    fn lwe_amount_chip_rejects_wrong_t_commitment() {
        let u = array::from_fn(|i| (i as u32).wrapping_mul(17).wrapping_add(3));
        let t = array::from_fn(|i| (i as u32).wrapping_mul(29).wrapping_add(11));
        let r = array::from_fn(|i| (i as u32).wrapping_mul(31).wrapping_add(19));
        let v = 0xA5A5_5A5A;

        let ct_commit = LweAmountChip::poseidon_hash_ct_amt(&u, v);
        let wrong_t_commit = Fp::from(654321);

        let circuit = LweAmountCircuit { u, v, t, r };
        let prover = MockProver::run(
            17,
            &circuit,
            vec![vec![ct_commit], vec![wrong_t_commit]],
        )
        .expect("mock prover");
        assert!(prover.verify().is_err());
    }
}
