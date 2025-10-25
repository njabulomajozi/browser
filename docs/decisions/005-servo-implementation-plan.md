# Servo Implementation Plan

**Date**: 2025-10-25
**Status**: In Progress
**Milestone**: 1.3 - Servo Integration

---

## Overview

Based on official libservo API documentation (https://doc.servo.org/servo/), this document outlines the implementation steps for integrating Servo into our browser.

---

## API Summary

### Core Types

**Servo** - Main engine instance
- Created via `ServoBuilder::new(rendering_context).build()`
- Methods: `spin_event_loop()`, `get_webview_handle()`, `set_delegate()`

**WebView** - Handle to a webview instance
- Created via `WebViewBuilder::new(&servo).url(url).build()`
- Methods: `load(url)`, `paint()`, `resize()`, `reload()`

**RenderingContext** - Trait for OpenGL context management
- Required methods: `size()`, `resize()`, `present()`, `make_current()`, `gleam_gl_api()`, `glow_gl_api()`

**EventLoopWaker** - Trait for cross-thread event loop waking
- Required: `clone_box() -> Box<dyn EventLoopWaker>`
- Optional: `wake()`

**WebViewDelegate** - Callbacks for webview events
- Methods: TBD (need to fetch documentation)

---

## Implementation Steps

### Step 1: Implement EventLoopWaker ✅

**File**: `packages/renderer/src/waker.rs`

**Implementation**:
```rust
use libservo::EventLoopWaker;

impl EventLoopWaker for EguiEventLoopWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }

    fn wake(&self) {
        // Invoke registered callback
        if let Some(ref callback) = *self.callback.lock().unwrap() {
            callback();
        }
    }
}
```

**Integration with egui**:
- Use `egui::Context::request_repaint()` as the wake callback
- This ensures Servo rendering triggers UI updates

---

### Step 2: Implement RenderingContext

**File**: `packages/renderer/src/rendering_context.rs` (new)

**Options**:

1. **WindowRenderingContext** - Native OpenGL window
   - Use `surfman` to create GL context
   - Requires `winit::Window`

2. **OffscreenRenderingContext** - Render to texture
   - Better for embedding in egui
   - Can copy rendered frame to egui texture

3. **SoftwareRenderingContext** - CPU rendering (fallback)
   - Slower but no GPU required

**Recommended**: Start with `OffscreenRenderingContext` for egui integration.

**Implementation**:
```rust
use libservo::RenderingContext;
use gleam::gl::Gl;
use glow::HasContext;

struct EguiRenderingContext {
    gl: Rc<dyn Gl>,
    glow_context: Rc<glow::Context>,
    size: PhysicalSize<u32>,
    // Framebuffer and texture IDs
}

impl RenderingContext for EguiRenderingContext {
    fn size(&self) -> Size2D<i32, DevicePixel> {
        Size2D::new(self.size.width as i32, self.size.height as i32)
    }

    fn resize(&mut self, new_size: Size2D<i32, DevicePixel>) {
        self.size = PhysicalSize::new(new_size.width as u32, new_size.height as u32);
        // Recreate framebuffer with new size
    }

    fn present(&mut self) {
        // Swap buffers or copy to texture
    }

    fn make_current(&mut self) {
        // Bind GL context
    }

    fn gleam_gl_api(&self) -> Rc<dyn Gl> {
        self.gl.clone()
    }

    fn glow_gl_api(&self) -> Rc<glow::Context> {
        self.glow_context.clone()
    }
}
```

---

### Step 3: Implement WebViewDelegate

**File**: `packages/renderer/src/delegate.rs` (update existing)

**Required Methods** (from Servo API):
- `notify_new_frame_ready()` - Frame ready to paint
- `on_load_start()` - Navigation started
- `on_load_complete()` - Navigation finished
- `on_title_changed()` - Page title updated
- `on_url_changed()` - URL changed

**Implementation**:
```rust
use libservo::WebViewDelegate;

impl WebViewDelegate for BrowserWebViewDelegate {
    fn notify_new_frame_ready(&self, webview_id: WebViewId) {
        // Update DelegateState
        self.state.set_loading(false);
        // Wake event loop to trigger repaint
        self.waker.wake();
    }

    fn on_load_start(&self, webview_id: WebViewId, url: ServoUrl) {
        self.state.set_loading(true);
        self.state.set_url(Some(url.to_string()));
        self.state.set_progress(0.1);
    }

    fn on_load_complete(&self, webview_id: WebViewId, load_data: LoadData) {
        self.state.set_loading(false);
        self.state.set_progress(1.0);
    }

    fn on_title_changed(&self, webview_id: WebViewId, title: Option<String>) {
        self.state.set_title(title);
    }

    // ... other methods
}
```

---

### Step 4: Initialize Servo in ServoRenderer

**File**: `packages/renderer/src/servo_wrapper.rs`

**Implementation**:
```rust
pub struct ServoRenderer {
    config: RendererConfig,
    delegate_state: DelegateState,
    waker: EguiEventLoopWaker,

    // Servo instances
    servo: Option<Servo>,
    webview: Option<WebView>,
    rendering_context: Rc<dyn RenderingContext>,
}

impl ServoRenderer {
    pub fn initialize(&mut self) -> Result<()> {
        // 1. Create rendering context
        let rendering_context = Rc::new(EguiRenderingContext::new(
            self.config.width,
            self.config.height,
        )?);

        // 2. Create EventLoopWaker
        let waker = Box::new(self.waker.clone());

        // 3. Build Servo
        let servo = ServoBuilder::new(rendering_context.clone())
            .event_loop_waker(waker)
            .build();

        // 4. Set delegate
        let delegate = Rc::new(BrowserWebViewDelegate::new(
            self.delegate_state.clone(),
            self.waker.clone(),
        ));
        servo.set_delegate(delegate);

        self.servo = Some(servo);
        self.rendering_context = rendering_context;
        self.initialized = true;

        Ok(())
    }
}
```

---

### Step 5: Create WebView and Load URLs

**Implementation**:
```rust
pub fn load_url(&mut self, url_str: &str) -> Result<()> {
    if !self.initialized {
        self.initialize()?;
    }

    let url = Url::parse(url_str)?;
    let servo = self.servo.as_ref().unwrap();

    // Create WebView if not exists
    if self.webview.is_none() {
        let delegate = Rc::new(BrowserWebViewDelegate::new(
            self.delegate_state.clone(),
            self.waker.clone(),
        ));

        let webview = WebViewBuilder::new(servo)
            .delegate(delegate)
            .url(url.clone())
            .size(PhysicalSize::new(self.config.width, self.config.height))
            .hidpi_scale_factor(self.config.device_pixel_ratio)
            .build();

        self.webview = Some(webview);
    } else {
        // Load URL in existing WebView
        self.webview.as_ref().unwrap().load(url);
    }

    Ok(())
}
```

---

### Step 6: Event Loop Integration

**Implementation**:
```rust
pub fn spin_event_loop(&mut self) -> bool {
    if let Some(ref mut servo) = self.servo {
        servo.spin_event_loop()
    } else {
        true
    }
}

pub fn paint(&mut self) {
    if let Some(ref webview) = self.webview {
        webview.paint();
        self.rendering_context.present();
    }
}
```

**Desktop app integration** (`apps/desktop/src/main.rs`):
```rust
impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Spin Servo event loop
        if let Some(ref mut renderer) = self.renderer {
            renderer.spin_event_loop();
        }

        // 2. Render UI
        self.render_toolbar(ctx);
        self.render_content(ctx);

        // 3. Request repaint for next frame
        ctx.request_repaint();
    }
}
```

---

## Build Requirements

### System Dependencies

**macOS**:
```bash
brew install cmake pkg-config
```

**Linux**:
```bash
sudo apt install libx11-dev libxcb-dev libssl-dev cmake pkg-config
```

**Windows**:
- Visual Studio Build Tools 2019+
- CMake

### Rust Dependencies

Already added to `packages/renderer/Cargo.toml`:
- `libservo = { git = "https://github.com/servo/servo", branch = "main" }`
- `surfman = "0.9"`
- `gleam = "0.15"`
- `euclid = "0.22"`

---

## Testing Strategy

### Phase 1: Smoke Test
1. Initialize Servo without crashing
2. Create WebView
3. Load `data:text/html,<h1>Hello</h1>`
4. Verify frame rendering

### Phase 2: URL Navigation
1. Load `http://example.com`
2. Verify delegate callbacks fire
3. Check rendered output
4. Test navigation (back/forward)

### Phase 3: Integration
1. Multiple tabs with separate WebViews
2. Tab switching
3. Concurrent page loads
4. Memory leak testing

---

## Known Challenges

### Challenge 1: RenderingContext Complexity
- Servo expects OpenGL context
- egui uses `wgpu` or `glow` backend
- **Solution**: Use `egui_glow` to get OpenGL context, create offscreen framebuffer for Servo

### Challenge 2: Thread Safety
- Servo's types are `!Send`, `!Sync`
- Must stay on main thread
- **Solution**: All Servo operations in main thread, use channels for cross-thread communication

### Challenge 3: Event Loop Integration
- Servo has own event loop via `spin_event_loop()`
- egui has frame-based updates
- **Solution**: Call `spin_event_loop()` in every `update()` call

---

## Next Steps

1. ✅ Add libservo dependency
2. ✅ Fetch Servo dependencies
3. ⏳ Wait for Servo compilation (in progress)
4. ⏳ Implement EventLoopWaker trait
5. ⏳ Implement RenderingContext
6. ⏳ Wire up ServoBuilder
7. ⏳ Test basic initialization
8. ⏳ Load first URL

---

## References

- Servo API Docs: https://doc.servo.org/servo/
- Official Embedding Example: https://github.com/paulrouget/servo-embedding-example
- Servo Book: https://book.servo.org/
- Decision 004: Servo Integration Investigation
