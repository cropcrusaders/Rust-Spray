#![feature(portable_simd)]

//! Core lane spraying pipeline for agricultural robotics.
//!
//! Processes camera frames through three stages:
//! 1. **Vision** — multi-cue vegetation detection ([`vision::PlantVision`])
//! 2. **Lane reduction** — per-lane coverage with hysteresis ([`lanes::LaneReducer`])
//! 3. **Actuation** — GPIO relay control ([`io_gpio::NozzleControl`])
//!
//! The crate builds both as an rlib (used by the `rustspray` binary and the
//! examples) and as a cdylib exposing the C ABI in [`ffi`]. The stdin/stdout
//! protocol used to embed the binary inside an outer shell such as
//! OpenWeedLocator lives in [`ipc`].

pub mod config;
pub mod exg;
pub mod ffi;
pub mod io_gpio;
pub mod ipc;
pub mod lanes;
pub mod pipeline;
pub mod vision;

#[cfg(test)]
mod kernel_tests {
    //! End-to-end tests of the detection kernel (vision + lane reduction)
    //! against synthetic RGB24 frames with known weed/no-weed patches.

    use crate::lanes::LaneReducer;
    use crate::vision::PlantVision;

    const WIDTH: usize = 64;
    const HEIGHT: usize = 16;

    const GREEN: [u8; 3] = [40, 210, 40]; // healthy weed
    const SOIL: [u8; 3] = [120, 90, 70]; // dry soil background

    /// Paint `pixel` at every (x, y) where `predicate(x)` holds, soil elsewhere.
    fn frame_where(predicate: impl Fn(usize) -> bool) -> Vec<u8> {
        let mut frame = vec![0u8; WIDTH * HEIGHT * 3];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = (y * WIDTH + x) * 3;
                let px = if predicate(x) { GREEN } else { SOIL };
                frame[idx..idx + 3].copy_from_slice(&px);
            }
        }
        frame
    }

    fn detect_lanes(frame: &[u8], num_lanes: usize) -> Vec<bool> {
        let vision = PlantVision::default();
        let mut reducer = LaneReducer::new(num_lanes, 0.3, 0.15);
        let mask = vision.detect(frame);
        reducer.reduce(&mask, WIDTH, HEIGHT)
    }

    #[test]
    fn all_weed_activates_every_lane() {
        let frame = frame_where(|_| true);
        assert_eq!(detect_lanes(&frame, 4), vec![true; 4]);
    }

    #[test]
    fn no_weed_activates_no_lane() {
        let frame = frame_where(|_| false);
        assert_eq!(detect_lanes(&frame, 4), vec![false; 4]);
    }

    #[test]
    fn single_pixel_weed_stays_below_threshold() {
        // One green pixel out of 16x16 in a lane is ~0.4% coverage — far
        // under the 30% on-threshold, so no lane may fire.
        let mut frame = frame_where(|_| false);
        frame[0..3].copy_from_slice(&GREEN);
        assert_eq!(detect_lanes(&frame, 4), vec![false; 4]);
    }

    #[test]
    fn weed_patches_map_to_their_lanes() {
        // Green in the first and third quarters -> lanes 0 and 2.
        let frame = frame_where(|x| x < WIDTH / 4 || (WIDTH / 2..3 * WIDTH / 4).contains(&x));
        assert_eq!(detect_lanes(&frame, 4), vec![true, false, true, false]);
    }

    #[test]
    fn weed_on_lane_boundary_fires_only_the_covered_lane() {
        // Green strip covering half of lane 1 only (columns 16..24 of a
        // 16-wide lane): 50% coverage fires lane 1, neighbours stay off.
        let frame = frame_where(|x| (16..24).contains(&x));
        assert_eq!(detect_lanes(&frame, 4), vec![false, true, false, false]);
    }
}
