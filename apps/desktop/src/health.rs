//! Health check system following AWS ORR patterns
//!
//! Implements readiness and liveness probes for operational monitoring.
//!
//! # Health Check Types
//!
//! - **Liveness**: Is the process running? (WebViews exist, event loop active)
//! - **Readiness**: Can handle requests? (Database accessible, metrics healthy)
//!
//! # AWS Pattern: ORR Health Checks
//!
//! Follows AWS Operational Readiness Review standards:
//! - Clear health indicators
//! - Automatic recovery detection
//! - Integration with monitoring systems

// Allow dead code temporarily - APIs will be integrated in Week 2.3 (structured logging)
#![allow(dead_code)]

use crate::metrics::Metrics;
use std::path::PathBuf;
use std::sync::Arc;
use storage::Database;
use tracing::{error, info};

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded but operational
    Degraded,
    /// Service is unhealthy
    Unhealthy,
}

impl HealthStatus {
    /// Is service operational? (Healthy or Degraded)
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }
}

/// Health check result with details
#[derive(Debug, Clone)]
pub struct HealthCheck {
    /// Overall health status
    pub status: HealthStatus,
    /// Database connectivity
    pub database_healthy: bool,
    /// Metrics system health
    pub metrics_healthy: bool,
    /// WebView availability
    pub webview_healthy: bool,
    /// Detailed message
    pub message: String,
}

impl HealthCheck {
    /// Check if service is ready to handle requests
    pub fn is_ready(&self) -> bool {
        self.status == HealthStatus::Healthy
    }

    /// Check if service is alive (process running)
    pub fn is_alive(&self) -> bool {
        self.status.is_operational()
    }
}

/// Health checker for browser application
///
/// AWS ORR pattern: Centralized health monitoring
pub struct HealthChecker {
    db_path: PathBuf,
    metrics: Arc<Metrics>,
}

impl HealthChecker {
    /// Create new health checker
    pub fn new(db_path: PathBuf, metrics: Arc<Metrics>) -> Self {
        Self { db_path, metrics }
    }

    /// Perform comprehensive health check
    ///
    /// Checks:
    /// - Database connectivity (can open connection?)
    /// - Metrics health (error rate <5%?)
    /// - Overall system status
    pub fn check_health(&self) -> HealthCheck {
        let database_healthy = self.check_database();
        let metrics_healthy = self.metrics.is_healthy();

        // Determine overall status
        let status = if database_healthy && metrics_healthy {
            HealthStatus::Healthy
        } else if database_healthy || metrics_healthy {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        let message = match status {
            HealthStatus::Healthy => "All systems operational".to_string(),
            HealthStatus::Degraded => {
                let mut issues = vec![];
                if !database_healthy {
                    issues.push("database unreachable");
                }
                if !metrics_healthy {
                    issues.push("high error rate");
                }
                format!("Degraded: {}", issues.join(", "))
            }
            HealthStatus::Unhealthy => "Multiple systems failing".to_string(),
        };

        info!("Health check: {:?} - {}", status, message);

        HealthCheck {
            status,
            database_healthy,
            metrics_healthy,
            webview_healthy: true, // Always true if process running
            message,
        }
    }

    /// Check liveness (is process alive?)
    ///
    /// Returns true if the process is running (always true when called)
    pub fn check_liveness(&self) -> bool {
        true // If we can call this, process is alive
    }

    /// Check readiness (can handle requests?)
    ///
    /// Returns true if database is accessible and metrics are healthy
    pub fn check_readiness(&self) -> bool {
        let health = self.check_health();
        health.is_ready()
    }

    /// Check database connectivity
    fn check_database(&self) -> bool {
        match Database::new(&self.db_path) {
            Ok(_) => {
                info!("✅ Database health check passed");
                true
            }
            Err(e) => {
                error!("❌ Database health check failed: {}", e);
                false
            }
        }
    }

    /// Get health check result as HTTP status code equivalent
    ///
    /// For integration with monitoring systems
    pub fn get_status_code(&self) -> u16 {
        let health = self.check_health();
        match health.status {
            HealthStatus::Healthy => 200,   // OK
            HealthStatus::Degraded => 503,  // Service Unavailable
            HealthStatus::Unhealthy => 503, // Service Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_health_check_with_healthy_database() {
        let temp_file = NamedTempFile::new().unwrap();
        let metrics = Metrics::new();

        // Initialize database
        Database::new(temp_file.path()).unwrap();

        let checker = HealthChecker::new(temp_file.path().to_path_buf(), metrics);
        let health = checker.check_health();

        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.database_healthy);
        assert!(health.metrics_healthy);
        assert!(health.is_ready());
        assert!(health.is_alive());
    }

    #[test]
    fn test_health_check_with_missing_database() {
        let metrics = Metrics::new();
        let checker = HealthChecker::new(PathBuf::from("/nonexistent/path.db"), metrics);

        let health = checker.check_health();

        assert!(!health.database_healthy);
        assert_ne!(health.status, HealthStatus::Healthy);
    }

    #[test]
    fn test_liveness_always_true() {
        let metrics = Metrics::new();
        let checker = HealthChecker::new(PathBuf::from("test.db"), metrics);

        assert!(checker.check_liveness());
    }

    #[test]
    fn test_readiness_requires_all_systems() {
        let temp_file = NamedTempFile::new().unwrap();
        let metrics = Metrics::new();
        Database::new(temp_file.path()).unwrap();

        let checker = HealthChecker::new(temp_file.path().to_path_buf(), metrics);
        assert!(checker.check_readiness());
    }

    #[test]
    fn test_status_code_mapping() {
        let temp_file = NamedTempFile::new().unwrap();
        let metrics = Metrics::new();
        Database::new(temp_file.path()).unwrap();

        let checker = HealthChecker::new(temp_file.path().to_path_buf(), metrics);
        assert_eq!(checker.get_status_code(), 200);
    }

    #[test]
    fn test_health_status_is_operational() {
        assert!(HealthStatus::Healthy.is_operational());
        assert!(HealthStatus::Degraded.is_operational());
        assert!(!HealthStatus::Unhealthy.is_operational());
    }
}
