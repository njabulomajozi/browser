//! Browser rendering engine wrapper
//!
//! This crate wraps platform-native WebViews via wry (WKWebView on macOS,
//! WebView2 on Windows, WebKitGTK on Linux) and provides a simplified API
//! for embedding in the browser application.
//!
//! # wry Architecture Overview
//!
//! wry is a cross-platform WebView rendering library that wraps platform WebViews:
//!
//! - **macOS**: WKWebView (WebKit-based, same as Safari)
//! - **Windows**: WebView2 (Chromium-based, Microsoft Edge engine)
//! - **Linux**: WebKitGTK (WebKit-based)
//!
//! Key features:
//! - **Platform-native**: Uses OS-provided WebView components
//! - **Lightweight**: No bundled browser engine (relies on system WebView)
//! - **Cross-platform**: Single API for all platforms
//! - **IPC Support**: JavaScript ↔ Rust communication
//! - **Secure**: Leverages platform security features
//!
//! # This Crate's Architecture
//!
//! We implement a simplified wrapper pattern:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │ BrowserApp (HTML/CSS/JS chrome UI)                  │
//! │  - Tab bar, URL bar, navigation buttons             │
//! │  - User input handling                              │
//! │  - IPC message passing to Rust                      │
//! └────────────┬────────────────────────────────────────┘
//!              │ uses
//!              ▼
//! ┌─────────────────────────────────────────────────────┐
//! │ WryRenderer (this crate)                            │
//! │  - Simplified API for browser embedding             │
//! │  - Manages wry WebView lifecycle                    │
//! │  - Handles platform-specific differences            │
//! └────────────┬────────────────────────────────────────┘
//!              │ wraps
//!              ▼
//! ┌─────────────────────────────────────────────────────┐
//! │ wry (Platform WebView wrapper)                      │
//! │  ├─ macOS: WKWebView (WebKit)                       │
//! │  ├─ Windows: WebView2 (Chromium/Edge)               │
//! │  └─ Linux: WebKitGTK (WebKit)                       │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Dual WebView Architecture
//!
//! The browser uses two WebViews:
//!
//! 1. **Chrome WebView** (top, fixed height 88px):
//!    - Displays HTML UI (tabs, URL bar, buttons)
//!    - Handles user interactions
//!    - Sends IPC messages to Rust backend
//!
//! 2. **Content WebView** (below chrome, fills remaining space):
//!    - Displays actual web pages
//!    - Navigates to user-requested URLs
//!    - Reports navigation events back to chrome
//!
//! This separation allows custom browser UI while avoiding X-Frame-Options
//! issues when embedding web content.
//!
//! ## Key Types
//!
//! - **`WryRenderer`**: Main wrapper around wry WebView
//!   - Manages WebView lifecycle
//!   - Provides simple load_url() / get_frame() API (future)
//!   - Handles platform-specific initialization
//!
//! - **`RendererConfig`**: Configuration for WebView creation
//!   - Window size, devtools, IPC handlers
//!
//! - **`RendererError`**: Error types for WebView operations
//!   - Initialization, navigation, IPC failures
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use renderer::WryRenderer;
//!
//! // Initialize renderer (platform-specific WebView)
//! let mut renderer = WryRenderer::new()?;
//!
//! // Renderer is managed by desktop app's WebViewManager
//! // See apps/desktop/src/webview_manager.rs for actual usage
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Platform-Specific Notes
//!
//! ### macOS (WKWebView)
//! - Requires macOS 10.12+
//! - Uses Metal for GPU acceleration
//! - Native WebKit engine (same as Safari)
//! - Excellent standards compliance
//!
//! ### Windows (WebView2)
//! - Requires Windows 7+ with Edge WebView2 Runtime
//! - Chromium-based (Edge engine)
//! - Auto-updates with Windows
//! - Best Chromium compatibility
//!
//! ### Linux (WebKitGTK)
//! - Requires webkit2gtk-4.0
//! - WebKit engine (similar to Safari)
//! - Install: `sudo apt install libwebkit2gtk-4.0-dev`
//!
//! ## Future Considerations
//!
//! We may migrate to libservo (pure Rust engine) when it reaches v1.0 stability.
//! This would provide:
//! - Complete Rust-based rendering pipeline
//! - Servo's parallel layout and style system
//! - Better integration with Rust ecosystem
//! - Independent of platform WebView availability
//!
//! See [docs/decisions/006-wry-architecture.md](../../docs/decisions/006-wry-architecture.md)
//! for architectural decision rationale.

use thiserror::Error;

// wry renderer implementation (actual WebView wrapper)
mod wry_renderer;
pub use wry_renderer::WryRenderer;

/// Errors that can occur during rendering operations
#[derive(Debug, Error)]
pub enum RendererError {
    /// Renderer not initialized before use
    #[error("Rendering engine not initialized. Hint: Call WryRenderer::new() first")]
    NotInitialized,

    /// Failed to load URL
    #[error("Failed to load URL: {0}")]
    LoadFailed(String),

    /// Platform WebView initialization failed
    #[error("Platform WebView initialization failed: {0}")]
    InitFailed(String),

    /// WebView creation failed
    #[error("WebView creation failed: {0}")]
    WebViewCreationFailed(String),

    /// Other errors
    #[error("{0}")]
    Other(String),
}

/// Result type for renderer operations
pub type Result<T> = std::result::Result<T, RendererError>;

// Configuration types
mod types;
pub use types::RendererConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RendererError::NotInitialized;
        // Error messages now include hints for recovery
        assert!(err.to_string().contains("engine not initialized"));
        assert!(err.to_string().contains("Hint:"));
    }
}
