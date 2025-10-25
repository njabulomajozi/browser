# Renderer Package Architecture

## Overview

This package provides a clean, safe API for embedding the Servo browser engine in Rust applications. It wraps Servo's complex WebView API with a simplified interface suitable for MVP browser development.

## Design Philosophy

### 1. Separation of Concerns

```
Servo (libservo) ← This Package → Browser App (egui)
   Complex API      Simple API       User Interface
```

- **Servo**: Handles all web standards (HTML, CSS, JS, DOM, networking, etc.)
- **This Package**: Provides embedder-friendly API (load_url, get_frame, etc.)
- **Browser App**: Focuses on UI chrome (tabs, URL bar, bookmarks, etc.)

### 2. Servo 2025 WebView API Integration

Based on Servo's modernized embedding API (Feb 2025), we implement:

**Delegate Pattern:**
```rust
impl WebViewDelegate for BrowserWebViewDelegate {
    fn notify_new_frame_ready(&self, webview: WebView) { ... }
    fn notify_page_title_changed(&self, webview: WebView, title: Option<String>) { ... }
    fn notify_load_status_changed(&self, webview: WebView, status: LoadStatus) { ... }
    // ... other lifecycle callbacks
}
```

**Event Loop Integration:**
```rust
// Main event loop
loop {
    renderer.spin_event_loop();  // Process compositor messages
    renderer.paint();             // Trigger WebView rendering

    let frame = renderer.get_frame()?;
    display_in_ui(frame);
}
```

**Builder Pattern:**
```rust
// Servo initialization
let servo = ServoBuilder::new(rendering_context)
    .event_loop_waker(waker)
    .build();

// WebView creation
let webview = WebViewBuilder::new(servo)
    .delegate(delegate)
    .url(url)
    .size(size)
    .build();
```

## Component Details

### ServoRenderer

**Responsibilities:**
- Initialize Servo engine (once per process)
- Create and manage WebView instances
- Coordinate rendering pipeline
- Provide pixel buffers for display
- Handle lifecycle (load, reload, stop, resize)

**State Management:**
```rust
pub struct ServoRenderer {
    // Configuration
    config: RendererConfig,
    initialized: bool,

    // Servo instances
    rendering_context: Option<Rc<SoftwareRenderingContext>>,
    servo: Option<Servo>,
    webview: Option<WebView>,

    // State synchronization
    delegate_state: DelegateState,
    waker: EguiEventLoopWaker,
}
```

**Key Methods:**
- `initialize()`: Set up Servo engine
- `load_url()`: Navigate to URL
- `get_frame()`: Read rendered pixels
- `spin_event_loop()`: Process compositor
- `paint()`: Trigger rendering

### BrowserWebViewDelegate

**Responsibilities:**
- Implement WebViewDelegate trait
- Receive Servo callbacks (load events, title changes, etc.)
- Update shared DelegateState
- Wake event loop when UI update needed

**Callback Flow:**
```
Servo (background thread)
  ↓ page load complete
WebViewDelegate::notify_load_status_changed()
  ↓ update state
DelegateState.set_loading(false)
  ↓ wake event loop
EventLoopWaker.wake()
  ↓ trigger repaint
egui::Context.request_repaint()
```

### Event LoopWaker

**Responsibilities:**
- Allow Servo to wake main event loop from background thread
- Bridge between Servo's EventLoopWaker trait and egui's repaint system

**Implementation:**
```rust
impl EventLoopWaker for EguiEventLoopWaker {
    fn wake(&self) {
        if let Some(callback) = self.callback.lock().unwrap().as_ref() {
            callback();  // Calls ctx.request_repaint()
        }
    }
}
```

### Rendering Context

**SoftwareRenderingContext** (MVP):
- CPU-based rendering (no GPU required)
- Slower but simpler to integrate
- Provides `read_to_image()` for pixel access
- Used for MVP to avoid OpenGL complexity

**Future: OffscreenRenderingContext**:
- GPU-accelerated via OpenGL
- Significantly faster (hardware compositing)
- Requires GL context management
- Planned for post-MVP performance optimization

## Rendering Pipeline

### Servo's Internal Pipeline

(Based on research from Servo Book - Layout 2020 architecture)

