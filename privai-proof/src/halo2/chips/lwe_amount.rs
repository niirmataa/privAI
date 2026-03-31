use std::array;

use halo2_gadgets::poseidon::{
    Hash, Pow5Chip, Pow5Config,
    primitives::{self as poseidon, ConstantLength, P128Pow5T3},
};
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    plonk::{
        Advice, Column, ConstraintSystem, Error, Expression, Fixed, Instance, Selector,
        TableColumn,
    },
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
// For n = 512 and 32-bit limbs:
//   max dot product < 512 * (2^32 - 1)^2 < 2^73
// After reduction mod 2^32, the quotient therefore fits below 2^41.
// We keep one extra bit of slack and constrain the witness quotient to 42 bits.
const DOT_REDUCTION_QUOTIENT_BITS: usize = 42;

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
    pub q_dot_first: Selector,
    pub q_dot_mul: Selector,
    pub q_dot_acc: Selector,
    pub q_reduce_eq: Selector,
    pub q_reduce_bind: Selector,
    pub q_reduce_bit: Selector,
    pub q_reduce_acc_first: Selector,
    pub q_reduce_acc_step: Selector,
    pub q_noise_relation: Selector,
    pub limb_value: Column<Advice>,
    pub limb_lo: Column<Advice>,
    pub limb_hi: Column<Advice>,
    pub dot_coeff: Column<Fixed>,
    pub dot_value: Column<Advice>,
    pub dot_product: Column<Advice>,
    pub dot_accumulator: Column<Advice>,
    pub reduce_coeff: Column<Fixed>,
    pub reduce_full: Column<Advice>,
    pub reduce_reduced: Column<Advice>,
    pub reduce_quotient: Column<Advice>,
    pub reduce_bit: Column<Advice>,
    pub reduce_accumulator: Column<Advice>,
    pub noise_reduced: Column<Advice>,
    pub noise_centered: Column<Advice>,
    pub noise_output: Column<Advice>,
    pub noise_wrap_positive: Column<Advice>,
    pub noise_wrap_negative: Column<Advice>,
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
        let q_dot_first = meta.selector();
        let q_dot_mul = meta.selector();
        let q_dot_acc = meta.selector();
        let q_reduce_eq = meta.selector();
        let q_reduce_bind = meta.selector();
        let q_reduce_bit = meta.selector();
        let q_reduce_acc_first = meta.selector();
        let q_reduce_acc_step = meta.selector();
        let q_noise_relation = meta.selector();
        let limb_value = meta.advice_column();
        let limb_lo = meta.advice_column();
        let limb_hi = meta.advice_column();
        let dot_coeff = meta.fixed_column();
        let dot_value = meta.advice_column();
        let dot_product = meta.advice_column();
        let dot_accumulator = meta.advice_column();
        let reduce_coeff = meta.fixed_column();
        let reduce_full = meta.advice_column();
        let reduce_reduced = meta.advice_column();
        let reduce_quotient = meta.advice_column();
        let reduce_bit = meta.advice_column();
        let reduce_accumulator = meta.advice_column();
        let noise_reduced = meta.advice_column();
        let noise_centered = meta.advice_column();
        let noise_output = meta.advice_column();
        let noise_wrap_positive = meta.advice_column();
        let noise_wrap_negative = meta.advice_column();
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
        meta.enable_equality(dot_value);
        meta.enable_equality(dot_accumulator);
        meta.enable_equality(reduce_full);
        meta.enable_equality(reduce_reduced);
        meta.enable_equality(reduce_quotient);
        meta.enable_equality(noise_reduced);
        meta.enable_equality(noise_centered);
        meta.enable_equality(noise_output);
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

        meta.create_gate("fixed-coefficient dot product multiply step", |meta| {
            let q = meta.query_selector(q_dot_mul);
            let coeff = meta.query_fixed(dot_coeff);
            let value = meta.query_advice(dot_value, Rotation::cur());
            let product = meta.query_advice(dot_product, Rotation::cur());

            vec![q * (product - coeff * value)]
        });

        meta.create_gate("fixed-coefficient dot product accumulator init", |meta| {
            let q = meta.query_selector(q_dot_first);
            let acc = meta.query_advice(dot_accumulator, Rotation::cur());
            let product = meta.query_advice(dot_product, Rotation::cur());

            vec![q * (acc - product)]
        });

        meta.create_gate("fixed-coefficient dot product accumulator step", |meta| {
            let q = meta.query_selector(q_dot_acc);
            let acc = meta.query_advice(dot_accumulator, Rotation::cur());
            let prev_acc = meta.query_advice(dot_accumulator, Rotation::prev());
            let product = meta.query_advice(dot_product, Rotation::cur());

            vec![q * (acc - prev_acc - product)]
        });

        meta.create_gate("dot-product mod 2^32 equality", |meta| {
            let q = meta.query_selector(q_reduce_eq);
            let full = meta.query_advice(reduce_full, Rotation::cur());
            let reduced = meta.query_advice(reduce_reduced, Rotation::cur());
            let quotient = meta.query_advice(reduce_quotient, Rotation::cur());
            let two_pow_32 = Expression::Constant(Fp::from(1u64 << 32));

            vec![q * (full - reduced - quotient * two_pow_32)]
        });

        meta.create_gate("dot-product quotient bit is boolean", |meta| {
            let q = meta.query_selector(q_reduce_bit);
            let bit = meta.query_advice(reduce_bit, Rotation::cur());

            vec![q * bit.clone() * (bit - Expression::Constant(Fp::from(1u64)))]
        });

        meta.create_gate("dot-product quotient accumulator init", |meta| {
            let q = meta.query_selector(q_reduce_acc_first);
            let coeff = meta.query_fixed(reduce_coeff);
            let bit = meta.query_advice(reduce_bit, Rotation::cur());
            let acc = meta.query_advice(reduce_accumulator, Rotation::cur());

            vec![q * (acc - bit * coeff)]
        });

        meta.create_gate("dot-product quotient accumulator step", |meta| {
            let q = meta.query_selector(q_reduce_acc_step);
            let coeff = meta.query_fixed(reduce_coeff);
            let bit = meta.query_advice(reduce_bit, Rotation::cur());
            let acc = meta.query_advice(reduce_accumulator, Rotation::cur());
            let prev_acc = meta.query_advice(reduce_accumulator, Rotation::prev());

            vec![q * (acc - prev_acc - bit * coeff)]
        });

        meta.create_gate("dot-product quotient matches accumulated bits", |meta| {
            let q = meta.query_selector(q_reduce_bind);
            let quotient = meta.query_advice(reduce_quotient, Rotation::cur());
            let acc = meta.query_advice(reduce_accumulator, Rotation::cur());

            vec![q * (quotient - acc)]
        });

        meta.create_gate("noise-adjusted u32 relation", |meta| {
            let q = meta.query_selector(q_noise_relation);
            let reduced = meta.query_advice(noise_reduced, Rotation::cur());
            let noise = meta.query_advice(noise_centered, Rotation::cur());
            let output = meta.query_advice(noise_output, Rotation::cur());
            let wrap_positive = meta.query_advice(noise_wrap_positive, Rotation::cur());
            let wrap_negative = meta.query_advice(noise_wrap_negative, Rotation::cur());
            let one = Expression::Constant(Fp::from(1u64));
            let two_pow_32 = Expression::Constant(Fp::from(1u64 << 32));

            vec![
                q.clone()
                    * (output - reduced - noise - two_pow_32 * (wrap_positive.clone() - wrap_negative.clone())),
                q.clone() * wrap_positive.clone() * (wrap_positive - one.clone()),
                q.clone() * wrap_negative.clone() * (wrap_negative - one.clone()),
                q * meta.query_advice(noise_wrap_positive, Rotation::cur())
                    * meta.query_advice(noise_wrap_negative, Rotation::cur()),
            ]
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
            q_dot_first,
            q_dot_mul,
            q_dot_acc,
            q_reduce_eq,
            q_reduce_bind,
            q_reduce_bit,
            q_reduce_acc_first,
            q_reduce_acc_step,
            q_noise_relation,
            limb_value,
            limb_lo,
            limb_hi,
            dot_coeff,
            dot_value,
            dot_product,
            dot_accumulator,
            reduce_coeff,
            reduce_full,
            reduce_reduced,
            reduce_quotient,
            reduce_bit,
            reduce_accumulator,
            noise_reduced,
            noise_centered,
            noise_output,
            noise_wrap_positive,
            noise_wrap_negative,
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

    fn assign_canonical_single_limb_cell(
        &self,
        layouter: impl Layouter<Fp>,
        name: &'static str,
        limb: u32,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let cells = self.assign_canonical_limb_cells(layouter, name, &[limb])?;
        cells.into_iter().next().ok_or(Error::Synthesis)
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
                // SECURITY_TODO(privai-v0): `e2` is wired into the chip, but
                // the relation `v = t^T r + e2 + Δ·amount` is not constrained
                // yet. This will be closed together with the scalar `v`
                // well-formedness gate.
                Ok(())
            },
        )
    }

    pub fn assign_fixed_dot_product_scaffold(
        &self,
        mut layouter: impl Layouter<Fp>,
        name: &'static str,
        coeffs: &[u32; LWE_DIMENSION_V0],
        value_cells: &[AssignedCell<Fp, Fp>],
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        if value_cells.len() != LWE_DIMENSION_V0 {
            return Err(Error::Synthesis);
        }

        layouter.assign_region(
            || name,
            |mut region| {
                let mut running_acc = Fp::from(0u64);
                let mut last_acc = None;

                for (offset, (&coeff, value_cell)) in coeffs.iter().zip(value_cells.iter()).enumerate() {
                    let coeff_fp = Fp::from(coeff as u64);
                    let value_fp = value_cell.value().map(|value| *value);
                    let product_fp = value_fp.map(|value| coeff_fp * value);

                    self.config.q_dot_mul.enable(&mut region, offset)?;
                    if offset == 0 {
                        self.config.q_dot_first.enable(&mut region, offset)?;
                    } else {
                        self.config.q_dot_acc.enable(&mut region, offset)?;
                    }

                    region.assign_fixed(
                        || format!("{name}_coeff_{offset}"),
                        self.config.dot_coeff,
                        offset,
                        || Value::known(coeff_fp),
                    )?;

                    value_cell.copy_advice(
                        || format!("{name}_value_{offset}"),
                        &mut region,
                        self.config.dot_value,
                        offset,
                    )?;

                    region.assign_advice(
                        || format!("{name}_product_{offset}"),
                        self.config.dot_product,
                        offset,
                        || product_fp,
                    )?;

                    let acc_value = value_fp.map(|value| {
                        running_acc += coeff_fp * value;
                        running_acc
                    });
                    let acc_cell = region.assign_advice(
                        || format!("{name}_acc_{offset}"),
                        self.config.dot_accumulator,
                        offset,
                        || acc_value,
                    )?;
                    last_acc = Some(acc_cell);
                }

                last_acc.ok_or(Error::Synthesis)
            },
        )
    }

    pub fn assign_dot_product_mod_u32_reduction(
        &self,
        mut layouter: impl Layouter<Fp>,
        name: &'static str,
        full_value: AssignedCell<Fp, Fp>,
        reduced_value: u32,
        quotient: u64,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        if quotient >= (1u64 << DOT_REDUCTION_QUOTIENT_BITS) {
            return Err(Error::Synthesis);
        }

        let reduced_cell = self.assign_canonical_single_limb_cell(
            layouter.namespace(|| format!("{name} reduced limb")),
            "dot_reduced",
            reduced_value,
        )?;
        let reduced_copy = reduced_cell.clone();

        layouter.assign_region(
            || name,
            |mut region| {
                let quotient_cell = region.assign_advice(
                    || format!("{name}_quotient_0"),
                    self.config.reduce_quotient,
                    0,
                    || Value::known(Fp::from(quotient)),
                )?;
                full_value.copy_advice(
                    || format!("{name}_full"),
                    &mut region,
                    self.config.reduce_full,
                    0,
                )?;
                reduced_copy.copy_advice(
                    || format!("{name}_reduced"),
                    &mut region,
                    self.config.reduce_reduced,
                    0,
                )?;

                let mut running_acc = 0u64;
                for offset in 0..DOT_REDUCTION_QUOTIENT_BITS {
                    let coeff = 1u64 << offset;
                    let bit = (quotient >> offset) & 1;
                    running_acc += bit * coeff;

                    region.assign_fixed(
                        || format!("{name}_coeff_{offset}"),
                        self.config.reduce_coeff,
                        offset,
                        || Value::known(Fp::from(coeff)),
                    )?;
                    region.assign_advice(
                        || format!("{name}_bit_{offset}"),
                        self.config.reduce_bit,
                        offset,
                        || Value::known(Fp::from(bit)),
                    )?;
                    region.assign_advice(
                        || format!("{name}_acc_{offset}"),
                        self.config.reduce_accumulator,
                        offset,
                        || Value::known(Fp::from(running_acc)),
                    )?;

                    self.config.q_reduce_bit.enable(&mut region, offset)?;
                    if offset == 0 {
                        self.config.q_reduce_eq.enable(&mut region, offset)?;
                        self.config.q_reduce_acc_first.enable(&mut region, offset)?;
                    } else {
                        self.config.q_reduce_acc_step.enable(&mut region, offset)?;
                    }

                    if offset == DOT_REDUCTION_QUOTIENT_BITS - 1 {
                        quotient_cell.copy_advice(
                            || format!("{name}_quotient_final"),
                            &mut region,
                            self.config.reduce_quotient,
                            offset,
                        )?;
                        self.config.q_reduce_bind.enable(&mut region, offset)?;
                    }
                }

                Ok(())
            },
        )?;

        Ok(reduced_cell)
    }

    pub fn assign_noise_adjusted_u32_relation(
        &self,
        mut layouter: impl Layouter<Fp>,
        name: &'static str,
        reduced_value: AssignedCell<Fp, Fp>,
        noise_value: &AssignedCell<Fp, Fp>,
        output_value: &AssignedCell<Fp, Fp>,
        wrap_positive: bool,
        wrap_negative: bool,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || name,
            |mut region| {
                self.config.q_noise_relation.enable(&mut region, 0)?;
                reduced_value.copy_advice(
                    || format!("{name}_reduced"),
                    &mut region,
                    self.config.noise_reduced,
                    0,
                )?;
                noise_value.copy_advice(
                    || format!("{name}_noise"),
                    &mut region,
                    self.config.noise_centered,
                    0,
                )?;
                output_value.copy_advice(
                    || format!("{name}_output"),
                    &mut region,
                    self.config.noise_output,
                    0,
                )?;
                region.assign_advice(
                    || format!("{name}_wrap_positive"),
                    self.config.noise_wrap_positive,
                    0,
                    || Value::known(Fp::from(wrap_positive as u64)),
                )?;
                region.assign_advice(
                    || format!("{name}_wrap_negative"),
                    self.config.noise_wrap_negative,
                    0,
                    || Value::known(Fp::from(wrap_negative as u64)),
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

    use crate::halo2::{NoiseClassChip, NoiseClassConfig, params::AmountCipherParams};

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

    #[derive(Clone)]
    struct LweAmountDotProductCircuit {
        u: [u32; LWE_DIMENSION_V0],
        v: u32,
        t: [u32; LWE_DIMENSION_V0],
        r: [u32; LWE_DIMENSION_V0],
        coeffs: [u32; LWE_DIMENSION_V0],
    }

    impl Default for LweAmountDotProductCircuit {
        fn default() -> Self {
            Self {
                u: [0; LWE_DIMENSION_V0],
                v: 0,
                t: [0; LWE_DIMENSION_V0],
                r: [0; LWE_DIMENSION_V0],
                coeffs: [0; LWE_DIMENSION_V0],
            }
        }
    }

    #[derive(Clone)]
    struct LweAmountSingleColumnReductionCircuit {
        u: [u32; LWE_DIMENSION_V0],
        v: u32,
        t: [u32; LWE_DIMENSION_V0],
        r: [u32; LWE_DIMENSION_V0],
        coeffs: [u32; LWE_DIMENSION_V0],
        reduced_u0: u32,
        quotient: u64,
    }

    impl Default for LweAmountSingleColumnReductionCircuit {
        fn default() -> Self {
            Self {
                u: [0; LWE_DIMENSION_V0],
                v: 0,
                t: [0; LWE_DIMENSION_V0],
                r: [0; LWE_DIMENSION_V0],
                coeffs: [0; LWE_DIMENSION_V0],
                reduced_u0: 0,
                quotient: 0,
            }
        }
    }

    #[derive(Clone)]
    struct LweAmountSingleColumnNoiseRelationConfig {
        lwe_amount: LweAmountConfig,
        noise_class: NoiseClassConfig,
    }

    #[derive(Clone)]
    struct LweAmountSingleColumnNoiseRelationCircuit {
        u: [u32; LWE_DIMENSION_V0],
        v: u32,
        t: [u32; LWE_DIMENSION_V0],
        r: [u32; LWE_DIMENSION_V0],
        coeffs: [u32; LWE_DIMENSION_V0],
        reduced_u0: u32,
        quotient: u64,
        e1: [i16; LWE_DIMENSION_V0],
        e2: i16,
        noise_class: u8,
        wrap_positive: bool,
        wrap_negative: bool,
    }

    impl Default for LweAmountSingleColumnNoiseRelationCircuit {
        fn default() -> Self {
            Self {
                u: [0; LWE_DIMENSION_V0],
                v: 0,
                t: [0; LWE_DIMENSION_V0],
                r: [0; LWE_DIMENSION_V0],
                coeffs: [0; LWE_DIMENSION_V0],
                reduced_u0: 0,
                quotient: 0,
                e1: [0; LWE_DIMENSION_V0],
                e2: 0,
                noise_class: 0,
                wrap_positive: false,
                wrap_negative: false,
            }
        }
    }

    impl Circuit<Fp> for LweAmountSingleColumnNoiseRelationCircuit {
        type Config = LweAmountSingleColumnNoiseRelationConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
            Self::Config {
                lwe_amount: LweAmountChip::configure(meta),
                noise_class: NoiseClassChip::configure(meta),
            }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<Fp>,
        ) -> Result<(), Error> {
            let chip = LweAmountChip::new(config.lwe_amount.clone(), AmountCipherParams::default());
            let noise_chip = NoiseClassChip::new(config.noise_class);
            noise_chip.load_lookup_table(layouter.namespace(|| "load noise table"))?;
            chip.load_u16_table(layouter.namespace(|| "load u16 table"))?;

            let noise_outputs = noise_chip.assign(
                layouter.namespace(|| "assign noise"),
                &self.e1,
                self.e2,
                self.noise_class,
            )?;
            let outputs = chip.assign_with_noise_cells(
                layouter.namespace(|| "assign lwe amount"),
                &self.u,
                self.v,
                &self.t,
                &self.r,
                &noise_outputs.e1_cells,
                &noise_outputs.e2_cell,
            )?;
            let dot_output = chip.assign_fixed_dot_product_scaffold(
                layouter.namespace(|| "column_0 dot scaffold"),
                "column_0_dot",
                &self.coeffs,
                &outputs.r_cells,
            )?;
            let reduced_cell = chip.assign_dot_product_mod_u32_reduction(
                layouter.namespace(|| "column_0 reduction"),
                "column_0_reduction",
                dot_output.clone(),
                self.reduced_u0,
                self.quotient,
            )?;
            chip.assign_noise_adjusted_u32_relation(
                layouter.namespace(|| "column_0 noise relation"),
                "column_0_noise_relation",
                reduced_cell,
                &noise_outputs.e1_cells[0],
                &outputs.u_cells[0],
                self.wrap_positive,
                self.wrap_negative,
            )?;

            layouter.constrain_instance(outputs.u_cells[0].cell(), config.lwe_amount.ct_amt_commit, 1)?;
            layouter.constrain_instance(outputs.ct_amt_commit.cell(), config.lwe_amount.ct_amt_commit, 0)?;
            layouter.constrain_instance(outputs.t_commit.cell(), config.lwe_amount.t_commit, 0)?;
            Ok(())
        }
    }

    impl Circuit<Fp> for LweAmountSingleColumnReductionCircuit {
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
            let chip = LweAmountChip::new(config.clone(), AmountCipherParams::default());
            chip.load_u16_table(layouter.namespace(|| "load u16 table"))?;
            let outputs = chip.assign(
                layouter.namespace(|| "assign lwe amount"),
                &self.u,
                self.v,
                &self.t,
                &self.r,
            )?;
            let dot_output = chip.assign_fixed_dot_product_scaffold(
                layouter.namespace(|| "column_0 dot scaffold"),
                "column_0_dot",
                &self.coeffs,
                &outputs.r_cells,
            )?;
            let reduced_cell = chip.assign_dot_product_mod_u32_reduction(
                layouter.namespace(|| "column_0 reduction"),
                "column_0_reduction",
                dot_output,
                self.reduced_u0,
                self.quotient,
            )?;
            layouter.constrain_instance(reduced_cell.cell(), config.ct_amt_commit, 1)?;
            Ok(())
        }
    }

    impl Circuit<Fp> for LweAmountDotProductCircuit {
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
            let chip = LweAmountChip::new(config.clone(), AmountCipherParams::default());
            chip.load_u16_table(layouter.namespace(|| "load u16 table"))?;
            let outputs = chip.assign(
                layouter.namespace(|| "assign lwe amount"),
                &self.u,
                self.v,
                &self.t,
                &self.r,
            )?;
            let dot_output = chip.assign_fixed_dot_product_scaffold(
                layouter.namespace(|| "dot scaffold"),
                "dot_scaffold",
                &self.coeffs,
                &outputs.r_cells,
            )?;
            layouter.constrain_instance(dot_output.cell(), config.ct_amt_commit, 1)?;
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

    #[test]
    fn lwe_amount_fixed_dot_product_scaffold_accepts_expected_output() {
        let u = array::from_fn(|i| (i as u32).wrapping_mul(17).wrapping_add(3));
        let t = array::from_fn(|i| (i as u32).wrapping_mul(29).wrapping_add(11));
        let r = array::from_fn(|i| (i as u32).wrapping_mul(31).wrapping_add(19));
        let coeffs = array::from_fn(|i| (i as u32).wrapping_mul(7).wrapping_add(5));
        let v = 0xA5A5_5A5A;

        let ct_commit = LweAmountChip::poseidon_hash_ct_amt(&u, v);
        let t_commit = LweAmountChip::poseidon_hash_t(&t);
        let expected_dot = coeffs
            .iter()
            .zip(r.iter())
            .fold(Fp::from(0u64), |acc, (&coeff, &value)| {
                acc + Fp::from(coeff as u64) * Fp::from(value as u64)
            });

        let circuit = LweAmountDotProductCircuit {
            u,
            v,
            t,
            r,
            coeffs,
        };
        let prover = MockProver::run(
            18,
            &circuit,
            vec![vec![ct_commit, expected_dot], vec![t_commit]],
        )
        .expect("mock prover");
        prover.assert_satisfied();
    }

    #[test]
    fn lwe_amount_fixed_dot_product_scaffold_rejects_wrong_output() {
        let u = array::from_fn(|i| (i as u32).wrapping_mul(17).wrapping_add(3));
        let t = array::from_fn(|i| (i as u32).wrapping_mul(29).wrapping_add(11));
        let r = array::from_fn(|i| (i as u32).wrapping_mul(31).wrapping_add(19));
        let coeffs = array::from_fn(|i| (i as u32).wrapping_mul(7).wrapping_add(5));
        let v = 0xA5A5_5A5A;

        let ct_commit = LweAmountChip::poseidon_hash_ct_amt(&u, v);
        let t_commit = LweAmountChip::poseidon_hash_t(&t);

        let circuit = LweAmountDotProductCircuit {
            u,
            v,
            t,
            r,
            coeffs,
        };
        let prover = MockProver::run(
            18,
            &circuit,
            vec![vec![ct_commit, Fp::from(123456u64)], vec![t_commit]],
        )
        .expect("mock prover");
        assert!(prover.verify().is_err());
    }

    #[test]
    fn lwe_amount_single_column_reduction_accepts_expected_u0() {
        let coeffs = array::from_fn(|i| (i as u32).wrapping_mul(37).wrapping_add(1000));
        let r = array::from_fn(|i| (i as u32).wrapping_mul(7).wrapping_add(3));
        let dot = coeffs
            .iter()
            .zip(r.iter())
            .fold(0u128, |acc, (&coeff, &value)| {
                acc + (coeff as u128) * (value as u128)
            });
        let expected_u0 = coeffs
            .iter()
            .zip(r.iter())
            .fold(0u32, |acc, (&coeff, &value)| acc.wrapping_add(coeff.wrapping_mul(value)));
        let quotient = (dot >> 32) as u64;

        let mut u = [0u32; LWE_DIMENSION_V0];
        u[0] = expected_u0;
        let t = array::from_fn(|i| (i as u32).wrapping_mul(29).wrapping_add(11));
        let v = 0xA5A5_5A5A;

        let ct_commit = LweAmountChip::poseidon_hash_ct_amt(&u, v);
        let t_commit = LweAmountChip::poseidon_hash_t(&t);

        let circuit = LweAmountSingleColumnReductionCircuit {
            u,
            v,
            t,
            r,
            coeffs,
            reduced_u0: expected_u0,
            quotient,
        };
        let prover = MockProver::run(
            18,
            &circuit,
            vec![vec![ct_commit, Fp::from(expected_u0 as u64)], vec![t_commit]],
        )
        .expect("mock prover");
        prover.assert_satisfied();
    }

    #[test]
    fn lwe_amount_single_column_reduction_rejects_wrong_quotient() {
        let coeffs = array::from_fn(|i| (i as u32).wrapping_mul(37).wrapping_add(1000));
        let r = array::from_fn(|i| (i as u32).wrapping_mul(7).wrapping_add(3));
        let dot = coeffs
            .iter()
            .zip(r.iter())
            .fold(0u128, |acc, (&coeff, &value)| {
                acc + (coeff as u128) * (value as u128)
            });
        let expected_u0 = coeffs
            .iter()
            .zip(r.iter())
            .fold(0u32, |acc, (&coeff, &value)| acc.wrapping_add(coeff.wrapping_mul(value)));
        let quotient = (dot >> 32) as u64;

        let mut u = [0u32; LWE_DIMENSION_V0];
        u[0] = expected_u0;
        let t = array::from_fn(|i| (i as u32).wrapping_mul(29).wrapping_add(11));
        let v = 0xA5A5_5A5A;

        let ct_commit = LweAmountChip::poseidon_hash_ct_amt(&u, v);
        let t_commit = LweAmountChip::poseidon_hash_t(&t);

        let circuit = LweAmountSingleColumnReductionCircuit {
            u,
            v,
            t,
            r,
            coeffs,
            reduced_u0: expected_u0,
            quotient: quotient + 1,
        };
        let prover = MockProver::run(
            18,
            &circuit,
            vec![vec![ct_commit, Fp::from(expected_u0 as u64)], vec![t_commit]],
        )
        .expect("mock prover");
        assert!(prover.verify().is_err());
    }

    #[test]
    fn lwe_amount_single_column_noise_relation_accepts_negative_wrap() {
        let coeffs = {
            let mut coeffs = [0u32; LWE_DIMENSION_V0];
            coeffs[0] = 1;
            coeffs
        };
        let r = {
            let mut r = [0u32; LWE_DIMENSION_V0];
            r[0] = 5;
            r
        };
        let mut e1 = [0i16; LWE_DIMENSION_V0];
        e1[0] = -12;
        let reduced_dot = 5u32;
        let expected_u0 = reduced_dot.wrapping_add(e1[0] as u32);
        let quotient = 0u64;

        let mut u = [0u32; LWE_DIMENSION_V0];
        u[0] = expected_u0;
        let t = array::from_fn(|i| (i as u32).wrapping_mul(29).wrapping_add(11));
        let v = 0xA5A5_5A5A;

        let ct_commit = LweAmountChip::poseidon_hash_ct_amt(&u, v);
        let t_commit = LweAmountChip::poseidon_hash_t(&t);

        let circuit = LweAmountSingleColumnNoiseRelationCircuit {
            u,
            v,
            t,
            r,
            coeffs,
            reduced_u0: reduced_dot,
            quotient,
            e1,
            e2: 0,
            noise_class: 0,
            wrap_positive: true,
            wrap_negative: false,
        };
        let prover = MockProver::run(
            18,
            &circuit,
            vec![vec![ct_commit, Fp::from(expected_u0 as u64)], vec![t_commit]],
        )
        .expect("mock prover");
        prover.assert_satisfied();
    }

    #[test]
    fn lwe_amount_single_column_noise_relation_rejects_wrong_wrap() {
        let coeffs = {
            let mut coeffs = [0u32; LWE_DIMENSION_V0];
            coeffs[0] = 1;
            coeffs
        };
        let r = {
            let mut r = [0u32; LWE_DIMENSION_V0];
            r[0] = 5;
            r
        };
        let mut e1 = [0i16; LWE_DIMENSION_V0];
        e1[0] = -12;
        let reduced_dot = 5u32;
        let expected_u0 = reduced_dot.wrapping_add(e1[0] as u32);
        let quotient = 0u64;

        let mut u = [0u32; LWE_DIMENSION_V0];
        u[0] = expected_u0;
        let t = array::from_fn(|i| (i as u32).wrapping_mul(29).wrapping_add(11));
        let v = 0xA5A5_5A5A;

        let ct_commit = LweAmountChip::poseidon_hash_ct_amt(&u, v);
        let t_commit = LweAmountChip::poseidon_hash_t(&t);

        let circuit = LweAmountSingleColumnNoiseRelationCircuit {
            u,
            v,
            t,
            r,
            coeffs,
            reduced_u0: reduced_dot,
            quotient,
            e1,
            e2: 0,
            noise_class: 0,
            wrap_positive: false,
            wrap_negative: false,
        };
        let prover = MockProver::run(
            18,
            &circuit,
            vec![vec![ct_commit, Fp::from(expected_u0 as u64)], vec![t_commit]],
        )
        .expect("mock prover");
        assert!(prover.verify().is_err());
    }
}
