use opencv::{imgcodecs, core::{self, Vector}, prelude::*};
use rscam::{Camera as PiCamera, Config as PiConfig};
use std::error::Error;

pub struct Picam {
    camera: PiCamera,
}

impl Picam {
    pub fn new(device: &str, width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        let mut cam = PiCamera::new(device)?;
        cam.start(&PiConfig {
            interval: (1, 30),
            resolution: (width, height),
            format: b"MJPG",
            ..Default::default()
        })?;
        Ok(Picam { camera: cam })
    }

    pub fn capture(&mut self) -> Result<Mat, Box<dyn Error>> {
        let frame = self.camera.capture()?;
        let data = Vector::from_slice(&frame);
        let mat = imgcodecs::imdecode(&data, imgcodecs::IMREAD_COLOR)?;
        Ok(mat)
    }
}
