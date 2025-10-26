# Browser MVP Development Workflow

Cross-platform web browser built with Rust and wry (platform WebView wrapper). Desktop application (Linux, macOS, Windows) focused on performance and memory safety.

---

## 🎯 Quick Reference

**Tech Stack:** Rust 1.70+ · wry 0.47 (WebView) · tao 0.30 (windowing) · SQLite (storage) · tokio (async)

**Key Concepts:**

- Platform WebView rendering (WKWebView/WebView2/WebKitGTK)
- Memory safety via Rust ownership model
- Future: Multi-process architecture for tab isolation
- Custom HTTP client with caching
- Production-ready stack (powers Tauri)

**Architecture:** See [/docs/plan/architecture.md](docs/plan/architecture.md) · [/docs/plan/requirements.md](docs/plan/requirements.md)

---

## ⚠️ Critical Workflow Requirements

**These requirements MUST be followed for every task. No exceptions.**

### 1. Task Management & Bug Tracking

- **ALWAYS use TodoWrite tool** to track ALL tasks, including bugs found during testing
- **NEVER mark tasks complete** without actually performing them
- When bugs are discovered:
  - Add them to todo list immediately
  - Include severity (critical/high/medium/low)
  - Include steps to reproduce
- Keep todo list current - update status as you work

### 2. Rust Toolchain Management

**Before starting development**, ensure correct Rust toolchain:

```bash
# Install/update Rust toolchain (specified in rust-toolchain.toml)
rustup show

# Verify Rust version
rustc --version  # Should be 1.70+

# Install required components
rustup component add clippy rustfmt
```

**Why**: Project uses specific Rust features requiring recent toolchain.

### 3. Build & Development (MANDATORY WORKFLOW)

**CRITICAL**: Always follow this order for development:

```bash
# 1. Check code quality BEFORE building
just check        # Runs cargo check + clippy
just fmt          # Format code with rustfmt

# 2. Build the project
just build        # Builds all workspace crates

# 3. Run tests
just test         # Runs all tests

# 4. Run the browser
just run          # Starts the desktop app
```

**Common Issues:**

- **wry platform dependencies**: Ensure WebView libraries installed (WebKitGTK on Linux, WebView2 on Windows)
- **Linker errors**: May need system dependencies (see Platform Setup below)
- **clippy warnings**: Fix ALL clippy warnings before committing

### 4. Testing Requirements (CRITICAL - NO SHORTCUTS)

**For browser testing, you MUST**:

- ✅ Build project without errors (`cargo build`)
- ✅ Run clippy with zero warnings (`cargo clippy`)
- ✅ Run rustfmt (`cargo fmt --all`)
- ✅ Test on actual target platform (not just cross-compile)
- ✅ **Test with real websites** - don't just test with hardcoded HTML
- ✅ **Check console for WebView errors** - logs go to stderr

**Testing Rules:**

- ❌ **NEVER skip platform-specific testing** - bugs often platform-dependent
- ❌ **NEVER assume it works if it compiles** - Rust prevents memory bugs, not logic bugs
- ❌ **NEVER mark tests complete without running them**
- ✅ **ALWAYS test navigation flow end-to-end**
- ✅ **ALWAYS check memory usage** - use Activity Monitor/htop
- ✅ **ALWAYS test tab isolation** - crash one tab, others should survive

**Test Websites (Use These for Verification):**

```bash
# Simple sites (should render perfectly)
http://example.com
http://info.cern.ch

# Modern sites (target: 90% rendering accuracy)
https://www.wikipedia.org
https://github.com
https://news.ycombinator.com

# Complex sites (known issues acceptable for MVP)
https://www.youtube.com
https://www.twitter.com
```

### 5. Communication & Collaboration

**When you encounter issues or blockers**:

- ❌ **DO NOT** skip the task and mark it complete
- ❌ **DO NOT** work around WebView limitations without documenting them
- ✅ **DO** use AskUserQuestion tool to clarify requirements
- ✅ **DO** add the blocker to todo list and report it
- ✅ **DO** check wry/tao issue trackers for known bugs before debugging
- ✅ **DO** involve the user before making architectural changes

**Examples**:

- wry API confusing? Check Tauri examples repo, ask user
- Build failing? Check system deps (WebView libraries), report full error
- Unclear requirements? Clarify with user first
- Found better approach? Discuss with user before changing plan

**Reporting Style - BLUF (Bottom Line Up Front)**:

Use military-style concise reporting. Prioritize clarity and speed over grammar.

✅ **DO**:

