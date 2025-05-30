# Rust-Spray

Rust-Spray is a small example project that uses a camera to detect weeds and pulse up to four sprayer outputs via GPIO pins. It targets Linux boards such as the Raspberry Pi. The detection pipeline is implemented with OpenCV in Rust.

## Beginner Quickstart

1. **Install Rust, Cargo and OpenCV development libraries**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   # Add cargo to PATH for the current shell
   source "$HOME/.cargo/env"
   sudo apt-get update
   sudo apt-get install libopencv-dev pkg-config build-essential

   cargo install cargo-opencv --git https://github.com/twistedfall/opencv-rust
   ```
2. **Clone this repository**
   ```bash
   git clone https://github.com/cropcrusaders/Rust-Spray.git
   cd Rust-Spray
   ```
3. **Build the project**
   ```bash
   cargo build --release
   ```
   For Raspberry Pi targets install the appropriate cross toolchain and add the target.
   For 64-bit OS:
   ```bash
   sudo apt-get install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
   rustup target add aarch64-unknown-linux-gnu
   ```
   For 32-bit OS:
   ```bash
   sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf
   rustup target add armv7-unknown-linux-gnueabihf
   ```
   Then build with:
   ```bash
   cargo build --release --target aarch64-unknown-linux-gnu
   # or for 32-bit
   cargo build --release --target armv7-unknown-linux-gnueabihf
   ```
   Add `--features picam` if the Raspberry Pi camera module is required.

4. **Copy the example configuration**
   ```bash
   cp config/Config.toml config/config.toml
   ```
5. **Run the binary** (use sudo for GPIO access):
   ```bash
   sudo ./target/release/rust-spray --config config/config.toml --show-display
   ```

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

To cross compile for Raspberry Pi choose the appropriate target.
For 64-bit OS:
   ```bash
   sudo apt-get install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
   rustup target add aarch64-unknown-linux-gnu
   cargo build --release --target aarch64-unknown-linux-gnu
   ```
For 32-bit OS:
   ```bash
   sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf
   rustup target add armv7-unknown-linux-gnueabihf
   cargo build --release --target armv7-unknown-linux-gnueabihf
   ```
Add `--features picam` when the Raspberry Pi camera module is required.
The repository's `.cargo/config.toml` configures the linker as `aarch64-linux-gnu-gcc` for this target.
It also defines a linker for the 32-bit `armv7-unknown-linux-gnueabihf` target so you can cross compile for older Raspberry Pi models.
Alternatively, uncomment the `target` line in `.cargo/config.toml` to make cross compilation the default.
When using `cross`, ensure Docker is available. If it prints `Falling back to cargo on the host`, the container could not start and the build may fail.
Install the `aarch64-linux-gnu` or `arm-linux-gnueabihf` toolchain and run `cargo build --target <target>` as a fallback.

## Building with Docker

If you prefer to build inside a container you can create the images used by
`cross` from the provided Dockerfiles.

```bash
# For 64-bit ARM targets
docker build -f Dockerfile.pi-opencv -t custom/aarch64-opencv .

# For 32-bit ARM targets
docker build -f Dockerfile.pi-opencv-armv7 -t custom/armv7-opencv .
```

Install [`cross`](https://github.com/cross-rs/cross) and build using the image:

```bash
cargo install cross
cross build --release --target aarch64-unknown-linux-gnu
```

Replace the target as needed. You can also run arbitrary commands inside the
image, for example:

```bash
docker run --rm -it -v $(pwd):/project -w /project custom/aarch64-opencv cargo test
```

### Windows

Rust-Spray targets Linux boards, but you can cross compile from Windows using
the Docker setup above. Install [Docker Desktop](https://www.docker.com/products/docker-desktop)
with the WSL2 backend and ensure `cargo` is available (either through WSL or
native rustup). Clone the repository and run:

```bash
git clone https://github.com/cropcrusaders/Rust-Spray.git
cd Rust-Spray
cargo install cross
cross build --release --target aarch64-unknown-linux-gnu
```

Make sure Docker Desktop is running so that `cross` can launch its container.
You may also run other commands with `docker run` as shown above.

### Pre-built Raspberry Pi Package

GitHub Actions builds a Debian package for Raspberry Pi (ARM64) on every
release. Download the latest `rustspray_*.deb` from the
[releases page](https://github.com/cropcrusaders/Rust-Spray/releases) and
install it on the Pi with:

```bash
sudo dpkg -i rustspray_*_arm64.deb
```

### Yocto Build

See [docs/yocto.md](docs/yocto.md) for steps to build Rust-Spray with the Yocto Project.

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

## Hardware Setup

- Connect a USB webcam or the Raspberry Pi camera module.
- Wire solenoid valves (or other actuators) to GPIO pins 23–26 as shown above.
- Provide external drivers and power for the valves and verify fail‑safe behaviour.
- Optional peripherals such as CAN bus adapters, flow sensors and GPS receivers can also be attached.
- See [wiki/Wiring.md](wiki/Wiring.md) for diagrams and more details.

## Running

```bash
sudo ./target/release/rust-spray --config config/config.toml --show-display
```

The program opens the camera, runs the detection algorithm and pulses the sprayers whenever weeds are detected. Use `--show-display` to view the annotated video stream.

## License

Rust-Spray is released under the MIT license.
