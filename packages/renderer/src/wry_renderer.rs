//! Wry WebView renderer
//!
//! This module provides a WebView-based rendering engine using wry.
//! Unlike Servo which renders to pixels, wry uses platform native WebViews.
//!
//! # Architecture
//!
//! wry renders directly to the window using platform APIs:
//! - macOS: WKWebView (WebKit)
//! - Windows: WebView2 (Chromium)
//! - Linux: WebKitGTK
//!
//! # Integration with egui
//!
//! Since wry renders to a window region (not pixels), we use child WebView:
//! 1. Main window holds both egui and wry
//! 2. egui renders chrome (tabs, URL bar) in top area
//! 3. wry WebView positioned below chrome for content
//!
//! This is a hybrid approach: egui for UI chrome, wry for web content.

use crate::{RendererError, Result};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};
use url::Url;
use winit::raw_window_handle::HasWindowHandle;
use wry::WebView;

/// Shared state for WebView callbacks
#[derive(Clone, Default)]
struct WebViewState {
    url: Arc<Mutex<String>>,
    title: Arc<Mutex<String>>,
    loading: Arc<Mutex<bool>>,
}

impl WebViewState {
    fn new() -> Self {
        Self::default()
    }

    fn set_url(&self, url: String) {
        *self.url.lock().unwrap() = url;
    }

    #[allow(dead_code)]
    fn set_title(&self, title: String) {
        *self.title.lock().unwrap() = title;
    }

    fn set_loading(&self, loading: bool) {
        *self.loading.lock().unwrap() = loading;
    }

    fn get_url(&self) -> String {
        self.url.lock().unwrap().clone()
    }

    fn get_title(&self) -> String {
        self.title.lock().unwrap().clone()
    }

    fn is_loading(&self) -> bool {
        *self.loading.lock().unwrap()
    }
}

/// Wry-based WebView renderer
///
/// This renderer manages a wry WebView that displays web content.
/// Unlike pixel-based renderers, wry renders directly to the window.
pub struct WryRenderer {
    state: WebViewState,
    webview: Option<WebView>,
}

impl WryRenderer {
    /// Create a new wry renderer (without WebView)
    ///
    /// Call `create_webview()` after obtaining the window handle
    pub fn new() -> Result<Self> {
        info!("Initializing Wry renderer");

        Ok(Self {
            state: WebViewState::new(),
            webview: None,
        })
    }

    /// Create the actual WebView
    ///
    /// This must be called from the main thread with a valid window.
    /// The WebView will be created as a child with the specified bounds.
    ///
    /// # Arguments
    /// * `window` - The winit window to attach the WebView to
    /// * `bounds` - The position and size of the WebView (for child WebView)
    #[cfg(not(target_os = "linux"))]
    pub fn create_webview(
        &mut self,
        window: &impl HasWindowHandle,
        url: &str,
    ) -> Result<()> {
        use wry::WebViewBuilder;

        info!("Creating wry WebView with URL: {}", url);

        let state = self.state.clone();

        let webview = WebViewBuilder::new()
            .with_url(url)
            .with_devtools(cfg!(debug_assertions))
            // Navigation handler - update state when URL changes
            .with_navigation_handler(move |uri: String| {
                debug!("Navigation: {}", uri);
                state.set_url(uri);
                true // Allow navigation
            })
            .build(window)
            .map_err(|e| RendererError::WebViewCreationFailed(e.to_string()))?;

        self.webview = Some(webview);
        info!("Wry WebView created successfully");

        Ok(())
    }

    /// Create WebView on Linux (requires GTK)
    #[cfg(target_os = "linux")]
    pub fn create_webview(
        &mut self,
        window: &impl HasWindowHandle,
        url: &str,
    ) -> Result<()> {
        use wry::WebViewBuilder;

        info!("Creating wry WebView (Linux/GTK) with URL: {}", url);

        let state = self.state.clone();

        let webview = WebViewBuilder::new()
            .with_url(url)
            .with_devtools(cfg!(debug_assertions))
            .with_navigation_handler(move |uri: String| {
                debug!("Navigation: {}", uri);
                state.set_url(uri);
                true
            })
            .build(window)
            .map_err(|e| RendererError::WebViewCreationFailed(e.to_string()))?;

        self.webview = Some(webview);
        info!("Wry WebView created successfully (Linux/GTK)");

        Ok(())
    }

