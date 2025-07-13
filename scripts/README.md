# Local CI Testing Scripts

This directory contains scripts to run the same CI checks locally that are performed in the GitHub Actions workflow.

## 📋 Available Scripts

### 🐧 Linux/macOS: `ci-local.sh`
Bash script for Unix-like systems.

```bash
# Make executable and run all checks
chmod +x scripts/ci-local.sh
./scripts/ci-local.sh

# Run specific check types
./scripts/ci-local.sh --format-only
./scripts/ci-local.sh --clippy-only
./scripts/ci-local.sh --build-only
./scripts/ci-local.sh --test-only
./scripts/ci-local.sh --help
```

### 🪟 Windows: `ci-local.ps1`
PowerShell script for Windows systems.

```powershell
# Run all checks
.\scripts\ci-local.ps1

# Run specific check types
.\scripts\ci-local.ps1 -FormatOnly
.\scripts\ci-local.ps1 -ClippyOnly
.\scripts\ci-local.ps1 -BuildOnly
.\scripts\ci-local.ps1 -TestOnly
.\scripts\ci-local.ps1 -Help
```

### 🪟 Windows: `ci-local.bat`
Batch file for Windows Command Prompt.

```cmd
# Run all checks
scripts\ci-local.bat
```

### 🛠️ Make: `Makefile` (Root Directory)
Convenient make targets for development.

```bash
# Show available commands
make help

# Run all CI checks
make ci-all

# Quick development checks
make format clippy

# Individual checks
make format
make clippy
make build
make test
make docs
make audit

# Install required tools
make install-deps
```

## ✅ What Gets Tested

All scripts test the same things as the GitHub Actions CI:

1. **Code Formatting** - `cargo fmt --all -- --check`
2. **Clippy Analysis** - Multiple feature combinations:
   - Host features (with OpenCV)
   - No features
   - Raspberry Pi features
3. **Build Tests** - All feature combinations
4. **Unit Tests** - Host and no-features scenarios
5. **Example Compilation** - `basic_usage` example
6. **Documentation Build** - With host features
7. **Security Audit** - `cargo audit`

## 🚀 Quick Start

### Option 1: Use Make (Recommended)
```bash
# Install required tools
make install-deps

# Run all CI checks
make ci-all
```

### Option 2: Use Platform Script
```bash
# Linux/macOS
./scripts/ci-local.sh

# Windows PowerShell
.\scripts\ci-local.ps1

# Windows Command Prompt
scripts\ci-local.bat
```

## 📦 Prerequisites

### Required:
- [Rust](https://rustup.rs/) toolchain
- Git

### Auto-installed by scripts:
- `clippy` component
- `cargo-audit` tool

### For OpenCV features (Linux):
```bash
# Ubuntu/Debian
sudo apt-get install libopencv-dev clang libclang-dev pkg-config

# Fedora/RHEL
sudo dnf install opencv-devel clang-devel

# macOS
brew install opencv pkg-config
```

### For cross-compilation (optional):
```bash
# Install cross tool
cargo install cross --git https://github.com/cross-rs/cross

# Or use make target
make install-cross
```

## 🔧 Development Workflow

### Before Committing:
```bash
# Quick checks
make format clippy

# Or comprehensive pre-commit
make pre-commit
```

### Before Creating PR:
```bash
# Full CI simulation
make ci-all
```

### During Development:
```bash
# Watch mode (requires cargo-watch)
make watch

# Auto-fix formatting
make format-fix
```

## 🐛 Troubleshooting

### Common Issues:

1. **OpenCV not found**: Install system OpenCV development packages
2. **Clippy not available**: Run `rustup component add clippy`
3. **cargo-audit missing**: Scripts auto-install, or run `cargo install cargo-audit`
4. **Permission denied (Linux)**: Run `chmod +x scripts/ci-local.sh`

### Script Debugging:
All scripts provide colored output and detailed error messages. Failed checks are clearly listed at the end.

### Feature Combinations:
- **host**: Includes OpenCV, for development/testing
- **raspberry-pi**: ARM-specific features, no OpenCV
- **no features**: Minimal build for compatibility testing

## 📊 Exit Codes

- `0`: All checks passed
- `1`: One or more checks failed

Use in CI/automation:
```bash
if ./scripts/ci-local.sh; then
    echo "Ready to push!"
else
    echo "Fix issues before pushing"
    exit 1
fi
```
