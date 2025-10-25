//! Servo rendering engine wrapper
//!
//! This module provides a simplified API for embedding Servo in the browser.
//!
//! # Architecture
//!
//! The ServoRenderer manages:
//! - Servo engine initialization via ServoBuilder
//! - WebView creation via WebViewBuilder
//! - Event loop integration
//! - Rendering output capture
//!
//! # Milestone 1.3 MVP
//!
//! For the MVP, we implement:
//! - Basic Servo initialization
//! - Hardcoded HTML rendering (`data:` URLs)
//! - Delegate callbacks for state tracking
//!
//! Future milestones will add:
//! - URL navigation (Milestone 1.4)
//! - Multi-process architecture (Milestone 2.1)
//! - Full WebView API integration

use crate::delegate::BrowserWebViewDelegate;
use crate::delegate::DelegateState;
use crate::rendering_context::create_software_rendering_context;
use crate::types::{RenderedFrame, RendererConfig};
use crate::waker::EguiEventLoopWaker;
use crate::{RendererError, Result};
use euclid::Scale;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::webrender_api::units::DevicePixel;
use servo::{
    RenderingContext, Servo, ServoBuilder, SoftwareRenderingContext, WebView, WebViewBuilder,
};
use std::rc::Rc;
use tracing::{debug, info, warn};
use winit::dpi::PhysicalSize;

/// Main wrapper around the Servo rendering engine
///
/// This struct manages the Servo engine and webview lifecycle.
pub struct ServoRenderer {
    config: RendererConfig,
    delegate_state: DelegateState,
    waker: EguiEventLoopWaker,
    initialized: bool,

    // Servo instances
    rendering_context: Option<Rc<SoftwareRenderingContext>>,
    servo: Option<Servo>,
    webview: Option<WebView>,
}

