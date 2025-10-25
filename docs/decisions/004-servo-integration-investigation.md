# Servo Integration Investigation

**Date**: 2025-10-25
**Status**: Investigation Complete
**Decision**: Deferred - Continue Browser Features in Parallel

---

## Problem Statement

Milestone 1.4 implemented URL navigation architecture, but actual HTTP requests and page rendering not working. Root cause: Servo rendering engine not integrated.

**Current State**:
- ✅ URL validation (parse, scheme check)
- ✅ Navigation UI (spinner, progress bar, error display)
- ✅ State management (loading flag, URL/title tracking)
- ❌ **Real HTTP requests to URLs**
- ❌ **Actual page rendering from network**
- ❌ **DOM parsing and display**

---

## Investigation Findings

### Servo is NOT a Crate

**Discovery**: Servo is not published to crates.io and cannot be added via standard Cargo dependency.

**servoshell Cargo.toml (Line 68)**:
```toml
libservo = { path = "../../components/servo", features = [...] }
```

Servo exists as:
1. **Workspace member** in Servo monorepo (`components/servo/`)
2. **Local path dependency** - not a git or registry dependency
3. **Not published** - no `servo` crate on crates.io

### Servo Build Structure

**From investigation of `/tmp/servo/`**:

```
servo/
├── components/
│   ├── servo/          # Main Servo library (libservo)
│   ├── servo_arc/
│   ├── servo_config/
│   └── ... (100+ component crates)
│
├── ports/
│   ├── servoshell/     # Desktop browser reference implementation
│   │   ├── desktop/
│   │   │   ├── app.rs           # Servo initialization
│   │   │   ├── app_state.rs     # WebView creation
│   │   │   └── minibrowser.rs   # Minimal browser UI
│   │   └── Cargo.toml
│   └── ...
│
└── Cargo.toml (workspace root)
```

**Build Requirements**:
- 128,688 files (!)
- Full Cargo workspace with 100+ crates
- 10-30 minute initial build
- Custom build system (`./mach build`)

---

## Servo API Patterns (Discovered)

### 1. Servo Initialization

**Source**: `ports/servoshell/desktop/app.rs:150-165`

```rust
use servo::{Servo, ServoBuilder};

let servo = ServoBuilder::new(window.rendering_context())
    .opts(opts)                          // Config options
    .preferences(preferences)            // User preferences
    .user_content_manager(ucm)           // User scripts
    .protocol_registry(registry)         // Custom protocols
    .event_loop_waker(waker)             // Event loop integration
    .build();

servo.setup_logging();
servo.set_delegate(Rc::new(delegate));
```

**Key Points**:
- `ServoBuilder` - Builder pattern API
- Requires `RenderingContext` (GL/OpenGL context)
- Event loop waker for compositor thread communication
- Delegate pattern for callbacks

### 2. WebView Creation & URL Loading

**Source**: `ports/servoshell/desktop/app_state.rs:152-161`

```rust
use servo::{WebView, WebViewBuilder};

let webview = WebViewBuilder::new(&servo)
    .url(url)                            // Initial URL to load
    .hidpi_scale_factor(scale)           // DPI scaling
    .delegate(delegate)                  // WebView-specific delegate
    .build();

webview.notify_theme_change(theme);
webview.focus_and_raise_to_top(true);
webview.paint();                         // Render frame
```

**Key Points**:
- `WebViewBuilder` - Creates individual web views
- `url()` - Automatically loads URL on creation
- `delegate` - Implements `WebViewDelegate` trait
- Multiple webviews per Servo instance (multi-tab)

### 3. Delegate Pattern

**Source**: `ports/servoshell/desktop/app_state.rs`

```rust
impl WebViewDelegate for RunningAppState {
    fn on_load_start(&self, webview_id: WebViewId, url: ServoUrl) {
        // Called when navigation starts
    }

    fn on_load_complete(&self, webview_id: WebViewId, load_data: LoadData) {
        // Called when page finishes loading
    }

    fn on_title_changed(&self, webview_id: WebViewId, title: Option<String>) {
        // Update tab title
    }

    // ... more delegate methods
}
```

### 4. Event Loop Integration

**Source**: `ports/servoshell/desktop/app.rs:210-237`

```rust
// Main loop pattern
loop {
    // 1. Handle winit events
    event_loop.poll_events(|event| {
        app.handle_events(event);
    });

    // 2. Spin Servo event loop
    if !servo.spin_event_loop() {
        break; // Shutdown requested
    }

    // 3. Repaint if needed
    if needs_repaint {
        webview.paint();
        rendering_context.present();
    }
}
```

---

## UPDATED: Correct Integration Approach (Based on Official Docs)

