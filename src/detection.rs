//! Green-on-Brown detection front-end.

use anyhow::{bail, Result as AResult};
use opencv::{
    core::{self, Mat, Point, Scalar, Size},
    imgproc,
    types::VectorOfVectorOfPoint,
    Result,
};
use std::collections::HashMap;

use crate::utils::algorithms::{
    exg, exgr, gndvi, maxg, nexg, exhsv, hsv,
};

/* ------------------------------------------------------------------ */
/*  function-pointer aliases                                          */
/* ------------------------------------------------------------------ */

type AlgFn             = fn(&Mat) -> Result<Mat>;
type AlgFnWithParams   = fn(
    &Mat,
    i32,i32,            // exg min/max  (ignored by plain HSV)
    i32,i32,            // hue min/max
    i32,i32,            // sat min/max
    i32,i32,            // val min/max
    bool,               // invert hue?
) -> Result<(Mat,bool)>;

/* ------------------------------------------------------------------ */
/*  wrapper so `hsv` matches the 8-int signature                       */
/* ------------------------------------------------------------------ */
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
) -> Result<(Mat,bool)> {
    hsv(src, h_min, h_max, s_min, s_max, v_min, v_max, invert)
}

/* ------------------------------------------------------------------ */

pub struct GreenOnBrown {
    alg: String,
    kernel: Mat,
    simple:  HashMap<String, AlgFn>,
    param:   HashMap<String, AlgFnWithParams>,
}

impl GreenOnBrown {
    pub fn new(algorithm: &str) -> Result<Self> {
        let kernel = imgproc::get_structuring_element(
            imgproc::MORPH_ELLIPSE,
            Size::new(3,3),
            Point::new(-1,-1),
        )?;

        let mut simple = HashMap::new();
        simple.insert("exg".into(),   exg   as AlgFn);
        simple.insert("exgr".into(),  exgr  as AlgFn);
        simple.insert("maxg".into(),  maxg  as AlgFn);
        simple.insert("nexg".into(),  nexg  as AlgFn);
        simple.insert("gndvi".into(), gndvi as AlgFn);

        let mut param = HashMap::new();
        param.insert("exhsv".into(), exhsv        as AlgFnWithParams);
        param.insert("hsv".into(),   hsv_wrapper  as AlgFnWithParams);

        Ok(Self { alg: algorithm.into(), kernel, simple, param })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inference(
        &self,
        frame: &Mat,
        exg_min: i32,
        exg_max: i32,
        h_min: i32, h_max: i32,
        s_min: i32, s_max: i32,
        v_min: i32, v_max: i32,
        min_area: f64,
        show: bool,
        algorithm: &str,
        invert_hue: bool,
        label: &str,
    ) -> Result<(VectorOfVectorOfPoint, Vec<[i32;4]>, Vec<[i32;2]>, Mat)> {

        /* ---- 1  build mask -------------------------------------------- */
        let (mut mask, pre_thresh): (Mat,bool) = if let Some(f) = self.simple.get(algorithm) {
            (f(frame)?, false)
        } else if let Some(f) = self.param.get(algorithm) {
            f(frame,
              exg_min, exg_max,
              h_min, h_max,
              s_min, s_max,
              v_min, v_max,
              invert_hue)?
        } else {
            bail!("unknown algorithm '{}'", algorithm);
        };

        if !pre_thresh {
            imgproc::threshold(
                &mask, &mut mask,
                0.0, 255.0,
                imgproc::THRESH_BINARY | imgproc::THRESH_OTSU,
            )?;
        }

        /* ---- 2  morphology clean-up ----------------------------------- */
        imgproc::erode  (&mask, &mut mask, &self.kernel, Point::new(-1,-1), 1, core::BORDER_CONSTANT, imgproc::morphology_default_border_value()?)?;
        imgproc::dilate(&mask, &mut mask, &self.kernel, Point::new(-1,-1), 2, core::BORDER_CONSTANT, imgproc::morphology_default_border_value()?)?;

        /* ---- 3  contours / boxes / centres ---------------------------- */
        let mut contours = VectorOfVectorOfPoint::new();
        imgproc::find_contours(
            &mask,
            &mut contours,
            imgproc::RETR_EXTERNAL,
            imgproc::CHAIN_APPROX_SIMPLE,
            Point::new(0,0),
        )?;

        let mut boxes = Vec::new();
        let mut ctrs  = Vec::new();
        let mut out   = frame.clone();

        for c in contours.iter() {
            let area = imgproc::contour_area(&c, false)?;
            if area < min_area { continue; }

            let rect = imgproc::bounding_rect(&c)?;
            boxes.push([rect.x, rect.y, rect.width, rect.height]);

            let cx = rect.x + rect.width  / 2;
            let cy = rect.y + rect.height / 2;
            ctrs.push([cx,cy]);

            imgproc::rectangle(&mut out, rect, Scalar::new(0.0,255.0,0.0,0.0), 2, imgproc::LINE_8, 0)?;
            imgproc::put_text(&mut out, label, Point::new(rect.x, rect.y-3), imgproc::FONT_HERSHEY_SIMPLEX, 0.5, Scalar::new(0.0,255.0,0.0,0.0), 1, imgproc::LINE_AA, false)?;
        }

        if show {
            // caller displays via highgui
        }

        Ok((contours, boxes, ctrs, out))
    }
}
