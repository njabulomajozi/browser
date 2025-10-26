//! WebView Manager - Single-threaded ownership of WebView lifecycle
//!
//! Following AWS service-oriented architecture patterns:
//! - **Single Responsibility**: Manages chrome + content WebViews only
//! - **Loose Coupling**: Clear interface, independent of navigation/state logic
//! - **Testability**: Can be mocked for testing
//!
//! # Architecture
//!
//! Manages dual WebView architecture:
//! - Chrome WebView (88px top): HTML UI (tabs, URL bar, navigation)
//! - Content WebView (below): Actual web pages

// Allow dead code temporarily - APIs will be integrated in Week 2
#![allow(dead_code)]

use crate::error::{BrowserError, Result};
use std::rc::Rc;
use tao::window::Window;
use tracing::{error, info};
use wry::{Rect, WebView, WebViewBuilder};

/// Configuration for WebView creation
#[derive(Debug, Clone)]
pub struct WebViewConfig {
    /// Height of chrome WebView in pixels (tab bar + URL bar)
    pub chrome_height: f64,
    /// Enable devtools in WebViews
    pub devtools_enabled: bool,
    /// Initial URL for content WebView
    pub initial_url: String,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            chrome_height: 88.0, // tab bar (40px) + URL bar (48px)
            devtools_enabled: cfg!(debug_assertions),
            initial_url: "https://example.com".to_string(),
        }
    }
}

/// WebView Manager - Single-threaded ownership pattern
///
/// Manages lifecycle of chrome + content WebViews following AWS
/// two-pizza team pattern (small, focused responsibility).
pub struct WebViewManager {
    /// Chrome WebView (top 88px) - HTML UI
    chrome_webview: Rc<WebView>,

    /// Content WebView (below chrome) - Web pages
    content_webview: Option<WebView>,

    /// Configuration
    config: WebViewConfig,
}

impl WebViewManager {
    /// Create new WebView manager with chrome WebView
    ///
    /// # Arguments
    /// * `window` - tao window to attach WebViews to
    /// * `config` - WebView configuration
    /// * `html_ui` - HTML content for chrome WebView
    /// * `ipc_handler` - IPC message handler closure
    ///
    /// # Errors
    /// Returns `BrowserError::WebViewCreation` if WebView creation fails
    pub fn new<F>(
        window: &Window,
        config: WebViewConfig,
        html_ui: &str,
        ipc_handler: F,
    ) -> Result<Self>
    where
        F: Fn(&str) + 'static,
    {
        info!("Creating WebView manager");

        // Create chrome WebView with IPC handler
        let chrome_webview = WebViewBuilder::new()
            .with_html(html_ui)
            .with_devtools(config.devtools_enabled)
            .with_ipc_handler(move |request| {
                let message_body = request.body();
                ipc_handler(message_body);
            })
            .build(window)
            .map_err(|e| BrowserError::WebViewCreation(e.to_string()))?;

        info!("✅ Chrome WebView created");

        Ok(Self {
            chrome_webview: Rc::new(chrome_webview),
            content_webview: None,
            config,
        })
    }

    /// Create content WebView (positioned below chrome)
    ///
    /// # Arguments
    /// * `window` - tao window to attach content WebView to
    /// * `url` - Initial URL to load
    /// * `navigation_handler` - Closure called on URL navigation
    ///
    /// # Errors
    /// Returns `BrowserError::WebViewCreation` if WebView creation fails
    pub fn create_content_webview<F>(
        &mut self,
        window: &Window,
        url: &str,
        navigation_handler: F,
    ) -> Result<()>
    where
        F: Fn(String) -> bool + 'static,
    {
        let window_size = window.inner_size();

        // Calculate content WebView bounds (below chrome)
        let content_bounds = Rect {
            position: tao::dpi::LogicalPosition::new(0.0, self.config.chrome_height).into(),
            size: tao::dpi::LogicalSize::new(
                window_size.width as f64,
                window_size.height as f64 - self.config.chrome_height,
            )
            .into(),
        };

        info!(
            "Creating content WebView at y={}, height={}",
            self.config.chrome_height,
            window_size.height as f64 - self.config.chrome_height
        );

        let content_webview = WebViewBuilder::new()
            .with_url(url)
            .with_bounds(content_bounds)
            .with_devtools(self.config.devtools_enabled)
            .with_navigation_handler(navigation_handler)
            .build_as_child(window)
            .map_err(|e| BrowserError::WebViewCreation(e.to_string()))?;

        self.content_webview = Some(content_webview);

        info!("✅ Content WebView created for: {}", url);

        Ok(())
    }

