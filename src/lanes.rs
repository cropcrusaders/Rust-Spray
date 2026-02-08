//! Lane reduction with hysteresis and temporal smoothing.

/// Reduces a mask into fixed lanes and applies hysteresis.
///
/// The reducer divides each frame into vertical strips (lanes), computes
/// the vegetation density in each, and applies hysteresis thresholds to
/// produce stable on/off spray decisions. Optional temporal hold and ROI
/// settings further stabilise output for real-world use.
pub struct LaneReducer {
    lanes: usize,
    on: f32,
    off: f32,
    state: Vec<bool>,
    density: Vec<f32>,
    frames_in_state: Vec<u32>,
    hold_on: u32,
    hold_off: u32,
    roi: (f32, f32),
}

impl LaneReducer {
    /// Create a new reducer.
    ///
    /// * `lanes` - number of lanes.
    /// * `on` - ratio threshold to switch lane on.
    /// * `off` - ratio threshold to switch lane off.
    pub fn new(lanes: usize, on: f32, off: f32) -> Self {
        assert!(lanes > 0, "Number of lanes must be greater than 0");
        Self {
            lanes,
            on,
            off,
            state: vec![false; lanes],
            density: vec![0.0; lanes],
            frames_in_state: vec![0; lanes],
            hold_on: 0,
            hold_off: 0,
            roi: (0.0, 1.0),
        }
    }

    /// Set temporal hold frames to prevent rapid toggling.
    ///
    /// * `hold_on` - minimum frames a lane stays ON before it can turn OFF.
    /// * `hold_off` - minimum frames a lane stays OFF before it can turn ON.
    pub fn with_hold(mut self, hold_on: u32, hold_off: u32) -> Self {
        self.hold_on = hold_on;
        self.hold_off = hold_off;
        self
    }

    /// Set vertical region of interest as fractions of frame height.
    ///
    /// Only rows within `[top * height, bottom * height)` are analysed.
    /// Defaults to `(0.0, 1.0)` (full frame).
    pub fn with_roi(mut self, top: f32, bottom: f32) -> Self {
        assert!(
            top >= 0.0 && top < bottom && bottom <= 1.0,
            "ROI must satisfy 0 <= top < bottom <= 1"
        );
        self.roi = (top, bottom);
        self
    }

    /// Per-lane vegetation density from the last `reduce` call.
    pub fn density(&self) -> &[f32] {
        &self.density
    }

    /// Current lane activation states.
    pub fn state(&self) -> &[bool] {
        &self.state
    }

    /// Number of configured lanes.
    pub fn lane_count(&self) -> usize {
        self.lanes
    }

