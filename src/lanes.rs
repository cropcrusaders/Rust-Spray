//! Lane reduction and valve timing logic.

use crate::config::SprayCfg;

/// Reduce a binary mask into four lane ratios.
#[inline]
pub fn reduce_lanes(mask: &[u8], width: usize, height: usize, bottom_frac: f32) -> [f32; 4] {
    assert_eq!(mask.len(), width * height);
    let start_row = (height as f32 * (1.0 - bottom_frac)) as usize;
    let mut on = [0u32; 4];
    let mut total = [0u32; 4];
    for y in start_row..height {
        let row_offset = y * width;
        for x in 0..width {
            let lane = x * 4 / width;
            let idx = row_offset + x;
            total[lane] += 1;
            on[lane] += mask[idx] as u32;
        }
    }
    let mut ratios = [0.0f32; 4];
    for i in 0..4 {
        ratios[i] = if total[i] > 0 {
            on[i] as f32 / total[i] as f32
        } else {
            0.0
        };
    }
    ratios
}

/// Maintain lane timing with minimum-on and cooldown windows.
#[derive(Debug, Clone)]
pub struct LaneController {
    cfg: SprayCfg,
    fire_rem: [u32; 4],
    cooldown_rem: [u32; 4],
    armed: [bool; 4],
}

impl LaneController {
    pub fn new(cfg: SprayCfg) -> Self {
        Self {
            cfg,
            fire_rem: [0; 4],
            cooldown_rem: [0; 4],
            armed: [false; 4],
        }
    }

    /// Update lane states based on new ratios and elapsed time.
    pub fn update(&mut self, ratios: [f32; 4], dt_ms: u32) -> [bool; 4] {
        let mut states = [false; 4];
        for lane in 0..4 {
            if self.fire_rem[lane] > 0 {
                if self.fire_rem[lane] > dt_ms {
                    self.fire_rem[lane] -= dt_ms;
                    states[lane] = true;
                } else {
                    self.fire_rem[lane] = 0;
                    self.cooldown_rem[lane] = self.cfg.holdoff_ms;
                }
                continue;
            }
            if self.cooldown_rem[lane] > 0 {
                if self.cooldown_rem[lane] > dt_ms {
                    self.cooldown_rem[lane] -= dt_ms;
                    continue;
                } else {
                    self.cooldown_rem[lane] = 0;
                }
            }
            let threshold = if self.armed[lane] {
                self.cfg.min_ratio * (1.0 - self.cfg.hysteresis)
            } else {
                self.cfg.min_ratio * (1.0 + self.cfg.hysteresis)
            };
            if ratios[lane] >= threshold {
                self.armed[lane] = true;
                self.fire_rem[lane] = self.cfg.fire_ms;
                states[lane] = true;
            } else {
                self.armed[lane] = false;
            }
        }
        states
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_reduction() {
        // 4x2 image, bottom_frac=0.5 -> use last row
        let mask = [1, 0, 0, 0, 1, 1, 0, 0];
        let ratios = reduce_lanes(&mask, 4, 2, 0.5);
        assert_eq!(ratios, [1.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn timing_logic() {
        let cfg = SprayCfg {
            fire_ms: 40,
            holdoff_ms: 100,
            min_ratio: 0.01,
            hysteresis: 0.5,
            ..SprayCfg::default()
        };
        let mut ctrl = LaneController::new(cfg);
        // First frame triggers lane 0
        let mut states = ctrl.update([0.02, 0.0, 0.0, 0.0], 0);
        assert_eq!(states, [true, false, false, false]);
        // Next frame within fire_ms keeps it on regardless of ratio
        states = ctrl.update([0.0, 0.0, 0.0, 0.0], 20);
        assert_eq!(states, [true, false, false, false]);
        // After fire_ms passes, lane turns off and enters cooldown
        states = ctrl.update([0.0, 0.0, 0.0, 0.0], 20);
        assert_eq!(states, [false, false, false, false]);
        // Cooldown prevents retrigger
        states = ctrl.update([0.02, 0.0, 0.0, 0.0], 20);
        assert_eq!(states, [false, false, false, false]);
        // After cooldown, trigger again
        states = ctrl.update([0.02, 0.0, 0.0, 0.0], 80);
        assert_eq!(states, [true, false, false, false]);
    }
}
