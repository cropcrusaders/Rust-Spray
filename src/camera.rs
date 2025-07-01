//! Camera abstraction for different camera backends
//!
//! This module provides a unified interface for both OpenCV-based cameras
//! (USB cameras, video files) and Raspberry Pi camera modules.

use opencv::{
    prelude::*,
    videoio::{self, VideoCapture},
};
use thiserror::Error;

#[cfg(feature = "picam")]
mod picam;

#[derive(Error, Debug)]
pub enum CameraError {
    #[error("OpenCV error: {0}")]
    OpenCv(#[from] opencv::Error),
    #[error("Failed to open camera: {0}")]
    Open(String),
    #[error("Failed to capture frame")]
    Capture,
    #[error("Camera feature not available: {0}")]
    FeatureNotAvailable(String),
}

/// Camera backend enumeration
pub enum CameraBackend {
    OpenCv(VideoCapture),
    #[cfg(feature = "picam")]
    Pi(picam::Picam),
}

/// Unified camera interface
pub struct Camera {
    backend: CameraBackend,
}

impl Camera {
    /// Create a new camera instance
    ///
    /// # Arguments
    /// * `device` - Device path or index (e.g., "/dev/video0" or "0")
    /// * `width` - Desired frame width
    /// * `height` - Desired frame height
    /// * `use_rpi` - Whether to use Raspberry Pi camera backend
    ///
    /// # Returns
    /// * `Result<Self, CameraError>` - New camera instance or error
    pub fn new(device: &str, width: u32, height: u32, use_rpi: bool) -> Result<Self, CameraError> {
        if use_rpi {
            #[cfg(feature = "picam")]
            {
                let cam = picam::Picam::new(device, width, height)
                    .map_err(|e| CameraError::Open(format!("Pi camera: {}", e)))?;
                return Ok(Camera {
                    backend: CameraBackend::Pi(cam),
                });
            }
            #[cfg(not(feature = "picam"))]
            {
                return Err(CameraError::FeatureNotAvailable(
                    "picam feature not enabled".to_string(),
                ));
            }
        }

        // Fallback to OpenCV backend
        let mut capture = if let Ok(index) = device.parse::<i32>() {
            videoio::VideoCapture::new(index, videoio::CAP_ANY)?
        } else {
            videoio::VideoCapture::from_file(device, videoio::CAP_ANY)?
        };

        if !capture.is_opened()? {
            return Err(CameraError::Open(format!(
                "Failed to open camera: {}",
                device
            )));
        }

        // Set resolution if it's a live camera (not a file)
        if device.parse::<i32>().is_ok() {
            let _ = capture.set(videoio::CAP_PROP_FRAME_WIDTH, width as f64);
            let _ = capture.set(videoio::CAP_PROP_FRAME_HEIGHT, height as f64);
        }

        Ok(Camera {
            backend: CameraBackend::OpenCv(capture),
        })
    }

    /// Capture a frame from the camera
    ///
    /// # Returns
    /// * `Result<Mat, CameraError>` - Captured frame or error
    pub fn capture(&mut self) -> Result<Mat, CameraError> {
        match &mut self.backend {
            CameraBackend::OpenCv(cap) => {
                let mut frame = Mat::default();
                cap.read(&mut frame)?;
                if frame.empty() {
                    return Err(CameraError::Capture);
                }
                Ok(frame)
            }
            #[cfg(feature = "picam")]
            CameraBackend::Pi(cam) => cam
                .capture()
                .map_err(|e| CameraError::Open(format!("Pi camera capture: {}", e))),
        }
    }

    /// Get camera properties (if supported by backend)
    ///
    /// # Returns
    /// * `Option<(u32, u32)>` - Width and height if available
    pub fn get_resolution(&self) -> Option<(u32, u32)> {
        match &self.backend {
            CameraBackend::OpenCv(cap) => {
                if let (Ok(w), Ok(h)) = (
                    cap.get(videoio::CAP_PROP_FRAME_WIDTH),
                    cap.get(videoio::CAP_PROP_FRAME_HEIGHT),
                ) {
                    Some((w as u32, h as u32))
                } else {
                    None
                }
            }
            #[cfg(feature = "picam")]
            CameraBackend::Pi(_) => None, // Could be implemented if picam module supports it
        }
    }
}
