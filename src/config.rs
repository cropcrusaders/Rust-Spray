use serde::Deserialize;
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse configuration: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Deserialize)]
pub struct CameraConfig {
    pub device: String,
    pub resolution_width: u32,
    pub resolution_height: u32,
    /// Use the Raspberry Pi camera via V4L2
    #[serde(default)]
    pub use_rpi_cam: bool,
}

#[derive(Deserialize)]
pub struct DetectionConfig {
    pub algorithm: String,
    pub exg_min: i32,
    pub exg_max: i32,
    pub hue_min: i32,
    pub hue_max: i32,
    pub brightness_min: i32,
    pub brightness_max: i32,
    pub saturation_min: i32,
    pub saturation_max: i32,
    pub min_area: f64,
    pub invert_hue: bool,
}

#[derive(Deserialize)]
pub struct SprayConfig {
    pub pins: [u8; 4],
    /// Sprayer activation time in **milliseconds**
    pub activation_duration_ms: u32,
}

#[derive(Deserialize)]
pub struct Config {
    pub camera: CameraConfig,
    pub detection: DetectionConfig,
    pub spray: SprayConfig,
}

impl Config {
    /// Load configuration from the given TOML file path.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if the file cannot be read or parsed.
    pub fn load(path: &str) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }
}
