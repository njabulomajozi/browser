# Browser MVP

**A cross-platform web browser built with Rust and wry (platform WebView wrapper).**

[![CI](https://github.com/yourusername/browser/workflows/CI/badge.svg)](https://github.com/yourusername/browser/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

---

## 🎯 Project Status

**Phase 1: Foundation** 🚧 In Progress

- ✅ Cargo workspace structure (5 crates)
- ✅ CI/CD pipeline
- ✅ Linting and formatting configuration
- ✅ Basic UI with egui (tabs, URL bar, navigation)
- ✅ wry WebView integration (platform-native rendering)
- ⏳ URL navigation functionality

**Current Focus**: Building browser UI with wry WebView

---

## 🚀 Quick Start

### 🆘 First-Time Setup Issues?

**Just installed Rust and getting `cargo: command not found`?**

→ **RESTART YOUR TERMINAL** or run: `source "$HOME/.cargo/env"`

**Build failing with weird errors?**

→ Check [Troubleshooting section](#-troubleshooting) below

---

### Prerequisites (macOS)

#### Step 1: Install Homebrew

```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Verify installation
brew --version
```

#### Step 2: Install Rust

```bash
# Install Rust and rustup via Homebrew
brew install rust rustup-init
rustup-init

# Install nightly toolchain (required for this project)
rustup toolchain install nightly

# Verify installation
cargo --version
rustc --version
rustup toolchain list | grep nightly
```

#### Step 3: Install Build Tools

```bash
# Install cmake and pkg-config
brew install cmake pkg-config

# Verify installation
cmake --version
pkg-config --version
```

#### Step 4: Install just (Optional)

```bash
# Install just command runner
brew install just

# Verify installation
just --version
```

**Note**: `just` is optional - you can use `cargo` commands directly (see justfile).

#### Step 5: Verify Everything

```bash
# Check all installations
brew --version          # Homebrew
cargo --version         # Rust
rustc --version         # Rust compiler
cmake --version         # CMake
pkg-config --version    # pkg-config
just --version          # just (if installed)

# Verify nightly toolchain
rustup toolchain list | grep nightly
```

✅ If all commands work, you're ready to build!

---

### Build and Run

```bash
# Clone the repository
git clone https://github.com/yourusername/browser
cd browser

# Build the project
# Note: First build downloads ~511 packages and takes 2-3 minutes
# Subsequent builds take <1 second
just build

# Run tests
just test

# Run the browser
just run
```

**First Build Notes:**
- Downloads and compiles 511+ dependencies
- Build time: 2-3 minutes on Apple Silicon, may vary
- Disk space: ~2GB for dependencies and build artifacts

### Development Commands

```bash
just                # Run checks and tests
just build          # Build all crates
just test           # Run all tests
just check          # Fast code check + clippy
just fmt            # Format code
just run            # Run the browser
just pre-commit     # Full pre-commit checks
```

See `just --list` for all available commands.

---

## 📁 Project Structure

```
browser/
├── apps/
│   └── desktop/              # Main desktop application
├── packages/
│   ├── renderer/             # Servo rendering engine wrapper
│   ├── network/              # HTTP client and caching
│   ├── storage/              # SQLite database layer
│   └── shared/               # Shared types and utilities
├── docs/
│   └── plan/                 # Planning documentation
│       ├── architecture.md   # System design
│       ├── requirements.md   # Requirements and decisions
│       └── tasks.md          # Implementation roadmap
├── tests/                    # Integration tests
├── benches/                  # Performance benchmarks
└── .github/workflows/        # CI/CD configuration
```

---

## 🛠️ Tech Stack

- **Language**: Rust 1.70+ (memory safety, performance)
- **Rendering Engine**: Servo (non-Chromium, standards-compliant)
- **UI Framework**: egui (immediate-mode GUI)
- **Graphics**: wgpu (WebGPU for hardware acceleration)
- **Networking**: hyper + reqwest (HTTP/HTTPS)
- **Storage**: SQLite via rusqlite (history, bookmarks)
- **Async Runtime**: tokio (non-blocking I/O)

---

## 🎓 Development Workflow

### Before Committing

```bash
# Format code
just fmt

# Run all checks
just check

# Run tests
just test

# Full pre-commit validation
just pre-commit
```

### Testing

```bash
# Run all tests
just test

# Run specific test
just test-one test_name

# Run tests with output
cargo test -- --nocapture
```

### Platform Testing

CI tests on macOS (Intel + Apple Silicon)

---

## 🗺️ Roadmap

### Phase 1: Foundation (6 milestones)
- ✅ Project setup & tooling
- ✅ Basic UI (tabs, URL bar, navigation)
- ⏳ Servo rendering engine integration
- ⏳ URL navigation and page loading
- ⏳ Back/Forward navigation history
- ⏳ Persistent storage (SQLite)

### Phase 2: Core Features (6 milestones)
- Multi-tab support with process isolation
- Bookmarks management UI
- Settings and preferences page
- Developer console for debugging
- HTTP client with intelligent caching
- Performance profiling and optimization

### Phase 3: Polish (6 milestones)
- Security hardening and sandboxing
- Accessibility features (screen readers, keyboard nav)
- Comprehensive cross-platform testing
- Memory optimization and leak prevention
- UI/UX refinement
- User and developer documentation

### Phase 4: Launch (4 milestones)
- Private beta testing program
- Public beta release
- Performance testing at scale
- Security audit
- v1.0 stable release

See [docs/plan/tasks.md](docs/plan/tasks.md) for detailed roadmap.

---

## 📖 Documentation

- [Architecture](docs/plan/architecture.md) - System design and component interaction
- [Requirements](docs/plan/requirements.md) - Technical and functional requirements
- [Tasks](docs/plan/tasks.md) - Milestone-driven implementation roadmap
- [CLAUDE.md](CLAUDE.md) - Development workflow guide

---

## 🔧 Troubleshooting

### Homebrew Issues

**❌ Error: `brew: command not found`**
```bash
Problem: Homebrew not installed or not in PATH

Solution: Install Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Follow post-install instructions to add brew to PATH
# Usually: echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile

# Verify
brew --version
```

### Rust Issues

**❌ Error: `cargo: command not found`** ← Most common!
```bash
Problem: Cargo not in PATH after installation

Solution: RESTART YOUR TERMINAL
# Or reinstall:
brew uninstall rust rustup-init
brew install rust rustup-init
rustup-init

# Verify
cargo --version
rustc --version
```

**❌ Error: `feature edition2024 is required`**
```bash
Problem: Missing nightly toolchain

Solution:
rustup toolchain install nightly
rustup show  # Verify project uses nightly
```

### Build Tool Issues

**❌ Error: `cmake not found` or `pkg-config not found`**
```bash
Problem: Build tools not installed

Solution:
brew install cmake pkg-config

# Verify
cmake --version
pkg-config --version
```

### Runtime Issues

**Warning: `fields id and url are never read`**
```
Status: Non-critical warning, safe to ignore
These fields will be used in future Servo integration
```

**Browser window doesn't open**
```
Check:
1. Build completed without errors
2. No other instance running
3. Graphics drivers up to date (wgpu/OpenGL requirement)
```

**Slow first build**
```
Expected: First build takes 2-3 minutes
- Downloads 511+ crate dependencies
- Compiles egui, wgpu, tokio, and other large dependencies
- Subsequent builds are <1 second
```

### Getting Help

- Check [docs/plan/architecture.md](docs/plan/architecture.md) for technical details
- Review [CLAUDE.md](CLAUDE.md) for development workflow
- Report issues: [GitHub Issues](https://github.com/yourusername/browser/issues)

---

## 🤝 Contributing

We welcome contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Getting Started

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `just test`
5. Run pre-commit checks: `just pre-commit`
6. Commit with conventional commits: `git commit -m "feat(renderer): add feature"`
7. Push and create a pull request

---

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

## 🙏 Acknowledgments

- [Servo](https://servo.org/) - Modern browser engine
- [egui](https://github.com/emilk/egui) - Immediate-mode GUI
- [Rust](https://www.rust-lang.org/) - Memory-safe systems programming

---

## 📬 Contact

- Issues: [GitHub Issues](https://github.com/yourusername/browser/issues)
- Discussions: [GitHub Discussions](https://github.com/yourusername/browser/discussions)

---

**Status**: Phase 1 Foundation in Progress | **Next**: Servo Integration
