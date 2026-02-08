//! Wiring of ExG mask -> lane reduction -> GPIO output.

use crate::{io_gpio::NozzleControl, lanes::LaneReducer, vision::PlantVision};

/// Processing pipeline using a boxed GPIO implementation.
pub struct Pipeline {
    reducer: LaneReducer,
    gpio: Box<dyn NozzleControl>,
    vision: PlantVision,
    width: usize,
    height: usize,
}

impl Pipeline {
    /// Create a new pipeline.
    pub fn new(
        reducer: LaneReducer,
        gpio: Box<dyn NozzleControl>,
        vision: PlantVision,
        width: usize,
        height: usize,
    ) -> Self {
        Self {
            reducer,
            gpio,
            vision,
            width,
            height,
        }
    }

    /// Process one RGB frame and return the lane activation states.
    pub fn process(&mut self, frame: &[u8]) -> Vec<bool> {
        assert_eq!(
            frame.len(),
            self.width * self.height * 3,
            "Frame length must match width * height * 3",
        );
        let mask = self.vision.detect(frame);
        let lanes = self.reducer.reduce(&mask, self.width, self.height);
        self.gpio.apply(&lanes);
        lanes
    }

    /// Per-lane vegetation density from the most recent frame.
    pub fn lane_density(&self) -> &[f32] {
        self.reducer.density()
    }
}
