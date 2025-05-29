# Rust-Spray

Rust-Spray is a small example project that uses a camera to detect weeds and pulse up to four sprayer outputs via GPIO pins. It targets Linux boards such as the Raspberry Pi. The detection pipeline is implemented with OpenCV in Rust.

## Features

- Capture frames from a USB or Raspberry Pi camera.
- Weed detection using ExG or HSV colour thresholds.
- Control four GPIO-driven sprayers via rppal.
- Configuration via `config/config.toml`.
- Optional display window for debugging.

## Building

1. Install Rust (via [rustup](https://rustup.rs)) and OpenCV development libraries.
2. Clone the repository:
   ```bash
   git clone https://github.com/cropcrusaders/Rust-Spray.git
   cd Rust-Spray
   ```
3. Build for the host platform:
   ```bash
   cargo build --release
   ```

To cross compile for Raspberry Pi (AArch64):
   ```bash
   rustup target add aarch64-unknown-linux-gnu
   cargo build --release --target aarch64-unknown-linux-gnu
   ```

## Configuration

Copy `config/Config.toml` to `config/config.toml` and edit as needed:

```toml
[camera]
device = "/dev/video2"
resolution_width = 1280
resolution_height = 720
use_rpi_cam = false

[detection]
algorithm = "hsv"
exg_min = 20
exg_max = 200
hue_min = 25
hue_max = 100
brightness_min = 10
brightness_max = 220
saturation_min = 40
saturation_max = 250
min_area = 15.0
invert_hue = true

[spray]
pins = [23, 24, 25, 26]
activation_duration_ms = 200
```

## Running

```bash
sudo ./target/release/rust-spray --config config/config.toml --show-display
```

The program opens the camera, runs the detection algorithm and pulses the sprayers whenever weeds are detected. Use `--show-display` to view the annotated video stream.

## License

Rust-Spray is released under the MIT license.
