//! Typed error system for COE (Correction of Errors) analysis
//!
//! Following AWS operational excellence patterns, errors are:
//! - **Typed**: Clear categorization for metrics tracking
//! - **Contextual**: Include URL, operation, stack traces
//! - **Actionable**: Suggest remediation paths
//!
//! This enables post-incident COE analysis and error rate tracking.

use chrono::{DateTime, Utc};
use std::fmt;
use thiserror::Error;

/// Browser application errors with COE context
#[derive(Debug, Error)]
pub enum BrowserError {
    /// WebView creation failed
    #[error("WebView creation failed: {0}")]
    WebViewCreation(String),

    /// Navigation to URL failed
    #[error("Navigation failed: url={url}, reason={reason}")]
    NavigationFailed { url: String, reason: String },

    /// Database operation failed
    #[error("Database error: {0}")]
    Database(String),

    /// IPC message handling failed
    #[error("IPC error: {0}")]
    IpcError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Window/UI error
    #[error("Window error: {0}")]
    WindowError(String),

    /// Generic error for unexpected cases
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl BrowserError {
    /// Get error type as string (for metrics categorization)
    pub fn error_type(&self) -> &'static str {
        match self {
            Self::WebViewCreation(_) => "webview_creation",
            Self::NavigationFailed { .. } => "navigation_failed",
            Self::Database(_) => "database",
            Self::IpcError(_) => "ipc",
            Self::ConfigError(_) => "config",
            Self::WindowError(_) => "window",
            Self::Unexpected(_) => "unexpected",
        }
    }

    /// Get user-facing error message
    pub fn user_message(&self) -> String {
        match self {
            Self::WebViewCreation(_) => {
                "Failed to create browser window. Please restart the application.".to_string()
            }
            Self::NavigationFailed { url, .. } => {
                format!(
                    "Failed to load {}. Please check your internet connection.",
                    url
                )
            }
            Self::Database(_) => {
                "Browser data error. Your history and bookmarks may not be saved.".to_string()
            }
            Self::IpcError(_) => {
                "Internal communication error. Please reload the page.".to_string()
            }
            Self::ConfigError(_) => "Configuration error. Please check your settings.".to_string(),
            Self::WindowError(_) => "Window error. Please restart the application.".to_string(),
            Self::Unexpected(_) => "An unexpected error occurred. Please try again.".to_string(),
        }
    }

    /// Get suggested remediation (for COE reports)
    pub fn suggested_fix(&self) -> &'static str {
        match self {
            Self::WebViewCreation(_) => {
                "Check platform WebView availability (WKWebView/WebView2/WebKitGTK)"
            }
            Self::NavigationFailed { .. } => "Verify URL validity, check network connectivity",
            Self::Database(_) => "Check database file permissions, verify disk space",
            Self::IpcError(_) => "Investigate IPC message format, check serialization",
            Self::ConfigError(_) => "Validate configuration file, reset to defaults",
            Self::WindowError(_) => "Check windowing system, verify graphics drivers",
            Self::Unexpected(_) => "Enable debug logging, capture full stack trace",
        }
    }

    /// Generate COE (Correction of Errors) report
    ///
    /// AWS pattern for post-incident analysis
    pub fn to_coe_report(&self) -> ErrorReport {
        ErrorReport {
            error_type: self.error_type().to_string(),
            error_message: self.to_string(),
            user_message: self.user_message(),
            timestamp: Utc::now(),
            suggested_fix: self.suggested_fix().to_string(),
            context: self.get_context(),
        }
    }

    /// Get contextual information for debugging
    fn get_context(&self) -> String {
        match self {
            Self::NavigationFailed { url, reason } => {
                format!("URL: {}, Reason: {}", url, reason)
            }
            _ => self.to_string(),
        }
    }
}

/// COE (Correction of Errors) error report
///
/// Used for post-incident analysis following AWS operational excellence
#[derive(Debug, Clone)]
pub struct ErrorReport {
    pub error_type: String,
    pub error_message: String,
    pub user_message: String,
    pub timestamp: DateTime<Utc>,
    pub suggested_fix: String,
    pub context: String,
}

impl ErrorReport {
    /// Format as log entry
    pub fn to_log_entry(&self) -> String {
        format!(
            "[COE] type={} time={} msg=\"{}\" fix=\"{}\"",
            self.error_type,
            self.timestamp.to_rfc3339(),
            self.error_message,
            self.suggested_fix
        )
    }
}

/// Log a BrowserError with COE context
///
/// AWS Pattern: Structured error logging for operational analysis
pub fn log_error_with_coe(error: &BrowserError) {
    use tracing::error;

    let report = error.to_coe_report();
    error!(
        error_type = %report.error_type,
        timestamp = %report.timestamp.to_rfc3339(),
        message = %report.error_message,
        suggested_fix = %report.suggested_fix,
        context = %report.context,
        "{}",
        report.to_log_entry()
    );
}

impl fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "COE Report:\n\
             Type: {}\n\
             Timestamp: {}\n\
             Error: {}\n\
             User Message: {}\n\
             Suggested Fix: {}\n\
             Context: {}",
            self.error_type,
            self.timestamp.to_rfc3339(),
            self.error_message,
            self.user_message,
            self.suggested_fix,
            self.context
        )
    }
}

/// Result type for browser operations
pub type Result<T> = std::result::Result<T, BrowserError>;

/// Convert anyhow errors to BrowserError
impl From<anyhow::Error> for BrowserError {
    fn from(err: anyhow::Error) -> Self {
        BrowserError::Unexpected(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        let error = BrowserError::NavigationFailed {
            url: "https://example.com".to_string(),
            reason: "timeout".to_string(),
        };

        assert_eq!(error.error_type(), "navigation_failed");
        assert!(error.user_message().contains("Failed to load"));
        assert!(error.suggested_fix().contains("network"));
    }

    #[test]
    fn test_coe_report_generation() {
        let error = BrowserError::Database("Connection failed".to_string());
        let report = error.to_coe_report();

        assert_eq!(report.error_type, "database");
        assert!(!report.suggested_fix.is_empty());
        assert!(report.timestamp.timestamp() > 0);
    }

    #[test]
    fn test_error_display() {
        let error = BrowserError::WebViewCreation("Platform error".to_string());
        let display = format!("{}", error);
        assert!(display.contains("WebView creation failed"));
    }

    #[test]
    fn test_coe_report_log_entry() {
        let error = BrowserError::IpcError("Invalid message".to_string());
        let report = error.to_coe_report();
        let log_entry = report.to_log_entry();

        assert!(log_entry.contains("[COE]"));
        assert!(log_entry.contains("type=ipc"));
        assert!(log_entry.contains("msg="));
        assert!(log_entry.contains("fix="));
    }

    #[test]
    fn test_all_error_types_have_suggestions() {
        let errors = vec![
            BrowserError::WebViewCreation("test".to_string()),
            BrowserError::NavigationFailed {
                url: "test".to_string(),
                reason: "test".to_string(),
            },
            BrowserError::Database("test".to_string()),
            BrowserError::IpcError("test".to_string()),
            BrowserError::ConfigError("test".to_string()),
            BrowserError::WindowError("test".to_string()),
            BrowserError::Unexpected("test".to_string()),
        ];

        for error in errors {
            assert!(!error.suggested_fix().is_empty());
            assert!(!error.user_message().is_empty());
            assert!(!error.error_type().is_empty());
        }
    }
}
