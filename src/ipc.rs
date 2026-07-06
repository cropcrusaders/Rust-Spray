//! IPC mode: length-prefixed RGB24 frames in, newline-delimited JSON out.
//!
//! This is the contract used by host processes (e.g. OpenWeedLocator)
//! that own frame capture and supervise `rustspray` as a subprocess.
//! See `INTEGRATION.md` for the full protocol specification.
//!
//! Wire format, protocol v1:
//!
//! ```text
//! stdin  <- [width: u32 LE][height: u32 LE][width * height * 3 bytes RGB24]  (repeated)
//! stdout -> {"v":1,"frame":1,"ts_us":...,"lanes":[...],"latency_us":...}\n   (one per frame)
//! ```
//!
//! GPIO is driven exactly as in standalone mode; stdout carries only the
//! protocol stream (all logging goes to stderr).

use crate::{io_gpio::NozzleControl, lanes::LaneReducer, pipeline::Pipeline, vision::PlantVision};
use crossbeam::channel::{bounded, Receiver, RecvTimeoutError, Sender};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Version of the stdin/stdout IPC contract implemented by this build.
pub const PROTOCOL_VERSION: u32 = 1;

/// Maximum accepted frame width/height — sanity cap so a corrupt header
/// cannot trigger a multi-gigabyte allocation.
pub const MAX_DIMENSION: u32 = 4096;

/// One line of the stdout protocol stream.
#[derive(Debug, Serialize, Deserialize)]
pub struct FrameResponse {
    /// Protocol version ([`PROTOCOL_VERSION`]).
    pub v: u32,
    /// Monotonically increasing frame counter, starting at 1.
    pub frame: u64,
    /// Unix time in microseconds when the frame was fully received.
    pub ts_us: u64,
    /// Per-lane spray states, in lane order (left to right).
    pub lanes: Vec<bool>,
    /// Detection latency for this frame in microseconds.
    pub latency_us: u64,
}

/// Why the IPC loop terminated abnormally.
#[derive(Debug)]
pub enum IpcError {
    /// No frame arrived within the stall timeout — the host is wedged.
    Stalled,
    /// Unrecoverable protocol or I/O failure (details in the message).
    Fatal(String),
}

