use clap::Parser;
use log::info;
use opencv::highgui;
use std::error::Error;

// Declare project modules
mod camera;
mod config;
mod detection;
mod error;
mod spray;
mod utils;

// Import necessary items from modules
use camera::Camera;
use config::Config;
use detection::GreenOnBrown;
use spray::SprayController;

// Define command-line arguments
#[derive(Parser)]
struct Cli {
    /// Path to the configuration file
    #[arg(long, default_value = "config/config.toml")]
    config: String,

    /// Enable display of detection results
    #[arg(long)]
    show_display: bool,
}

/// Entry point of the RustSpray application
fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the logger for debugging and monitoring
    env_logger::init();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Load configuration from the specified file
    let config = Config::load(&cli.config)?;
    info!("Configuration loaded from {}", cli.config);

    // Initialize the camera with settings from the configuration
    let mut camera = Camera::new(&config.camera.device)?;
    info!("Camera initialized");

    // Initialize the weed detection mechanism
    let gob = GreenOnBrown::new(&config.detection.algorithm)?;
    info!("Detection algorithm '{}' initialized", config.detection.algorithm);

    // Initialize the spraying hardware
    let mut spray_controller = SprayController::new(config.spray.pins)?;
    info!("Spray controller initialized");

    // Set up the display window if show_display is enabled
    if cli.show_display {
        highgui::named_window("Detection", highgui::WINDOW_AUTOSIZE)?;
    }

    // Run the main processing loop
    run(&mut camera, &gob, &mut spray_controller, &config, cli.show_display)?;

    Ok(())
}

/// Runs the main loop for capturing images, detecting weeds, and spraying
fn run(
    camera: &mut Camera,
    gob: &GreenOnBrown,
    spray_controller: &mut SprayController,
    config: &Config,
    show_display: bool,
) -> Result<(), Box<dyn Error>> {
    loop {
        // Capture an image from the camera
        let image = camera.capture()?;
        info!("Image captured");

        // Perform weed detection on the captured image
        let (contours, boxes, weed_centres, image_out) = gob.inference(
            &image,
            config.detection.exg_min,
            config.detection.exg_max,
            config.detection.hue_min,
            config.detection.hue_max,
            config.detection.brightness_min,
            config.detection.brightness_max,
            config.detection.saturation_min,
            config.detection.saturation_max,
            config.detection.min_area,
            show_display,
            &config.detection.algorithm,
            config.detection.invert_hue,
            "WEED",
        )?;
        info!("Detection completed with {} weeds found", weed_centres.len());

        // Activate the sprayer if weeds are detected
        if !weed_centres.is_empty() {
            spray_controller.activate_all();
            std::thread::sleep(std::time::Duration::from_millis(config.spray.activation_duration_ms as u64));
            spray_controller.deactivate_all();
            info!("Sprayers activated");
        }

        // Display the detection results if enabled
        if show_display {
            highgui::imshow("Detection", &image_out)?;
            // Exit the loop if 'q' is pressed
            if highgui::wait_key(1)? == 'q' as i32 {
                break;
            }
        }
    }
    Ok(())
}
