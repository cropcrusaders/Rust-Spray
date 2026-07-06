//! Production binary for the Rust-Spray pipeline.
//!
//! Reads raw RGB24 frames from stdin (piped from a camera capture tool,
//! or framed by an outer shell in `--ipc-mode`), runs the vegetation
//! detection pipeline, and drives GPIO pins to control spray nozzles.

mod watchdog;

use clap::Parser;
use log::{error, info};
use rustspray_core::{
    config::Config,
    io_gpio::{MockGpio, NozzleControl},
    ipc,
    lanes::LaneReducer,
    pipeline::Pipeline,
    vision::PlantVision,
};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use watchdog::Watchdog;

/// Exit code when the camera stops delivering frames (stall fail-safe).
const EXIT_STALLED: i32 = 3;

const CAMERA_HELP: &str = "\
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
    camera setup based on config.toml.

IPC MODE:
    With --ipc-mode an outer shell (e.g. OpenWeedLocator) writes framed
    RGB24 to stdin — an 8-byte little-endian [width:u32][height:u32]
    header before each frame — and receives one JSON line per frame on
    stdout. See INTEGRATION.md for the full protocol contract.";

/// SIMD-accelerated vegetation detection and spray control.
#[derive(Parser, Debug)]
#[command(name = "rustspray", version, about, after_help = CAMERA_HELP)]
struct Cli {
    /// Configuration file
    #[arg(short, long, default_value = "/etc/rustspray/config.toml")]
    config: String,

    /// Skip GPIO hardware; log lane state changes to stderr instead
    #[arg(long)]
    mock_gpio: bool,

    /// Use synthetic green/soil frames (no camera needed)
    #[arg(long, conflicts_with = "ipc_mode")]
    test_pattern: bool,

    /// Process one frame then exit
    #[arg(long)]
    oneshot: bool,

    /// Stop after N frames (0 = unlimited)
    #[arg(long, default_value_t = 0)]
    frames: u64,

    /// Override log level (trace/debug/info/warn/error)
    #[arg(long)]
    log_level: Option<String>,

    /// Read framed RGB24 from stdin and write one JSON line per frame
    /// to stdout (protocol v1 — see INTEGRATION.md)
    #[arg(long)]
    ipc_mode: bool,

    /// Override the TOML [gpio] pins: comma-separated BCM numbers, one
    /// per lane (e.g. 27,22,23,24). Outer shells pass their own relay
    /// wiring here so both sides address the same solenoids by
    /// construction instead of by hand-synchronised config files.
    #[arg(long, value_delimiter = ',', value_name = "BCM,BCM,...")]
    gpio_pins: Option<Vec<u8>>,

    /// Print version and IPC protocol number as JSON, then exit
    #[arg(long)]
    output_version: bool,
}

fn main() {
    let cli = Cli::parse();

    // Startup handshake for outer shells: must work without a config
    // file and must print nothing else on stdout.
    if cli.output_version {
        println!(
            "{{\"rustspray_version\":\"{}\",\"ipc_protocol\":{}}}",
            env!("CARGO_PKG_VERSION"),
            ipc::IPC_PROTOCOL_VERSION,
        );
        return;
    }

    // Load configuration.
    let mut config = match Config::load(std::path::Path::new(&cli.config)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(2);
        }
    };
    // The pin override must land before validation so a lane/pin count
    // mismatch from the outer shell is a hard startup error too.
    if let Some(pins) = cli.gpio_pins.clone() {
        config.gpio.pins = pins;
    }
    if let Err(e) = config.validate() {
        eprintln!("Error: invalid configuration: {e}");
        std::process::exit(2);
    }

    // Initialise logging. Precedence: --log-level flag > RUST_LOG env
    // > config file.
    let mut log_builder = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(&config.logging.level),
    );
    if let Some(level) = &cli.log_level {
        log_builder.parse_filters(level);
    }
    log_builder.init();

    info!("rustspray {} starting", env!("CARGO_PKG_VERSION"));
    info!(
        "config: {}x{} @ {} fps, {} lanes",
        config.camera.width, config.camera.height, config.camera.fps, config.lanes.count,
    );

    let mock_gpio = cli.mock_gpio || config.gpio.mock;

    // Graceful shutdown on SIGINT / SIGTERM.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("failed to install signal handler");

    // Build pipeline components.
    let gpio: Box<dyn NozzleControl> = if mock_gpio {
        info!("using mock GPIO (stderr)");
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

    let mut watchdog = Watchdog::new();

    if cli.ipc_mode {
        info!(
            "IPC mode: framed RGB24 on stdin, JSON v{} on stdout",
            ipc::IPC_PROTOCOL_VERSION,
        );
        let exit_code = run_ipc(
            vision,
            reducer,
            gpio,
            cli.frames,
            cli.oneshot,
            &running,
            &mut watchdog,
        );
        info!("all nozzles off — shutdown complete");
        std::process::exit(exit_code);
    }

    let w = config.camera.width;
    let h = config.camera.height;
    let mut pipeline = Pipeline::new(reducer, gpio, vision, w, h);

    let frame_size = w * h * 3;
    let frame_interval = Duration::from_secs_f64(1.0 / config.camera.fps as f64);
    let stall_timeout = Duration::from_secs(config.camera.stall_timeout_secs);

    info!("pipeline ready — frame size {} bytes", frame_size);

    let stalled = if cli.test_pattern {
        info!("running with test pattern");
        run_test_pattern(
            &mut pipeline,
            w,
            h,
            cli.frames,
            cli.oneshot,
            &running,
            frame_interval,
            &mut watchdog,
        );
        false
    } else {
        // Detect whether stdin is a pipe/file or a terminal.
        use std::io::IsTerminal;
        if std::io::stdin().is_terminal() {
            error!("no input source: stdin is a terminal");
            eprintln!("Pipe camera frames into stdin or use --test-pattern. See --help.");
            std::process::exit(1);
        }
        info!(
            "reading RGB24 frames from stdin (stall timeout: {})",
            if stall_timeout.is_zero() {
                "disabled".to_string()
            } else {
                format!("{} s", stall_timeout.as_secs())
            },
        );
        run_stdin(
            &mut pipeline,
            frame_size,
            cli.frames,
            &running,
            stall_timeout,
            &mut watchdog,
        )
    };

    // Fail safe: never exit with a valve left open.
    pipeline.all_off();
    info!("all nozzles off — shutdown complete");
    if stalled {
        std::process::exit(EXIT_STALLED);
    }
}

