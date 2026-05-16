//! Retry configuration.

use std::time::Duration;

/// Jitter strategy applied on top of exponential backoff.
///
/// AWS recommends "Full" for production retry loops because it minimizes
/// thundering-herd contention. "Equal" is in between. "None" is determinist.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Jitter {
    /// `delay = base * 2^attempt`, capped at `max_delay`.
    None,
    /// `delay = capped / 2 + rand(0, capped / 2)`.
    Equal,
    /// `delay = rand(0, capped)`. AWS-recommended default.
    #[default]
    Full,
}

/// Tunables for a retry loop.
///
/// Defaults: 6 attempts, 500ms base delay, 30s cap, full jitter.
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    /// Total attempts including the first (so `1` means "no retries").
    pub max_attempts: u32,
    /// Base delay before the first retry sleep. Doubles each attempt.
    pub base_delay: Duration,
    /// Hard cap on a single sleep duration.
    pub max_delay: Duration,
    /// Jitter strategy.
    pub jitter: Jitter,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 6,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            jitter: Jitter::Full,
        }
    }
}

impl RetryConfig {
    /// Set `max_attempts` (chainable).
    pub fn with_max_attempts(mut self, n: u32) -> Self {
        self.max_attempts = n;
        self
    }
    /// Set `base_delay` (chainable).
    pub fn with_base_delay(mut self, d: Duration) -> Self {
        self.base_delay = d;
        self
    }
    /// Set `max_delay` (chainable).
    pub fn with_max_delay(mut self, d: Duration) -> Self {
        self.max_delay = d;
        self
    }
    /// Set `jitter` (chainable).
    pub fn with_jitter(mut self, j: Jitter) -> Self {
        self.jitter = j;
        self
    }

    /// Compute the sleep delay before retry `attempt_index` (0-based: 0 is
    /// the first sleep, 1 is the second, etc.).
    pub fn delay_for(&self, attempt_index: u32, rng: &mut impl rand::Rng) -> Duration {
        let base = self.base_delay.as_secs_f64();
        let cap = self.max_delay.as_secs_f64();
        let raw = (base * 2f64.powi(attempt_index as i32)).min(cap);
        let secs = match self.jitter {
            Jitter::None => raw,
            Jitter::Equal => {
                let half = raw / 2.0;
                half + rng.gen_range(0.0..=half)
            }
            Jitter::Full => rng.gen_range(0.0..=raw),
        };
        Duration::from_secs_f64(secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn no_jitter_is_deterministic() {
        let cfg = RetryConfig::default()
            .with_base_delay(Duration::from_secs(1))
            .with_max_delay(Duration::from_secs(8))
            .with_jitter(Jitter::None);
        let mut rng = StdRng::seed_from_u64(0);
        assert_eq!(cfg.delay_for(0, &mut rng), Duration::from_secs(1));
        assert_eq!(cfg.delay_for(1, &mut rng), Duration::from_secs(2));
        assert_eq!(cfg.delay_for(2, &mut rng), Duration::from_secs(4));
        assert_eq!(cfg.delay_for(3, &mut rng), Duration::from_secs(8));
        // capped
        assert_eq!(cfg.delay_for(5, &mut rng), Duration::from_secs(8));
    }

    #[test]
    fn full_jitter_stays_in_bounds() {
        let cfg = RetryConfig::default()
            .with_base_delay(Duration::from_secs(1))
            .with_max_delay(Duration::from_secs(4))
            .with_jitter(Jitter::Full);
        let mut rng = StdRng::seed_from_u64(42);
        for i in 0..6 {
            let d = cfg.delay_for(i, &mut rng);
            let capped = Duration::from_secs(1).as_secs_f64() * 2f64.powi(i as i32);
            let upper = capped.min(4.0);
            assert!(d.as_secs_f64() >= 0.0);
            assert!(d.as_secs_f64() <= upper + 1e-9);
        }
    }

    #[test]
    fn equal_jitter_is_at_least_half() {
        let cfg = RetryConfig::default()
            .with_base_delay(Duration::from_secs(2))
            .with_max_delay(Duration::from_secs(2))
            .with_jitter(Jitter::Equal);
        let mut rng = StdRng::seed_from_u64(7);
        for _ in 0..10 {
            let d = cfg.delay_for(0, &mut rng);
            assert!(d.as_secs_f64() >= 1.0 - 1e-9);
            assert!(d.as_secs_f64() <= 2.0 + 1e-9);
        }
    }
}
