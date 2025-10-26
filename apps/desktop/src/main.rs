//! Browser MVP - Desktop Application
//!
//! Working browser using tao + wry (proven pattern from Tauri)
//! Note: tao is Tauri's fork of winit, designed for wry compatibility

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

    info!("Starting Browser MVP (tao + wry)");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Browser MVP - tao + wry")
        .build(&event_loop)?;

    let _webview = WebViewBuilder::new()
        .with_url("https://example.com")
        .with_devtools(cfg!(debug_assertions))
        .build(&window)?;

    info!("✅ Browser MVP ready - Rendering https://example.com");

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
