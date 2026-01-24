# LaneRustSpray Spec

## Goal

Use Lane AI (row/lane geometry + position locking) to drive a targeted rust spray workflow:

1. Lock the lane (row centerline + vehicle offset).
2. Scan the canopy with RGB (optionally multispectral).
3. Detect rust lesions in-lane only (ignore out-of-lane noise).
4. Convert pixel hits → ground hits (distance + camera model).
5. Schedule nozzle fire using forward speed + boom offset.
6. Log everything for proof + map layers + ROI.

## System Assumptions

- **Platform:** tractor/UTV/spray rig or drone (ground rig easiest first).
- **Sensors:**
  - Front camera (RGB) for lane + rust.
  - Speed source (wheel, GPS, CAN).
  - Optional depth (stereo / ToF) OR calibrated fixed height geometry.
- **Actuation:**
  - Boom with individually addressable nozzles (PWM or solenoid).
  - Known nozzle spacing + boom offset from camera.

## Config Files

### `config/rig.yaml`

```yaml
platform: "ground_rig"
camera_to_boom_m: 2.40        # camera forward of boom
camera_height_m: 1.80
camera_pitch_deg: 18
boom_width_m: 12.0
nozzle_spacing_m: 0.50
max_nozzles: 24
min_speed_mps: 1.0
max_speed_mps: 8.0
```

### `config/nozzles.yaml`

```yaml
nozzles:
  - id: 0
    x_m: -5.75
  - id: 1
    x_m: -5.25
  # ...
  - id: 23
    x_m: 5.75
valve:
  type: "pwm"
  hz: 20
  min_duty: 0.10
  max_duty: 0.70
```

## Core Runtime Loop (Codex Spec)

```python
"""
LaneRustSpray runtime:
- LaneAI gives row centerline + lane polygon in image space.
- RustAI detects rust blobs (boxes/masks) in image space.
- We filter detections to lane polygon, project to ground x-offset,
  assign nozzle, compute time-to-fire, schedule pulses.
"""

from lane_ai.lane_model import LaneModel
from lane_ai.lane_tracker import LaneTracker
from rust_ai.rust_model import RustModel
from rust_ai.rust_post import rust_to_targets
from fusion.geo import pixel_to_ground_x
from fusion.timing import compute_fire_time
from control.spray_scheduler import SprayScheduler
from control.safety import SafetyGate
from control.nozzle_driver import NozzleDriver

def main():
    lane_model = LaneModel(weights="weights/lane.pt")
    lane_tracker = LaneTracker()
    rust_model = RustModel(weights="weights/rust.pt")

    nozzle_driver = NozzleDriver(port="/dev/ttyUSB0")
    scheduler = SprayScheduler(nozzle_driver)
    safety = SafetyGate()

    cam = open_camera(index=0, width=1280, height=720, fps=30)
    cfg = load_all_configs()

    while True:
        frame, ts = cam.read()

        # 1) Lane inference + tracking
        lane_pred = lane_model.infer(frame)
        lane_state = lane_tracker.update(lane_pred, ts)
        lane_poly = lane_state.lane_polygon  # image-space polygon mask/vertices
        lane_offset_m = lane_state.lane_offset_m  # rig lateral offset estimate

        # 2) Rust inference
        rust_pred = rust_model.infer(frame)

        # 3) Convert rust detections into actionable targets
        targets = rust_to_targets(
            rust_pred=rust_pred,
            lane_polygon=lane_poly,
            min_conf=0.45,
            min_area_px=120
        )
        # each target: {cx_px, cy_px, conf, area_px}

        # 4) Safety + speed gating
        speed_mps = read_speed_mps()
        if not safety.ok(speed_mps=speed_mps, lane_lock=lane_state.locked):
            scheduler.flush(ts)
            continue

        # 5) For each target, project to ground and schedule nozzle pulse
        for t in targets:
            x_m = pixel_to_ground_x(
                cx_px=t["cx_px"],
                cy_px=t["cy_px"],
                camera_cfg=cfg.camera,
                rig_cfg=cfg.rig
            )

            # correct with lane offset (so nozzle aims relative to lane center)
            x_m_corrected = x_m - lane_offset_m

            nozzle_id = select_nozzle_id(x_m_corrected, cfg.nozzles)
            if nozzle_id is None:
                continue

            fire_ts = compute_fire_time(
                now_ts=ts,
                speed_mps=speed_mps,
                camera_to_boom_m=cfg.rig.camera_to_boom_m
            )

            pulse_ms = dose_to_pulse_ms(
                conf=t["conf"],
                area_px=t["area_px"],
                base_ms=65,
                max_ms=180
            )

            scheduler.schedule(
                nozzle_id=nozzle_id,
                fire_ts=fire_ts,
                pulse_ms=pulse_ms,
                meta={"conf": t["conf"], "area_px": t["area_px"]}
            )

        # 6) Execute due pulses + housekeeping
        scheduler.tick(ts)
        log_frame(ts, lane_state, targets)

if __name__ == "__main__":
    main()
```

