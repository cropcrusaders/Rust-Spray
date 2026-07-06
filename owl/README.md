# OWL Integration Files

This directory contains the OpenWeedLocator (OWL) side of the Rust-Spray
integration: a drop-in Python detector backend that runs the `rustspray`
binary as a high-performance inner loop (detection + GPIO), while OWL's
Python remains the outer shell (picamera2 capture, scheduling, logging,
dashboard, YOLO fallback).

These files live in the Rust-Spray repository so the two sides of the
versioned IPC contract (see [`INTEGRATION.md`](../INTEGRATION.md)) ship
together. To use them, copy `detectors/rustspray_detector.py` into OWL's
source tree and apply the wiring below.

## 1. Install the detector

```bash
cp detectors/rustspray_detector.py <owl-repo>/owl/detectors/rustspray_detector.py
```

The module depends only on `numpy` and the Python standard library.

## 2. Config schema

Add to OWL's config (YAML shown; TOML equivalent works the same):

```yaml
detector_backend: rustspray   # Options: exg | hsv | exhsv | yolo | rustspray

rustspray:
  binary: /usr/local/bin/rustspray   # or relative path
  config: /etc/rustspray/config.toml
  mock_gpio: false
  frame_timeout_ms: 100
  max_restarts: 3
```

`rustspray.config` must point to a Rust-Spray TOML whose `[gpio] pins` and
`[lanes] count` match the relay wiring OWL would otherwise drive — Rust-Spray
actuates the solenoids itself in IPC mode. Set `mock_gpio: true` if OWL
should keep exclusive control of the relays (Rust-Spray then only reports
lane states).

## 3. Detector factory wiring

Wherever OWL instantiates its detector (e.g. `owl.py` / `greenonbrown.py`):

```python
if cfg.detector_backend == "rustspray":
    from owl.detectors.rustspray_detector import RustSprayDetector
    detector = RustSprayDetector(
        binary_path=cfg.rustspray.binary,
        config_path=cfg.rustspray.config,
        num_lanes=cfg.num_nozzles,
        mock_gpio=cfg.rustspray.mock_gpio,
        frame_timeout_s=cfg.rustspray.frame_timeout_ms / 1000.0,
        max_restarts=cfg.rustspray.max_restarts,
    )
```

`RustSprayDetector.detect(frame)` matches the duck-typed interface of the
existing detectors and returns `(boxes, annotated_frame, lane_states)`:
per-active-lane bounding boxes for the logger/dashboard, the frame with
active lanes outlined, and the per-lane bool states.

**Frames must be RGB.** picamera2's `RGB888` stream is already correct; if
the frame came through OpenCV (BGR), convert with `frame[:, :, ::-1]` first.

## 4. Automatic fallback

The wrapper restarts a crashed or timed-out subprocess up to
`max_restarts` times. After that, `detect()` raises `RuntimeError` — catch
it in the outer loop and swap in the Python ExG detector:

```python
try:
    boxes, annotated, lanes = detector.detect(frame)
except RuntimeError:
    logger.exception("rustspray backend failed — falling back to Python ExG")
    detector.close()
    detector = GreenOnBrown(algorithm="exg", ...)
    boxes, annotated, lanes = detector.detect(frame)
```

Rust-Spray forces all lanes off on every exit path, so a crash never
leaves a valve open.

## 5. Graceful shutdown

Register `detector.close()` so `systemctl stop owl` terminates the
subprocess cleanly (the wrapper also registers an `atexit` hook itself):

```python
import atexit, signal

atexit.register(detector.close)
signal.signal(signal.SIGTERM, lambda *_: (detector.close(), sys.exit(0)))
```

`close()` shuts the subprocess's stdin; the binary sees EOF, drives every
lane off, and exits 0.

## Testing without hardware

```bash
# From the Rust-Spray repo root:
cargo build --release
pip install pytest numpy
pytest tests/test_rustspray_detector.py
```
