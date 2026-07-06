# CLAUDE.md

Guidelines for AI assistants working on the Rust-Spray codebase.

## Project Overview

Rust-Spray is a minimal four-lane spray pipeline for agricultural robotics. It processes camera frames to detect vegetation using SIMD-accelerated color analysis and controls spray nozzles via GPIO. The primary target is embedded Linux on Raspberry Pi (ARM/ARM64), but it builds and runs on desktop with mock GPIO.

It can also run as the **inner loop** of an outer shell such as OpenWeedLocator (OWL): `--ipc-mode` reads framed RGB24 from stdin and writes JSON lane states to stdout (protocol contract in `INTEGRATION.md`), and the cdylib `librustspray_core.so` exposes a C FFI (`rustspray_detect` in `src/ffi.rs`). The OWL-side Python wrapper lives in `owl/detectors/rustspray_detector.py`.

**Package name:** `rustspray` (library crate name: `rustspray_core`, built as rlib + cdylib)
**Edition:** Rust 2021
**Toolchain:** Nightly (required for `#![feature(portable_simd)]`)
**License:** MIT

## Build Commands

```bash
# Standard build (nightly required)
cargo build --release

# Run all unit tests
cargo test

# Run the demo example
cargo run --example four_lane -- --mock-gpio

# Run the Python integration tests (IPC protocol + OWL wrapper)
cargo build --release && pytest tests/test_rustspray_detector.py

# Check formatting and lints (CI enforces both)
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings

# Check with optional camera features
cargo check --no-default-features --features camera-nokhwa
cargo check --no-default-features --features camera-gstreamer

# Cross-compile for Raspberry Pi (requires `cross` tool + Docker)
cross build --release --target aarch64-unknown-linux-gnu --features rpi
cross build --release --target armv7-unknown-linux-gnueabihf --features rpi
```

## CI Pipeline

CI runs on every push and pull request (`.github/workflows/ci.yml`). The full pipeline:

1. `cargo fmt --all -- --check` — formatting must pass
2. `cargo clippy --all-targets -- -D warnings` — lints must pass
3. `cargo check` with `camera-nokhwa` feature — must compile
4. `cargo check` with `camera-gstreamer` feature — must compile
5. `cargo check --features rpi` — desktop fallback path must compile
6. `cargo check --target aarch64-unknown-linux-gnu --features rpi` and `cargo check --target armv7-unknown-linux-gnueabihf --features rpi` — real GPIO code must compile for both Raspberry Pi targets
7. `cargo build --release` — release build must succeed
8. `cargo test` — all tests must pass
9. `cargo run --example four_lane -- --mock-gpio` — example must run
10. `cargo run --release -- --test-pattern --mock-gpio --oneshot` — production binary must run
11. Python job: `pytest tests/test_rustspray_detector.py` against the release binary — IPC protocol and OWL wrapper tests must pass

A separate workflow (`.github/workflows/release.yml`) cross-compiles `rustspray-aarch64` and `rustspray-armv7` on `v*` tags and attaches them to the GitHub release.

Always run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` before committing.

## Repository Structure

```
src/
  main.rs         # Production binary: clap CLI, config, logging, frame loops (incl. IPC)
  watchdog.rs     # Binary-only sd_notify client for the systemd watchdog
  lib.rs          # Crate root; enables portable_simd, exports all modules, kernel tests
  config.rs       # TOML configuration loading + validation
  exg.rs          # SIMD-based Excess Green (ExG) mask computation
  vision.rs       # Adaptive multi-cue vegetation detector (PlantVision)
  lanes.rs        # Lane reduction with hysteresis (LaneReducer)
  pipeline.rs     # Main Pipeline struct combining vision + lanes + GPIO
  io_gpio.rs      # NozzleControl trait, MockGpio, RppalGpio (feature-gated)
  ipc.rs          # IPC protocol v1: framed stdin reader + stdout JSON writer
  ffi.rs          # C FFI entry point (rustspray_detect) for the cdylib
examples/
  four_lane.rs    # Demo: synthetic 640x480 frame through the pipeline
owl/
  detectors/rustspray_detector.py  # OWL Python wrapper (subprocess IPC client)
  README.md       # OWL wiring guide: config schema, factory, shutdown
