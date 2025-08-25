//! Simple threaded pipeline connecting capture, processing and GPIO.

use crossbeam_channel::Receiver;

use crate::{
    config::SprayCfg,
    exg::exg_mask,
    io_gpio::Actuator,
    lanes::{reduce_lanes, LaneController},
};

/// A frame of RGB pixels.
pub struct RgbFrame {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

/// Trait producing RGB frames.
pub trait FrameSource: Send + 'static {
    fn start(self) -> Receiver<RgbFrame>;
}

/// Run the processing pipeline.
///
/// The source produces RGB frames which are turned into masks and reduced
/// into lane states.  Actuator receives final states.
pub fn run<S: FrameSource, A: Actuator>(source: S, mut act: A, cfg: SprayCfg) {
    let rx = source.start();
    let mut mask = Vec::new();
    let mut ctrl = LaneController::new(cfg.clone());
    let mut last = std::time::Instant::now();
    while let Ok(frame) = rx.recv() {
        let pixel_count = frame.width * frame.height;
        if mask.len() != pixel_count {
            mask.resize(pixel_count, 0);
        }
        exg_mask(&frame.data, &mut mask, cfg.thr);
        let ratios = reduce_lanes(&mask, frame.width, frame.height, cfg.bottom_frac);
        let now = std::time::Instant::now();
        let dt = now.duration_since(last).as_millis() as u32;
        last = now;
        let states = ctrl.update(ratios, dt);
        let _ = act.apply(ratios, states);
    }
}