    /// Resize content WebView to match window size
    ///
    /// Called when window is resized.
    pub fn resize_content(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        if let Some(ref content_webview) = self.content_webview {
            let content_bounds = Rect {
                position: tao::dpi::LogicalPosition::new(0.0, self.config.chrome_height).into(),
                size: tao::dpi::LogicalSize::new(
                    new_width as f64,
                    new_height as f64 - self.config.chrome_height,
                )
                .into(),
            };

            content_webview
                .set_bounds(content_bounds)
                .map_err(|e| BrowserError::WindowError(e.to_string()))?;

            info!(
                "✅ Resized content WebView to {}x{}",
                new_width,
                new_height as f64 - self.config.chrome_height
            );
        }

        Ok(())
    }

    /// Evaluate JavaScript in chrome WebView
    ///
    /// # Arguments
    /// * `script` - JavaScript code to execute
    ///
    /// # Errors
    /// Returns `BrowserError::IpcError` if script evaluation fails
    pub fn evaluate_chrome_script(&self, script: &str) -> Result<()> {
        self.chrome_webview.evaluate_script(script).map_err(|e| {
            error!("Chrome script eval failed: {}", e);
            BrowserError::IpcError(format!("Chrome script eval failed: {}", e))
        })?;
        Ok(())
    }

    /// Evaluate JavaScript in content WebView
    ///
    /// # Arguments
    /// * `script` - JavaScript code to execute
    ///
    /// # Errors
    /// Returns `BrowserError::IpcError` if script evaluation fails
    /// Returns `BrowserError::NotInitialized` if content WebView not created
    pub fn evaluate_content_script(&self, script: &str) -> Result<()> {
        if let Some(ref content_webview) = self.content_webview {
            content_webview.evaluate_script(script).map_err(|e| {
                error!("Content script eval failed: {}", e);
                BrowserError::IpcError(format!("Content script eval failed: {}", e))
            })?;
            Ok(())
        } else {
            Err(BrowserError::ConfigError(
                "Content WebView not initialized".to_string(),
            ))
        }
    }

    /// Get reference to chrome WebView
    pub fn chrome_webview(&self) -> &Rc<WebView> {
        &self.chrome_webview
    }

    /// Get reference to content WebView if exists
    pub fn content_webview(&self) -> Option<&WebView> {
        self.content_webview.as_ref()
    }

    /// Check if content WebView is created
    pub fn has_content_webview(&self) -> bool {
        self.content_webview.is_some()
    }

    /// Get chrome height configuration
    pub fn chrome_height(&self) -> f64 {
        self.config.chrome_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = WebViewConfig::default();
        assert_eq!(config.chrome_height, 88.0);
        assert_eq!(config.initial_url, "https://example.com");
    }

    #[test]
    fn test_config_custom() {
        let config = WebViewConfig {
            chrome_height: 100.0,
            devtools_enabled: true,
            initial_url: "https://custom.com".to_string(),
        };
        assert_eq!(config.chrome_height, 100.0);
        assert!(config.devtools_enabled);
    }

    // Note: WebView creation tests require a window, which needs event loop.
    // These would be integration tests in tests/integration/
}
