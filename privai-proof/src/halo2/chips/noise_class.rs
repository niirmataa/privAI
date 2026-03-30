use halo2_proofs::{
    arithmetic::Field,
    plonk::{Advice, Column, ConstraintSystem},
};

/// Scaffold config for fresh-noise bound checks.
#[derive(Clone, Debug)]
pub struct NoiseClassConfig {
    pub noise_class: Column<Advice>,
}

#[derive(Clone, Debug)]
pub struct NoiseClassChip {
    config: NoiseClassConfig,
}

impl NoiseClassChip {
    pub fn configure<F: Field>(
        _meta: &mut ConstraintSystem<F>,
        noise_class: Column<Advice>,
    ) -> NoiseClassConfig {
        NoiseClassConfig { noise_class }
    }

    pub fn new(config: NoiseClassConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &NoiseClassConfig {
        &self.config
    }
}