    /// Load a URL in the WebView
    pub fn load_url(&mut self, url_str: &str) -> Result<()> {
        // Validate URL
        let url = Url::parse(url_str)
            .map_err(|e| RendererError::LoadFailed(format!("Invalid URL: {}", e)))?;

        info!("Loading URL: {}", url);

        if let Some(ref webview) = self.webview {
            webview
                .load_url(url.as_str())
                .map_err(|e| RendererError::LoadFailed(e.to_string()))?;

            self.state.set_loading(true);
            self.state.set_url(url.to_string());

            Ok(())
        } else {
            Err(RendererError::NotInitialized)
        }
    }

    /// Reload the current page
    pub fn reload(&mut self) -> Result<()> {
        info!("Reloading page");

        if let Some(ref webview) = self.webview {
            // wry doesn't have direct reload - use JavaScript
            let _ = webview.evaluate_script("window.location.reload();");
            Ok(())
        } else {
            Err(RendererError::NotInitialized)
        }
    }

    /// Navigate back
    pub fn go_back(&mut self) -> Result<()> {
        info!("Navigating back");

        if let Some(ref webview) = self.webview {
            // Use JavaScript for back navigation
            let _ = webview.evaluate_script("window.history.back();");
            Ok(())
        } else {
            Err(RendererError::NotInitialized)
        }
    }

    /// Navigate forward
    pub fn go_forward(&mut self) -> Result<()> {
        info!("Navigating forward");

        if let Some(ref webview) = self.webview {
            // Use JavaScript for forward navigation
            let _ = webview.evaluate_script("window.history.forward();");
            Ok(())
        } else {
            Err(RendererError::NotInitialized)
        }
    }

    /// Stop loading the current page
    pub fn stop(&mut self) -> Result<()> {
        info!("Stopping page load");

        if let Some(ref webview) = self.webview {
            // wry doesn't expose stop - use JavaScript
            let _ = webview.evaluate_script("window.stop();");
            self.state.set_loading(false);
            Ok(())
        } else {
            Err(RendererError::NotInitialized)
        }
    }

    /// Check if a page is currently loading
    pub fn is_loading(&self) -> bool {
        self.state.is_loading()
    }

    /// Get the current URL
    pub fn get_url(&self) -> Option<String> {
        let url = self.state.get_url();
        if url.is_empty() {
            None
        } else {
            Some(url)
        }
    }

    /// Get the page title
    pub fn get_title(&self) -> Option<String> {
        let title = self.state.get_title();
        if title.is_empty() {
            None
        } else {
            Some(title)
        }
    }

    /// Execute JavaScript in the WebView
    pub fn eval_script(&self, script: &str) -> Result<()> {
        if let Some(ref webview) = self.webview {
            webview
                .evaluate_script(script)
                .map_err(|e| RendererError::Other(format!("Script eval failed: {}", e)))?;
            Ok(())
        } else {
            Err(RendererError::NotInitialized)
        }
    }

    /// Get mutable reference to WebView (for advanced usage)
    pub fn webview_mut(&mut self) -> Option<&mut WebView> {
        self.webview.as_mut()
    }

    /// Get reference to WebView
    pub fn webview(&self) -> Option<&WebView> {
        self.webview.as_ref()
    }
}

impl Default for WryRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create WryRenderer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_renderer() {
        let renderer = WryRenderer::new();
        assert!(renderer.is_ok());
    }

    #[test]
    fn test_state_management() {
        let state = WebViewState::new();

        state.set_url("https://example.com".to_string());
        assert_eq!(state.get_url(), "https://example.com");

        state.set_title("Example".to_string());
        assert_eq!(state.get_title(), "Example");

        state.set_loading(true);
        assert!(state.is_loading());
    }
}