- Lead with conclusion/result first (BLUF principle)
- Use telegram-style brevity: omit articles (a/an/the), auxiliary verbs when clear
- State facts directly: "Found 3 bugs" not "I have found three bugs"
- Use bullet points for multiple items
- File paths with line numbers: `wry_renderer.rs:142`
- Status first, details after: "✅ Complete. Implemented X, Y, Z"

❌ **DON'T**:

- Lead with process descriptions before results
- Use filler words: "I think", "maybe", "perhaps", "it seems"
- Write full sentences when fragments suffice
- Repeat information already visible to user
- Apologize for being concise

**Examples**:

```
❌ Verbose:
"I have completed the implementation of the tab management system.
I created the TabManager struct with methods for creating, closing,
and switching between tabs."

✅ Concise (BLUF):
"✅ Tab management complete. Created:
- TabManager:1-89 - create/close/switch tabs
- Tab:1-45 - per-tab state and history
- IPC:23-56 - tab process isolation"

❌ Verbose:
"I encountered a build error when trying to compile the project.
It appears that WebKitGTK requires some system libraries that are not installed."

✅ Concise (BLUF):
"❌ Build failed. Missing webkit2gtk-4.0.
Fix: `sudo apt install libwebkit2gtk-4.0-dev`"
```

### 6. Testing Integrity

**NEVER mark these tasks complete without actually doing them**:

- Cargo build (`cargo build --workspace`)
- Clippy (`cargo clippy --workspace`)
- Tests (`cargo test --workspace`)
- Rustfmt (`cargo fmt --all --check`)
- Platform testing (test on Linux/macOS/Windows)
- Memory leak checks (long-running session test)

**If a test fails**:

1. Add failure to todo list as a bug
2. Debug and fix the issue
3. Re-run the test
4. Only mark complete when test passes

---

## 1️⃣ Discovery Phase

### Research & Documentation

**ALWAYS Use Context7 MCP for Library Documentation:**

Context7 provides up-to-date documentation for Rust crates. Use it as your PRIMARY source for API references and examples.

```bash
# Examples of Context7 usage:
@context7 wry webview                      # WebView rendering
@context7 tao windowing                    # Window management
@context7 tokio async runtime              # Async programming
@context7 rusqlite sqlite database         # Storage layer
@context7 hyper http client                # Networking
```

**When to use Context7:**

- ✅ API documentation for any Rust crate
- ✅ Code examples and patterns
- ✅ Version-specific features
- ✅ Migration guides between versions
- ✅ Configuration options

**Use Internet Search for:**

- Browser architecture best practices (Chromium, Firefox design docs)
- wry/Tauri-specific issues (GitHub issues, Discord)
- Cross-platform WebView patterns
- Performance optimization techniques
- Security best practices for browsers

**Project Documentation:**

- `/docs/plan/architecture.md` - System design and component interaction
- `/docs/plan/requirements.md` - Technical and functional requirements
- `/docs/plan/tasks.md` - Milestone-driven implementation roadmap
- `/docs/api/` - API documentation (future)
- `/docs/guides/` - Developer guides (future)

### Full Task Workflow Template

Every task should follow this complete workflow:

#### 1. Discovery

- [ ] Research requirements and constraints
- [ ] Review existing code and patterns
- [ ] Use Context7 for Rust crate documentation
- [ ] Check wry/Tauri examples and documentation
- [ ] Design component API and data structures
- [ ] Create implementation plan with TodoWrite

#### 2. Development

- [ ] Write code following Rust best practices
- [ ] Use `cargo check` frequently during development
- [ ] Implement with proper error handling (Result<T, E>)
- [ ] Write inline documentation for public APIs
- [ ] Test locally as you build

#### 3. Testing

- [ ] Write unit tests for new functions
- [ ] Run `cargo test --workspace`
- [ ] Run `cargo clippy --workspace` - fix all warnings
- [ ] Run `cargo fmt --all` - format code
- [ ] Test with real websites (see Testing Requirements)
- [ ] **Track all bugs found** in todo list with severity

#### 4. Pre-Commit

- [ ] Build project (`cargo build --workspace`)
- [ ] Run full test suite
- [ ] **Verify all tests passed** - NEVER skip
- [ ] **Update docs/plan/tasks.md** - mark completed milestones/tasks with ✅
- [ ] Update documentation if API changed
- [ ] Review diff for unintended changes
- [ ] **Reference milestone in commit message** - see Git Commit Standards

### System Design Approach

Think in layers before coding:

1. **Data Model** - What structs and enums? Use Rust ownership to encode invariants
2. **Business Logic** - What workflows and state transitions? Document with comments
3. **API Surface** - What functions are public? Use type system for safety
4. **UI Components** - egui immediate mode or iced declarative? Choose based on complexity
5. **Concurrency** - What needs async? Use tokio for I/O, rayon for CPU parallelism
6. **Safety** - Memory safety via ownership, thread safety via Send/Sync

