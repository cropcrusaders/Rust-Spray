#!/bin/bash
# Local CI Testing Script for Rust-Spray
# This script runs all the same checks that the GitHub Actions CI runs

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Function to run a command and capture its result
run_check() {
    local check_name="$1"
    local command="$2"
    
    print_status "Running: $check_name"
    echo "Command: $command"
    
    if eval "$command"; then
        print_success "$check_name passed"
        return 0
    else
        print_error "$check_name failed"
        return 1
    fi
}

# Main CI checks
main() {
    local failed_checks=()
    
    echo "🚀 Starting Local CI Checks for Rust-Spray"
    echo "=========================================="
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    fi
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    print_status "Rust version: $(rustc --version)"
    print_status "Cargo version: $(cargo --version)"
    
    echo ""
    print_status "1. FORMATTING CHECK"
    echo "==================="
    if ! run_check "Code formatting" "cargo fmt --all -- --check"; then
        failed_checks+=("formatting")
        print_warning "To fix formatting issues, run: cargo fmt --all"
    fi
    
    echo ""
    print_status "2. CLIPPY ANALYSIS"
    echo "=================="
    
    # Install clippy if not available
    if ! rustup component list --installed | grep -q clippy; then
        print_status "Installing clippy component..."
        rustup component add clippy
    fi
    
    if ! run_check "Clippy (host features)" "cargo clippy --all-targets --features host -- -D warnings"; then
        failed_checks+=("clippy-host")
    fi
    
    if ! run_check "Clippy (no features)" "cargo clippy --all-targets --no-default-features -- -D warnings"; then
        failed_checks+=("clippy-no-features")
    fi
    
    if ! run_check "Clippy (raspberry-pi features)" "cargo clippy --all-targets --features raspberry-pi -- -D warnings"; then
        failed_checks+=("clippy-rpi")
    fi
    
    echo ""
    print_status "3. BUILD TESTS"
    echo "=============="
    
    if ! run_check "Build (host features)" "cargo build --verbose --features host"; then
        failed_checks+=("build-host")
    fi
    
    if ! run_check "Build (no features)" "cargo build --verbose --no-default-features"; then
        failed_checks+=("build-no-features")
    fi
    
    if ! run_check "Build (raspberry-pi features)" "cargo build --verbose --features raspberry-pi"; then
        failed_checks+=("build-rpi")
    fi
    
    echo ""
    print_status "4. UNIT TESTS"
    echo "============="
    
    if ! run_check "Tests (host features)" "cargo test --verbose --features host"; then
        failed_checks+=("test-host")
    fi
    
    if ! run_check "Tests (no features)" "cargo test --verbose --no-default-features"; then
        failed_checks+=("test-no-features")
    fi
    
    echo ""
    print_status "5. EXAMPLE COMPILATION"
    echo "======================"
    
    if ! run_check "Example (basic_usage)" "cargo check --example basic_usage --features host"; then
        failed_checks+=("example")
    fi
    
    echo ""
    print_status "6. DOCUMENTATION BUILD"
    echo "======================"
    
    if ! run_check "Documentation (host features)" "cargo doc --no-deps --features host"; then
        failed_checks+=("docs")
    fi
    
    echo ""
    print_status "7. SECURITY AUDIT"
    echo "================="
    
    # Install cargo-audit if not available
    if ! command -v cargo-audit &> /dev/null; then
        print_status "Installing cargo-audit..."
        cargo install cargo-audit
    fi
    
    if ! run_check "Security audit" "cargo audit"; then
        failed_checks+=("security")
    fi
    
    # Summary
    echo ""
    echo "🏁 CI CHECK SUMMARY"
    echo "==================="
    
    if [ ${#failed_checks[@]} -eq 0 ]; then
        print_success "All CI checks passed! ✅"
        print_success "Your code is ready for CI/CD pipeline."
        exit 0
    else
        print_error "Failed checks: ${failed_checks[*]}"
        print_error "Please fix the above issues before pushing to CI."
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    --format-only)
        print_status "Running formatting check only..."
        cargo fmt --all -- --check
        ;;
    --clippy-only)
        print_status "Running clippy checks only..."
        cargo clippy --all-targets --features host -- -D warnings
        cargo clippy --all-targets --no-default-features -- -D warnings
        cargo clippy --all-targets --features raspberry-pi -- -D warnings
        ;;
    --build-only)
        print_status "Running build checks only..."
        cargo build --features host
        cargo build --no-default-features
        cargo build --features raspberry-pi
        ;;
    --test-only)
        print_status "Running tests only..."
        cargo test --features host
        cargo test --no-default-features
        ;;
    --help|-h)
        echo "Local CI Testing Script for Rust-Spray"
        echo ""
        echo "Usage: $0 [OPTION]"
        echo ""
        echo "Options:"
        echo "  --format-only     Run only formatting checks"
        echo "  --clippy-only     Run only clippy analysis"
        echo "  --build-only      Run only build checks"
        echo "  --test-only       Run only unit tests"
        echo "  --help, -h        Show this help message"
        echo ""
        echo "Without options, runs all CI checks."
        ;;
    *)
        main
        ;;
esac
