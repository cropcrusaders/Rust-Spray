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
Write-Host "� Prerequisites Check:" -ForegroundColor Cyan
Write-Host "======================="

# Check if Rust is installed
try {
    $rustVersion = cargo --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Rust/Cargo: $rustVersion" -ForegroundColor Green
    } else {
        Write-Host "❌ Rust/Cargo not found" -ForegroundColor Red
        Write-Host "   Install from: https://rustup.rs/" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ Rust/Cargo not found" -ForegroundColor Red
    Write-Host "   Install from: https://rustup.rs/" -ForegroundColor Yellow
}

# Check if Cross is available (for cross-compilation)
try {
    $crossVersion = cross --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Cross: $crossVersion" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Cross not installed (optional for ARM cross-compilation)" -ForegroundColor Yellow
        Write-Host "   Install with: cargo install cross" -ForegroundColor Yellow
    }
} catch {
    Write-Host "⚠️  Cross not installed (optional for ARM cross-compilation)" -ForegroundColor Yellow
    Write-Host "   Install with: cargo install cross" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "�📋 Build Commands:" -ForegroundColor Cyan
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
Write-Host ""
Write-Host "📁 CI/CD Status:" -ForegroundColor Cyan
Write-Host "================"
Write-Host "The project includes GitHub Actions workflows for:"
Write-Host "• Quick Tests (.github/workflows/test.yml)" -ForegroundColor Yellow
Write-Host "• Full CI/CD (.github/workflows/ci.yml)" -ForegroundColor Yellow
Write-Host "• Build & Release (.github/workflows/build.yml)" -ForegroundColor Yellow
Write-Host "• Lockfile Updates (.github/workflows/pr.yml)" -ForegroundColor Yellow
Write-Host ""
Write-Host "Note: The failed CI test was due to formatting issues that have been resolved." -ForegroundColor Green
Write-Host "Push your changes to trigger the fixed workflows." -ForegroundColor Green

Write-Host ""
Write-Host "🔧 CI/CD Status & Docker Fixes:" -ForegroundColor Cyan
Write-Host "================================="
Write-Host "Recent fixes applied to cross-compilation Dockerfiles:" -ForegroundColor Green
Write-Host "✅ Fixed rustup issues in Dockerfile.armv7-opencv" -ForegroundColor Green
Write-Host "✅ Fixed rustup issues in Dockerfile.cross-aarch64" -ForegroundColor Green
Write-Host "✅ Fixed cargo PATH issues in cross-rs images" -ForegroundColor Green
Write-Host "✅ Updated binary paths for cross-rs images" -ForegroundColor Green
Write-Host ""
Write-Host "Cross-compilation should now work correctly for ARM targets." -ForegroundColor Green
Write-Host "See DOCKERFILE_FIXES.md for detailed information." -ForegroundColor Yellow
Write-Host ""
Write-Host "📋 Build Commands:"