---

## 2️⃣ Development Phase

### Platform Setup

**Linux (Ubuntu/Debian)**:

```bash
# Install system dependencies for wry (WebKitGTK on Linux)
sudo apt install -y \
  libwebkit2gtk-4.0-dev \
  libssl-dev pkg-config \
  libgtk-3-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install just (command runner)
cargo install just
```

**macOS**:

```bash
# Install Homebrew if needed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install cmake pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install just
cargo install just
```

**Windows**:

```powershell
# Install Visual Studio Build Tools (required for Rust)
# Download from: https://visualstudio.microsoft.com/downloads/

# Install Rust
# Download from: https://rustup.rs/

# Install just
cargo install just
```

### Quick Start

```bash
# 1. Clone repository
git clone <repo-url>
cd browser

# 2. Install Rust toolchain (if not already)
rustup show

# 3. Install just command runner
cargo install just

# 4. Build the project
just build

# 5. Run tests
just test

# 6. Run the browser
just run
```

### Cargo Workspace Structure

This is a monorepo with multiple Rust crates:

```
browser/
├── apps/
│   └── desktop/              # Main desktop application binary
├── packages/
│   ├── renderer/             # wry WebView wrapper
│   ├── network/              # HTTP client and caching
│   ├── storage/              # SQLite database layer
│   └── shared/               # Shared types and utilities
├── Cargo.toml                # Workspace manifest
└── Cargo.lock                # Dependency lock file
```

**Workspace Dependencies:**

All shared dependencies defined in root `Cargo.toml`:

```toml
[workspace.dependencies]
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
tao = "0.30"
wry = "0.47"
```

**Using Workspace Dependencies:**

```toml
# In packages/renderer/Cargo.toml
[dependencies]
wry = { workspace = true }
tokio = { workspace = true }
```

### Rust Best Practices

**Error Handling:**

```rust
// ✅ Use Result for fallible operations
pub fn load_url(&mut self, url: &str) -> Result<(), BrowserError> {
    let parsed = Url::parse(url)
        .map_err(|e| BrowserError::InvalidUrl(e.to_string()))?;

    self.renderer.load(parsed)?;
    Ok(())
}

// ✅ Use anyhow for application-level errors
use anyhow::{Result, Context};

pub fn initialize_database(path: &Path) -> Result<Database> {
    let conn = Connection::open(path)
        .context("Failed to open database")?;

    conn.execute_batch(SCHEMA_SQL)
        .context("Failed to initialize schema")?;

    Ok(Database { conn })
}

// ❌ Don't panic in library code
pub fn get_tab(&self, id: TabId) -> &Tab {
    &self.tabs[id]  // ❌ Will panic if id doesn't exist
}

// ✅ Return Option instead
pub fn get_tab(&self, id: TabId) -> Option<&Tab> {
    self.tabs.get(&id)
}
```

**Ownership & Borrowing:**

```rust
// ✅ Use references for read-only access
pub fn render(&self, ctx: &egui::Context) {
    ctx.label(&self.title);  // Borrow title
}

// ✅ Use mutable references for modification
pub fn navigate(&mut self, url: Url) {
    self.current_url = url;
    self.history.push(url);
}

// ✅ Use owned values when transferring ownership
pub fn create_tab(&mut self, url: Url) -> TabId {
    let id = self.next_id;
    let tab = Tab::new(id, url);  // url moved into tab
    self.tabs.insert(id, tab);
    id
}
```

**Pattern Matching:**

```rust
// ✅ Handle all cases explicitly
match tab_state {
    TabState::Loading => { /* show spinner */ }
    TabState::Loaded(ref page) => { /* render page */ }
    TabState::Error(ref e) => { /* show error */ }
}

// ✅ Use if let for single case
if let Some(tab) = self.get_active_tab() {
    tab.reload();
}
```

**Async/Await:**

```rust
// ✅ Use async for I/O operations
pub async fn fetch_page(&self, url: Url) -> Result<String> {
    let response = self.http_client.get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

// ✅ Spawn tasks for concurrent work
use tokio::spawn;

pub async fn load_resources(&self, urls: Vec<Url>) -> Vec<Result<Resource>> {
    let tasks: Vec<_> = urls.into_iter()
        .map(|url| spawn(self.fetch_resource(url)))
        .collect();

    futures::future::join_all(tasks).await
}
```

### Code Quality Standards

**Readability & Maintainability:**

