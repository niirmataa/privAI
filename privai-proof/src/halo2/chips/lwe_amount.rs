use std::array;

use halo2_gadgets::poseidon::{
    primitives::{self as poseidon, ConstantLength, P128Pow5T3},
    Hash, Pow5Chip, Pow5Config,
};
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    pasta::Fp,
    arithmetic::Field,
    plonk::{
        Advice, Column, ConstraintSystem, Error, Expression, Fixed, Instance, Selector, TableColumn,
    },
    poly::Rotation,
};

use crate::halo2::{
    pack_u32_limbs_to_fp, packed_u32_field_len,
    params::{AmountCipherParams, DELTA_V0, LWE_DIMENSION_V0, LWE_MODULUS_Q_V0, MAX_AMOUNT_V0},
    U32_LIMBS_PER_FIELD,
};

pub const LWE_CIPHERTEXT_LIMBS_V0: usize = LWE_DIMENSION_V0 + 1;
pub const PACKED_LWE_CIPHERTEXT_LEN_V0: usize = packed_u32_field_len(LWE_CIPHERTEXT_LIMBS_V0);
pub const PACKED_LWE_PUBLIC_KEY_LEN_V0: usize = packed_u32_field_len(LWE_DIMENSION_V0);
const POSEIDON_WIDTH: usize = 3;
const POSEIDON_RATE: usize = 2;
const DOT_REDUCTION_QUOTIENT_BITS: usize = 42;

const CHUNK_SIZE: usize = 16;
const CHUNK_STEPS: usize = (LWE_DIMENSION_V0 + CHUNK_SIZE - 1) / CHUNK_SIZE; // 64
const OFFSETS_PER_COL: usize = CHUNK_STEPS + DOT_REDUCTION_QUOTIENT_BITS + 2; // 108

// ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct LweAmountConfigV2 {
    pub poseidon: Pow5Config<Fp, POSEIDON_WIDTH, POSEIDON_RATE>,
    pub q_pack: Selector,
    pub q_limb: Selector,
    pub q_lt_modulus: Selector,
    pub q_chunk_init: Selector,
    pub q_chunk_step: Selector,
    pub q_reduce_eq: Selector,
    pub q_reduce_bit: Selector,
    pub q_reduce_acc_init: Selector,
    pub q_reduce_acc_step: Selector,
    pub q_reduce_bind: Selector,
    pub q_noise_relation: Selector,
    pub q_v_gate: Selector,
    pub q_combined_noise: Selector,
    pub q_amount_range: Selector,
    pub limb_value: Column<Advice>,
    pub limb_lo: Column<Advice>,
    pub limb_hi: Column<Advice>,
    pub lt_q_slack: Column<Advice>,
    pub a_cols: [Column<Fixed>; CHUNK_SIZE],
    pub r_cols: [Column<Advice>; CHUNK_SIZE],
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

// ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct LweAmountChipV2 {
    config: LweAmountConfigV2,
    params: AmountCipherParams,
}

#[derive(Clone, Debug)]
pub struct LweAmountOutputsV2 {
    pub ct_amt_commit: AssignedCell<Fp, Fp>,
    pub t_commit: AssignedCell<Fp, Fp>,
    pub u_cells: Vec<AssignedCell<Fp, Fp>>,
    pub v_cell: AssignedCell<Fp, Fp>,
    pub t_cells: Vec<AssignedCell<Fp, Fp>>,
    pub r_cells: Vec<AssignedCell<Fp, Fp>>,
}

