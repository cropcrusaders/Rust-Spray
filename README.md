# Rust-Spray

Minimal four-lane spray pipeline in Rust.

## Quick Start

```bash
cargo build --release
cargo run --example four_lane -- --mock-gpio
```

The example processes a synthetic frame and prints which lanes would spray.

## Cross Compile

Install [`cross`](https://github.com/cross-rs/cross) and build for Raspberry Pi:

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

