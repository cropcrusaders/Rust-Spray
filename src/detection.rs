use opencv::{
    core::{self, Mat, Point, Scalar, Size, Vector},
    imgproc,
    types::VectorOfVectorOfPoint,
    Result,
};
use crate::utils::algorithms::exg;

pub struct GreenOnBrown {
    algorithm: String,
    kernel: Mat,
}

impl GreenOnBrown {
    pub fn new(algorithm: &str) -> Result<Self> {
        let kernel = imgproc::get_structuring_element(
            imgproc::MORPH_ELLIPSE,
            Size::new(3, 3),
            Point::new(-1, -1),
        )?;
        Ok(GreenOnBrown {
            algorithm: algorithm.to_string(),
            kernel,
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
        let mut output = exg(image);
        let mut threshold_out = Mat::default();
        imgproc::threshold(&output, &mut threshold_out, 30.0, 255.0, imgproc::THRESH_BINARY)?;

        let mut contours = VectorOfVectorOfPoint::new();
        imgproc::find_contours(
            &threshold_out,
            &mut contours,
            imgproc::RETR_EXTERNAL,
            imgproc::CHAIN_APPROX_SIMPLE,
            Point::new(0, 0),
        )?;

        let mut boxes = Vec::new();
        let mut weed_centres = Vec::new();
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