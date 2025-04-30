use opencv::{
    core::{self, Mat, Point, Scalar, Size},
    imgproc,
    prelude::*,                    // <- brings `MatTraitConst` for `convert_to`
    types::VectorOfVectorOfPoint,
    Result,
};
use std::collections::HashMap;

use crate::utils::algorithms::{exg, exgr, maxg, nexg, exhsv, hsv, gndvi};

// Type alias for algorithm functions
type AlgorithmFn = fn(&Mat) -> Mat;
type AlgorithmFnWithParams = fn(
    &Mat,
    i32, i32, i32, i32, i32, i32, bool,
) -> Result<(Mat, bool)>;

pub struct GreenOnBrown {
    algorithm: String,
    kernel: Mat,
    algorithms: HashMap<String, AlgorithmFn>,
    algorithms_with_params: HashMap<String, AlgorithmFnWithParams>,
}

impl GreenOnBrown {
    pub fn new(algorithm: &str) -> Result<Self> {
        let kernel = imgproc::get_structuring_element(
            imgproc::MORPH_ELLIPSE,
            Size::new(3, 3),
            Point::new(-1, -1),
        )?;

        let mut algorithms = HashMap::new();
        algorithms.insert("exg".to_string(), exg as AlgorithmFn);
        algorithms.insert("exgr".to_string(), exgr as AlgorithmFn);
        algorithms.insert("maxg".to_string(), maxg as AlgorithmFn);
        algorithms.insert("nexg".to_string(), nexg as AlgorithmFn);
        algorithms.insert("gndvi".to_string(), gndvi as AlgorithmFn);

        let mut algorithms_with_params = HashMap::new();
        algorithms_with_params.insert("exhsv".to_string(), exhsv as AlgorithmFnWithParams);
        algorithms_with_params.insert("hsv".to_string(), hsv as AlgorithmFnWithParams);

        Ok(GreenOnBrown {
            algorithm: algorithm.to_string(),
            kernel,
            algorithms,
            algorithms_with_params,
        })
    }

    pub fn inference(
        &self,
        image: &Mat,
        exg_min: i32,
        exg_max: i32,
        hue_min: i32,
        hue_max: i32,
        brightness_min: i32,
        brightness_max: i32,
        saturation_min: i32,
        saturation_max: i32,
        min_area: f64,
        show_display: bool,
        algorithm: &str,
        invert_hue: bool,
        label: &str,
    ) -> Result<(VectorOfVectorOfPoint, Vec<[i32; 4]>, Vec<[i32; 2]>, Mat)> {
        let mut threshed_already = false;
        let mut output: Mat;

        if let Some(func) = self.algorithms_with_params.get(algorithm) {
            let (result, threshed) = func(
                image,
                hue_min,
                hue_max,
                brightness_min,
                brightness_max,
                saturation_min,
                saturation_max,
                invert_hue,
            )?;
            output = result;
            threshed_already = threshed;
        } else {
            let func = self.algorithms.get(algorithm).unwrap_or(&nexg);
            output = func(image);
        }

        let mut weed_centres: Vec<[i32; 2]> = Vec::new();
        let mut boxes: Vec<[i32; 4]> = Vec::new();

        if !threshed_already {
            let mut clipped = Mat::default();
            core::in_range(&output, &Scalar::from(exg_min as f64), &Scalar::from(exg_max as f64), &mut clipped)?;
            output = Mat::default();
            clipped.convert_to(&mut output, core::CV_8U, 1.0, 0.0)?;

            let mut threshold_out = Mat::default();
            imgproc::adaptive_threshold(
                &output,
                &mut threshold_out,
                255.0,
                imgproc::ADAPTIVE_THRESH_GAUSSIAN_C,
                imgproc::THRESH_BINARY_INV,
                31,
                2.0,
            )?;

            imgproc::morphology_ex(
                &threshold_out,
                &mut threshold_out,
                imgproc::MORPH_CLOSE,
                &self.kernel,
                Point::new(-1, -1),
                1,
                core::BORDER_CONSTANT,
                Scalar::default(),
            )?;
            output = threshold_out;
        } else {
            imgproc::morphology_ex(
                &output,
                &mut output,
                imgproc::MORPH_CLOSE,
                &self.kernel,
                Point::new(-1, -1),
                5,
                core::BORDER_CONSTANT,
                Scalar::default(),
            )?;
        }

        let mut contours = VectorOfVectorOfPoint::new();
        imgproc::find_contours(
            &output,
            &mut contours,
            imgproc::RETR_EXTERNAL,
            imgproc::CHAIN_APPROX_SIMPLE,
            Point::new(0, 0),
        )?;

        for contour in contours.iter() {
            let area = imgproc::contour_area(&contour, false)?;
            if area > min_area {
                let rect = imgproc::bounding_rect(&contour)?;
                boxes.push([rect.x, rect.y, rect.width, rect.height]);
                weed_centres.push([rect.x + rect.width / 2, rect.y + rect.height / 2]);
            }
        }

        let image_out = if show_display {
            let mut img_copy = image.clone();
            for box_coords in &boxes {
                let [start_x, start_y, w, h] = *box_coords;
                let end_x = start_x + w;
                let end_y = start_y + h;

                imgproc::rectangle(
                    &mut img_copy,
                    core::Rect::new(start_x, start_y, w, h),
                    Scalar::from((0.0, 0.0, 255.0)),
                    2,
                    8,
                    0,
                )?;

                imgproc::put_text(
                    &mut img_copy,
                    label,
                    Point::new(start_x, start_y + 30),
                    imgproc::FONT_HERSHEY_SIMPLEX,
                    1.0,
                    Scalar::from((255.0, 0.0, 0.0)),
                    2,
                    8,
                    false,
                )?;
            }
            img_copy
        } else {
            image.clone()
        };

        Ok((contours, boxes, weed_centres, image_out))
    }
}
