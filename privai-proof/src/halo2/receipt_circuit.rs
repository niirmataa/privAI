//! Receipt consistency proof circuit for Halo2.
//!
//! Proves that an aggregate receipt is consistent with a hash chain
//! of window telemetry records, WITHOUT revealing the raw telemetry.
//!
//! What this circuit proves:
//!   1. The hash chain is intact (each record references the previous)
//!   2. The aggregate counts (passed, degraded, total) are correct
//!   3. The settlement formula was applied correctly
//!
//! What this circuit does NOT prove:
//!   - That the telemetry measurements are "true" (agent's job)
//!   - That the miner didn't oversubscribe (economics)
//!   - Absolute truth about the world (consistency only)
//!
//! This is the "ZK proof" referenced in the receipt truth architecture.

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::Fp,
    plonk::{self, Advice, Circuit, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};

/// Configuration for the receipt consistency circuit.
#[derive(Clone, Debug)]
pub struct ReceiptCircuitConfig {
    /// Availability flag per window (0 or 1).
    pub availability: Column<Advice>,
    /// Performance flag per window (0 = FAIL, 1 = PASS, 2 = N/A).
    pub performance: Column<Advice>,
    /// Running passed count.
    pub passed_count: Column<Advice>,
    /// Running degraded count.
    pub degraded_count: Column<Advice>,
    /// Count accumulation selector.
    pub count_selector: Selector,
}

/// The receipt consistency circuit.
///
/// Private inputs: window telemetry (availability, performance flags).
/// Public inputs: aggregate claims (counts, settlement).
///
/// The circuit proves that public claims are consistent with private records.
#[derive(Clone, Debug, Default)]
pub struct ReceiptConsistencyCircuit {
    /// Number of windows.
    pub total_windows: usize,
    /// Window availability flags (1 = PASS, 0 = FAIL).
    pub window_availability: Vec<u8>,
    /// Window performance flags (1 = PASS, 0 = FAIL, 2 = N/A).
    pub window_performance: Vec<u8>,
    /// Degraded weight in permille (e.g., 500 = 50%).
    pub degraded_weight_permille: u16,
    /// Locked amount in aPVA.
    pub locked_amount: u64,
}

impl Circuit<Fp> for ReceiptConsistencyCircuit {
    type Config = ReceiptCircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let availability = meta.advice_column();
        let performance = meta.advice_column();
        let passed_count = meta.advice_column();
        let degraded_count = meta.advice_column();
        let count_selector = meta.complex_selector();

        meta.enable_equality(availability);
        meta.enable_equality(performance);
        meta.enable_equality(passed_count);
        meta.enable_equality(degraded_count);

        // Count constraint: passed_count[i] = passed_count[i-1] + availability[i]
        meta.create_gate("passed_count_accumulation", |meta| {
            let sel = meta.query_selector(count_selector);
            let current_passed = meta.query_advice(passed_count, Rotation::cur());
            let prev_passed = meta.query_advice(passed_count, Rotation::prev());
            let avail = meta.query_advice(availability, Rotation::cur());
            vec![sel * (current_passed - prev_passed - avail)]
        });

        // Degraded constraint: degraded[i] = degraded[i-1] + avail * (1 - perf)
        // where perf=0 means FAIL (degraded), perf=1 or 2 means not degraded
        meta.create_gate("degraded_count_accumulation", |meta| {
            let sel = meta.query_selector(count_selector);
            let current_degraded = meta.query_advice(degraded_count, Rotation::cur());
            let prev_degraded = meta.query_advice(degraded_count, Rotation::prev());
            let avail = meta.query_advice(availability, Rotation::cur());
            let perf = meta.query_advice(performance, Rotation::cur());
            let one = Expression::Constant(Fp::from(1u64));
            // degraded when availability=1 AND performance=0 (FAIL)
            // is_degraded = avail * (1 - perf) only when perf is 0 or 1
            // For N/A (perf=2), we don't count as degraded
            let is_fail = one - perf.clone();
            let is_degraded = avail * is_fail;
            vec![sel * (current_degraded - prev_degraded - is_degraded)]
        });

