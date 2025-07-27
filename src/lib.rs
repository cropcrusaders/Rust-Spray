//! Rust-Spray: A camera-based precision spraying system for agriculture
//!
//! This crate provides a complete system for detecting weeds using computer vision
//! and controlling sprayer hardware via GPIO pins.

#[cfg(feature = "picam")]
pub mod picam;

#[cfg(feature = "opencv")]
pub mod camera;
pub mod config;
#[cfg(feature = "opencv")]
pub mod detection;
pub mod spray;
#[cfg(feature = "opencv")]
pub mod utils;

// Re-export the main types for easier usage
#[cfg(feature = "opencv")]
pub use camera::{Camera, CameraError};
pub use config::{Config, ConfigError};
#[cfg(feature = "opencv")]
pub use detection::{DetectionParams, DetectionResult, GreenOnBrown};
pub use spray::{SprayController, SprayError};
