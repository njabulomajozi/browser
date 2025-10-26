//! Metrics system for operational excellence
//!
//! Implements DORA metrics tracking and service health monitoring
//! following AWS operational excellence patterns.
//!
//! # Metrics Tracked
//!
//! - **Navigation success rate**: Change failure rate analog
//! - **Page load time**: p50, p95, p99 percentiles
//! - **Error rate**: Last 5 minutes
//! - **MTTR**: Mean time to recovery (time between errors)

// Allow dead code temporarily - APIs will be integrated in Week 2
#![allow(dead_code)]

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use tracing::info;

/// Metrics snapshot for reporting
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_navigations: u64,
    pub failed_navigations: u64,
    pub error_rate: f64,
    pub avg_load_time_ms: f64,
    pub p95_load_time_ms: f64,
    pub p99_load_time_ms: f64,
    pub last_error: Option<String>,
    pub mttr_seconds: f64,
}

/// Metrics collector following AWS operational excellence patterns
pub struct Metrics {
    // DORA metrics
    navigation_count: AtomicU64,
    navigation_errors: AtomicU64,

    // Performance metrics
    page_load_times: Mutex<Vec<Duration>>,

    // Error tracking for COE/MTTR
    last_error: Mutex<Option<(Instant, String)>>,
    last_recovery: Mutex<Option<Instant>>,
}

