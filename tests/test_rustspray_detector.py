"""Integration tests for the OWL RustSprayDetector wrapper.

Requires the release binary: ``cargo build --release`` (or set
``RUSTSPRAY_BIN`` to a rustspray binary built with IPC support).
"""

import json
import os
import signal
import struct
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
import pytest

REPO_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO_ROOT / "owl"))

from detectors.rustspray_detector import RustSprayDetector  # noqa: E402

BINARY = os.environ.get("RUSTSPRAY_BIN", str(REPO_ROOT / "target" / "release" / "rustspray"))
CONFIG = str(REPO_ROOT / "config" / "rustspray.toml")

pytestmark = pytest.mark.skipif(
    not os.path.isfile(BINARY),
    reason=f"rustspray binary not found at {BINARY}; run `cargo build --release`",
)

WIDTH, HEIGHT = 64, 16
GREEN = (40, 210, 40)
SOIL = (120, 90, 70)


def synthetic_frame(green_lanes, width=WIDTH, height=HEIGHT, lanes=4):
    """RGB24 frame with the given lanes painted green, soil elsewhere."""
    frame = np.empty((height, width, 3), dtype=np.uint8)
    frame[:] = SOIL
    lane_w = width // lanes
    for lane in green_lanes:
        frame[:, lane * lane_w : (lane + 1) * lane_w] = GREEN
    return frame


@pytest.fixture
def detector():
    det = RustSprayDetector(BINARY, CONFIG, num_lanes=4, mock_gpio=True)
    yield det
    det.close()


class TestProtocolHandshake:
    def test_output_version_reports_protocol(self):
        out = subprocess.run(
            [BINARY, "--output-version"], capture_output=True, timeout=10, check=True
        ).stdout
        info = json.loads(out)
        assert info["ipc_protocol"] == RustSprayDetector.PROTOCOL_VERSION
        assert "rustspray_version" in info

    def test_version_mismatch_raises(self):
        class FutureDetector(RustSprayDetector):
            PROTOCOL_VERSION = 2

        with pytest.raises(RuntimeError, match="protocol mismatch"):
            FutureDetector(BINARY, CONFIG, num_lanes=4, mock_gpio=True)

    def test_missing_binary_raises(self):
        with pytest.raises(RuntimeError, match="not found"):
            RustSprayDetector("/nonexistent/rustspray", CONFIG, mock_gpio=True)


class TestDetection:
    def test_green_lanes_activate(self, detector):
        boxes, annotated, lanes = detector.detect(synthetic_frame({0, 2}))
        assert lanes == [True, False, True, False]
        assert len(boxes) == 2
        assert annotated.shape == (HEIGHT, WIDTH, 3)

    def test_clean_frame_no_lanes(self, detector):
        boxes, annotated, lanes = detector.detect(synthetic_frame(set()))
        assert lanes == [False, False, False, False]
        assert boxes == []
        # No annotation on an all-off frame.
        assert np.array_equal(annotated, synthetic_frame(set()))

    def test_lane_boxes_cover_lane_strips(self, detector):
        boxes, _, _ = detector.detect(synthetic_frame({1}))
        assert boxes == [(16, 0, 16, HEIGHT)]

    def test_hysteresis_keeps_lane_on_across_frames(self, detector):
        # Full green -> on; then a lane at ~25% coverage (between the 15%
        # off- and 30% on-thresholds) stays on only because of hysteresis.
        assert detector.detect(synthetic_frame({0}))[2][0] is True
        partial = synthetic_frame(set())
        partial[:, 0:4] = GREEN  # 4 of lane 0's 16 columns = 25%
        assert detector.detect(partial)[2][0] is True

    def test_frame_counter_is_monotonic(self, detector):
        frame = synthetic_frame(set())
        first = detector._send_frame(frame.tobytes(), WIDTH, HEIGHT)
        second = detector._send_frame(frame.tobytes(), WIDTH, HEIGHT)
        assert second["frame"] == first["frame"] + 1
        assert second["ts_us"] >= first["ts_us"]

    def test_rejects_non_rgb24_input(self, detector):
        with pytest.raises(ValueError):
            detector.detect(np.zeros((HEIGHT, WIDTH), dtype=np.uint8))
        with pytest.raises(ValueError):
            detector.detect(np.zeros((HEIGHT, WIDTH, 3), dtype=np.float32))


