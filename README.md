# Rust-Spray

SIMD-accelerated vegetation detection and precision spray control for
Raspberry Pi. Detects green plants in a camera feed using a multi-cue
vision algorithm and drives relay modules to activate/deactivate spray
nozzles across independent lanes — all in real time.

## Features

- **Fast** — SIMD-accelerated Excess Green (ExG) computation processes
  640x480 frames in under 5 ms on a Raspberry Pi 4
- **Multi-cue vision** — fuses ExG, green-ratio, and chroma cues with
  tunable weights for robust detection across lighting conditions
- **Hysteresis** — lane activation uses separate on/off thresholds to
  prevent rapid nozzle toggling near the decision boundary
- **Configurable** — all thresholds, weights, pin mappings, and camera
  settings live in a single TOML config file
- **Production-ready** — systemd service, install scripts, graceful
  shutdown, and structured logging via the journal
- **Cross-compile friendly** — build on x86_64, deploy to ARM with one
  command using the included `deploy.sh` script

## Hardware Requirements

### Minimum

| Component | Recommendation |
|-----------|---------------|
| **Board** | Raspberry Pi 4 Model B (2 GB+) or Raspberry Pi 5 |
| **Camera** | Any USB UVC camera or Raspberry Pi Camera Module v2/v3 |
| **Relay module** | 4-channel 5 V relay board (for 4-lane setup) |
| **Nozzle solenoids** | 12 V normally-closed solenoid valves (one per lane) |
| **Power** | 5 V / 3 A for the Pi, 12 V supply for solenoids |
| **SD card** | 8 GB+ with Raspberry Pi OS (Bookworm) |

### Supported Boards

| Board | Target | Notes |
|-------|--------|-------|
| Raspberry Pi 5 | `aarch64-unknown-linux-gnu` | Best performance |
| Raspberry Pi 4 | `aarch64-unknown-linux-gnu` | Recommended minimum |
| Raspberry Pi 3 | `armv7-unknown-linux-gnueabihf` | 32-bit, lower throughput |

### Camera Options

| Camera | Config `backend` | Notes |
|--------|-----------------|-------|
| USB UVC camera | `v4l2` | Plug-and-play, wide selection |
| RPi Camera Module v2 (IMX219) | `libcamera` | CSI connector, good in daylight |
| RPi Camera Module v3 (IMX708) | `libcamera` | Autofocus, HDR, best quality |
| Any GStreamer source | custom pipeline | Advanced — pipe into stdin |

## Wiring

### GPIO Pin Mapping (default)

The default configuration uses four GPIO pins to drive four relay
channels. Each relay controls one solenoid nozzle valve.

```
                    Raspberry Pi GPIO Header
                    ┌─────────────────────┐
                    │  (pin 1) 3V3   5V   │
                    │  (pin 3) SDA   5V   │
                    │  (pin 5) SCL   GND  │
                    │  (pin 7) GP4   TX   │
                    │  (pin 9) GND   RX   │
        Lane 0 ──── │  (pin 11) GP17  GP18 │
        Lane 1 ──── │  (pin 13) GP27  GND  │
        Lane 2 ──── │  (pin 15) GP22  GP23 │ ──── Lane 3
                    │  (pin 17) 3V3  GP24 │
                    │  ...             ... │
                    └─────────────────────┘

Default pin assignment (BCM numbering):
  Lane 0 → GPIO 17 (physical pin 11)
  Lane 1 → GPIO 27 (physical pin 13)
  Lane 2 → GPIO 22 (physical pin 15)
  Lane 3 → GPIO 23 (physical pin 16)
```

### Relay Wiring Diagram

```
Raspberry Pi                 4-Channel Relay Board         Solenoid Valves
┌──────────┐                ┌────────────────────┐        ┌─────────────┐
│      GP17 ├───────────────┤ IN1          COM1  ├────────┤ Valve 1     │
│      GP27 ├───────────────┤ IN2          COM2  ├────────┤ Valve 2     │
│      GP22 ├───────────────┤ IN3          COM3  ├────────┤ Valve 3     │
│      GP23 ├───────────────┤ IN4          COM4  ├────────┤ Valve 4     │
│           │               │                    │        │             │
│       GND ├───────────────┤ GND                │        │             │
│        5V ├───────────────┤ VCC                │        │             │
│           │               │              NO1-4 ├──┐     │             │
└──────────┘                └────────────────────┘  │     └──────┬──────┘
                                                    │            │
                            12V Power Supply ───────┴────────────┘

IN1-4  = Signal inputs from GPIO (active low or high depending on board)
COM1-4 = Common terminals → connect to solenoid
NO1-4  = Normally Open terminals → connect to 12V supply
GND    = Shared ground with Pi
VCC    = 5V from Pi (for relay coil logic)
```

