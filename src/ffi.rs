//! C FFI for embedding the detection kernel without a subprocess.
//!
//! Built into `librustspray_core.so` (crate-type `cdylib`); callable from
//! C, Python `ctypes`, or any FFI-capable language. See `INTEGRATION.md`
//! for the ABI contract.
//!
//! FFI v1 is **stateless**: each call runs vision + lane reduction on one
//! frame with a fresh reducer, so the `off` hysteresis threshold never
//! applies — a lane is on when its coverage exceeds the `on` threshold.
//! Hosts that want hysteresis and GPIO actuation should use the stateful
//! subprocess (`--ipc-mode`) instead.

use crate::{config::Config, lanes::LaneReducer, vision::PlantVision};
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};

const EINVAL: i32 = -22;
const ENOENT: i32 = -2;
const EIO: i32 = -5;

/// Detect vegetation in a single RGB24 frame.
///
/// # Parameters
/// - `rgb24`: pointer to `width * height * 3` bytes, row-major, R first
/// - `width`, `height`: frame dimensions in pixels
/// - `config`: NUL-terminated path to a TOML config file, or NULL to use
///   compiled-in defaults (only `[vision]` and `[lanes]` sections apply)
/// - `lane_states`: caller-allocated bool array, length >= `num_lanes`
/// - `num_lanes`: number of lanes to populate
///
/// # Returns
/// `0` on success; negative errno on failure:
/// - `-22` (EINVAL): NULL data/output pointer, zero or inconsistent
///   dimensions, `width < num_lanes`, or unparseable config file
/// - `-2` (ENOENT): `config` path does not exist
/// - `-5` (EIO): internal panic (caught; never unwinds across the FFI)
///
/// # Safety
/// `rgb24` must point to at least `width * height * 3` readable bytes and
/// `lane_states` to at least `num_lanes` writable bools for the duration
/// of the call. `config`, if non-NULL, must be NUL-terminated.
#[no_mangle]
pub extern "C" fn rustspray_detect(
    rgb24: *const u8,
    width: u32,
    height: u32,
    config: *const c_char,
    lane_states: *mut bool,
    num_lanes: u32,
) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        detect_impl(rgb24, width, height, config, lane_states, num_lanes)
    }))
    .unwrap_or(EIO)
}

fn detect_impl(
    rgb24: *const u8,
    width: u32,
    height: u32,
    config: *const c_char,
    lane_states: *mut bool,
    num_lanes: u32,
) -> i32 {
    if rgb24.is_null() || lane_states.is_null() {
        return EINVAL;
    }
    if width == 0 || height == 0 || num_lanes == 0 {
        return EINVAL;
    }
    let (w, h, lanes) = (width as usize, height as usize, num_lanes as usize);
    if w < lanes {
        return EINVAL;
    }
    let Some(len) = w.checked_mul(h).and_then(|p| p.checked_mul(3)) else {
        return EINVAL;
    };

    let cfg = if config.is_null() {
        Config::default()
    } else {
        // SAFETY: caller guarantees `config` is a NUL-terminated string.
        let Ok(path) = unsafe { std::ffi::CStr::from_ptr(config) }.to_str() else {
            return EINVAL;
        };
        let path = std::path::Path::new(path);
        if !path.exists() {
            return ENOENT;
        }
        match Config::load(path) {
            Ok(cfg) => cfg,
            Err(_) => return EINVAL,
        }
    };

    let vision = PlantVision::new(
        cfg.vision.exg_threshold,
        cfg.vision.green_ratio_floor,
        cfg.vision.chroma_floor,
        (
            cfg.vision.weights.exg,
            cfg.vision.weights.green_ratio,
            cfg.vision.weights.chroma,
            cfg.vision.weights.bias,
        ),
    );

    // SAFETY: caller guarantees `rgb24` points to `len` readable bytes.
    let frame = unsafe { std::slice::from_raw_parts(rgb24, len) };
    let mask = vision.detect(frame);

    let mut reducer = LaneReducer::new(lanes, cfg.lanes.on_threshold, cfg.lanes.off_threshold);
    let states = reducer.reduce(&mask, w, h);

    // SAFETY: caller guarantees `lane_states` has room for `num_lanes`.
    let out = unsafe { std::slice::from_raw_parts_mut(lane_states, lanes) };
    out.copy_from_slice(&states);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Left half green, right half soil.
    fn test_frame(width: usize, height: usize) -> Vec<u8> {
        let mut frame = Vec::with_capacity(width * height * 3);
        for _y in 0..height {
            for x in 0..width {
                if x < width / 2 {
                    frame.extend_from_slice(&[20, 200, 20]);
                } else {
                    frame.extend_from_slice(&[120, 90, 70]);
                }
            }
        }
        frame
    }

    #[test]
    fn detects_with_default_config() {
        let frame = test_frame(8, 2);
        let mut lanes = [false; 2];
        let rc = rustspray_detect(
            frame.as_ptr(),
            8,
            2,
            std::ptr::null(),
            lanes.as_mut_ptr(),
            2,
        );
        assert_eq!(rc, 0);
        assert_eq!(lanes, [true, false]);
    }

    #[test]
    fn rejects_null_pointers() {
        let mut lanes = [false; 2];
        assert_eq!(
            rustspray_detect(
                std::ptr::null(),
                8,
                2,
                std::ptr::null(),
                lanes.as_mut_ptr(),
                2
            ),
            EINVAL,
        );
        let frame = test_frame(8, 2);
        assert_eq!(
            rustspray_detect(
                frame.as_ptr(),
                8,
                2,
                std::ptr::null(),
                std::ptr::null_mut(),
                2
            ),
            EINVAL,
        );
    }

    #[test]
    fn rejects_bad_dimensions() {
        let frame = test_frame(8, 2);
        let mut lanes = [false; 4];
        // Zero dimension.
        assert_eq!(
            rustspray_detect(
                frame.as_ptr(),
                0,
                2,
                std::ptr::null(),
                lanes.as_mut_ptr(),
                2
            ),
            EINVAL,
        );
        // Width smaller than lane count.
        assert_eq!(
            rustspray_detect(
                frame.as_ptr(),
                2,
                2,
                std::ptr::null(),
                lanes.as_mut_ptr(),
                4
            ),
            EINVAL,
        );
    }

    #[test]
    fn missing_config_is_enoent() {
        let frame = test_frame(8, 2);
        let mut lanes = [false; 2];
        let path = std::ffi::CString::new("/nonexistent/rustspray-ffi-test.toml").unwrap();
        assert_eq!(
            rustspray_detect(frame.as_ptr(), 8, 2, path.as_ptr(), lanes.as_mut_ptr(), 2),
            ENOENT,
        );
    }
}
