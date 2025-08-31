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
    while i + 48 <= rgb.len() {
        let r = u8x16::from_slice(&rgb[i..i + 16]);
        let g = u8x16::from_slice(&rgb[i + 16..i + 32]);
        let b = u8x16::from_slice(&rgb[i + 32..i + 48]);
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
}
