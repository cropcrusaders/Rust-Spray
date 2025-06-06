# Installation Guide

This page expands on the installation steps found in the main README.
It explains how to install dependencies, build the project and deploy
it to embedded targets like the Raspberry Pi.

## Prerequisites

1. **Rust Toolchain** – Install with rustup:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **System Libraries** – OpenCV headers and build tools are required.
   On Ubuntu/Debian:
   ```bash
   sudo apt-get update
   sudo apt-get install libopencv-dev pkg-config build-essential
   ```
   To cross compile install the appropriate toolchain.
   For 64-bit OS:
   ```bash
   sudo apt-get install gcc-aarch64-linux-gnu
   rustup target add aarch64-unknown-linux-gnu
   ```
   For 32-bit OS:
   ```bash
   sudo apt-get install gcc-arm-linux-gnueabihf
   rustup target add armv7-unknown-linux-gnueabihf
   ```

## Building

Clone the repository and build for your platform:

```bash
git clone https://github.com/cropcrusaders/Rust-Spray.git
cd Rust-Spray
cargo build --release
```

For Raspberry Pi cross‑compilation choose the appropriate target. The
recommended approach is to use the `cross` tool together with the Docker
images provided in this repository. This ensures that the OpenCV
development libraries for the target architecture are available.

Install `cross` from the GitHub repository and build with:

```bash
cargo install --git https://github.com/cross-rs/cross cross --locked
cross build --release --target aarch64-unknown-linux-gnu
```
Replace the target as needed (e.g. `armv7-unknown-linux-gnueabihf` for
32‑bit). Add `--features picam` when the Raspberry Pi camera module is
required.

After compilation copy the binary from
`target/aarch64-unknown-linux-gnu/release/` to your device.

Alternatively download the pre-built Debian package from the
[GitHub releases](https://github.com/cropcrusaders/Rust-Spray/releases)
page and install it on the Raspberry Pi:

```bash
sudo dpkg -i rustspray_*_arm64.deb
```

## Deployment

1. Ensure the target device has OpenCV runtime libraries installed.
2. Bring up the CAN interface:
   ```bash
   sudo ip link set can0 up type can bitrate 250000
   ```
3. Confirm the GPS serial port (e.g. `/dev/ttyACM0`) is accessible.
4. Copy `config/Config.toml` to `config/config.toml` and adjust settings
   for your hardware.
5. Run the controller with the appropriate command line options:
   ```bash
   sudo ./target/release/rust-spray --sections 16 \
        --gps /dev/ttyACM0 --can if=can0,bitrate=250000 \
        --config config/config.toml
   ```