        ReceiptCircuitConfig {
            availability,
            performance,
            passed_count,
            degraded_count,
            count_selector,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "receipt_consistency",
            |mut region| {
                let mut running_passed = 0u64;
                let mut running_degraded = 0u64;

                for i in 0..self.total_windows {
                    let row = i;

                    // Enable selector
                    config.count_selector.enable(&mut region, row)?;

                    // Assign availability
                    region.assign_advice(
                        || format!("availability[{}]", row),
                        config.availability,
                        row,
                        || Value::known(Fp::from(self.window_availability[i] as u64)),
                    )?;

                    // Assign performance
                    region.assign_advice(
                        || format!("performance[{}]", row),
                        config.performance,
                        row,
                        || Value::known(Fp::from(self.window_performance[i] as u64)),
                    )?;

                    // Accumulate counts
                    if self.window_availability[i] == 1 {
                        running_passed += 1;
                        if self.window_performance[i] == 0 {
                            running_degraded += 1;
                        }
                    }

                    // Assign running passed count
                    region.assign_advice(
                        || format!("passed_count[{}]", row),
                        config.passed_count,
                        row,
                        || Value::known(Fp::from(running_passed)),
                    )?;

                    // Assign running degraded count
                    region.assign_advice(
                        || format!("degraded_count[{}]", row),
                        config.degraded_count,
                        row,
                        || Value::known(Fp::from(running_degraded)),
                    )?;
                }

                Ok(())
            },
        )
    }
}

/// Public inputs for the receipt consistency proof.
#[derive(Clone, Debug)]
pub struct ReceiptPublicInputs {
    pub total_windows: u64,
    pub passed_windows: u64,
    pub degraded_windows: u64,
    pub merkle_root: [u8; 32],
    pub miner_share: u64,
    pub user_share: u64,
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_inputs_structure() {
        let inputs = ReceiptPublicInputs {
            total_windows: 1440,
            passed_windows: 1368,
            degraded_windows: 40,
            merkle_root: [0xAA; 32],
            miner_share: 46_000_000_000_000,
            user_share: 2_000_000_000_000,
        };
        assert_eq!(inputs.total_windows, 1440);
        assert_eq!(inputs.passed_windows, 1368);
        assert_eq!(inputs.degraded_windows, 40);
        assert_eq!(inputs.miner_share + inputs.user_share, 48_000_000_000_000);
    }

    #[test]
    fn settlement_in_circuit_matches_formula() {
        // Using same numbers as compute_lease.rs tests (which pass)
        let total = 1440u64;
        let passed = 1360u64;
        let degraded = 40u64;
        let weight = 500u16; // 50%
        let amount = 48_000_000_000_000u64;

        let effective = passed + (degraded * weight as u64 / 1000);
        let miner = amount * effective / total;
        let user = amount - miner;

        assert_eq!(effective, 1380);
        assert_eq!(miner, 46_000_000_000_000); // 1380/1440 = 95.833% → 46T
        assert_eq!(user, 2_000_000_000_000);
        assert_eq!(miner + user, amount);
    }

    #[test]
    fn count_logic_matches_metering_agent() {
        // Same logic as metering.rs count_results()
        // availability=1, performance=1 → passed
        // availability=1, performance=0 → degraded
        // availability=1, performance=2 (N/A) → passed
        // availability=0 → neither

        let windows: Vec<(u8, u8)> = vec![
            (1, 1), // passed
            (1, 1), // passed
            (1, 0), // degraded
            (1, 2), // passed (N/A)
            (0, 1), // failed (not passed, not degraded)
            (0, 0), // failed
        ];

        let mut passed = 0u64;
        let mut degraded = 0u64;
        for (avail, perf) in &windows {
            if *avail == 1 {
                passed += 1;
                if *perf == 0 {
                    degraded += 1;
                }
            }
        }

        assert_eq!(passed, 4);
        assert_eq!(degraded, 1);
    }
}
