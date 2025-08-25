//! Excess-Green mask generation.
//!
//! The function operates on interleaved RGB data and writes a binary mask
//! into the provided slice without allocating.  A future iteration can
//! leverage platform-specific SIMD instructions, but the scalar version is
//! fast enough for the test suite and serves as a portable baseline.

/// Compute an excess green mask.
#[inline(always)]
pub fn exg_mask(rgb: &[u8], mask: &mut [u8], thr: i16) {
    assert_eq!(rgb.len(), mask.len() * 3);
    for (i, pix) in rgb.chunks_exact(3).enumerate() {
        let r = pix[0] as i16;
        let g = pix[1] as i16;
        let b = pix[2] as i16;
        mask[i] = if 2 * g - r - b >= thr { 1 } else { 0 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exg_basic() {
        let rgb = [10u8, 50, 10, 50, 10, 10];
        let mut mask = [0u8; 2];
        exg_mask(&rgb, &mut mask, 16);
        assert_eq!(mask, [1, 0]);
    }
}
