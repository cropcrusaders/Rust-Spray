#!/usr/bin/env bash
# rustspray-camera — capture frames and output raw RGB24 to stdout
#
# Reads camera settings from /etc/rustspray/config.toml (or path passed
# as $1) and launches the appropriate capture pipeline. Output is raw
# RGB24 at the configured resolution — pipe directly into `rustspray`.
#
# Usage:
#   rustspray-camera                              # uses default config
#   rustspray-camera /path/to/config.toml         # custom config
#   rustspray-camera | rustspray --config ...      # typical pipeline

set -euo pipefail

CONFIG="${1:-/etc/rustspray/config.toml}"

# ── Parse config (simple key extraction from TOML) ─────────────────

get_value() {
    local section="$1" key="$2" default="$3"
    local val
    val=$(sed -n "/^\[${section}\]/,/^\[/{s/^${key}[[:space:]]*=[[:space:]]*//p}" "$CONFIG" 2>/dev/null \
        | tr -d '"' | tr -d "'" | head -1 | xargs)
    echo "${val:-$default}"
}

WIDTH=$(get_value camera width 640)
HEIGHT=$(get_value camera height 480)
FPS=$(get_value camera fps 30)
BACKEND=$(get_value camera backend v4l2)
DEVICE=$(get_value camera device /dev/video0)

echo "rustspray-camera: ${BACKEND} ${WIDTH}x${HEIGHT}@${FPS}" >&2

# ── Launch camera pipeline ─────────────────────────────────────────

case "$BACKEND" in
    libcamera)
        # Raspberry Pi Camera Module v2/v3 via rpicam-vid (Pi OS Bookworm+).
        # Falls back to libcamera-vid for older installs.
        CAM_CMD="rpicam-vid"
        if ! command -v rpicam-vid &>/dev/null; then
            CAM_CMD="libcamera-vid"
        fi

        exec "$CAM_CMD" \
            -t 0 \
            --width "$WIDTH" --height "$HEIGHT" --framerate "$FPS" \
            --codec yuv420 --nopreview -o - 2>/dev/null \
        | exec ffmpeg -loglevel error \
            -f rawvideo -pix_fmt yuv420p -s "${WIDTH}x${HEIGHT}" \
            -framerate "$FPS" -i - \
            -f rawvideo -pix_fmt rgb24 pipe:1
        ;;

    v4l2|*)
        # USB camera or any V4L2 device.
        exec ffmpeg -loglevel error \
            -f v4l2 -framerate "$FPS" -video_size "${WIDTH}x${HEIGHT}" \
            -i "$DEVICE" \
            -f rawvideo -pix_fmt rgb24 pipe:1
        ;;
esac
