#!/usr/bin/env bash
set -euo pipefail
#
# Rust-Spray installer for Raspberry Pi
#
# Installs the nightly Rust toolchain, builds the project with GPIO support,
# copies the binary into /usr/local/bin, and optionally installs a systemd
# service so the pipeline starts on boot.
#
# Usage:
#   curl <this-script> | bash          # one-liner from GitHub
#   bash scripts/install.sh            # from the repo root
#   bash scripts/install.sh --no-service  # skip systemd setup

INSTALL_SERVICE=true
for arg in "$@"; do
    case "$arg" in
        --no-service) INSTALL_SERVICE=false ;;
        --help|-h)
            echo "Usage: install.sh [--no-service]"
            exit 0
            ;;
    esac
done

echo "=== Rust-Spray Installer ==="

# ---------- architecture check ----------
ARCH=$(uname -m)
case "$ARCH" in
    aarch64)  echo "Detected aarch64 (RPi 4/5)" ;;
    armv7l)   echo "Detected armv7l  (RPi 3)" ;;
    *)        echo "Warning: expected ARM, got $ARCH — continuing anyway" ;;
esac

# ---------- system packages ----------
echo ""
echo "[1/5] Installing system packages ..."
sudo apt-get update -qq
sudo apt-get install -y -qq \
    build-essential \
    pkg-config \
    curl \
    git

# ---------- Rust toolchain ----------
echo ""
echo "[2/5] Setting up Rust nightly toolchain ..."
if ! command -v rustup &>/dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain nightly
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
else
    rustup install nightly --quiet
    rustup default nightly
fi
rustup component add rustfmt 2>/dev/null || true

# ---------- build ----------
echo ""
echo "[3/5] Building Rust-Spray (release + GPIO) ..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

FEATURES="rpi"
# Also build with model support if tract-onnx is wanted
if [ "${WITH_MODEL:-}" = "1" ]; then
    FEATURES="rpi,model"
fi
cargo build --release --features "$FEATURES"

# ---------- install binary ----------
echo ""
echo "[4/5] Installing binaries ..."
INSTALL_DIR="/usr/local/bin"

# The crate is a library; install the example binary.
if [ -f target/release/examples/four_lane ]; then
    sudo install -m 0755 target/release/examples/four_lane "$INSTALL_DIR/rustspray"
    echo "  -> $INSTALL_DIR/rustspray"
else
    echo "  Warning: example binary not found — skipping install"
fi

# ---------- GPIO permissions ----------
sudo usermod -aG gpio "$USER" 2>/dev/null || true

# ---------- systemd service ----------
echo ""
if $INSTALL_SERVICE; then
    echo "[5/5] Installing systemd service ..."
    sudo tee /etc/systemd/system/rustspray.service >/dev/null <<'SERVICE'
[Unit]
Description=Rust-Spray 4-lane vegetation pipeline
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/rustspray --mock-gpio
Restart=on-failure
RestartSec=5
User=pi
Group=gpio

[Install]
WantedBy=multi-user.target
SERVICE
    sudo systemctl daemon-reload
    echo "  Service installed.  Enable on boot with:"
    echo "    sudo systemctl enable rustspray"
    echo "  Start now with:"
    echo "    sudo systemctl start rustspray"
else
    echo "[5/5] Skipping systemd service (--no-service)"
fi

echo ""
echo "=== Done ==="
echo "Quick test:  rustspray --mock-gpio"
echo ""
echo "To use a trained ONNX model:"
echo "  pip install torch numpy onnx"
echo "  python scripts/train_model.py"
echo "  Rebuild with:  WITH_MODEL=1 bash scripts/install.sh"
