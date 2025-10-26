//! Browser rendering engine wrapper
//!
//! This crate wraps the Servo rendering engine (2025 WebView API)
//! and provides a simplified API for embedding in the browser application.
//!
//! # Servo Architecture Overview
//!
//! Servo is a modern, parallel browser engine written in Rust. Key components:
//!
//! - **Compositor**: Manages WebRender, handles frame scheduling and painting
//! - **WebView**: Represents a single browsing context with its own rendering pipeline
//! - **Delegates**: Callback interfaces for page lifecycle events (load, title changes, etc.)
//! - **EventLoopWaker**: Cross-thread communication mechanism for async events
//! - **RenderingContext**: Abstraction over OpenGL/Software rendering surfaces
//!
//! # This Crate's Architecture
//!
//! We implement the embedder pattern:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │ BrowserApp (egui)                                   │
//! │  - UI rendering                                     │
//! │  - User input handling                              │
//! └────────────┬────────────────────────────────────────┘
//!              │ uses
//!              ▼
//! ┌─────────────────────────────────────────────────────┐
//! │ ServoRenderer (this crate)                          │
//! │  - Simplified API for browser embedding             │
//! │  - Manages Servo lifecycle                          │
//! │  - Provides pixel frames for display                │
//! └────────────┬────────────────────────────────────────┘
//!              │ wraps
//!              ▼
//! ┌─────────────────────────────────────────────────────┐
//! │ Servo Engine (libservo)                             │
//! │  - HTML/CSS/JS parsing and execution                │
//! │  - Layout (Box tree → Fragment tree → Display list) │
//! │  - WebRender (GPU-accelerated compositor)           │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Types
//!
//! - **`ServoRenderer`**: Main wrapper around Servo engine
//!   - Manages Servo/WebView lifecycle
//!   - Provides simple load_url() / get_frame() API
//!   - Handles delegate callbacks and event loop integration
//!
//! - **`BrowserWebViewDelegate`**: Implements WebViewDelegate trait
//!   - Receives callbacks from Servo (page load, title change, etc.)
//!   - Updates shared DelegateState for UI synchronization
//!   - Wakes event loop when repainting is needed
//!
//! - **`EventLoopWaker`**: Implements EventLoopWaker trait
//!   - Allows Servo (background thread) to wake main event loop
//!   - Calls egui's request_repaint() when new frames are ready
//!
//! - **`SoftwareRenderingContext`**: CPU-based rendering fallback
//!   - Doesn't require OpenGL/GPU
//!   - Slower than GL but simpler for MVP
//!   - Provides read_to_image() for pixel buffer access
//!
//! # Usage Example
//!
//! ```ignore
//! use renderer::ServoRenderer;
//!
//! // Create and initialize renderer
//! let mut renderer = ServoRenderer::new()?;
//!
//! // Set up event loop waker (for egui integration)
//! let ctx = cc.egui_ctx.clone();
//! renderer.set_waker_callback(move || ctx.request_repaint());
//!
//! // Load a URL
//! renderer.load_url("https://example.com")?;
//!
//! // In your event loop:
//! loop {
//!     renderer.spin_event_loop();  // Process compositor events
//!     renderer.paint();             // Render frame
//!
//!     let frame = renderer.get_frame()?;  // Get pixels
//!     // Display frame.pixels in your UI
//! }
//! ```
//!
//! # Servo 2025 WebView API Patterns
//!
//! This implementation follows Servo's modern embedding patterns:
//!
//! 1. **ServoBuilder**: Initializes Servo engine with rendering context and waker
//! 2. **WebViewBuilder**: Creates WebViews with delegate and initial URL
//! 3. **Delegate Pattern**: Callbacks instead of message passing
//! 4. **Frame Synchronization**: spin_event_loop() + paint() cycle
//!
//! See: https://servo.org/blog/2025/02/19/this-month-in-servo/ (delegate API)
//!
//! # Thread Safety
//!
//! - Servo runs the compositor on a background thread
//! - WebView operations must be called from the same thread (main thread)
//! - EventLoopWaker enables cross-thread communication (compositor → main)
//! - DelegateState uses Arc<Mutex<T>> for thread-safe shared state

use thiserror::Error;

// wry-based renderer (MVP implementation)
mod wry_renderer;
pub use wry_renderer::WryRenderer;

// Servo-based renderer (future - when libservo v1.0 releases)
// mod delegate;
// mod rendering_context;
// mod servo_wrapper;
// mod waker;
// pub use servo_wrapper::ServoRenderer;

// Types
mod types;
pub use types::RendererConfig;

/// Errors that can occur during rendering operations
#[derive(Error, Debug)]
pub enum RendererError {
    /// Servo initialization failed
    ///
    /// This usually indicates missing system dependencies or incompatible configuration.
    /// Recovery: Check that Servo dependencies are installed, see README.md
    #[error("Failed to initialize Servo: {0}\n\nHint: Ensure Servo dependencies are installed. See packages/renderer/README.md")]
    InitializationFailed(String),

    /// Failed to create WebView
    ///
    /// This can happen if Servo is not properly initialized or resources are exhausted.
    /// Recovery: Ensure initialize() was called successfully before creating WebViews
    #[error("Failed to create WebView: {0}\n\nHint: Call initialize() before loading URLs")]
    WebViewCreationFailed(String),

    /// Failed to load content
    ///
    /// This can be due to invalid URLs, network errors, or unsupported protocols.
    /// Recovery: Validate URL format, check network connectivity, ensure supported scheme (http/https/data)
    #[error("Failed to load content: {0}\n\nHint: Check URL format and network connectivity")]
    LoadFailed(String),

    /// GL context error
    ///
    /// OpenGL context errors typically indicate driver or compatibility issues.
    /// Recovery: Update graphics drivers, use software rendering fallback
    #[error("OpenGL context error: {0}\n\nHint: Update graphics drivers or use software rendering")]
    GlContextError(String),

    /// Servo not initialized
    ///
    /// Operations requiring Servo engine were attempted before initialization.
    /// Recovery: Call initialize() before performing rendering operations
    #[error("Servo engine not initialized\n\nHint: Call ServoRenderer::new().initialize() before use")]
    NotInitialized,

    /// Rendering context lost
    ///
    /// The rendering surface became invalid, possibly due to resize or GPU reset.
    /// Recovery: Recreate the rendering context
    #[error("Rendering context lost: {0}\n\nHint: Recreate renderer or resize window")]
    ContextLost(String),

    /// Generic error
    #[error("Renderer error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for RendererError {
    fn from(err: anyhow::Error) -> Self {
        // Try to provide more specific error types based on error message
        let msg = err.to_string();

        if msg.contains("GL") || msg.contains("OpenGL") {
            RendererError::GlContextError(msg)
        } else if msg.contains("initialize") || msg.contains("init") {
            RendererError::InitializationFailed(msg)
        } else if msg.contains("WebView") {
            RendererError::WebViewCreationFailed(msg)
        } else {
            RendererError::Other(msg)
        }
    }
}

/// Result type for renderer operations
pub type Result<T> = std::result::Result<T, RendererError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RendererError::NotInitialized;
        // Error messages now include hints for recovery
        assert!(err.to_string().contains("Servo engine not initialized"));
        assert!(err.to_string().contains("Hint:"));
    }
}
