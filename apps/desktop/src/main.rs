//! Browser MVP - Desktop Application
//!
//! Dual WebView architecture:
//! - Chrome WebView: HTML UI (tabs, URL bar, buttons)
//! - Content WebView: Child WebView for actual web pages (bypasses iframe X-Frame-Options)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tracing::{error, info, warn, Level};
use wry::{Rect, WebView, WebViewBuilder};

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

/// Browser application state
struct BrowserApp {
    #[allow(dead_code)] // Kept alive for IPC handler closure
    chrome_webview: WebView,
    content_webview: Option<WebView>,
    chrome_height: f64,
}

impl BrowserApp {
    fn new(window: &tao::window::Window) -> Result<Rc<RefCell<Option<Self>>>> {
        info!("Creating browser application");

        // Chrome height: tab bar (40px) + URL bar (48px) = 88px
        let chrome_height = 88.0;

        let html_ui = include_str!("ui.html");

        let app_holder: Rc<RefCell<Option<BrowserApp>>> = Rc::new(RefCell::new(None));
        let app_clone = app_holder.clone();

        // Create chrome WebView with IPC handler
        let chrome_webview = WebViewBuilder::new()
            .with_html(html_ui)
            .with_devtools(cfg!(debug_assertions))
            .with_ipc_handler(move |request| {
                let message_body = request.body();
                info!("IPC message received: {:?}", message_body);

                match serde_json::from_str::<IpcMessage>(message_body) {
                    Ok(IpcMessage::Navigate { url }) => {
                        info!("Navigate command: {}", url);
                        if let Some(ref mut app) = *app_clone.borrow_mut() {
                            if let Err(e) = app.navigate(&url) {
                                error!("Navigation failed: {}", e);
                            }
                        }
                    }
                    Ok(IpcMessage::GoBack) => {
                        info!("Go back command");
                        // TODO: Implement back navigation
                        warn!("Back navigation not yet implemented for child WebView");
                    }
                    Ok(IpcMessage::GoForward) => {
                        info!("Go forward command");
                        // TODO: Implement forward navigation
                        warn!("Forward navigation not yet implemented for child WebView");
                    }
                    Ok(IpcMessage::Reload) => {
                        info!("Reload command");
                        // TODO: Implement reload
                        warn!("Reload not yet implemented for child WebView");
                    }
                    Ok(IpcMessage::Stop) => {
                        info!("Stop command");
                        // TODO: Implement stop
                        warn!("Stop not yet implemented for child WebView");
                    }
                    Err(e) => {
                        error!("Failed to parse IPC message: {}", e);
                    }
                }
            })
            .build(window)?;

        // Create BrowserApp and store in holder
        let browser_app = BrowserApp {
            chrome_webview,
            content_webview: None,
            chrome_height,
        };
        *app_holder.borrow_mut() = Some(browser_app);

        info!("✅ Browser chrome WebView created");

        Ok(app_holder)
    }

    fn navigate(&mut self, url: &str) -> Result<()> {
        info!("Navigating content WebView to: {}", url);

        if let Some(ref content_webview) = self.content_webview {
            // Use wry's load_url to navigate existing WebView
            content_webview
                .load_url(url)
                .map_err(|e| anyhow::anyhow!("Failed to load URL: {}", e))?;
            info!("✅ Content WebView navigated to: {}", url);
        } else {
            warn!("No content WebView exists to navigate");
        }

        Ok(())
    }

    fn create_content_webview(&mut self, window: &tao::window::Window, url: &str) -> Result<()> {
        let window_size = window.inner_size();

        // Calculate content WebView bounds (below chrome)
        let content_bounds = Rect {
            position: LogicalPosition::new(0.0, self.chrome_height).into(),
            size: LogicalSize::new(
                window_size.width as f64,
                window_size.height as f64 - self.chrome_height,
            )
            .into(),
        };

        info!(
            "Creating content WebView at y={}, height={}",
            self.chrome_height,
            window_size.height as f64 - self.chrome_height
        );

        let content_webview = WebViewBuilder::new()
            .with_url(url)
            .with_bounds(content_bounds)
            .with_devtools(cfg!(debug_assertions))
            .build_as_child(window)?;

        self.content_webview = Some(content_webview);

        info!("✅ Content WebView created for: {}", url);

        Ok(())
    }

    fn resize_content_webview(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        if let Some(ref content_webview) = self.content_webview {
            let content_bounds = Rect {
                position: LogicalPosition::new(0.0, self.chrome_height).into(),
                size: LogicalSize::new(new_width as f64, new_height as f64 - self.chrome_height)
                    .into(),
            };

            content_webview.set_bounds(content_bounds)?;

            info!(
                "✅ Resized content WebView to {}x{}",
                new_width,
                new_height as f64 - self.chrome_height
            );
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Starting Browser MVP (Dual WebView architecture)");

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
        }
    }

    info!("✅ Browser MVP ready (chrome + content WebViews)");

    let app_for_resize = app.clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => {
                    info!("Close requested");
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(new_size) => {
                    info!("Window resized to {}x{}", new_size.width, new_size.height);

                    let mut app_guard = app_for_resize.borrow_mut();
                    if let Some(ref mut browser_app) = *app_guard {
                        if let Err(e) =
                            browser_app.resize_content_webview(new_size.width, new_size.height)
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