- Clear variable/function names that explain intent
- Keep functions focused (<100 lines as guideline)
- Use Rust's type system to encode invariants
- Write doc comments for public APIs (`///`)
- Use `#[must_use]` for important return values
- Leverage `clippy` suggestions

**Error Handling:**

- Always propagate errors with `?` or explicit handling
- Provide context with `.context()` from anyhow
- Use custom error types for domain errors
- Never use `.unwrap()` in library code (tests OK)
- Log errors before returning them

**Performance:**

- Avoid allocations in hot paths (profile first!)
- Use `Arc` for shared immutable data across threads
- Use `Rc` for shared immutable data within thread
- Prefer iterators over explicit loops
- Use `rayon` for data parallelism
- Profile with `cargo flamegraph`

**Safety:**

- Never use `unsafe` without thorough justification
- If using `unsafe`, document invariants clearly
- Use `#[deny(unsafe_code)]` in most crates
- Let wry handle unsafe platform WebView code

### Development Commands

**Just Commands** (defined in `justfile`):

```bash
# Build all workspace crates
just build

# Build in release mode (optimized)
just build-release

# Run the browser
just run

# Run tests
just test

# Run specific test
just test-one test_name

# Check code (fast, no build)
just check

# Format code
just fmt

# Check formatting without changing
just fmt-check

# Run clippy linter
just clippy

# Security audit
just audit

# Benchmark performance
just bench

# Profile binary size
just bloat

# Clean build artifacts
just clean
```

**Cargo Commands** (if not using just):

```bash
# Build
cargo build --workspace
cargo build --workspace --release

# Test
cargo test --workspace
cargo test --package renderer --test test_name

# Check (faster than build)
cargo check --workspace

# Format
cargo fmt --all
cargo fmt --all --check  # CI mode

# Lint
cargo clippy --workspace -- -D warnings

# Audit dependencies
cargo audit
cargo deny check

# Benchmarks
cargo bench --workspace

# Binary size analysis
cargo bloat --release -n 20

# Clean
cargo clean
```

### wry WebView Integration Guidelines

**wry Integration Pattern (Tauri-based)**

wry provides platform-native WebView rendering. Follow these principles:

**1. Use tao for Windowing:**

```rust
// tao is Tauri's winit fork, designed for wry compatibility
use tao::{
    event_loop::EventLoop,
    window::WindowBuilder,
};
use wry::WebViewBuilder;

let event_loop = EventLoop::new();
let window = WindowBuilder::new()
    .with_title("Browser MVP")
    .build(&event_loop)?;

let webview = WebViewBuilder::new()
    .with_url("https://example.com")
    .build(&window)?;
```

**2. Handle Navigation:**

```rust
use wry::WebViewBuilder;

let webview = WebViewBuilder::new()
    .with_url(initial_url)
    .with_navigation_handler(|uri: String| {
        println!("Navigating to: {}", uri);
        true // Allow navigation
    })
    .build(&window)?;

// Navigate programmatically
webview.load_url("https://example.com")?;
```

**3. JavaScript Evaluation:**

```rust
// Execute JavaScript in WebView
webview.evaluate_script("document.title")?;

// Use for navigation controls
webview.evaluate_script("window.history.back()")?;
webview.evaluate_script("window.history.forward()")?;
webview.evaluate_script("window.location.reload()")?;
```

**4. Platform-Specific Considerations:**

```rust
// macOS: Use WKWebView (WebKit)
#[cfg(target_os = "macos")]
let webview = WebViewBuilder::new()
    .with_url(url)
    .build(&window)?;

// Windows: Use WebView2 (Chromium-based)
#[cfg(target_os = "windows")]
let webview = WebViewBuilder::new()
    .with_url(url)
    .build(&window)?;

// Linux: Use WebKitGTK
#[cfg(target_os = "linux")]
let webview = WebViewBuilder::new()
    .with_url(url)
    .build(&window)?;
```

### GUI Framework Best Practices

**egui (Immediate Mode) - Recommended for MVP:**

```rust
impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel: Tab bar
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, tab) in self.tabs.iter().enumerate() {
                    if ui.selectable_label(i == self.active_tab, &tab.title).clicked() {
                        self.active_tab = i;
                    }
                }
                if ui.button("+").clicked() {
                    self.create_tab();
                }
            });
        });

        // URL bar
        egui::TopBottomPanel::top("url_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("←").clicked() {
                    self.go_back();
                }
                if ui.button("→").clicked() {
                    self.go_forward();
                }
                if ui.text_edit_singleline(&mut self.url_input).lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.navigate(&self.url_input.clone());
                }
            });
        });

        // Central panel: Rendered page
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tab) = self.get_active_tab() {
                self.render_page_content(ui, tab);
            }
        });

        // Request repaint for animations
        ctx.request_repaint();
    }
}
```