**Research Update (2025-10-25)**: After reviewing official Servo documentation (https://book.servo.org/, https://servo.org/blog/, and official embedding example), the correct approach is:

### ✅ Official Method: Git Dependency (RECOMMENDED)

**Approach**: Add libservo as Git dependency in Cargo.toml

**Source**: Official embedding example (https://github.com/paulrouget/servo-embedding-example)

**Implementation**:
```toml
[dependencies]
libservo = { git = "https://github.com/servo/servo" }
# OR pin to specific commit:
# libservo = { git = "https://github.com/servo/servo", rev = "COMMIT_HASH" }
# OR use tagged release:
# libservo = { git = "https://github.com/servo/servo", tag = "v0.0.1" }
```

**Pros**:
- ✅ Official recommended approach
- ✅ Simple setup (one line in Cargo.toml)
- ✅ Cargo handles everything automatically
- ✅ No git submodule complexity
- ✅ Matches official embedding example
- ✅ New WebView API (2025) - "cuts code from 200 lines to under 50"

**Cons**:
- Cargo downloads full Servo repo (large but one-time)
- First build takes 10-30 minutes
- Requires system dependencies (OpenGL, etc.)

**Why NOT Submodules?**
- Git submodules add unnecessary complexity
- Official examples use direct Git dependencies
- Cargo handles versioning and caching
- Submodules only needed if modifying Servo source

**Servo 0.0.1 Release (Oct 20, 2025)**:
- First official tagged release
- Monthly releases planned
- NOT published on crates.io (explicit decision)
- Must use Git dependency as shown above

**Estimated Timeline**: 1-2 weeks
- Days 1-2: Add dependency, resolve system deps, initial build
- Days 3-5: Implement RenderingContext and EventLoopWaker
- Days 6-8: Wire ServoBuilder and WebViewBuilder
- Days 9-10: Implement WebViewDelegate, test with real URLs

---

## New WebView API (2025 Improvements)

Servo made major embedding improvements in 2025:

**January 2025**: Handle-based WebView API
- Lifetime of handles controls webview lifetime
- Embedder has full control over creation/destruction
- Much simpler than old API

**February 2025**: WebViewDelegate and ServoDelegate
- Event loop calls delegate methods on events
- Clean callback interface for navigation, title changes, etc.

**April 2025**: WebViewBuilder pattern
- Builder pattern for WebView configuration
- Configure size, HiDPI scaling, initial URL, etc.

**Result**: Code reduced from ~200 lines to <50 lines for embedding

---

## Alternative Options (NOT Recommended)

### Option A: Git Submodule + Path Dependency

**Approach**: Clone Servo as submodule, reference with `path = "..."`

**When to use**: Only if you need to modify Servo source code

**Cons**: Unnecessary complexity for standard embedding

---

### Option B: Wait for crates.io Publication

**Status**: ❌ Not happening
- Servo team explicitly stated "no plans to publish on crates.io"
- GitHub releases only (binaries for end users)
- Git dependency is the official method

---

## UPDATED: Revised Decision - Direct Integration

**Based on official documentation research**, the integration is simpler than initially thought.

### ✅ Recommended Approach: Immediate Servo Integration

**Why Now (Not Deferred)**:
- Official Git dependency approach is straightforward
- New WebView API (2025) dramatically simplified embedding
- No submodule complexity needed
- Milestone 1.4 already has navigation architecture ready

**Implementation Plan** (1-2 weeks):

**Week 1: Core Integration**
1. Add `libservo = { git = "..." }` to packages/renderer/Cargo.toml
2. Install system dependencies (already done: OpenGL, etc.)
3. Implement RenderingContext for egui_glow integration
4. Implement EventLoopWaker for cross-thread communication
5. Initial build and dependency resolution

**Week 2: WebView API Wiring**
6. Initialize Servo with ServoBuilder in ServoRenderer::initialize()
7. Create WebView with WebViewBuilder in load_url()
8. Implement WebViewDelegate for navigation callbacks
9. Wire event loop with servo.spin_event_loop()
10. Test with real URLs (example.com, wikipedia.org)

### Alternative: Two-Track Approach (If Blocked)

**IF** Servo integration encounters blockers (build failures, API issues):

**Track 1**: Continue with Milestones 1.5-1.6
- Back/Forward Navigation (works with current simulated renderer)
- Persistent Storage (independent of rendering)

**Track 2**: Debug Servo integration in parallel
- Engage Servo community (Zulip, GitHub discussions)
- Test minimal embedding example separately
- Document blockers and workarounds

**However**: Given the official embedding example and improved 2025 API, direct integration should succeed.

---

## Rendering Context Challenge

**Problem**: Servo requires OpenGL rendering context, but we use egui (immediate mode GUI).

**servoshell approach**:
```rust
use surfman::{Connection, ContextAttribut es, GLVersion};
use winit::window::Window;

// Create GL context with surfman
let connection = Connection::new()?;
let adapter = connection.create_adapter()?;
let context = connection.create_context(&adapter, &attributes)?;
```

**Our options**:
1. **egui_glow** - OpenGL backend for egui (matches servoshell)
2. **wgpu** - Modern graphics API (Servo supports this too)
3. **Separate windows** - egui controls, Servo rendering (complex)

**Recommendation**: Use `egui_glow` for OpenGL compatibility with Servo.

---

## Build Configuration Requirements

### System Dependencies

**macOS**:
```bash
brew install cmake pkg-config python3
rustup toolchain install stable
```

**Linux (Ubuntu/Debian)**:
```bash
sudo apt install build-essential cmake python3 \\
    libssl-dev pkg-config \\
    libx11-dev libxcb-dev mesa-common-dev \\
    libgl1-mesa-dev libgles2-mesa-dev
```

**Windows**:
- Visual Studio Build Tools 2019+
- Python 3.8+
- CMake

### Rust Toolchain

```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

### Cargo Features

```toml
[dependencies]
libservo = { path = "external/servo/components/servo", features = [
    "background_hang_monitor",  # Hang detection
    "bluetooth",                # WebBluetooth API
    "webgpu",                   # WebGPU support
    "media-gstreamer",          # Video/audio (Linux)
] }
```

---

## Testing Strategy

### Phase 1: Minimal Example (Week 1)

```rust
// test_servo.rs - Standalone test
fn main() {
    let servo = ServoBuilder::new(gl_context)
        .build();

    let webview = WebViewBuilder::new(&servo)
        .url("http://example.com".parse().unwrap())
        .build();

    loop {
        servo.spin_event_loop();
        webview.paint();
    }
}
```

**Success criteria**:
- ✅ Builds without errors
- ✅ Opens window
- ✅ Loads example.com
- ✅ Displays something (even if broken)

### Phase 2: Integration (Week 2-3)

1. **Wire ServoRenderer**:
   - Replace simulated `load_url()` with real WebViewBuilder
   - Connect delegate callbacks
   - Handle frame rendering

2. **Test websites**:
   - http://example.com (simple)
   - http://info.cern.ch (basic HTML)
   - https://www.wikipedia.org (modern CSS)

3. **Verify features**:
   - URL navigation works
   - Title updates in tabs
   - Progress tracking
   - Error handling (404, DNS failures)

---

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| **Servo build fails** | High | Medium | Test build early, document errors, ask Servo community |
| **OpenGL integration complex** | High | High | Study servoshell GL setup, use egui_glow |
| **API changes in Servo** | Medium | Medium | Pin Servo commit hash, track upstream |
| **Performance issues** | Medium | Low | Profile early, optimize rendering loop |
| **Cross-platform issues** | Medium | High | Test on all platforms via CI, platform-specific code |

---

## Next Steps (Immediate)

### This Session (Remaining Time):
1. ✅ Update CLAUDE.md (remove co-authoring) - DONE
2. ✅ Investigate Servo integration - DONE
3. ✅ Document findings - IN PROGRESS
4. ⏳ Commit investigation document

### Next Session:
1. **Start Milestone 1.5** - Back/Forward navigation
2. **System dependency setup** (parallel) - Install OpenGL, build tools
3. **Create minimal Servo example** (parallel) - Test build

### Week 2-3:
1. **Complete Milestone 1.6** - Persistent storage
2. **Servo submodule integration** - Add to project
3. **Rendering context setup** - egui_glow + Servo

---

## References

**Servo Documentation**:
- Servo Book: https://book.servo.org
- API Docs: https://doc.servo.org/servo/
- GitHub: https://github.com/servo/servo

**servoshell Source** (Reference Implementation):
- `/tmp/servo/ports/servoshell/desktop/app.rs` - Servo initialization
- `/tmp/servo/ports/servoshell/desktop/app_state.rs` - WebView creation
- `/tmp/servo/ports/servoshell/desktop/minibrowser.rs` - Minimal browser UI

**Related Issues**:
- BUG-001: Servo dependency disabled (package not found) - Milestone 1.3
- Servo embedding API discussion: https://github.com/servo/servo/discussions/27595

---

## Conclusion (UPDATED)

**Servo integration is straightforward with official approach**. Requires:
1. ✅ Git dependency in Cargo.toml (NOT submodule - simpler!)
2. ✅ OpenGL rendering context (egui_glow)
3. ✅ Builder pattern API (ServoBuilder → WebViewBuilder)
4. ✅ Delegate callbacks for state updates
5. ✅ 1-2 week timeline (improved 2025 API much simpler)

**Key Insight from Official Docs**:
- Submodule approach was INCORRECT (unnecessary complexity)
- Official method: Direct Git dependency
- New 2025 WebView API reduced embedding code from ~200 to <50 lines
- Servo 0.0.1 released Oct 2025 with stable API

**Recommendation**: **Direct integration NOW** using official Git dependency approach.

**Updated Project Status**:
- Phase 1: Foundation - Milestone 1.4 complete (navigation architecture)
- Next: Complete Servo integration (1-2 weeks)
- ETA for real rendering: 1-2 weeks (not 3-4 weeks as initially estimated)

**References**:
- Official Embedding Example: https://github.com/paulrouget/servo-embedding-example
- Servo Book: https://book.servo.org/
- API Docs: https://doc.servo.org/servo/
- Servo 0.0.1 Release: https://servo.org/blog/2025/10/20/servo-0.0.1-release/
