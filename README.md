# Rust-Spray

**Rust-Spray** is an open-source, real-time section spraying controller written in **Rust** for precision agriculture. It’s designed to run on embedded Linux platforms like Raspberry Pi or NVIDIA Jetson and controls sprayer boom sections in real time, leveraging inputs from CAN-bus systems and GPS. The software can operate as a standalone PWM boom controller or integrate with ISOBUS, and it includes optional camera-based weed detection via OpenCV for “see-and-spray” applications. Rust-Spray emphasizes modularity, high performance, and safety for field operation.

| CI | Crate | Docs | License |
|----|-------|------|---------|
| ![CI](https://github.com/cropcrusaders/Rust-Spray/actions/workflows/ci.yml/badge.svg) | Coming&nbsp;Soon | Coming&nbsp;Soon | MIT&nbsp;OR&nbsp;Apache‑2.0 + **Safety Rider** |

> **Safety Rider** – Users are **solely responsible** for safe operation; see [`LICENSE-SAFETY.md`](LICENSE-SAFETY.md).

---

## Table of Contents
1. [Features](#features)
2. [Architecture and Design](#architecture-and-design)
3. [Supported Hardware and Interfaces](#supported-hardware-and-interfaces)
4. [Installation and Building](#installation-and-building)
   - [1. Install Prerequisites](#1-install-prerequisites)
   - [2. Clone the Repository](#2-clone-the-repository)
   - [3. Build for Native Platform](#3-build-for-native-platform)
   - [4. Cross-Compile for ARM (Raspberry Pi)](#4-cross-compile-for-arm-raspberry-pi)
   - [5. Deployment on Target Device](#5-deployment-on-target-device)
5. [Configuration](#configuration)
6. [Running the Controller](#running-the-controller)
7. [OpenCV Vision Integration](#opencv-vision-integration)
8. [Licensing and Safety](#licensing-and-safety)
9. [Contributing](#contributing)
10. [Support and Contact](#support-and-contact)

---

## Features

- **High-Speed CAN / ISOBUS Interface:** underdev
  Integrates with SocketCAN at up to a 1 kHz loop rate for real-time message handling. Supports ISOBUS PGNs if used on a tractor’s ISO network.

- **Section Control (Boom Valves):**  
  Controls up to **16** sections with individually configurable widths (or up to **64** uniform sections). Each valve can be PWM controlled or switched on/off.

- **GPS Integration (Rate & Coverage):**  underdev
  Uses GNSS input for application rate adjustments and coverage logging. Compatible with NMEA 0183 or u-blox UBX receivers. Handles high baud rates (e.g. 460800 for RTK F9P).

- **OpenCV Hooks for Camera-based Spraying:**  
  Provides hooks for camera-based weed detection and feed-forward control, enabling “see-and-spray” operations that reduce chemical usage.

- **Modular HAL (Hardware Abstraction Layer):**  underdev
  Cleanly separates core logic from hardware drivers. Swap CAN backends, valve drivers, or GPS sources without modifying the control logic.

- **Async Runtime with Watchdog:**  underdev
  Uses Tokio’s asynchronous runtime for concurrency. Critical tasks (CAN, GPS, control loop) have back-pressure handling and a watchdog that can initiate a safe shutdown if they stall.

- **Telemetry & Monitoring:**  
  Exposes Prometheus metrics for real-time monitoring. Integrates with dashboards like Grafana to track performance, coverage, and system health.

- **Pluggable UIs:**
  - **CLI (`clap`)** for simple command-line control.  
  - **gRPC Service** for remote control and custom integrations.  
  - **Web Dashboard (WIP)** built with Tauri/Leptos for a browser-based GUI.

---

## Architecture and Design

Rust-Spray is implemented as a set of cooperative asynchronous tasks orchestrated by a central controller. These tasks include:

- **Input Handlers:**  
  - *CAN Bus Handler* for sending/receiving CAN frames (SocketCAN).  
  - *GPS Receiver Parser* for reading NMEA 0183 or UBX data from a serial port or USB interface.  
  - *Camera Processor (optional)* for OpenCV-based weed detection or other vision tasks.

- **Control Logic:**  
  Uses sensor inputs to decide how much to spray each section. Automatically handles overlap prevention, variable-rate application based on speed, and camera-driven spot spraying.

- **Output Drivers:**  
  - *Valve Driver:* Sends PWM or on/off commands to boom valves (e.g. TeeJet 344E).  
  - *Flow Meter Reading (optional):* Closes the control loop by sampling pulses from a flow sensor (e.g. DigiFlow 200) to maintain a target rate.

- **Async Coordination:**  
  Built on the Tokio runtime. Tasks communicate via channels; if a task falls behind, back-pressure prevents the entire system from stalling. A watchdog monitors for missed deadlines and enforces a fail-safe shutdown if critical tasks lag.

- **Safety Mechanisms:**  
  Ensures the sprayer defaults to off on errors or timeouts. Provides hooks for external E-stop or manual overrides.

---

## Supported Hardware and Interfaces

- **SBC:** Tested on Raspberry Pi 4/5 (64-bit) and NVIDIA Jetson Nano/Xavier NX.  
- **CAN Interface:** PiCAN 3 HAT or any SocketCAN-compatible transceiver.  
- **Boom Valves:** TeeJet 344E (or other PWM/on-off solenoid valves).  
- **Flow Sensor:** DigiFlow 200 for closed-loop flow control.  
- **GPS Receiver:** u-blox F9P (RTK), typical 10+ Hz update rate.  
- **Camera (Optional):** Pi HQ Camera v2 or USB camera for OpenCV processing.

Refer to the project wiki for wiring diagrams and recommended peripherals.

---

## Installation and Building

### 1. Install Prerequisites

- **Rust Toolchain:**  
  Install via [rustup](https://rustup.rs):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **System Libraries:**
  - **Ubuntu/Debian (native x86_64):**
    ```bash
    sudo apt-get update
    sudo apt-get install libopencv-dev pkg-config build-essential
    ```
  - **For cross-compilation (ARM64):**
    ```bash
    sudo apt-get update
    sudo apt-get install gcc-aarch64-linux-gnu libopencv-dev pkg-config
    rustup target add aarch64-unknown-linux-gnu
    ```

> **OpenCV version**: Rust OpenCV crate (`opencv = "0.89"`) requires OpenCV ≥ 4.8.0. If you see build errors, ensure you have recent OpenCV dev packages (`libopencv-core-dev`, `libopencv-imgproc-dev`, etc.).

### 2. Clone the Repository

```bash
git clone https://github.com/cropcrusaders/Rust-Spray.git
cd Rust-Spray
```

### 3. Build for Native Platform

```bash
cargo build --release
```
Produces `target/release/rust-spray`.

### 4. Cross-Compile for ARM (Raspberry Pi)

```bash
sudo apt-get install gcc-aarch64-linux-gnu pkg-config libopencv-dev
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```
Then copy `target/aarch64-unknown-linux-gnu/release/rust-spray` to the ARM device.

### 5. Deployment on Target Device

- Ensure the device has [OpenCV runtime libs](#1-install-prerequisites).
- Bring up the CAN interface:
  ```bash
  sudo ip link set can0 up type can bitrate 250000
  ```
- Confirm the serial port for your GPS (e.g. `/dev/ttyACM0`) is accessible.

## Configuration


Configuration files in TOML format reside in `config/`. Copy
`config/Config.toml` to `config/config.toml` and adjust the values for your
hardware. The sample file looks like this:

```toml
[camera]
device = "/dev/video2"
resolution_width = 1280
resolution_height = 720

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

- `camera.device` is the path to the video capture device.
- `detection.algorithm` can be `"exg"` or `"hsv"`.
- `spray.pins` lists the GPIO pins used to drive the sprayers.

## Running the Controller

Example command:

```bash
sudo ./target/release/rust-spray \
    --sections 16 \
    --gps /dev/ttyACM0 \
    --can if=can0,bitrate=250000 \
    --config config/default.toml
```

- `--sections 16`: Number of boom sections.  
- `--gps /dev/ttyACM0`: Path to GPS serial port.  
- `--can if=can0,bitrate=250000`: SocketCAN interface name and bitrate.  
- `--config config/default.toml`: Path to TOML config file.

Use `--help` to see all options. Logs go to `logs/rust_spray.log` by default; add `--json` for JSON output.

## OpenCV Vision Integration

- **Camera Capture:** Captures frames via OpenCV from CSI or USB cameras.  
- **Weed Detection:** Plug in your own ML model or OpenCV pipeline to identify weeds/crops.  
- **Feed-Forward Control:** Predict when targets will be under each section and trigger selective spraying.  
- **Performance:** Ensure vision tasks don’t delay the main loop; use hardware acceleration where possible.

## Licensing and Safety

Rust-Spray is **dual-licensed** under **MIT or Apache 2.0**, plus a **Safety Rider** (`LICENSE-SAFETY.md`):

- **User Responsibility:** End users assume all liability for safe operation.  
- **Safety Features:** Modifications must preserve existing safety interlocks and warnings.  
- **Acknowledgment:** Commercial derivatives should credit the original licenses and Safety Rider.

## Contributing

1. **Fork & Branch:**  
   ```bash
   git checkout -b feat/my-new-feature
   ```
2. **Code Quality:**  
   ```bash
   cargo fmt && cargo clippy -- -D warnings
   ```
3. **Commit Messages:** Use [Conventional Commits](https://www.conventionalcommits.org).  
4. **Pull Request:** Submit against `main` with clear descriptions and linked issues.

## Support and Contact

- **GitHub Issues:** For bugs and feature requests.  
- **Discord:** Join the `#rust-spray` channel (invite in Wiki).  
- **Commercial Inquiries:** [support@cropcrusaders.com.au](mailto:support@cropcrusaders.com.au)

**Happy Spraying!**