## Rust Target Extraction (Lane-Filtered)

```python
def rust_to_targets(rust_pred, lane_polygon, min_conf=0.45, min_area_px=120):
    """
    rust_pred could be:
      - boxes: [x1,y1,x2,y2,conf]
      - or masks with conf per instance
    lane_polygon: polygon vertices or binary mask
    Returns point targets for spraying.
    """
    targets = []
    for det in rust_pred.detections:
        if det.conf < min_conf:
            continue

        cx, cy = det.center()
        if not point_in_polygon((cx, cy), lane_polygon):
            continue

        if det.area_px < min_area_px:
            continue

        targets.append({"cx_px": cx, "cy_px": cy, "conf": det.conf, "area_px": det.area_px})
    return targets
```

## Pixel → Ground X Projection (Simple Calibrated Geometry)

Start with a practical “good enough” transform for a fixed camera height/pitch, then upgrade later to stereo/depth.

```python
import math

def pixel_to_ground_x(cx_px, cy_px, camera_cfg, rig_cfg):
    """
    Returns lateral ground offset (meters) relative to camera optical centerline.
    camera_cfg: fx, fy, cx0, cy0 intrinsics
    rig_cfg: camera_height_m, camera_pitch_deg
    """
    fx = camera_cfg.fx
    cx0 = camera_cfg.cx
    h  = rig_cfg.camera_height_m
    pitch = math.radians(rig_cfg.camera_pitch_deg)

    # normalized image coords
    x_n = (cx_px - cx0) / fx
    y_n = (cy_px - camera_cfg.cy) / camera_cfg.fy

    # crude flat-ground intersection:
    # ray in camera coords -> rotate by pitch -> intersect z=0 ground plane
    # This is intentionally minimal; refine once you’ve got real calibration data.
    ray_cam = normalize([x_n, y_n, 1.0])
    ray_world = rotate_x(ray_cam, pitch)

    # camera at (0,0,h), ground at z=0
    t = h / max(1e-6, -ray_world[2])
    x_ground = ray_world[0] * t
    return x_ground
```

## Timing (Hit It When Boom Arrives Over the Target)

```python
def compute_fire_time(now_ts, speed_mps, camera_to_boom_m):
    """
    When should we fire so the nozzle hits the target detected at the camera?
    """
    dt = camera_to_boom_m / max(speed_mps, 0.01)
    return now_ts + dt
```

## Safety Gate (Don’t Spray When It’s Dumb)

```python
class SafetyGate:
    def ok(self, speed_mps, lane_lock):
        if not lane_lock:
            return False
        if speed_mps < 1.0:
            return False
        if speed_mps > 8.0:
            return False
        return True
```

## What Puts Lane AI into Action

Lane AI is the spatial reference frame:

- **Lane polygon** = “only spray in here.”
- **Lane offset estimate** = “shift nozzle assignment left/right so you stay on row.”
- **Lane tracking** = “don’t spray if lock is lost” (dust, sun glare, gaps).

That bridge turns AI detection into real actuation.

## Next Upgrade Steps

1. Rust severity scoring (lesion % area → variable rate pulse).
2. Multi-hit clustering (spray once per cluster, not 40 tiny pulses).
3. Add multispectral feature (rust vs nutrient vs dust discrimination).
4. Create a `RustHeatMap.tif` from logged targets (proof + ROI + planning).
5. Edge deployment on Jetson/Orin + TensorRT.
