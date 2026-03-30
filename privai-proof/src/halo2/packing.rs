use halo2_proofs::{arithmetic::Field, pasta::Fp};

pub const U32_LIMBS_PER_FIELD: usize = 7;

pub const fn packed_u32_field_len(limbs_len: usize) -> usize {
    if limbs_len == 0 {
        0
    } else {
        (limbs_len + U32_LIMBS_PER_FIELD - 1) / U32_LIMBS_PER_FIELD
    }
}

pub fn pack_u32_limbs_to_fp(limbs: &[u32]) -> Vec<Fp> {
    if limbs.is_empty() {
        return Vec::new();
    }

    let radix = Fp::from(1u64 << 32);
    limbs
        .chunks(U32_LIMBS_PER_FIELD)
        .map(|chunk| {
            let mut coeff = Fp::ONE;
            let mut acc = Fp::ZERO;
            for limb in chunk {
                acc += Fp::from(*limb as u64) * coeff;
                coeff *= radix;
            }
            acc
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use halo2_proofs::{arithmetic::Field, pasta::Fp};

    use super::{pack_u32_limbs_to_fp, packed_u32_field_len};

    #[test]
    fn packing_reports_expected_field_count() {
        assert_eq!(packed_u32_field_len(0), 0);
        assert_eq!(packed_u32_field_len(1), 1);
        assert_eq!(packed_u32_field_len(7), 1);
        assert_eq!(packed_u32_field_len(8), 2);
        assert_eq!(packed_u32_field_len(512), 74);
    }

    #[test]
    fn packing_combines_up_to_seven_u32_limbs_per_field() {
        let packed = pack_u32_limbs_to_fp(&[1, 2, 3, 4, 5, 6, 7, 9]);
        assert_eq!(packed.len(), 2);

        let radix = Fp::from(1u64 << 32);
        let mut coeff = Fp::ONE;
        let mut expected_first = Fp::ZERO;
        for limb in [1u32, 2, 3, 4, 5, 6, 7] {
            expected_first += Fp::from(limb as u64) * coeff;
            coeff *= radix;
        }

        assert_eq!(packed[0], expected_first);
        assert_eq!(packed[1], Fp::from(9));
    }

    #[test]
    fn packing_is_little_endian_and_zero_pads_tail_deterministically() {
        let packed = pack_u32_limbs_to_fp(&[5, 6, 7]);
        assert_eq!(packed.len(), 1);

        let radix = Fp::from(1u64 << 32);
        let expected = Fp::from(5)
            + Fp::from(6) * radix
            + Fp::from(7) * radix.square();

        assert_eq!(packed[0], expected);
    }
}
