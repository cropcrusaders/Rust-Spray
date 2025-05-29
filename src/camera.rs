use opencv::{
    prelude::*,
    videoio::{self, VideoCapture},
};
use std::error::Error;

#[cfg(feature = "picam")]
mod picam;

pub enum CameraBackend {
    OpenCv(VideoCapture),
    #[cfg(feature = "picam")]
    Pi(picam::Picam),
}

pub struct Camera {
    backend: CameraBackend,
}

impl Camera {
    pub fn new(device: &str, width: u32, height: u32, use_rpi: bool) -> Result<Self, Box<dyn Error>> {
        if use_rpi {
            #[cfg(feature = "picam")]
            {
                let cam = picam::Picam::new(device, width, height)?;
                return Ok(Camera { backend: CameraBackend::Pi(cam) });
            }
            #[cfg(not(feature = "picam"))]
            {
                return Err("picam feature not enabled".into());
            }
        }

        // Fallback to OpenCV backend
        let capture = if let Ok(index) = device.parse::<i32>() {
            videoio::VideoCapture::new(index, videoio::CAP_ANY)?
        } else {
            videoio::VideoCapture::from_file(device, videoio::CAP_ANY)?
        };

        if !capture.is_opened()? {
            return Err("Failed to open camera".into());
        }

        Ok(Camera { backend: CameraBackend::OpenCv(capture) })
    }

    pub fn capture(&mut self) -> Result<Mat, Box<dyn Error>> {
        match &mut self.backend {
            CameraBackend::OpenCv(cap) => {
                let mut frame = Mat::default();
                cap.read(&mut frame)?;
                if frame.empty() {
                    return Err("Failed to capture image".into());
                }
                Ok(frame)
            }
            #[cfg(feature = "picam")]
            CameraBackend::Pi(cam) => cam.capture(),
        }
    }
}
