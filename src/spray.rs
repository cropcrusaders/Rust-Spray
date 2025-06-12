use rppal::gpio::{Gpio, OutputPin};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SprayError {
    #[error("GPIO error: {0}")]
    Gpio(#[from] rppal::gpio::Error),
    #[error("Invalid sprayer index")]
    InvalidIndex,
}

pub struct Sprayer {
    pin: OutputPin,
}

impl Sprayer {
    /// Create a new sprayer attached to the given GPIO pin.
    pub fn new(pin_num: u8) -> Result<Self, SprayError> {
        let gpio = Gpio::new()?;
        let pin = gpio.get(pin_num)?.into_output();
        Ok(Sprayer { pin })
    }

    /// Set the GPIO pin high to activate the sprayer.
    pub fn activate(&mut self) {
        self.pin.set_high();
    }

    /// Set the GPIO pin low to deactivate the sprayer.
    pub fn deactivate(&mut self) {
        self.pin.set_low();
    }

    /// Pulse the sprayer for the specified duration.
    pub fn pulse(&mut self, duration: Duration) {
        self.activate();
        std::thread::sleep(duration);
        self.deactivate();
    }
}

pub struct SprayController {
    sprayers: [Sprayer; 4],
}

impl SprayController {
    /// Create a controller for four sprayers using the supplied GPIO pins.
    pub fn new(pins: [u8; 4]) -> Result<Self, SprayError> {
        let sprayer0 = Sprayer::new(pins[0])?;
        let sprayer1 = Sprayer::new(pins[1])?;
        let sprayer2 = Sprayer::new(pins[2])?;
        let sprayer3 = Sprayer::new(pins[3])?;
        Ok(SprayController {
            sprayers: [sprayer0, sprayer1, sprayer2, sprayer3],
        })
    }

    /// Activate a specific sprayer by index.
    pub fn activate_sprayer(&mut self, index: usize) -> Result<(), SprayError> {
        if index < 4 {
            self.sprayers[index].activate();
            Ok(())
        } else {
            Err(SprayError::InvalidIndex)
        }
    }

    /// Deactivate a specific sprayer by index.
    pub fn deactivate_sprayer(&mut self, index: usize) -> Result<(), SprayError> {
        if index < 4 {
            self.sprayers[index].deactivate();
            Ok(())
        } else {
            Err(SprayError::InvalidIndex)
        }
    }

    /// Pulse a specific sprayer for a duration.
    pub fn pulse_sprayer(&mut self, index: usize, duration: Duration) -> Result<(), SprayError> {
        if index < 4 {
            self.sprayers[index].pulse(duration);
            Ok(())
        } else {
            Err(SprayError::InvalidIndex)
        }
    }

    /// Activate all sprayers simultaneously.
    pub fn activate_all(&mut self) {
        for sprayer in &mut self.sprayers {
            sprayer.activate();
        }
    }

    /// Deactivate all sprayers.
    pub fn deactivate_all(&mut self) {
        for sprayer in &mut self.sprayers {
            sprayer.deactivate();
        }
    }
}
