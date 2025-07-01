# Build verification script for Rust-Spray project

Write-Host "🔧 Rust-Spray Build Verification" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan

# Check if Cargo.toml exists and is valid
if (Test-Path "Cargo.toml") {
    Write-Host "✅ Cargo.toml found" -ForegroundColor Green
} else {
    Write-Host "❌ Cargo.toml not found" -ForegroundColor Red
    exit 1
}

# Check if source files exist
Write-Host "📁 Checking source files..." -ForegroundColor Yellow
$files = @(
    "src/main.rs",
    "src/lib.rs", 
    "src/config.rs",
    "src/camera.rs",
    "src/detection.rs",
    "src/spray.rs",
    "src/utils/mod.rs",
    "src/utils/algorithms.rs"
)

foreach ($file in $files) {
    if (Test-Path $file) {
        Write-Host "✅ $file" -ForegroundColor Green
    } else {
        Write-Host "❌ $file missing" -ForegroundColor Red
    }
}

# Check if config file exists
if (Test-Path "config/Config.toml") {
    Write-Host "✅ config/Config.toml" -ForegroundColor Green
} else {
    Write-Host "⚠️  config/Config.toml not found (needed for runtime)" -ForegroundColor Yellow
}

# Check if tests exist
Write-Host "🧪 Checking tests..." -ForegroundColor Yellow
if (Test-Path "tests/integration_tests.rs") {
    Write-Host "✅ integration_tests.rs" -ForegroundColor Green
}

if (Test-Path "tests/config_tests.rs") {
    Write-Host "✅ config_tests.rs" -ForegroundColor Green
}

# Check if examples exist
if (Test-Path "examples/basic_usage.rs") {
    Write-Host "✅ examples/basic_usage.rs" -ForegroundColor Green
}

Write-Host ""
Write-Host "📋 Build Commands:" -ForegroundColor Cyan
Write-Host "=================="
Write-Host "Standard build:          cargo build"
Write-Host "Release build:           cargo build --release"
Write-Host "With GPIO support:       cargo build --features with-rppal"
Write-Host "With Pi camera:          cargo build --features picam"
Write-Host "Full Pi support:         cargo build --features raspberry-pi"
Write-Host "Cross compile (ARM64):   cross build --target aarch64-unknown-linux-gnu --release"
Write-Host "Cross compile (ARMv7):   cross build --target armv7-unknown-linux-gnueabihf --release"
Write-Host ""
Write-Host "🧪 Test Commands:" -ForegroundColor Cyan
Write-Host "================="
Write-Host "Run tests:               cargo test"
Write-Host "Run with features:       cargo test --features with-rppal"
Write-Host ""
Write-Host "📖 Example Commands:" -ForegroundColor Cyan
Write-Host "===================="
Write-Host "Run example:             cargo run --example basic_usage"
Write-Host ""
Write-Host "🚀 Ready for compilation!" -ForegroundColor Green
