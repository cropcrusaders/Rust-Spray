//! Basic usage example for Rust-Spray
//!
//! This example demonstrates how to use the Rust-Spray library components
//! programmatically instead of through the main application.

#[cfg(feature = "opencv")]
use rustspray::{Camera, Config, DetectionParams, GreenOnBrown, SprayController};
#[cfg(not(feature = "opencv"))]
use rustspray::{Config, SprayController};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    #[cfg(feature = "opencv")]
    {
        run_with_opencv()
    }
    
    #[cfg(not(feature = "opencv"))]
    {
        run_without_opencv()
    }
}

#[cfg(feature = "opencv")]
fn run_with_opencv() -> Result<(), Box<dyn std::error::Error>> {
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

#[cfg(not(feature = "opencv"))]
fn run_without_opencv() -> Result<(), Box<dyn std::error::Error>> {
    println!("OpenCV feature not enabled - running basic configuration test");
    
    // Load configuration to test basic functionality
    let config = Config::load("config/config.toml").unwrap_or_else(|_| {
        println!("No config file found, using default configuration test");
        // Create a default config for testing
        return rustspray::Config {
            camera: rustspray::config::CameraConfig {
                device: "0".to_string(),
                resolution_width: 640,
                resolution_height: 480,
                use_rpi_cam: false,
            },
            detection: rustspray::config::DetectionConfig {
                algorithm: "ExG".to_string(),
                exg_min: 25,
                exg_max: 255,
                hue_min: 60,
                hue_max: 80,
                brightness_min: 60,
                brightness_max: 255,
                saturation_min: 50,
                saturation_max: 255,
                min_area: 500.0,
                invert_hue: false,
            },
            spray: rustspray::config::SprayConfig {
                pins: [18, 19, 20, 21],
                activation_duration_ms: 500,
            },
        };
    });

    println!("Configuration loaded (camera: {})", config.camera.device);

    // Test spray controller (without GPIO it will be a mock)
    let mut spray_controller = SprayController::new(config.spray.pins)?;
    println!("Mock spray controller initialized with {} sprayers", spray_controller.sprayer_count());

    // Simulate some spray activity
    spray_controller.pulse_all(Duration::from_millis(100));
    println!("Mock spray test completed");

    println!("Example completed (without OpenCV)");
    Ok(())
}
