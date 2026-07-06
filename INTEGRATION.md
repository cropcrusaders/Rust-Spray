# Rust-Spray Integration Guide

How to embed Rust-Spray as a high-performance detection + actuation inner
loop inside a host system (e.g. [OpenWeedLocator](https://github.com/geezacoleman/OpenWeedLocator)),
where the host owns frame capture, scheduling, logging, and supervision.

Two integration surfaces are provided:

| Surface | Mechanism | State | Use when |
|---------|-----------|-------|----------|
| **IPC subprocess** (`rustspray --ipc-mode`) | frames on stdin, JSON on stdout | Stateful: lane hysteresis, GPIO actuation | Production. The host supervises the process. |
| **C FFI** (`librustspray_core.so`) | direct function call | Stateless: detection only, no GPIO | Embedding without a subprocess; custom actuation. |

---

## 1. Protocol version history

| Version | Date | Changes |
|---------|------|---------|
| 1 | 2026-07 | Initial protocol: LE `u32` width/height header + RGB24 payload in; NDJSON lane states out. |

The protocol version is bumped for any breaking change to the frame
encoding, the response schema (removing/renaming fields), or the
handshake. *Adding* JSON fields is non-breaking within a version; hosts
must ignore unknown fields.

## 2. Frame encoding (host → rustspray, stdin)

Each frame is sent as a header immediately followed by the pixel payload.
Frames are sent back-to-back with no delimiters.

| Offset | Size | Type | Meaning |
|--------|------|------|---------|
| 0 | 4 bytes | `u32` little-endian | frame width in pixels |
| 4 | 4 bytes | `u32` little-endian | frame height in pixels |
| 8 | `width * height * 3` bytes | `u8` | RGB24 pixels, row-major, interleaved `R₀G₀B₀R₁G₁B₁…`, top-left first |

Rules:

- Valid dimensions are `1..=4096` per axis. Out-of-range dimensions are a
  fatal protocol error.
- The **first frame fixes the dimensions** for the session. A later frame
  with different dimensions is a fatal protocol error — restart the
  subprocess to change resolution.
- Frame width must be ≥ the configured lane count.
- Closing stdin at a frame boundary is the clean shutdown signal
  (exit code 0). EOF mid-header or mid-payload is a fatal error.
- Write the header and payload in as few `write` calls as possible
  (ideally one) to minimise latency; rustspray handles arbitrary
  fragmentation correctly either way.

## 3. Response JSON schema (rustspray → host, stdout)

Exactly one JSON object per processed frame, newline-terminated
(NDJSON), flushed immediately. stdout carries **only** this stream;
all logging goes to stderr.

```json
{"v":1,"frame":42,"ts_us":1718000000123456,"lanes":[true,false,false,true],"latency_us":1840}
```

| Field | Type | Units | Range / meaning |
|-------|------|-------|-----------------|
| `v` | integer | — | Protocol version. Always `1` for this document. |
| `frame` | integer (u64) | — | Monotonically increasing counter, **starts at 1**, increments by 1 per frame. |
| `ts_us` | integer (u64) | microseconds | Unix time when the frame was fully received. |
| `lanes` | array of bool | — | One entry per configured spray lane (`lanes.count` in the TOML), lane 0 first (leftmost image strip). `true` = spraying. |
| `latency_us` | integer (u64) | microseconds | Wall-clock detection + actuation time for this frame. |

The lane count comes from Rust-Spray's TOML config, not from the frame —
the host and the TOML must agree on the number of lanes.

## 4. Startup handshake

Before streaming frames, the host verifies compatibility:

```console
$ rustspray --output-version
{"rustspray_version":"0.3.0","ipc_protocol":1}
```

The host must check `ipc_protocol` equals the version it implements and
refuse to start (or fall back to its own detector) on mismatch.

## 5. Error behaviour

- **stderr** carries human-readable logs (`env_logger` format; level set
  by `--log-level`, `RUST_LOG`, or the TOML). Nothing on stderr is
  machine-parseable contract.
- Exit codes:

| Code | Meaning |
|------|---------|
| 0 | Clean shutdown: stdin EOF at a frame boundary, or SIGINT/SIGTERM. |
| 1 | Fatal protocol error (bad/changed dimensions, truncated stream, stdout write failure) or no input source. |
| 2 | Configuration error (unparseable TOML, failed validation, conflicting flags). |
| 3 | Stall fail-safe: no frame received for `camera.stall_timeout_secs` (default 10 s; `0` disables). |

- **All nozzles are switched off before exiting on every path**, including
  errors and signals. On SIGKILL no cleanup can run — hosts should send
  SIGTERM and allow a grace period, and wire valves normally-closed.
- If the host stops reading stdout, rustspray eventually blocks on the
  pipe; the stall detector does not fire while frames keep arriving, so
  hosts must keep draining responses.

## 6. GPIO backend abstraction

Actuation goes through one trait
([`src/io_gpio.rs`](src/io_gpio.rs)):

```rust
pub trait NozzleControl {
    /// Apply lane activations, one bool per lane, lane 0 first.
    fn apply(&mut self, lanes: &[bool]);
}
```

Implementations included: `RppalGpio` (Raspberry Pi header pins via
`rppal`, feature `rpi`) and `MockGpio` (stderr logging). To drive a CAN
bus, ISOBUS gateway, or serial relay board instead:

1. Add a struct implementing `NozzleControl` in `src/io_gpio.rs` (or a
   new module). `apply` is called once per frame with the full lane
   state; implementations should be idempotent and cheap for unchanged
   states.
2. If it needs hardware-specific dependencies, gate it behind a Cargo
   feature the way `rpi` gates `rppal`.
3. Construct it in `build_real_gpio()` in `src/main.rs` (typically
   selected by a new field in the `[gpio]` TOML section).
4. Make `Drop`/shutdown drive all outputs to the OFF state — the
   pipeline calls `apply(&[false, …])` on shutdown, but your backend is
   the last line of defence.

## 7. Build targets

Rust-Spray requires **nightly Rust** (`portable_simd`); the toolchain is
pinned by `rust-toolchain.toml`.

| Target | Hardware | Notes |
|--------|----------|-------|
| `aarch64-unknown-linux-gnu` | Raspberry Pi 4/5, 64-bit Pi OS | Recommended. Prebuilt `rustspray-aarch64` attached to releases. |
| `armv7-unknown-linux-gnueabihf` | Raspberry Pi 3B+, 32-bit Pi OS | Prebuilt `rustspray-armv7` attached to releases. |
| `x86_64-unknown-linux-gnu` | Desktop/CI | Mock GPIO only. |

Minimum platform: Linux kernel ≥ 4.19 with glibc (Raspberry Pi OS
Bullseye or newer). Real GPIO additionally needs `/dev/gpiomem*` (root or
`gpio` group membership) and the `rpi` Cargo feature.

The FFI shared library is produced by `cargo build --release` as
`target/release/librustspray_core.so` (crate-type `cdylib`).

## 8. Example integration (Python)

```python
import json, struct, subprocess
import numpy as np

proc = subprocess.Popen(
    ["rustspray", "--ipc-mode", "--config", "/etc/rustspray/config.toml"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE)

def detect(frame: np.ndarray) -> list[bool]:   # frame: HxWx3 uint8 RGB
    h, w, _ = frame.shape
    proc.stdin.write(struct.pack("<II", w, h) + frame.tobytes())
    proc.stdin.flush()
    return json.loads(proc.stdout.readline())["lanes"]
```

Production hosts should additionally: verify `--output-version` at
startup, enforce a per-frame read timeout, restart the subprocess on
death/timeout (bounded retries, then fall back to a native detector),
and call `proc.terminate()` on shutdown.

### FFI alternative (stateless, no GPIO)

```python
import ctypes
lib = ctypes.CDLL("librustspray_core.so")
lanes = (ctypes.c_bool * 4)()
rc = lib.rustspray_detect(
    frame.ctypes.data_as(ctypes.POINTER(ctypes.c_ubyte)),
    w, h, None, lanes, 4)
```

`rustspray_detect` returns `0` on success or a negative errno (see
[`src/ffi.rs`](src/ffi.rs)). FFI v1 is stateless: no lane hysteresis
(the `on` threshold alone decides), no GPIO actuation, and the config
file (if given) is re-read on every call — prefer the IPC subprocess
for per-frame hot loops.
