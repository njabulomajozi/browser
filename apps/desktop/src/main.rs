//! Browser MVP Desktop Application
//!
//! Main entry point for the browser.
//! Milestone 1.3: Servo integration with hardcoded HTML rendering
//! Milestone 1.5: Back/Forward navigation

mod history;

use eframe::egui;
use history::TabHistory;
use renderer::ServoRenderer;

fn main() -> Result<(), eframe::Error> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize rustls crypto provider for Servo
    // MUST be called before any TLS operations
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing::info!("Starting Browser MVP");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Browser MVP")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Browser MVP",
        options,
        Box::new(|cc| {
            // Set custom font and theme
            setup_custom_fonts(&cc.egui_ctx);
            setup_theme(&cc.egui_ctx);

            // Create browser app
            let app = BrowserApp::default();

            // Wire up Servo event loop waker to egui repaint
            if let Some(ref renderer) = app.renderer {
                let ctx = cc.egui_ctx.clone();
                renderer.set_waker_callback(move || {
                    ctx.request_repaint();
                });
                tracing::info!("Servo event loop waker connected to egui");
            }

            Ok(Box::new(app))
        }),
    )
}

/// Load application icon
fn load_icon() -> egui::IconData {
    // Create a simple 32x32 icon (browser icon placeholder)
    let icon_size = 32;
    let mut rgba = vec![0u8; icon_size * icon_size * 4];

    // Create a simple gradient icon
    for y in 0..icon_size {
        for x in 0..icon_size {
            let idx = (y * icon_size + x) * 4;
            // Blue gradient
            rgba[idx] = (x * 8) as u8; // R
            rgba[idx + 1] = (y * 8) as u8; // G
            rgba[idx + 2] = 200; // B
            rgba[idx + 3] = 255; // A
        }
    }

    egui::IconData {
        rgba,
        width: icon_size as u32,
        height: icon_size as u32,
    }
}

/// Set up custom fonts
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Add proportional font
    fonts.families.insert(
        egui::FontFamily::Proportional,
        vec![
            "Hack".to_owned(),
            "Ubuntu-Light".to_owned(),
            "NotoEmoji-Regular".to_owned(),
        ],
    );

    // Add monospace font
    fonts.families.insert(
        egui::FontFamily::Monospace,
        vec!["Hack".to_owned(), "Ubuntu-Light".to_owned()],
    );

    ctx.set_fonts(fonts);
}

/// Set up custom theme
fn setup_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Browser-specific styling
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8.0);

    // Make buttons slightly rounded
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
    style.visuals.widgets.active.rounding = egui::Rounding::same(4.0);

    ctx.set_style(style);
}

/// Main browser application
struct BrowserApp {
    url_input: String,
    tabs: Vec<Tab>,
    active_tab: usize,
    show_menu: bool,
    // Servo renderer (Milestone 1.3)
    renderer: Option<ServoRenderer>,
    renderer_error: Option<String>,
    // Navigation state (Milestone 1.4)
    navigation_error: Option<String>,
    last_navigation_time: std::time::Instant,
    // Visual rendering (Phase 3)
    frame_texture: Option<egui::TextureHandle>,
}

/// Represents a browser tab
#[derive(Clone)]
#[allow(dead_code)]
struct Tab {
    id: usize,
    title: String,
    url: String,
    history: TabHistory,
}

