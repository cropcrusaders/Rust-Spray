use clap::Parser;
use crossbeam_channel::bounded;
use rustspray::{
    io_gpio::MockActuator,
    pipeline::{self, FrameSource, RgbFrame},
    SprayCfg,
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 1280)]
    width: u32,
    #[arg(long, default_value_t = 720)]
    height: u32,
    #[arg(long, default_value_t = 16)]
    thr: i16,
    #[arg(long, default_value_t = 0.3)]
    bottom_frac: f32,
    #[arg(long, default_value_t = 0.008)]
    min_ratio: f32,
    #[arg(long, default_value_t = 60)]
    fire_ms: u32,
    #[arg(long, default_value_t = 200)]
    holdoff_ms: u32,
    #[arg(long, default_value_t = 0.5)]
    hysteresis: f32,
}

struct DummySource {
    width: u32,
    height: u32,
}

impl FrameSource for DummySource {
    fn start(self) -> crossbeam_channel::Receiver<RgbFrame> {
        let (tx, rx) = bounded(2);
        std::thread::spawn(move || {
            let mut frame = vec![0u8; (self.width * self.height * 3) as usize];
            let mut toggle = false;
            loop {
                // simple animation: toggle all pixels between green and red
                for pix in frame.chunks_mut(3) {
                    if toggle {
                        pix.copy_from_slice(&[0, 255, 0]);
                    } else {
                        pix.copy_from_slice(&[255, 0, 0]);
                    }
                }
                toggle = !toggle;
                if tx
                    .send(RgbFrame {
                        data: frame.clone(),
                        width: self.width as usize,
                        height: self.height as usize,
                    })
                    .is_err()
                {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(33));
            }
        });
        rx
    }
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    let cfg = SprayCfg {
        thr: args.thr,
        bottom_frac: args.bottom_frac,
        min_ratio: args.min_ratio,
        fire_ms: args.fire_ms,
        holdoff_ms: args.holdoff_ms,
        hysteresis: args.hysteresis,
    };
    let src = DummySource {
        width: args.width,
        height: args.height,
    };
    let act = MockActuator::default();
    pipeline::run(src, act, cfg);
}
