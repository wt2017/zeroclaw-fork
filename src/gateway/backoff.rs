//! Exponential backoff for channel restart and retry logic.

use std::time::Duration;

/// Exponential backoff with configurable bounds and jitter.
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    /// Current attempt number (0-indexed)
    attempt: u32,
    /// Base delay for the first retry
    base: Duration,
    /// Maximum delay cap
    max: Duration,
    /// Multiplier per attempt
    factor: f64,
}

impl ExponentialBackoff {
    /// Create a new backoff starting at `base` delay, capped at `max`.
    pub fn new(base: Duration, max: Duration) -> Self {
        Self {
            attempt: 0,
            base,
            max,
            factor: 2.0,
        }
    }

    /// Create with a custom multiplier factor.
    pub fn with_factor(mut self, factor: f64) -> Self {
        self.factor = factor.max(1.0);
        self
    }

    /// Get the next delay and advance the attempt counter.
    pub fn next_delay(&mut self) -> Duration {
        let delay = self.base.mul_f64(self.factor.powi(self.attempt as i32));
        self.attempt = self.attempt.saturating_add(1);

        // Add jitter (+-25%)
        let jitter_factor = 0.75 + (pseudo_random() * 0.5);
        let jittered = delay.mul_f64(jitter_factor);

        jittered.min(self.max)
    }

    /// Reset the backoff counter (e.g., after a successful connection).
    pub fn reset(&mut self) {
        self.attempt = 0;
    }

    /// Current attempt number.
    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    /// Async sleep for the next backoff duration.
    pub async fn wait(&mut self) {
        let delay = self.next_delay();
        tokio::time::sleep(delay).await;
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new(
            Duration::from_secs(1),
            Duration::from_secs(300), // 5 minutes max
        )
    }
}

/// Simple deterministic pseudo-random for jitter (no external dep needed).
fn pseudo_random() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    f64::from(nanos % 1000) / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_backoff() {
        let mut b = ExponentialBackoff::default();
        let d1 = b.next_delay();
        let _d2 = b.next_delay();
        let d3 = b.next_delay();
        // Each delay should generally increase (with jitter)
        assert!(d1 < Duration::from_secs(5));
        assert!(d3 <= Duration::from_secs(300));
    }

    #[test]
    fn test_max_cap() {
        let mut b = ExponentialBackoff::new(Duration::from_secs(60), Duration::from_secs(120));
        for _ in 0..20 {
            let d = b.next_delay();
            assert!(d <= Duration::from_secs(120));
        }
    }

    #[test]
    fn test_reset() {
        let mut b = ExponentialBackoff::default();
        b.next_delay();
        b.next_delay();
        assert_eq!(b.attempt(), 2);
        b.reset();
        assert_eq!(b.attempt(), 0);
    }
}
