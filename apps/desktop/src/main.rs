//! Browser MVP - Desktop Application
//!
//! HTML/CSS UI chrome loaded in wry WebView (Tauri pattern)

use anyhow::Result;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tracing::{info, Level};
use wry::WebViewBuilder;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Starting Browser MVP (HTML UI in wry)");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Browser MVP")
        .with_inner_size(tao::dpi::LogicalSize::new(1024.0, 768.0))
        .build(&event_loop)?;

    // Load HTML UI from file
    let html_ui = include_str!("ui.html");

    let _webview = WebViewBuilder::new()
        .with_html(html_ui)
        .with_devtools(cfg!(debug_assertions))
        .build(&window)?;

    info!("✅ Browser MVP ready with HTML UI chrome");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            info!("Close requested");
            *control_flow = ControlFlow::Exit
        }
    });
}