struct FrameMsg {
    ts_us: u64,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

/// Run the IPC loop until end-of-input, shutdown signal, stall, or error.
///
/// Frames are read on a dedicated thread so the loop stays responsive to
/// `running` (SIGINT/SIGTERM) and can detect a host that stops sending
/// frames without closing the pipe. Frame dimensions come from the frame
/// headers: the first frame fixes them, and every later frame must match.
///
/// All nozzles are switched off before returning, on every path.
/// Returns the number of frames processed on clean end-of-input.
pub fn run<R, W>(
    input: R,
    mut output: W,
    vision: PlantVision,
    reducer: LaneReducer,
    gpio: Box<dyn NozzleControl>,
    running: Arc<AtomicBool>,
    stall_timeout: Duration,
) -> Result<u64, IpcError>
where
    R: Read + Send + 'static,
    W: Write,
{
    let lane_count = reducer.lane_count();
    let (frame_tx, frame_rx) = bounded::<Result<FrameMsg, String>>(1);
    // Recycle payload buffers to keep the hot path allocation-free.
    let (free_tx, free_rx) = bounded::<Vec<u8>>(2);

    std::thread::spawn(move || reader_loop(input, frame_tx, free_rx));

    // Components move into the Pipeline once the first frame fixes the
    // dimensions.
    let mut parts = Some((vision, reducer, gpio));
    let mut pipeline: Option<Pipeline> = None;
    let mut dims: Option<(u32, u32)> = None;

    // Poll in short intervals so SIGINT/SIGTERM stays responsive while
    // waiting for frames.
    let poll = Duration::from_millis(200);
    let mut last_frame = Instant::now();
    let mut count: u64 = 0;

    let result = loop {
        if !running.load(Ordering::SeqCst) {
            break Ok(count);
        }
        match frame_rx.recv_timeout(poll) {
            Ok(Ok(msg)) => {
                match dims {
                    None => {
                        if (msg.width as usize) < lane_count {
                            break Err(IpcError::Fatal(format!(
                                "frame width {} is smaller than the configured lane count {}",
                                msg.width, lane_count,
                            )));
                        }
                        dims = Some((msg.width, msg.height));
                        let (vision, reducer, gpio) = parts.take().expect("parts taken once");
                        pipeline = Some(Pipeline::new(
                            reducer,
                            gpio,
                            vision,
                            msg.width as usize,
                            msg.height as usize,
                        ));
                    }
                    Some(d) if d != (msg.width, msg.height) => {
                        break Err(IpcError::Fatal(format!(
                            "frame dimensions changed mid-stream: {}x{} -> {}x{}",
                            d.0, d.1, msg.width, msg.height,
                        )));
                    }
                    _ => {}
                }

                let start = Instant::now();
                let lanes = pipeline
                    .as_mut()
                    .expect("pipeline built on first frame")
                    .process(&msg.data);
                let latency_us = start.elapsed().as_micros() as u64;
                let _ = free_tx.try_send(msg.data);
                count += 1;
                last_frame = Instant::now();

                let response = FrameResponse {
                    v: PROTOCOL_VERSION,
                    frame: count,
                    ts_us: msg.ts_us,
                    lanes,
                    latency_us,
                };
                if let Err(e) = write_response(&mut output, &response) {
                    break Err(IpcError::Fatal(format!("failed to write response: {e}")));
                }
            }
            Ok(Err(e)) => break Err(IpcError::Fatal(e)),
            Err(RecvTimeoutError::Timeout) => {
                if !stall_timeout.is_zero() && last_frame.elapsed() >= stall_timeout {
                    break Err(IpcError::Stalled);
                }
            }
            Err(RecvTimeoutError::Disconnected) => break Ok(count),
        }
    };

    // Fail safe: never leave a valve open, on any exit path.
    if let Some(p) = pipeline.as_mut() {
        p.all_off();
    } else if let Some((_, _, mut gpio)) = parts.take() {
        gpio.apply(&vec![false; lane_count]);
    }
    result
}

fn reader_loop<R: Read>(
    mut input: R,
    tx: Sender<Result<FrameMsg, String>>,
    free: Receiver<Vec<u8>>,
) {
    loop {
        let (width, height) = match read_header(&mut input) {
            Ok(Some(dims)) => dims,
            Ok(None) => return, // clean end-of-input
            Err(e) => {
                let _ = tx.send(Err(e));
                return;
            }
        };
        if let Err(e) = validate_dims(width, height) {
            let _ = tx.send(Err(e));
            return;
        }
        let len = width as usize * height as usize * 3;
        let mut buf = free.try_recv().unwrap_or_default();
        buf.resize(len, 0);
        if let Err(e) = input.read_exact(&mut buf) {
            let _ = tx.send(Err(format!("truncated frame payload: {e}")));
            return;
        }
        let ts_us = unix_micros();
        if tx
            .send(Ok(FrameMsg {
                ts_us,
                width,
                height,
                data: buf,
            }))
            .is_err()
        {
            return; // main loop is gone
        }
    }
}

/// Read one 8-byte frame header.
///
/// Returns `Ok(None)` on clean end-of-input (EOF at a frame boundary);
/// EOF partway through a header is an error.
fn read_header<R: Read>(input: &mut R) -> Result<Option<(u32, u32)>, String> {
    let mut header = [0u8; 8];
    let mut filled = 0;
    while filled < header.len() {
        match input.read(&mut header[filled..]) {
            Ok(0) => {
                return if filled == 0 {
                    Ok(None)
                } else {
                    Err(format!("truncated frame header ({filled}/8 bytes)"))
                };
            }
            Ok(n) => filled += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(format!("header read error: {e}")),
        }
    }
    let width = u32::from_le_bytes(header[0..4].try_into().unwrap());
    let height = u32::from_le_bytes(header[4..8].try_into().unwrap());
    Ok(Some((width, height)))
}

fn validate_dims(width: u32, height: u32) -> Result<(), String> {
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(format!(
            "invalid frame dimensions {width}x{height} (must be 1..={MAX_DIMENSION})",
        ));
    }
    Ok(())
}

