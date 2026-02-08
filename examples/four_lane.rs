use rustspray::{
    io_gpio::{MockGpio, NozzleControl},
    lanes::LaneReducer,
    pipeline::Pipeline,
    vision::PlantVision,
};

/// Demo binary that runs the 4-lane pipeline on a synthetic frame.
fn main() {
    let mock = std::env::args().any(|a| a == "--mock-gpio");
    if !mock {
        eprintln!("This example currently supports only --mock-gpio on desktop");
    }
    let gpio: Box<dyn NozzleControl> = Box::new(MockGpio::default());
    let reducer = LaneReducer::new(4, 0.3, 0.15)
        .with_hold(2, 0)
        .with_roi(0.2, 0.8);
    let vision = PlantVision::default();
    let mut pipeline = Pipeline::new(reducer, gpio, vision, WIDTH, HEIGHT);

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

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