> **Note:** Many relay boards are active-low (relay energises when the
> GPIO pin goes LOW). The `RppalGpio` driver sets pins HIGH for active
> lanes. If your relays are active-low, either use a board with
> optocoupler isolation that accepts active-high, or swap the wiring
> to use the NC (Normally Closed) terminals instead of NO.
>
> All pins are initialised **LOW** at startup and driven LOW again on
> shutdown, so nozzles stay off whenever the pipeline is not actively
> spraying.

## Quick Start (Desktop / Testing)

```bash
# Build (requires Rust nightly)
cargo build --release

# Run with synthetic test frames — no hardware needed
cargo run --release -- --test-pattern --mock-gpio
```

## Installation on Raspberry Pi

### Option 1: Cross-Compile and Deploy (recommended)

Build on your development machine and transfer to the Pi over SSH.

**Prerequisites on your dev machine:**
- Rust nightly toolchain
- Docker (for cross-compilation)
- `cross` tool

```bash
# Install cross
cargo install --git https://github.com/cross-rs/cross cross --locked

# Deploy to Pi (one command)
./scripts/deploy.sh pi@192.168.1.100

# For Raspberry Pi 3 (32-bit):
./scripts/deploy.sh pi@192.168.1.100 armv7
```

### Option 2: Build on the Pi

```bash
# Install Rust nightly on the Pi
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly

# Install build dependencies
sudo apt-get update
sudo apt-get install -y build-essential pkg-config ffmpeg

# Clone and build
git clone https://github.com/cropcrusaders/Rust-Spray.git
cd Rust-Spray
cargo build --release --features rpi

# Install
sudo bash scripts/install.sh
```

### Option 3: Manual Install

```bash
# Copy the binary
sudo cp target/release/rustspray /usr/local/bin/
sudo cp scripts/rustspray-camera.sh /usr/local/bin/rustspray-camera
sudo chmod +x /usr/local/bin/rustspray /usr/local/bin/rustspray-camera

# Set up config
sudo mkdir -p /etc/rustspray
sudo cp config/rustspray.toml /etc/rustspray/config.toml

# Install systemd service
sudo cp deploy/rustspray.service /etc/systemd/system/
sudo systemctl daemon-reload
```

## Configuration

Edit `/etc/rustspray/config.toml` to match your hardware setup. The
complete reference configuration with defaults is in `config/rustspray.toml`.

### Key Settings to Change

```toml
[camera]
backend = "libcamera"   # Change to "libcamera" for RPi Camera Module
device  = "/dev/video0" # Only for V4L2 backend
width   = 640
height  = 480
fps     = 30

[gpio]
pins = [17, 27, 22, 23] # BCM pin numbers — must match your wiring
mock = false             # Set true to test without relay hardware

[lanes]
count = 4               # Must match the number of GPIO pins

[vision]
exg_threshold     = 20   # Lower = more sensitive to faint green
green_ratio_floor = 0.36 # Lower = accepts less-green vegetation
chroma_floor      = 0.08 # Lower = accepts more washed-out colors
```

### Tuning Detection Sensitivity

The vision algorithm scores each pixel using three weighted cues:

| Cue | What it measures | When to increase weight |
|-----|-----------------|------------------------|
| `exg` (0.50) | Excess green: 2G − R − B | Bright, clearly green vegetation |
| `green_ratio` (0.35) | Green share of total brightness | Mixed or shaded vegetation |
| `chroma` (0.15) | Color saturation (max−min)/255 | Distinguishing green from grey |

**Too many false positives?** Raise `exg_threshold` and
`green_ratio_floor`.

**Missing real plants?** Lower thresholds and consider raising the
`green_ratio` weight.

## Usage

### Test Without Hardware

```bash
# Synthetic frames, mock GPIO output
rustspray --test-pattern --mock-gpio

# Specify config
rustspray --test-pattern --mock-gpio --config ./config/rustspray.toml

# Process exactly 10 frames then exit
rustspray --test-pattern --mock-gpio --frames 10

# Verbose logging
rustspray --test-pattern --mock-gpio --log-level debug
```

### Test With Camera (No Relays)

```bash
# Start camera, pipe to spray pipeline with mock GPIO
rustspray-camera | rustspray --mock-gpio
```

### Production

```bash
# Start the systemd service
sudo systemctl enable --now rustspray

# Check status
sudo systemctl status rustspray

# View live logs
journalctl -u rustspray -f

# Stop
sudo systemctl stop rustspray
```

### Manual Pipeline

