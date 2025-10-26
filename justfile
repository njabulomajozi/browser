# Browser MVP - Just command runner
# Run `just` to see available commands

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
    cargo deny check advisories
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

# Install development dependencies
install-deps:
    cargo install just cargo-watch cargo-audit cargo-deny cargo-bloat

# Show workspace structure
tree:
    @echo "Browser Workspace Structure:"
    @tree -L 3 -I target