impl Default for BrowserApp {
    fn default() -> Self {
        // Try to create Servo renderer
        let (renderer, renderer_error) = match ServoRenderer::new() {
            Ok(mut r) => {
                // Test with diagnostic data: URL (red background, large text)
                // This helps verify the rendering pipeline works before testing network
                let test_url = "data:text/html,<html><body style='background:red;color:white;font-size:48px;text-align:center;padding:100px'><h1>SERVO RENDERING TEST</h1><p>If you see this in RED, Servo is working!</p></body></html>";
                tracing::info!("Testing Servo with diagnostic URL");
                match r.load_url(test_url) {
                    Ok(_) => {
                        tracing::info!(
                            "Servo renderer initialized successfully - loading diagnostic page"
                        );
                        (Some(r), None)
                    }
                    Err(e) => {
                        tracing::error!("Failed to load URL: {}", e);
                        (Some(r), Some(format!("Failed to load URL: {}", e)))
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Servo renderer not available: {}", e);
                tracing::info!("This is expected if Servo hasn't been built yet");
                (None, Some(format!("Servo not initialized: {}", e)))
            }
        };

        Self {
            url_input: String::from("https://example.com"),
            tabs: vec![Tab {
                id: 0,
                title: "New Tab".to_string(),
                url: "about:blank".to_string(),
                history: TabHistory::new(),
            }],
            active_tab: 0,
            show_menu: false,
            renderer,
            renderer_error,
            navigation_error: None,
            last_navigation_time: std::time::Instant::now(),
            frame_texture: None,
        }
    }
}

impl BrowserApp {
    /// Navigate to a URL (Milestone 1.4)
    fn navigate(&mut self, url_str: &str) {
        tracing::info!("Navigating to: {}", url_str);

        // Clear previous navigation error
        self.navigation_error = None;

        // Normalize URL - add scheme if missing
        let normalized_url = if url_str.starts_with("http://")
            || url_str.starts_with("https://")
            || url_str.starts_with("data:")
        {
            url_str.to_string()
        } else if url_str.contains('.') {
            // Looks like a domain - add https://
            format!("https://{}", url_str)
        } else if url_str.starts_with("about:") {
            url_str.to_string()
        } else {
            // Treat as search query or invalid
            format!(
                "https://www.google.com/search?q={}",
                urlencoding::encode(url_str)
            )
        };

        // Try to navigate with renderer
        if let Some(ref mut renderer) = self.renderer {
            match renderer.load_url(&normalized_url) {
                Ok(_) => {
                    tracing::info!("Navigation initiated successfully");
                    self.url_input = normalized_url.clone();
                    self.last_navigation_time = std::time::Instant::now();

                    // Update active tab and record in history (Milestone 1.5)
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        tab.url = normalized_url.clone();
                        tab.title = "Loading...".to_string();
                        // Record navigation in history
                        tab.history.push(normalized_url.clone(), Some(tab.title.clone()));
                    }
                }
                Err(e) => {
                    tracing::error!("Navigation failed: {}", e);
                    self.navigation_error = Some(format!("Navigation failed: {}", e));
                }
            }
        } else {
            self.navigation_error =
                Some("Renderer not available. Build Servo to enable navigation.".to_string());
        }
    }

    /// Reload the current page
    fn reload(&mut self) {
        if let Some(ref mut renderer) = self.renderer {
            if let Err(e) = renderer.reload() {
                tracing::error!("Reload failed: {}", e);
                self.navigation_error = Some(format!("Reload failed: {}", e));
            } else {
                self.last_navigation_time = std::time::Instant::now();
            }
        }
    }

    /// Stop loading the current page
    fn stop(&mut self) {
        if let Some(ref mut renderer) = self.renderer {
            if let Err(e) = renderer.stop() {
                tracing::error!("Stop failed: {}", e);
            }
        }
    }

    /// Go back in history (Milestone 1.5)
    fn go_back(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if let Some(entry) = tab.history.go_back() {
                let url = entry.url.clone();
                tracing::info!("Going back to: {}", url);

                // Navigate to the previous URL
                if let Some(ref mut renderer) = self.renderer {
                    match renderer.load_url(&url) {
                        Ok(_) => {
                            self.url_input = url.clone();
                            tab.url = url;
                            if let Some(title) = &entry.title {
                                tab.title = title.clone();
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to navigate back: {}", e);
                            self.navigation_error = Some(format!("Failed to go back: {}", e));
                        }
                    }
                }
            } else {
                tracing::debug!("Cannot go back - already at beginning of history");
            }
        }
    }

    /// Go forward in history (Milestone 1.5)
    fn go_forward(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if let Some(entry) = tab.history.go_forward() {
                let url = entry.url.clone();
                tracing::info!("Going forward to: {}", url);

                // Navigate to the next URL
                if let Some(ref mut renderer) = self.renderer {
                    match renderer.load_url(&url) {
                        Ok(_) => {
                            self.url_input = url.clone();
                            tab.url = url;
                            if let Some(title) = &entry.title {
                                tab.title = title.clone();
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to navigate forward: {}", e);
                            self.navigation_error = Some(format!("Failed to go forward: {}", e));
                        }
                    }
                }
            } else {
                tracing::debug!("Cannot go forward - already at end of history");
            }
        }
    }

    /// Check if can go back (for UI state)
    fn can_go_back(&self) -> bool {
        self.tabs.get(self.active_tab)
            .map(|tab| tab.history.can_go_back())
            .unwrap_or(false)
    }

    /// Check if can go forward (for UI state)
    fn can_go_forward(&self) -> bool {
        self.tabs.get(self.active_tab)
            .map(|tab| tab.history.can_go_forward())
            .unwrap_or(false)
    }

    /// Update tab state from renderer
    fn update_tab_from_renderer(&mut self) {
        if let Some(ref renderer) = self.renderer {
            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                // Update title from renderer
                if let Some(title) = renderer.get_title() {
                    tab.title = title;
                } else if renderer.is_loading() {
                    tab.title = "Loading...".to_string();
                }

                // Update URL from renderer
                if let Some(url) = renderer.get_url() {
                    tab.url = url.clone();
                    // Update URL bar if not loading (to avoid fighting user input)
                    if !renderer.is_loading() {
                        self.url_input = url;
                    }
                }
            }
        }
    }

