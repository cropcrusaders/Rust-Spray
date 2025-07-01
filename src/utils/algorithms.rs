//! Lightweight vegetation-index helpers for weed detection
//!
//! This module provides various computer vision algorithms for detecting
//! vegetation in agricultural images, including ExG, HSV thresholding,
//! and hybrid approaches.

use opencv::{
    core::{self, Mat, Scalar, Vector},
    imgproc,
    prelude::*,
    Result,
};

// ─── Helper functions ───────────────────────────────────────────────────────

/// Split a BGR image into its component channels
/// 
/// # Arguments
/// * `src` - Input BGR image
/// 
/// # Returns
/// * `Result<(Mat, Mat, Mat)>` - Blue, Green, Red channels or error
fn split_bgr(src: &Mat) -> Result<(Mat, Mat, Mat)> {
    let mut v: Vector<Mat> = Vector::new();
    core::split(src, &mut v)?;
    Ok((v.get(0)?, v.get(1)?, v.get(2)?)) // (B,G,R)
}

/// Convert a floating-point image to 8-bit unsigned
/// 
/// # Arguments
/// * `src` - Input floating-point image
/// 
/// # Returns
/// * `Result<Mat>` - 8-bit image or error
fn to_u8(src: &Mat) -> Result<Mat> {
    let mut out = Mat::default();
    src.convert_to(&mut out, core::CV_8U, 1.0, 0.0)?;
    Ok(out)
}

// ─── Vegetation Index Algorithms ───────────────────────────────────────────

/// Excess Green (ExG) vegetation index
/// 
/// Calculates ExG = 2*G - R - B, useful for detecting green vegetation
/// against brown soil backgrounds.
/// 
/// # Arguments
/// * `src` - Input BGR image
/// 
/// # Returns
/// * `Result<Mat>` - ExG index image or error
pub fn exg(src: &Mat) -> Result<Mat> {
    let (b, g, r) = split_bgr(src)?;
    let mut mix1 = Mat::default();
    core::add_weighted(&g, 2.0, &r, -1.0, 0.0, &mut mix1, -1)?;
    let mut mix2 = Mat::default();
    core::add_weighted(&mix1, 1.0, &b, -1.0, 0.0, &mut mix2, -1)?;
    to_u8(&mix2)
}

/// Excess Green minus Excess Red (ExGR) vegetation index
/// 
/// Combines ExG with ExR to improve vegetation detection by reducing
/// reddish soil interference.
/// 
/// # Arguments
/// * `src` - Input BGR image
/// 
/// # Returns
/// * `Result<Mat>` - ExGR index image or error
pub fn exgr(src: &Mat) -> Result<Mat> {
    let exg_img = exg(src)?;
    let (_, _, r) = split_bgr(src)?;
    let mut diff = Mat::default();
    core::subtract(&exg_img, &r, &mut diff, &Mat::default(), -1)?;
    Ok(diff)
}

/// Maximum Green (MaxG) vegetation index
/// 
/// Takes the maximum of the green channel against red and blue channels.
/// Simple but effective for certain lighting conditions.
/// 
/// # Arguments
/// * `src` - Input BGR image
/// 
/// # Returns
/// * `Result<Mat>` - MaxG index image or error
pub fn maxg(src: &Mat) -> Result<Mat> {
    let (b, g, r) = split_bgr(src)?;
    let mut tmp = Mat::default();
    core::max(&g, &r, &mut tmp)?;
    let mut out = Mat::default();
    core::max(&tmp, &b, &mut out)?;
    Ok(out)
}

/* NExG ─────────────────────────────────────────────────────────────── */
pub fn nexg(src: &Mat) -> Result<Mat> {
    let (b, g, r) = split_bgr(src)?;

    let mut num = Mat::default();
    core::subtract(&g, &r, &mut num, &Mat::default(), -1)?;

    let mut sum = Mat::default();
    core::add(&g, &r, &mut sum, &Mat::default(), -1)?;
    let mut denom = Mat::default();
    core::add(&sum, &b, &mut denom, &Mat::default(), -1)?;

    let mut inv = Mat::default();
    core::divide(127.0, &denom, &mut inv, core::CV_32F)?;

    let mut scaled = Mat::default();
    core::multiply(&num, &inv, &mut scaled, 1.0, -1)?;
    to_u8(&scaled)
}

/* GNDVI (blue as fake NIR) ─────────────────────────────────────────── */
pub fn gndvi(src: &Mat) -> Result<Mat> {
    let (b, g, _) = split_bgr(src)?;

    let mut num = Mat::default();
    core::subtract(&g, &b, &mut num, &Mat::default(), -1)?;

    let mut denom = Mat::default();
    core::add(&g, &b, &mut denom, &Mat::default(), -1)?;

    let mut inv = Mat::default();
    core::divide(127.0, &denom, &mut inv, core::CV_32F)?;

    let mut scaled = Mat::default();
    core::multiply(&num, &inv, &mut scaled, 1.0, -1)?;
    to_u8(&scaled)
}

/* HSV threshold ────────────────────────────────────────────────────── */
pub fn hsv(
    src: &Mat,
    h_min: i32,
    h_max: i32,
    s_min: i32,
    s_max: i32,
    v_min: i32,
    v_max: i32,
    invert: bool,
) -> Result<(Mat, bool)> {
    let mut hsv = Mat::default();
    imgproc::cvt_color(src, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;

    let lower = Scalar::new(h_min as f64, s_min as f64, v_min as f64, 0.0);
    let upper = Scalar::new(h_max as f64, s_max as f64, v_max as f64, 0.0);
    let mut mask = Mat::default();
    core::in_range(&hsv, &lower, &upper, &mut mask)?;

    if invert {
        let mut neg = Mat::default();
        core::bitwise_not(&mask, &mut neg, &Mat::default())?;
        mask = neg;
    }
    Ok((mask, true))
}

/* ExHSV (ExG mask AND HSV mask) ────────────────────────────────────── */
pub fn exhsv(
    src: &Mat,
    exg_min: i32,
    exg_max: i32,
    h_min: i32,
    h_max: i32,
    s_min: i32,
    s_max: i32,
    v_min: i32,
    v_max: i32,
    invert: bool,
) -> Result<(Mat, bool)> {
    let exg_mask = {
        let exg_img = exg(src)?;
        let mut m = Mat::default();
        core::in_range(
            &exg_img,
            &Scalar::new(exg_min as f64, 0.0, 0.0, 0.0),
            &Scalar::new(exg_max as f64, 0.0, 0.0, 0.0),
            &mut m,
        )?;
        m
    };

    let (hsv_mask, _) = hsv(src, h_min, h_max, s_min, s_max, v_min, v_max, invert)?;

    let mut combined = Mat::default();
    core::bitwise_and(&exg_mask, &hsv_mask, &mut combined, &Mat::default())?;
    Ok((combined, true))
}