tests/
  test_rustspray_detector.py       # Python integration tests (pytest + numpy)
INTEGRATION.md    # Versioned IPC/FFI contract for embedding Rust-Spray
```

## Architecture

Three-stage processing pipeline:

1. **Vision** (`vision.rs`): `PlantVision::detect(rgb)` scores each pixel using weighted fusion of ExG, green ratio, and chroma cues. Returns `Vec<bool>` mask.
2. **Lane Reduction** (`lanes.rs`): `LaneReducer::reduce(mask, w, h)` divides the mask into vertical strips and applies hysteresis thresholds to produce per-lane on/off states.
3. **Actuation** (`io_gpio.rs`): `NozzleControl::apply(lanes)` drives GPIO pins (or logs `[MOCK GPIO] lane=N state=ON/OFF` state changes to stderr in mock mode — stdout is reserved for the IPC protocol).

`Pipeline` in `pipeline.rs` orchestrates all three stages for the fixed-size camera path. In `--ipc-mode`, `main.rs` composes the stages directly because frame dimensions arrive per-frame in the stream header.

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

The standalone `exg_mask` function in `exg.rs` provides a fast single-cue mask using `std::simd`. It processes 16 pixels (48 interleaved bytes) per iteration: the R/G/B channels are deinterleaved with stride 3 into `u8x16` vectors, widened to `i16x16` for the ExG arithmetic, and a scalar loop handles the remainder. Note: `PlantVision::detect()` uses its own scalar multi-cue scoring and does not call `exg_mask`.

## Code Conventions

- **Formatting:** Run `cargo fmt` before every commit. CI enforces `cargo fmt --check`.
- **Documentation:** Use Rustdoc comments (`///`) on public functions and types.
- **Functions:** Keep small and focused, single responsibility.
- **Error handling:** Assertions (`assert!`) with descriptive messages for preconditions in the pipeline hot path. Configuration loading (`Config::load`) and validation (`Config::validate`) return `Result<_, String>` — a bad config file must be a hard startup error, never a silent fallback, because default GPIO pins on miswired hardware could actuate the wrong valves. A *missing* config file still yields defaults so testing stays easy.
- **Testing:** Unit tests in `#[cfg(test)]` modules within each source file. Panic tests use `#[should_panic(expected = "...")]`.
- **SIMD:** `exg.rs` uses `std::simd` with `u8x16`/`i16x16` vectors. Scalar fallback handles remainder pixels.
- **Feature gating:** Hardware-specific code uses `#[cfg(all(feature = "rpi", any(target_arch = "arm", target_arch = "aarch64")))]` — both conditions are required because `rppal` is a target-specific dependency, so `--features rpi` on a desktop host must still compile (falling back to mock GPIO).

## Dependencies

- **bytemuck** (1.15): Safe type transmutation with `extern_crate_alloc`
- **crossbeam** (0.8): Concurrency primitives
- **clap** (4.5): CLI argument parsing (derive)
- **serde_json** (1): IPC response serialization
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

All Rust unit tests live alongside their modules:

- `config.rs`: 6 tests (sane defaults, partial TOML, full round-trip, missing file, validation failures)
- `exg.rs`: 4 tests (green detection, non-green rejection, interleaved SIMD path, SIMD-vs-scalar agreement)
- `vision.rs`: 3 tests (bright green, dry soil, weight overrides)
- `lanes.rs`: 5 tests (hysteresis, zero-lanes panic, edge cases)
- `io_gpio.rs`: 1 test (mock GPIO state-change tracking)
- `ipc.rs`: 7 tests (framing round-trip, clean EOF, truncation, bad headers, JSON schema)
- `ffi.rs`: 5 tests (detection via the C ABI, null/invalid args, missing config)
- `lib.rs` (`kernel_tests`): 5 end-to-end kernel tests (all weed, no weed, single pixel, lane mapping, lane boundary)

Run with `cargo test`. Tests use synthetic pixel data and need no external fixtures or hardware.

Python integration tests in `tests/test_rustspray_detector.py` (pytest + numpy) exercise the built release binary end-to-end: protocol handshake, detection, hysteresis across frames, subprocess kill/restart, timeout, shutdown, and the raw wire format. Run `cargo build --release` first.
