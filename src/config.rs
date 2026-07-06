//! Runtime configuration for the spray pipeline.
//!
//! Loads settings from a TOML file. All sections use `#[serde(default)]`
//! so the file can be partial or missing entirely — sensible defaults
//! matching the values in [`crate::vision::PlantVision`] and the
//! `four_lane` example are used when keys are absent.

use serde::Deserialize;
use std::path::Path;

/// Top-level configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub camera: CameraConfig,
    pub vision: VisionConfig,
    pub lanes: LanesConfig,
    pub gpio: GpioConfig,
    pub logging: LoggingConfig,
}

/// Camera / frame-input settings.
///
/// The binary itself reads raw RGB24 frames from stdin. These values
/// tell it the expected frame dimensions and are also read by the
/// `rustspray-camera` helper script to configure the capture tool.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct CameraConfig {
    /// Frame width in pixels.
    pub width: usize,
    /// Frame height in pixels.
    pub height: usize,
    /// Target frames per second.
    pub fps: u32,
    /// Fail safe if the camera stops delivering frames: exit (nozzles
    /// off, systemd restarts the service) after this many seconds
    /// without a complete frame. `0` disables stall detection.
    pub stall_timeout_secs: u64,
    /// Camera backend for the helper script: `"v4l2"` or `"libcamera"`.
    pub backend: String,
    /// V4L2 device path (used when `backend = "v4l2"`).
    pub device: String,
}

/// PlantVision tuning parameters.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct VisionConfig {
    pub exg_threshold: i16,
    pub green_ratio_floor: f32,
    pub chroma_floor: f32,
    pub weights: VisionWeights,
}

/// Fusion weights for the multi-cue scorer.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct VisionWeights {
    pub exg: f32,
    pub green_ratio: f32,
    pub chroma: f32,
    pub bias: f32,
}

/// Lane-reduction settings.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LanesConfig {
    /// Number of spray lanes.
    pub count: usize,
    /// Coverage ratio to turn a lane **on**.
    pub on_threshold: f32,
    /// Coverage ratio to turn a lane **off** (hysteresis).
    pub off_threshold: f32,
}

/// GPIO pin configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct GpioConfig {
    /// BCM pin numbers, one per lane, controlling the relay/MOSFET for
    /// each nozzle solenoid.
    pub pins: Vec<u8>,
    /// Force mock GPIO even when compiled with real hardware support.
    pub mock: bool,
}

/// Logging configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level filter: `trace`, `debug`, `info`, `warn`, or `error`.
    pub level: String,
}

// ---------------------------------------------------------------------------
// Defaults — mirror the values from PlantVision::default() and the example.
// ---------------------------------------------------------------------------

impl Default for Config {
    fn default() -> Self {
        Self {
            camera: CameraConfig::default(),
            vision: VisionConfig::default(),
            lanes: LanesConfig::default(),
            gpio: GpioConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            fps: 30,
            stall_timeout_secs: 10,
            backend: "v4l2".to_string(),
            device: "/dev/video0".to_string(),
        }
    }
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            exg_threshold: 20,
            green_ratio_floor: 0.36,
            chroma_floor: 0.08,
            weights: VisionWeights::default(),
        }
    }
}

impl Default for VisionWeights {
    fn default() -> Self {
        Self {
            exg: 0.5,
            green_ratio: 0.35,
            chroma: 0.15,
            bias: 0.0,
        }
    }
}

impl Default for LanesConfig {
    fn default() -> Self {
        Self {
            count: 4,
            on_threshold: 0.30,
            off_threshold: 0.15,
        }
    }
}

impl Default for GpioConfig {
    fn default() -> Self {
        Self {
            pins: vec![17, 27, 22, 23],
            mock: false,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file.
    ///
    /// A missing file yields the defaults (with a note on stderr) so the
    /// binary stays usable for testing. A file that exists but cannot be
    /// read or parsed is an **error** — silently falling back to default
    /// GPIO pins on a misconfigured sprayer could actuate the wrong
    /// hardware.
    pub fn load(path: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content)
                .map_err(|e| format!("failed to parse {}: {}", path.display(), e)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                eprintln!("Config file {} not found, using defaults", path.display());
                Ok(Self::default())
            }
            Err(e) => Err(format!("failed to read {}: {}", path.display(), e)),
        }
    }

