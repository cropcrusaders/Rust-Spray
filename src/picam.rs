//! Camera support for Raspberry Pi
//!
//! This module provides camera capture using the V4L2 interface
//! via the rscam crate for Raspberry Pi cameras.

#[cfg(all(feature = "opencv", feature = "picam", any(target_arch = "arm", target_arch = "aarch64")))]
use opencv::{
    core::{Vector},
    imgcodecs,
    prelude::*,
};
#[cfg(all(feature = "picam", any(target_arch = "arm", target_arch = "aarch64")))]
use rscam::{Camera as PiCamera, Config as PiConfig};
use std::error::Error;

#[cfg(all(feature = "picam", any(target_arch = "arm", target_arch = "aarch64")))]
pub struct Picam {
    camera: PiCamera,
}

#[cfg(all(feature = "picam", any(target_arch = "arm", target_arch = "aarch64")))]
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

    #[cfg(feature = "opencv")]
    pub fn capture(&mut self) -> Result<opencv::core::Mat, Box<dyn Error>> {
        let frame = self.camera.capture()?;
        let data = Vector::from_slice(&frame);
        let mat = imgcodecs::imdecode(&data, imgcodecs::IMREAD_COLOR)?;
        Ok(mat)
    }
    
    #[cfg(not(feature = "opencv"))]
    pub fn capture(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let frame = self.camera.capture()?;
        Ok(frame.to_vec())
    }
}
