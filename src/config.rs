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
    /// Enable/disable spraying (false for logging-only mode)
    #[serde(default = "default_spray_enabled")]
    pub enabled: bool,
}

/// Default value for spray enabled
fn default_spray_enabled() -> bool {
    true
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub camera: CameraConfig,
    pub detection: DetectionConfig,
    pub spray: SprayConfig,
    /// GPS configuration
    #[serde(default)]
    pub gps: GpsConfig,
    /// Data logging configuration
    #[serde(default)]
    pub logging: LoggingConfigToml,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GpsConfig {
    /// Enable GPS tracking
    #[serde(default)]
    pub enabled: bool,
    /// Base latitude for mock GPS (when hardware not available)
    #[serde(default = "default_latitude")]
    pub mock_latitude: f64,
    /// Base longitude for mock GPS (when hardware not available)
    #[serde(default = "default_longitude")]
    pub mock_longitude: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoggingConfigToml {
    /// Enable data logging
    #[serde(default = "default_logging_enabled")]
    pub enabled: bool,
    /// Output file prefix (extensions added automatically)
    #[serde(default = "default_output_file")]
    pub output_file: String,
    /// Output format: "json", "csv", or "both"
    #[serde(default = "default_format")]
    pub format: String,
    /// Auto-flush after each write
    #[serde(default = "default_auto_flush")]
    pub auto_flush: bool,
}

// Default value functions
fn default_latitude() -> f64 { 42.0 }
fn default_longitude() -> f64 { -93.5 }
fn default_logging_enabled() -> bool { true }
fn default_output_file() -> String { "weed_detections".to_string() }
fn default_format() -> String { "json".to_string() }
fn default_auto_flush() -> bool { true }

impl Default for GpsConfig {
    fn default() -> Self {
        Self {
            enabled: false, // GPS disabled by default
            mock_latitude: default_latitude(),
            mock_longitude: default_longitude(),
        }
    }
}

impl Default for LoggingConfigToml {
    fn default() -> Self {
        Self {
            enabled: default_logging_enabled(),
            output_file: default_output_file(),
            format: default_format(),
            auto_flush: default_auto_flush(),
        }
    }
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

impl LoggingConfigToml {
    /// Convert to the logging module's LoggingConfig
    pub fn to_logging_config(&self) -> crate::logging::LoggingConfig {
        use crate::logging::{LoggingConfig, LogFormat};
        
        let format = match self.format.as_str() {
            "csv" => LogFormat::Csv,
            "both" => LogFormat::Both,
            _ => LogFormat::Json, // Default to JSON
        };

        LoggingConfig {
            enabled: self.enabled,
            output_file: self.output_file.clone(),
            format,
            buffer_size: 1024,
            auto_flush: self.auto_flush,
        }
    }
}
