//! Lane reduction with hysteresis.

/// Reduces a mask into fixed lanes and applies hysteresis.
pub struct LaneReducer {
    lanes: usize,
    on: f32,
    off: f32,
    state: Vec<bool>,
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
        }
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

        let lane_width = width / self.lanes;
        let mut out = vec![false; self.lanes];
        for (lane, lane_out) in out.iter_mut().enumerate().take(self.lanes) {
            let mut count = 0u32;
            for y in 0..height {
                let start = y * width + lane * lane_width;
                let end = start + lane_width.min(width - lane * lane_width);
                for &px in &mask[start..end] {
                    if px {
                        count += 1;
                    }
                }
            }
            let total = (lane_width * height) as f32;
            let ratio = if total > 0.0 {
                count as f32 / total
            } else {
                0.0
            };
            let is_on = if self.state[lane] {
                ratio > self.off
            } else {
                ratio > self.on
            };
            *lane_out = is_on;
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
}
