use serde::Serialize;

/// Wall-clock PLC execution cost, deliberately separate from deterministic snapshots and replay fingerprints.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ScanTiming {
    pub scans: u64,
    pub last_us: f64,
    pub average_us: f64,
    pub maximum_us: f64,
    pub interval_ms: f64,
}

impl ScanTiming {
    pub fn record(&mut self, elapsed: std::time::Duration) {
        let us = elapsed.as_secs_f64() * 1_000_000.0;
        self.scans += 1;
        self.last_us = us;
        self.average_us += (us - self.average_us) / self.scans as f64;
        self.maximum_us = self.maximum_us.max(us);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn timing_reports_last_mean_and_worst_execution() {
        let mut timing = ScanTiming::default();
        for us in [40, 120, 20] {
            timing.record(Duration::from_micros(us));
        }
        assert_eq!(timing.scans, 3);
        assert!((timing.last_us - 20.0).abs() < 1e-8);
        assert!((timing.average_us - 60.0).abs() < 1e-8);
        assert!((timing.maximum_us - 120.0).abs() < 1e-8);
    }
}
