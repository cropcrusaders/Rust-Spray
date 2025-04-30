//! Green-on-Brown weed-detection engine.
//!
//! 1. Runs a selected vegetation index on every frame
//! 2. Morph-cleans the mask
//! 3. Extracts contours + bounding boxes + centroids
//!
//! The heavy pixel math lives in `utils::algorithms`.

use opencv::{
    core::{self, Mat, Point, Scalar, Size},
    imgproc,
    prelude::*,
    types::VectorOfVectorOfPoint,
    Result,
};
use std::collections::HashMap;

use crate::utils::algorithms::{
    exg, exgr, maxg, nexg, exhsv, hsv, gndvi,
};

type AlgorithmFn           = fn(&Mat) -> Result<Mat>;
type AlgorithmFnWithParams = fn(
    &Mat,
    i32, i32,          // exg_min / exg_max   (only for exhsv)
    i32, i32,          // hue_min / hue_max
    i32, i32,          // sat_min / sat_max
    i32, i32,          // val_min / val_max
    bool,              // invert hue?
) -> Result<(Mat, bool)>;

pub struct GreenOnBrown {
    algorithm: String,
    kernel:    Mat,
    algorithms:              HashMap<String, AlgorithmFn>,
    algorithms_with_params:  HashMap<String, AlgorithmFnWithParams>,
}

impl GreenOnBrown {
    pub fn new(algorithm: &str) -> Result<Self> {
        let kernel = imgproc::get_structuring_element(
            imgproc::MORPH_ELLIPSE,
            Size::new(3, 3),
            Point::new(-1, -1),
        )?;

        let mut algorithms = HashMap::new();
        algorithms.insert("exg".into(),   exg   as AlgorithmFn);
        algorithms.insert("exgr".into(),  exgr  as AlgorithmFn);
        algorithms.insert("maxg".into(),  maxg  as AlgorithmFn);
        algorithms.insert("nexg".into(),  nexg  as AlgorithmFn);
        algorithms.insert("gndvi".into(), gndvi as AlgorithmFn);

        let mut algorithms_with_params = HashMap::new();
        algorithms_with_params.insert("exhsv".into(), exhsv as AlgorithmFnWithParams);
        algorithms_with_params.insert("hsv".into(),   hsv   as AlgorithmFnWithParams);

        Ok(Self {
            algorithm: algorithm.to_owned(),
            kernel,
            algorithms,
            algorithms_with_params,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inference(
        &self,
        frame: &Mat,
        exg_min: i32,
        exg_max: i32,
        hue_min: i32,
        hue_max: i32,
        sat_min: i32,
        sat_max: i32,
        val_min: i32,
        val_max: i32,
        min_area: f64,
        show_display: bool,
        algorithm: &str,
        invert_hue: bool,
        label: &str,
    ) -> Result<(VectorOfVectorOfPoint, Vec<[i32; 4]>, Vec<[i32; 2]>, Mat)> {
        // ------------------------------------------------------------------ #
        // 1. build binary mask
        // ------------------------------------------------------------------ #
        let (mask, pre_thresh) = if let Some(f) = self.algorithms.get(algorithm) {
            (f(frame)?, false)
        } else if let Some(f) = self.algorithms_with_params.get(algorithm) {
            f(
                frame,
                exg_min, exg_max,
                hue_min, hue_max,
                sat_min, sat_max,
                val_min, val_max,
                invert_hue,
            )?
        } else {
            anyhow::bail!("unknown algorithm {}", algorithm);
        };

        // if algorithm already thresholded -> mask is binary; if not, apply Otsu
        let mut thresh = Mat::default();
        if pre_thresh {
            thresh = mask;
        } else {
            imgproc::threshold(
                &mask,
                &mut thresh,
                0.0,
                255.0,
                imgproc::THRESH_BINARY | imgproc::THRESH_OTSU,
            )?;
        }

        // ------------------------------------------------------------------ #
        // 2. morphology cleanup
        // ------------------------------------------------------------------ #
        imgproc::erode(&thresh, &mut thresh, &self.kernel, Point::new(-1, -1), 1, core::BORDER_CONSTANT, imgproc::morphology_default_border_value()?)?;
        imgproc::dilate(&thresh, &mut thresh, &self.kernel, Point::new(-1, -1), 2, core::BORDER_CONSTANT, imgproc::morphology_default_border_value()?)?;

        // ------------------------------------------------------------------ #
        // 3. extract contours
        // ------------------------------------------------------------------ #
        let mut contours = VectorOfVectorOfPoint::new();
        imgproc::find_contours(
            &thresh,
            &mut contours,
            imgproc::RETR_EXTERNAL,
            imgproc::CHAIN_APPROX_SIMPLE,
            Point::new(0, 0),
        )?;

        let mut boxes   = Vec::new();
        let mut centres = Vec::new();
        let mut colour  = frame.clone();

        for contour in contours.iter() {
            let area = imgproc::contour_area(&contour, false)?;
            if area < min_area { continue; }

            let rect = imgproc::bounding_rect(&contour)?;
            boxes.push([rect.x, rect.y, rect.width, rect.height]);

            let cx = rect.x + rect.width / 2;
            let cy = rect.y + rect.height / 2;
            centres.push([cx, cy]);

            imgproc::rectangle(&mut colour, rect, Scalar::new(0.0, 255.0, 0.0, 0.0), 2, imgproc::LINE_8, 0)?;
            imgproc::put_text(&mut colour, label, Point::new(rect.x, rect.y - 3), imgproc::FONT_HERSHEY_SIMPLEX, 0.5, Scalar::new(0.0,255.0,0.0,0.0), 1, imgproc::LINE_AA, false)?;
        }

        // optional display handled in main.rs
        Ok((contours, boxes, centres, colour))
    }
}
