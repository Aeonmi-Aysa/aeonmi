//! Anomaly Detection — rate-limit signing ops; distort glyph on threshold breach.

use crate::glyph::gdf::GlyphParams;
use std::time::{Duration, Instant};

/// Tracks signing events and triggers glyph distortion when rate exceeds threshold.
pub struct AnomalyDetector {
    /// Timestamps of recent signing events.
    sign_times: Vec<Instant>,
    /// Max allowed signs per window before anomaly is triggered.
    sign_limit: usize,
    /// Sliding window duration.
    window: Duration,
    /// Whether an anomaly is currently active.
    is_anomalous: bool,
}

impl AnomalyDetector {
    pub fn new(sign_limit: usize, window_secs: u64) -> Self {
        Self {
            sign_times: Vec::new(),
            sign_limit,
            window: Duration::from_secs(window_secs),
            is_anomalous: false,
        }
    }

    /// Record a signing event. Returns true if this triggers an anomaly.
    pub fn record_sign(&mut self) -> bool {
        let now = Instant::now();
        // Evict events outside the window
        self.sign_times.retain(|t| now.duration_since(*t) < self.window);
        self.sign_times.push(now);

        if self.sign_times.len() > self.sign_limit {
            self.is_anomalous = true;
        }
        self.is_anomalous
    }

    pub fn is_anomalous(&self) -> bool {
        self.is_anomalous
    }

    /// Clear anomaly state after re-authentication.
    pub fn clear(&mut self) {
        self.is_anomalous = false;
        self.sign_times.clear();
    }

    /// Apply anomaly state to glyph — distorts if anomalous, harmless if not.
    pub fn apply_to_glyph(&self, glyph: &mut GlyphParams) {
        if self.is_anomalous {
            glyph.distort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_anomaly_under_limit() {
        let mut det = AnomalyDetector::new(5, 60);
        for _ in 0..5 {
            assert!(!det.record_sign(), "Should not trigger under limit");
        }
        assert!(!det.is_anomalous());
    }

    #[test]
    fn test_anomaly_over_limit() {
        let mut det = AnomalyDetector::new(3, 60);
        for _ in 0..3 {
            det.record_sign();
        }
        let result = det.record_sign(); // 4th — over limit
        assert!(result, "4th sign should trigger anomaly");
        assert!(det.is_anomalous());
    }

    #[test]
    fn test_clear_resets() {
        let mut det = AnomalyDetector::new(1, 60);
        det.record_sign();
        det.record_sign(); // triggers
        assert!(det.is_anomalous());
        det.clear();
        assert!(!det.is_anomalous(), "After clear anomaly should be gone");
    }
}
