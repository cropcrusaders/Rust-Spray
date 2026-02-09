# CLAUDE.md

Guidelines for AI assistants working on the Rust-Spray codebase.

## Project Overview

Rust-Spray is a minimal four-lane spray pipeline for agricultural robotics. It processes camera frames to detect vegetation using SIMD-accelerated color analysis and controls spray nozzles via GPIO. The primary target is embedded Linux on Raspberry Pi (ARM/ARM64), but it builds and runs on desktop with mock GPIO.

**Package name:** `rustspray`
**Edition:** Rust 2021
**Toolchain:** Nightly (required for `#![feature(portable_simd)]`)
**License:** MIT

## Build Commands

```bash
# Standard build (nightly required)
cargo build --release

# Run all unit tests (10 tests)
cargo test

# Run the demo example
cargo run --example four_lane -- --mock-gpio

# Check formatting
cargo fmt --all -- --check

# Check with optional camera features
cargo check --no-default-features --features camera-nokhwa
cargo check --no-default-features --features camera-gstreamer

# Cross-compile for Raspberry Pi (requires `cross` tool + Docker)
cross build --release --target aarch64-unknown-linux-gnu --features rpi
cross build --release --target armv7-unknown-linux-gnueabihf
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
  vision.rs       # Adaptive multi-cue vegetation detector (PlantVision)
  lanes.rs        # Lane reduction with hysteresis (LaneReducer)
  pipeline.rs     # Main Pipeline struct combining vision + lanes + GPIO
  io_gpio.rs      # NozzleControl trait, MockGpio, RppalGpio (feature-gated)
examples/
  four_lane.rs    # Demo: synthetic 640x480 frame through the pipeline
```

## Architecture

Three-stage processing pipeline:

1. **Vision** (`vision.rs`): `PlantVision::detect(rgb)` scores each pixel using weighted fusion of ExG, green ratio, and chroma cues. Returns `Vec<bool>` mask.
2. **Lane Reduction** (`lanes.rs`): `LaneReducer::reduce(mask, w, h)` divides the mask into vertical strips and applies hysteresis thresholds to produce per-lane on/off states.
3. **Actuation** (`io_gpio.rs`): `NozzleControl::apply(lanes)` drives GPIO pins (or prints to stdout in mock mode).

`Pipeline` in `pipeline.rs` orchestrates all three stages.

## Key Types and Traits

| Type | Module | Purpose |
|------|--------|---------|
| `PlantVision` | `vision.rs` | Configurable vegetation detector with tunable weights and thresholds |
| `LaneReducer` | `lanes.rs` | Reduces 2D boolean mask to N lane states with hysteresis |
| `Pipeline` | `pipeline.rs` | Combines vision, reducer, and GPIO into a single `process(frame)` call |
| `NozzleControl` (trait) | `io_gpio.rs` | Abstraction for spray actuation; implementations: `MockGpio`, `RppalGpio` |

## Cargo Features

| Feature | Effect |
|---------|--------|
| `rpi` | Enables `rppal` for real GPIO on ARM/ARM64 |
| `camera-nokhwa` | Enables V4L2 camera input via `nokhwa` |
| `camera-gstreamer` | Enables GStreamer camera input |

No features are enabled by default.

## Data Formats

All image data uses **interleaved RGB** byte layout: `R₀G₀B₀R₁G₁B₁…Rₙ₋₁Gₙ₋₁Bₙ₋₁`.

- `frame.len()` must equal `width * height * 3`.
- Pixel values are `u8` (0–255).
- `PlantVision::detect()` returns a `Vec<bool>` with one entry per pixel (`width * height` elements).
- `LaneReducer::reduce()` expects the mask plus `width` and `height` to compute per-lane coverage ratios.

## Default Configuration Values

### PlantVision (from `Default` impl in `vision.rs`)

| Parameter | Value | Description |
|-----------|-------|-------------|
| `exg_threshold` | `20` | Minimum ExG response before considering a pixel |
| `green_ratio_floor` | `0.36` | Minimum green/(R+G+B+1) ratio |
| `chroma_floor` | `0.08` | Minimum (max−min)/255 to reject grey/brown |
| Weights | `exg=0.5, green_ratio=0.35, chroma=0.15, bias=0.0` | Fusion weights for the scoring function |

### LaneReducer (typical values from the `four_lane` example)

| Parameter | Value | Description |
|-----------|-------|-------------|
| `lanes` | `4` | Number of spray lanes |
| `on` threshold | `0.3` | Coverage ratio to activate a lane |
| `off` threshold | `0.15` | Coverage ratio to deactivate (hysteresis) |

## Algorithm Details

### Scoring (`PlantVision::score_pixel`)

Each pixel gets a weighted linear score:

```
score = w_exg * (ExG − threshold) / 255
      + w_gr  * (green_ratio − floor)
      + w_chr * (chroma − floor)
      + bias
```

Where:
- `ExG = 2G − R − B` (Excess Green index)
- `green_ratio = G / (R + G + B + 1)` (normalized green share)
- `chroma = (max(R,G,B) − min(R,G,B)) / 255` (color saturation)

A pixel is marked as vegetation when `score > 0.0`.

### Lane Hysteresis (`LaneReducer::reduce`)

Each lane occupies a vertical strip of the image. The lane width is `floor(width/lanes)`, with the first `width % lanes` lanes getting one extra column. For each lane the ratio of `true` pixels to total pixels is computed:

- **Off → On:** ratio must exceed the `on` threshold.
- **On → Off:** ratio must drop below the `off` threshold.

This prevents rapid toggling when vegetation coverage hovers near the threshold.

### ExG SIMD (`exg::exg_mask`)

The standalone `exg_mask` function in `exg.rs` provides a fast single-cue mask using `std::simd`. It processes 16 pixels per iteration using `u8x16`/`i16x16` vectors, with a scalar loop handling the remainder. Note: `PlantVision::detect()` uses its own scalar multi-cue scoring and does not call `exg_mask`.

## Code Conventions

- **Formatting:** Run `cargo fmt` before every commit. CI enforces `cargo fmt --check`.
- **Documentation:** Use Rustdoc comments (`///`) on public functions and types.
- **Functions:** Keep small and focused, single responsibility.
- **Error handling:** Assertions (`assert!`) with descriptive messages for preconditions. No `Result`/`Error` types in the current API — panics on invalid input (appropriate for the embedded use case).
- **Testing:** Unit tests in `#[cfg(test)]` modules within each source file. Panic tests use `#[should_panic(expected = "...")]`.
- **SIMD:** `exg.rs` uses `std::simd` with `u8x16`/`i16x16` vectors. Scalar fallback handles remainder pixels.
- **Feature gating:** Hardware-specific code uses `#[cfg(feature = "rpi")]` and target-arch cfg guards.

## Dependencies

- **bytemuck** (1.15): Safe type transmutation with `extern_crate_alloc`
- **crossbeam** (0.8): Concurrency primitives
- **rppal** (0.17, optional, ARM only): Raspberry Pi GPIO control
- **nokhwa** (0.10, optional): V4L2 camera capture
- **gstreamer** (0.21, optional): GStreamer media framework

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

All 10 unit tests live alongside their modules:

- `exg.rs`: 2 tests (green detection, non-green rejection)
- `vision.rs`: 3 tests (bright green, dry soil, weight overrides)
- `lanes.rs`: 5 tests (hysteresis, zero-lanes panic, edge cases)

Run with `cargo test`. Tests use synthetic pixel data and need no external fixtures or hardware.
