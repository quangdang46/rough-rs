const LEGACY_MULTIPLIER: i32 = 48_271;
const LEGACY_MASK: u32 = 0x7fff_ffff;
const LEGACY_DENOMINATOR: f64 = 2_147_483_648.0;

#[derive(Debug, Clone)]
pub struct RngHelper {
    seed: i32,
}

impl RngHelper {
    pub fn new(seed: u64) -> Self {
        Self { seed: seed as i32 }
    }

    pub fn next_f64(&mut self) -> f64 {
        if self.seed == 0 {
            return random_unit();
        }

        self.seed = self.seed.wrapping_mul(LEGACY_MULTIPLIER);
        ((self.seed as u32) & LEGACY_MASK) as f64 / LEGACY_DENOMINATOR
    }

    pub fn offset(&mut self, min: f64, max: f64, roughness: f64, roughness_gain: f64) -> f64 {
        roughness * roughness_gain * (self.next_f64() * (max - min) + min)
    }

    pub fn offset_symmetric(&mut self, x: f64, roughness: f64, roughness_gain: f64) -> f64 {
        self.offset(-x, x, roughness, roughness_gain)
    }
}

pub fn random_seed() -> u64 {
    #[cfg(feature = "rand")]
    {
        use rand::Rng;
        rand::thread_rng().gen_range(0..(1_u64 << 31))
    }

    #[cfg(not(feature = "rand"))]
    {
        0
    }
}

pub(crate) fn random_unit() -> f64 {
    #[cfg(feature = "rand")]
    {
        use rand::Rng;
        rand::thread_rng().gen_range(0.0..1.0)
    }

    #[cfg(not(feature = "rand"))]
    {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use serde_json::Value;

    #[test]
    fn seeded_rng_matches_legacy_roughjs_fixture() {
        let fixture: Value =
            serde_json::from_str(include_str!("../tests/fixtures/reference.json")).unwrap();

        for seed in [1_u64, 42, 12345, 2_147_483_647] {
            let mut rng = RngHelper::new(seed);
            let expected = fixture["rng"][seed.to_string()]
                .as_array()
                .expect("fixture seed should exist");

            for value in expected {
                assert_relative_eq!(
                    rng.next_f64(),
                    value.as_f64().expect("fixture value should be numeric"),
                    epsilon = 1e-15
                );
            }
        }
    }

    #[test]
    fn random_offsets_respect_expected_range() {
        let mut rng = RngHelper::new(42);

        for _ in 0..100 {
            let value = rng.offset_symmetric(3.0, 2.0, 0.5);
            assert!((-3.0..=3.0).contains(&value));
        }
    }

    #[test]
    fn seed_zero_uses_nondeterministic_fallback_range() {
        let mut rng = RngHelper::new(0);

        for _ in 0..10 {
            let value = rng.next_f64();
            assert!((0.0..1.0).contains(&value));
        }
    }

    #[test]
    fn random_seed_matches_roughjs_integer_range() {
        for _ in 0..10 {
            assert!(random_seed() < (1_u64 << 31));
        }
    }
}
