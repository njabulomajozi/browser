//! Navigation Service - Decoupled URL loading and history management
//!
//! Following AWS service-oriented architecture:
//! - **Loose Coupling**: Independent of WebView implementation
//! - **Single Responsibility**: Navigation logic only
//! - **Metrics Integration**: Tracks navigation success/failure for DORA
//!
//! # Responsibilities
//!
//! - URL validation and loading
//! - History persistence (database)
//! - Back/forward navigation via JavaScript
//! - Page reload and stop
//! - Metrics tracking for operational excellence

// Allow dead code temporarily - APIs will be integrated in Week 2
#![allow(dead_code)]

use crate::error::{BrowserError, Result};
use crate::metrics::Metrics;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use storage::Database;
use tracing::{info, warn};
use wry::WebView;

/// Navigation result returned after URL load
#[derive(Debug, Clone)]
pub struct NavigationResult {
    /// Final URL after redirects
    pub url: String,

    /// Page title (extracted from URL domain as fallback)
    pub title: String,

    /// Whether to add to history
    pub should_add_history: bool,
}

/// Navigation Service following AWS service patterns
///
/// Manages URL navigation with:
/// - Metrics tracking (DORA: navigation success rate)
/// - History persistence
/// - Error handling with COE reports
pub struct NavigationService {
    /// Database path for history
    db_path: PathBuf,

    /// Current URL (if any)
    current_url: Option<String>,

    /// Metrics collector
    metrics: Arc<Metrics>,
}

impl NavigationService {
    /// Create new navigation service
    ///
    /// # Arguments
    /// * `db_path` - Path to SQLite database
    /// * `metrics` - Metrics collector for DORA tracking
    pub fn new(db_path: PathBuf, metrics: Arc<Metrics>) -> Self {
        Self {
            db_path,
            current_url: None,
            metrics,
        }
    }

    /// Navigate to URL
    ///
    /// # Arguments
    /// * `url` - URL to navigate to
    ///
    /// # Returns
    /// NavigationResult with processed URL and title
    ///
    /// # Metrics
    /// Records navigation attempt (success/failure) with duration
    pub fn navigate(&mut self, url: &str) -> Result<NavigationResult> {
        let start = Instant::now();

        info!("Navigating to: {}", url);

        // Extract title from URL (domain as fallback)
        let title = url.split('/').nth(2).unwrap_or("New Tab").to_string();

        // Save to history
        match self.save_to_history(url, Some(&title)) {
            Ok(_) => {
                let duration = start.elapsed();
                self.metrics.record_navigation(true, duration);
                info!("✅ Navigation successful: {} ({:?})", url, duration);
            }
            Err(e) => {
                self.metrics
                    .record_error(&format!("History save failed: {}", e));
                warn!("Failed to save history: {}", e);
                // Continue navigation even if history save fails
            }
        }

        self.current_url = Some(url.to_string());

        Ok(NavigationResult {
            url: url.to_string(),
            title,
            should_add_history: true,
        })
    }

    /// Go back in history (via JavaScript)
    ///
    /// # Arguments
    /// * `webview` - Content WebView to execute back navigation
    pub fn go_back(&self, webview: &WebView) -> Result<()> {
        info!("Go back");
        webview
            .evaluate_script("window.history.back()")
            .map_err(|e| BrowserError::NavigationFailed {
                url: self.current_url.clone().unwrap_or_default(),
                reason: format!("Go back failed: {}", e),
            })?;
        Ok(())
    }

    /// Go forward in history (via JavaScript)
    ///
    /// # Arguments
    /// * `webview` - Content WebView to execute forward navigation
    pub fn go_forward(&self, webview: &WebView) -> Result<()> {
        info!("Go forward");
        webview
            .evaluate_script("window.history.forward()")
            .map_err(|e| BrowserError::NavigationFailed {
                url: self.current_url.clone().unwrap_or_default(),
                reason: format!("Go forward failed: {}", e),
            })?;
        Ok(())
    }

