//! Fast, no-allocation vegetation-index helpers.
//! Each function returns `Result<Mat>` where the output is an 8-bit, single-channel image.

use opencv::{
    core::{self, Mat, Scalar},
    imgproc,
    prelude::*,
    Result,
};

fn split_bgr(src: &Mat) -> Result<(Mat, Mat, Mat)> {
    let mut v = opencv::types::VectorOfMat::new();
    core::split(src, &mut v)?;
    Ok((v.get(0)?, v.get(1)?, v.get(2)?)) // (B,G,R)
}

/* --------------------------------------------------------------------- */
/*  Simple indices                                                       */
/* --------------------------------------------------------------------- */

pub fn exg(src: &Mat) -> Result<Mat> {
    let (b, g, r) = split_bgr(src)?;
    let mut tmp = Mat::default();
    core::add_weighted(&g, 2.0, &r, -1.0, 0.0, &mut tmp, -1)?;
    core::add_weighted(&tmp, 1.0, &b, -1.0, 0.0, &mut tmp, -1)?;
    let mut out = Mat::default();
    tmp.convert_to(&mut out, core::CV_8U, 1.0, 0.0)?;
    Ok(out)
}

pub fn exgr(src: &Mat) -> Result<Mat> {
    let exg = exg(src)?;
    let (_, _, r) = split_bgr(src)?;
    let mut out = Mat::default();
    core::subtract(&exg, &r, &mut out, &Mat::default(), -1)?;
    Ok(out)
}

pub fn maxg(src: &Mat) -> Result<Mat> {
    let (b, g, r) = split_bgr(src)?;
    let mut max = Mat::default();
    core::max(&g, &r, &mut max)?;
    core::max(&max, &b, &mut max)?;
    Ok(max)
}

pub fn nexg(src: &Mat) -> Result<Mat> {
    let (b, g, r) = split_bgr(src)?;
    let mut top  = Mat::default();
    core::subtract(&g, &r, &mut top, &Mat::default(), -1)?;

    let mut denom = Mat::default();
    core::add(&g, &r, &mut denom, &Mat::default(), -1)?;
    core::add(&denom, &b, &mut denom, &Mat::default(), -1)?;

    // dst = (top / denom) * 127   →   divide(scale, src2, dst, dtype)
    core::divide(127.0, &denom, &mut denom, core::CV_32F)?;
    core::multiply(&top, &denom, &mut denom, 1.0, -1)?;
    let mut out = Mat::default();
    denom.convert_to(&mut out, core::CV_8U, 1.0, 0.0)?;
    Ok(out)
}

pub fn gndvi(src: &Mat) -> Result<Mat> {
    let (b, g, _) = split_bgr(src)?;
    let mut num = Mat::default();
    core::subtract(&g, &b, &mut num, &Mat::default(), -1)?;

    let mut denom = Mat::default();
    core::add(&g, &b, &mut denom, &Mat::default(), -1)?;

    core::divide(127.0, &denom, &mut denom, core::CV_32F)?;
    core::multiply(&num, &denom, &mut denom, 1.0, -1)?;
    let mut out = Mat::default();
    denom.convert_to(&mut out, core::CV_8U, 1.0, 0.0)?;
    Ok(out)
}

/* --------------------------------------------------------------------- */
/*  HSV threshold helpers                                                */
/* --------------------------------------------------------------------- */

pub fn hsv(
    src: &Mat,
    h_min: i32, h_max: i32,
    s_min: i32, s_max: i32,
    v_min: i32, v_max: i32,
    invert_hue: bool,
) -> Result<(Mat, bool)> {
    let mut hsv = Mat::default();
    imgproc::cvt_color(src, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;

    let lower = Scalar::new(h_min as f64, s_min as f64, v_min as f64, 0.0);
    let upper = Scalar::new(h_max as f64, s_max as f64, v_max as f64, 0.0);
    let mut mask = Mat::default();
    core::in_range(&hsv, &lower, &upper, &mut mask)?;

    if invert_hue {
        core::bitwise_not(&mask, &mut mask, &Mat::default())?;
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
    core::bitwise_and(&exg_mask, &hsv_mask, &mut combined, &Mat::default())?;
    Ok((combined, true))
}
