use opencv::{
    prelude::*,
    videoio::{self, VideoCapture},
};
use std::error::Error;

pub struct Camera {
    capture: VideoCapture,
}

impl Camera {
    pub fn new(device: &str) -> Result<Self, Box<dyn Error>> {
        let capture = match device.parse::<i32>() {
            Ok(index) => videoio::VideoCapture::new(index, videoio::CAP_ANY)?,
            Err(_) => videoio::VideoCapture::from_file(device, videoio::CAP_ANY)?,
        };
        if !capture.is_opened()? {
            return Err("Failed to open camera".into());
        }
        Ok(Camera { capture })
    }

    pub fn capture(&mut self) -> Result<Mat, Box<dyn Error>> {
        let mut frame = Mat::default();
        self.capture.read(&mut frame)?;
        if frame.empty() {
            return Err("Failed to capture image".into());
        }
        Ok(frame)
    }
}
