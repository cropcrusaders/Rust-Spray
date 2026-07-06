"""OpenWeedLocator (OWL) detector backend backed by the Rust-Spray binary.

Drop this file into OWL's source tree (e.g. ``owl/detectors/``) and wire it
into the detector factory as described in ``owl/README.md`` in the
Rust-Spray repository. The IPC contract is documented in ``INTEGRATION.md``.
"""

from __future__ import annotations

import atexit
import json
import logging
import os
import queue
import struct
import subprocess
import threading
from collections import deque

import numpy as np

logger = logging.getLogger(__name__)

# Sentinel placed on the reader queue when the subprocess closes stdout.
_EOF = object()


class RustSprayDetector:
    """
    Wraps the Rust-Spray binary as a high-performance inner loop.

    IPC protocol v1:
      stdin  <- [u32 width LE][u32 height LE][width*height*3 bytes RGB24]
      stdout -> {"v":1,"frame":N,"ts_us":T,"lanes":[bool,...],"latency_us":L}\\n

    The subprocess drives GPIO itself (unless ``mock_gpio``), so the lane
    states returned here are for OWL's logging/dashboard and any additional
    actuation OWL performs. Frames must be RGB (not BGR), uint8, HxWx3.

    Failure policy: if the subprocess dies or a frame times out, it is
    restarted transparently up to ``MAX_RESTARTS`` times over the detector's
    lifetime. Once the budget is exhausted, :meth:`detect` raises
    ``RuntimeError`` so OWL's outer loop can fall back to its Python ExG
    detector.
    """

    PROTOCOL_VERSION = 1
    STARTUP_TIMEOUT_S = 5.0
    FRAME_TIMEOUT_S = 0.10  # 100 ms — safe at up to 30 km/h
    MAX_RESTARTS = 3

    def __init__(
        self,
        binary_path: str,
        config_path: str,
        num_lanes: int = 4,
        mock_gpio: bool = False,
        *,
        frame_timeout_s: float | None = None,
        max_restarts: int | None = None,
    ):
        self.binary_path = binary_path
        self.config_path = config_path
        self.num_lanes = num_lanes
        self.mock_gpio = mock_gpio
        self.frame_timeout_s = (
            self.FRAME_TIMEOUT_S if frame_timeout_s is None else frame_timeout_s
        )
        self.max_restarts = self.MAX_RESTARTS if max_restarts is None else max_restarts

        self._proc: subprocess.Popen | None = None
        self._stdout_queue: queue.Queue = queue.Queue()
        self._stderr_tail: deque[str] = deque(maxlen=20)
        self._restarts = 0
        self._lock = threading.Lock()
        self._closed = False

        if not os.path.isfile(self.binary_path):
            raise RuntimeError(f"rustspray binary not found: {self.binary_path}")

        self._verify_protocol_version()
        self._start_process()
        atexit.register(self.close)

    # ------------------------------------------------------------------
    # Public detector interface (duck-typed to match OWL's detectors)
    # ------------------------------------------------------------------

    def detect(
        self,
        frame: np.ndarray,
        confidence: float = 0.5,
        filter_id: int = 0,
        **kwargs,
    ) -> tuple[list, np.ndarray, list]:
        """
        Drop-in replacement for GreenOnBrown.detect().

        Returns ``(boxes, annotated_frame, lane_states)``:

        - ``boxes`` — one ``(x, y, w, h)`` box per **active** lane, covering
          that lane's vertical strip, so OWL's logger/dashboard have a
          region to display.
        - ``annotated_frame`` — the input frame with active lane strips
          outlined in green.
        - ``lane_states`` — list of bool, one per spray lane, in lane order.

        ``confidence`` and ``filter_id`` are accepted for interface
        compatibility; thresholds live in Rust-Spray's TOML config.
        """
        if frame.ndim != 3 or frame.shape[2] != 3 or frame.dtype != np.uint8:
            raise ValueError(
                f"expected HxWx3 uint8 RGB frame, got shape {frame.shape} dtype {frame.dtype}"
            )
        height, width = frame.shape[:2]
        payload = np.ascontiguousarray(frame).tobytes()

        with self._lock:
            response = self._send_frame_with_restart(payload, width, height)

        lane_states = list(response["lanes"])[: self.num_lanes]
        boxes = self._lane_boxes(lane_states, width, height)
        annotated = self._annotate(frame, boxes)
        return boxes, annotated, lane_states

    def close(self) -> None:
        """Terminate subprocess cleanly. Safe to call more than once."""
        self._closed = True
        proc = self._proc
        self._proc = None
        if proc is None or proc.poll() is not None:
            return
        try:
            # Closing stdin is the protocol's clean-shutdown signal: the
            # binary sees EOF, forces all lanes off, and exits 0.
            if proc.stdin:
                proc.stdin.close()
            proc.wait(timeout=2.0)
        except (OSError, subprocess.TimeoutExpired):
            proc.terminate()
            try:
                proc.wait(timeout=2.0)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait()
        logger.info("rustspray subprocess stopped")

    # ------------------------------------------------------------------
    # Subprocess management
    # ------------------------------------------------------------------

    def _verify_protocol_version(self) -> None:
        """Startup handshake: refuse to run against an incompatible binary."""
        try:
            out = subprocess.run(
                [self.binary_path, "--output-version"],
                capture_output=True,
                timeout=self.STARTUP_TIMEOUT_S,
                check=True,
            ).stdout
            info = json.loads(out)
        except (OSError, subprocess.SubprocessError, ValueError) as exc:
            raise RuntimeError(f"rustspray --output-version failed: {exc}") from exc

        protocol = info.get("ipc_protocol")
        if protocol != self.PROTOCOL_VERSION:
            raise RuntimeError(
                f"rustspray IPC protocol mismatch: binary speaks v{protocol}, "
                f"this detector requires v{self.PROTOCOL_VERSION} "
                f"(binary version {info.get('rustspray_version')})"
            )
        logger.info(
            "rustspray %s (IPC protocol v%s) at %s",
            info.get("rustspray_version"),
            protocol,
            self.binary_path,
        )

    def _start_process(self) -> None:
        """Spawn rustspray subprocess. Called at init and on restart."""
        cmd = [self.binary_path, "--ipc-mode", "--config", self.config_path]
        if self.mock_gpio:
            cmd.append("--mock-gpio")
        self._proc = subprocess.Popen(
            cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            bufsize=0,
        )
        # Fresh queue so stale responses from a previous incarnation can
        # never be matched against new frames.
        self._stdout_queue = queue.Queue()
        threading.Thread(
            target=self._stdout_reader, args=(self._proc, self._stdout_queue), daemon=True
        ).start()
        threading.Thread(
            target=self._stderr_reader, args=(self._proc,), daemon=True
        ).start()
        logger.info("rustspray subprocess started (pid %d)", self._proc.pid)

    @staticmethod
    def _stdout_reader(proc: subprocess.Popen, out_queue: queue.Queue) -> None:
        """Daemon thread: move JSON lines from the pipe onto a queue so the
        caller can wait with a timeout instead of blocking OWL's loop."""
        try:
            for line in proc.stdout:
                out_queue.put(line)
        except (OSError, ValueError):
            pass
        out_queue.put(_EOF)

    def _stderr_reader(self, proc: subprocess.Popen) -> None:
        """Daemon thread: drain stderr (logs / [MOCK GPIO] lines) so the
        subprocess can never block on a full pipe."""
        try:
            for line in proc.stderr:
                text = line.decode(errors="replace").rstrip()
                if text:
                    self._stderr_tail.append(text)
                    logger.debug("rustspray: %s", text)
        except (OSError, ValueError):
            pass

    def _health_check(self) -> bool:
        """Return True if subprocess is alive and IPC protocol version matches.

        The version was pinned by the startup handshake; a running process
        is by construction speaking the verified protocol.
        """
        return self._proc is not None and self._proc.poll() is None

    # ------------------------------------------------------------------
    # IPC
    # ------------------------------------------------------------------

    def _send_frame(self, frame_rgb24: bytes, width: int, height: int) -> dict:
        """Write frame to stdin, read JSON response from stdout."""
        if not self._health_check():
            raise BrokenPipeError("rustspray subprocess is not running")

        header = struct.pack("<II", width, height)
        # Single write so the header and pixels can never be interleaved
        # with anything else or torn by a crash between two writes.
        self._proc.stdin.write(header + frame_rgb24)
        self._proc.stdin.flush()

        try:
            line = self._stdout_queue.get(timeout=self.frame_timeout_s)
        except queue.Empty:
            raise TimeoutError(
                f"no response from rustspray within {self.frame_timeout_s * 1000:.0f} ms"
            ) from None
        if line is _EOF:
            raise BrokenPipeError("rustspray closed its stdout")

        response = json.loads(line)
        if response.get("v") != self.PROTOCOL_VERSION:
            raise RuntimeError(
                f"rustspray response protocol v{response.get('v')} != "
                f"expected v{self.PROTOCOL_VERSION}"
            )
        return response

    def _send_frame_with_restart(
        self, frame_rgb24: bytes, width: int, height: int
    ) -> dict:
        """Send a frame, restarting the subprocess on failure until the
        restart budget is exhausted, then raise RuntimeError so the caller
        can fall back to a Python detector."""
        while True:
            try:
                return self._send_frame(frame_rgb24, width, height)
            except (OSError, TimeoutError, ValueError, RuntimeError) as exc:
                self._handle_failure(exc)

    def _handle_failure(self, exc: Exception) -> None:
        if self._closed:
            raise RuntimeError("rustspray detector is closed") from exc

        proc, self._proc = self._proc, None
        exit_code = None
        if proc is not None:
            proc.kill()
            exit_code = proc.wait()

        stderr_tail = "\n".join(self._stderr_tail)
        logger.error(
            "rustspray failure (%s; exit code %s); restarts used %d/%d\n%s",
            exc,
            exit_code,
            self._restarts,
            self.max_restarts,
            stderr_tail,
        )

        if self._restarts >= self.max_restarts:
            raise RuntimeError(
                f"rustspray failed after {self._restarts} restarts: {exc}; "
                "falling back to the Python detector is recommended"
            ) from exc
        self._restarts += 1
        logger.warning(
            "restarting rustspray (attempt %d/%d)", self._restarts, self.max_restarts
        )
        self._start_process()

    # ------------------------------------------------------------------
    # Presentation helpers
    # ------------------------------------------------------------------

    def _lane_boxes(
        self, lane_states: list[bool], width: int, height: int
    ) -> list[tuple[int, int, int, int]]:
        """One (x, y, w, h) box per active lane, matching Rust-Spray's lane
        geometry: base width ``width // lanes`` with the first
        ``width % lanes`` lanes one pixel wider."""
        n = len(lane_states)
        if n == 0:
            return []
        base, remainder = divmod(width, n)
        boxes = []
        x = 0
        for lane, active in enumerate(lane_states):
            lane_w = base + (1 if lane < remainder else 0)
            if active:
                boxes.append((x, 0, lane_w, height))
            x += lane_w
        return boxes

    @staticmethod
    def _annotate(
        frame: np.ndarray, boxes: list[tuple[int, int, int, int]]
    ) -> np.ndarray:
        """Outline active lane strips in green (no OpenCV dependency)."""
        if not boxes:
            return frame
        annotated = frame.copy()
        green = (0, 255, 0)
        for x, y, w, h in boxes:
            annotated[y : y + 2, x : x + w] = green
            annotated[y + h - 2 : y + h, x : x + w] = green
            annotated[y : y + h, x : x + 2] = green
            annotated[y : y + h, x + w - 2 : x + w] = green
        return annotated