    /// Render the tab bar
    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;

            // Render tabs
            for (idx, tab) in self.tabs.iter().enumerate() {
                let is_active = idx == self.active_tab;

                let tab_response = ui.selectable_label(is_active, &tab.title);

                if tab_response.clicked() {
                    self.active_tab = idx;
                }

                // Show close button on hover (disabled for now)
                if is_active && ui.button("×").clicked() {
                    tracing::info!("Close tab clicked (not implemented yet)");
                }
            }

            // New tab button
            if ui.button("+").clicked() {
                tracing::info!("New tab clicked (not implemented yet)");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Menu button
                if ui.button("☰").clicked() {
                    self.show_menu = !self.show_menu;
                }
            });
        });
    }

    /// Render the navigation toolbar
    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let is_loading = self.renderer.as_ref().is_some_and(|r| r.is_loading());

            // Back button (Milestone 1.5)
            let can_back = self.can_go_back();
            let back_btn = ui.add_enabled(can_back, egui::Button::new("←"));
            if back_btn.clicked() {
                self.go_back();
            }
            back_btn.on_hover_text(if can_back {
                "Go back (Alt+Left)"
            } else {
                "No previous page"
            });

            // Forward button (Milestone 1.5)
            let can_forward = self.can_go_forward();
            let fwd_btn = ui.add_enabled(can_forward, egui::Button::new("→"));
            if fwd_btn.clicked() {
                self.go_forward();
            }
            fwd_btn.on_hover_text(if can_forward {
                "Go forward (Alt+Right)"
            } else {
                "No next page"
            });

            // Refresh/Stop button (Milestone 1.4)
            if is_loading {
                if ui.button("✕").on_hover_text("Stop loading").clicked() {
                    self.stop();
                }
            } else if ui.button("⟳").on_hover_text("Reload").clicked() {
                self.reload();
            }

            // Home button
            let home_btn = ui.button("⌂");
            if home_btn.clicked() {
                self.navigate("https://example.com");
            }
            home_btn.on_hover_text("Home");

            // Loading spinner (Milestone 1.4)
            if is_loading {
                ui.spinner();
            }

            // URL bar (takes remaining space)
            let url_response = ui.add(
                egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(f32::INFINITY)
                    .hint_text("Enter URL or search..."),
            );

            // Navigate on Enter key (Milestone 1.4)
            if url_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let url = self.url_input.clone();
                self.navigate(&url);
            }

            // Go button (Milestone 1.4)
            if ui.button("Go").clicked() {
                let url = self.url_input.clone();
                self.navigate(&url);
            }
        });

        // Progress bar (Milestone 1.4)
        if let Some(ref renderer) = self.renderer {
            if renderer.is_loading() {
                let progress = renderer.get_progress();
                ui.add(egui::ProgressBar::new(progress).show_percentage());
            }
        }

        // Error display (Milestone 1.4)
        if let Some(error) = self.navigation_error.clone() {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::RED, "⚠");
                ui.label(&error);
                if ui.small_button("✕").clicked() {
                    self.navigation_error = None;
                }
            });
        }
    }

    /// Render the menu
    fn render_menu(&mut self, ctx: &egui::Context) {
        if !self.show_menu {
            return;
        }

        egui::Window::new("Menu")
            .collapsible(false)
            .resizable(false)
            .fixed_pos(egui::pos2(
                ctx.screen_rect().max.x - 220.0,
                ctx.screen_rect().min.y + 80.0,
            ))
            .show(ctx, |ui| {
                ui.set_width(200.0);

                ui.menu_button("File", |ui| {
                    if ui.button("New Tab").clicked() {
                        tracing::info!("File > New Tab");
                        self.show_menu = false;
                    }
                    if ui.button("New Window").clicked() {
                        tracing::info!("File > New Window");
                        self.show_menu = false;
                    }
                    ui.separator();
                    if ui.button("Close Tab").clicked() {
                        tracing::info!("File > Close Tab");
                        self.show_menu = false;
                    }
                    if ui.button("Quit").clicked() {
                        tracing::info!("File > Quit");
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Cut").clicked() {
                        tracing::info!("Edit > Cut");
                        self.show_menu = false;
                    }
                    if ui.button("Copy").clicked() {
                        tracing::info!("Edit > Copy");
                        self.show_menu = false;
                    }
                    if ui.button("Paste").clicked() {
                        tracing::info!("Edit > Paste");
                        self.show_menu = false;
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Zoom In").clicked() {
                        tracing::info!("View > Zoom In");
                        self.show_menu = false;
                    }
                    if ui.button("Zoom Out").clicked() {
                        tracing::info!("View > Zoom Out");
                        self.show_menu = false;
                    }
                    if ui.button("Reset Zoom").clicked() {
                        tracing::info!("View > Reset Zoom");
                        self.show_menu = false;
                    }
                    ui.separator();
                    if ui.button("Full Screen").clicked() {
                        tracing::info!("View > Full Screen");
                        self.show_menu = false;
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Documentation").clicked() {
                        tracing::info!("Help > Documentation");
                        self.show_menu = false;
                    }
                    if ui.button("Report Issue").clicked() {
                        tracing::info!("Help > Report Issue");
                        self.show_menu = false;
                    }
                    ui.separator();
                    if ui.button("About").clicked() {
                        tracing::info!("Help > About");
                        self.show_menu = false;
                    }
                });

                if ui.button("Close Menu").clicked() {
                    self.show_menu = false;
                }
            });
    }

    /// Render the main content area (Phase 3: Visual Rendering)
    fn render_content(&mut self, ui: &mut egui::Ui) {
        if let Some(ref mut renderer) = self.renderer {
            // Servo is initialized - render actual pixels
            match renderer.get_frame() {
                Ok(frame) => {
                    // Debug: Log frame status periodically (every 60 frames)
                    use std::sync::atomic::{AtomicU64, Ordering};
                    static FRAME_COUNT: AtomicU64 = AtomicU64::new(0);

                    let count = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
                    if count % 60 == 0 {
                        tracing::debug!(
                            "Frame #{}: {}x{}, empty={}, pixels={}",
                            count,
                            frame.width,
                            frame.height,
                            frame.is_empty(),
                            frame.pixels.len()
                        );
                    }

                    // Skip if frame is empty
                    if frame.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(100.0);
                            ui.spinner();
                            ui.add_space(10.0);
                            ui.label("Loading page...");
                            if let Some(url) = renderer.get_url() {
                                ui.label(format!("URL: {}", url));
                            }
                            ui.add_space(10.0);
                            ui.label(format!("Frame: {}x{} (all black/transparent)", frame.width, frame.height));
                        });
                        return;
                    }

                    // Convert RGBA pixel data to egui ColorImage
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [frame.width as usize, frame.height as usize],
                        &frame.pixels,
                    );

                    // Load texture (overwrites previous if exists)
                    let texture = ui.ctx().load_texture(
                        "servo_frame",
                        color_image,
                        egui::TextureOptions::LINEAR,
                    );

                    // Cache texture handle for reference
                    self.frame_texture = Some(texture.clone());

                    // Calculate available space
                    let available_size = ui.available_size();

                    // Calculate scaled size preserving aspect ratio
                    let frame_aspect = frame.width as f32 / frame.height as f32;
                    let available_aspect = available_size.x / available_size.y;

                    let display_size = if frame_aspect > available_aspect {
                        // Frame is wider - fit to width
                        egui::vec2(available_size.x, available_size.x / frame_aspect)
                    } else {
                        // Frame is taller - fit to height
                        egui::vec2(available_size.y * frame_aspect, available_size.y)
                    };

                    // Display the rendered frame
                    ui.centered_and_justified(|ui| {
                        ui.image(egui::load::SizedTexture {
                            id: texture.id(),
                            size: display_size,
                        });
                    });
                }
                Err(e) => {
                    // Frame retrieval error - show diagnostic info
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.colored_label(egui::Color32::RED, "⚠ Frame Error");
                        ui.add_space(10.0);
                        ui.label(format!("Error: {}", e));
                        ui.add_space(20.0);

                        ui.separator();
                        ui.add_space(10.0);

                        // Show renderer state for debugging
                        ui.horizontal(|ui| {
                            ui.label("Loading:");
                            ui.label(if renderer.is_loading() { "Yes" } else { "No" });
                        });

                        if let Some(title) = renderer.get_title() {
                            ui.horizontal(|ui| {
                                ui.label("Title:");
                                ui.label(title);
                            });
                        }

                        if let Some(url) = renderer.get_url() {
                            ui.horizontal(|ui| {
                                ui.label("URL:");
                                ui.label(url);
                            });
                        }
                    });
                }
            }
        } else {
            // Servo not initialized - show instructions
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("⏳ Milestone 1.3: Servo Integration");
                ui.add_space(20.0);

                if let Some(ref error) = self.renderer_error {
                    ui.colored_label(egui::Color32::YELLOW, "Servo renderer not initialized:");
                    ui.label(error);
                } else {
                    ui.label("Servo renderer is initializing...");
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                ui.label("✅ Implementation Complete:");
                ui.label("  • Renderer package structure");
                ui.label("  • Servo dependency configured");
                ui.label("  • API design and documentation");
                ui.label("  • Integration with desktop app");

                ui.add_space(10.0);
                ui.label("⏳ To complete Servo build:");
                ui.label("  1. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh");
                ui.label("  2. Install system dependencies (see packages/renderer/README.md)");
                ui.label("  3. Run: cargo build (first build: 10-30 minutes)");

                ui.add_space(10.0);
                ui.label("📖 See packages/renderer/README.md for full instructions");
            });
        }
    }

    /// Render status bar
    fn render_status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Show Servo status
            if self.renderer.is_some() {
                ui.label("✅ Servo: Initialized");
            } else {
                ui.colored_label(egui::Color32::YELLOW, "⏳ Servo: Not built");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Tabs: {}", self.tabs.len()));
                ui.separator();
                ui.label("Milestone 1.5");
            });
        });
    }
}

impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Servo event loop integration (Milestone 1.3)
        if let Some(ref mut renderer) = self.renderer {
            // Process Servo compositor events
            let should_continue = renderer.spin_event_loop();

            if !should_continue {
                tracing::warn!("Servo requested shutdown");
            }

            // Render the current frame
            renderer.paint();
        }

        // Update tab state from renderer (Milestone 1.4)
        self.update_tab_from_renderer();

        // Handle keyboard shortcuts (Milestone 1.5)
        ctx.input(|i| {
            // Alt+Left: Go back
            if i.modifiers.alt && i.key_pressed(egui::Key::ArrowLeft) && self.can_go_back() {
                self.go_back();
            }
            // Alt+Right: Go forward
            if i.modifiers.alt && i.key_pressed(egui::Key::ArrowRight) && self.can_go_forward() {
                self.go_forward();
            }
        });

        // Close menu if clicked outside
        if self.show_menu && ctx.input(|i| i.pointer.any_click()) {
            let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
            if let Some(pos) = pointer_pos {
                let menu_rect = egui::Rect::from_min_size(
                    egui::pos2(ctx.screen_rect().max.x - 220.0, 80.0),
                    egui::vec2(200.0, 300.0),
                );
                if !menu_rect.contains(pos) {
                    self.show_menu = false;
                }
            }
        }

        // Top panel: Tab bar
        egui::TopBottomPanel::top("tab_bar")
            .exact_height(32.0)
            .show(ctx, |ui| {
                self.render_tab_bar(ui);
            });

        // Second panel: Toolbar with URL bar
        egui::TopBottomPanel::top("toolbar")
            .exact_height(36.0)
            .show(ctx, |ui| {
                self.render_toolbar(ui);
            });

        // Bottom panel: Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .show(ctx, |ui| {
                self.render_status_bar(ui);
            });

        // Central panel: Content area
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_content(ui);
        });

        // Render menu (as overlay)
        self.render_menu(ctx);
    }
}
