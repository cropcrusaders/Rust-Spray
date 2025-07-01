use serde::Deserialize;
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse configuration: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("Invalid configuration: {0}")]
    Validation(String),
}

#[derive(Deserialize, Debug, Clone)]
pub struct CameraConfig {
    pub device: String,
    pub resolution_width: u32,
    pub resolution_height: u32,
    /// Use the Raspberry Pi camera via V4L2
    #[serde(default)]
    pub use_rpi_cam: bool,
}

#[derive(Deserialize, Debug, Clone)]
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
    #[serde(default)]
    pub invert_hue: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SprayConfig {
    pub pins: [u8; 4],
    /// Sprayer activation time in **milliseconds**
    pub activation_duration_ms: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub camera: CameraConfig,
    pub detection: DetectionConfig,
    pub spray: SprayConfig,
}

impl Config {
    /// Load configuration from a TOML file
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// * `Result<Self, ConfigError>` - Loaded configuration or error
    pub fn load(path: &str) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration for common issues
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate camera resolution
        if self.camera.resolution_width == 0 || self.camera.resolution_height == 0 {
            return Err(ConfigError::Validation(
                "Camera resolution must be greater than 0".to_string(),
            ));
        }

        // Validate detection parameters
        if self.detection.min_area < 0.0 {
            return Err(ConfigError::Validation(
                "Minimum area must be non-negative".to_string(),
            ));
        }

        // Validate spray timing
        if self.spray.activation_duration_ms == 0 {
            return Err(ConfigError::Validation(
                "Spray activation duration must be greater than 0".to_string(),
            ));
        }

        // Check for supported algorithms
        let supported_algorithms = ["exg", "exgr", "maxg", "nexg", "gndvi", "hsv", "exhsv"];
        if !supported_algorithms.contains(&self.detection.algorithm.as_str()) {
            return Err(ConfigError::Validation(format!(
                "Unsupported algorithm '{}'. Supported: {:?}",
                self.detection.algorithm, supported_algorithms
            )));
        }

        Ok(())
    }
}
