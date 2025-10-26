# Renderer Package

This package wraps the Servo rendering engine and provides a simplified API for embedding in the browser application.

## Status: Milestone 1.3 - Initial Integration

### Completed

- ✅ Package structure created
- ✅ Error types defined
- ✅ EventLoopWaker implemented
- ✅ WebViewDelegate pattern established
- ✅ ServoRenderer wrapper with MVP API
- ✅ Hardcoded HTML rendering method
- ✅ Comprehensive documentation

### Pending

- ⏳ Rust toolchain installation required
- ⏳ Servo build (10-30 minutes first build)
- ⏳ Complete Servo API integration (marked with `TODO` comments)
- ⏳ OpenGL context setup
- ⏳ Actual frame capture from Servo

## System Requirements

### Rust Toolchain

Servo requires **Rust nightly 1.86+**. Install via rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup install nightly
```

### Platform Dependencies

**macOS:**
```bash
brew install cmake pkg-config
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install -y \
  libx11-dev libxcb-dev libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libssl-dev pkg-config cmake python3 \
  libfontconfig1-dev libfreetype6-dev libharfbuzz-dev \
  libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev
```

**Windows:**
- Install Visual Studio Build Tools
- Install CMake and add to PATH

## Architecture

```
packages/renderer/
├── src/
│   ├── lib.rs              // Public API and error types
│   ├── servo_wrapper.rs    // Main ServoRenderer implementation
│   ├── delegate.rs         // WebViewDelegate callbacks
│   ├── waker.rs            // EventLoopWaker for Servo→main thread signaling
│   └── types.rs            // Shared types (RenderedFrame, Config)
└── README.md               // This file
```

## API Usage

### Basic Example

```rust
use renderer::{ServoRenderer, Result};

fn main() -> Result<()> {
    // Create renderer with default config (1024x768)
    let mut renderer = ServoRenderer::new()?;

    // Initialize Servo engine
    renderer.initialize()?;

    // Load hardcoded HTML (Milestone 1.3)
    renderer.load_html("<h1>Hello, Servo!</h1>")?;

    // In your render loop:
    loop {
        // Process Servo events
        renderer.update()?;

        // Get rendered frame
        let frame = renderer.get_frame()?;

        // Display frame in your UI (egui, etc.)
        display_frame(frame);
    }

    Ok(())
}
```

### With Custom Configuration

```rust
use renderer::{ServoRenderer, RendererConfig};

let config = RendererConfig {
    width: 1920,
    height: 1080,
    device_pixel_ratio: 2.0,  // For HiDPI displays
    enable_webgl: true,
    enable_javascript: true,
};

let mut renderer = ServoRenderer::with_config(config)?;
```

## Integration with egui

The renderer is designed to integrate with egui. See `apps/desktop/` for full example:

```rust
impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Render UI chrome (tabs, URL bar, etc.)
        self.render_ui(ctx);

        // Get frame from Servo
        if let Ok(frame) = self.renderer.get_frame() {
            // Convert to egui texture
            let texture = ctx.load_texture(
                "servo_frame",
                egui::ColorImage::from_rgba_unmultiplied(
                    [frame.width as usize, frame.height as usize],
                    &frame.pixels,
                ),
                Default::default(),
            );

            // Display in central panel
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.image(&texture);
            });
        }

        // Request repaint for animations
        ctx.request_repaint();
    }
}
```

## Building

### First Time Build (Expected: 10-30 minutes)

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system dependencies (see above)

# Build renderer package
cargo build --package renderer

# This will:
# 1. Clone Servo from git
# 2. Build Servo (slow first time, ~10-30 min)
# 3. Build renderer wrapper
```

### Subsequent Builds

After the first build, incremental builds are much faster (<1 minute).

## TODO: Completing Servo Integration

The implementation has `TODO` comments marking areas that need Servo API integration:

###servo_wrapper.rs

1. **Initialize Servo** (line ~70):
   ```rust
   let servo = servo::ServoBuilder::new()
       .with_size(config.width, config.height)
       .with_device_pixel_ratio(config.device_pixel_ratio)
       .build()?;
   ```

2. **Create WebView** (line ~95):
   ```rust
   let delegate = Rc::new(BrowserWebViewDelegate::new());
   let webview = servo::WebViewBuilder::new(&servo)
       .delegate(delegate)
       .size(PhysicalSize::new(config.width, config.height))
       .build()?;
   ```

3. **Load URL** (line ~130):
   ```rust
   let url = ServoUrl::parse(&data_url)?;
   webview.load_url(url);
   ```

4. **Get Frame** (line ~160):
   ```rust
   let frame_data = webview.get_frame_buffer();
   ```

### delegate.rs

Implement actual Servo traits:
```rust
impl servo::WebViewDelegate for BrowserWebViewDelegate {
    fn on_load_start(&self, webview_id: WebViewId, url: ServoUrl) {
        self.on_load_start(url.as_str());
    }
    // ... other trait methods
}
```

### waker.rs

Implement Servo's EventLoopWaker trait:
```rust
impl servo::compositing::windowing::EventLoopWaker for EguiEventLoopWaker {
    fn wake(&self) {
        self.wake();
    }

    fn clone_box(&self) -> Box<dyn servo::compositing::windowing::EventLoopWaker> {
        Box::new(self.clone())
    }
}
```

## Known Limitations (MVP)

### Milestone 1.3
- ✅ Hardcoded HTML only (data: URLs)
- ⏳ No URL navigation yet (comes in Milestone 1.4)
- ⏳ Single-process (multi-process in Milestone 2.1)
- ⏳ No JavaScript console yet
- ⏳ No network requests yet

### Servo Build Issues

If Servo fails to build:

1. **Check system dependencies**: Ensure all platform dependencies installed
2. **Check Rust version**: `rustc --version` should be 1.86+ nightly
3. **Clean build**: `cargo clean && cargo build`
4. **Check Servo issues**: https://github.com/servo/servo/issues

Common errors:
- **Missing libX11**: Install X11 development headers
- **Python not found**: Install Python 3.8+
- **Linker errors**: Install build-essential/Xcode Command Line Tools

## Development Workflow

```bash
# Check code quality
cargo check --package renderer
cargo clippy --package renderer -- -D warnings

# Run tests
cargo test --package renderer

# Format code
cargo fmt --all

# Build in release mode (optimized)
cargo build --package renderer --release
```

## References

- [Servo Book](https://book.servo.org/)
- [Servo API Docs](https://doc.servo.org/servo/)
- [Servo WebViewBuilder](https://doc.servo.org/servo/struct.WebViewBuilder.html)
- [Servo GitHub](https://github.com/servo/servo)

## Milestone Roadmap

- **Milestone 1.3 (Current)**: Initial Servo integration, hardcoded HTML ✅
- **Milestone 1.4**: URL navigation, network requests
- **Milestone 1.5**: Back/forward history
- **Milestone 2.1**: Multi-process architecture, tab isolation

---

**Last Updated**: 2025-10-25
**Status**: Initial implementation complete, awaiting Servo build
