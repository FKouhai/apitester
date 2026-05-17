use hdrhistogram::Histogram;
use std::time::Duration;

// Latency histogram and RPS/error counters
//
pub struct Stats {
    histogram: Histogram<u64>,
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

impl Stats {
    pub fn new() -> Self {
        let h = Histogram::<u64>::new(2).unwrap();
        Stats { histogram: h }
    }
    pub fn record(&mut self, latency: Duration) {
        self.histogram.record(latency.as_micros() as u64).unwrap();
    }
    pub fn p50(&self) -> Duration {
        Duration::from_micros(self.histogram.value_at_quantile(0.50))
    }
    pub fn p90(&self) -> Duration {
        Duration::from_micros(self.histogram.value_at_quantile(0.90))
    }
    pub fn p99(&self) -> Duration {
        Duration::from_micros(self.histogram.value_at_quantile(0.99))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_histogram_returns_zero() {
        let stats = Stats::new();
        assert_eq!(stats.p50(), Duration::ZERO);
        assert_eq!(stats.p90(), Duration::ZERO);
        assert_eq!(stats.p99(), Duration::ZERO);
    }

    #[test]
    fn records_single_value() {
        let mut stats = Stats::new();
        stats.record(Duration::from_millis(100));
        assert!(stats.p50() >= Duration::from_millis(99));
        assert!(stats.p50() <= Duration::from_millis(101));
    }

    #[test]
    fn percentiles_reflect_distribution() {
        let mut stats = Stats::new();
        for _ in 0..90 {
            stats.record(Duration::from_millis(10));
        }
        for _ in 0..9 {
            stats.record(Duration::from_millis(100));
        }
        stats.record(Duration::from_millis(1000));

        assert!(stats.p50() <= Duration::from_millis(15));
        assert!(stats.p90() <= Duration::from_millis(110));
        assert!(stats.p99() >= Duration::from_millis(100));
    }

    #[test]
    fn p99_higher_than_p50() {
        let mut stats = Stats::new();
        for ms in [10, 20, 30, 40, 50, 60, 70, 80, 90, 1000] {
            stats.record(Duration::from_millis(ms));
        }
        assert!(stats.p99() > stats.p50());
    }
}
