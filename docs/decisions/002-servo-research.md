# Servo Embedding Research

**Date**: 2025-10-25
**Status**: 📚 Research Complete
**Next**: Milestone 1.3 implementation

---

## Overview

Servo is a modern, parallel browser engine written in Rust. Developed by Mozilla Research, it prioritizes performance, parallelism, and memory safety.

---

## Key Findings

### 1. Embedding API

**Status**: Servo provides embedding capabilities but documentation is sparse.

**Best Resources**:
- Official examples: https://github.com/servo/servo/tree/main/ports
- `servo/ports/servoshell` - Reference implementation
- Community discussions on Servo Zulip chat

**API Approach**:
- Servo uses WindowMethods trait for integration
- Events passed via WindowEvent enum
- Rendering output via compositing layer

### 2. Architecture

**Process Model**:
```
Main Process (Our Browser)
    ↓
Servo Engine (Embedded)
    ├── Script Task (JS execution)
    ├── Layout Task (CSS, layout)
    ├── Paint Task (rendering)
    └── Constellation (coordination)
```

**For MVP**: Single-process embedding
**Future**: Multi-process with IPC

### 3. Rendering Pipeline

```
URL Load → HTML Parser → DOM Tree → Style Resolution
    → Layout → Paint → Composite → Display
```

**Integration Points**:
1. **Input**: Send WindowEvent::LoadUrl
2. **Output**: Receive rendered frames via compositing
3. **Display**: Blit to our egui canvas

### 4. Dependencies

**Servo Requires**:
- Rust 1.70+ (same as our project ✅)
- CMake, Python 3 (build tools)
- System libraries:
  - Linux: X11, fontconfig, freetype
  - macOS: Core frameworks (auto-linked)
  - Windows: MSVC toolchain

**Build Time**: First build ~20-30 minutes
**Binary Size**: ~100MB unoptimized, ~50MB with LTO

### 5. Web Standards Support

**Well Supported**:
- ✅ HTML5 (parser, DOM APIs)
- ✅ CSS3 (Grid, Flexbox, animations)
- ✅ JavaScript (ES2024 via SpiderMonkey)
- ✅ WebGL (hardware-accelerated)
- ✅ WebAssembly

**Limited/Missing**:
- ⚠️ WebRTC (experimental)
- ⚠️ Service Workers (in progress)
- ❌ Extensions API (not planned)
- ❌ DRM/EME (won't support)

**For MVP**: Core features well-supported. Advanced APIs not critical.

---

## Code Patterns

### Basic Embedding (Pseudocode)

```rust
use servo::compositing::windowing::{WindowMethods, WindowEvent};
use servo::servo_url::ServoUrl;

pub struct ServoRenderer {
    servo: Servo<MyWindow>,
}

impl ServoRenderer {
    pub fn new() -> Self {
        // Initialize Servo with our window
        let servo = Servo::new(MyWindow::new());
        Self { servo }
    }

    pub fn load_url(&mut self, url: &str) {
        let servo_url = ServoUrl::parse(url).unwrap();
        self.servo.handle_events(vec![WindowEvent::LoadUrl(servo_url)]);
    }

    pub fn get_frame(&mut self) -> RenderedFrame {
        // Get next frame from compositor
        self.servo.get_next_frame()
    }
}

// Implement WindowMethods for integration
impl WindowMethods for MyWindow {
    fn present(&self) {
        // Called when Servo has a frame ready
    }

    fn prepare_for_composite(&self) -> bool {
        // Called before compositing
        true
    }

    // ... more required methods
}
```

### Event Handling

```rust
// User clicks link → notify Servo
self.servo.handle_events(vec![
    WindowEvent::MouseWindowEventClass(MouseWindowEvent::Click(
        MouseButton::Left,
        Point2D::new(x, y),
    ))
]);

// User types in form
self.servo.handle_events(vec![
    WindowEvent::KeyEvent(KeyboardEvent { ... })
]);

// Navigation controls
self.servo.handle_events(vec![
    WindowEvent::Reload,          // Refresh
    WindowEvent::Navigation(NavigationType::Back),  // Back
]);
```

---

## Challenges Identified

### 1. Documentation Gap
**Issue**: Servo embedding docs are minimal
**Mitigation**: Study servoshell source, join Servo Zulip, contribute docs upstream

### 2. Build Complexity
**Issue**: Large dependency tree, long build times
**Mitigation**: CI caching, incremental builds, document setup carefully

### 3. Platform Differences
**Issue**: Different windowing systems (X11/Wayland/Win32/Cocoa)
**Mitigation**: Use winit abstraction, test on all platforms via CI

### 4. Stability
**Issue**: Servo is pre-1.0, APIs may change
**Mitigation**: Pin to specific git commit, test updates before upgrading

---

## Integration Plan

### Milestone 1.3: Initial Integration

1. Add Servo as git dependency
2. Create packages/renderer with Servo wrapper
3. Implement WindowMethods trait
4. Render hardcoded HTML first
5. Display in egui window

### Milestone 1.4: URL Navigation

1. Wire URL bar to Servo LoadUrl
2. Handle navigation events
3. Update UI on load complete
4. Error handling (404, network failures)

### Milestone 1.5: History & Navigation

1. Intercept navigation events
2. Build history stack
3. Implement back/forward

---

## Performance Considerations

### Memory Usage

**Servo Baseline**: ~50-100MB
**Per Tab**: ~20-50MB (depends on page)
**Our Target**: <500MB for 10 tabs

**Strategies**:
- Tab sleeping (unload inactive tabs)
- Lazy image loading
- DOM node limits

### Rendering Performance

**Servo Uses**:
- Parallel layout (Rayon)
- GPU compositing (WebRender)
- Incremental rendering

**Our Optimizations**:
- Request repaints only on changes
- Cache rendered frames when possible
- Limit frame rate to 60 FPS

---

## Security Model

**Servo Provides**:
- Memory safety via Rust
- Sandboxed script execution (SpiderMonkey)
- Content Security Policy support

**We Add**:
- Process isolation (future)
- HTTPS enforcement
- Cookie management
- Privacy controls

---

## Testing Strategy

### Unit Tests
- Mock Servo events
- Test state transitions
- Verify error handling

### Integration Tests
- Load real websites
- Test navigation flow
- Verify rendering output

### Target Sites for Testing
1. Simple: http://example.com
2. Modern: https://github.com
3. Complex: https://wikipedia.org

---

## References

### Official Resources
- Servo Repo: https://github.com/servo/servo
- Servo Wiki: https://github.com/servo/servo/wiki
- Servo Blog: https://servo.org/blog/
- Zulip Chat: https://servo.zulipchat.com/

### Examples & Ports
- servoshell: Main embedding example
- WebView: Android embedding
- Gonk: B2G/Firefox OS port

### Related Projects
- SpiderMonkey: https://spidermonkey.dev/ (JS engine)
- WebRender: https://github.com/servo/webrender (GPU renderer)
- Stylo: Servo's CSS engine (used in Firefox)

---

## Next Steps

1. ✅ Research complete
2. ✅ Milestone 1.2: Basic UI complete
3. ⏳ Milestone 1.3: Begin Servo integration
4. ⏳ Document integration challenges as we encounter them
5. ⏳ Contribute docs back to Servo project

---

## Questions for Servo Community

1. Best practices for embedding in 2025?
2. Recommended git commit/tag for stability?
3. Memory optimization tips for multi-tab browser?
4. Known platform-specific issues?

**Where to ask**: Servo Zulip #embedding channel
