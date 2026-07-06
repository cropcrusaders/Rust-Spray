//! IPC protocol v1 for embedding Rust-Spray as an inner loop.
//!
//! An outer shell (e.g. OpenWeedLocator's Python process) owns the camera
//! and pipes frames to the `rustspray` binary running with `--ipc-mode`:
//!
//! * **stdin** — a stream of framed RGB24 images. Each frame is an 8-byte
//!   little-endian header `[width: u32][height: u32]` followed immediately
//!   by `width * height * 3` bytes of interleaved RGB pixel data.
//! * **stdout** — one newline-delimited JSON object per processed frame
//!   (see [`IpcResponse`]). Nothing else is ever written to stdout in IPC
//!   mode; logs and mock-GPIO output go to stderr.
//!
//! The full contract (versioning, error behaviour, handshake) is documented
//! in `INTEGRATION.md` at the repository root.

use serde::Serialize;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

/// Version of the stdin/stdout IPC protocol implemented by this build.
///
/// Bump this whenever the frame header layout or the response JSON schema
/// changes incompatibly, and record the change in `INTEGRATION.md`.
pub const IPC_PROTOCOL_VERSION: u32 = 1;

/// Size of the per-frame header: `[width: u32 LE][height: u32 LE]`.
pub const FRAME_HEADER_BYTES: usize = 8;

/// Upper bound on a single frame's pixel payload (64 MiB, ~22 megapixels).
/// A header requesting more than this is treated as a corrupt stream.
pub const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;

/// Per-frame result written to stdout as one JSON line.
#[derive(Debug, Serialize)]
pub struct IpcResponse {
    /// Protocol version ([`IPC_PROTOCOL_VERSION`]).
    pub v: u32,
    /// Monotonically increasing frame counter, starting at 0.
    pub frame: u64,
    /// Unix time in microseconds at frame receipt.
    pub ts_us: u64,
    /// Lane activation states, one per configured spray lane, in lane order.
    pub lanes: Vec<bool>,
    /// Detection latency for this frame in microseconds.
    pub latency_us: u64,
}

/// Frame dimensions decoded from a header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    pub width: u32,
    pub height: u32,
}

impl FrameHeader {
    /// Pixel payload size in bytes (`width * height * 3`).
    ///
    /// Returns an error if the dimensions are zero, overflow, or exceed
    /// [`MAX_FRAME_BYTES`].
    pub fn payload_len(&self) -> Result<usize, String> {
        if self.width == 0 || self.height == 0 {
            return Err(format!(
                "invalid frame header: dimensions {}x{} must be non-zero",
                self.width, self.height,
            ));
        }
        let len = (self.width as usize)
            .checked_mul(self.height as usize)
            .and_then(|px| px.checked_mul(3))
            .ok_or_else(|| {
                format!(
                    "invalid frame header: {}x{} overflows the frame size",
                    self.width, self.height,
                )
            })?;
        if len > MAX_FRAME_BYTES {
            return Err(format!(
                "invalid frame header: {}x{} ({} bytes) exceeds the {} byte limit",
                self.width, self.height, len, MAX_FRAME_BYTES,
            ));
        }
        Ok(len)
    }
}

/// Read one framed RGB24 image from `reader` into `buf`.
///
/// Returns `Ok(None)` on a clean end of stream (EOF exactly at a frame
/// boundary — the normal way for the outer shell to shut us down). A
/// truncated header or payload, or a header describing an impossible
/// frame, is an error: the stream is unrecoverably out of sync and the
/// caller must fail safe.
pub fn read_frame<R: Read>(
    reader: &mut R,
    buf: &mut Vec<u8>,
) -> std::io::Result<Option<FrameHeader>> {
    let mut header = [0u8; FRAME_HEADER_BYTES];
    let mut filled = 0;
    while filled < FRAME_HEADER_BYTES {
        let n = reader.read(&mut header[filled..])?;
        if n == 0 {
            if filled == 0 {
                return Ok(None); // clean EOF at frame boundary
            }
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("truncated frame header ({filled} of {FRAME_HEADER_BYTES} bytes)"),
            ));
        }
        filled += n;
    }

    let hdr = FrameHeader {
        width: u32::from_le_bytes(header[0..4].try_into().unwrap()),
        height: u32::from_le_bytes(header[4..8].try_into().unwrap()),
    };
    let len = hdr
        .payload_len()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    buf.resize(len, 0);
    reader.read_exact(buf)?;
    Ok(Some(hdr))
}

/// Write one [`IpcResponse`] as a JSON line and flush.
///
/// Flushing per frame is required: the outer shell blocks on this line to
/// decide lane actuation, so buffering across frames would add latency.
pub fn write_response<W: Write>(writer: &mut W, response: &IpcResponse) -> std::io::Result<()> {
    serde_json::to_writer(&mut *writer, response)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

/// Current Unix time in microseconds.
pub fn unix_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn framed(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&width.to_le_bytes());
        out.extend_from_slice(&height.to_le_bytes());
        out.extend_from_slice(pixels);
        out
    }

    #[test]
    fn reads_framed_rgb24() {
        let pixels: Vec<u8> = (0..2 * 2 * 3).map(|i| i as u8).collect();
        let stream = framed(2, 2, &pixels);
        let mut cursor = Cursor::new(stream);
        let mut buf = Vec::new();
        let hdr = read_frame(&mut cursor, &mut buf).unwrap().unwrap();
        assert_eq!(
            hdr,
            FrameHeader {
                width: 2,
                height: 2
            }
        );
        assert_eq!(buf, pixels);
        // Stream is exhausted: next read is a clean EOF.
        assert!(read_frame(&mut cursor, &mut buf).unwrap().is_none());
    }

    #[test]
    fn clean_eof_returns_none() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let mut buf = Vec::new();
        assert!(read_frame(&mut cursor, &mut buf).unwrap().is_none());
    }

    #[test]
    fn truncated_header_is_an_error() {
        let mut cursor = Cursor::new(vec![1, 0, 0]);
        let mut buf = Vec::new();
        let err = read_frame(&mut cursor, &mut buf).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn truncated_payload_is_an_error() {
        let mut stream = framed(4, 4, &[]);
        stream.extend_from_slice(&[0u8; 10]); // needs 48 bytes
        let mut cursor = Cursor::new(stream);
        let mut buf = Vec::new();
        let err = read_frame(&mut cursor, &mut buf).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn zero_dimension_header_is_rejected() {
        let stream = framed(0, 480, &[]);
        let mut cursor = Cursor::new(stream);
        let mut buf = Vec::new();
        let err = read_frame(&mut cursor, &mut buf).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn oversized_header_is_rejected() {
        let stream = framed(u32::MAX, u32::MAX, &[]);
        let mut cursor = Cursor::new(stream);
        let mut buf = Vec::new();
        let err = read_frame(&mut cursor, &mut buf).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn response_serializes_to_expected_schema() {
        let response = IpcResponse {
            v: IPC_PROTOCOL_VERSION,
            frame: 42,
            ts_us: 1_718_000_000_123_456,
            lanes: vec![true, false, false, true],
            latency_us: 1840,
        };
        let mut out = Vec::new();
        write_response(&mut out, &response).unwrap();
        let line = String::from_utf8(out).unwrap();
        assert!(line.ends_with('\n'));
        let parsed: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(parsed["v"], 1);
        assert_eq!(parsed["frame"], 42);
        assert_eq!(parsed["ts_us"], 1_718_000_000_123_456u64);
        assert_eq!(
            parsed["lanes"],
            serde_json::json!([true, false, false, true])
        );
        assert_eq!(parsed["latency_us"], 1840);
    }
}