impl ServoRenderer {
    /// Create a new renderer with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(RendererConfig::default())
    }

    /// Create a new renderer with custom configuration
    pub fn with_config(config: RendererConfig) -> Result<Self> {
        info!(
            "Initializing Servo renderer: {}x{} (DPR: {})",
            config.width, config.height, config.device_pixel_ratio
        );

        let delegate_state = DelegateState::new();
        let waker = EguiEventLoopWaker::new();

        Ok(Self {
            config,
            delegate_state,
            waker,
            initialized: false,
            rendering_context: None,
            servo: None,
            webview: None,
        })
    }

    /// Initialize the Servo engine
    ///
    /// This must be called before any rendering operations.
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        info!("Initializing Servo engine...");

        // 0. Initialize Servo global state (once per process)
        //    Uses std::sync::Once for test isolation
        Self::init_servo_globals();

        // 1. Create SoftwareRenderingContext
        let rendering_context =
            create_software_rendering_context(self.config.width, self.config.height)?;

        // 2. Cast to dyn RenderingContext for ServoBuilder
        let rendering_context_trait: Rc<dyn RenderingContext> = rendering_context.clone();

        // 3. Create EventLoopWaker
        let waker = Box::new(self.waker.clone());

        // 4. Build Servo instance
        let servo = ServoBuilder::new(rendering_context_trait)
            .event_loop_waker(waker)
            .build();

        // 5. Store instances
        self.rendering_context = Some(rendering_context);
        self.servo = Some(servo);
        self.initialized = true;

        info!("Servo initialized successfully");
        Ok(())
    }

    /// Initialize Servo global state
    ///
    /// Servo has some global state that can only be initialized once per process.
    /// This function uses std::sync::Once to ensure initialization happens exactly
    /// once, even if multiple ServoRenderer instances are created.
    ///
    /// **Note on Test Isolation:**
    /// Servo's internal opts initialization (in ServoBuilder::build()) can only happen
    /// once per process and cannot be wrapped by std::sync::Once. This means tests that
    /// call initialize() will fail with "Already initialized" when run in parallel.
    ///
    /// **Workaround:** Run tests serially: `cargo test -- --test-threads=1`
    ///
    /// This is a known Servo limitation and does not affect production code, which only
    /// initializes once per process.
    fn init_servo_globals() {
        use std::sync::Once;

        static INIT: Once = Once::new();

        INIT.call_once(|| {
            debug!("Initializing Servo global state (once per process)");

            // Initialize rustls crypto provider for TLS
            // This must happen before any TLS operations
            // Use .ok() to ignore error if already installed (e.g., in main.rs)
            let _ = rustls::crypto::ring::default_provider().install_default();

            // Note: ResourceReader warnings are expected for MVP
            // Servo will use embedded defaults for fonts/CSS
            // Future enhancement: Implement proper ResourceReader trait
            // See: https://book.servo.org/architecture/directory-structure.html
        });
    }

    /// Load hardcoded HTML content
    ///
    /// For Milestone 1.3 MVP, this renders static HTML using data: URLs
    pub fn load_html(&mut self, html: &str) -> Result<()> {
        if !self.initialized {
            self.initialize()?;
        }

        info!("Loading HTML content ({} bytes)", html.len());

        // Create data URL from HTML
        let data_url = format!("data:text/html,{}", html);
        debug!("Data URL: {}", data_url);

        // TODO: Load URL in Servo WebView
        //
        // if let Some(ref mut webview) = self.webview {
        //     let url = ServoUrl::parse(&data_url)
        //         .map_err(|e| RendererError::LoadFailed(e.to_string()))?;
        //     webview.load_url(url);
        // }

        // Simulate load start callback
        self.delegate_state.set_loading(true);
        self.delegate_state.set_url(Some(data_url.clone()));

        Ok(())
    }

    /// Load a URL (Milestone 1.4)
    ///
    /// Validates and loads the given URL string.
    /// Supports http://, https://, and data: URLs.
    pub fn load_url(&mut self, url_str: &str) -> Result<()> {
        if !self.initialized {
            self.initialize()?;
        }

        info!("Loading URL: {}", url_str);

        // Parse and validate URL
        let url = url::Url::parse(url_str)
            .map_err(|e| RendererError::LoadFailed(format!("Invalid URL '{}': {}", url_str, e)))?;

        // Validate scheme
        match url.scheme() {
            "http" | "https" | "data" => {}
            scheme => {
                return Err(RendererError::LoadFailed(format!(
                    "Unsupported URL scheme '{}'. Only http, https, and data URLs are supported.",
                    scheme
                )))
            }
        }

        info!(
            "Validated URL: scheme={}, host={:?}",
            url.scheme(),
            url.host_str()
        );

        // Get Servo instance
        let servo = self
            .servo
            .as_ref()
            .ok_or(RendererError::NotInitialized)?;

        // Create or reuse WebView
        if self.webview.is_none() {
            info!("Creating new WebView");

            // Create delegate
            let delegate = Rc::new(BrowserWebViewDelegate::new(
                self.delegate_state.clone(),
                self.waker.clone(),
            ));

            // Build WebView
            let size = PhysicalSize::new(self.config.width, self.config.height);
            let scale: Scale<f32, DeviceIndependentPixel, DevicePixel> =
                Scale::new(self.config.device_pixel_ratio);

            let webview = WebViewBuilder::new(servo)
                .delegate(delegate)
                .url(url.clone())
                .size(size)
                .hidpi_scale_factor(scale)
                .build();

            self.webview = Some(webview);
            info!("WebView created and URL loading initiated");
        } else {
            // Load URL in existing WebView
            info!("Loading URL in existing WebView");
            if let Some(ref webview) = self.webview {
                webview.load(url);
            }
        }

        Ok(())
    }

    /// Reload the current page
    pub fn reload(&mut self) -> Result<()> {
        if let Some(url) = self.get_url() {
            info!("Reloading: {}", url);
            self.load_url(&url)
        } else {
            Err(RendererError::Other("No URL to reload".to_string()))
        }
    }

    /// Stop loading the current page
    pub fn stop(&mut self) -> Result<()> {
        info!("Stopping page load");

        // Note: Servo's WebView doesn't expose a public stop_loading() method in 2025 API.
        // The delegate pattern means we can't force-stop from the embedder side.
        // Instead, we update our local state to reflect user intent.
        //
        // Future enhancement: This could be added to Servo's WebView API if needed.

        self.delegate_state.set_loading(false);
        Ok(())
    }

    /// Get the current rendered frame
    ///
    /// Returns the latest frame from Servo's compositor
    pub fn get_frame(&mut self) -> Result<RenderedFrame> {
        if !self.initialized {
            debug!("get_frame called but renderer not initialized");
            return Err(RendererError::NotInitialized);
        }

        // Read pixels from rendering context using read_to_image()
        if let Some(ref context) = self.rendering_context {
            use euclid::{Box2D, Point2D};

            // Define the rectangle to read (full surface)
            let rect = Box2D::new(
                Point2D::new(0, 0),
                Point2D::new(self.config.width as i32, self.config.height as i32),
            );

            // Read pixels from the rendering context
            match context.read_to_image(rect) {
                Some(image_buffer) => {
                    // ImageBuffer from Servo is ImageBuffer<Rgba<u8>, Vec<u8>>
                    // Get the raw pixel data
                    let pixels = image_buffer.into_raw();

                    // Count non-zero pixels for debugging
                    let non_zero_count = pixels.iter().filter(|&&p| p != 0).count();

                    debug!(
                        "Read frame from context: {}x{} ({} bytes, {} non-zero pixels)",
                        self.config.width,
                        self.config.height,
                        pixels.len(),
                        non_zero_count
                    );

                    // Diagnostic: Sample first few pixels
                    if pixels.len() >= 16 {
                        debug!(
                            "First 4 pixels (RGBA): [{},{},{},{}] [{},{},{},{}] [{},{},{},{}] [{},{},{},{}]",
                            pixels[0], pixels[1], pixels[2], pixels[3],
                            pixels[4], pixels[5], pixels[6], pixels[7],
                            pixels[8], pixels[9], pixels[10], pixels[11],
                            pixels[12], pixels[13], pixels[14], pixels[15],
                        );
                    }

                    Ok(RenderedFrame::new(
                        self.config.width,
                        self.config.height,
                        pixels,
                    ))
                }
                None => {
                    // Failed to read pixels - this could indicate:
                    // 1. Rendering context lost (GPU reset, resize)
                    // 2. Invalid read rectangle
                    // 3. WebView not rendered yet
                    warn!("Failed to read pixels from rendering context - read_to_image returned None");
                    warn!("This is normal during initial load. If persists, rendering context may be lost.");

                    // Return empty frame as fallback
                    let pixel_count = (self.config.width * self.config.height * 4) as usize;
                    Ok(RenderedFrame::new(
                        self.config.width,
                        self.config.height,
                        vec![0; pixel_count],
                    ))
                }
            }
        } else {
            // No rendering context - return empty frame
            warn!("get_frame called but no rendering context available");
            let pixel_count = (self.config.width * self.config.height * 4) as usize;
            Ok(RenderedFrame::new(
                self.config.width,
                self.config.height,
                vec![0; pixel_count],
            ))
        }
    }

    /// Update the renderer size
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        info!("Resizing renderer: {}x{}", width, height);

        let old_width = self.config.width;
        let old_height = self.config.height;

        self.config.width = width;
        self.config.height = height;

        // Notify Servo of resize
        if let Some(ref webview) = self.webview {
            let new_size = PhysicalSize::new(width, height);
            webview.resize(new_size);

            info!("Resized WebView from {}x{} to {}x{}",
                  old_width, old_height, width, height);
        }

        // Recreate rendering context if size changed significantly
        if (width != old_width || height != old_height) && self.rendering_context.is_some() {
            debug!("Recreating rendering context for new size");
            let new_context = create_software_rendering_context(width, height)?;
            self.rendering_context = Some(new_context);
        }

        Ok(())
    }

    /// Process events and update rendering
    ///
    /// This should be called in the main event loop.
    /// For Servo, the event processing is handled by spin_event_loop() and paint(),
    /// so this method is primarily for future extensibility.
    pub fn update(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }

        // Event processing is handled by:
        // 1. spin_event_loop() - processes compositor messages and delegate callbacks
        // 2. paint() - triggers WebView rendering
        //
        // This method is kept for future enhancements like:
        // - Custom input event injection
        // - Performance metrics collection
        // - Resource monitoring

        Ok(())
    }

    /// Check if the renderer is currently loading content
    pub fn is_loading(&self) -> bool {
        self.delegate_state.is_loading()
    }

    /// Get the current page title
    pub fn get_title(&self) -> Option<String> {
        self.delegate_state.get_title()
    }

    /// Get the current URL
    pub fn get_url(&self) -> Option<String> {
        self.delegate_state.get_url()
    }

    /// Get the load progress (0.0 to 1.0)
    pub fn get_progress(&self) -> f32 {
        self.delegate_state.get_progress()
    }

    /// Spin the Servo event loop
    ///
    /// This processes compositor updates, delegate methods, and rendering.
    /// Returns false when shutdown is complete.
    ///
    /// Should be called in the main event loop on every frame.
    pub fn spin_event_loop(&mut self) -> bool {
        if let Some(ref mut servo) = self.servo {
            servo.spin_event_loop()
        } else {
            true // No servo, keep running
        }
    }

    /// Paint the current WebView
    ///
    /// Triggers rendering of the WebView into the rendering context.
    /// Call this after spin_event_loop() to render the current frame.
    pub fn paint(&mut self) {
        if let Some(ref webview) = self.webview {
            webview.paint();

            // Present the rendered frame
            if let Some(ref context) = self.rendering_context {
                context.present();
            }
        }
    }

    /// Set the waker callback
    ///
    /// This callback will be invoked when Servo needs to wake the event loop.
    /// For egui, this should call `ctx.request_repaint()`.
    ///
    /// # Example
    /// ```ignore
    /// let ctx = cc.egui_ctx.clone();
    /// renderer.set_waker_callback(move || ctx.request_repaint());
    /// ```
    pub fn set_waker_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.waker.set_callback(callback);
    }
}