// ---------------------------------------------------------------------------
// Frame sources
// ---------------------------------------------------------------------------

/// IPC inner-loop: framed frames in, JSON lane states out.
///
/// The outer shell owns pacing and timeouts, so a blocking read on the
/// main thread is correct here — when the shell shuts down it closes our
/// stdin and the read returns EOF. Every exit path forces all lanes off.
///
/// Returns the process exit code.
fn run_ipc(
    vision: PlantVision,
    mut reducer: LaneReducer,
    mut gpio: Box<dyn NozzleControl>,
    max_frames: u64,
    oneshot: bool,
    running: &Arc<AtomicBool>,
    watchdog: &mut Watchdog,
) -> i32 {
    let lane_count = reducer.lane_count();
    let all_off = |gpio: &mut Box<dyn NozzleControl>| gpio.apply(&vec![false; lane_count]);

    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let mut buf: Vec<u8> = Vec::new();
    let mut count: u64 = 0;

    let exit_code = loop {
        if !running.load(Ordering::SeqCst) {
            info!("signal received — leaving IPC loop");
            break 0;
        }
        let header = match ipc::read_frame(&mut stdin, &mut buf) {
            Ok(Some(h)) => h,
            Ok(None) => {
                info!("end of input stream");
                break 0;
            }
            Err(e) => {
                // The stream is out of sync; continuing could misread
                // pixel bytes as headers. Fail safe and let the shell
                // restart us.
                error!("IPC stream error: {e}");
                break 2;
            }
        };
        let ts_us = ipc::unix_micros();

        let width = header.width as usize;
        let height = header.height as usize;
        if width < lane_count {
            error!(
                "frame width {} is smaller than the {} configured lanes",
                width, lane_count,
            );
            break 2;
        }

        let start = Instant::now();
        let mask = vision.detect(&buf);
        let lanes = reducer.reduce(&mask, width, height);
        gpio.apply(&lanes);
        let latency_us = start.elapsed().as_micros() as u64;

        let response = ipc::IpcResponse {
            v: ipc::IPC_PROTOCOL_VERSION,
            frame: count,
            ts_us,
            lanes,
            latency_us,
        };
        if let Err(e) = ipc::write_response(&mut stdout, &response) {
            // Broken pipe: the outer shell is gone.
            error!("failed to write IPC response: {e}");
            break 2;
        }

        count += 1;
        watchdog.ping();
        if oneshot || (max_frames > 0 && count >= max_frames) {
            break 0;
        }
    };

    all_off(&mut gpio);
    info!("processed {} frames", count);
    exit_code
}

