#!/usr/bin/env bash
# install.sh — install Rust-Spray on a Raspberry Pi
#
# Run as root (or with sudo) on the Pi itself:
#   sudo bash scripts/install.sh
#
# Expects the built binary at ./target/release/rustspray (native build)
# or accepts a path via BINARY env var.

set -euo pipefail

BINARY="${BINARY:-target/release/rustspray}"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/rustspray"
SERVICE_FILE="/etc/systemd/system/rustspray.service"

echo "=== Rust-Spray Installer ==="

# ── Check root ─────────────────────────────────────────────────────

if [ "$(id -u)" -ne 0 ]; then
    echo "Error: run as root (sudo bash $0)" >&2
    exit 1
fi

# ── Install dependencies ──────────────────────────────────────────

echo "[1/6] Installing system dependencies..."
apt-get update -qq
apt-get install -y -qq ffmpeg > /dev/null

# ── Install binary ─────────────────────────────────────────────────

echo "[2/6] Installing binary..."
if [ ! -f "$BINARY" ]; then
    echo "Error: binary not found at $BINARY" >&2
    echo "Build first: cargo build --release --features rpi" >&2
    echo "Or set BINARY=/path/to/rustspray" >&2
    exit 1
fi
install -m 0755 "$BINARY" "$INSTALL_DIR/rustspray"
echo "  -> $INSTALL_DIR/rustspray"

# ── Install camera helper ─────────────────────────────────────────

echo "[3/6] Installing camera helper script..."
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
install -m 0755 "$SCRIPT_DIR/rustspray-camera.sh" "$INSTALL_DIR/rustspray-camera"
echo "  -> $INSTALL_DIR/rustspray-camera"

# ── Install configuration ─────────────────────────────────────────

echo "[4/6] Setting up configuration..."
mkdir -p "$CONFIG_DIR"
if [ -f "$CONFIG_DIR/config.toml" ]; then
    echo "  -> $CONFIG_DIR/config.toml already exists, not overwriting"
    echo "  -> new default saved as $CONFIG_DIR/config.toml.default"
    cp config/rustspray.toml "$CONFIG_DIR/config.toml.default"
else
    cp config/rustspray.toml "$CONFIG_DIR/config.toml"
    echo "  -> $CONFIG_DIR/config.toml"
fi

# ── Install systemd service ───────────────────────────────────────

echo "[5/6] Installing systemd service..."
cp deploy/rustspray.service "$SERVICE_FILE"
systemctl daemon-reload
echo "  -> $SERVICE_FILE"

# ── Summary ────────────────────────────────────────────────────────

echo "[6/6] Done!"
echo ""
echo "Next steps:"
echo "  1. Edit /etc/rustspray/config.toml"
echo "     - Set camera backend (v4l2 or libcamera)"
echo "     - Set GPIO pins to match your wiring"
echo "     - Tune vision thresholds for your conditions"
echo ""
echo "  2. Test without hardware:"
echo "     rustspray --test-pattern --mock-gpio"
echo ""
echo "  3. Test with camera:"
echo "     rustspray-camera | rustspray --mock-gpio"
echo ""
echo "  4. Enable on boot:"
echo "     sudo systemctl enable --now rustspray"
echo ""
echo "  5. View logs:"
echo "     journalctl -u rustspray -f"
