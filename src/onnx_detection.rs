use opencv::{
    core::{self, Mat, Size},
    imgproc,
    prelude::*,
};
use ort::{Environment, Session, SessionBuilder, Value};
use anyhow::Result;
use ndarray::Array4;

pub struct OnnxDetector {
    session: Session,
    input_dims: [usize; 4],
}

impl OnnxDetector {
    pub fn new(model_path: &str) -> Result<Self> {
        let environment = Environment::builder().with_name("rustspray").build()?;
        let session = SessionBuilder::new(&environment)?.with_model_from_file(model_path)?;
        let input = session.inputs[0].dimensions();
        let mut dims = [1usize; 4];
        for (i, d) in input.iter().enumerate().take(4) {
            dims[i] = d.unwrap_or(1) as usize;
        }
        Ok(Self { session, input_dims: dims })
    }

    pub fn detect(&self, frame: &Mat) -> Result<Vec<[f32; 6]>> {
        let width = self.input_dims[3] as i32;
        let height = self.input_dims[2] as i32;
        let mut resized = Mat::default();
        imgproc::resize(frame, &mut resized, Size::new(width, height), 0.0, 0.0, imgproc::INTER_LINEAR)?;
        let mut rgb = Mat::default();
        imgproc::cvt_color(&resized, &mut rgb, imgproc::COLOR_BGR2RGB, 0)?;
        let total = (self.input_dims[1] * self.input_dims[2] * self.input_dims[3]) as usize;
        let data = rgb.data_bytes()?;
        let vec: Vec<f32> = data.iter().map(|b| *b as f32 / 255.0).collect();
        let input_array = Array4::from_shape_vec(self.input_dims, vec)?;
        let input = Value::from_array(self.session.allocator(), &input_array)?;
        let outputs: Vec<ort::Tensor<f32>> = self.session.run(vec![input])?;
        let first = outputs.get(0).unwrap();
        let view = first.view();
        let mut detections = Vec::new();
        for row in view.outer_iter() {
            if row.len() >= 6 {
                let arr = [row[0], row[1], row[2], row[3], row[4], row[5]];
                detections.push(arr);
            }
        }
        Ok(detections)
    }
}