**iced (Declarative) - Alternative:**

```rust
// iced uses Elm-style message passing
#[derive(Debug, Clone)]
pub enum Message {
    UrlChanged(String),
    Navigate,
    TabSelected(usize),
    NewTab,
}

impl Application for BrowserApp {
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::UrlChanged(url) => {
                self.url_input = url;
                Command::none()
            }
            Message::Navigate => {
                self.navigate(&self.url_input.clone());
                Command::none()
            }
            Message::TabSelected(index) => {
                self.active_tab = index;
                Command::none()
            }
            Message::NewTab => {
                self.create_tab();
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        // Declare UI structure
        column![
            self.tab_bar(),
            self.url_bar(),
            self.page_content(),
        ].into()
    }
}
```

### Database Patterns (SQLite)

**Schema Migration:**

```rust
// packages/storage/src/migrations.rs
pub const MIGRATIONS: &[&str] = &[
    // Migration 1: Initial schema
    include_str!("../migrations/001_initial.sql"),
    // Migration 2: Add bookmarks
    include_str!("../migrations/002_bookmarks.sql"),
];

pub fn run_migrations(conn: &Connection) -> Result<()> {
    let current_version = get_schema_version(conn)?;

    for (i, migration) in MIGRATIONS.iter().enumerate() {
        let version = i + 1;
        if version > current_version {
            conn.execute_batch(migration)?;
            set_schema_version(conn, version)?;
        }
    }

    Ok(())
}
```

**Query Patterns:**

```rust
// ✅ Use prepared statements
pub fn get_history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
    let mut stmt = self.conn.prepare(
        "SELECT url, title, visit_time FROM history
         ORDER BY visit_time DESC LIMIT ?1"
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

// ✅ Use transactions for atomicity
pub fn add_bookmark_batch(&self, bookmarks: Vec<Bookmark>) -> Result<()> {
    let tx = self.conn.transaction()?;

    for bookmark in bookmarks {
        tx.execute(
            "INSERT INTO bookmarks (url, title, folder) VALUES (?1, ?2, ?3)",
            params![bookmark.url, bookmark.title, bookmark.folder],
        )?;
    }

    tx.commit()?;
    Ok(())
}
```

### Git Commit Standards

Follow Conventional Commits for Rust projects:

**Format:**

```
<type>(<scope>): <subject (max 50 chars)>

<body: wrap at 72 chars, explain WHY not HOW>
```

**Types:** feat, fix, docs, style, refactor, perf, test, build, chore
**Scopes:** renderer, ui, network, storage, desktop, wry

**Example:**

```
feat(renderer): add multi-tab process isolation

Each tab now runs in separate renderer process for stability.
If one tab crashes, others continue working.

Implementation:
- RendererProcess struct spawns subprocess per tab
- IPC via serde-based message passing
- Process lifecycle managed by TabManager

Completes Milestone 2.1 (docs/plan/tasks.md)
Follows architecture.md section 3.1
```

**Required:**

- **ALWAYS reference milestone from docs/plan/tasks.md** when completing work:
  - "Completes Milestone 1.3" or "Part of Milestone 2.1"
  - Link to specific milestone section when possible
- **ALWAYS update docs/plan/tasks.md** to mark milestones/tasks complete
  - Change ⏳ to ✅ for completed items
  - Update status indicators in milestone sections

**Always Avoid:**

- Past tense ("added" → use "add")
- Time-based references (use milestone numbers: "Milestone 1.2", not "Week 3-4")
- Multiple unrelated changes in one commit
- Committing without running tests
- Committing without updating tasks.md status
- **Co-authoring lines** (no "Co-Authored-By:", no AI attribution footers)

---

## 3️⃣ Testing Phase

### Unit Testing

**Test Structure:**

```rust
// packages/storage/src/database.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_add_history_entry() {
        // Arrange
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();

        let entry = HistoryEntry {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            visit_time: Utc::now(),
        };

        // Act
        db.add_history(entry.clone()).unwrap();

        // Assert
        let history = db.get_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].url, entry.url);
    }

    #[test]
    fn test_concurrent_history_access() {
        use std::sync::Arc;
        use std::thread;

        let temp_file = NamedTempFile::new().unwrap();
        let db = Arc::new(Database::new(temp_file.path()).unwrap());

        let handles: Vec<_> = (0..10).map(|i| {
            let db = Arc::clone(&db);
            thread::spawn(move || {
                let entry = HistoryEntry {
                    url: format!("https://example{}.com", i),
                    title: format!("Example {}", i),
                    visit_time: Utc::now(),
                };
                db.add_history(entry).unwrap();
            })
        }).collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let history = db.get_history(100).unwrap();
        assert_eq!(history.len(), 10);
    }
}
```

