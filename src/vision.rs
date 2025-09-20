//! Adaptive vegetation detector combining multiple color cues.

/// High-level vegetation detector tuned for spotting green plants.
///
/// The detector fuses a classic Excess Green measurement with
/// chromaticity and green dominance ratios. This hybrid approach keeps
/// the responsiveness of ExG on bright vegetation while being more
/// robust against neutral backgrounds such as bare soil or stubble.
#[derive(Debug, Clone)]
pub struct PlantVision {
    /// Minimum ExG response required before a pixel is considered.
    pub exg_threshold: i16,
    /// Minimum share of green compared to overall brightness.
    pub green_ratio_floor: f32,
    /// Minimum chroma to reject grey/brown backgrounds.
    pub chroma_floor: f32,
    weights: Weights,
}

#[derive(Debug, Clone, Copy)]
struct Weights {
    exg: f32,
    green_ratio: f32,
    chroma: f32,
    bias: f32,
}

impl Default for PlantVision {
    fn default() -> Self {
        Self {
            exg_threshold: 20,
            green_ratio_floor: 0.36,
            chroma_floor: 0.08,
            weights: Weights {
                exg: 0.5,
                green_ratio: 0.35,
                chroma: 0.15,
                bias: 0.0,
            },
        }
    }
}

impl PlantVision {
    /// Create a detector with custom thresholds and weights.
    pub fn new(
        exg_threshold: i16,
        green_ratio_floor: f32,
        chroma_floor: f32,
        weights: (f32, f32, f32, f32),
    ) -> Self {
        let (exg, green_ratio, chroma, bias) = weights;
        Self {
            exg_threshold,
            green_ratio_floor,
            chroma_floor,
            weights: Weights {
                exg,
                green_ratio,
                chroma,
                bias,
            },
        }
    }

    /// Compute a vegetation mask for an interleaved RGB image.
    pub fn detect(&self, rgb: &[u8]) -> Vec<bool> {
        assert!(
            rgb.len().is_multiple_of(3),
            "RGB slice must be multiple of 3",
        );
        let mut mask = Vec::with_capacity(rgb.len() / 3);
        for chunk in rgb.chunks_exact(3) {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            mask.push(self.score_pixel(r, g, b) > 0.0);
        }
        mask
    }

    #[inline]
    fn score_pixel(&self, r: u8, g: u8, b: u8) -> f32 {
        let r_f = r as f32;
        let g_f = g as f32;
        let b_f = b as f32;
        let sum = r_f + g_f + b_f + 1.0; // avoid division by zero
        let exg = 2.0 * g_f - r_f - b_f;
        let exg_term = (exg - self.exg_threshold as f32) / 255.0;
        let green_ratio = g_f / sum;
        let green_ratio_term = green_ratio - self.green_ratio_floor;
        let maxc = r.max(g).max(b) as f32;
        let minc = r.min(g).min(b) as f32;
        let chroma = (maxc - minc) / 255.0;
        let chroma_term = chroma - self.chroma_floor;

        self.weights.exg * exg_term
            + self.weights.green_ratio * green_ratio_term
            + self.weights.chroma * chroma_term
            + self.weights.bias
    }
}

#[cfg(test)]
mod tests {
    use super::PlantVision;

    #[test]
    fn bright_green_is_detected() {
        let detector = PlantVision::default();
        let mask = detector.detect(&[60, 220, 60]);
        assert!(mask[0]);
    }

    #[test]
    fn dry_soil_is_rejected() {
        let detector = PlantVision::default();
        let mask = detector.detect(&[120, 90, 70]);
        assert!(!mask[0]);
    }

    #[test]
    fn override_weights_changes_sensitivity() {
        let conservative = PlantVision::new(180, 0.6, 0.3, (0.6, 0.3, 0.1, 0.0));
        let mask = conservative.detect(&[70, 150, 60]);
        assert!(!mask[0]);
        let aggressive = PlantVision::new(10, 0.25, 0.02, (0.5, 0.3, 0.2, 0.1));
        let mask = aggressive.detect(&[70, 150, 60]);
        assert!(mask[0]);
    }
}