    /// Reduce the mask given image width/height.
    pub fn reduce(&mut self, mask: &[bool], width: usize, height: usize) -> Vec<bool> {
        assert!(
            width >= self.lanes,
            "Width must be greater than or equal to number of lanes"
        );
        assert_eq!(
            mask.len(),
            width * height,
            "Mask length must equal width * height"
        );

        let roi_start = (self.roi.0 * height as f32) as usize;
        let roi_end = ((self.roi.1 * height as f32) as usize).min(height);
        let roi_height = roi_end - roi_start;

        let base_width = width / self.lanes;
        let remainder = width % self.lanes;
        let mut x_start = 0usize;
        let mut out = vec![false; self.lanes];
        for (lane, lane_out) in out.iter_mut().enumerate().take(self.lanes) {
            let lane_width = base_width + usize::from(lane < remainder);
            let mut count = 0u32;
            for y in roi_start..roi_end {
                let start = y * width + x_start;
                let end = start + lane_width;
                for &px in &mask[start..end] {
                    if px {
                        count += 1;
                    }
                }
            }
            let total = (lane_width * roi_height) as f32;
            let ratio = if total > 0.0 {
                count as f32 / total
            } else {
                0.0
            };
            self.density[lane] = ratio;

            // Hysteresis decision
            let wants_on = if self.state[lane] {
                ratio > self.off
            } else {
                ratio > self.on
            };

            // Temporal hold: prevent rapid toggling between frames.
            self.frames_in_state[lane] += 1;
            let is_on = if wants_on != self.state[lane] {
                let min_hold = if self.state[lane] {
                    self.hold_on
                } else {
                    self.hold_off
                };
                if self.frames_in_state[lane] >= min_hold {
                    self.frames_in_state[lane] = 0;
                    wants_on
                } else {
                    self.state[lane]
                }
            } else {
                wants_on
            };

            *lane_out = is_on;
            x_start += lane_width;
        }
        self.state.clone_from_slice(&out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_activation_with_hysteresis() {
        let width = 4;
        let height = 1;
        // First two pixels active -> lane 0
        let mask = [true, true, false, false];
        let mut reducer = LaneReducer::new(2, 0.5, 0.25);
        let lanes = reducer.reduce(&mask, width, height);
        assert_eq!(lanes, vec![true, false]);
        // Half active -> stays on due to hysteresis off=0.25
        let mask = [true, false, false, false];
        let lanes = reducer.reduce(&mask, width, height);
        assert_eq!(lanes, vec![true, false]);
        // Below off threshold -> turns off
        let mask = [false, false, false, false];
        let lanes = reducer.reduce(&mask, width, height);
        assert_eq!(lanes, vec![false, false]);
    }

    #[test]
    #[should_panic(expected = "Number of lanes must be greater than 0")]
    fn test_zero_lanes_panics() {
        LaneReducer::new(0, 0.5, 0.25);
    }

    #[test]
    #[should_panic(expected = "Width must be greater than or equal to number of lanes")]
    fn test_width_less_than_lanes_panics() {
        let mut reducer = LaneReducer::new(5, 0.5, 0.25);
        let mask = [true, true];
        reducer.reduce(&mask, 2, 1); // width=2, lanes=5
    }

    #[test]
    fn test_edge_case_width_equals_lanes() {
        let width = 2;
        let height = 1;
        let mask = [true, false];
        let mut reducer = LaneReducer::new(2, 0.5, 0.25);
        let lanes = reducer.reduce(&mask, width, height);
        assert_eq!(lanes, vec![true, false]);
    }

    #[test]
    fn test_lane_width_with_remainder() {
        let width = 5;
        let height = 1;
        let mask = [true, true, true, false, false];
        let mut reducer = LaneReducer::new(2, 0.5, 0.25);
        let lanes = reducer.reduce(&mask, width, height);
        assert_eq!(lanes, vec![true, false]);
    }

    #[test]
    fn hold_on_prevents_early_deactivation() {
        let width = 4;
        let height = 1;
        let mut reducer = LaneReducer::new(2, 0.5, 0.25).with_hold(2, 0);

        // Activate lane 0
        let lanes = reducer.reduce(&[true, true, false, false], width, height);
        assert_eq!(lanes, vec![true, false]);

        // Vegetation disappears, but hold_on=2 keeps it spraying
        let lanes = reducer.reduce(&[false, false, false, false], width, height);
        assert_eq!(lanes[0], true, "lane held ON by hold_on");

        // After enough frames the hold expires and lane turns off
        let lanes = reducer.reduce(&[false, false, false, false], width, height);
        assert_eq!(lanes[0], false, "hold expired, lane OFF");
    }

    #[test]
    fn hold_off_prevents_early_reactivation() {
        let width = 4;
        let height = 1;
        let mut reducer = LaneReducer::new(2, 0.5, 0.25).with_hold(0, 3);

        // Frame 1: vegetation detected but hold_off=3 blocks (counter=1)
        let lanes = reducer.reduce(&[true, true, false, false], width, height);
        assert_eq!(lanes[0], false, "blocked by hold_off");

        // Frame 2: still blocked (counter=2)
        let lanes = reducer.reduce(&[true, true, false, false], width, height);
        assert_eq!(lanes[0], false, "still blocked by hold_off");

        // Frame 3: hold_off satisfied (counter=3 >= 3), lane activates
        let lanes = reducer.reduce(&[true, true, false, false], width, height);
        assert_eq!(lanes[0], true, "hold_off expired, lane ON");
    }

    #[test]
    fn roi_restricts_analysed_rows() {
        let width = 4;
        let height = 10;
        // Only rows 5-9 have green (bottom half)
        let mut mask = vec![false; width * height];
        for y in 5..10 {
            for x in 0..2 {
                mask[y * width + x] = true;
            }
        }

        // Full frame: 50% green in lane 0 → above on=0.4 → ON
        let mut full = LaneReducer::new(2, 0.4, 0.2);
        let lanes = full.reduce(&mask, width, height);
        assert!(lanes[0], "full frame sees green");

        // ROI top half only: no green there → OFF
        let mut top_only = LaneReducer::new(2, 0.4, 0.2).with_roi(0.0, 0.5);
        let lanes = top_only.reduce(&mask, width, height);
        assert!(!lanes[0], "top ROI misses the green");

        // ROI bottom half: all green → ON
        let mut bot_only = LaneReducer::new(2, 0.4, 0.2).with_roi(0.5, 1.0);
        let lanes = bot_only.reduce(&mask, width, height);
        assert!(lanes[0], "bottom ROI sees the green");
    }

    #[test]
    fn density_tracks_per_lane_ratio() {
        let width = 4;
        let height = 1;
        let mask = [true, true, false, false];
        let mut reducer = LaneReducer::new(2, 0.5, 0.25);
        reducer.reduce(&mask, width, height);
        let d = reducer.density();
        assert!((d[0] - 1.0).abs() < f32::EPSILON, "lane 0 fully green");
        assert!((d[1] - 0.0).abs() < f32::EPSILON, "lane 1 no green");
    }

    #[test]
    fn four_lane_symmetric_detection() {
        // Simulate a 640-wide frame with green in lanes 0 and 2
        let width = 640;
        let height = 1;
        let mut mask = vec![false; width];
        for x in 0..160 {
            mask[x] = true; // lane 0
        }
        for x in 320..480 {
            mask[x] = true; // lane 2
        }
        let mut reducer = LaneReducer::new(4, 0.3, 0.15);
        let lanes = reducer.reduce(&mask, width, height);
        assert_eq!(lanes, vec![true, false, true, false]);
        let d = reducer.density();
        assert!((d[0] - 1.0).abs() < f32::EPSILON);
        assert!((d[1] - 0.0).abs() < f32::EPSILON);
        assert!((d[2] - 1.0).abs() < f32::EPSILON);
        assert!((d[3] - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    #[should_panic(expected = "ROI must satisfy")]
    fn invalid_roi_panics() {
        LaneReducer::new(4, 0.3, 0.15).with_roi(0.8, 0.2);
    }
}
