//! Green-on-Brown detection front-end.
//!
//! This module provides weed detection functionality using various computer vision
//! algorithms including ExG, HSV thresholding, and hybrid approaches.

use opencv::{
    core::{self, Mat, Point, Scalar, Size, Vector},
    imgproc, Result,
};
use std::collections::HashMap;

use crate::utils::algorithms::{exg, exgr, exhsv, gndvi, hsv, maxg, nexg};

/// Detection parameters structure to reduce function parameter count
#[derive(Debug, Clone)]
pub struct DetectionParams {
    pub exg_min: i32,
    pub exg_max: i32,
    pub hue_min: i32,
    pub hue_max: i32,
    pub brightness_min: i32,
    pub brightness_max: i32,
    pub saturation_min: i32,
    pub saturation_max: i32,
    pub min_area: f64,
    pub invert_hue: bool,
    pub algorithm: String,
}

/// Detection result containing all relevant information
#[derive(Debug)]
pub struct DetectionResult {
    pub contours: Vector<Vector<Point>>,
    pub bounding_boxes: Vec<[i32; 4]>,
    pub centers: Vec<[i32; 2]>,
    pub annotated_frame: Mat,
}

/* ───────────── function-pointer aliases ───────────── */

type AlgFn = fn(&Mat) -> Result<Mat>;
type AlgFnWithParams = fn(
    &Mat,
    i32,
    i32, // exg min / max (ignored by plain HSV)
    i32,
    i32, // hue min / max
    i32,
    i32, // sat min / max
    i32,
    i32,  // val min / max
    bool, // invert hue?
) -> Result<(Mat, bool)>;

/// Wrapper so plain HSV fits the parameterized function signature
#[allow(clippy::too_many_arguments)]
fn hsv_wrapper(
    src: &Mat,
    _exg_min: i32,
    _exg_max: i32,
    h_min: i32,
    h_max: i32,
    s_min: i32,
    s_max: i32,
    v_min: i32,
    v_max: i32,
    invert: bool,
) -> Result<(Mat, bool)> {
    hsv(src, h_min, h_max, s_min, s_max, v_min, v_max, invert)
}

/// Green-on-Brown detection engine
pub struct GreenOnBrown {
    kernel: Mat,
    simple: HashMap<String, AlgFn>,
    param: HashMap<String, AlgFnWithParams>,
}

impl GreenOnBrown {
    /// Create a new GreenOnBrown detector
    ///
    /// # Arguments
    /// * `default_alg` - Default algorithm to validate (for early error detection)
    ///
    /// # Returns
    /// * `Result<Self>` - New detector instance or error if algorithm is unsupported
    pub fn new(default_alg: &str) -> Result<Self> {
        let kernel = imgproc::get_structuring_element(
            imgproc::MORPH_ELLIPSE,
            Size::new(3, 3),
            Point::new(-1, -1),
        )?;

        let mut simple = HashMap::new();
        simple.insert("exg".into(), exg as AlgFn);
        simple.insert("exgr".into(), exgr as AlgFn);
        simple.insert("maxg".into(), maxg as AlgFn);
        simple.insert("nexg".into(), nexg as AlgFn);
        simple.insert("gndvi".into(), gndvi as AlgFn);

        let mut param = HashMap::new();
        param.insert("exhsv".into(), exhsv as AlgFnWithParams);
        param.insert("hsv".into(), hsv_wrapper as AlgFnWithParams);

        if !simple.contains_key(default_alg) && !param.contains_key(default_alg) {
            return Err(opencv::Error::new(
                core::StsError,
                format!("unknown detection algorithm '{default_alg}'"),
            ));
        }

        Ok(Self {
            kernel,
            simple,
            param,
        })
    }

