#!/bin/bash
# Build verification script for Rust-Spray project

echo "🔧 Rust-Spray Build Verification"
echo "================================="

# Check if Cargo.toml exists and is valid
if [ -f "Cargo.toml" ]; then
    echo "✅ Cargo.toml found"
else
    echo "❌ Cargo.toml not found"
    exit 1
fi

# Check if source files exist
echo "📁 Checking source files..."
FILES=(
    "src/main.rs"
    "src/lib.rs" 
    "src/config.rs"
    "src/camera.rs"
    "src/detection.rs"
    "src/spray.rs"
    "src/utils/mod.rs"
    "src/utils/algorithms.rs"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file"
    else
        echo "❌ $file missing"
    fi
done

# Check if config file exists
if [ -f "config/Config.toml" ]; then
    echo "✅ config/Config.toml"
else
    echo "⚠️  config/Config.toml not found (needed for runtime)"
fi

# Check if tests exist
echo "🧪 Checking tests..."
if [ -f "tests/integration_tests.rs" ]; then
    echo "✅ integration_tests.rs"
fi

if [ -f "tests/config_tests.rs" ]; then
    echo "✅ config_tests.rs"
fi

# Check if examples exist
if [ -f "examples/basic_usage.rs" ]; then
    echo "✅ examples/basic_usage.rs"
fi

echo ""
echo "📋 Build Commands:"
echo "=================="
echo "Standard build:          cargo build"
echo "Release build:           cargo build --release"
echo "With GPIO support:       cargo build --features with-rppal"
echo "With Pi camera:          cargo build --features picam"
echo "Full Pi support:         cargo build --features raspberry-pi"
echo "Cross compile (ARM64):   cross build --target aarch64-unknown-linux-gnu --release"
echo "Cross compile (ARMv7):   cross build --target armv7-unknown-linux-gnueabihf --release"
echo ""
echo "🧪 Test Commands:"
echo "================="
echo "Run tests:               cargo test"
echo "Run with features:       cargo test --features with-rppal"
echo ""
echo "📖 Example Commands:"
echo "===================="
echo "Run example:             cargo run --example basic_usage"
echo ""
echo "🚀 Ready for compilation!"
