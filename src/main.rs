//! Production binary for the Rust-Spray pipeline.
//!
//! Reads raw RGB24 frames from stdin (piped from a camera capture tool)
//! or generates synthetic test frames, runs the vegetation detection
//! pipeline, and drives GPIO pins to control spray nozzles.

use log::{error, info, warn};
use rustspray::{
    config::Config,
    io_gpio::{MockGpio, NozzleControl},
    lanes::LaneReducer,
    pipeline::Pipeline,
    vision::PlantVision,
};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return;
    }

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("rustspray {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Load configuration.
    let config_path = get_arg_value(&args, "--config")
        .or_else(|| get_arg_value(&args, "-c"))
        .unwrap_or_else(|| "/etc/rustspray/config.toml".to_string());
    let config = Config::load(std::path::Path::new(&config_path));

    // Initialise logging.
    let log_level = get_arg_value(&args, "--log-level").unwrap_or(config.logging.level.clone());
    std::env::set_var("RUST_LOG", &log_level);
    env_logger::init();

    info!("rustspray {} starting", env!("CARGO_PKG_VERSION"));
    info!(
        "config: {}x{} @ {} fps, {} lanes",
        config.camera.width, config.camera.height, config.camera.fps, config.lanes.count,
    );

    let mock_gpio = args.iter().any(|a| a == "--mock-gpio") || config.gpio.mock;
    let test_pattern = args.iter().any(|a| a == "--test-pattern");
    let oneshot = args.iter().any(|a| a == "--oneshot");
    let max_frames: u64 = get_arg_value(&args, "--frames")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    // Graceful shutdown on SIGINT / SIGTERM.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("failed to install signal handler");

    // Build pipeline components.
    let gpio: Box<dyn NozzleControl> = if mock_gpio {
        info!("using mock GPIO (stdout)");
        Box::new(MockGpio::default())
    } else {
        build_real_gpio(&config)
    };

    let vision = PlantVision::new(
        config.vision.exg_threshold,
        config.vision.green_ratio_floor,
        config.vision.chroma_floor,
        (
            config.vision.weights.exg,
            config.vision.weights.green_ratio,
            config.vision.weights.chroma,
            config.vision.weights.bias,
        ),
    );

    let reducer = LaneReducer::new(
        config.lanes.count,
        config.lanes.on_threshold,
        config.lanes.off_threshold,
    );

    let w = config.camera.width;
    let h = config.camera.height;
    let mut pipeline = Pipeline::new(reducer, gpio, vision, w, h);

    let frame_size = w * h * 3;
    let frame_interval = Duration::from_secs_f64(1.0 / config.camera.fps as f64);

    info!("pipeline ready — frame size {} bytes", frame_size);

    if test_pattern {
        info!("running with test pattern");
        run_test_pattern(
            &mut pipeline,
            w,
            h,
            max_frames,
            oneshot,
            &running,
            frame_interval,
        );
    } else {
        // Detect whether stdin is a pipe/file or a terminal.
        use std::io::IsTerminal;
        if std::io::stdin().is_terminal() {
            error!("no input source: stdin is a terminal");
            eprintln!("Pipe camera frames into stdin or use --test-pattern. See --help.");
            std::process::exit(1);
        }
        info!("reading RGB24 frames from stdin");
        run_stdin(&mut pipeline, frame_size, max_frames, &running);
    }

    info!("shutdown complete");
}

// ---------------------------------------------------------------------------
// Frame sources
// ---------------------------------------------------------------------------

