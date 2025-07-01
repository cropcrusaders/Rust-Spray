//! Basic usage example for Rust-Spray
//!
//! This example demonstrates how to use the Rust-Spray library components
//! programmatically instead of through the main application.

use rustspray::{Camera, Config, DetectionParams, GreenOnBrown, SprayController};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Load configuration
    let config = Config::load("config/config.toml")?;
    println!("Loaded configuration");

    // Initialize camera
    let mut camera = Camera::new(
        &config.camera.device,
        config.camera.resolution_width,
        config.camera.resolution_height,
        config.camera.use_rpi_cam,
    )?;
    println!("Camera initialized");

    // Initialize detection
    let detector = GreenOnBrown::new(&config.detection.algorithm)?;
    println!("Detection engine ready");

    // Initialize spray controller
    #[cfg(feature = "with-rppal")]
    let mut spray_controller = SprayController::new(config.spray.pins)?;
    #[cfg(not(feature = "with-rppal"))]
    let mut spray_controller = {
        println!("Warning: GPIO not available, using mock spray controller");
        SprayController::new(config.spray.pins)?
    };

    // Create detection parameters
    let detection_params = DetectionParams {
        exg_min: config.detection.exg_min,
        exg_max: config.detection.exg_max,
        hue_min: config.detection.hue_min,
        hue_max: config.detection.hue_max,
        brightness_min: config.detection.brightness_min,
        brightness_max: config.detection.brightness_max,
        saturation_min: config.detection.saturation_min,
        saturation_max: config.detection.saturation_max,
        min_area: config.detection.min_area,
        invert_hue: config.detection.invert_hue,
        algorithm: config.detection.algorithm.clone(),
    };

    // Process a few frames
    for frame_num in 1..=10 {
        println!("Processing frame {}", frame_num);

        // Capture frame
        let frame = camera.capture()?;

        // Run detection
        let result = detector.detect(&frame, &detection_params, false, "WEED")?;

        if !result.centers.is_empty() {
            println!("Found {} weeds", result.centers.len());

            // Spray for the configured duration
            let duration = Duration::from_millis(config.spray.activation_duration_ms as u64);
            spray_controller.pulse_all(duration);
            println!("Sprayed!");
        } else {
            println!("No weeds detected");
        }

        // Small delay between frames
        std::thread::sleep(Duration::from_millis(100));
    }

    println!("Example completed");
    Ok(())
}