```
HTML/CSS Input
  ↓
1. DOM Construction
   (Parse HTML → DOM tree)
  ↓
2. Style Resolution
   (Apply CSS → Styled DOM)
  ↓
3. Layout: Box Tree Construction
   (Formatting contexts: block, inline, flex, grid)
  ↓
4. Layout: Fragment Tree Construction
   (Compute positions, handle line breaking, pagination)
  ↓
5. Display List Construction
   (WebRender commands: rectangles, text, images)
  ↓
6. Compositing
   (WebRender generates GPU commands)
  ↓
7. Rasterization
   (GPU draws pixels → framebuffer)
```

### Our Integration Points

We interact at the final output stage:

```
WebRender (Servo's compositor)
  ↓ renders to
SoftwareRenderingContext surface
  ↓ read via
read_to_image(rect) → ImageBuffer<Rgba<u8>>
  ↓ convert to
RenderedFrame { width, height, pixels: Vec<u8> }
  ↓ display in
egui ColorImage → Texture → Image widget
```

## State Synchronization

### DelegateState Pattern

Thread-safe state sharing between Servo and UI:

```rust
#[derive(Clone)]
pub struct DelegateState {
    title: Arc<Mutex<Option<String>>>,
    url: Arc<Mutex<Option<String>>>,
    is_loading: Arc<Mutex<bool>>,
    load_progress: Arc<Mutex<f32>>,
}
```

**Flow:**
1. Servo callback updates DelegateState (any thread)
2. Wake event loop
3. UI thread reads DelegateState in update()
4. Display in UI (title bar, progress bar, etc.)

## Error Handling Strategy

### Error Types

- **InitializationFailed**: Servo setup failed (missing deps, bad config)
- **WebViewCreationFailed**: Can't create WebView (not initialized, resource exhaustion)
- **LoadFailed**: URL load error (invalid URL, network error, unsupported scheme)
- **GlContextError**: Graphics driver issues (rare with software rendering)
- **NotInitialized**: Operation attempted before initialize()
- **ContextLost**: Rendering surface invalidated (resize, GPU reset)

### Recovery Strategies

Each error includes hints for recovery:

```rust
#[error("Failed to initialize Servo: {0}\n\nHint: Ensure Servo dependencies are installed. See packages/renderer/README.md")]
InitializationFailed(String),
```

- User-friendly error messages
- Actionable recovery steps
- Links to documentation

## Testing Strategy

### Unit Tests

- Component creation and initialization
- URL validation and loading
- State synchronization
- Error handling

### Integration Tests

- End-to-end page loading
- Multi-step navigation
- Resize handling
- Concurrent operations

### Known Limitations

- **Test Parallelism**: Servo's global state means tests must run serially:
  ```bash
  cargo test -- --test-threads=1
  ```
- This doesn't affect production (one initialization per process)

## Performance Considerations

### Current (MVP)

- **SoftwareRenderingContext**: CPU rendering
- **Expected**: ~30-60 FPS for simple pages
- **Memory**: ~200-500 MB per WebView

### Future Optimizations

1. **GPU Rendering**: Switch to OffscreenRenderingContext
   - Expected: 60+ FPS for complex pages
   - Hardware-accelerated compositing

2. **Damage Tracking**: Only repaint changed regions
   - Reduce CPU usage when page is static

3. **Resource Caching**: Cache decoded images, fonts
   - Reduce memory allocations

4. **Tab Sleeping**: Unload inactive WebViews
   - Lower memory footprint for many tabs

## Future Enhancements

### Milestone 2.1: Multi-Tab Process Isolation

- One Servo process per tab
- IPC for cross-process communication
- Crash isolation (one tab crash doesn't kill browser)

### Milestone 3.x: Developer Tools

- Implement DevTools Protocol
- Console message capture
- Network request inspection
- DOM tree visualization

### Phase 4: Extensions API

- WebExtensions compatibility
- Extension sandboxing
- Permissions system

## References

- [Servo Book - Compositor Architecture](https://book.servo.org/architecture/)
- [Servo WebView API (2025)](https://servo.org/blog/2025/02/19/this-month-in-servo/)
- [WebRender Architecture](https://github.com/servo/webrender)
- [Layout 2020 Design](https://book.servo.org/architecture/layout.html)
