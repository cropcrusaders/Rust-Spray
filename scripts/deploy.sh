#!/usr/bin/env bash
# deploy.sh — cross-compile and deploy Rust-Spray to a Raspberry Pi
#
# Run on your development machine (Linux/macOS x86_64).
# Requires `cross` (https://github.com/cross-rs/cross) and Docker.
#
# Usage:
#   ./scripts/deploy.sh pi@192.168.1.100         # RPi 4/5 (64-bit)
#   ./scripts/deploy.sh pi@192.168.1.100 armv7   # RPi 3 (32-bit)

set -euo pipefail

PI_HOST="${1:?Usage: $0 <user@host> [arch]}"
ARCH="${2:-aarch64}"

case "$ARCH" in
    aarch64|arm64|rpi4|rpi5)
        TARGET="aarch64-unknown-linux-gnu"
        ;;
    armv7|arm32|rpi3)
        TARGET="armv7-unknown-linux-gnueabihf"
        ;;
    *)
        echo "Unknown arch: $ARCH (use aarch64 or armv7)" >&2
        exit 1
        ;;
esac

echo "=== Rust-Spray Deploy ==="
echo "Target:  $TARGET"
echo "Host:    $PI_HOST"
echo ""

# ── Cross-compile ──────────────────────────────────────────────────

echo "[1/4] Cross-compiling for $TARGET..."

if ! command -v cross &>/dev/null; then
    echo "Error: 'cross' not found. Install with:" >&2
    echo "  cargo install --git https://github.com/cross-rs/cross cross --locked" >&2
    exit 1
fi

cross build --release --target "$TARGET" --features rpi
BINARY="target/${TARGET}/release/rustspray"

echo "  -> $BINARY ($(du -h "$BINARY" | cut -f1) stripped size)"

# ── Transfer files ─────────────────────────────────────────────────

echo "[2/4] Transferring files to $PI_HOST..."

REMOTE_TMP="/tmp/rustspray-deploy"
ssh "$PI_HOST" "mkdir -p $REMOTE_TMP"
scp -q "$BINARY" "$PI_HOST:$REMOTE_TMP/rustspray"
scp -q scripts/rustspray-camera.sh "$PI_HOST:$REMOTE_TMP/rustspray-camera.sh"
scp -q config/rustspray.toml "$PI_HOST:$REMOTE_TMP/rustspray.toml"
scp -q deploy/rustspray.service "$PI_HOST:$REMOTE_TMP/rustspray.service"
scp -q scripts/install.sh "$PI_HOST:$REMOTE_TMP/install.sh"

# ── Install on Pi ──────────────────────────────────────────────────

echo "[3/4] Installing on $PI_HOST..."

ssh "$PI_HOST" "sudo bash -c '
set -e
install -m 0755 $REMOTE_TMP/rustspray /usr/local/bin/rustspray
install -m 0755 $REMOTE_TMP/rustspray-camera.sh /usr/local/bin/rustspray-camera
mkdir -p /etc/rustspray
if [ ! -f /etc/rustspray/config.toml ]; then
    cp $REMOTE_TMP/rustspray.toml /etc/rustspray/config.toml
fi
cp $REMOTE_TMP/rustspray.service /etc/systemd/system/rustspray.service
systemctl daemon-reload
rm -rf $REMOTE_TMP
'"

# ── Verify ─────────────────────────────────────────────────────────

echo "[4/4] Verifying installation..."

ssh "$PI_HOST" "rustspray --version"

echo ""
echo "Deploy complete! On the Pi, run:"
echo "  sudo nano /etc/rustspray/config.toml   # edit config"
echo "  rustspray --test-pattern --mock-gpio    # dry-run test"
echo "  sudo systemctl enable --now rustspray   # start service"
echo "  journalctl -u rustspray -f              # view logs"
