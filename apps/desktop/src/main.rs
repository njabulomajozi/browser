//! Browser MVP - Desktop Application (Refactored with AWS Service Architecture)
//!
//! Service-Oriented Architecture following AWS patterns:
//! - **Dependency Injection**: Services injected into BrowserApp
//! - **Loose Coupling**: Clear service boundaries
//! - **Operational Excellence**: Metrics tracking, error handling
//!
//! # Architecture
//!
//! ```text
//! BrowserApp (orchestrator)
//! ├── WebViewManager (chrome + content WebViews)
//! ├── NavigationService (URL loading, history)
//! ├── StateManager (tabs, active tab)
//! └── Metrics (DORA tracking)
//! ```

mod error;
mod health;
mod metrics;
mod navigation;
mod state;
mod webview_manager;

use crate::error::{log_error_with_coe, BrowserError, Result};
use crate::health::HealthChecker;
use crate::metrics::Metrics;
use crate::navigation::NavigationService;
use crate::state::StateManager;
use crate::webview_manager::{WebViewConfig, WebViewManager};

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use storage::Database;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tracing::{error, info, Level};

/// IPC message from JavaScript to Rust
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "cmd", content = "data")]
enum IpcMessage {
    Navigate { url: String },
    GoBack,
    GoForward,
    Reload,
    Stop,
}

/// Browser application with service-oriented architecture
///
/// AWS Pattern: Orchestrator with dependency-injected services
struct BrowserApp {
    /// WebView management service
    webview_manager: WebViewManager,

    /// Navigation service
    navigation_service: NavigationService,

    /// State management service (will be integrated in Week 2)
    #[allow(dead_code)]
    state_manager: StateManager,

    /// Metrics collector
    metrics: Arc<Metrics>,

    /// Health checker (ORR pattern)
    health_checker: HealthChecker,
}

