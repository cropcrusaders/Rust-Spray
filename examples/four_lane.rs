use rustspray::{
    io_gpio::{MockGpio, NozzleControl},
    lanes::LaneReducer,
    pipeline::Pipeline,
    vision::{Detector, PlantVision},
};

/// Demo binary that runs the 4-lane pipeline on a synthetic frame.
///
/// Uses the built-in PlantVision detector by default.  When compiled with
/// `--features model` and given `--model <path.onnx>`, it loads an ONNX
/// model instead.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let _mock = args.iter().any(|a| a == "--mock-gpio");

    // Choose detector: ONNX model or built-in PlantVision.
    let detector: Box<dyn Detector> = make_detector(&args);

    let gpio: Box<dyn NozzleControl> = Box::new(MockGpio::default());
    let reducer = LaneReducer::new(4, 0.3, 0.15)
        .with_hold(2, 0)
        .with_roi(0.2, 0.8);
    let mut pipeline = Pipeline::new(reducer, gpio, detector, WIDTH, HEIGHT);

    // Generate a synthetic frame with green lanes 0 and 2
    let mut frame = vec![0u8; WIDTH * HEIGHT * 3];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let idx = (y * WIDTH + x) * 3;
            if x < WIDTH / 4 || (x >= WIDTH / 2 && x < 3 * WIDTH / 4) {
                frame[idx + 1] = 255; // strong green
            }
        }
    }

    // Frame 1: vegetation detected, lanes activate
    let lanes = pipeline.process(&frame);
    println!("frame 1 lanes: {:?}", lanes);
    println!("  density: {:?}", pipeline.lane_density());

    // Frame 2: vegetation gone but hold_on=2 keeps spraying
    let empty = vec![0u8; WIDTH * HEIGHT * 3];
    let lanes = pipeline.process(&empty);
    println!("frame 2 lanes: {:?} (held by temporal hold)", lanes);

    // Frame 3: hold expires, lanes deactivate
    let lanes = pipeline.process(&empty);
    println!("frame 3 lanes: {:?} (hold expired)", lanes);
}

fn make_detector(args: &[String]) -> Box<dyn Detector> {
    // Check for --model <path> argument.
    if let Some(pos) = args.iter().position(|a| a == "--model") {
        let path = args
            .get(pos + 1)
            .expect("--model requires a path to an ONNX file");

        #[cfg(feature = "model")]
        {
            println!("Loading ONNX model: {path}");
            return Box::new(rustspray::model::ModelDetector::load(
                path, WIDTH, HEIGHT, 0.5,
            ));
        }

        #[cfg(not(feature = "model"))]
        {
            let _ = path;
            eprintln!("Error: --model requires the `model` feature.");
            eprintln!("  cargo run --features model --example four_lane -- --mock-gpio --model vegetation.onnx");
            std::process::exit(1);
        }
    }

    // Default: built-in colour detector.
    Box::new(PlantVision::default())
}

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