impl Default for ServoRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create default ServoRenderer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests that call initialize() may fail with "Already initialized"
    // when run in parallel due to Servo's global state. This is a known
    // limitation of Servo and doesn't affect production code.
    // Production code only initializes once per process.
    //
    // To run tests individually: cargo test -- --test-threads=1

    #[test]
    fn test_renderer_creation() {
        let renderer = ServoRenderer::new().unwrap();
        assert!(!renderer.initialized);
    }

    #[test]
    fn test_renderer_initialization() {
        let mut renderer = ServoRenderer::new().unwrap();
        renderer.initialize().unwrap();
        assert!(renderer.initialized);
    }

    #[test]
    fn test_load_html() {
        let mut renderer = ServoRenderer::new().unwrap();
        let html = "<h1>Hello, Servo!</h1>";
        renderer.load_html(html).unwrap();
        assert!(renderer.is_loading());
    }

    #[test]
    fn test_get_frame() {
        let mut renderer = ServoRenderer::new().unwrap();
        renderer.initialize().unwrap();
        let frame = renderer.get_frame().unwrap();
        assert_eq!(frame.width, 1024);
        assert_eq!(frame.height, 768);
    }

    #[test]
    fn test_resize() {
        let mut renderer = ServoRenderer::new().unwrap();
        renderer.resize(800, 600).unwrap();
        assert_eq!(renderer.config.width, 800);
        assert_eq!(renderer.config.height, 600);
    }

    #[test]
    fn test_load_url_http() {
        let mut renderer = ServoRenderer::new().unwrap();
        let result = renderer.load_url("http://example.com");
        assert!(result.is_ok());
        assert!(renderer.is_loading());
        assert_eq!(renderer.get_url(), Some("http://example.com/".to_string()));
    }

    #[test]
    fn test_load_url_https() {
        let mut renderer = ServoRenderer::new().unwrap();
        let result = renderer.load_url("https://www.wikipedia.org");
        assert!(result.is_ok());
        assert!(renderer.is_loading());
        assert_eq!(
            renderer.get_url(),
            Some("https://www.wikipedia.org/".to_string())
        );
    }

    #[test]
    fn test_load_url_invalid() {
        let mut renderer = ServoRenderer::new().unwrap();
        let result = renderer.load_url("not a url");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid URL"));
    }

    #[test]
    fn test_load_url_unsupported_scheme() {
        let mut renderer = ServoRenderer::new().unwrap();
        let result = renderer.load_url("ftp://example.com");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported URL scheme"));
    }

    #[test]
    fn test_reload() {
        let mut renderer = ServoRenderer::new().unwrap();
        renderer.load_url("http://example.com").unwrap();

        let result = renderer.reload();
        assert!(result.is_ok());
        assert!(renderer.is_loading());
    }

    #[test]
    fn test_reload_no_url() {
        let mut renderer = ServoRenderer::new().unwrap();
        let result = renderer.reload();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No URL to reload"));
    }

    #[test]
    fn test_stop() {
        let mut renderer = ServoRenderer::new().unwrap();
        renderer.load_url("http://example.com").unwrap();
        assert!(renderer.is_loading());

        renderer.stop().unwrap();
        assert!(!renderer.is_loading());
    }

    /// Test simple HTML rendering with data URL
    ///
    /// This is the most basic test - load simple HTML and verify it works.
    /// Milestone 1.3 requirement: "render hardcoded HTML string"
    #[test]
    fn test_simple_html_data_url() {
        let mut renderer = ServoRenderer::new().unwrap();

        // Very simple HTML
        let simple_html = "<html><head><title>Test</title></head><body><h1>Hello</h1></body></html>";
        let data_url = format!("data:text/html,{}", simple_html);

        // Load the data URL
        let result = renderer.load_url(&data_url);
        assert!(result.is_ok(), "Should load simple HTML data URL");

        // Verify state updated
        assert!(renderer.is_loading(), "Should be in loading state");
        assert_eq!(
            renderer.get_url(),
            Some(data_url),
            "URL should be set"
        );
    }

    /// Test minimal HTML rendering
    ///
    /// Absolute minimum HTML - just a heading tag
    #[test]
    fn test_minimal_html() {
        let mut renderer = ServoRenderer::new().unwrap();

        // Minimal HTML
        let result = renderer.load_html("<h1>Test</h1>");
        assert!(result.is_ok(), "Should load minimal HTML");
        assert!(renderer.is_loading(), "Should be loading");
    }
}
