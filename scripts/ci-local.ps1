# Local CI Testing Script for Rust-Spray (PowerShell Version)
# This script runs all the same checks that the GitHub Actions CI runs

param(
    [switch]$FormatOnly,
    [switch]$ClippyOnly,
    [switch]$BuildOnly,
    [switch]$TestOnly,
    [switch]$Help
)

# Colors for output (PowerShell)
function Write-Status { 
    param($Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue 
}

function Write-Success { 
    param($Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green 
}

function Write-Error { 
    param($Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red 
}

function Write-Warning { 
    param($Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow 
}

# Function to run a command and capture its result
function Invoke-Check {
    param(
        [string]$CheckName,
        [string]$Command
    )
    
    Write-Status "Running: $CheckName"
    Write-Host "Command: $Command"
    
    try {
        Invoke-Expression $Command
        if ($LASTEXITCODE -eq 0) {
            Write-Success "$CheckName passed"
            return $true
        } else {
            Write-Error "$CheckName failed"
            return $false
        }
    } catch {
        Write-Error "$CheckName failed with exception: $_"
        return $false
    }
}

# Main CI checks
function Invoke-MainChecks {
    $failedChecks = @()
    
    Write-Host "🚀 Starting Local CI Checks for Rust-Spray" -ForegroundColor Cyan
    Write-Host "==========================================="
    
    # Check if we're in the right directory
    if (-not (Test-Path "Cargo.toml")) {
        Write-Error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    }
    
    # Check if Rust is installed
    try {
        $null = Get-Command cargo -ErrorAction Stop
    } catch {
        Write-Error "Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    }
    
    Write-Status "Rust version: $(rustc --version)"
    Write-Status "Cargo version: $(cargo --version)"
    
    Write-Host ""
    Write-Status "1. FORMATTING CHECK"
    Write-Host "==================="
    if (-not (Invoke-Check "Code formatting" "cargo fmt --all -- --check")) {
        $failedChecks += "formatting"
        Write-Warning "To fix formatting issues, run: cargo fmt --all"
    }
    
    Write-Host ""
    Write-Status "2. CLIPPY ANALYSIS"
    Write-Host "=================="
    
    # Install clippy if not available
    $components = rustup component list --installed
    if ($components -notmatch "clippy") {
        Write-Status "Installing clippy component..."
        rustup component add clippy
    }
    
    if (-not (Invoke-Check "Clippy (host features)" "cargo clippy --all-targets --features host -- -D warnings")) {
        $failedChecks += "clippy-host"
    }
    
    if (-not (Invoke-Check "Clippy (no features)" "cargo clippy --all-targets --no-default-features -- -D warnings")) {
        $failedChecks += "clippy-no-features"
    }
    
    if (-not (Invoke-Check "Clippy (raspberry-pi features)" "cargo clippy --all-targets --features raspberry-pi -- -D warnings")) {
        $failedChecks += "clippy-rpi"
    }
    
    Write-Host ""
    Write-Status "3. BUILD TESTS"
    Write-Host "=============="
    
    if (-not (Invoke-Check "Build (host features)" "cargo build --verbose --features host")) {
        $failedChecks += "build-host"
    }
    
    if (-not (Invoke-Check "Build (no features)" "cargo build --verbose --no-default-features")) {
        $failedChecks += "build-no-features"
    }
    
    if (-not (Invoke-Check "Build (raspberry-pi features)" "cargo build --verbose --features raspberry-pi")) {
        $failedChecks += "build-rpi"
    }
    
    Write-Host ""
    Write-Status "4. UNIT TESTS"
    Write-Host "============="
    
    if (-not (Invoke-Check "Tests (host features)" "cargo test --verbose --features host")) {
        $failedChecks += "test-host"
    }
    
    if (-not (Invoke-Check "Tests (no features)" "cargo test --verbose --no-default-features")) {
        $failedChecks += "test-no-features"
    }
    
    Write-Host ""
    Write-Status "5. EXAMPLE COMPILATION"
    Write-Host "======================"
    
    if (-not (Invoke-Check "Example (basic_usage)" "cargo check --example basic_usage --features host")) {
        $failedChecks += "example"
    }
    
    Write-Host ""
    Write-Status "6. DOCUMENTATION BUILD"
    Write-Host "======================"
    
    if (-not (Invoke-Check "Documentation (host features)" "cargo doc --no-deps --features host")) {
        $failedChecks += "docs"
    }
    
    Write-Host ""
    Write-Status "7. SECURITY AUDIT"
    Write-Host "================="
    
    # Check if cargo-audit is installed
    try {
        $null = Get-Command cargo-audit -ErrorAction Stop
    } catch {
        Write-Status "Installing cargo-audit..."
        cargo install cargo-audit
    }
    
    if (-not (Invoke-Check "Security audit" "cargo audit")) {
        $failedChecks += "security"
    }
    
    # Summary
    Write-Host ""
    Write-Host "🏁 CI CHECK SUMMARY" -ForegroundColor Cyan
    Write-Host "==================="
    
    if ($failedChecks.Count -eq 0) {
        Write-Success "All CI checks passed! ✅"
        Write-Success "Your code is ready for CI/CD pipeline."
        exit 0
    } else {
        Write-Error "Failed checks: $($failedChecks -join ', ')"
        Write-Error "Please fix the above issues before pushing to CI."
        exit 1
    }
}

# Handle script parameters
if ($Help) {
    Write-Host "Local CI Testing Script for Rust-Spray (PowerShell)" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\ci-local.ps1 [-Option]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -FormatOnly     Run only formatting checks"
    Write-Host "  -ClippyOnly     Run only clippy analysis"
    Write-Host "  -BuildOnly      Run only build checks"
    Write-Host "  -TestOnly       Run only unit tests"
    Write-Host "  -Help           Show this help message"
    Write-Host ""
    Write-Host "Without options, runs all CI checks."
    exit 0
}

if ($FormatOnly) {
    Write-Status "Running formatting check only..."
    cargo fmt --all -- --check
    exit $LASTEXITCODE
}

if ($ClippyOnly) {
    Write-Status "Running clippy checks only..."
    cargo clippy --all-targets --features host -- -D warnings
    cargo clippy --all-targets --no-default-features -- -D warnings
    cargo clippy --all-targets --features raspberry-pi -- -D warnings
    exit $LASTEXITCODE
}

if ($BuildOnly) {
    Write-Status "Running build checks only..."
    cargo build --features host
    cargo build --no-default-features
    cargo build --features raspberry-pi
    exit $LASTEXITCODE
}

if ($TestOnly) {
    Write-Status "Running tests only..."
    cargo test --features host
    cargo test --no-default-features
    exit $LASTEXITCODE
}

# Default: run all checks
Invoke-MainChecks
