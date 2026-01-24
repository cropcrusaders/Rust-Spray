# LaneRustSpray

LaneRustSpray turns lane detection into a targeted rust spray workflow.
The runtime locks onto the row, detects rust only inside the lane, projects
pixel hits onto the ground, and schedules nozzle pulses based on rig geometry
and forward speed.

## Goals

1. Lock the lane (row centerline + vehicle offset).
2. Scan the canopy with RGB (optionally multispectral).
3. Detect rust lesions in-lane only (ignore out-of-lane noise).
4. Convert pixel hits → ground hits (distance + camera model).
5. Schedule nozzle fire using forward speed + boom offset.
6. Log everything for proof + map layers + ROI.

## Repository Layout

```
LaneRustSpray/
  config/
    rig.yaml
    camera.yaml
    nozzles.yaml
  src/
    lane_ai/
      lane_model.py
      lane_tracker.py
    rust_ai/
      rust_model.py
      rust_post.py
    fusion/
      geo.py
      timing.py
    control/
      nozzle_driver.py
      spray_scheduler.py
      safety.py
    app.py
  logs/
  tests/
```

## Quick Start

```bash
cargo build --release
cargo run --example four_lane -- --mock-gpio
```

The example processes a synthetic frame and prints which lanes would spray.

## Documentation

* [System spec and runtime flow](spec.md)

## Cross Compile

Install [`cross`](https://github.com/cross-rs/cross) and build for Raspberry Pi:

```bash
cargo install --git https://github.com/cross-rs/cross cross --locked
cross build --release --target aarch64-unknown-linux-gnu --features rpi
```

Adjust the target for other Pi variants. The project uses `std::simd` for the ExG mask, lane reduction with hysteresis and optional GPIO control via `rppal`.

## Tests

Run the unit tests:

```bash
cargo test
```

## License

MIT
