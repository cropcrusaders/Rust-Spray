//! C FFI entry point for the detection kernel.
//!
//! Built as part of the `cdylib` (`librustspray_core.so`) so the vegetation
//! detector can be called in-process from C, Python (`ctypes`), or embedded
//! firmware without spawning the `rustspray` binary.
//!
//! The FFI surface is a **pure detection kernel**: it computes lane states
//! from a frame and never touches GPIO. Actuation stays with the caller.
//! Each call is stateless — lane hysteresis needs frame-to-frame state, so
//! the `on` threshold alone decides lane activation here. Callers that want
//! hysteresis should keep the subprocess IPC mode instead.

use crate::config::Config;
use crate::lanes::LaneReducer;
use crate::vision::PlantVision;
use std::os::raw::c_char;

// Negated on return, matching the "negative errno" convention.
const ENOENT: i32 = 2;
const EIO: i32 = 5;
const EINVAL: i32 = 22;

/// Detect weeds in a single RGB24 frame.
///
/// # Parameters
/// - `rgb24`: pointer to `width * height * 3` bytes, row-major, R first
/// - `width`, `height`: frame dimensions in pixels (both non-zero, and
///   `width >= num_lanes`)
/// - `config`: pointer to a NUL-terminated path to a TOML config file,
///   or NULL to use compiled-in defaults
/// - `lane_states`: caller-allocated bool array, length >= `num_lanes`
/// - `num_lanes`: number of lanes to populate (non-zero)
///
/// # Returns
/// - `0` on success
/// - `-EINVAL` (-22) for NULL data pointers, invalid dimensions, or an
///   unparseable/invalid config file
/// - `-ENOENT` (-2) if `config` names a file that does not exist
/// - `-EIO` (-5) if the kernel panics internally
///
/// # Safety
/// `rgb24` must point to at least `width * height * 3` readable bytes and
/// `lane_states` to at least `num_lanes` writable bools for the duration
/// of the call. `config`, when non-NULL, must be a valid NUL-terminated
/// string.
#[no_mangle]
pub extern "C" fn rustspray_detect(
    rgb24: *const u8,
    width: u32,
    height: u32,
    config: *const c_char,
    lane_states: *mut bool,
    num_lanes: u32,
) -> i32 {
    // The kernel must never unwind across the FFI boundary.
    std::panic::catch_unwind(|| detect_impl(rgb24, width, height, config, lane_states, num_lanes))
        .unwrap_or(-EIO)
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
        return -EINVAL;
    }
    if width == 0 || height == 0 || num_lanes == 0 || (width as u64) < num_lanes as u64 {
        return -EINVAL;
    }
    let frame_len = match (width as usize)
        .checked_mul(height as usize)
        .and_then(|px| px.checked_mul(3))
    {
        Some(len) => len,
        None => return -EINVAL,
    };

    let cfg = if config.is_null() {
        Config::default()
    } else {
        // SAFETY: caller guarantees `config` is a valid NUL-terminated string.
        let path = match unsafe { std::ffi::CStr::from_ptr(config) }.to_str() {
            Ok(p) => p,
            Err(_) => return -EINVAL,
        };
        let path = std::path::Path::new(path);
        // An explicitly named config file must exist: silently running on
        // defaults could apply the wrong detection tuning in the field.
        if !path.exists() {
            return -ENOENT;
        }
        match Config::load(path) {
            Ok(c) => c,
            Err(_) => return -EINVAL,
        }
    };
    if cfg.validate().is_err() {
        return -EINVAL;
    }

    // SAFETY: caller guarantees the buffer sizes documented above.
    let frame = unsafe { std::slice::from_raw_parts(rgb24, frame_len) };
    let out = unsafe { std::slice::from_raw_parts_mut(lane_states, num_lanes as usize) };

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
    let mut reducer = LaneReducer::new(
        num_lanes as usize,
        cfg.lanes.on_threshold,
        cfg.lanes.off_threshold,
    );

    let mask = vision.detect(frame);
    let lanes = reducer.reduce(&mask, width as usize, height as usize);
    out.copy_from_slice(&lanes);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a width x height RGB24 frame where columns in [green_from,
    /// green_to) are bright green and everything else is dry soil.
    fn synthetic_frame(width: usize, height: usize, green_from: usize, green_to: usize) -> Vec<u8> {
        let mut frame = vec![0u8; width * height * 3];
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 3;
                if x >= green_from && x < green_to {
                    frame[idx] = 40;
                    frame[idx + 1] = 210;
                    frame[idx + 2] = 40;
                } else {
                    frame[idx] = 120;
                    frame[idx + 1] = 90;
                    frame[idx + 2] = 70;
                }
            }
        }
        frame
    }

    #[test]
    fn detects_weed_in_first_lane() {
        let (w, h) = (64usize, 16usize);
        let frame = synthetic_frame(w, h, 0, 16); // lane 0 fully green
        let mut lanes = [false; 4];
        let rc = rustspray_detect(
            frame.as_ptr(),
            w as u32,
            h as u32,
            std::ptr::null(),
            lanes.as_mut_ptr(),
            4,
        );
        assert_eq!(rc, 0);
        assert_eq!(lanes, [true, false, false, false]);
    }

    #[test]
    fn clean_frame_yields_no_lanes() {
        let (w, h) = (64usize, 16usize);
        let frame = synthetic_frame(w, h, 0, 0); // all soil
        let mut lanes = [true; 4]; // pre-set to verify they are overwritten
        let rc = rustspray_detect(
            frame.as_ptr(),
            w as u32,
            h as u32,
            std::ptr::null(),
            lanes.as_mut_ptr(),
            4,
        );
        assert_eq!(rc, 0);
        assert_eq!(lanes, [false; 4]);
    }

    #[test]
    fn null_pointers_are_rejected() {
        let mut lanes = [false; 4];
        assert_eq!(
            rustspray_detect(
                std::ptr::null(),
                64,
                16,
                std::ptr::null(),
                lanes.as_mut_ptr(),
                4
            ),
            -EINVAL,
        );
        let frame = synthetic_frame(64, 16, 0, 0);
        assert_eq!(
            rustspray_detect(
                frame.as_ptr(),
                64,
                16,
                std::ptr::null(),
                std::ptr::null_mut(),
                4
            ),
            -EINVAL,
        );
    }

    #[test]
    fn invalid_dimensions_are_rejected() {
        let frame = synthetic_frame(64, 16, 0, 0);
        let mut lanes = [false; 4];
        for (w, h, n) in [(0u32, 16u32, 4u32), (64, 0, 4), (64, 16, 0), (2, 16, 4)] {
            assert_eq!(
                rustspray_detect(
                    frame.as_ptr(),
                    w,
                    h,
                    std::ptr::null(),
                    lanes.as_mut_ptr(),
                    n
                ),
                -EINVAL,
                "expected -EINVAL for w={w} h={h} lanes={n}",
            );
        }
    }

    #[test]
    fn missing_config_path_returns_enoent() {
        let frame = synthetic_frame(64, 16, 0, 0);
        let mut lanes = [false; 4];
        let path = std::ffi::CString::new("/nonexistent/rustspray-ffi-test.toml").unwrap();
        assert_eq!(
            rustspray_detect(frame.as_ptr(), 64, 16, path.as_ptr(), lanes.as_mut_ptr(), 4),
            -ENOENT,
        );
    }
}