```bash
# USB camera via ffmpeg
ffmpeg -f v4l2 -framerate 30 -video_size 640x480 \
       -i /dev/video0 -f rawvideo -pix_fmt rgb24 pipe:1 | \
  rustspray --config /etc/rustspray/config.toml

# RPi Camera Module via libcamera
rpicam-vid -t 0 --width 640 --height 480 --framerate 30 \
           --codec yuv420 --nopreview -o - | \
  ffmpeg -f rawvideo -pix_fmt yuv420p -s 640x480 -i - \
         -f rawvideo -pix_fmt rgb24 pipe:1 | \
  rustspray --config /etc/rustspray/config.toml
```

## Architecture

```
Camera ──► [RGB24 frames via stdin]
                    │
                    ▼
            ┌──────────────┐
            │ PlantVision  │  Multi-cue pixel scoring
            │  (vision.rs) │  ExG + green ratio + chroma
            └──────┬───────┘
                   │  Vec<bool> mask
                   ▼
            ┌──────────────┐
            │ LaneReducer  │  Vertical strip coverage
            │  (lanes.rs)  │  with hysteresis
            └──────┬───────┘
                   │  Vec<bool> lane states
                   ▼
            ┌──────────────┐
            │ NozzleControl│  GPIO pin driver
            │ (io_gpio.rs) │  (real or mock)
            └──────────────┘
                   │
                   ▼
            Relay Board ──► Solenoid Valves
```

Processing pipeline per frame:

1. **PlantVision::detect()** — scores every pixel with a weighted
   linear combination of ExG, green ratio, and chroma cues. Returns a
   boolean mask.
2. **LaneReducer::reduce()** — divides the mask into vertical strips,
   computes coverage ratios, and applies hysteresis thresholds. Returns
   per-lane on/off states.
3. **NozzleControl::apply()** — sets GPIO pins high/low to
   energise/de-energise relay coils.

## Troubleshooting

### "no input source: stdin is a terminal"

The binary expects raw RGB24 frames piped into stdin. Either:
- Use `--test-pattern` for testing
- Pipe camera output: `rustspray-camera | rustspray`

### Camera not detected

```bash
# List V4L2 devices
v4l2-ctl --list-devices

# Test libcamera
rpicam-still -o test.jpg

# Check device permissions
ls -la /dev/video*
```

### GPIO permission denied

```bash
# Run as root (recommended for production)
sudo rustspray --config /etc/rustspray/config.toml

# Or add user to gpio group
sudo usermod -aG gpio $USER
```

### All lanes always on/off

- Check `exg_threshold` — too low triggers on everything, too high
  misses real plants
- Use `--log-level debug` to see per-frame timing and scoring
- Test with `--mock-gpio` to see lane state changes
- Ensure camera is outputting correct resolution matching config

### Service won't start

```bash
# Check service status
sudo systemctl status rustspray

# Check logs
journalctl -u rustspray --no-pager -n 50

# Test manually
rustspray-camera | rustspray --config /etc/rustspray/config.toml --mock-gpio
```

### ffmpeg "no such device" or "permission denied"

```bash
# Install ffmpeg
sudo apt-get install ffmpeg

# Check camera device exists
ls -la /dev/video0

# For RPi Camera Module, use libcamera backend in config:
# backend = "libcamera"
```

## Development

```bash
# Run tests (18 unit tests)
cargo test

# Format code
cargo fmt

# Run the demo example
cargo run --example four_lane -- --mock-gpio

# Build for different camera backends
cargo check --no-default-features --features camera-nokhwa
cargo check --no-default-features --features camera-gstreamer
```

## Cross-Compilation

```bash
# Install cross tool
cargo install --git https://github.com/cross-rs/cross cross --locked

# RPi 4/5 (64-bit)
cross build --release --target aarch64-unknown-linux-gnu --features rpi

# RPi 3 (32-bit)
cross build --release --target armv7-unknown-linux-gnueabihf --features rpi
```

## Project Structure

```
src/
  main.rs         Production binary (CLI, logging, signal handling)
  lib.rs          Crate root
  config.rs       TOML configuration loading
  exg.rs          SIMD Excess Green mask (u8x16/i16x16)
  vision.rs       Multi-cue vegetation detector (PlantVision)
  lanes.rs        Lane reduction with hysteresis (LaneReducer)
  pipeline.rs     Pipeline orchestrator
  io_gpio.rs      GPIO abstraction (MockGpio, RppalGpio)
examples/
  four_lane.rs    Synthetic frame demo
config/
  rustspray.toml  Default configuration (copy to /etc/rustspray/)
deploy/
  rustspray.service  Systemd unit file
scripts/
  install.sh         Pi installation script
  deploy.sh          Cross-compile + SSH deploy script
  rustspray-camera.sh  Camera capture helper
```

## License

MIT — see [LICENSE](LICENSE).
