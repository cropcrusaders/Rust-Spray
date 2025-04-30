//! Vegetation-index helpers used by Green-on-Brown detection.

use opencv::{
    core::{self, Mat, Scalar, Vec3b},
    imgproc,
    prelude::*,
    Result,
};

/// Generic helper: split BGR channels into separate Mats.
fn split_bgr(src: &Mat) -> Result<(Mat, Mat, Mat)> {
    let mut chans = opencv::types::VectorOfMat::new();
    core::split(src, &mut chans)?;
    Ok((chans.get(0)?, chans.get(1)?, chans.get(2)?)) // (B,G,R)
}

/* ------------------------------------------------------------------------- */
/*  Single-channel indices – return a Mat with 8-bit unsigned values         */
/* ------------------------------------------------------------------------- */

pub fn exg(src: &Mat) -> Result<Mat> {
    let (b, g, r) = split_bgr(src)?;
    let mut exg = Mat::default();
    // exg  = 2G – R – B   (clamp to [0,255])
    core::add_weighted(&g, 2.0, &r, -1.0, 0.0, &mut exg, core::CV_32F)?;
    core::add_weighted(&exg, 1.0, &b, -1.0, 0.0, &mut exg, core::CV_32F)?;
    let mut u8 = Mat::default();
    exg.convert_to(&mut u8, core::CV_8U, 1.0, 0.0)?;
    Ok(u8)
}

pub fn exgr(src: &Mat) -> Result<Mat> {
    // exgr = exg – (R / (G + 1)) * 255   (simple variant)
    let exg = exg(src)?;
    let (_, _, r) = split_bgr(src)?;
    let mut g_plus1 = Mat::default();
    let (_, g, _) = split_bgr(src)?;
    core::add(&g, &Scalar::all(1.0), &mut g_plus1, &core::no_array()?, -1)?;
    let mut ratio = Mat::default();
    core::divide(&r, &g_plus1, &mut ratio, 255.0, core::CV_32F)?;
    let mut exgr = Mat::default();
    core::subtract(&exg, &ratio, &mut exgr, &core::no_array()?, -1)?;
    let mut u8 = Mat::default();
    exgr.convert_to(&mut u8, core::CV_8U, 1.0, 0.0)?;
    Ok(u8)
}

pub fn maxg(src: &Mat) -> Result<Mat> {
    let (b, g, r) = split_bgr(src)?;
    let mut max = Mat::default();
    core::max(&g, &r, &mut max)?;
    core::max(&max, &b, &mut max)?;
    Ok(max)
}

pub fn nexg(src: &Mat) -> Result<Mat> {
    // Normalised ExG  =  (G-R) / (G+R+B)
    let (b, g, r) = split_bgr(src)?;
    let mut gr = Mat::default();
    core::subtract(&g, &r, &mut gr, &core::no_array()?, -1)?;
    let mut sum = Mat::default();
    core::add(&g, &r, &mut sum, &core::no_array()?, -1)?;
    core::add(&sum, &b, &mut sum, &core::no_array()?, -1)?;
    let mut nexg = Mat::default();
    core::divide(&gr, &sum, &mut nexg, 127.0, core::CV_32F)?;
    let mut u8 = Mat::default();
    nexg.convert_to(&mut u8, core::CV_8U, 1.0, 0.0)?;
    Ok(u8)
}

pub fn gndvi(src: &Mat) -> Result<Mat> {
    // Very rough  – using B as NIR placeholder (for demo purposes)
    let (b, g, _) = split_bgr(src)?;
    let mut num = Mat::default();
    core::subtract(&g, &b, &mut num, &core::no_array()?, -1)?;
    let mut den = Mat::default();
    core::add(&g, &b, &mut den, &core::no_array()?, -1)?;
    let mut gndvi = Mat::default();
    core::divide(&num, &den, &mut gndvi, 127.0, core::CV_32F)?;
    let mut u8 = Mat::default();
    gndvi.convert_to(&mut u8, core::CV_8U, 1.0, 0.0)?;
    Ok(u8)
}

/* ------------------------------------------------------------------------- */
/*  HSV-based threshold helpers                                              */
/* ------------------------------------------------------------------------- */

pub fn hsv(
    src: &Mat,
    h_min: i32, h_max: i32,
    s_min: i32, s_max: i32,
    v_min: i32, v_max: i32,
    invert_hue: bool,
) -> Result<(Mat, bool)> {
    let mut hsv = Mat::default();
    imgproc::cvt_color(src, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;
    let lower = core::Scalar::new(h_min as f64, s_min as f64, v_min as f64, 0.0);
    let upper = core::Scalar::new(h_max as f64, s_max as f64, v_max as f64, 0.0);
    let mut mask = Mat::default();
    core::in_range(&hsv, &lower, &upper, &mut mask)?;
    if invert_hue {
        core::bitwise_not(&mask, &mut mask, &core::no_array()?)?;
    }
    Ok((mask, true))
}

pub fn exhsv(
    src: &Mat,
    exg_min: i32,
    exg_max: i32,
    h_min: i32, h_max: i32,
    s_min: i32, s_max: i32,
    v_min: i32, v_max: i32,
    invert_hue: bool,
) -> Result<(Mat, bool)> {
    let exg = exg(src)?;
    let mut exg_mask = Mat::default();
    core::in_range(
        &exg,
        &Scalar::new(exg_min as f64, 0.0, 0.0, 0.0),
        &Scalar::new(exg_max as f64, 0.0, 0.0, 0.0),
        &mut exg_mask,
    )?;
    let (hsv_mask, _) = hsv(src, h_min, h_max, s_min, s_max, v_min, v_max, invert_hue)?;
    let mut combined = Mat::default();
    core::bitwise_and(&exg_mask, &hsv_mask, &mut combined, &core::no_array()?)?;
    Ok((combined, true))
}
