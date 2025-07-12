@echo off
REM Local CI Testing Script for Rust-Spray (Windows Batch Version)
REM This script runs the same checks that the GitHub Actions CI runs

setlocal enabledelayedexpansion

echo 🚀 Starting Local CI Checks for Rust-Spray
echo ===========================================

REM Check if we're in the right directory
if not exist "Cargo.toml" (
    echo [ERROR] Cargo.toml not found. Please run this script from the project root.
    exit /b 1
)

REM Check if Rust is installed
where cargo >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Cargo not found. Please install Rust: https://rustup.rs/
    exit /b 1
)

echo [INFO] Rust version:
rustc --version

echo [INFO] Cargo version:
cargo --version

echo.
echo [INFO] 1. FORMATTING CHECK
echo ===================
echo [INFO] Running: Code formatting check
cargo fmt --all -- --check
if errorlevel 1 (
    echo [ERROR] Formatting check failed
    echo [WARNING] To fix formatting issues, run: cargo fmt --all
    set "failed_checks=!failed_checks! formatting"
) else (
    echo [SUCCESS] Formatting check passed
)

echo.
echo [INFO] 2. CLIPPY ANALYSIS
echo ==================

REM Install clippy if not available
rustup component list --installed | findstr clippy >nul
if errorlevel 1 (
    echo [INFO] Installing clippy component...
    rustup component add clippy
)

echo [INFO] Running: Clippy (host features)
cargo clippy --all-targets --features host -- -D warnings
if errorlevel 1 (
    echo [ERROR] Clippy (host features) failed
    set "failed_checks=!failed_checks! clippy-host"
) else (
    echo [SUCCESS] Clippy (host features) passed
)

echo [INFO] Running: Clippy (no features)
cargo clippy --all-targets --no-default-features -- -D warnings
if errorlevel 1 (
    echo [ERROR] Clippy (no features) failed
    set "failed_checks=!failed_checks! clippy-no-features"
) else (
    echo [SUCCESS] Clippy (no features) passed
)

echo [INFO] Running: Clippy (raspberry-pi features)
cargo clippy --all-targets --features raspberry-pi -- -D warnings
if errorlevel 1 (
    echo [ERROR] Clippy (raspberry-pi features) failed
    set "failed_checks=!failed_checks! clippy-rpi"
) else (
    echo [SUCCESS] Clippy (raspberry-pi features) passed
)

echo.
echo [INFO] 3. BUILD TESTS
echo ==============

echo [INFO] Running: Build (host features)
cargo build --verbose --features host
if errorlevel 1 (
    echo [ERROR] Build (host features) failed
    set "failed_checks=!failed_checks! build-host"
) else (
    echo [SUCCESS] Build (host features) passed
)

echo [INFO] Running: Build (no features)
cargo build --verbose --no-default-features
if errorlevel 1 (
    echo [ERROR] Build (no features) failed
    set "failed_checks=!failed_checks! build-no-features"
) else (
    echo [SUCCESS] Build (no features) passed
)

echo [INFO] Running: Build (raspberry-pi features)
cargo build --verbose --features raspberry-pi
if errorlevel 1 (
    echo [ERROR] Build (raspberry-pi features) failed
    set "failed_checks=!failed_checks! build-rpi"
) else (
    echo [SUCCESS] Build (raspberry-pi features) passed
)

echo.
echo [INFO] 4. UNIT TESTS
echo =============

echo [INFO] Running: Tests (host features)
cargo test --verbose --features host
if errorlevel 1 (
    echo [ERROR] Tests (host features) failed
    set "failed_checks=!failed_checks! test-host"
) else (
    echo [SUCCESS] Tests (host features) passed
)

echo [INFO] Running: Tests (no features)
cargo test --verbose --no-default-features
if errorlevel 1 (
    echo [ERROR] Tests (no features) failed
    set "failed_checks=!failed_checks! test-no-features"
) else (
    echo [SUCCESS] Tests (no features) passed
)

echo.
echo [INFO] 5. EXAMPLE COMPILATION
echo ======================

echo [INFO] Running: Example (basic_usage)
cargo check --example basic_usage --features host
if errorlevel 1 (
    echo [ERROR] Example compilation failed
    set "failed_checks=!failed_checks! example"
) else (
    echo [SUCCESS] Example compilation passed
)

echo.
echo [INFO] 6. DOCUMENTATION BUILD
echo ======================

echo [INFO] Running: Documentation (host features)
cargo doc --no-deps --features host
if errorlevel 1 (
    echo [ERROR] Documentation build failed
    set "failed_checks=!failed_checks! docs"
) else (
    echo [SUCCESS] Documentation build passed
)

echo.
echo [INFO] 7. SECURITY AUDIT
echo =================

REM Check if cargo-audit is installed
where cargo-audit >nul 2>&1
if errorlevel 1 (
    echo [INFO] Installing cargo-audit...
    cargo install cargo-audit
)

echo [INFO] Running: Security audit
cargo audit
if errorlevel 1 (
    echo [ERROR] Security audit failed
    set "failed_checks=!failed_checks! security"
) else (
    echo [SUCCESS] Security audit passed
)

echo.
echo 🏁 CI CHECK SUMMARY
echo ===================

if "!failed_checks!"=="" (
    echo [SUCCESS] All CI checks passed! ✅
    echo [SUCCESS] Your code is ready for CI/CD pipeline.
    exit /b 0
) else (
    echo [ERROR] Failed checks:!failed_checks!
    echo [ERROR] Please fix the above issues before pushing to CI.
    exit /b 1
)
