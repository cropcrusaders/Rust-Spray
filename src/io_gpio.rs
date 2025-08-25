//! GPIO abstraction for nozzle control.

/// Trait for controlling spray nozzles.
pub trait NozzleControl {
    /// Apply lane activations.
    fn apply(&mut self, lanes: &[bool]);
}

/// Mock implementation that prints activations.
#[derive(Default)]
pub struct MockGpio;

impl NozzleControl for MockGpio {
    fn apply(&mut self, lanes: &[bool]) {
        println!("mock gpio: {:?}", lanes);
    }
}

#[cfg(feature = "rpi")]
use rppal::gpio::{Gpio, OutputPin};

#[cfg(feature = "rpi")]
/// GPIO implementation using `rppal`.
pub struct RppalGpio {
    pins: Vec<OutputPin>,
}

#[cfg(feature = "rpi")]
impl RppalGpio {
    /// Create from BCM pin numbers.
    pub fn new(pins: &[u8]) -> Self {
        let gpio = Gpio::new().expect("gpio");
        let pins = pins
            .iter()
            .map(|&p| gpio.get(p).unwrap().into_output())
            .collect();
        Self { pins }
    }
}

#[cfg(feature = "rpi")]
impl NozzleControl for RppalGpio {
    fn apply(&mut self, lanes: &[bool]) {
        for (pin, &active) in self.pins.iter_mut().zip(lanes) {
            if active {
                pin.set_high();
            } else {
                pin.set_low();
            }
        }
    }
}