    /// Run inference on a frame with simplified parameter structure
    ///
    /// # Arguments
    /// * `frame` - Input image frame
    /// * `params` - Detection parameters
    /// * `show_window` - Whether to prepare frame for display
    /// * `label` - Label to draw on detected objects
    ///
    /// # Returns
    /// * `Result<DetectionResult>` - Detection results or error
    pub fn detect(
        &self,
        frame: &Mat,
        params: &DetectionParams,
        show_window: bool,
        label: &str,
    ) -> Result<DetectionResult> {
        self.inference(
            frame,
            params.exg_min,
            params.exg_max,
            params.hue_min,
            params.hue_max,
            params.brightness_min,
            params.brightness_max,
            params.saturation_min,
            params.saturation_max,
            params.min_area,
            show_window,
            &params.algorithm,
            params.invert_hue,
            label,
        )
        .map(|(contours, boxes, centers, annotated)| DetectionResult {
            contours,
            bounding_boxes: boxes,
            centers,
            annotated_frame: annotated,
        })
    }

    /// Legacy inference method (kept for backward compatibility)
    ///
    /// Consider using `detect()` with `DetectionParams` for cleaner code.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    pub fn inference(
        &self,
        frame: &Mat,
        exg_min: i32,
        exg_max: i32,
        h_min: i32,
        h_max: i32,
        s_min: i32,
        s_max: i32,
        v_min: i32,
        v_max: i32,
        min_area: f64,
        show_window: bool,
        algorithm: &str,
        invert_hue: bool,
        label: &str,
    ) -> Result<(Vector<Vector<Point>>, Vec<[i32; 4]>, Vec<[i32; 2]>, Mat)> {
        /* 1 ─ build mask */
        let (mut mask, already_thresh) = if let Some(f) = self.simple.get(algorithm) {
            (f(frame)?, false)
        } else if let Some(f) = self.param.get(algorithm) {
            f(
                frame, exg_min, exg_max, h_min, h_max, s_min, s_max, v_min, v_max, invert_hue,
            )?
        } else {
            return Err(opencv::Error::new(
                core::StsError,
                format!("unknown algorithm '{algorithm}'"),
            ));
        };

        /* threshold (temp Mat avoids borrow clash) */
        if !already_thresh {
            let mut tmp = Mat::default();
            imgproc::threshold(
                &mask,
                &mut tmp,
                0.0,
                255.0,
                imgproc::THRESH_BINARY | imgproc::THRESH_OTSU,
            )?;
            mask = tmp;
        }

        /* 2 ─ morphology cleanup */
        {
            let mut tmp = Mat::default();
            imgproc::erode(
                &mask,
                &mut tmp,
                &self.kernel,
                Point::new(-1, -1),
                1,
                core::BORDER_CONSTANT,
                imgproc::morphology_default_border_value()?,
            )?;
            mask = tmp;
        }
        {
            let mut tmp = Mat::default();
            imgproc::dilate(
                &mask,
                &mut tmp,
                &self.kernel,
                Point::new(-1, -1),
                2,
                core::BORDER_CONSTANT,
                imgproc::morphology_default_border_value()?,
            )?;
            mask = tmp;
        }

        /* 3 ─ contours, boxes, centres */
        let mut contours: Vector<Vector<Point>> = Vector::new();
        imgproc::find_contours(
            &mask,
            &mut contours,
            imgproc::RETR_EXTERNAL,
            imgproc::CHAIN_APPROX_SIMPLE,
            Point::new(0, 0),
        )?;

        let mut boxes = Vec::new();
        let mut centres = Vec::new();
        let mut annotated = frame.clone();

        for c in contours.iter() {
            if imgproc::contour_area(&c, false)? < min_area {
                continue;
            }

            let rect = imgproc::bounding_rect(&c)?;
            boxes.push([rect.x, rect.y, rect.width, rect.height]);
            centres.push([rect.x + rect.width / 2, rect.y + rect.height / 2]);

            imgproc::rectangle(
                &mut annotated,
                rect,
                Scalar::new(0.0, 255.0, 0.0, 0.0),
                2,
                imgproc::LINE_8,
                0,
            )?;
            imgproc::put_text(
                &mut annotated,
                label,
                Point::new(rect.x, rect.y - 3),
                imgproc::FONT_HERSHEY_SIMPLEX,
                0.5,
                Scalar::new(0.0, 255.0, 0.0, 0.0),
                1,
                imgproc::LINE_AA,
                false,
            )?;
        }

        if show_window {
            // caller displays via highgui
        }

        Ok((contours, boxes, centres, annotated))
    }
}