    /// Reload current page (via JavaScript)
    ///
    /// # Arguments
    /// * `webview` - Content WebView to reload
    pub fn reload(&self, webview: &WebView) -> Result<()> {
        info!("Reload");
        webview
            .evaluate_script("window.location.reload()")
            .map_err(|e| BrowserError::NavigationFailed {
                url: self.current_url.clone().unwrap_or_default(),
                reason: format!("Reload failed: {}", e),
            })?;
        Ok(())
    }

    /// Stop page loading (via JavaScript)
    ///
    /// # Arguments
    /// * `webview` - Content WebView to stop
    pub fn stop(&self, webview: &WebView) -> Result<()> {
        info!("Stop loading");
        webview
            .evaluate_script("window.stop()")
            .map_err(|e| BrowserError::NavigationFailed {
                url: self.current_url.clone().unwrap_or_default(),
                reason: format!("Stop failed: {}", e),
            })?;
        Ok(())
    }

    /// Get current URL
    pub fn current_url(&self) -> Option<&str> {
        self.current_url.as_deref()
    }

    /// Save URL to history database
    ///
    /// # Arguments
    /// * `url` - URL to save
    /// * `title` - Page title (optional)
    ///
    /// # Errors
    /// Returns `BrowserError::Database` if save fails
    fn save_to_history(&self, url: &str, title: Option<&str>) -> Result<()> {
        let db = Database::new(&self.db_path).map_err(|e| BrowserError::Database(e.to_string()))?;

        db.add_history(url, title)
            .map_err(|e| BrowserError::Database(e.to_string()))?;

        info!("✅ Saved to history: {} - {:?}", url, title);
        Ok(())
    }

    /// Get reference to metrics
    pub fn metrics(&self) -> &Arc<Metrics> {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_navigation_service_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let metrics = Metrics::new();
        let service = NavigationService::new(temp_file.path().to_path_buf(), metrics);

        assert!(service.current_url().is_none());
    }

    #[test]
    fn test_navigate() {
        let temp_file = NamedTempFile::new().unwrap();
        let metrics = Metrics::new();
        let mut service = NavigationService::new(temp_file.path().to_path_buf(), metrics);

        let result = service.navigate("https://example.com").unwrap();

        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.title, "example.com");
        assert!(result.should_add_history);
        assert_eq!(service.current_url(), Some("https://example.com"));
    }

    #[test]
    fn test_title_extraction() {
        let temp_file = NamedTempFile::new().unwrap();
        let metrics = Metrics::new();
        let mut service = NavigationService::new(temp_file.path().to_path_buf(), metrics);

        let result = service
            .navigate("https://www.github.com/user/repo")
            .unwrap();
        assert_eq!(result.title, "www.github.com");

        let result = service.navigate("http://localhost:3000").unwrap();
        assert_eq!(result.title, "localhost:3000");
    }

    #[test]
    fn test_history_persistence() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        {
            let metrics = Metrics::new();
            let mut service = NavigationService::new(temp_path.clone(), metrics);
            service.navigate("https://example.com").unwrap();
            service.navigate("https://github.com").unwrap();
        }

        // Verify history persisted
        let db = Database::new(&temp_path).unwrap();
        let history = db.get_recent_history(10).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].url, "https://github.com");
        assert_eq!(history[1].url, "https://example.com");
    }

    #[test]
    fn test_metrics_tracking() {
        let temp_file = NamedTempFile::new().unwrap();
        let metrics = Metrics::new();
        let mut service = NavigationService::new(temp_file.path().to_path_buf(), metrics.clone());

        service.navigate("https://example.com").unwrap();
        service.navigate("https://github.com").unwrap();

        let stats = metrics.get_stats();
        assert_eq!(stats.total_navigations, 2);
        assert_eq!(stats.failed_navigations, 0);
    }

    // Note: go_back/forward/reload/stop tests require WebView instance
    // These would be integration tests in tests/integration/
}