fn run_test_pattern(
    pipeline: &mut Pipeline,
    width: usize,
    height: usize,
    max_frames: u64,
    oneshot: bool,
    running: &Arc<AtomicBool>,
    interval: Duration,
) {
    let mut frame = vec![0u8; width * height * 3];
    // Green in lanes 0 and 2 (quarters 1 and 3), soil elsewhere.
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;
            if x < width / 4 || (x >= width / 2 && x < 3 * width / 4) {
                frame[idx] = 20;
                frame[idx + 1] = 200;
                frame[idx + 2] = 20;
            } else {
                frame[idx] = 120;
                frame[idx + 1] = 90;
                frame[idx + 2] = 70;
            }
        }
    }

    let mut count: u64 = 0;
    while running.load(Ordering::SeqCst) {
        let start = Instant::now();
        pipeline.process(&frame);
        count += 1;

        let elapsed = start.elapsed();
        if count % 100 == 0 || count == 1 {
            info!("frame {}: {:.1} ms", count, elapsed.as_secs_f64() * 1000.0,);
        }

        if oneshot || (max_frames > 0 && count >= max_frames) {
            break;
        }

        if let Some(sleep) = interval.checked_sub(elapsed) {
            std::thread::sleep(sleep);
        }
    }
    info!("processed {} frames", count);
}

fn run_stdin(
    pipeline: &mut Pipeline,
    frame_size: usize,
    max_frames: u64,
    running: &Arc<AtomicBool>,
) {
    let mut stdin = std::io::stdin().lock();
    let mut buf = vec![0u8; frame_size];
    let mut count: u64 = 0;

    while running.load(Ordering::SeqCst) {
        match stdin.read_exact(&mut buf) {
            Ok(()) => {}
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    info!("end of input stream");
                } else {
                    error!("stdin read error: {}", e);
                }
                break;
            }
        }

        let start = Instant::now();
        pipeline.process(&buf);
        count += 1;

        let elapsed = start.elapsed();
        if count % 100 == 0 || count == 1 {
            info!("frame {}: {:.1} ms", count, elapsed.as_secs_f64() * 1000.0,);
        }

        if max_frames > 0 && count >= max_frames {
            break;
        }
    }
    info!("processed {} frames", count);
}

// ---------------------------------------------------------------------------
// GPIO construction
// ---------------------------------------------------------------------------

#[cfg(feature = "rpi")]
fn build_real_gpio(config: &Config) -> Box<dyn NozzleControl> {
    use rustspray::io_gpio::RppalGpio;
    info!("using real GPIO pins: {:?}", config.gpio.pins);
    Box::new(RppalGpio::new(&config.gpio.pins))
}

#[cfg(not(feature = "rpi"))]
fn build_real_gpio(_config: &Config) -> Box<dyn NozzleControl> {
    warn!("real GPIO unavailable (not compiled with --features rpi); falling back to mock");
    Box::new(MockGpio::default())
}

// ---------------------------------------------------------------------------
// CLI helpers
// ---------------------------------------------------------------------------

fn get_arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn print_usage() {
    println!(
        "\
rustspray {version} — agricultural spray pipeline

USAGE:
    rustspray [OPTIONS]
    <camera-source> | rustspray [OPTIONS]

OPTIONS:
    -c, --config <PATH>     Configuration file [default: /etc/rustspray/config.toml]
        --mock-gpio         Print lane states to stdout instead of driving GPIO
        --test-pattern      Use synthetic green/soil frames (no camera needed)
        --oneshot           Process one frame then exit
        --frames <N>        Stop after N frames (0 = unlimited)
        --log-level <LVL>   Override log level (trace/debug/info/warn/error)
    -h, --help              Print this help
    -V, --version           Print version

CAMERA SETUP:
    Pipe camera output as raw RGB24 into stdin. Examples:

    # Raspberry Pi Camera Module (CSI) via libcamera + ffmpeg
    rpicam-vid -t 0 --width 640 --height 480 --framerate 30 \\
               --codec yuv420 --nopreview -o - | \\
    ffmpeg -f rawvideo -pix_fmt yuv420p -s 640x480 -i - \\
           -f rawvideo -pix_fmt rgb24 pipe:1 | \\
    rustspray --config /etc/rustspray/config.toml

    # USB camera via ffmpeg + V4L2
    ffmpeg -f v4l2 -framerate 30 -video_size 640x480 \\
           -i /dev/video0 -f rawvideo -pix_fmt rgb24 pipe:1 | \\
    rustspray --config /etc/rustspray/config.toml

    # Dry-run without any camera
    rustspray --test-pattern --mock-gpio

    See the provided rustspray-camera helper script for automatic
    camera setup based on config.toml.",
        version = env!("CARGO_PKG_VERSION"),
    );
}
