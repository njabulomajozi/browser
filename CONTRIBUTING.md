# Contributing to Browser MVP

Thank you for your interest in contributing! This document provides guidelines for contributing to the Browser MVP project.

---

## 📋 Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Code Standards](#code-standards)
5. [Commit Guidelines](#commit-guidelines)
6. [Pull Request Process](#pull-request-process)
7. [Testing Requirements](#testing-requirements)

---

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Assume good intentions

---

## Getting Started

### Prerequisites

1. **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
2. **just**: `cargo install just`
3. **System dependencies**:
   - Linux: `sudo apt install libx11-dev libxcb-dev libssl-dev pkg-config`
   - macOS: `brew install cmake pkg-config`
   - Windows: Visual Studio Build Tools

### Setup

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/browser
cd browser

# Build and test
just build
just test
```

---

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/my-feature
# or
git checkout -b fix/my-bugfix
```

Branch naming conventions:
- `feature/*` - New features
- `fix/*` - Bug fixes
- `docs/*` - Documentation updates
- `refactor/*` - Code refactoring
- `test/*` - Test additions/fixes

### 2. Make Changes

Follow the [Code Standards](#code-standards) below.

### 3. Test Locally

```bash
# Format code
just fmt

# Run clippy
just clippy

# Run tests
just test

# Full pre-commit check
just pre-commit
```

### 4. Commit

Follow [Commit Guidelines](#commit-guidelines) below.

### 5. Push and Create PR

```bash
git push origin feature/my-feature
```

Then create a pull request on GitHub.

---

## Code Standards

### Rust Code Style

**Formatting**: Use `rustfmt` (automatically configured)

```bash
just fmt
```

**Linting**: Fix all `clippy` warnings

```bash
just clippy
```

**Code Quality**:
- Clear variable and function names
- Functions < 100 lines (guideline)
- Use Rust type system to encode invariants
- Document public APIs with `///` comments
- Use `#[must_use]` for important return values

### Error Handling

**Use `Result` for fallible operations:**

```rust
// ✅ Good
pub fn load_url(&mut self, url: &str) -> Result<(), BrowserError> {
    let parsed = Url::parse(url)?;
    self.renderer.load(parsed)?;
    Ok(())
}

// ❌ Bad - panics
pub fn load_url(&mut self, url: &str) {
    let parsed = Url::parse(url).unwrap();
    self.renderer.load(parsed).unwrap();
}
```

**Use `anyhow` for application-level errors:**

```rust
use anyhow::{Result, Context};

pub fn initialize(&self) -> Result<()> {
    self.setup()
        .context("Failed to initialize browser")?;
    Ok(())
}
```

### Documentation

**Document public APIs:**

```rust
/// Loads a URL in the current tab
///
/// # Arguments
///
/// * `url` - The URL to load
///
/// # Errors
///
/// Returns `BrowserError::InvalidUrl` if the URL is malformed
pub fn load_url(&mut self, url: &str) -> Result<(), BrowserError> {
    // ...
}
```

**Add module-level documentation:**

```rust
//! Browser rendering engine wrapper
//!
//! This module provides a simplified API for the Servo rendering engine.
```

---

## Commit Guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/).

### Format

```
<type>(<scope>): <subject>

<body>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvement
- `test`: Adding or updating tests
- `build`: Build system or dependencies
- `ci`: CI configuration
- `chore`: Other changes (tools, etc.)

### Scopes

- `renderer`: Rendering engine
- `ui`: User interface
- `network`: Networking layer
- `storage`: Database/storage
- `desktop`: Desktop application
- `servo`: Servo integration
- `docs`: Documentation
- `ci`: CI/CD

### Examples

```
feat(renderer): add multi-tab process isolation

Each tab now runs in separate renderer process for stability.
If one tab crashes, others continue working.

Implementation:
- RendererProcess struct spawns subprocess per tab
- IPC via serde-based message passing
- Process lifecycle managed by TabManager
```

```
fix(network): handle connection timeout gracefully

Previously crashed on network timeout. Now returns error
and shows user-friendly message.

Fixes #123
```

---

## Pull Request Process

### Before Creating PR

1. ✅ Code formatted: `just fmt`
2. ✅ All clippy warnings fixed: `just clippy`
3. ✅ All tests passing: `just test`
4. ✅ New tests added for new features
5. ✅ Documentation updated if needed

### PR Description

**Title**: Use conventional commit format

```
feat(renderer): add multi-tab support
```

**Description Template**:

```markdown
## Summary
Brief description of changes

## Changes
- Added X
- Fixed Y
- Updated Z

## Testing
How was this tested?

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Clippy warnings fixed
- [ ] All tests passing
```

### Review Process

1. Automated CI checks must pass
2. At least one maintainer approval required
3. Address review feedback
4. Squash commits if needed

---

## Testing Requirements

### Unit Tests

**Add tests for new functions:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parsing() {
        let url = parse_url("https://example.com").unwrap();
        assert_eq!(url.scheme(), "https");
    }
}
```

### Integration Tests

**Place in `tests/` directory:**

```rust
// tests/integration/navigation.rs
use browser_desktop::Browser;

#[test]
fn test_navigation() {
    let mut browser = Browser::new().unwrap();
    browser.navigate("https://example.com").unwrap();
    // ...
}
```

### Running Tests

```bash
# All tests
just test

# Specific test
just test-one test_name

# With output
cargo test -- --nocapture
```

---

## Questions?

- Open a [GitHub Discussion](https://github.com/yourusername/browser/discussions)
- Check [CLAUDE.md](CLAUDE.md) for development workflow details
- Review [docs/plan/](docs/plan/) for architecture and roadmap

---

Thank you for contributing! 🎉