fn write_response<W: Write>(output: &mut W, response: &FrameResponse) -> std::io::Result<()> {
    serde_json::to_writer(&mut *output, response).map_err(std::io::Error::from)?;
    output.write_all(b"\n")?;
    output.flush()
}

fn unix_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io_gpio::MockGpio;
    use std::io::Cursor;

    const W: u32 = 8;
    const H: u32 = 2;

    /// Encode one frame: header + pixels, left half green, right half soil.
    fn frame_bytes(width: u32, height: u32) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&width.to_le_bytes());
        out.extend_from_slice(&height.to_le_bytes());
        for _y in 0..height {
            for x in 0..width {
                if x < width / 2 {
                    out.extend_from_slice(&[20, 200, 20]);
                } else {
                    out.extend_from_slice(&[120, 90, 70]);
                }
            }
        }
        out
    }

    fn run_on(input: Vec<u8>) -> (Result<u64, IpcError>, Vec<String>) {
        let mut output = Vec::new();
        let result = run(
            Cursor::new(input),
            &mut output,
            PlantVision::default(),
            LaneReducer::new(2, 0.3, 0.15),
            Box::new(MockGpio::default()),
            Arc::new(AtomicBool::new(true)),
            Duration::ZERO,
        );
        let lines = String::from_utf8(output)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect();
        (result, lines)
    }

    #[test]
    fn processes_frames_and_emits_json() {
        let mut input = frame_bytes(W, H);
        input.extend_from_slice(&frame_bytes(W, H));
        let (result, lines) = run_on(input);
        assert_eq!(result.unwrap(), 2);
        assert_eq!(lines.len(), 2);
        for (i, line) in lines.iter().enumerate() {
            let resp: FrameResponse = serde_json::from_str(line).unwrap();
            assert_eq!(resp.v, PROTOCOL_VERSION);
            assert_eq!(resp.frame, i as u64 + 1);
            assert_eq!(resp.lanes, vec![true, false]);
            assert!(resp.ts_us > 1_600_000_000_000_000, "ts_us not unix micros");
        }
    }

    #[test]
    fn empty_input_is_clean_eof() {
        let (result, lines) = run_on(Vec::new());
        assert_eq!(result.unwrap(), 0);
        assert!(lines.is_empty());
    }

    #[test]
    fn truncated_header_is_fatal() {
        let (result, _) = run_on(vec![1, 2, 3]);
        assert!(matches!(result, Err(IpcError::Fatal(_))));
    }

    #[test]
    fn truncated_payload_is_fatal() {
        let mut input = frame_bytes(W, H);
        input.truncate(input.len() - 1);
        let (result, _) = run_on(input);
        assert!(matches!(result, Err(IpcError::Fatal(_))));
    }

    #[test]
    fn dimension_change_is_fatal() {
        let mut input = frame_bytes(W, H);
        input.extend_from_slice(&frame_bytes(W * 2, H));
        let (result, lines) = run_on(input);
        assert!(matches!(result, Err(IpcError::Fatal(_))));
        assert_eq!(lines.len(), 1, "first frame should still be answered");
    }

    #[test]
    fn oversized_dimensions_are_fatal() {
        let mut input = Vec::new();
        input.extend_from_slice(&(MAX_DIMENSION + 1).to_le_bytes());
        input.extend_from_slice(&H.to_le_bytes());
        let (result, _) = run_on(input);
        assert!(matches!(result, Err(IpcError::Fatal(_))));
    }
}