#[allow(clippy::too_many_arguments)]
fn run_test_pattern(
    pipeline: &mut Pipeline,
    width: usize,
    height: usize,
    max_frames: u64,
    oneshot: bool,
    running: &Arc<AtomicBool>,
    interval: Duration,
    watchdog: &mut Watchdog,
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
        watchdog.ping();

        let elapsed = start.elapsed();
        if count.is_multiple_of(100) || count == 1 {
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

/// Read frames from stdin on a dedicated thread and process them here.
///
/// Blocking `read_exact` on the main thread cannot detect a camera that
/// hangs *without* closing the pipe — the process would sit forever with
/// the last lane state applied to the valves. The reader thread plus a
/// polled channel lets the main loop notice missing frames and fail safe.
///
/// Returns `true` if the loop ended because the camera stalled.
fn run_stdin(
    pipeline: &mut Pipeline,
    frame_size: usize,
    max_frames: u64,
    running: &Arc<AtomicBool>,
    stall_timeout: Duration,
    watchdog: &mut Watchdog,
) -> bool {
    use crossbeam::channel::{bounded, RecvTimeoutError};

    let (frame_tx, frame_rx) = bounded::<Vec<u8>>(1);
    // Recycle buffers back to the reader to keep the hot path allocation-free.
    let (free_tx, free_rx) = bounded::<Vec<u8>>(2);
    for _ in 0..2 {
        let _ = free_tx.try_send(vec![0u8; frame_size]);
    }

    std::thread::spawn(move || {
        let mut stdin = std::io::stdin().lock();
        loop {
            let mut buf = free_rx.try_recv().unwrap_or_else(|_| vec![0u8; frame_size]);
            match stdin.read_exact(&mut buf) {
                Ok(()) => {
                    if frame_tx.send(buf).is_err() {
                        break; // main loop is gone
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        info!("end of input stream");
                    } else {
                        error!("stdin read error: {}", e);
                    }
                    break; // dropping frame_tx disconnects the channel
                }
            }
        }
    });

    // Poll in short intervals so SIGINT/SIGTERM stays responsive while
    // waiting for frames.
    let poll = Duration::from_millis(200);
    let mut last_frame = Instant::now();
    let mut count: u64 = 0;

    while running.load(Ordering::SeqCst) {
        match frame_rx.recv_timeout(poll) {
            Ok(buf) => {
                let start = Instant::now();
                pipeline.process(&buf);
                let _ = free_tx.try_send(buf);
                count += 1;
                last_frame = Instant::now();
                watchdog.ping();

                let elapsed = start.elapsed();
                if count.is_multiple_of(100) || count == 1 {
                    info!("frame {}: {:.1} ms", count, elapsed.as_secs_f64() * 1000.0,);
                }

                if max_frames > 0 && count >= max_frames {
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if !stall_timeout.is_zero() && last_frame.elapsed() >= stall_timeout {
                    error!(
                        "no frame received for {} s — camera stalled, failing safe",
                        stall_timeout.as_secs(),
                    );
                    info!("processed {} frames", count);
                    return true;
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    info!("processed {} frames", count);
    false
}

// ---------------------------------------------------------------------------
// GPIO construction
// ---------------------------------------------------------------------------

#[cfg(all(feature = "rpi", any(target_arch = "arm", target_arch = "aarch64")))]
fn build_real_gpio(config: &Config) -> Box<dyn NozzleControl> {
    use rustspray_core::io_gpio::RppalGpio;
    info!("using real GPIO pins: {:?}", config.gpio.pins);
    Box::new(RppalGpio::new(&config.gpio.pins))
}

#[cfg(not(all(feature = "rpi", any(target_arch = "arm", target_arch = "aarch64"))))]
fn build_real_gpio(_config: &Config) -> Box<dyn NozzleControl> {
    log::warn!(
        "real GPIO unavailable (requires an ARM build with --features rpi); falling back to mock"
    );
    Box::new(MockGpio::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpio_pins_flag_parses_comma_separated_bcm_numbers() {
        let cli = Cli::parse_from(["rustspray", "--gpio-pins", "27,22,23,24"]);
        assert_eq!(cli.gpio_pins, Some(vec![27, 22, 23, 24]));
    }

    #[test]
    fn gpio_pins_flag_is_optional() {
        let cli = Cli::parse_from(["rustspray"]);
        assert_eq!(cli.gpio_pins, None);
    }

    #[test]
    fn gpio_pins_flag_rejects_non_numeric_values() {
        assert!(Cli::try_parse_from(["rustspray", "--gpio-pins", "27,BOARD13"]).is_err());
    }

    #[test]
    fn gpio_pins_override_fails_validation_on_lane_mismatch() {
        // Two pins for four lanes must be caught by Config::validate
        // once the override is applied.
        let mut config = Config::default();
        config.gpio.pins = vec![27, 22];
        let err = config.validate().unwrap_err();
        assert!(err.contains("gpio.pins"), "unexpected error: {err}");
    }
}
