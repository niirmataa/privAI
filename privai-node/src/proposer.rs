use privai_chain::Hash32;

use crate::config::ValidatorConfig;

pub fn select_proposer(
    validators: &[ValidatorConfig],
    epoch_seed_hash: &Hash32,
    round: u32,
) -> Option<Hash32> {
    if validators.is_empty() {
        return None;
    }

    let total_weight: u128 = validators.iter().map(ValidatorConfig::score).sum();
    if total_weight == 0 {
        return Some(validators[0].pk_hash);
    }

    let mut selector = u128::from(u64::from_le_bytes(epoch_seed_hash[..8].try_into().ok()?));
    
    // INFO/TODO: Modulo z użyciem wag może nieść lekki bias dla specyficznie różniących się wariantów wag.
    // Dla v0 (scaffold/MVP) dystrybucja wag jest odpowiednio akceptowalna. W następnych iteracjach 
    // należy zaimplementować dokładniejszą pseudo-losowość (np. rejection sampling / Hash do Curve).
    selector = (selector + round as u128) % total_weight;

    let mut cumulative = 0_u128;
    for validator in validators {
        cumulative += validator.score();
        if selector < cumulative {
            return Some(validator.pk_hash);
        }
    }

    validators.last().map(|validator| validator.pk_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposer_selection_prefers_weighted_set() {
        let validators = vec![
            ValidatorConfig {
                pk_hash: [1; 32],
                stake_weight: 10,
                availability: 1,
                proof_score: 1,
            },
            ValidatorConfig {
                pk_hash: [2; 32],
                stake_weight: 90,
                availability: 1,
                proof_score: 1,
            },
        ];

        let proposer = select_proposer(&validators, &[0; 32], 50).expect("proposer");
        assert_eq!(proposer, [2; 32]);
    }
}