### Integration Testing

**End-to-End Tab Management:**

```rust
// tests/integration/tab_management.rs
use browser_desktop::Browser;

#[test]
fn test_create_and_close_tabs() {
    let mut browser = Browser::new().unwrap();

    // Create 3 tabs
    let tab1 = browser.create_tab("https://example.com".to_string()).unwrap();
    let tab2 = browser.create_tab("https://wikipedia.org".to_string()).unwrap();
    let tab3 = browser.create_tab("https://github.com".to_string()).unwrap();

    assert_eq!(browser.tab_count(), 3);

    // Close middle tab
    browser.close_tab(tab2).unwrap();
    assert_eq!(browser.tab_count(), 2);

    // Remaining tabs should still work
    assert!(browser.get_tab(tab1).is_some());
    assert!(browser.get_tab(tab3).is_some());
}

#[tokio::test]
async fn test_concurrent_page_loads() {
    let mut browser = Browser::new().unwrap();

    let urls = vec![
        "https://example.com",
        "https://wikipedia.org",
        "https://github.com",
    ];

    let tab_ids: Vec<_> = urls.iter()
        .map(|url| browser.create_tab(url.to_string()).unwrap())
        .collect();

    // Wait for all pages to load
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Verify all tabs loaded successfully
    for id in tab_ids {
        let tab = browser.get_tab(id).unwrap();
        assert!(matches!(tab.state, TabState::Loaded(_)));
    }
}
```

### Performance Testing

**Benchmarking with Criterion:**

```rust
// benches/page_load.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use browser_renderer::WryRenderer;

fn bench_simple_page_load(c: &mut Criterion) {
    c.bench_function("load_simple_html", |b| {
        b.iter(|| {
            let mut renderer = WryRenderer::new();
            renderer.load_html(black_box("<html><body><h1>Test</h1></body></html>"));
        });
    });
}

fn bench_complex_page_load(c: &mut Criterion) {
    c.bench_function("load_wikipedia", |b| {
        b.iter(|| {
            let mut renderer = WryRenderer::new();
            renderer.load_url(black_box("https://en.wikipedia.org/wiki/Rust"));
        });
    });
}

criterion_group!(benches, bench_simple_page_load, bench_complex_page_load);
criterion_main!(benches);
```

**Memory Profiling:**

```bash
# Linux: valgrind
valgrind --tool=massif --massif-out-file=massif.out target/release/desktop
ms_print massif.out

# macOS: Instruments
instruments -t "Allocations" target/release/desktop

# Cross-platform: heaptrack (Linux)
heaptrack target/release/desktop
heaptrack_gui heaptrack.desktop.*.gz
```

### Platform-Specific Testing

**Test Matrix:**

| Platform    | Test Required         | Tools                           |
| ----------- | --------------------- | ------------------------------- |
| **Linux**   | Wayland + X11         | GitHub Actions (ubuntu-latest)  |
| **macOS**   | Intel + Apple Silicon | GitHub Actions (macos-latest)   |
| **Windows** | MSVC + GNU            | GitHub Actions (windows-latest) |

**CI Configuration** (`.github/workflows/ci.yml`):

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy, rustfmt

      - name: Install system dependencies (Linux)
        if: runner.os == 'Linux'
        run: |
          sudo apt update
          sudo apt install -y libx11-dev libxcb-dev libssl-dev

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Build
        run: cargo build --workspace --verbose

      - name: Test
        run: cargo test --workspace --verbose

      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Format check
        run: cargo fmt --all --check
```

---

## 4️⃣ Deployment Phase

### Building Release Binaries

**Linux:**

```bash
# Build optimized binary
cargo build --release

# Create AppImage (all dependencies bundled)
# Install appimagetool first
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x appimagetool-x86_64.AppImage

# Create AppDir structure
mkdir -p AppDir/usr/bin
cp target/release/desktop AppDir/usr/bin/browser
cp browser.desktop AppDir/
cp browser.png AppDir/

# Build AppImage
./appimagetool-x86_64.AppImage AppDir browser-x86_64.AppImage
```

**macOS:**

```bash
# Build universal binary (Intel + Apple Silicon)
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create universal binary
lipo -create \
  target/x86_64-apple-darwin/release/desktop \
  target/aarch64-apple-darwin/release/desktop \
  -output browser-universal