    /// Check invariants the pipeline relies on, returning a description
    /// of the first problem found.
    pub fn validate(&self) -> Result<(), String> {
        if self.camera.width == 0 || self.camera.height == 0 {
            return Err("camera.width and camera.height must be non-zero".into());
        }
        if self.camera.fps == 0 {
            return Err("camera.fps must be non-zero".into());
        }
        if self.lanes.count == 0 {
            return Err("lanes.count must be non-zero".into());
        }
        if self.camera.width < self.lanes.count {
            return Err(format!(
                "camera.width ({}) must be >= lanes.count ({})",
                self.camera.width, self.lanes.count,
            ));
        }
        if self.lanes.on_threshold < self.lanes.off_threshold {
            return Err(format!(
                "lanes.on_threshold ({}) must be >= lanes.off_threshold ({}) for hysteresis",
                self.lanes.on_threshold, self.lanes.off_threshold,
            ));
        }
        if self.gpio.pins.len() != self.lanes.count {
            return Err(format!(
                "gpio.pins has {} entries but lanes.count is {} — one pin per lane required",
                self.gpio.pins.len(),
                self.lanes.count,
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let cfg = Config::default();
        assert_eq!(cfg.camera.width, 640);
        assert_eq!(cfg.camera.height, 480);
        assert_eq!(cfg.lanes.count, 4);
        assert_eq!(cfg.gpio.pins.len(), 4);
    }

    #[test]
    fn partial_toml_fills_defaults() {
        let input = r#"
[lanes]
count = 6
"#;
        let cfg: Config = toml::from_str(input).unwrap();
        assert_eq!(cfg.lanes.count, 6);
        // Everything else is default.
        assert_eq!(cfg.camera.width, 640);
        assert_eq!(cfg.vision.exg_threshold, 20);
    }

    #[test]
    fn full_toml_round_trip() {
        let input = r#"
[camera]
width = 320
height = 240
fps = 15
stall_timeout_secs = 7
backend = "libcamera"
device = "/dev/video1"

[vision]
exg_threshold = 30
green_ratio_floor = 0.40
chroma_floor = 0.10

[vision.weights]
exg = 0.6
green_ratio = 0.25
chroma = 0.15
bias = 0.05

[lanes]
count = 6
on_threshold = 0.35
off_threshold = 0.20

[gpio]
pins = [5, 6, 13, 19, 26, 21]
mock = true

[logging]
level = "debug"
"#;
        let cfg: Config = toml::from_str(input).unwrap();
        assert_eq!(cfg.camera.width, 320);
        assert_eq!(cfg.camera.stall_timeout_secs, 7);
        assert_eq!(cfg.camera.backend, "libcamera");
        assert_eq!(cfg.vision.exg_threshold, 30);
        assert!((cfg.vision.weights.bias - 0.05).abs() < f32::EPSILON);
        assert_eq!(cfg.lanes.count, 6);
        assert_eq!(cfg.gpio.pins, vec![5, 6, 13, 19, 26, 21]);
        assert!(cfg.gpio.mock);
        assert_eq!(cfg.logging.level, "debug");
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn missing_file_yields_defaults() {
        let cfg = Config::load(Path::new("/nonexistent/rustspray-test.toml")).unwrap();
        assert_eq!(cfg.camera.width, 640);
    }

    #[test]
    fn validate_rejects_pin_lane_mismatch() {
        let cfg: Config = toml::from_str(
            r#"
[lanes]
count = 6
"#,
        )
        .unwrap();
        // Default has 4 pins but 6 lanes were requested.
        let err = cfg.validate().unwrap_err();
        assert!(err.contains("gpio.pins"), "unexpected error: {err}");
    }

    #[test]
    fn validate_rejects_zero_fps() {
        let cfg: Config = toml::from_str(
            r#"
[camera]
fps = 0
"#,
        )
        .unwrap();
        assert!(cfg.validate().is_err());
    }
}
