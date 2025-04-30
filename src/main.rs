use clap::Parser;
use log::info;
use opencv::highgui;
use std::error::Error;

// ─── Project modules ────────────────────────────────────────────────────────
mod camera;
mod config;
mod detection;
mod error;
mod spray;
mod utils;

use camera::Camera;
use config::Config;
use detection::GreenOnBrown;
use spray::SprayController;

// ─── CLI args ───────────────────────────────────────────────────────────────
#[derive(Parser)]
struct Cli {
    /// Path to the configuration file
    #[arg(long, default_value = "config/config.toml")]
    config: String,

    /// Display the annotated video stream
    #[arg(long)]
    show_display: bool,
}

// ─── main ───────────────────────────────────────────────────────────────────
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = Cli::parse();

    // 1. Load config
    let config = Config::load(&cli.config)?;
    info!("Config loaded from {}", cli.config);

    // 2. Camera
    let mut camera = Camera::new(&config.camera.device)?;
    info!("Camera initialised");

    // 3. Detection
    let gob = GreenOnBrown::new(&config.detection.algorithm)?;
    info!("Detector '{}'", config.detection.algorithm);

    // 4. Sprayer
    let mut spray = SprayController::new(config.spray.pins)?;
    info!("Spray controller ready");

    // 5. Optional display window
    if cli.show_display {
        highgui::named_window("Detection", highgui::WINDOW_AUTOSIZE)?;
    }

    // 6. Main loop
    run(&mut camera, &gob, &mut spray, &config, cli.show_display)
}

// ─── processing loop ────────────────────────────────────────────────────────
fn run(
    camera: &mut Camera,
    gob: &GreenOnBrown,
    spray: &mut SprayController,
    cfg: &Config,
    show_display: bool,
) -> Result<(), Box<dyn Error>> {
    loop {
        // ── capture
        let frame = camera.capture()?;
        info!("Frame captured");

        // ── detect
        let (_contours, _boxes, centres, annotated) = gob.inference(
            &frame,
            cfg.detection.exg_min,
            cfg.detection.exg_max,
            cfg.detection.hue_min,
            cfg.detection.hue_max,
            cfg.detection.brightness_min,
            cfg.detection.brightness_max,
            cfg.detection.saturation_min,
            cfg.detection.saturation_max,
            cfg.detection.min_area,
            show_display,
            &cfg.detection.algorithm,
            cfg.detection.invert_hue,
            "WEED",
        )?;
        info!("Found {} weeds", centres.len());

        // ── spray if needed
        if !centres.is_empty() {
            spray.activate_all();
            std::thread::sleep(std::time::Duration::from_millis(
                cfg.spray.activation_duration_ms as u64,
            ));
            spray.deactivate_all();
            info!("Sprayers pulsed");
        }

        // ── optional display & exit key
        if show_display {
            highgui::imshow("Detection", &annotated)?;
            if highgui::wait_key(1)? == 'q' as i32 {
                break;
            }
        }
    }
    Ok(())
}
