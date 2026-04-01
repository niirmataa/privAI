use privai_chain::Hash32;

use crate::config::ValidatorConfig;

/// Oblicza deterministyczny selector z seed + round + attempt.
/// Używamy mnożenia z liczbą pierwszą (Fibonacci hashing / splitmix64)
/// dla dobrej dystrybucji i dużej wrażliwości na zmiany rundy.
fn derive_selector(seed: &Hash32, attempt: u32, round: u32) -> Option<u64> {
    let seed_lo = u64::from_le_bytes(seed[..8].try_into().ok()?);
    let seed_hi = u64::from_le_bytes(seed[8..16].try_into().ok()?);

    const GOLDEN: u64 = 0x9E3779B97F4A7C15;

    // Mieszaj seed, round i attempt
    let mut h = seed_lo ^ ((round as u64).wrapping_mul(GOLDEN));
    h ^= (attempt as u64).wrapping_mul(0x517cc1b727220a95);
    h ^= seed_hi.rotate_left((round % 32) as u32);
    // splitmix64 finalize
    h = h.wrapping_mul(0x94d049bb133111eb);
    h ^= h >> 32;
    h = h.wrapping_mul(0x94d049bb133111eb);
    h ^= h >> 32;

    Some(h)
}

/// Wybiera proposera używając rejection sampling dla jednorodnej dystrybucji.
///
/// Algorytm:
/// 1. Generuj selector = derive_selector(seed, attempt, round) % total_weight
/// 2. Użyj selector do wyboru walidatora proporcjonalnie do wagi
///
/// Używamy mnożnika (selector * total_weight / u64::MAX) zamiast prostego modulo
/// aby zminimalizować bias — to standardowa technika rejection sampling.
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

    // Próbujemy kilka attemptów z rejection sampling
    // Akceptujemy selector tylko jeśli jest w zakresie [0, limit) gdzie limit
    // jest największą wielokrotnością total_weight mniejszą od u64::MAX.
    // To eliminuje bias modulo.
    let max_val = u64::MAX;
    let limit = (max_val / total_weight as u64) * total_weight as u64;

    for attempt in 0..16u32 {
        if let Some(raw_selector) = derive_selector(epoch_seed_hash, attempt, round) {
            if raw_selector < limit {
                // Zaakceptowano — zero bias
                let selector = (raw_selector as u128) % total_weight;
                let mut cumulative = 0_u128;
                for validator in validators {
                    cumulative += validator.score();
                    if selector < cumulative {
                        return Some(validator.pk_hash);
                    }
                }
                return validators.last().map(|v| v.pk_hash);
            }
            // raw_selector >= limit → odrzuć, próbuj ponownie
        }
    }

    // Fallback: użyj wyniku mimo potential bias (bardzo rzadkie przy 16 próbach)
    if let Some(raw) = derive_selector(epoch_seed_hash, 0, round) {
        let selector = (raw as u128) % total_weight;
        let mut cumulative = 0_u128;
        for validator in validators {
            cumulative += validator.score();
            if selector < cumulative {
                return Some(validator.pk_hash);
            }
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

        // Weryfikujemy weighted property na wielu seed'ach:
        // Validator 2 (weight 90) powinien być wybierany ~90% czasu.
        let mut count_v1 = 0u32;
        let mut count_v2 = 0u32;
        let rounds = 1000u32;

        for round in 0..rounds {
            let mut seed = [0u8; 32];
            seed[..4].copy_from_slice(&round.to_le_bytes());
            if let Some(proposer) = select_proposer(&validators, &seed, round) {
                if proposer == [1; 32] {
                    count_v1 += 1;
                } else if proposer == [2; 32] {
                    count_v2 += 1;
                }
            }
        }

        // Validator 2 (90% wagi) powinien dominować — tolerancja ±10%
        assert!(
            count_v2 > 800,
            "Validator 2 (weight 90) should be selected most of the time, got {}/{}",
            count_v2,
            rounds
        );
        assert!(
            count_v1 < 200,
            "Validator 1 (weight 10) should be selected rarely, got {}/{}",
            count_v1,
            rounds
        );
    }

    #[test]
    fn proposer_selection_is_deterministic() {
        let validators = vec![
            ValidatorConfig {
                pk_hash: [1; 32],
                stake_weight: 50,
                availability: 1,
                proof_score: 1,
            },
            ValidatorConfig {
                pk_hash: [2; 32],
                stake_weight: 50,
                availability: 1,
                proof_score: 1,
            },
        ];

        let seed = [42; 32];
        // Ten sam seed + round powinien zawsze dawać ten sam wynik
        let p1 = select_proposer(&validators, &seed, 7);
        let p2 = select_proposer(&validators, &seed, 7);
        assert_eq!(p1, p2, "Selection must be deterministic for same inputs");
    }

    #[test]
    fn proposer_selection_returns_none_on_empty() {
        assert_eq!(select_proposer(&[], &[0; 32], 0), None);
    }

    #[test]
    fn proposer_selection_round_changes_proposer() {
        let validators = vec![
            ValidatorConfig {
                pk_hash: [1; 32],
                stake_weight: 50,
                availability: 1,
                proof_score: 1,
            },
            ValidatorConfig {
                pk_hash: [2; 32],
                stake_weight: 50,
                availability: 1,
                proof_score: 1,
            },
        ];

        // Sprawdzamy na wielu seed'ach czy round wpływa na wybór proposera
        let mut seeds_with_variation = 0u32;
        let num_seeds = 50u32;

        for seed_base in 0..num_seeds {
            let mut seed = [0u8; 32];
            seed[..4].copy_from_slice(&seed_base.to_le_bytes());

            let p0 = select_proposer(&validators, &seed, 0).expect("proposer");
            let mut found_diff = false;

            for round in 1..30 {
                let p = select_proposer(&validators, &seed, round).expect("proposer");
                if p != p0 {
                    found_diff = true;
                    break;
                }
            }

            if found_diff {
                seeds_with_variation += 1;
            }
        }

        // Większość seed'ów powinna dać różnego proposera przy zmianie rundy
        assert!(
            seeds_with_variation > num_seeds / 2,
            "Most seeds should produce different proposers across rounds, got {}/{}",
            seeds_with_variation,
            num_seeds
        );
    }
}
