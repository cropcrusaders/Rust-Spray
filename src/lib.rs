//! Rust-Spray: A camera-based precision spraying system for agriculture
//!
//! This crate provides a complete system for detecting weeds using computer vision
//! and controlling sprayer hardware via GPIO pins.

#[cfg(feature = "picam")]
pub mod picam;

pub mod camera;
pub mod config;
pub mod detection;
pub mod gps;
pub mod logging;
pub mod spray;
pub mod utils;

// Re-export the main types for easier usage
pub use camera::{Camera, CameraError};
pub use config::{Config, ConfigError};
pub use detection::{DetectionParams, DetectionResult, GreenOnBrown};
pub use gps::{GpsController, GpsCoordinate};
pub use logging::{WeedDetectionLogger, WeedDetectionEvent, LoggingConfig, LogFormat};
pub use spray::{SprayController, SprayError};
