//! GPIO actuator abstraction.

use std::fmt;

/// Trait representing a four-lane actuator device.
pub trait Actuator: Send + 'static {
    fn apply(&mut self, ratios: [f32; 4], states: [bool; 4]) -> Result<(), ActuatorError>;
}

/// Simple error type for GPIO operations.
#[derive(Debug, Clone)]
pub struct ActuatorError(pub String);

impl fmt::Display for ActuatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ActuatorError {}

/// Mock actuator used on non-embedded platforms.
#[derive(Default)]
pub struct MockActuator;

impl Actuator for MockActuator {
    fn apply(&mut self, ratios: [f32; 4], states: [bool; 4]) -> Result<(), ActuatorError> {
        print!("[");
        for i in 0..4 {
            let state = if states[i] { "ON" } else { "off" };
            print!("{:.3}:{}", ratios[i], state);
            if i != 3 {
                print!(" ");
            }
        }
        println!("]");
        Ok(())
    }
}

#[cfg(target_arch = "arm")]
mod rpi {
    use super::{Actuator, ActuatorError};
    use rppal::gpio::{Gpio, OutputPin};

    /// Raspberry Pi GPIO actuator using the `rppal` crate.
    pub struct RppalActuator {
        pins: [OutputPin; 4],
    }

    impl RppalActuator {
        pub fn new(pins: [u8; 4]) -> Result<Self, ActuatorError> {
            let mut out = Vec::new();
            let gpio = Gpio::new().map_err(|e| ActuatorError(e.to_string()))?;
            for &p in &pins {
                out.push(
                    gpio.get(p)
                        .map_err(|e| ActuatorError(e.to_string()))?
                        .into_output(),
                );
            }
            // SAFETY: we push exactly four elements
            let pins: [OutputPin; 4] = out.try_into().unwrap();
            Ok(Self { pins })
        }
    }

    impl Actuator for RppalActuator {
        fn apply(&mut self, _ratios: [f32; 4], states: [bool; 4]) -> Result<(), ActuatorError> {
            for (pin, &state) in self.pins.iter_mut().zip(states.iter()) {
                if state {
                    pin.set_high();
                } else {
                    pin.set_low();
                }
            }
            Ok(())
        }
    }
}
