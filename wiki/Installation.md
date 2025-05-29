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
   To cross compile for ARM64 also install:
   ```bash
   sudo apt-get install gcc-aarch64-linux-gnu
   rustup target add aarch64-unknown-linux-gnu
   ```

## Building

Clone the repository and build for your platform:

```bash
git clone https://github.com/cropcrusaders/Rust-Spray.git
cd Rust-Spray
cargo build --release
```

For Raspberry Pi (AArch64) cross‑compilation run:

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

After compilation copy the binary from
`target/aarch64-unknown-linux-gnu/release/` to your device.

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
