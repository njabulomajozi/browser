# Browser MVP Requirements

**Project**: Non-Chromium Browser MVP
**Language**: Rust (Recommended)
**Last Updated**: 2025-10-25
**Status**: Planning Phase

---

## Executive Summary

Build a minimal viable browser that is **NOT dependent on Chromium**, using Rust and the Servo rendering engine. The MVP focuses on core browsing functionality with modern web standards support, security, and performance.

**Why Rust over Golang?**
- 30-12x better performance than Go
- Memory safety without garbage collection
- Proven browser engine (Servo) built in Rust
- No GC overhead (~10% in Go)
- Better concurrency and parallelism for rendering

---

## 1. Core Requirements

### 1.1 Browser Engine
- **Rendering Engine**: Servo (Rust-based, non-Chromium)
- **JavaScript Engine**: SpiderMonkey or embedded JS engine
- **Layout Engine**: Servo's layout engine with CSS Grid, Flexbox support
- **HTML Parser**: Servo's HTML5 parser
- **Networking**: Hyper or Reqwest (Rust HTTP clients)

### 1.2 User Interface
- **UI Framework**: egui (immediate mode GUI) or iced (Elm-inspired GUI)
- **Platform Support**:
  - Primary: Linux, macOS, Windows
  - Future: Android, iOS (via Servo mobile support)
- **Window Management**: winit (cross-platform window creation)
- **Rendering Backend**: wgpu (WebGPU for modern graphics)

### 1.3 Core Features (MVP)
- [x] URL bar with navigation
- [x] Back/Forward navigation
- [x] Refresh/Stop controls
- [x] Basic tab management (open, close, switch)
- [x] Bookmarks (simple list)
- [x] History tracking
- [x] Settings page
- [x] Developer console (basic)

### 1.4 Web Standards Support
- **HTML5**: Full support via Servo
- **CSS3**: Modern CSS including Grid, Flexbox, animations
- **JavaScript**: ES2024 support
- **WebAssembly**: WASM support via Servo
- **WebGL/WebGPU**: Hardware-accelerated graphics
- **Web APIs**:
  - DOM manipulation
  - Fetch API
  - LocalStorage/SessionStorage
  - Web Workers (future)

---

## 2. Non-Functional Requirements

### 2.1 Performance
- **Page Load**: < 3s for average webpage
- **Memory Usage**: < 500MB for 10 tabs (vs Chromium ~1GB+)
- **CPU Usage**: Efficient parallelism via Rust
- **Startup Time**: < 2s cold start
- **Frame Rate**: 60 FPS for animations and scrolling

### 2.2 Security
- **Memory Safety**: Rust's compile-time guarantees
- **Sandboxing**: Process isolation for tabs (future enhancement)
- **HTTPS**: TLS 1.3 support, certificate validation
- **Content Security Policy**: CSP support
- **Privacy**:
  - No telemetry by default
  - Cookie management
  - Do Not Track (DNT) header
  - Private browsing mode (future)

### 2.3 Reliability
- **Crash Recovery**: Tab isolation to prevent full browser crash
- **Data Persistence**: SQLite for history, bookmarks, settings
- **Error Handling**: Graceful degradation for rendering errors
- **Updates**: Auto-update mechanism (future)

### 2.4 Developer Experience
- **Build Time**: < 5 minutes clean build
- **Hot Reload**: Incremental compilation via Cargo
- **Debugging**: Chrome DevTools Protocol support (future)
- **Logging**: Structured logging with tracing crate
- **Testing**: Unit, integration, and browser tests

---

## 3. Technical Requirements

### 3.1 Technology Stack
```
Rendering:    Servo (v0.0.1+)
Language:     Rust (1.70+)
UI:           egui 0.30+ or iced 0.13+
Windowing:    winit 0.30+
Networking:   hyper 1.0+ or reqwest 0.12+
Graphics:     wgpu 24.0+
Database:     rusqlite 0.32+
Logging:      tracing 0.1+
Build:        Cargo, just (command runner)
CI/CD:        GitHub Actions
```

### 3.2 Architecture Patterns
- **Multi-process**: Main process + renderer processes
- **Event-driven**: Async/await with tokio runtime
- **Plugin System**: Dynamic loading for extensions (future)
- **IPC**: Inter-process communication via channels
- **State Management**: Reactive state with signals/observers

### 3.3 Data Storage
- **SQLite** for persistent data:
  - Bookmarks table
  - History table (URL, timestamp, title)
  - Settings/preferences
  - Download history
- **In-memory cache**: LRU cache for page resources
- **Disk cache**: On-disk cache for HTTP responses

---

## 4. Comparison: Rust vs Golang

