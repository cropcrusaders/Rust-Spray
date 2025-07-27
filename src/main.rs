//! Rust-Spray: Camera-based precision spraying system
//!
//! This application captures video frames, detects weeds using computer vision,
//! and controls sprayer hardware via GPIO pins.

use std::error::Error;
use std::process;

use clap::Parser;
use log::{error, info, warn};
use opencv::highgui;

// ─── Project modules ────────────────────────────────────────────────────────
mod camera;
mod config;
mod detection;
mod gps;
mod logging;
mod spray;
mod utils;

use camera::{Camera, CameraError};
use config::{Config, ConfigError};
use detection::{DetectionParams, GreenOnBrown};
use gps::GpsController;
use logging::{WeedDetectionLogger, DetectionInfo, ActionTaken};
use spray::{SprayController, SprayError};

// ─── Error handling ─────────────────────────────────────────────────────────

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("Camera error: {0}")]
    Camera(#[from] CameraError),
    #[error("Detection error: {0}")]
    Detection(#[from] opencv::Error),
    #[error("Spray controller error: {0}")]
    Spray(#[from] SprayError),
    #[error("GPS error: {0}")]
    Gps(#[from] gps::GpsError),
    #[error("Logging error: {0}")]
    Logging(#[from] logging::LoggingError),
    #[error("Application error: {0}")]
    General(String),
}

type Result<T> = std::result::Result<T, AppError>;

// ─── CLI args ───────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "rustspray",
    about = "A Rust-based camera-and-sprayer system for precision weed control",
    version
)]
struct Cli {
    /// Path to the configuration file
    #[arg(long, default_value = "config/config.toml")]
    config: String,

    /// Display the annotated video stream
    #[arg(long)]
    show_display: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Enable weed locator mode (logging only, no spraying)
    #[arg(long)]
    locator_mode: bool,

    /// Override output file for detections
    #[arg(long)]
    output_file: Option<String>,
}

// ─── Main function ──────────────────────────────────────────────────────────

fn main() {
    // Initialize the application and handle any errors gracefully
    if let Err(e) = run() {
        error!("Application error: {}", e);

        // Print the error chain
        let mut source = e.source();
        while let Some(err) = source {
            error!("  Caused by: {}", err);
            source = err.source();
        }

        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    info!("Starting Rust-Spray v{}", env!("CARGO_PKG_VERSION"));

    // 1. Load configuration
    let mut config = Config::load(&cli.config)?;
    info!("Configuration loaded from {}", cli.config);

    // Override spray mode if locator_mode is enabled
    if cli.locator_mode {
        config.spray.enabled = false;
        info!("Locator mode enabled - spraying disabled");
    }

    // Override output file if specified
    if let Some(output_file) = &cli.output_file {
        config.logging.output_file = output_file.clone();
        info!("Output file overridden to: {}", output_file);
    }

    // 2. Initialize camera
    let mut camera = Camera::new(
        &config.camera.device,
        config.camera.resolution_width,
        config.camera.resolution_height,
        config.camera.use_rpi_cam,
    )?;

    if let Some((w, h)) = camera.get_resolution() {
        info!("Camera initialized: {}x{}", w, h);
    } else {
        info!("Camera initialized: {}", config.camera.device);
    }

    // 3. Initialize detection
    let detector = GreenOnBrown::new(&config.detection.algorithm)?;
    info!("Detection engine ready: {}", config.detection.algorithm);

    // 4. Initialize spray controller
    let mut spray_controller = SprayController::new(config.spray.pins)?;
    if config.spray.enabled {
        info!(
            "Spray controller ready: {} sprayers",
            spray_controller.sprayer_count()
        );
    } else {
        info!("Spray controller initialized but spraying disabled");
    }

    // 5. Initialize GPS controller
    let mut gps_controller = if config.gps.enabled {
        info!("GPS enabled - using coordinates from GPS");
        GpsController::new_mock(config.gps.mock_latitude, config.gps.mock_longitude)
    } else {
        info!("GPS disabled - using mock coordinates");
        GpsController::new_default()
    };

    // 6. Initialize data logger
    let mut logger = WeedDetectionLogger::new(config.logging.to_logging_config())?;
    if logger.is_enabled() {
        info!("Weed detection logging enabled");
    } else {
        info!("Weed detection logging disabled");
    }

    // 7. Optional display window
    if cli.show_display {
        match highgui::named_window("Rust-Spray Detection", highgui::WINDOW_AUTOSIZE) {
            Ok(_) => info!("Display window created"),
            Err(e) => warn!("Failed to create display window: {}", e),
        }
    }

    // 8. Main processing loop
    process_frames(
        &mut camera,
        &detector,
        &mut spray_controller,
        &mut gps_controller,
        &mut logger,
        &config,
        cli.show_display,
    )
}

// ─── Processing loop ────────────────────────────────────────────────────────

fn process_frames(
    camera: &mut Camera,
    detector: &GreenOnBrown,
    spray_controller: &mut SprayController,
    gps_controller: &mut GpsController,
    logger: &mut WeedDetectionLogger,
    config: &Config,
    show_display: bool,
) -> Result<()> {
    info!("Starting main processing loop");
    
    if config.spray.enabled {
        info!("Mode: Spray and Log");
    } else {
        info!("Mode: Open Weed Locator (Log Only)");
    }

    // Create detection parameters from config
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

    let spray_duration =
        std::time::Duration::from_millis(config.spray.activation_duration_ms as u64);

    let mut frame_count = 0;
    let start_time = std::time::Instant::now();

    loop {
        // Capture frame
        let frame = match camera.capture() {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to capture frame: {}", e);
                continue;
            }
        };

        frame_count += 1;

        // Get current GPS location (if enabled)
        let current_location = if config.gps.enabled || logger.is_enabled() {
            match gps_controller.get_location() {
                Ok(loc) => Some(loc),
                Err(e) => {
                    warn!("GPS read failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Run detection
        let detection_result = detector.detect(&frame, &detection_params, show_display, "WEED")?;
        let weed_count = detection_result.centers.len();

        if weed_count > 0 {
            info!("Detected {} weeds in frame {}", weed_count, frame_count);

            // Process each detected weed
            for (i, center) in detection_result.centers.iter().enumerate() {
                let bounding_box = if i < detection_result.bounding_boxes.len() {
                    detection_result.bounding_boxes[i]
                } else {
                    [center[0] - 10, center[1] - 10, 20, 20] // Default box if missing
                };

                let detection_info = DetectionInfo {
                    algorithm: config.detection.algorithm.clone(),
                    center_x: center[0],
                    center_y: center[1],
                    bounding_box,
                    area: config.detection.min_area, // Could calculate actual area from contours
                    confidence: None, // Could be added to detection algorithms
                    frame_number: frame_count,
                };

                let action_taken = if config.spray.enabled {
                    // Activate sprayers
                    spray_controller.pulse_all(spray_duration);
                    info!("Sprayed for {}ms", config.spray.activation_duration_ms);
                    
                    ActionTaken::SprayActivated {
                        duration_ms: config.spray.activation_duration_ms,
                        sprayers: config.spray.pins.to_vec(),
                    }
                } else {
                    ActionTaken::LoggedOnly
                };

                // Log the detection
                if let Err(e) = logger.log_detection(current_location.clone(), detection_info, action_taken) {
                    error!("Failed to log detection: {}", e);
                }
            }
        }

        // Optional display
        if show_display {
            match highgui::imshow("Rust-Spray Detection", &detection_result.annotated_frame) {
                Ok(_) => {}
                Err(e) => warn!("Display error: {}", e),
            }

            // Check for exit key
            match highgui::wait_key(1) {
                Ok(key) if key == 'q' as i32 || key == 27 => {
                    // 'q' or ESC
                    info!("Exit key pressed");
                    break;
                }
                Ok(_) => {}
                Err(e) => warn!("Key input error: {}", e),
            }
        }

        // Print periodic statistics
        if frame_count % 100 == 0 {
            let elapsed = start_time.elapsed();
            let fps = frame_count as f64 / elapsed.as_secs_f64();
            info!(
                "Processed {} frames at {:.1} FPS, logged {} detections",
                frame_count, fps, logger.event_count()
            );
        }
    }

    // Final flush of logger
    logger.flush()?;

    info!(
        "Processing completed. Total frames: {}, Total detections: {}",
        frame_count,
        logger.event_count()
    );
    Ok(())
}
