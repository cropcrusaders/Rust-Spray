# CLAUDE.md

Guidelines for AI assistants working on the Rust-Spray codebase.

## Project Overview

Rust-Spray is a minimal four-lane spray pipeline for agricultural robotics. It processes camera frames to detect vegetation using SIMD-accelerated color analysis (or a trained ONNX model) and controls spray nozzles via GPIO. The primary target is embedded Linux on Raspberry Pi (ARM/ARM64), but it builds and runs on desktop with mock GPIO.

**Package name:** `rustspray`
**Edition:** Rust 2021
**Toolchain:** Nightly (required for `#![feature(portable_simd)]`)
**License:** MIT

## Build Commands

```bash
# Standard build (nightly required)
cargo build --release

# Run all unit tests (16 tests)
cargo test

# Run the demo example
cargo run --example four_lane -- --mock-gpio

# Run with a trained ONNX model
cargo run --features model --example four_lane -- --mock-gpio --model vegetation.onnx

# Check formatting
cargo fmt --all -- --check

# Check with optional camera features
cargo check --no-default-features --features camera-nokhwa
cargo check --no-default-features --features camera-gstreamer

# Build with ONNX model support
cargo build --release --features model

# Cross-compile for Raspberry Pi (requires `cross` tool + Docker)
cross build --release --target aarch64-unknown-linux-gnu --features rpi
cross build --release --target armv7-unknown-linux-gnueabihf
```

## Raspberry Pi Installation

One-step install on a Raspberry Pi:

```bash
bash scripts/install.sh
```

The script installs system packages, sets up Rust nightly, builds with GPIO support, and optionally installs a systemd service. Use `--no-service` to skip systemd setup. To include ONNX model support:

```bash
WITH_MODEL=1 bash scripts/install.sh
```

## CI Pipeline

CI runs on every push and pull request (`.github/workflows/ci.yml`). The full pipeline:

1. `cargo fmt --all -- --check` — formatting must pass
2. `cargo check` with `camera-nokhwa` feature — must compile
3. `cargo check` with `camera-gstreamer` feature — must compile
4. `cargo build --release` — release build must succeed
5. `cargo test` — all tests must pass
6. `cargo run --example four_lane -- --mock-gpio` — example must run

Always run `cargo fmt` and `cargo test` before committing.

## Repository Structure

```
src/
  lib.rs          # Crate root; enables portable_simd, exports all modules
  exg.rs          # SIMD-based Excess Green (ExG) mask computation
  vision.rs       # Detector trait, adaptive PlantVision detector
  model.rs        # ONNX model inference via tract (feature-gated: model)
  lanes.rs        # Lane reduction with hysteresis, temporal hold, and ROI
  pipeline.rs     # Main Pipeline struct combining detector + lanes + GPIO
  io_gpio.rs      # NozzleControl trait, MockGpio, RppalGpio (feature-gated)
examples/
  four_lane.rs    # Demo: synthetic 640x480 frame through the pipeline
scripts/
  train_model.py  # Train vegetation MLP in PyTorch, export to ONNX
  install.sh      # One-step Raspberry Pi installer with systemd service
```

## Architecture

Three-stage processing pipeline:

1. **Detection** (`vision.rs`, `model.rs`): A `Detector` classifies each pixel as vegetation. Built-in `PlantVision` uses weighted ExG/green-ratio/chroma fusion. Optional `ModelDetector` loads a trained ONNX model for learned classification.
2. **Lane Reduction** (`lanes.rs`): `LaneReducer::reduce(mask, w, h)` divides the mask into vertical strips and applies hysteresis thresholds with optional temporal hold and vertical ROI to produce per-lane on/off states. Exposes per-lane vegetation density.
3. **Actuation** (`io_gpio.rs`): `NozzleControl::apply(lanes)` drives GPIO pins (or prints to stdout in mock mode).

`Pipeline` in `pipeline.rs` orchestrates all three stages and accepts any `Box<dyn Detector>`.

## Key Types and Traits

