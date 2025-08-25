//! Wiring of ExG mask -> lane reduction -> GPIO output.

use crate::{exg::exg_mask, io_gpio::NozzleControl, lanes::LaneReducer};

/// Processing pipeline using a boxed GPIO implementation.
pub struct Pipeline {
    reducer: LaneReducer,
    gpio: Box<dyn NozzleControl>,
    threshold: i16,
    width: usize,
    height: usize,
}

impl Pipeline {
    /// Create a new pipeline.
    pub fn new(
        reducer: LaneReducer,
        gpio: Box<dyn NozzleControl>,
        threshold: i16,
        width: usize,
        height: usize,
    ) -> Self {
        Self {
            reducer,
            gpio,
            threshold,
            width,
            height,
        }
    }

    /// Process one RGB frame.
    pub fn process(&mut self, frame: &[u8]) {
        let mask = exg_mask(frame, self.threshold);
        let lanes = self.reducer.reduce(&mask, self.width, self.height);
        self.gpio.apply(&lanes);
    }
}