| Criteria | Rust | Golang | Winner |
|----------|------|--------|--------|
| **Performance** | 30-12x faster | Baseline | ✅ Rust |
| **Memory Safety** | Compile-time guarantees, no GC | Runtime checks, GC overhead | ✅ Rust |
| **Existing Engine** | Servo (production-ready) | None (webview wrappers only) | ✅ Rust |
| **Browser Ecosystem** | Firefox components, Servo | None | ✅ Rust |
| **Concurrency** | Fine-grained parallelism | Good, but GC pauses | ✅ Rust |
| **Development Speed** | Steeper learning curve | Faster initial development | ⚖️ Tie |
| **Binary Size** | Smaller (no GC runtime) | Larger (includes GC) | ✅ Rust |
| **GUI Libraries** | egui, iced, Tauri | Wails, Fyne | ✅ Rust (Tauri) |
| **Use Case Fit** | Browser rendering engine | Webview wrapper apps | ✅ Rust |

**Decision**: Use **Rust** for the browser MVP.

**Why Not Golang?**
- Wails/Go are excellent for webview-based apps (wrapping Chromium/WebKit)
- No native rendering engine in Go
- GC overhead unacceptable for browser performance
- Rust has proven browser code (Servo, Firefox components)

---

## 5. Constraints & Assumptions

### 5.1 Constraints
- **No Chromium dependency**: Must use alternative rendering engine
- **Cross-platform**: Linux, macOS, Windows support
- **Memory budget**: Stay under 500MB for typical usage
- **Team size**: 1-3 developers for MVP
- **Approach**: Milestone-driven, flexible timeline

### 5.2 Assumptions
- Users have modern hardware (4GB+ RAM, multi-core CPU)
- Target audience: Privacy-conscious users, developers
- Not targeting enterprise/legacy browser needs
- Web standards support prioritized over legacy site compatibility
- Development on modern OS (no Windows XP/Vista support)

### 5.3 Out of Scope (MVP)
- Browser extensions/add-ons
- Sync across devices
- Password manager integration
- PDF viewer
- Print functionality
- Accessibility tools (initial MVP)
- Mobile platforms (future roadmap)
- DRM/EME support

---

## 6. Success Criteria

### 6.1 MVP Launch Criteria
- [ ] Renders 90%+ of top 100 websites correctly
- [ ] No memory leaks after 1 hour browsing session
- [ ] Passes Acid3 test (web standards compliance)
- [ ] Lighthouse performance score > 80
- [ ] Startup time < 2 seconds
- [ ] Can browse GitHub, Wikipedia, HackerNews without issues
- [ ] Tab management for 20+ tabs without crash
- [ ] Passes security audit (no critical vulnerabilities)

### 6.2 User Acceptance
- Users can browse daily websites (news, social media, dev sites)
- Perceived as "fast" compared to Chrome/Firefox
- No frequent crashes or hangs
- Privacy features clearly communicated
- Clear browser identity (not "another Chrome clone")

---

## 7. Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Servo compatibility issues | High | Medium | Extensive testing, upstream contributions |
| Poor performance on complex sites | High | Medium | Profiling, optimization, caching |
| Missing web standards | Medium | High | Prioritize most-used APIs, progressive enhancement |
| Platform-specific bugs | Medium | High | CI/CD testing on all platforms |
| Rust learning curve | Low | High | Good documentation, community support |
| Limited browser expertise | High | Medium | Study Servo/Firefox codebase, hire expert |

---

## 8. Development Priorities

### Phase 1: Foundation (6 milestones)
1. Integrate Servo rendering engine
2. Build basic UI with egui/iced
3. Implement navigation (URL bar, back/forward)
4. Single tab browsing

### Phase 2: Core Features (6 milestones)
1. Tab management
2. Bookmarks and history
3. Settings page
4. Developer console basics
5. Performance profiling

### Phase 3: Polish (6 milestones)
1. Security hardening
2. Performance optimization
3. Bug fixes from testing
4. Documentation
5. Beta testing

### Phase 4: Launch (4 milestones)
1. Public beta release
2. Community feedback
3. Roadmap for v1.0
4. Extensions API design

---

## 9. Resources & References

### Servo Documentation
- Official Site: https://servo.org/
- GitHub: https://github.com/servo/servo
- Embedding Guide: https://github.com/servo/servo/wiki/Roadmap

### Rust GUI Libraries
- egui: https://github.com/emilk/egui (immediate mode, simple)
- iced: https://github.com/iced-rs/iced (Elm-inspired, reactive)
- Tauri: https://tauri.app/ (webview wrapper, not rendering engine)

### Inspiration
- Ladybird: https://ladybird.org/ (new browser engine, C++)
- Flow/Ekioh: https://www.ekioh.com/flow-browser/ (new engine)
- Servo Nightly: https://servo.org/download/ (test Servo capabilities)

### Community
- Rust Browser Working Group
- Servo Community Discord
- Reddit: r/rust, r/servo

---

**Next Steps**: Review architecture.md for technical design and tasks.md for implementation roadmap.