impl BrowserApp {
    /// Create browser application with dependency injection
    ///
    /// # AWS Pattern
    /// - Services created independently and injected
    /// - Metrics shared across services via Arc
    /// - Database path configurable for testing
    fn new(window: &tao::window::Window) -> Result<Rc<RefCell<Option<Self>>>> {
        info!("Creating browser application (AWS service architecture)");

        // Initialize metrics (shared across services)
        let metrics = Metrics::new();
        info!("✅ Metrics system initialized");

        // Initialize database
        let db_path = PathBuf::from("browser.db");
        let _db = Database::new(&db_path).map_err(|e| BrowserError::Database(e.to_string()))?;
        info!("✅ Database initialized at {:?}", db_path);

        // Create navigation service with metrics
        let navigation_service = NavigationService::new(db_path.clone(), metrics.clone());
        info!("✅ Navigation service initialized");

        // Create state manager
        let state_manager = StateManager::new();
        info!("✅ State manager initialized");

        // Create health checker (ORR pattern)
        let health_checker = HealthChecker::new(db_path, metrics.clone());
        info!("✅ Health checker initialized");

        let html_ui = include_str!("ui.html");

        let app_holder: Rc<RefCell<Option<BrowserApp>>> = Rc::new(RefCell::new(None));
        let app_clone = app_holder.clone();

        // WebView configuration
        let webview_config = WebViewConfig::default();

        // Create WebView manager with IPC handler
        let webview_manager =
            WebViewManager::new(window, webview_config, html_ui, move |message_body| {
                info!("IPC message received: {:?}", message_body);

                match serde_json::from_str::<IpcMessage>(message_body) {
                    Ok(IpcMessage::Navigate { url }) => {
                        info!("Navigate command: {}", url);
                        if let Some(ref mut app) = *app_clone.borrow_mut() {
                            if let Err(e) = app.handle_navigate(&url) {
                                log_error_with_coe(&e);
                                app.metrics
                                    .record_error(&format!("Navigation failed: {}", e));
                            }
                        }
                    }
                    Ok(IpcMessage::GoBack) => {
                        info!("Go back command");
                        if let Some(ref app) = *app_clone.borrow() {
                            if let Err(e) = app.handle_go_back() {
                                log_error_with_coe(&e);
                                app.metrics.record_error(&format!("Go back failed: {}", e));
                            }
                        }
                    }
                    Ok(IpcMessage::GoForward) => {
                        info!("Go forward command");
                        if let Some(ref app) = *app_clone.borrow() {
                            if let Err(e) = app.handle_go_forward() {
                                log_error_with_coe(&e);
                                app.metrics
                                    .record_error(&format!("Go forward failed: {}", e));
                            }
                        }
                    }
                    Ok(IpcMessage::Reload) => {
                        info!("Reload command");
                        if let Some(ref app) = *app_clone.borrow() {
                            if let Err(e) = app.handle_reload() {
                                log_error_with_coe(&e);
                                app.metrics.record_error(&format!("Reload failed: {}", e));
                            }
                        }
                    }
                    Ok(IpcMessage::Stop) => {
                        info!("Stop command");
                        if let Some(ref app) = *app_clone.borrow() {
                            if let Err(e) = app.handle_stop() {
                                log_error_with_coe(&e);
                                app.metrics.record_error(&format!("Stop failed: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse IPC message: {}", e);
                        if let Some(ref app) = *app_clone.borrow() {
                            app.metrics.record_error(&format!("IPC parse error: {}", e));
                        }
                    }
                }
            })?;

        info!("✅ WebView manager initialized");

        // Create BrowserApp with injected services
        let browser_app = BrowserApp {
            webview_manager,
            navigation_service,
            state_manager,
            metrics,
            health_checker,
        };

        *app_holder.borrow_mut() = Some(browser_app);

        info!("✅ Browser application ready (all services initialized)");

        Ok(app_holder)
    }

    /// Handle navigate IPC command
    ///
    /// Uses NavigationService for URL processing and metrics tracking
    fn handle_navigate(&mut self, url: &str) -> Result<()> {
        let start = Instant::now();

        // Navigate via service (handles history, metrics)
        let nav_result = self.navigation_service.navigate(url)?;

        // Update chrome URL bar
        let update_script = format!(
            "document.getElementById('url-input').value = '{}'",
            nav_result.url.replace('\'', "\\'")
        );
        self.webview_manager
            .evaluate_chrome_script(&update_script)?;

        // Update tab title
        let title_script = format!(
            "updateTabTitle('{}')",
            nav_result.title.replace('\'', "\\'")
        );
        self.webview_manager.evaluate_chrome_script(&title_script)?;

        // Navigate content WebView
        if let Some(content_webview) = self.webview_manager.content_webview() {
            content_webview.load_url(&nav_result.url).map_err(|e| {
                BrowserError::NavigationFailed {
                    url: nav_result.url.clone(),
                    reason: e.to_string(),
                }
            })?;

            let duration = start.elapsed();
            self.metrics.record_navigation(true, duration);
            info!("✅ Navigation successful: {} ({:?})", url, duration);
        } else {
            return Err(BrowserError::ConfigError(
                "Content WebView not initialized".to_string(),
            ));
        }

        Ok(())
    }

    /// Handle go back IPC command
    fn handle_go_back(&self) -> Result<()> {
        if let Some(content_webview) = self.webview_manager.content_webview() {
            self.navigation_service.go_back(content_webview)?;
            Ok(())
        } else {
            Err(BrowserError::ConfigError(
                "Content WebView not initialized".to_string(),
            ))
        }
    }

    /// Handle go forward IPC command
    fn handle_go_forward(&self) -> Result<()> {
        if let Some(content_webview) = self.webview_manager.content_webview() {
            self.navigation_service.go_forward(content_webview)?;
            Ok(())
        } else {
            Err(BrowserError::ConfigError(
                "Content WebView not initialized".to_string(),
            ))
        }
    }

    /// Handle reload IPC command
    fn handle_reload(&self) -> Result<()> {
        if let Some(content_webview) = self.webview_manager.content_webview() {
            self.navigation_service.reload(content_webview)?;
            Ok(())
        } else {
            Err(BrowserError::ConfigError(
                "Content WebView not initialized".to_string(),
            ))
        }
    }

    /// Handle stop IPC command
    fn handle_stop(&self) -> Result<()> {
        if let Some(content_webview) = self.webview_manager.content_webview() {
            self.navigation_service.stop(content_webview)?;
            Ok(())
        } else {
            Err(BrowserError::ConfigError(
                "Content WebView not initialized".to_string(),
            ))
        }
    }

    /// Create content WebView
    fn create_content_webview(&mut self, window: &tao::window::Window, url: &str) -> Result<()> {
        let chrome_webview = self.webview_manager.chrome_webview().clone();
        let db_path = PathBuf::from("browser.db");

        self.webview_manager
            .create_content_webview(window, url, move |url_str| {
                info!("Navigation event: {}", url_str);

                // Update URL bar
                let update_script = format!(
                    "document.getElementById('url-input').value = '{}'",
                    url_str.replace('\'', "\\'")
                );
                if let Err(e) = chrome_webview.evaluate_script(&update_script) {
                    error!("Failed to update URL bar: {}", e);
                }

                // Extract title from URL
                let title = url_str.split('/').nth(2).unwrap_or("New Tab").to_string();

                // Update tab title
                let title_script = format!("updateTabTitle('{}')", title.replace('\'', "\\'"));
                if let Err(e) = chrome_webview.evaluate_script(&title_script) {
                    error!("Failed to update tab title: {}", e);
                }

                // Save to history
                if let Ok(db) = Database::new(&db_path) {
                    if let Err(e) = db.add_history(&url_str, Some(&title)) {
                        error!("Failed to save history: {}", e);
                    } else {
                        info!("✅ Saved to history: {} - {}", url_str, title);
                    }
                }

                true // Allow navigation
            })?;

        Ok(())
    }

    /// Resize content WebView
    fn resize_content(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        self.webview_manager.resize_content(new_width, new_height)
    }

    /// Log metrics summary
    fn log_metrics(&self) {
        self.metrics.log_summary();
    }

    /// Perform health check (ORR pattern)
    fn check_health(&self) {
        let health = self.health_checker.check_health();
        info!(
            "Health check: {:?} - Readiness: {}, Liveness: {}",
            health.status,
            health.is_ready(),
            health.is_alive()
        );
    }
}

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Starting Browser MVP (AWS Service Architecture)");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Browser MVP")
        .with_inner_size(tao::dpi::LogicalSize::new(1024.0, 768.0))
        .build(&event_loop)?;

    let app = BrowserApp::new(&window)?;

    // Create initial content WebView
    {
        let mut app_guard = app.borrow_mut();
        if let Some(ref mut browser_app) = *app_guard {
            browser_app.create_content_webview(&window, "https://example.com")?;

            // Initial health check and metrics log
            browser_app.check_health();
            browser_app.log_metrics();
        }
    }

    info!("✅ Browser MVP ready (AWS service architecture)");

    let app_for_resize = app.clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => {
                    info!("Close requested");

                    // Log final metrics before exit
                    if let Some(ref app) = *app.borrow() {
                        app.log_metrics();
                    }

                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(new_size) => {
                    info!("Window resized to {}x{}", new_size.width, new_size.height);

                    let mut app_guard = app_for_resize.borrow_mut();
                    if let Some(ref mut browser_app) = *app_guard {
                        if let Err(e) = browser_app.resize_content(new_size.width, new_size.height)
                        {
                            error!("Failed to resize content WebView: {}", e);
                        }
                    }
                }
                _ => {}
            }
        }
    });
}