impl Metrics {
    /// Create new metrics collector
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            navigation_count: AtomicU64::new(0),
            navigation_errors: AtomicU64::new(0),
            page_load_times: Mutex::new(Vec::new()),
            last_error: Mutex::new(None),
            last_recovery: Mutex::new(None),
        })
    }

    /// Record a navigation attempt
    ///
    /// # Arguments
    /// * `success` - Whether navigation succeeded
    /// * `duration` - Time taken for navigation
    pub fn record_navigation(&self, success: bool, duration: Duration) {
        self.navigation_count.fetch_add(1, Ordering::Relaxed);

        if success {
            // Record successful load time
            let mut times = self.page_load_times.lock().unwrap();
            times.push(duration);

            // Keep only last 1000 samples for memory efficiency
            let len = times.len();
            if len > 1000 {
                times.drain(0..len - 1000);
            }

            // Mark recovery from previous error
            *self.last_recovery.lock().unwrap() = Some(Instant::now());
        } else {
            self.navigation_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record an error for COE analysis
    pub fn record_error(&self, error: &str) {
        *self.last_error.lock().unwrap() = Some((Instant::now(), error.to_string()));
        self.navigation_errors.fetch_add(1, Ordering::Relaxed);
        info!("[METRICS] Error recorded: {}", error);
    }

    /// Get error rate (DORA: change failure rate analog)
    ///
    /// Returns ratio of failed navigations to total navigations
    pub fn get_error_rate(&self) -> f64 {
        let total = self.navigation_count.load(Ordering::Relaxed);
        let errors = self.navigation_errors.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            (errors as f64) / (total as f64)
        }
    }

    /// Get mean time to recovery (MTTR) in seconds
    ///
    /// Time between last error and last recovery
    pub fn get_mttr(&self) -> Duration {
        let last_error = self.last_error.lock().unwrap().clone();
        let last_recovery = *self.last_recovery.lock().unwrap();

        match (last_error, last_recovery) {
            (Some((error_time, _)), Some(recovery_time)) => {
                if recovery_time > error_time {
                    recovery_time.duration_since(error_time)
                } else {
                    Duration::ZERO
                }
            }
            _ => Duration::ZERO,
        }
    }

    /// Check if service is healthy
    ///
    /// Health criteria:
    /// - Error rate < 5% (last 100 navigations)
    /// - No errors in last 5 minutes
    pub fn is_healthy(&self) -> bool {
        let error_rate = self.get_error_rate();

        // Check error rate threshold
        if error_rate > 0.05 {
            return false;
        }

        // Check last error time
        if let Some((error_time, _)) = *self.last_error.lock().unwrap() {
            if error_time.elapsed() < Duration::from_secs(300) {
                return false;
            }
        }

        true
    }

    /// Get complete metrics snapshot
    pub fn get_stats(&self) -> MetricsSnapshot {
        let total = self.navigation_count.load(Ordering::Relaxed);
        let errors = self.navigation_errors.load(Ordering::Relaxed);
        let error_rate = self.get_error_rate();

        let times = self.page_load_times.lock().unwrap();

        let (avg_ms, p95_ms, p99_ms) = if times.is_empty() {
            (0.0, 0.0, 0.0)
        } else {
            let avg = times.iter().sum::<Duration>().as_millis() as f64 / times.len() as f64;

            let mut sorted = times.clone();
            sorted.sort();

            let p95_idx = (sorted.len() as f64 * 0.95) as usize;
            let p99_idx = (sorted.len() as f64 * 0.99) as usize;

            let p95 = sorted.get(p95_idx).unwrap_or(&Duration::ZERO).as_millis() as f64;
            let p99 = sorted.get(p99_idx).unwrap_or(&Duration::ZERO).as_millis() as f64;

            (avg, p95, p99)
        };

        let last_error = self
            .last_error
            .lock()
            .unwrap()
            .as_ref()
            .map(|(_, err)| err.clone());

        let mttr = self.get_mttr().as_secs_f64();

        MetricsSnapshot {
            total_navigations: total,
            failed_navigations: errors,
            error_rate,
            avg_load_time_ms: avg_ms,
            p95_load_time_ms: p95_ms,
            p99_load_time_ms: p99_ms,
            last_error,
            mttr_seconds: mttr,
        }
    }

    /// Print metrics summary to logs
    pub fn log_summary(&self) {
        let stats = self.get_stats();

        info!("[METRICS] Summary:");
        info!("  Total navigations: {}", stats.total_navigations);
        info!("  Failed navigations: {}", stats.failed_navigations);
        info!("  Error rate: {:.2}%", stats.error_rate * 100.0);
        info!("  Avg load time: {:.2}ms", stats.avg_load_time_ms);
        info!("  P95 load time: {:.2}ms", stats.p95_load_time_ms);
        info!("  P99 load time: {:.2}ms", stats.p99_load_time_ms);
        info!("  MTTR: {:.2}s", stats.mttr_seconds);
        info!("  Healthy: {}", self.is_healthy());
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            navigation_count: AtomicU64::new(0),
            navigation_errors: AtomicU64::new(0),
            page_load_times: Mutex::new(Vec::new()),
            last_error: Mutex::new(None),
            last_recovery: Mutex::new(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        let metrics = Metrics::new();
        let stats = metrics.get_stats();

        assert_eq!(stats.total_navigations, 0);
        assert_eq!(stats.failed_navigations, 0);
        assert_eq!(stats.error_rate, 0.0);
        assert!(metrics.is_healthy());
    }

    #[test]
    fn test_record_successful_navigation() {
        let metrics = Metrics::new();

        metrics.record_navigation(true, Duration::from_millis(100));
        metrics.record_navigation(true, Duration::from_millis(150));
        metrics.record_navigation(true, Duration::from_millis(200));

        let stats = metrics.get_stats();
        assert_eq!(stats.total_navigations, 3);
        assert_eq!(stats.failed_navigations, 0);
        assert_eq!(stats.error_rate, 0.0);
        assert!(stats.avg_load_time_ms > 0.0);
    }

    #[test]
    fn test_record_failed_navigation() {
        let metrics = Metrics::new();

        metrics.record_navigation(true, Duration::from_millis(100));
        metrics.record_navigation(false, Duration::ZERO);
        metrics.record_navigation(true, Duration::from_millis(100));

        let stats = metrics.get_stats();
        assert_eq!(stats.total_navigations, 3);
        assert_eq!(stats.failed_navigations, 1);
        assert!((stats.error_rate - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_error_rate_calculation() {
        let metrics = Metrics::new();

        // 20% error rate (1/5)
        metrics.record_navigation(true, Duration::from_millis(100));
        metrics.record_navigation(true, Duration::from_millis(100));
        metrics.record_navigation(false, Duration::ZERO);
        metrics.record_navigation(true, Duration::from_millis(100));
        metrics.record_navigation(true, Duration::from_millis(100));

        let error_rate = metrics.get_error_rate();
        assert!((error_rate - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_health_check() {
        let metrics = Metrics::new();

        // Healthy: low error rate
        for _ in 0..100 {
            metrics.record_navigation(true, Duration::from_millis(100));
        }
        assert!(metrics.is_healthy());

        // Unhealthy: high error rate
        for _ in 0..10 {
            metrics.record_navigation(false, Duration::ZERO);
        }
        assert!(!metrics.is_healthy());
    }

    #[test]
    fn test_percentile_calculation() {
        let metrics = Metrics::new();

        // Add 100 samples: 0ms, 1ms, 2ms, ..., 99ms
        for i in 0..100 {
            metrics.record_navigation(true, Duration::from_millis(i));
        }

        let stats = metrics.get_stats();
        assert!(stats.p95_load_time_ms >= 90.0);
        assert!(stats.p99_load_time_ms >= 95.0);
    }

    #[test]
    fn test_record_error() {
        let metrics = Metrics::new();

        metrics.record_error("Test error");

        let stats = metrics.get_stats();
        assert_eq!(stats.last_error, Some("Test error".to_string()));
        assert_eq!(stats.failed_navigations, 1);
    }
}