| Type | Module | Purpose |
|------|--------|---------|
| `Detector` (trait) | `vision.rs` | Common interface for vegetation detection strategies |
| `PlantVision` | `vision.rs` | Built-in colour detector with tunable weights and thresholds |
| `ModelDetector` | `model.rs` | ONNX model inference via tract (feature: `model`) |
| `LaneReducer` | `lanes.rs` | Reduces 2D boolean mask to N lane states with hysteresis, temporal hold, ROI, and per-lane density |
| `Pipeline` | `pipeline.rs` | Combines detector, reducer, and GPIO into a single `process(frame)` call; returns lane states |
| `NozzleControl` (trait) | `io_gpio.rs` | Abstraction for spray actuation; implementations: `MockGpio`, `RppalGpio` |

## Cargo Features

| Feature | Effect |
|---------|--------|
| `rpi` | Enables `rppal` for real GPIO on ARM/ARM64 |
| `camera-nokhwa` | Enables V4L2 camera input via `nokhwa` |
| `camera-gstreamer` | Enables GStreamer camera input |
| `model` | Enables ONNX model inference via `tract-onnx` |

No features are enabled by default.

## ML Model Training

Train a vegetation classifier and export to ONNX:

```bash
pip install torch numpy onnx
python scripts/train_model.py                          # synthetic data
python scripts/train_model.py --data-dir path/to/imgs  # real images
```

The `--data-dir` expects `vegetation/` and `background/` subdirectories with PNG/JPG images. The exported `vegetation.onnx` can then be loaded by `ModelDetector::load()`.

## Code Conventions

- **Formatting:** Run `cargo fmt` before every commit. CI enforces `cargo fmt --check`.
- **Documentation:** Use Rustdoc comments (`///`) on public functions and types.
- **Functions:** Keep small and focused, single responsibility.
- **Error handling:** Assertions (`assert!`) with descriptive messages for preconditions. No `Result`/`Error` types in the current API — panics on invalid input (appropriate for the embedded use case).
- **Testing:** Unit tests in `#[cfg(test)]` modules within each source file. Panic tests use `#[should_panic(expected = "...")]`.
- **SIMD:** `exg.rs` uses `std::simd` with `u8x16`/`i16x16` vectors. Scalar fallback handles remainder pixels.
- **Feature gating:** Hardware-specific code uses `#[cfg(feature = "...")]` and target-arch cfg guards.

## Dependencies

- **bytemuck** (1.15): Safe type transmutation with `extern_crate_alloc`
- **crossbeam** (0.8): Concurrency primitives
- **rppal** (0.17, optional, ARM only): Raspberry Pi GPIO control
- **nokhwa** (0.10, optional): V4L2 camera capture
- **gstreamer** (0.21, optional): GStreamer media framework
- **tract-onnx** (0.21, optional): ONNX model inference (pure Rust, no system deps)

## System Dependencies (for CI / full feature builds)

```bash
sudo apt-get install -y \
  pkg-config \
  libgstreamer1.0-dev \
  libgstreamer-plugins-base1.0-dev \
  libglib2.0-dev \
  libudev-dev \
  libv4l-dev
```

## Cross-Compilation

The project uses the [`cross`](https://github.com/cross-rs/cross) tool with Docker. Configuration is in `Cross.toml`. Supported targets:

- `aarch64-unknown-linux-gnu` (RPi 4/5)
- `armv7-unknown-linux-gnueabihf` (RPi 3)

Target-specific rustflags are in `.cargo/config.toml`.

Install cross from Git (not crates.io):
```bash
cargo install --git https://github.com/cross-rs/cross cross --locked
```

## Testing

All 16 unit tests live alongside their modules:

- `exg.rs`: 2 tests (green detection, non-green rejection)
- `vision.rs`: 3 tests (bright green, dry soil, weight overrides)
- `lanes.rs`: 11 tests (hysteresis, temporal hold, ROI, density, 4-lane detection, panics, edge cases)

Run with `cargo test`. Tests use synthetic pixel data and need no external fixtures or hardware.
