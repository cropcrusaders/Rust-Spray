use serde::Deserialize;

/// Configuration for the spraying pipeline.
#[derive(Debug, Clone, Deserialize)]
pub struct SprayCfg {
    /// Excess green threshold applied after `2*G - R - B` computation.
    pub thr: i16,
    /// Fraction of image height considered for lane reduction.
    pub bottom_frac: f32,
    /// Minimum green ratio required to trigger a lane.
    pub min_ratio: f32,
    /// Minimum time (in ms) a valve should stay open once triggered.
    pub fire_ms: u32,
    /// Cooldown time (in ms) after a valve closes before it can fire again.
    pub holdoff_ms: u32,
    /// Hysteresis percentage around `min_ratio` to prevent flicker.
    pub hysteresis: f32,
}

impl Default for SprayCfg {
    fn default() -> Self {
        Self {
            thr: 16,
            bottom_frac: 0.3,
            min_ratio: 0.008,
            fire_ms: 60,
            holdoff_ms: 200,
            hysteresis: 0.5,
        }
    }
}
