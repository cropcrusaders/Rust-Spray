# Rust-Spray Integration Contract

Rust-Spray can run as the **inner loop** (detection + GPIO actuation) of a
larger system — the reference outer shell is
[OpenWeedLocator (OWL)](https://github.com/geezacoleman/OpenWeedLocator),
whose Python process owns frame capture, scheduling, logging, and the
dashboard. This document is the authoritative contract between the two
sides. It is versioned so any manufacturer can replace either side.

Two integration surfaces exist:

1. **Subprocess IPC** (recommended) — spawn `rustspray --ipc-mode`, pipe
   framed RGB24 into stdin, read JSON lane states from stdout. Rust-Spray
   drives GPIO itself and keeps lane hysteresis across frames.
2. **C FFI** — link `librustspray_core.so` and call `rustspray_detect()`
   in-process. Pure detection kernel: no GPIO, no cross-frame hysteresis.

---

## 1. Protocol version history

| Version | Date       | Changes |
|---------|------------|---------|
| 1       | 2026-07-06 | Initial protocol: 8-byte LE frame header + RGB24 payload on stdin; NDJSON responses (`v`, `frame`, `ts_us`, `lanes`, `latency_us`) on stdout; `--output-version` handshake. |

Compatibility rules:

- The `v` field in every response names the protocol the binary is
  speaking. Consumers **must** reject a `v` they do not implement.
- Incompatible changes (header layout, field removal, semantic changes)
  bump the version. New *optional* response fields may be added within a
  version; consumers must ignore unknown fields.
- The version constant lives in `src/ipc.rs`
  (`ipc::IPC_PROTOCOL_VERSION`) and is reported by `--output-version`.

## 2. Frame encoding (stdin)

Each frame is a header followed immediately by the pixel payload. Frames
are sent back-to-back with no delimiters or padding.

| Offset | Size            | Type    | Endianness    | Field |
|--------|-----------------|---------|---------------|-------|
| 0      | 4 bytes         | `u32`   | little-endian | `width` in pixels (> 0) |
| 4      | 4 bytes         | `u32`   | little-endian | `height` in pixels (> 0) |
| 8      | `width*height*3`| `u8[]`  | n/a           | RGB24 pixels, row-major, interleaved `R₀G₀B₀R₁G₁B₁…`, top-left pixel first |

Constraints:

- `width` must be ≥ the configured lane count.
- The payload must not exceed 64 MiB (`width * height * 3 ≤ 67 108 864`);
  larger headers are treated as stream corruption.
- Dimensions may vary between frames; lane hysteresis state is preserved.
- Write the header and payload in a **single write** where possible so a
  crashed producer never leaves a torn frame in the pipe.

Closing stdin at a frame boundary is the clean-shutdown signal: Rust-Spray
forces all lanes off and exits 0.

## 3. Response JSON schema (stdout)

One JSON object per processed frame, newline-delimited (NDJSON), flushed
after every frame. Nothing else is ever written to stdout in IPC mode.

```json
{"v":1,"frame":42,"ts_us":1718000000123456,"lanes":[true,false,false,true],"latency_us":1840}
```

| Field        | Type        | Units        | Range / semantics |
|--------------|-------------|--------------|-------------------|
| `v`          | integer     | —            | Protocol version; `1` for this document. |
| `frame`      | integer u64 | —            | Frame counter, starts at `0`, increments by 1 per processed frame. Resets when the process restarts. |
| `ts_us`      | integer u64 | microseconds | Unix time at frame receipt (after the full payload was read). |
| `lanes`      | bool array  | —            | One entry per configured spray lane (`[lanes] count` in the TOML), index 0 = leftmost image strip. `true` = spray. |
| `latency_us` | integer u64 | microseconds | Detection + actuation latency for this frame (excludes pipe transfer time). |

GPIO: unless `--mock-gpio` is passed (or `[gpio] mock = true`), Rust-Spray
applies `lanes` to its configured pins **before** the response is written,
so the JSON is a report of what was actuated, not a request.

## 4. Startup handshake

Before streaming frames, the outer shell verifies compatibility:

```console
$ rustspray --output-version
{"rustspray_version":"0.3.0","ipc_protocol":1}
```

- Prints exactly one JSON line to stdout and exits 0. Works without a
  config file.
- `ipc_protocol` is the integer protocol version this binary speaks.
- If it does not match the version the shell implements, do not start
  IPC mode — fall back or upgrade.

## 5. Error behaviour

stderr carries human-readable logs only (via `env_logger`; level set by
`--log-level`, `RUST_LOG`, or the TOML `[logging]` section). With
`--mock-gpio`, lane state changes are also logged to stderr as
`[MOCK GPIO] lane=N state=ON/OFF`. **Never parse stderr programmatically.**

Exit behaviour (every exit path first drives all lanes off):

| Condition | Behaviour |
|-----------|-----------|
| stdin closed at a frame boundary | Logs `end of input stream`, exits **0**. |
| SIGINT / SIGTERM | Finishes the in-flight frame, exits **0**. |
| Truncated header/payload, zero or oversized dimensions, `width` < lane count | Logs the reason, exits **2** — the stream is out of sync and cannot be resynchronised. |
| Config file unreadable/invalid at startup | Error on stderr, exits **2** before reading any frame. |
| stdout write fails (outer shell died) | Exits **2**. |
| Camera stall (non-IPC stdin mode only) | Exits **3**. |

The outer shell should treat any nonzero exit or response timeout as
"restart the subprocess", with a bounded restart budget and a fallback
detector after that (the reference wrapper in
`owl/detectors/rustspray_detector.py` implements exactly this).

## 6. GPIO pin configuration

Rust-Spray addresses pins by **BCM number** (rppal's numbering). Two
sources, in precedence order:

1. `--gpio-pins 27,22,23,24` — comma-separated BCM numbers, one per
   lane, on the command line. **Outer shells must use this**, passing
   the pins derived from their own relay config, so both sides address
   the same solenoids by construction. If the count does not match
   `[lanes] count`, startup fails with exit 2 before any frame is read.
2. `[gpio] pins` in the TOML — used only when the flag is absent
   (standalone deployments where the TOML is the single source of
   truth).

The defaults on both sides are OWL's stock relay wiring: OWL's
`[Relays]` BOARD (physical header) pins 13, 15, 16, 18 = Rust-Spray's
default BCM 27, 22, 23, 24. The reference wrapper
(`owl/detectors/rustspray_detector.py`) accepts OWL's BOARD pins via its
`board_pins` argument, translates them to BCM, and forwards them with
`--gpio-pins` — never hand-copy pin numbers between the two configs.

## 7. GPIO backend abstraction

Actuation goes through the `NozzleControl` trait
(`src/io_gpio.rs`):

```rust
pub trait NozzleControl {
    /// Apply lane activations, index 0 = leftmost lane.
    fn apply(&mut self, lanes: &[bool]);
}
```

Shipped implementations: `MockGpio` (stderr logging) and `RppalGpio`
(Raspberry Pi BCM pins, compiled only with `--features rpi` on ARM).

To add a new backend (CAN bus, ISOBUS section control, serial relay
board):

1. Implement `NozzleControl` in `src/io_gpio.rs` (or a new module).
   Feature-gate hardware-specific dependencies the same way `rppal` is
   gated: `#[cfg(all(feature = "...", any(target_arch = ...)))]`.
2. Make `apply` idempotent and cheap — it runs once per frame.
3. Guarantee the fail-safe: all lanes must be off after construction and
   after `Drop` (see `RppalGpio::drop`); never leave an output floating
   in a state that could energise a relay.
4. Construct it in `build_real_gpio()` in `src/main.rs`, selected by a
   new config key (e.g. `[gpio] backend = "canbus"`).
5. Config extensions (bitrates, node IDs, …) go in `src/config.rs` with
   validation in `Config::validate` — invalid actuation config must be a
   hard startup error.

## 8. Build targets

Requires **nightly Rust** (`#![feature(portable_simd)]`); the toolchain is
pinned by `rust-toolchain.toml`.

| Target | Hardware | Notes |
|--------|----------|-------|
| `aarch64-unknown-linux-gnu` | Raspberry Pi 4/5, 64-bit OS | Build with `--features rpi` for real GPIO. Release binary: `rustspray-aarch64`. |
| `armv7-unknown-linux-gnueabihf` | Raspberry Pi 3B+, 32-bit OS | Build with `--features rpi`. Release binary: `rustspray-armv7`. |
| `x86_64-unknown-linux-gnu` | Development / CI | GPIO falls back to mock. |

Minimum Linux kernel: **4.8** (required by `rppal`'s `/dev/gpiomem`
interface and the memory-mapped GPIO on Pi OS; any Raspberry Pi OS or
Ubuntu release from 2017 onward qualifies). glibc per the standard Rust
target requirements (2.17+).

Artifacts per release tag (see `.github/workflows/release.yml`):
`rustspray-aarch64`, `rustspray-armv7`, attached to the GitHub release.

The cdylib `librustspray_core.so` is produced by `cargo build --release`
for FFI embedding; its C ABI is documented in `src/ffi.rs`
(`rustspray_detect`).

## 9. Example integration (Python)

```python
import json, struct, subprocess

proc = subprocess.Popen(
    ["/usr/local/bin/rustspray", "--ipc-mode", "--config", "/etc/rustspray/config.toml"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, bufsize=0,
)
frame = get_rgb24_frame()                 # numpy uint8 array, HxWx3, RGB order
h, w = frame.shape[:2]
proc.stdin.write(struct.pack("<II", w, h) + frame.tobytes())
proc.stdin.flush()
lanes = json.loads(proc.stdout.readline())["lanes"]   # e.g. [True, False, False, True]
proc.stdin.close()                        # clean shutdown: all lanes off, exit 0
```

For production use (timeouts, restarts, protocol verification, fallback),
use the reference wrapper: `owl/detectors/rustspray_detector.py`.
