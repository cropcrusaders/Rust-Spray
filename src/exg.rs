//! SIMD-based Excess Green (ExG) mask.

use std::simd::prelude::*;
use std::simd::{i16x16, u8x16, Simd};

/// Computes an ExG mask from an interleaved RGB image.
///
/// * `rgb` - RGB pixel data in interleaved format.
/// * `threshold` - Activation threshold for ExG (values above trigger `true`).
///
/// Returns a vector of booleans representing the mask.
pub fn exg_mask(rgb: &[u8], threshold: i16) -> Vec<bool> {
    assert!(
        rgb.len().is_multiple_of(3),
        "RGB slice must be multiple of 3"
    );
    let mut out = Vec::with_capacity(rgb.len() / 3);
    let mut i = 0;
    // 16 pixels (48 interleaved bytes) per iteration. The channels must be
    // deinterleaved with stride 3 before the vector arithmetic.
    while i + 48 <= rgb.len() {
        let px = &rgb[i..i + 48];
        let r = u8x16::from_array(std::array::from_fn(|j| px[3 * j]));
        let g = u8x16::from_array(std::array::from_fn(|j| px[3 * j + 1]));
        let b = u8x16::from_array(std::array::from_fn(|j| px[3 * j + 2]));
        let r16: i16x16 = r.cast();
        let g16: i16x16 = g.cast();
        let b16: i16x16 = b.cast();
        let exg = g16 * Simd::splat(2) - r16 - b16;
        let mask = exg.simd_gt(Simd::splat(threshold));
        out.extend_from_slice(&mask.to_array());
        i += 48;
    }
    // Process remaining pixels scalar.
    while i < rgb.len() {
        let r = rgb[i] as i16;
        let g = rgb[i + 1] as i16;
        let b = rgb[i + 2] as i16;
        out.push(2 * g - r - b > threshold);
        i += 3;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exg_detects_green() {
        // Pixel with strong green component should pass threshold.
        let rgb = [10u8, 240u8, 10u8];
        let mask = exg_mask(&rgb, 100);
        assert!(mask[0]);
    }

    #[test]
    fn exg_rejects_non_green() {
        let rgb = [120u8, 120u8, 120u8];
        let mask = exg_mask(&rgb, 10);
        assert!(!mask[0]);
    }

    #[test]
    fn simd_path_handles_interleaved_pixels() {
        // 20 pixels exercises the 16-wide SIMD block plus the scalar
        // remainder. Alternate green / grey pixels so a channel mixup in
        // the deinterleave shows up immediately.
        let mut rgb = Vec::new();
        for i in 0..20 {
            if i % 2 == 0 {
                rgb.extend_from_slice(&[10, 240, 10]);
            } else {
                rgb.extend_from_slice(&[120, 120, 120]);
            }
        }
        let mask = exg_mask(&rgb, 20);
        assert_eq!(mask.len(), 20);
        for (i, &m) in mask.iter().enumerate() {
            assert_eq!(m, i % 2 == 0, "pixel {i}");
        }
    }

    #[test]
    fn simd_matches_scalar_reference() {
        // Deterministic varied pattern across many pixels; the SIMD path
        // must agree with the scalar ExG formula for every one.
        let mut rgb = Vec::new();
        for i in 0u32..333 {
            rgb.push((i * 7 % 256) as u8);
            rgb.push((i * 13 % 256) as u8);
            rgb.push((i * 29 % 256) as u8);
        }
        let mask = exg_mask(&rgb, 20);
        assert_eq!(mask.len(), 333);
        for (p, &m) in mask.iter().enumerate() {
            let r = rgb[3 * p] as i16;
            let g = rgb[3 * p + 1] as i16;
            let b = rgb[3 * p + 2] as i16;
            assert_eq!(m, 2 * g - r - b > 20, "pixel {p}");
        }
    }
}
