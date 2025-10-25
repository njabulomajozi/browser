# Browser MVP Architecture

**Project**: Non-Chromium Browser (Rust + Servo)
**Architecture Version**: 1.0
**Last Updated**: 2025-10-25

---

## Table of Contents
1. [System Overview](#1-system-overview)
2. [Architecture Diagram](#2-architecture-diagram)
3. [Component Design](#3-component-design)
4. [Data Flow](#4-data-flow)
5. [Repository Structure](#5-repository-structure)
6. [Technology Stack](#6-technology-stack)
7. [Security Architecture](#7-security-architecture)
8. [Performance Considerations](#8-performance-considerations)

---

## 1. System Overview

### 1.1 High-Level Architecture

The browser follows a **multi-process architecture** inspired by modern browsers:

```
┌─────────────────────────────────────────────────────────┐
│                    Main Process (Browser)                │
│  - Window Management                                     │
│  - Tab Management                                        │
│  - UI Rendering (egui/iced)                             │
│  - IPC Coordinator                                       │
│  - Bookmarks/History Manager                            │
└─────────────┬───────────────────────────────────────────┘
              │ IPC (channels/message passing)
    ┌─────────┴─────────┬──────────────┬─────────────┐
    │                   │              │             │
┌───▼────┐     ┌────────▼───┐  ┌──────▼─────┐  ┌───▼────┐
│Renderer│     │ Renderer   │  │  Network   │  │Storage │
│Process │     │  Process   │  │  Process   │  │Process │
│(Tab 1) │     │  (Tab 2)   │  │            │  │        │
├────────┤     ├────────────┤  ├────────────┤  ├────────┤
│ Servo  │     │   Servo    │  │ HTTP/HTTPS │  │ SQLite │
│ Engine │     │   Engine   │  │ DNS Cache  │  │ Cache  │
└────────┘     └────────────┘  │ TLS        │  └────────┘
                                └────────────┘
```

### 1.2 Design Principles

1. **Process Isolation**: Each tab runs in separate process for stability
2. **Memory Safety**: Rust's ownership model prevents common browser vulnerabilities
3. **Async-First**: Non-blocking I/O with tokio runtime
4. **Modular Design**: Clear separation of concerns
5. **Performance**: Zero-cost abstractions, minimal overhead
6. **Cross-Platform**: Same codebase for Linux, macOS, Windows

---

## 2. Architecture Diagram

### 2.1 Component Interaction

```
┌──────────────────────────────────────────────────────────────┐
│                         Browser UI Layer                      │
│  ┌────────────┐  ┌──────────┐  ┌─────────┐  ┌─────────────┐ │
│  │  Tab Bar   │  │ URL Bar  │  │ Toolbar │  │  Settings   │ │
│  └─────┬──────┘  └────┬─────┘  └────┬────┘  └──────┬──────┘ │
│        │              │             │              │        │
└────────┼──────────────┼─────────────┼──────────────┼────────┘
         │              │             │              │
         └──────────────┴─────────────┴──────────────┘
                        │
         ┌──────────────▼─────────────────┐
         │      Browser Core (Main)       │
         │  ┌──────────────────────────┐  │
         │  │   Tab Manager            │  │
         │  ├──────────────────────────┤  │
         │  │   Navigation Controller  │  │
         │  ├──────────────────────────┤  │
         │  │   IPC Manager            │  │
         │  ├──────────────────────────┤  │
         │  │   Event Loop (tokio)     │  │
         │  └──────────────────────────┘  │
         └──────────────┬─────────────────┘
                        │
         ┌──────────────┴─────────────────┐
         │                                │
    ┌────▼─────┐                    ┌────▼──────┐
    │ Renderer │                    │  Services │
    │  Engine  │                    │  Layer    │
    ├──────────┤                    ├───────────┤
    │  Servo   │                    │ Network   │
    │  WebGL   │                    │ Storage   │
    │  WASM    │                    │ History   │
    │  JS VM   │                    │ Bookmarks │
    └──────────┘                    └───────────┘
```

### 2.2 Data Flow

```
User Action (Click URL)
    │
    ▼
┌───────────────┐
│   UI Event    │ (egui/iced event handler)
└───────┬───────┘
        │
        ▼
┌───────────────┐
│ Main Process  │ (validate URL, create tab if needed)
└───────┬───────┘
        │
        ▼ (IPC: LoadURL message)
┌───────────────┐
│Renderer Process│ (spawn/reuse renderer)
└───────┬───────┘
        │
        ▼
┌───────────────┐
│  Servo Engine │ (parse HTML, layout, paint)
└───────┬───────┘
        │
        ▼ (IPC: RenderFrame message)
┌───────────────┐
│  Main Process │ (composite frame, display)
└───────┬───────┘
        │
        ▼
┌───────────────┐
│   Display     │ (wgpu renders to window)
└───────────────┘
```

---

## 3. Component Design

### 3.1 Main Process Components

#### 3.1.1 Browser Core
**Responsibility**: Orchestrate all browser operations

```rust
// src/browser/mod.rs
pub struct Browser {
    tabs: Arc<RwLock<Vec<Tab>>>,
    active_tab: Arc<AtomicUsize>,
    ui_state: UiState,
    settings: Settings,
    event_loop: tokio::runtime::Runtime,
}

impl Browser {
    pub fn new() -> Self { /* ... */ }
    pub async fn create_tab(&mut self, url: Url) -> TabId { /* ... */ }
    pub async fn close_tab(&mut self, id: TabId) { /* ... */ }
    pub fn run(&mut self) { /* ... */ } // Main event loop
}
```

#### 3.1.2 Tab Manager
**Responsibility**: Lifecycle management of browser tabs

```rust
// src/tabs/manager.rs
pub struct TabManager {
    tabs: HashMap<TabId, Tab>,
    renderer_handles: HashMap<TabId, RendererHandle>,
}

pub struct Tab {
    id: TabId,
    title: String,
    url: Url,
    favicon: Option<Image>,
    state: TabState, // Loading, Loaded, Error
    history: Vec<Url>,
    history_index: usize,
}

impl TabManager {
    pub fn create_tab(&mut self) -> TabId { /* ... */ }
    pub fn navigate(&mut self, id: TabId, url: Url) { /* ... */ }
    pub fn reload(&mut self, id: TabId) { /* ... */ }
    pub fn back(&mut self, id: TabId) { /* ... */ }
    pub fn forward(&mut self, id: TabId) { /* ... */ }
}
```

#### 3.1.3 UI Layer (egui)
**Responsibility**: Render browser chrome (tabs, URL bar, etc.)

```rust
// src/ui/mod.rs
pub struct BrowserUi {
    tab_bar: TabBar,
    url_bar: UrlBar,
    toolbar: Toolbar,
    settings_panel: Option<SettingsPanel>,
}

impl BrowserUi {
    pub fn render(&mut self, ctx: &egui::Context, browser: &mut Browser) {
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            self.tab_bar.render(ui, &browser.tabs);
        });

        egui::TopBottomPanel::top("url_bar").show(ctx, |ui| {
            self.url_bar.render(ui, browser);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Embed rendered page content here
            self.render_page_content(ui, browser);
        });
    }
}
```

### 3.2 Renderer Process Components

#### 3.2.1 Servo Integration
**Responsibility**: Render web pages using Servo engine

```rust
// src/renderer/servo.rs
use servo::servo_config::opts;
use servo::servo_url::ServoUrl;
use servo::compositing::windowing::WindowEvent;

pub struct ServoRenderer {
    servo: Servo<ServoWindow>,
    event_queue: VecDeque<WindowEvent>,
}

impl ServoRenderer {
    pub fn new() -> Self {
        let opts = opts::default_opts();
        let servo = Servo::new(opts);
        Self {
            servo,
            event_queue: VecDeque::new(),
        }
    }

    pub fn load_url(&mut self, url: ServoUrl) {
        self.servo.handle_events(vec![WindowEvent::LoadUrl(url)]);
    }

    pub fn render_frame(&mut self) -> RenderFrame {
        self.servo.get_next_frame()
    }
}
```

#### 3.2.2 JavaScript Engine
**Responsibility**: Execute JavaScript (via Servo's SpiderMonkey)

```rust
// src/renderer/js.rs
// Servo handles JS execution internally via SpiderMonkey
// We expose APIs for browser-specific functionality

pub trait JsApi {
    fn alert(&self, message: &str);
    fn confirm(&self, message: &str) -> bool;
    fn prompt(&self, message: &str, default: &str) -> Option<String>;
    fn open_window(&self, url: &str) -> Option<TabId>;
}
```

### 3.3 Network Layer

#### 3.3.1 HTTP Client
**Responsibility**: Handle HTTP/HTTPS requests

```rust
// src/network/http.rs
use hyper::{Client, Body, Request, Response};
use hyper_rustls::HttpsConnector;

pub struct HttpClient {
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    cache: LruCache<Url, CachedResponse>,
}

impl HttpClient {
    pub async fn fetch(&mut self, url: Url) -> Result<Response<Body>> {
        // Check cache first
        if let Some(cached) = self.cache.get(&url) {
            if !cached.is_expired() {
                return Ok(cached.response.clone());
            }
        }

        // Fetch from network
        let req = Request::get(url.as_str()).body(Body::empty())?;
        let resp = self.client.request(req).await?;

        // Cache response
        self.cache.put(url.clone(), CachedResponse::new(resp.clone()));

        Ok(resp)
    }
}
```

### 3.4 Storage Layer

#### 3.4.1 Persistent Storage (SQLite)
**Responsibility**: Store bookmarks, history, settings

```rust
// src/storage/database.rs
use rusqlite::{Connection, params};

pub struct BrowserDatabase {
    conn: Connection,
}

impl BrowserDatabase {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(include_str!("schema.sql"))?;
        Ok(Self { conn })
    }

    pub fn add_history(&self, entry: HistoryEntry) -> Result<()> {
        self.conn.execute(
            "INSERT INTO history (url, title, visit_time) VALUES (?1, ?2, ?3)",
            params![entry.url, entry.title, entry.visit_time],
        )?;
        Ok(())
    }

    pub fn add_bookmark(&self, bookmark: Bookmark) -> Result<()> {
        self.conn.execute(
            "INSERT INTO bookmarks (url, title, folder) VALUES (?1, ?2, ?3)",
            params![bookmark.url, bookmark.title, bookmark.folder],
        )?;
        Ok(())
    }

    pub fn get_history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT url, title, visit_time FROM history ORDER BY visit_time DESC LIMIT ?1"
        )?;
        let entries = stmt.query_map([limit], |row| {
            Ok(HistoryEntry {
                url: row.get(0)?,
                title: row.get(1)?,
                visit_time: row.get(2)?,
            })
        })?;
        entries.collect()
    }
}
```

**Database Schema** (`src/storage/schema.sql`):
```sql
CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    title TEXT,
    visit_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    visit_count INTEGER DEFAULT 1
);

CREATE TABLE IF NOT EXISTS bookmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL UNIQUE,
    title TEXT,
    folder TEXT DEFAULT 'Unsorted',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT
);

CREATE INDEX idx_history_time ON history(visit_time DESC);
CREATE INDEX idx_history_url ON history(url);
```

### 3.5 IPC Layer

#### 3.5.1 Message Passing
**Responsibility**: Communication between processes

```rust
// src/ipc/mod.rs
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;

#[derive(Serialize, Deserialize, Debug)]
pub enum IpcMessage {
    // Main -> Renderer
    LoadUrl { tab_id: TabId, url: Url },
    Reload { tab_id: TabId },
    Stop { tab_id: TabId },
    GoBack { tab_id: TabId },
    GoForward { tab_id: TabId },

    // Renderer -> Main
    NavigationStarted { tab_id: TabId, url: Url },
    NavigationComplete { tab_id: TabId, title: String },
    RenderFrame { tab_id: TabId, frame: RenderFrame },
    ConsoleMessage { tab_id: TabId, message: String, level: LogLevel },
}

pub struct IpcChannel {
    tx: mpsc::UnboundedSender<IpcMessage>,
    rx: mpsc::UnboundedReceiver<IpcMessage>,
}

impl IpcChannel {
    pub fn new() -> (Self, Self) {
        let (tx1, rx1) = mpsc::unbounded_channel();
        let (tx2, rx2) = mpsc::unbounded_channel();
        (
            Self { tx: tx1, rx: rx2 },
            Self { tx: tx2, rx: rx1 },
        )
    }

    pub fn send(&self, msg: IpcMessage) {
        let _ = self.tx.send(msg);
    }

    pub async fn recv(&mut self) -> Option<IpcMessage> {
        self.rx.recv().await
    }
}
```

---

## 4. Data Flow

### 4.1 Page Load Sequence

```
1. User enters URL in URL bar
   └─> UI captures input, validates URL

2. Main Process receives navigation request
   └─> TabManager.navigate(tab_id, url)
   └─> IPC: send LoadUrl message to renderer

3. Renderer Process receives LoadUrl
   └─> ServoRenderer.load_url(url)
   └─> HTTP request via NetworkLayer
   └─> Receive HTML response

4. Servo parses HTML, constructs DOM
   └─> CSS parsing, style resolution
   └─> Layout computation
   └─> Paint commands generated

5. Renderer sends RenderFrame to Main
   └─> Main Process receives frame
   └─> Composite with UI chrome (tabs, URL bar)
   └─> wgpu draws to window

6. Main Process updates UI state
   └─> Update tab title
   └─> Update URL bar
   └─> Add to history
   └─> UI re-renders
```

### 4.2 User Interaction Flow (Click Link)

```
1. User clicks link in page
   └─> Servo intercepts click event

2. Renderer extracts target URL
   └─> Check if external navigation or same-page

3. If external navigation:
   └─> Send NavigationStarted IPC to Main
   └─> Main updates UI (loading spinner)
   └─> Renderer loads new URL
   └─> [Follow Page Load Sequence above]

4. If same-page anchor:
   └─> Renderer scrolls to anchor
   └─> No IPC needed
```

---

## 5. Repository Structure

Inspired by `settled-inv` monorepo structure:

```
browser/
├── apps/
│   └── desktop/              # Main desktop application
│       ├── src/
│       │   ├── main.rs       # Entry point
│       │   ├── browser.rs    # Browser core
│       │   ├── ui/           # egui UI components
│       │   └── ipc/          # IPC layer
│       ├── Cargo.toml
│       └── build.rs
│
├── packages/
│   ├── renderer/             # Servo renderer wrapper
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── servo.rs      # Servo integration
│   │   │   └── compositor.rs
│   │   └── Cargo.toml
│   │
│   ├── network/              # HTTP/HTTPS client
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── http.rs
│   │   │   ├── cache.rs
│   │   │   └── dns.rs
│   │   └── Cargo.toml
│   │
│   ├── storage/              # SQLite storage
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── database.rs
│   │   │   ├── history.rs
│   │   │   └── bookmarks.rs
│   │   ├── migrations/
│   │   │   └── schema.sql
│   │   └── Cargo.toml
│   │
│   └── shared/               # Shared types and utilities
│       ├── src/
│       │   ├── lib.rs
│       │   ├── types.rs
│       │   └── utils.rs
│       └── Cargo.toml
│
├── infra/                    # Build and deployment
│   ├── build.rs              # Cross-compilation scripts
│   ├── package.sh            # Packaging for distros
│   └── ci.yml                # GitHub Actions CI/CD
│
├── docs/
│   ├── plan/
│   │   ├── architecture.md   # This file
│   │   ├── requirements.md
│   │   └── tasks.md
│   ├── api/                  # API documentation
│   └── guides/               # Developer guides
│
├── tests/
│   ├── integration/          # End-to-end tests
│   ├── rendering/            # Rendering tests
│   └── performance/          # Benchmarks
│
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
│
├── Cargo.toml                # Workspace manifest
├── Cargo.lock
├── rust-toolchain.toml       # Rust version pinning
├── .rustfmt.toml
├── .clippy.toml
└── README.md
```

**Workspace Configuration** (`Cargo.toml`):
```toml
[workspace]
members = [
    "apps/desktop",
    "packages/renderer",
    "packages/network",
    "packages/storage",
    "packages/shared",
]
resolver = "2"

[workspace.dependencies]
# Shared dependencies across workspace
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
anyhow = "1.0"

# Rendering
servo = { git = "https://github.com/servo/servo", branch = "main" }
egui = "0.30"
wgpu = "24.0"
winit = "0.30"

# Networking
hyper = { version = "1.0", features = ["full"] }
hyper-rustls = "0.27"
reqwest = { version = "0.12", features = ["rustls-tls"] }

# Storage
rusqlite = { version = "0.32", features = ["bundled"] }
lru = "0.12"

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
```

---

## 6. Technology Stack

### 6.1 Core Technologies

| Layer | Technology | Version | Purpose |
|-------|-----------|---------|---------|
| **Language** | Rust | 1.70+ | Memory-safe systems programming |
| **Rendering Engine** | Servo | v0.0.1+ | Web page rendering (HTML, CSS, JS) |
| **GUI Framework** | egui | 0.30+ | Immediate-mode UI for browser chrome |
| **Window Management** | winit | 0.30+ | Cross-platform windowing |
| **Graphics** | wgpu | 24.0+ | WebGPU for hardware-accelerated rendering |
| **Async Runtime** | tokio | 1.40+ | Async I/O and task scheduling |
| **HTTP Client** | hyper / reqwest | 1.0 / 0.12 | Networking layer |
| **Database** | rusqlite | 0.32+ | SQLite for persistence |
| **Logging** | tracing | 0.1 | Structured logging |
| **Serialization** | serde | 1.0 | Data serialization (IPC, config) |

### 6.2 Development Tools

```toml
# dev-dependencies
[workspace.dev-dependencies]
criterion = "0.5"          # Benchmarking
proptest = "1.4"           # Property-based testing
insta = "1.31"             # Snapshot testing
wiremock = "0.6"           # HTTP mocking
tempfile = "3.8"           # Temporary files for tests
```

**Tooling**:
- **just**: Command runner (like Make but better)
- **cargo-deny**: License and security checks
- **cargo-audit**: Security vulnerability scanner
- **cargo-flamegraph**: Performance profiling
- **cargo-bloat**: Binary size analysis

---

## 7. Security Architecture

### 7.1 Threat Model

**Threats**:
1. Malicious JavaScript execution
2. Memory corruption vulnerabilities
3. Network interception (MITM)
4. XSS attacks
5. CSRF attacks
6. Clickjacking

**Mitigations**:

| Threat | Mitigation | Implementation |
|--------|-----------|----------------|
| JS Execution | Sandboxing | Servo's JS VM isolation |
| Memory Corruption | Rust ownership | Compile-time memory safety |
| MITM | TLS 1.3 | rustls for TLS, certificate pinning |
| XSS | CSP | Content Security Policy enforcement |
| CSRF | SameSite cookies | Strict cookie handling |
| Clickjacking | X-Frame-Options | Respect security headers |

### 7.2 Process Isolation

```rust
// Each tab runs in isolated process with limited privileges
pub struct RendererProcess {
    pid: u32,
    permissions: PermissionSet, // No file system, limited network
}

pub struct PermissionSet {
    can_access_fs: bool,        // false for renderer
    can_spawn_process: bool,    // false for renderer
    network_access: NetworkPermission, // Proxied through main
}
```

### 7.3 Content Security Policy

```rust
// src/security/csp.rs
pub struct CspValidator {
    policies: Vec<CspPolicy>,
}

impl CspValidator {
    pub fn validate_script(&self, source: &str) -> bool {
        // Check against script-src directive
        self.policies.iter().any(|p| p.allows_script(source))
    }

    pub fn validate_style(&self, source: &str) -> bool {
        // Check against style-src directive
        self.policies.iter().any(|p| p.allows_style(source))
    }
}
```

---

## 8. Performance Considerations

### 8.1 Optimization Strategies

1. **Lazy Loading**: Load tabs on-demand, unload inactive tabs
2. **Caching**: LRU cache for HTTP responses, image decoding
3. **Parallel Parsing**: Use Rayon for parallel HTML/CSS parsing
4. **GPU Acceleration**: wgpu for compositing and rendering
5. **Incremental Layout**: Only re-layout changed DOM subtrees

### 8.2 Memory Management

```rust
// Tab sleeping: unload inactive tabs after 5 minutes
pub struct TabSleeper {
    inactive_timeout: Duration,
    sleeping_tabs: HashMap<TabId, SleepingTab>,
}

pub struct SleepingTab {
    url: Url,
    scroll_position: ScrollPosition,
    form_data: HashMap<String, String>,
    // Renderer process terminated, state serialized
}

impl TabSleeper {
    pub async fn check_tabs(&mut self, tabs: &TabManager) {
        for tab in tabs.inactive_tabs() {
            if tab.inactive_for() > self.inactive_timeout {
                self.sleep_tab(tab.id).await;
            }
        }
    }
}
```

### 8.3 Profiling Integration

```rust
// src/profiling/mod.rs
use tracing::{info_span, instrument};

#[instrument(skip(self))]
pub async fn load_page(&mut self, url: Url) {
    let _span = info_span!("page_load", url = %url).entered();

    let fetch_span = info_span!("fetch_html").entered();
    let html = self.fetch(url).await;
    drop(fetch_span);

    let parse_span = info_span!("parse_html").entered();
    let dom = parse_html(&html);
    drop(parse_span);

    // tracing outputs to JSON, analyzable with Tracy/Perfetto
}
```

### 8.4 Benchmarks

```rust
// benches/page_load.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_page_load(c: &mut Criterion) {
    c.bench_function("load_simple_page", |b| {
        b.iter(|| {
            let browser = Browser::new();
            browser.load_url(black_box("http://example.com"));
        });
    });
}

criterion_group!(benches, bench_page_load);
criterion_main!(benches);
```

---

## 9. Platform-Specific Considerations

### 9.1 Linux
- **Packaging**: AppImage, Flatpak, Snap
- **Display**: Wayland and X11 support via winit
- **GPU**: Vulkan backend for wgpu

### 9.2 macOS
- **Packaging**: .app bundle, DMG
- **Display**: Native Cocoa windows via winit
- **GPU**: Metal backend for wgpu
- **Sandboxing**: macOS App Sandbox for security

### 9.3 Windows
- **Packaging**: MSI installer, portable EXE
- **Display**: Win32 windows via winit
- **GPU**: DirectX 12 backend for wgpu
- **WebView2**: NOT used (we're building our own engine!)

---

## 10. Future Enhancements

### 10.1 Post-MVP Features
- **Extensions API**: WebExtensions-compatible API
- **Sync**: Cross-device sync via cloud backend
- **Mobile**: Android/iOS support with Servo mobile
- **Developer Tools**: Full Chrome DevTools Protocol
- **Accessibility**: Screen reader support, keyboard navigation
- **Performance**: Multi-threaded rendering, GPU rasterization

### 10.2 Research Areas
- **WebRTC**: Video/audio chat support
- **WebGPU**: Native WebGPU API (Servo supports this)
- **WASM**: Optimize WASM execution speed
- **Quantum CSS**: Integrate Servo's Stylo parallel CSS engine

---

## 11. References

### Official Documentation
- Servo Architecture: https://github.com/servo/servo/wiki/Design
- egui Documentation: https://docs.rs/egui/
- wgpu Guide: https://sotrh.github.io/learn-wgpu/
- Tokio Tutorial: https://tokio.rs/tokio/tutorial

### Inspiration
- Firefox Architecture: https://firefox-source-docs.mozilla.org/
- Chromium Design Docs: https://www.chromium.org/developers/design-documents
- WebKit Architecture: https://webkit.org/code/

---

**Next Steps**: Review tasks.md for implementation roadmap.