class TestRestartLogic:
    def test_restarts_after_subprocess_killed(self, detector):
        frame = synthetic_frame({3})
        assert detector.detect(frame)[2] == [False, False, False, True]

        detector._proc.kill()
        detector._proc.wait()

        # The next detect() must transparently restart and still answer.
        assert detector.detect(frame)[2] == [False, False, False, True]
        assert detector._restarts == 1

    def test_gives_up_after_max_restarts(self):
        det = RustSprayDetector(
            BINARY, CONFIG, num_lanes=4, mock_gpio=True, max_restarts=2
        )
        try:
            frame = synthetic_frame(set())
            det.detect(frame)  # healthy first

            # Sabotage every restart by killing the process as soon as a
            # frame is sent: point the detector at a stream that has
            # already ended.
            original_start = det._start_process

            def broken_start():
                original_start()
                det._proc.stdin.close()  # binary sees EOF and exits

            det._start_process = broken_start
            det._proc.kill()
            det._proc.wait()

            with pytest.raises(RuntimeError, match="after 2 restarts"):
                det.detect(frame)
        finally:
            det.close()

    def test_timeout_triggers_restart(self):
        det = RustSprayDetector(
            BINARY, CONFIG, num_lanes=4, mock_gpio=True, frame_timeout_s=0.05
        )
        try:
            frame = synthetic_frame(set())
            det.detect(frame)  # healthy first

            # Freeze the subprocess so the next frame is guaranteed to time
            # out; the wrapper must restart (SIGKILL works on a stopped
            # process) and answer from the fresh instance.
            os.kill(det._proc.pid, signal.SIGSTOP)
            assert det.detect(frame)[2] == [False, False, False, False]
            assert det._restarts == 1
        finally:
            det.close()


class TestShutdown:
    def test_close_terminates_subprocess(self, detector):
        proc = detector._proc
        detector.detect(synthetic_frame(set()))
        detector.close()
        assert proc.poll() is not None
        # Clean EOF shutdown exits 0.
        assert proc.returncode == 0

    def test_close_is_idempotent(self, detector):
        detector.close()
        detector.close()


class TestWireFormat:
    """Pin the byte-level protocol independently of the wrapper, so a
    manufacturer reimplementing either side can use this as a reference."""

    def test_raw_ipc_round_trip(self):
        frame = synthetic_frame({1})
        header = struct.pack("<II", WIDTH, HEIGHT)
        proc = subprocess.run(
            [BINARY, "--ipc-mode", "--mock-gpio", "--config", CONFIG],
            input=header + frame.tobytes(),
            capture_output=True,
            timeout=10,
        )
        assert proc.returncode == 0
        lines = proc.stdout.decode().splitlines()
        assert len(lines) == 1
        response = json.loads(lines[0])
        assert response["v"] == 1
        assert response["frame"] == 0
        assert response["lanes"] == [False, True, False, False]
        assert response["latency_us"] >= 0
        assert abs(response["ts_us"] / 1e6 - time.time()) < 60
        assert "[MOCK GPIO] lane=1 state=ON" in proc.stderr.decode()

    def test_truncated_frame_fails_safe(self):
        header = struct.pack("<II", WIDTH, HEIGHT)
        proc = subprocess.run(
            [BINARY, "--ipc-mode", "--mock-gpio", "--config", CONFIG],
            input=header + b"\x00" * 10,  # far short of WIDTH*HEIGHT*3
            capture_output=True,
            timeout=10,
        )
        assert proc.returncode == 2
        assert proc.stdout == b""