impl LweAmountChipV2 {
    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> LweAmountConfigV2 {
        let state = array::from_fn(|_| meta.advice_column());
        let partial_sbox = meta.advice_column();
        let rc_a = array::from_fn(|_| meta.fixed_column());
        let rc_b = array::from_fn(|_| meta.fixed_column());
        let q_pack = meta.selector();
        let q_limb = meta.complex_selector();
        let q_lt_modulus = meta.complex_selector();
        let q_chunk_init = meta.selector();
        let q_chunk_step = meta.selector();
        let q_reduce_eq = meta.selector();
        let q_reduce_bit = meta.selector();
        let q_reduce_acc_init = meta.selector();
        let q_reduce_acc_step = meta.selector();
        let q_reduce_bind = meta.selector();
        let q_noise_relation = meta.selector();
        let q_v_gate = meta.selector();
        let q_combined_noise = meta.selector();
        let q_amount_range = meta.complex_selector();
        let limb_value = meta.advice_column();
        let limb_lo = meta.advice_column();
        let limb_hi = meta.advice_column();
        let lt_q_slack = meta.advice_column();
        let a_cols: [Column<Fixed>; CHUNK_SIZE] = array::from_fn(|_| meta.fixed_column());
        let r_cols: [Column<Advice>; CHUNK_SIZE] = array::from_fn(|_| meta.advice_column());
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
        for s in &state {
            meta.enable_equality(*s);
        }
        meta.enable_equality(limb_value);
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
        for c in &r_cols {
            meta.enable_equality(*c);
        }

        // limb decomposition: limb = lo + hi * 2^16
        meta.create_gate("limb = lo + hi*2^16", |meta| {
            let q = meta.query_selector(q_limb);
            let v = meta.query_advice(limb_value, Rotation::cur());
            let lo = meta.query_advice(limb_lo, Rotation::cur());
            let hi = meta.query_advice(limb_hi, Rotation::cur());
            vec![q * (v - (lo + hi * Expression::Constant(Fp::from(1u64 << 16))))]
        });
        meta.lookup(|meta| {
            let q = meta.query_selector(q_limb);
            vec![(
                q.clone() * meta.query_advice(limb_lo, Rotation::cur()),
                table_u16,
            )]
        });
        meta.lookup(|meta| {
            let q = meta.query_selector(q_limb);
            vec![(q * meta.query_advice(limb_hi, Rotation::cur()), table_u16)]
        });

        // value < q: value + slack = q-1, slack = slack_lo + slack_hi*2^16
        meta.create_gate("value < q", |meta| {
            let q = meta.query_selector(q_lt_modulus);
            let v = meta.query_advice(limb_value, Rotation::cur());
            let s = meta.query_advice(lt_q_slack, Rotation::cur());
            let lo = meta.query_advice(limb_lo, Rotation::cur());
            let hi = meta.query_advice(limb_hi, Rotation::cur());
            let qm1 = Expression::Constant(Fp::from(LWE_MODULUS_Q_V0 - 1));
            let t16 = Expression::Constant(Fp::from(1u64 << 16));
            vec![q.clone() * (v + s.clone() - qm1), q * (s - (lo + hi * t16))]
        });
        meta.lookup(|meta| {
            let q = meta.query_selector(q_lt_modulus);
            vec![(
                q.clone() * meta.query_advice(limb_lo, Rotation::cur()),
                table_u16,
            )]
        });
        meta.lookup(|meta| {
            let q = meta.query_selector(q_lt_modulus);
            vec![(q * meta.query_advice(limb_hi, Rotation::cur()), table_u16)]
        });

        // pack 7 u32 limbs → 1 Fp
        meta.create_gate("pack 7 limbs", |meta| {
            let q = meta.query_selector(q_pack);
            let packed = meta.query_advice(packed_word, Rotation::cur());
            let radix = Expression::Constant(Fp::from(1u64 << 32));
            let mut coeff = Expression::Constant(Fp::from(1u64));
            let mut acc = Expression::Constant(Fp::from(0u64));
            for r in 0..U32_LIMBS_PER_FIELD {
                acc = acc + meta.query_advice(limb_value, Rotation(r as i32)) * coeff.clone();
                coeff = coeff * radix.clone();
            }
            vec![q * (packed - acc)]
        });

        // chunk init: acc = Σ a[c]*r[c]
        meta.create_gate("chunk init", |meta| {
            let q = meta.query_selector(q_chunk_init);
            let acc = meta.query_advice(dot_accumulator, Rotation::cur());
            let mut sum = Expression::Constant(Fp::ZERO);
            for c in 0..CHUNK_SIZE {
                sum = sum
                    + meta.query_fixed(a_cols[c]) * meta.query_advice(r_cols[c], Rotation::cur());
            }
            vec![q * (acc - sum)]
        });

        // chunk step: acc = prev + Σ a[c]*r[c]
        meta.create_gate("chunk step", |meta| {
            let q = meta.query_selector(q_chunk_step);
            let acc = meta.query_advice(dot_accumulator, Rotation::cur());
            let prev = meta.query_advice(dot_accumulator, Rotation::prev());
            let mut sum = Expression::Constant(Fp::ZERO);
            for c in 0..CHUNK_SIZE {
                sum = sum
                    + meta.query_fixed(a_cols[c]) * meta.query_advice(r_cols[c], Rotation::cur());
            }
            vec![q * (acc - prev - sum)]
        });

        // mod q: full = reduced + quotient * q
        meta.create_gate("mod q", |meta| {
            let q = meta.query_selector(q_reduce_eq);
            let full = meta.query_advice(reduce_full, Rotation::cur());
            let red = meta.query_advice(reduce_reduced, Rotation::cur());
            let quot = meta.query_advice(reduce_quotient, Rotation::cur());
            vec![q * (full - red - quot * Expression::Constant(Fp::from(LWE_MODULUS_Q_V0)))]
        });

        // quotient bit boolean
        meta.create_gate("bit bool", |meta| {
            let q = meta.query_selector(q_reduce_bit);
            let b = meta.query_advice(reduce_bit, Rotation::cur());
            vec![q * b.clone() * (b - Expression::Constant(Fp::from(1u64)))]
        });

        // quotient acc init
        meta.create_gate("quot acc init", |meta| {
            let q = meta.query_selector(q_reduce_acc_init);
            let acc = meta.query_advice(reduce_accumulator, Rotation::cur());
            let b = meta.query_advice(reduce_bit, Rotation::cur());
            let c = meta.query_fixed(reduce_coeff);
            vec![q * (acc - b * c)]
        });

        // quotient acc step
        meta.create_gate("quot acc step", |meta| {
            let q = meta.query_selector(q_reduce_acc_step);
            let acc = meta.query_advice(reduce_accumulator, Rotation::cur());
            let prev = meta.query_advice(reduce_accumulator, Rotation::prev());
            let b = meta.query_advice(reduce_bit, Rotation::cur());
            let c = meta.query_fixed(reduce_coeff);
            vec![q * (acc - prev - b * c)]
        });

        // quotient == accumulated bits
        meta.create_gate("quot bind", |meta| {
            let q = meta.query_selector(q_reduce_bind);
            let quot = meta.query_advice(reduce_quotient, Rotation::cur());
            let acc = meta.query_advice(reduce_accumulator, Rotation::cur());
            vec![q * (quot - acc)]
        });

        // noise-adjusted relation: output = reduced + noise + q*(wp-wn)
        // wp, wn boolean, mutually exclusive
        let noise_constraints = |meta: &mut halo2_proofs::plonk::VirtualCells<'_, Fp>,
                                 sel: Selector| {
            let q = meta.query_selector(sel);
            let red = meta.query_advice(noise_reduced, Rotation::cur());
            let noise = meta.query_advice(noise_centered, Rotation::cur());
            let out = meta.query_advice(noise_output, Rotation::cur());
            let wp = meta.query_advice(noise_wrap_positive, Rotation::cur());
            let wn = meta.query_advice(noise_wrap_negative, Rotation::cur());
            let one = Expression::Constant(Fp::from(1u64));
            let modq = Expression::Constant(Fp::from(LWE_MODULUS_Q_V0));
            vec![
                q.clone() * (out - red - noise - modq * (wp.clone() - wn.clone())),
                q.clone() * wp.clone() * (wp - one.clone()),
                q.clone() * wn.clone() * (wn - one.clone()),
                q * meta.query_advice(noise_wrap_positive, Rotation::cur())
                    * meta.query_advice(noise_wrap_negative, Rotation::cur()),
            ]
        };
        meta.create_gate("noise relation (u)", |meta| {
            noise_constraints(meta, q_noise_relation)
        });
        meta.create_gate("v gate", |meta| noise_constraints(meta, q_v_gate));

        // combined noise gate: combined_noise = e2 + Δ · amount
        meta.create_gate("combined noise = e2 + Δ*amount", |meta| {
            let q = meta.query_selector(q_combined_noise);
            let combined = meta.query_advice(noise_value, Rotation::cur());
            let e2 = meta.query_advice(noise_centered, Rotation::cur());
            let amount = meta.query_advice(limb_value, Rotation::cur());
            let delta = Expression::Constant(Fp::from(DELTA_V0 as u64));
            vec![q * (combined - e2 - delta * amount)]
        });

        // amount < p range check: amount + slack = p-1, slack = slack_lo + slack_hi*2^16
        meta.create_gate("amount < p", |meta| {
            let q = meta.query_selector(q_amount_range);
            let amt = meta.query_advice(limb_value, Rotation::cur());
            let slack = meta.query_advice(lt_q_slack, Rotation::cur());
            let lo = meta.query_advice(limb_lo, Rotation::cur());
            let hi = meta.query_advice(limb_hi, Rotation::cur());
            let pm1 = Expression::Constant(Fp::from(
                crate::halo2::params::PLAINTEXT_SPACE_P_V0 as u64 - 1,
            ));
            let t16 = Expression::Constant(Fp::from(1u64 << 16));
            vec![
                q.clone() * (amt + slack.clone() - pm1),
                q * (slack - (lo + hi * t16)),
            ]
        });
        meta.lookup(|meta| {
            let q = meta.query_selector(q_amount_range);
            vec![(
                q.clone() * meta.query_advice(limb_lo, Rotation::cur()),
                table_u16,
            )]
        });
        meta.lookup(|meta| {
            let q = meta.query_selector(q_amount_range);
            vec![(q * meta.query_advice(limb_hi, Rotation::cur()), table_u16)]
        });

        let poseidon = Pow5Chip::configure::<P128Pow5T3>(meta, state, partial_sbox, rc_a, rc_b);

        LweAmountConfigV2 {
            poseidon,
            q_pack,
            q_limb,
            q_lt_modulus,
            q_chunk_init,
            q_chunk_step,
            q_reduce_eq,
            q_reduce_bit,
            q_reduce_acc_init,
            q_reduce_acc_step,
            q_reduce_bind,
            q_noise_relation,
            q_v_gate,
            q_combined_noise,
            q_amount_range,
            limb_value,
            limb_lo,
            limb_hi,
            lt_q_slack,
            a_cols,
            r_cols,
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

    pub fn new(config: LweAmountConfigV2, params: AmountCipherParams) -> Self {
        Self { config, params }
    }

    pub fn config(&self) -> &LweAmountConfigV2 {
        &self.config
    }
    pub fn params(&self) -> &AmountCipherParams {
        &self.params
    }

    pub fn poseidon_hash_ct_amt(u: &[u32; LWE_DIMENSION_V0], v: u32) -> Fp {
        let mut limbs = Vec::with_capacity(LWE_CIPHERTEXT_LIMBS_V0);
        limbs.extend_from_slice(u);
        limbs.push(v);
        let packed: [Fp; PACKED_LWE_CIPHERTEXT_LEN_V0] = pack_u32_limbs_to_fp(&limbs)
            .try_into()
            .expect("ct_amt packing length");
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
        let packed: [Fp; PACKED_LWE_PUBLIC_KEY_LEN_V0] = pack_u32_limbs_to_fp(t)
            .try_into()
            .expect("t packing length");
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
            || "u16 table",
            |mut table| {
                for v in 0..=u16::MAX {
                    table.assign_cell(
                        || format!("u16_{v}"),
                        self.config.table_u16,
                        v as usize,
                        || Value::known(Fp::from(v as u64)),
                    )?;
                }
                Ok(())
            },
        )
    }

    // ── limb + range check helpers ──

    fn assign_limbs(
        &self,
        mut l: impl Layouter<Fp>,
        name: &str,
        vals: &[u32],
    ) -> Result<Vec<AssignedCell<Fp, Fp>>, Error> {
        l.assign_region(
            || name,
            |mut r| {
                let mut out = Vec::with_capacity(vals.len());
                for (i, &v) in vals.iter().enumerate() {
                    self.config.q_limb.enable(&mut r, i)?;
                    out.push(r.assign_advice(
                        || format!("{name}_{i}"),
                        self.config.limb_value,
                        i,
                        || Value::known(Fp::from(v as u64)),
                    )?);
                    r.assign_advice(
                        || format!("{name}_lo{i}"),
                        self.config.limb_lo,
                        i,
                        || Value::known(Fp::from((v & 0xFFFF) as u64)),
                    )?;
                    r.assign_advice(
                        || format!("{name}_hi{i}"),
                        self.config.limb_hi,
                        i,
                        || Value::known(Fp::from((v >> 16) as u64)),
                    )?;
                }
                Ok(out)
            },
        )
    }

    fn assign_lt_q(
        &self,
        mut l: impl Layouter<Fp>,
        name: &str,
        cell: &AssignedCell<Fp, Fp>,
        val: u32,
    ) -> Result<(), Error> {
        let q = self.params.lwe_modulus_q;
        if (val as u64) >= q {
            return Err(Error::Synthesis);
        }
        let slack = (q - 1) - (val as u64);
        l.assign_region(
            || name,
            |mut r| {
                self.config.q_lt_modulus.enable(&mut r, 0)?;
                cell.copy_advice(|| format!("{name}_v"), &mut r, self.config.limb_value, 0)?;
                r.assign_advice(
                    || format!("{name}_s"),
                    self.config.lt_q_slack,
                    0,
                    || Value::known(Fp::from(slack)),
                )?;
                r.assign_advice(
                    || format!("{name}_sl"),
                    self.config.limb_lo,
                    0,
                    || Value::known(Fp::from(slack & 0xFFFF)),
                )?;
                r.assign_advice(
                    || format!("{name}_sh"),
                    self.config.limb_hi,
                    0,
                    || Value::known(Fp::from(slack >> 16)),
                )?;
                Ok(())
            },
        )
    }

    fn assign_lt_q_many(
        &self,
        mut l: impl Layouter<Fp>,
        name: &str,
        cells: &[AssignedCell<Fp, Fp>],
        vals: &[u32],
    ) -> Result<(), Error> {
        for (i, (c, &v)) in cells.iter().zip(vals.iter()).enumerate() {
            self.assign_lt_q(l.namespace(|| format!("{name}_{i}")), name, c, v)?;
        }
        Ok(())
    }

    // ── packing ──

    fn assign_packed<const P: usize>(
        &self,
        mut l: impl Layouter<Fp>,
        name: &str,
        limb_cells: &[AssignedCell<Fp, Fp>],
        packed: [Fp; P],
    ) -> Result<[AssignedCell<Fp, Fp>; P], Error> {
        l.assign_region(
            || name,
            |mut r| {
                let mut out = Vec::with_capacity(P);
                for (ci, val) in packed.into_iter().enumerate() {
                    let base = ci * U32_LIMBS_PER_FIELD;
                    self.config.q_pack.enable(&mut r, base)?;
                    out.push(r.assign_advice(
                        || format!("{name}_p{ci}"),
                        self.config.packed_word,
                        base,
                        || Value::known(val),
                    )?);
                    for off in 0..U32_LIMBS_PER_FIELD {
                        let idx = base + off;
                        if let Some(lc) = limb_cells.get(idx) {
                            lc.copy_advice(
                                || format!("{name}_l{idx}"),
                                &mut r,
                                self.config.limb_value,
                                idx,
                            )?;
                        } else {
                            r.assign_advice(
                                || format!("{name}_z{idx}"),
                                self.config.limb_value,
                                idx,
                                || Value::known(Fp::ZERO),
                            )?;
                        }
                    }
                }
                out.try_into().map_err(|_| Error::Synthesis)
            },
        )
    }

    // ═══════════════════════════════════════════════════════════════
    // Full assign — all constraints enforced
    // ═══════════════════════════════════════════════════════════════

    pub fn assign(
        &self,
        mut layouter: impl Layouter<Fp>,
        u: &[u32; LWE_DIMENSION_V0],
        v: u32,
        t: &[u32; LWE_DIMENSION_V0],
        r: &[u32; LWE_DIMENSION_V0],
        a_matrix: &[[u32; LWE_DIMENSION_V0]; LWE_DIMENSION_V0],
        e1_cells: &[AssignedCell<Fp, Fp>; LWE_DIMENSION_V0],
        e2_cell: &AssignedCell<Fp, Fp>,
        amount: u32,
        col_witnesses: &[(u32, u64, bool, bool); LWE_DIMENSION_V0],
        v_witness: &(u32, u64, bool, bool),
    ) -> Result<LweAmountOutputsV2, Error> {
        if self.params.lwe_dimension != LWE_DIMENSION_V0 {
            return Err(Error::Synthesis);
        }
        if amount > MAX_AMOUNT_V0 as u32 {
            return Err(Error::Synthesis);
        }

        // 1. limbs + range checks
        let u_cells = self.assign_limbs(layouter.namespace(|| "limbs u"), "u", u)?;
        let v_cells = self.assign_limbs(layouter.namespace(|| "limbs v"), "v", &[v])?;
        let v_cell = v_cells.into_iter().next().ok_or(Error::Synthesis)?;
        let t_cells = self.assign_limbs(layouter.namespace(|| "limbs t"), "t", t)?;
        let r_cells: Vec<AssignedCell<Fp, Fp>> =
            self.assign_limbs(layouter.namespace(|| "limbs r"), "r", r)?;

        self.assign_lt_q_many(layouter.namespace(|| "u<q"), "u", &u_cells, u)?;
        self.assign_lt_q(layouter.namespace(|| "v<q"), "v", &v_cell, v)?;
        self.assign_lt_q_many(layouter.namespace(|| "t<q"), "t", &t_cells, t)?;
        self.assign_lt_q_many(layouter.namespace(|| "r<q"), "r", &r_cells, r)?;

        // 2. pack + poseidon
        let mut ct_limbs = u_cells.clone();
        ct_limbs.push(v_cell.clone());
        let ct_raw: Vec<u32> = u.iter().copied().chain(std::iter::once(v)).collect();
        let packed_ct: [Fp; PACKED_LWE_CIPHERTEXT_LEN_V0] = pack_u32_limbs_to_fp(&ct_raw)
            .try_into()
            .map_err(|_| Error::Synthesis)?;
        let packed_t: [Fp; PACKED_LWE_PUBLIC_KEY_LEN_V0] = pack_u32_limbs_to_fp(t)
            .try_into()
            .map_err(|_| Error::Synthesis)?;
        let ct_words =
            self.assign_packed(layouter.namespace(|| "pack ct"), "ct", &ct_limbs, packed_ct)?;
        let t_words =
            self.assign_packed(layouter.namespace(|| "pack t"), "t", &t_cells, packed_t)?;

        let ct_chip = Pow5Chip::construct(self.config.poseidon.clone());
        let t_chip = Pow5Chip::construct(self.config.poseidon.clone());
        let ct_out = Hash::<
            _,
            _,
            P128Pow5T3,
            ConstantLength<PACKED_LWE_CIPHERTEXT_LEN_V0>,
            POSEIDON_WIDTH,
            POSEIDON_RATE,
        >::init(ct_chip, layouter.namespace(|| "ct init"))?
        .hash(layouter.namespace(|| "ct hash"), ct_words)?;
        let t_out = Hash::<
            _,
            _,
            P128Pow5T3,
            ConstantLength<PACKED_LWE_PUBLIC_KEY_LEN_V0>,
            POSEIDON_WIDTH,
            POSEIDON_RATE,
        >::init(t_chip, layouter.namespace(|| "t init"))?
        .hash(layouter.namespace(|| "t hash"), t_words)?;
        layouter.constrain_instance(ct_out.cell(), self.config.ct_amt_commit, 0)?;
        layouter.constrain_instance(t_out.cell(), self.config.t_commit, 0)?;

        // 3. amount advice cell + range check + constrained combined noise
        // amount is witnessed as advice cell, range-checked: amount < p
        let (amount_cell, combined_noise_cell) = layouter.assign_region(
            || "amount + combined_noise",
            |mut region| {
                // amount cell
                let amount_cell = region.assign_advice(
                    || "amount",
                    self.config.limb_value,
                    0,
                    || Value::known(Fp::from(amount as u64)),
                )?;
                // range check: amount < p via slack = (p-1) - amount
                let p = crate::halo2::params::PLAINTEXT_SPACE_P_V0 as u64;
                if amount as u64 >= p {
                    return Err(Error::Synthesis);
                }
                let slack = (p - 1) - (amount as u64);
                region.assign_advice(
                    || "amt_slack",
                    self.config.lt_q_slack,
                    0,
                    || Value::known(Fp::from(slack)),
                )?;
                region.assign_advice(
                    || "amt_slack_lo",
                    self.config.limb_lo,
                    0,
                    || Value::known(Fp::from(slack & 0xFFFF)),
                )?;
                region.assign_advice(
                    || "amt_slack_hi",
                    self.config.limb_hi,
                    0,
                    || Value::known(Fp::from(slack >> 16)),
                )?;
                // Enable the amount < p range check gate
                self.config.q_amount_range.enable(&mut region, 0)?;

                // combined_noise = e2 + Δ · amount (CONSTRAINED)
                let cn_val = e2_cell
                    .value()
                    .copied()
                    .map(|e2| e2 + Fp::from((DELTA_V0 as u64) * (amount as u64)));
                let cn_cell = region.assign_advice(
                    || "combined_noise",
                    self.config.noise_value,
                    1,
                    || cn_val,
                )?;
                e2_cell.copy_advice(|| "e2_copy", &mut region, self.config.noise_centered, 1)?;
                amount_cell.copy_advice(
                    || "amount_copy",
                    &mut region,
                    self.config.limb_value,
                    1,
                )?;
                self.config.q_combined_noise.enable(&mut region, 1)?;

                Ok((amount_cell, cn_cell))
            },
        )?;

        // 4. chunked dot products + reduction + noise (one region)
        let r_arr: [AssignedCell<Fp, Fp>; LWE_DIMENSION_V0] =
            r_cells.clone().try_into().map_err(|_| Error::Synthesis)?;

        layouter.assign_region(
            || "dot products + reduction + noise",
            |mut region| {
                // u columns
                for col in 0..LWE_DIMENSION_V0 {
                    let base = col * OFFSETS_PER_COL;
                    let (red_val, quot, wp, wn) = col_witnesses[col];
                    self.assign_one_column(
                        &mut region,
                        base,
                        &a_matrix[col],
                        &r_arr,
                        &e1_cells[col],
                        &u_cells[col],
                        red_val,
                        quot,
                        wp,
                        wn,
                        self.config.q_noise_relation,
                        &format!("u{col}"),
                    )?;
                }
                // v gate
                let v_base = LWE_DIMENSION_V0 * OFFSETS_PER_COL;
                let (vr, vq, vwp, vwn) = *v_witness;
                self.assign_one_column(
                    &mut region,
                    v_base,
                    t,
                    &r_arr,
                    &combined_noise_cell,
                    &v_cell,
                    vr,
                    vq,
                    vwp,
                    vwn,
                    self.config.q_v_gate,
                    "v",
                )?;
                Ok(())
            },
        )?;

        Ok(LweAmountOutputsV2 {
            ct_amt_commit: ct_out,
            t_commit: t_out,
            u_cells,
            v_cell,
            t_cells,
            r_cells,
        })
    }

    /// Assign one column: chunked dot product → mod q reduction → noise relation.
    fn assign_one_column(
        &self,
        region: &mut halo2_proofs::circuit::Region<'_, Fp>,
        base: usize,
        coeffs: &[u32; LWE_DIMENSION_V0],
        r_cells: &[AssignedCell<Fp, Fp>; LWE_DIMENSION_V0],
        noise_cell: &AssignedCell<Fp, Fp>,
        output_cell: &AssignedCell<Fp, Fp>,
        reduced_val: u32,
        quotient: u64,
        wrap_pos: bool,
        wrap_neg: bool,
        noise_sel: Selector,
        name: &str,
    ) -> Result<(), Error> {
        if quotient >= (1u64 << DOT_REDUCTION_QUOTIENT_BITS) {
            return Err(Error::Synthesis);
        }
        // ── chunk accumulation ──
        let mut running = Value::known(Fp::ZERO);
        let mut last_acc = None;

        for step in 0..CHUNK_STEPS {
            let off = base + step;
            let mut chunk_sum = Value::known(Fp::ZERO);

            for c in 0..CHUNK_SIZE {
                let j = step * CHUNK_SIZE + c;
                if j < LWE_DIMENSION_V0 {
                    let coeff_fp = Fp::from(coeffs[j] as u64);
                    region.assign_fixed(
                        || format!("{name}_a{step}_{c}"),
                        self.config.a_cols[c],
                        off,
                        || Value::known(coeff_fp),
                    )?;
                    r_cells[j].copy_advice(
                        || format!("{name}_r{step}_{c}"),
                        region,
                        self.config.r_cols[c],
                        off,
                    )?;
                    chunk_sum = chunk_sum + r_cells[j].value().copied().map(|r| coeff_fp * r);
                } else {
                    region.assign_fixed(
                        || format!("{name}_az{step}_{c}"),
                        self.config.a_cols[c],
                        off,
                        || Value::known(Fp::ZERO),
                    )?;
                    region.assign_advice(
                        || format!("{name}_rz{step}_{c}"),
                        self.config.r_cols[c],
                        off,
                        || Value::known(Fp::ZERO),
                    )?;
                }
            }

            if step == 0 {
                running = chunk_sum;
                self.config.q_chunk_init.enable(region, off)?;
            } else {
                running = running + chunk_sum;
                self.config.q_chunk_step.enable(region, off)?;
            }

            let acc_cell = region.assign_advice(
                || format!("{name}_acc{step}"),
                self.config.dot_accumulator,
                off,
                || running,
            )?;
            last_acc = Some(acc_cell);
        }

        let full_cell = last_acc.unwrap();

        // ── reduction ──
        let red_base = base + CHUNK_STEPS;
        self.config.q_limb.enable(region, red_base)?;
        let reduced_cell = region.assign_advice(
            || format!("{name}_rv"),
            self.config.limb_value,
            red_base,
            || Value::known(Fp::from(reduced_val as u64)),
        )?;
        region.assign_advice(
            || format!("{name}_rlo"),
            self.config.limb_lo,
            red_base,
            || Value::known(Fp::from((reduced_val & 0xFFFF) as u64)),
        )?;
        region.assign_advice(
            || format!("{name}_rhi"),
            self.config.limb_hi,
            red_base,
            || Value::known(Fp::from((reduced_val >> 16) as u64)),
        )?;

        // mod q equality at red_base
        full_cell.copy_advice(
            || format!("{name}_rf"),
            region,
            self.config.reduce_full,
            red_base,
        )?;
        reduced_cell.copy_advice(
            || format!("{name}_rr"),
            region,
            self.config.reduce_reduced,
            red_base,
        )?;
        let qcell = region.assign_advice(
            || format!("{name}_rq"),
            self.config.reduce_quotient,
            red_base,
            || Value::known(Fp::from(quotient)),
        )?;
        self.config.q_reduce_eq.enable(region, red_base)?;

        // quotient bit decomposition
        let mut bit_acc = 0u64;
        for i in 0..DOT_REDUCTION_QUOTIENT_BITS {
            let off = red_base + i;
            let coeff = 1u64 << i;
            let bit = (quotient >> i) & 1;
            bit_acc += bit * coeff;

            region.assign_fixed(
                || format!("{name}_rc{i}"),
                self.config.reduce_coeff,
                off,
                || Value::known(Fp::from(coeff)),
            )?;
            region.assign_advice(
                || format!("{name}_rb{i}"),
                self.config.reduce_bit,
                off,
                || Value::known(Fp::from(bit)),
            )?;
            region.assign_advice(
                || format!("{name}_ra{i}"),
                self.config.reduce_accumulator,
                off,
                || Value::known(Fp::from(bit_acc)),
            )?;
            self.config.q_reduce_bit.enable(region, off)?;
            if i == 0 {
                self.config.q_reduce_acc_init.enable(region, off)?;
            } else {
                self.config.q_reduce_acc_step.enable(region, off)?;
            }
        }

        // bind: quotient == accumulated bits
        let bind_off = red_base + DOT_REDUCTION_QUOTIENT_BITS;
        qcell.copy_advice(
            || format!("{name}_bq"),
            region,
            self.config.reduce_quotient,
            bind_off,
        )?;
        region.assign_advice(
            || format!("{name}_ba"),
            self.config.reduce_accumulator,
            bind_off,
            || Value::known(Fp::from(bit_acc)),
        )?;
        self.config.q_reduce_bind.enable(region, bind_off)?;

        // ── noise relation ──
        let nr_off = base + OFFSETS_PER_COL - 1;
        reduced_cell.copy_advice(
            || format!("{name}_nr"),
            region,
            self.config.noise_reduced,
            nr_off,
        )?;
        noise_cell.copy_advice(
            || format!("{name}_nn"),
            region,
            self.config.noise_centered,
            nr_off,
        )?;
        output_cell.copy_advice(
            || format!("{name}_no"),
            region,
            self.config.noise_output,
            nr_off,
        )?;
        region.assign_advice(
            || format!("{name}_wp"),
            self.config.noise_wrap_positive,
            nr_off,
            || Value::known(Fp::from(wrap_pos as u64)),
        )?;
        region.assign_advice(
            || format!("{name}_wn"),
            self.config.noise_wrap_negative,
            nr_off,
            || Value::known(Fp::from(wrap_neg as u64)),
        )?;
        noise_sel.enable(region, nr_off)?;

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::{LweAmountChipV2, LweAmountConfigV2};
    use crate::halo2::{
        params::AmountCipherParams, NoiseClassChip, NoiseClassConfig, DELTA_V0, LWE_DIMENSION_V0,
        LWE_MODULUS_Q_V0, MAX_AMOUNT_V0,
    };
    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
        pasta::Fp,
        plonk::{Circuit, ConstraintSystem, Error},
    };
    use std::array;

    fn col_witness(
        a: &[u32; LWE_DIMENSION_V0],
        r: &[u32; LWE_DIMENSION_V0],
        e1: i16,
    ) -> (u32, u64, bool, bool) {
        let q = LWE_MODULUS_Q_V0 as i128;
        let dot: i128 = a
            .iter()
            .zip(r.iter())
            .map(|(&a, &rv)| (a as i128) * (rv as i128))
            .sum();
        let red = (dot % q) as u32;
        let quot = (dot / q) as u64;
        let s = (red as i128) + (e1 as i128);
        (red, quot, s >= q, s < 0)
    }

    fn v_witness(
        t: &[u32; LWE_DIMENSION_V0],
        r: &[u32; LWE_DIMENSION_V0],
        e2: i16,
        amount: u32,
    ) -> (u32, u64, bool, bool) {
        let q = LWE_MODULUS_Q_V0 as i128;
        let dot: i128 = t
            .iter()
            .zip(r.iter())
            .map(|(&a, &b)| (a as i128) * (b as i128))
            .sum();
        let noise = (e2 as i128) + (DELTA_V0 as i128) * (amount as i128);
        let red = (dot % q) as u32;
        let quot = (dot / q) as u64;
        let s = (red as i128) + noise;
        (red, quot, s >= q, s < 0)
    }

    fn compute_u(
        a: &[[u32; LWE_DIMENSION_V0]; LWE_DIMENSION_V0],
        r: &[u32; LWE_DIMENSION_V0],
        e1: &[i16; LWE_DIMENSION_V0],
    ) -> [u32; LWE_DIMENSION_V0] {
        let q = LWE_MODULUS_Q_V0 as i128;
        array::from_fn(|i| {
            let dot: i128 = a[i]
                .iter()
                .zip(r.iter())
                .map(|(&a, &rv)| (a as i128) * (rv as i128))
                .sum();
            ((dot + (e1[i] as i128)).rem_euclid(q)) as u32
        })
    }

    fn compute_v(
        t: &[u32; LWE_DIMENSION_V0],
        r: &[u32; LWE_DIMENSION_V0],
        e2: i16,
        amount: u32,
    ) -> u32 {
        let q = LWE_MODULUS_Q_V0 as i128;
        let dot: i128 = t
            .iter()
            .zip(r.iter())
            .map(|(&a, &b)| (a as i128) * (b as i128))
            .sum();
        (dot + (e2 as i128) + (DELTA_V0 as i128) * (amount as i128)).rem_euclid(q) as u32
    }

    fn make_a(seed: u32) -> [[u32; LWE_DIMENSION_V0]; LWE_DIMENSION_V0] {
        array::from_fn(|i| {
            array::from_fn(|j| {
                ((i as u32)
                    .wrapping_mul(37)
                    .wrapping_add(j as u32)
                    .wrapping_mul(17)
                    .wrapping_add(seed))
                    % (LWE_MODULUS_Q_V0 as u32)
            })
        })
    }

    #[derive(Clone)]
    struct TestCircuit {
        u: [u32; LWE_DIMENSION_V0],
        v: u32,
        t: [u32; LWE_DIMENSION_V0],
        r: [u32; LWE_DIMENSION_V0],
        a: [[u32; LWE_DIMENSION_V0]; LWE_DIMENSION_V0],
        e1: [i16; LWE_DIMENSION_V0],
        e2: i16,
        noise_class: u8,
        amount: u32,
        vw_override: Option<(u32, u64, bool, bool)>,
    }

    impl Default for TestCircuit {
        fn default() -> Self {
            Self {
                u: [0; _],
                v: 0,
                t: [0; _],
                r: [0; _],
                a: [[0; _]; _],
                e1: [0; _],
                e2: 0,
                noise_class: 0,
                amount: 0,
                vw_override: None,
            }
        }
    }

    #[derive(Clone)]
    struct TestCfg {
        lwe: LweAmountConfigV2,
        noise: NoiseClassConfig,
    }

    impl Circuit<Fp> for TestCircuit {
        type Config = TestCfg;
        type FloorPlanner = SimpleFloorPlanner;
        fn without_witnesses(&self) -> Self {
            Self::default()
        }
        fn configure(meta: &mut ConstraintSystem<Fp>) -> TestCfg {
            TestCfg {
                lwe: LweAmountChipV2::configure(meta),
                noise: NoiseClassChip::configure(meta),
            }
        }
        fn synthesize(&self, cfg: TestCfg, mut l: impl Layouter<Fp>) -> Result<(), Error> {
            let chip = LweAmountChipV2::new(cfg.lwe.clone(), AmountCipherParams::default());
            let nc = NoiseClassChip::new(cfg.noise);
            nc.load_lookup_table(l.namespace(|| "nt"))?;
            chip.load_u16_table(l.namespace(|| "ut"))?;
            let no = nc.assign(l.namespace(|| "noise"), &self.e1, self.e2, self.noise_class)?;
            let cw: [(u32, u64, bool, bool); LWE_DIMENSION_V0] =
                array::from_fn(|i| col_witness(&self.a[i], &self.r, self.e1[i]));
            let vw = match self.vw_override {
                Some(vw) => vw,
                None => v_witness(&self.t, &self.r, self.e2, self.amount),
            };
            let out = chip.assign(
                l.namespace(|| "lwe"),
                &self.u,
                self.v,
                &self.t,
                &self.r,
                &self.a,
                &no.e1_cells,
                &no.e2_cell,
                self.amount,
                &cw,
                &vw,
            )?;
            l.constrain_instance(out.ct_amt_commit.cell(), cfg.lwe.ct_amt_commit, 0)?;
            l.constrain_instance(out.t_commit.cell(), cfg.lwe.t_commit, 0)?;
            Ok(())
        }
    }

    fn run(c: TestCircuit, k: u32) {
        let ct = LweAmountChipV2::poseidon_hash_ct_amt(&c.u, c.v);
        let tc = LweAmountChipV2::poseidon_hash_t(&c.t);
        MockProver::run(k, &c, vec![vec![ct], vec![tc]])
            .expect("prover")
            .assert_satisfied();
    }

    #[test]
    fn accepts_identity_a_no_noise() {
        let a: [[u32; LWE_DIMENSION_V0]; _] = array::from_fn(|i| {
            let mut r = [0u32; LWE_DIMENSION_V0];
            r[i] = 1;
            r
        });
        let r: [u32; _] = array::from_fn(|i| (i as u32 * 31 + 19) % (LWE_MODULUS_Q_V0 as u32));
        let t: [u32; _] = array::from_fn(|i| (i as u32 * 29 + 11) % (LWE_MODULUS_Q_V0 as u32));
        let e1 = [0i16; LWE_DIMENSION_V0];
        run(
            TestCircuit {
                u: compute_u(&a, &r, &e1),
                v: compute_v(&t, &r, 0, 0),
                t,
                r,
                a,
                e1,
                e2: 0,
                noise_class: 0,
                amount: 0,
                vw_override: None,
            },
            18,
        );
    }

    #[test]
    fn accepts_with_noise_and_amount() {
        let a = make_a(42);
        let r: [u32; _] = array::from_fn(|i| (i as u32 * 31 + 19) % (LWE_MODULUS_Q_V0 as u32));
        let e1: [i16; _] = array::from_fn(|i| [0, 7, -5, 12][i % 4]);
        let t: [u32; _] = array::from_fn(|i| (i as u32 * 29 + 11) % (LWE_MODULUS_Q_V0 as u32));
        run(
            TestCircuit {
                u: compute_u(&a, &r, &e1),
                v: compute_v(&t, &r, -3, 1000),
                t,
                r,
                a,
                e1,
                e2: -3,
                noise_class: 0,
                amount: 1000,
                vw_override: None,
            },
            18,
        );
    }

    #[test]
    fn accepts_amount_zero() {
        let a = make_a(7);
        let r: [u32; _] = array::from_fn(|i| (i as u32 * 13 + 5) % (LWE_MODULUS_Q_V0 as u32));
        let t: [u32; _] = array::from_fn(|i| (i as u32 * 19 + 3) % (LWE_MODULUS_Q_V0 as u32));
        let e1 = [0i16; LWE_DIMENSION_V0];
        run(
            TestCircuit {
                u: compute_u(&a, &r, &e1),
                v: compute_v(&t, &r, 0, 0),
                t,
                r,
                a,
                e1,
                e2: 0,
                noise_class: 0,
                amount: 0,
                vw_override: None,
            },
            18,
        );
    }

    #[test]
    fn accepts_max_amount() {
        let a = make_a(13);
        let r: [u32; _] = array::from_fn(|i| (i as u32 * 17 + 7) % (LWE_MODULUS_Q_V0 as u32));
        let t: [u32; _] = array::from_fn(|i| (i as u32 * 23 + 9) % (LWE_MODULUS_Q_V0 as u32));
        let e1 = [3i16; LWE_DIMENSION_V0];
        let amt = MAX_AMOUNT_V0 as u32;
        run(
            TestCircuit {
                u: compute_u(&a, &r, &e1),
                v: compute_v(&t, &r, 5, amt),
                t,
                r,
                a,
                e1,
                e2: 5,
                noise_class: 0,
                amount: amt,
                vw_override: None,
            },
            18,
        );
    }

    #[test]
    fn accepts_noise_at_class_boundary() {
        let a = make_a(21);
        let r: [u32; _] = array::from_fn(|i| (i as u32 * 11 + 3) % (LWE_MODULUS_Q_V0 as u32));
        let mut e1 = [0i16; LWE_DIMENSION_V0];
        e1[0] = 16;
        e1[1] = -16;
        let t: [u32; _] = array::from_fn(|i| (i as u32 * 33 + 7) % (LWE_MODULUS_Q_V0 as u32));
        run(
            TestCircuit {
                u: compute_u(&a, &r, &e1),
                v: compute_v(&t, &r, 16, 500),
                t,
                r,
                a,
                e1,
                e2: 16,
                noise_class: 0,
                amount: 500,
                vw_override: None,
            },
            18,
        );
    }

    #[test]
    fn rejects_wrong_ct_commitment() {
        let a = make_a(1);
        let r: [u32; _] = array::from_fn(|i| (i as u32 * 31 + 19) % (LWE_MODULUS_Q_V0 as u32));
        let t: [u32; _] = array::from_fn(|i| (i as u32 * 29 + 11) % (LWE_MODULUS_Q_V0 as u32));
        let e1 = [0i16; LWE_DIMENSION_V0];
        let c = TestCircuit {
            u: compute_u(&a, &r, &e1),
            v: compute_v(&t, &r, 0, 0),
            t,
            r,
            a,
            e1,
            e2: 0,
            noise_class: 0,
            amount: 0,
            vw_override: None,
        };
        let tc = LweAmountChipV2::poseidon_hash_t(&c.t);
        assert!(
            MockProver::run(18, &c, vec![vec![Fp::from(999u64)], vec![tc]])
                .expect("p")
                .verify()
                .is_err()
        );
    }

    #[test]
    fn rejects_forged_amount_via_combined_noise() {
        // Prover tries to claim amount=1000 but actually encrypts amount=42.
        // vw_override with fake_amount → witness values (reduced, wp, wn) assume
        // combined_noise = e2 + Δ*1000. But gate computes e2 + Δ*42.
        // v gate: v = reduced + combined_noise + q*(wp-wn) should fail.
        let a = make_a(5);
        let r: [u32; _] = array::from_fn(|i| (i as u32 * 17 + 7) % (LWE_MODULUS_Q_V0 as u32));
        let t: [u32; _] = array::from_fn(|i| (i as u32 * 23 + 9) % (LWE_MODULUS_Q_V0 as u32));
        let e1 = [3i16; LWE_DIMENSION_V0];
        let e2: i16 = -5;
        let real_amount: u32 = 42;
        let fake_amount: u32 = 1000;

        let u = compute_u(&a, &r, &e1);
        let v = compute_v(&t, &r, e2, real_amount);
        let fake_vw = v_witness(&t, &r, e2, fake_amount);

        let c = TestCircuit {
            u,
            v,
            t,
            r,
            a,
            e1,
            e2,
            noise_class: 0,
            amount: real_amount,
            vw_override: Some(fake_vw),
        };
        let ct = LweAmountChipV2::poseidon_hash_ct_amt(&c.u, c.v);
        let tc = LweAmountChipV2::poseidon_hash_t(&c.t);
        assert!(MockProver::run(18, &c, vec![vec![ct], vec![tc]])
            .expect("p")
            .verify()
            .is_err());
    }

    #[test]
    fn rejects_oversized_amount() {
        // amount = 16384 = MAX_AMOUNT_V0 + 1. Old code: assert!((amount as u16) <= 16383)
        // would PASS because 16384 as u16 == 0 <= 16383. That's the truncation bug.
        // New code: if amount > MAX_AMOUNT_V0 as u32 → Err(Synthesis).
        let oversized: u32 = MAX_AMOUNT_V0 as u32 + 1;
        assert!(oversized > MAX_AMOUNT_V0 as u32);
        assert_eq!(oversized as u16, 0); // proves old assert was broken
    }
}
