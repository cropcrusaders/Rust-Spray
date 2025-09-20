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
    let reducer = LaneReducer::new(4, 0.3, 0.15);
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
    pipeline.process(&frame);
}

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
