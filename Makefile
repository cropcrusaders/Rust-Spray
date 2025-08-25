# Makefile for Rust-Spray Local CI Testing
# This provides convenient commands to run CI checks locally

.PHONY: help ci-all format clippy build test docs audit clean install-deps

# Default target
help:
	@echo "Rust-Spray Local CI Commands"
	@echo "============================="
	@echo ""
	@echo "Available targets:"
	@echo "  ci-all      - Run all CI checks (equivalent to GitHub Actions)"
	@echo "  format      - Check code formatting"
	@echo "  clippy      - Run clippy linting for all feature combinations"
	@echo "  build       - Build all feature combinations"
	@echo "  test        - Run tests for all feature combinations"
	@echo "  docs        - Build documentation"
	@echo "  audit       - Run security audit"
	@echo "  clean       - Clean build artifacts"
	@echo "  install-deps- Install required tools (clippy, cargo-audit)"
	@echo ""
	@echo "Quick checks:"
	@echo "  make format clippy  - Run formatting and linting only"
	@echo "  make build test     - Run build and test only"

# Install required tools
install-deps:
	@echo "Installing required Rust components and tools..."
	rustup component add clippy rustfmt
	cargo install cargo-audit

# Run all CI checks (matches GitHub Actions)
ci-all: format clippy build test docs audit
	@echo "✅ All CI checks completed successfully!"

# Format check
format:
	@echo "🔍 Checking code formatting..."
	cargo fmt --all -- --check

# Format fix
format-fix:
	@echo "🔧 Fixing code formatting..."
	cargo fmt --all

# Clippy analysis for all feature combinations
clippy:
	@echo "🔍 Running clippy analysis..."
	@echo "  • Host features (with OpenCV)..."
	cargo clippy --all-targets --features host -- -D warnings
	@echo "  • No features..."
	cargo clippy --all-targets --no-default-features -- -D warnings
	@echo "  • Raspberry Pi features..."
	cargo clippy --all-targets --features raspberry-pi -- -D warnings

# Build all feature combinations
build:
	@echo "🔨 Building all feature combinations..."
	@echo "  • Host features (with OpenCV)..."
	cargo build --verbose --features host
	@echo "  • No features..."
	cargo build --verbose --no-default-features
	@echo "  • Raspberry Pi features..."
	cargo build --verbose --features raspberry-pi

# Run tests for all feature combinations
test:
	@echo "🧪 Running tests..."
	@echo "  • Host features (with OpenCV)..."
	cargo test --verbose --features host
	@echo "  • No features..."
	cargo test --verbose --no-default-features

# Build documentation
docs:
	@echo "📚 Building documentation..."
	cargo doc --no-deps --features host

# Security audit
audit:
	@echo "🔒 Running security audit..."
	cargo audit

# Example compilation
examples:
	@echo "📋 Checking examples..."
	cargo check --example basic_usage --features host

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Quick development checks (faster subset)
dev-check: format clippy
	@echo "✅ Quick development checks completed!"

# Pre-commit checks (thorough but not all CI)
pre-commit: format clippy build
	@echo "✅ Pre-commit checks completed!"

# Release preparation
release-check: ci-all
	@echo "🚀 Release checks completed!"

# Cross-compilation check (if cross is installed)
cross-compile:
	@echo "🔀 Cross-compiling for ARM targets..."
	@if command -v cross >/dev/null 2>&1; then \
		echo "  • aarch64-unknown-linux-gnu..."; \
		cross build --target aarch64-unknown-linux-gnu --features raspberry-pi; \
		echo "  • armv7-unknown-linux-gnueabihf..."; \
		cross build --target armv7-unknown-linux-gnueabihf --features raspberry-pi; \
	else \
		echo "❌ 'cross' tool not installed. Install with: cargo install cross"; \
		exit 1; \
	fi

# Install cross-compilation tool
install-cross:
	./scripts/cross-build.sh
        cargo install cross --git https://github.com/cross-rs/cross

# Show project info
info:
	@echo "📊 Project Information"
	@echo "======================"
	@echo "Rust version: $$(rustc --version)"
	@echo "Cargo version: $$(cargo --version)"
	@echo "Project: $$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].name')"
	@echo "Version: $$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')"
	@echo ""
	@echo "Available features:"
        @cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].features | keys[]' | sort

# Watch mode for development (requires cargo-watch)
watch:
        @if command -v cargo-watch >/dev/null 2>&1; then \
                echo "👀 Starting watch mode..."; \
                cargo watch -x "check --features host" -x "test --features host"; \
        else \
                echo "❌ 'cargo-watch' not installed. Install with: cargo install cargo-watch"; \
                exit 1; \
        fi

# Simplified cross build targets
.PHONY: cross pi-aarch64 armv7

cross:
	./scripts/cross-build.sh

pi-aarch64:
	cross build --release --target aarch64-unknown-linux-gnu

armv7:
	cross build --release --target armv7-unknown-linux-gnueabihf