# Create .app bundle
mkdir -p Browser.app/Contents/{MacOS,Resources}
cp browser-universal Browser.app/Contents/MacOS/Browser
cp Info.plist Browser.app/Contents/
cp browser.icns Browser.app/Contents/Resources/

# Create DMG
hdiutil create -volname "Browser" -srcfolder Browser.app -ov -format UDZO browser.dmg
```

**Windows:**

```powershell
# Build release binary
cargo build --release

# Create installer with WiX Toolset
# Install WiX first: https://wixtoolset.org/
candle.exe browser.wxs
light.exe -out browser.msi browser.wixobj
```

### Release Checklist

Before releasing version X.Y.Z:

- [ ] All tests passing on all platforms
- [ ] Clippy warnings resolved
- [ ] Security audit clean (`cargo audit`)
- [ ] Version bumped in `Cargo.toml`
- [ ] CHANGELOG.md updated
- [ ] Documentation updated
- [ ] Benchmark results documented
- [ ] Memory leak testing (24-hour session)
- [ ] Cross-platform smoke tests:
  - [ ] Can open browser
  - [ ] Can navigate to https://example.com
  - [ ] Can create/close tabs
  - [ ] Can add bookmark
  - [ ] Settings persist
- [ ] Git tag created: `git tag -a v1.0.0 -m "Release v1.0.0"`
- [ ] Binaries built for all platforms
- [ ] Release notes written
- [ ] GitHub Release created with binaries attached

---

## 📚 Appendix

### Project Structure

```
browser/
├── apps/
│   └── desktop/              # Main desktop application
│       ├── src/
│       │   ├── main.rs       # Entry point
│       │   ├── browser.rs    # Browser core
│       │   ├── ui/           # egui UI components
│       │   └── ipc/          # IPC layer
│       └── Cargo.toml
│
├── packages/
│   ├── renderer/             # wry WebView wrapper
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── wry_renderer.rs
│   │   │   └── types.rs
│   │   └── Cargo.toml
│   │
│   ├── network/              # HTTP client
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── http.rs
│   │   │   └── cache.rs
│   │   └── Cargo.toml
│   │
│   ├── storage/              # SQLite storage
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── database.rs
│   │   │   └── migrations/
│   │   └── Cargo.toml
│   │
│   └── shared/               # Shared types
│       ├── src/
│       │   ├── lib.rs
│       │   └── types.rs
│       └── Cargo.toml
│
├── docs/
│   ├── plan/
│   │   ├── architecture.md
│   │   ├── requirements.md
│   │   └── tasks.md
│   └── guides/
│
├── tests/
│   ├── integration/
│   └── rendering/
│
├── benches/
│   └── page_load.rs
│
├── .github/
│   └── workflows/
│       └── ci.yml
│
├── Cargo.toml                # Workspace manifest
├── Cargo.lock
├── rust-toolchain.toml       # Rust version pinning
├── .rustfmt.toml
├── .clippy.toml
├── justfile                  # Command runner
└── README.md
```

### Common Dependencies

**Workspace Dependencies** (`Cargo.toml`):

```toml
[workspace.dependencies]
# Async runtime
tokio = { version = "1.40", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Rendering engine
tao = "0.30"
wry = "0.47"

# Storage
egui = "0.30"
eframe = { version = "0.30", features = ["persistence"] }
# OR: iced = "0.13"

# Graphics
wgpu = "24.0"
winit = "0.30"

# Networking
hyper = { version = "1.0", features = ["full"] }
hyper-rustls = "0.27"
reqwest = { version = "0.12", features = ["rustls-tls"] }

# Storage
rusqlite = { version = "0.32", features = ["bundled"] }
lru = "0.12"

# URL parsing
url = "2.5"

# Date/time
chrono = "0.4"
```

### Development Tools

**Install Recommended Tools:**

```bash
# Command runner
cargo install just

# Security audit
cargo install cargo-audit
cargo install cargo-deny

# Performance profiling
cargo install cargo-flamegraph

# Binary size analysis
cargo install cargo-bloat

# Code coverage
cargo install cargo-tarpaulin  # Linux only

# Watch for file changes
cargo install cargo-watch
```

**Usage:**

```bash
# Auto-rebuild on file change
cargo watch -x build

# Auto-test on file change
cargo watch -x test

# Flamegraph profiling (Linux)
cargo flamegraph --bin desktop

# Code coverage (Linux)
cargo tarpaulin --out Html
```

### Justfile Reference

**Example `justfile`:**

```makefile
# Default recipe (run if just 'just' is called)
default: check test

# Build all workspace crates
build:
    cargo build --workspace

# Build release (optimized)
build-release:
    cargo build --workspace --release

# Run the browser
run:
    cargo run -p desktop

# Run in release mode
run-release:
    cargo run -p desktop --release

# Run all tests
test:
    cargo test --workspace

# Run specific test
test-one name:
    cargo test {{name}} --workspace

# Check code (fast, no build)
check:
    cargo check --workspace
    cargo clippy --workspace -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all --check

# Lint with clippy
clippy:
    cargo clippy --workspace -- -D warnings

# Security audit
audit:
    cargo deny check
    cargo audit

# Benchmarks
bench:
    cargo bench --workspace

# Profile binary size
bloat:
    cargo bloat --release -n 20

# Clean build artifacts
clean:
    cargo clean

# Watch and rebuild on changes
watch:
    cargo watch -x build

# Full pre-commit check
pre-commit: fmt check test
    @echo "✅ Pre-commit checks passed!"
```

### Resources & References

**wry/Tauri Documentation:**

- wry GitHub: https://github.com/tauri-apps/wry
- Tauri Docs: https://tauri.app/
- tao (windowing): https://github.com/tauri-apps/tao
- Tauri Examples: https://github.com/tauri-apps/tauri/tree/dev/examples

**Rust GUI:**

- egui: https://github.com/emilk/egui
- iced: https://github.com/iced-rs/iced
- Tauri: https://tauri.app/ (webview wrapper, not for us)

**Browser Architecture:**

- Chromium Design Docs: https://www.chromium.org/developers/design-documents
- Firefox Source Docs: https://firefox-source-docs.mozilla.org/
- WebKit Architecture: https://webkit.org/code/

**Rust Learning:**

- The Rust Book: https://doc.rust-lang.org/book/
- Rust By Example: https://doc.rust-lang.org/rust-by-example/
- Async Book: https://rust-lang.github.io/async-book/

---

## 🎓 Development Best Practices Summary

**CRITICAL Requirements (MUST follow):**

- [ ] **Track ALL tasks in TodoWrite** - including bugs with severity
- [ ] **NEVER skip or mark tasks complete** without actually doing them
- [ ] **ALWAYS update docs/plan/tasks.md** - mark milestones ✅ when complete
- [ ] **ALWAYS reference milestone in commits** - "Completes Milestone X.Y (docs/plan/tasks.md)"
- [ ] **Use BLUF reporting style** - bottom line first, telegram brevity
- [ ] **Run clippy before every commit** - fix ALL warnings
- [ ] **Format code with rustfmt** - consistent style
- [ ] **Test on target platform** - not just cross-compile

**Quick Checklist for Every Task:**

- [ ] Use Context7 for Rust crate documentation
- [ ] Check wry/Tauri examples for API usage patterns
- [ ] Write unit tests for new functions
- [ ] Run `just check` before committing
- [ ] Use Rust ownership to encode invariants
- [ ] Handle errors with Result<T, E>, never unwrap in library code
- [ ] Document public APIs with doc comments (`///`)
- [ ] Profile performance with criterion/flamegraph
- [ ] Test on all platforms (CI covers this)
- [ ] **Mark milestone complete in docs/plan/tasks.md** (⏳ → ✅)
- [ ] **Reference milestone in commit message**

**When in Doubt:**

1. **wry API unclear?** → Check Tauri examples, search GitHub issues
2. **Build failing?** → Check system deps, read full error message
3. **Clippy warning?** → Read the suggestion, usually correct
4. **Performance issue?** → Profile first with flamegraph, optimize hot paths
5. **Ownership error?** → Draw lifetime diagram, clone if needed (profile later)
6. **GUI not responding?** → Check if blocking main thread, move to async task
7. **Memory leak?** → Use heaptrack/valgrind, check for reference cycles
8. **Stuck/Blocked?** → Ask user for help - don't skip the task

---

**Last Updated:** 2025-10-25
**Project Status:** Phase 1: Foundation - Milestone 1.2 Complete

**Current Milestone:** 1.3 - WebView Integration (Complete)

**Completed:**

- ✅ Milestone 1.1: Project Setup & Tooling
- ✅ Milestone 1.2: Basic UI (tabs, URL bar, navigation)
- ✅ Milestone 1.3: wry WebView Integration (platform WebView rendering)

**Architecture Decision:**

- Using wry v0.47 (platform WebView wrapper) for MVP
- Powers production apps via Tauri framework
- Can migrate to libservo when v1.0 releases (6-12 months)

**Implementation Status:**

- Phase 1: Foundation (6 milestones) - 2/6 complete
- Phase 2: Core Features (6 milestones) - Not Started
- Phase 3: Polish (6 milestones) - Not Started
- Phase 4: Launch (4 milestones) - Not Started
