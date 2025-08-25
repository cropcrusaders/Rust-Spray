//! Spray controller for GPIO-based sprayers
//!
//! This module provides control for up to 4 sprayer outputs via GPIO pins.
//! It's designed to work with Raspberry Pi GPIO using the rppal crate.

#![allow(dead_code)]

#[cfg(all(
    feature = "with-rppal",
    any(target_arch = "arm", target_arch = "aarch64")
))]
use rppal::gpio::{Gpio, OutputPin};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SprayError {
    #[cfg(all(
        feature = "with-rppal",
        any(target_arch = "arm", target_arch = "aarch64")
    ))]
    #[error("GPIO error: {0}")]
    Gpio(#[from] rppal::gpio::Error),
    #[error("Invalid sprayer index: {0} (valid range: 0-3)")]
    InvalidIndex(usize),
    #[error("GPIO feature not enabled - cannot control sprayers")]
    NoGpio,
}

/// Individual sprayer control
pub struct Sprayer {
    #[cfg(all(
        feature = "with-rppal",
        any(target_arch = "arm", target_arch = "aarch64")
    ))]
    pin: OutputPin,
    #[cfg(not(all(
        feature = "with-rppal",
        any(target_arch = "arm", target_arch = "aarch64")
    )))]
    pin_number: u8,
}

impl Sprayer {
    /// Create a new sprayer on the specified GPIO pin
    ///
    /// # Arguments
    /// * `pin_num` - GPIO pin number
    ///
    /// # Returns
    /// * `Result<Self, SprayError>` - New sprayer instance or error
    pub fn new(pin_num: u8) -> Result<Self, SprayError> {
        #[cfg(all(
            feature = "with-rppal",
            any(target_arch = "arm", target_arch = "aarch64")
        ))]
        {
            let gpio = Gpio::new()?;
            let pin = gpio.get(pin_num)?.into_output();
            Ok(Sprayer { pin })
        }
        #[cfg(not(all(
            feature = "with-rppal",
            any(target_arch = "arm", target_arch = "aarch64")
        )))]
        {
            log::warn!("GPIO feature not enabled - sprayer on pin {pin_num} will not function");
            Ok(Sprayer {
                pin_number: pin_num,
            })
        }
    }

    /// Activate the sprayer (turn on)
    pub fn activate(&mut self) {
        #[cfg(all(
            feature = "with-rppal",
            any(target_arch = "arm", target_arch = "aarch64")
        ))]
        {
            self.pin.set_high();
        }
        #[cfg(not(all(
            feature = "with-rppal",
            any(target_arch = "arm", target_arch = "aarch64")
        )))]
        {
            log::info!("Mock: Activating sprayer on pin {}", self.pin_number);
        }
    }

    /// Deactivate the sprayer (turn off)
    pub fn deactivate(&mut self) {
        #[cfg(all(
            feature = "with-rppal",
            any(target_arch = "arm", target_arch = "aarch64")
        ))]
        {
            self.pin.set_low();
        }
        #[cfg(not(all(
            feature = "with-rppal",
            any(target_arch = "arm", target_arch = "aarch64")
        )))]
        {
            log::info!("Mock: Deactivating sprayer on pin {}", self.pin_number);
        }
    }

    /// Pulse the sprayer for a specific duration
    ///
    /// # Arguments
    /// * `duration` - How long to keep the sprayer active
    pub fn pulse(&mut self, duration: Duration) {
        self.activate();
        std::thread::sleep(duration);
        self.deactivate();
    }
}

/// Controller for multiple sprayers
pub struct SprayController {
    sprayers: [Sprayer; 4],
}

impl SprayController {
    /// Create a new spray controller with 4 sprayers
    ///
    /// # Arguments
    /// * `pins` - Array of 4 GPIO pin numbers for the sprayers
    ///
    /// # Returns
    /// * `Result<Self, SprayError>` - New controller instance or error
    pub fn new(pins: [u8; 4]) -> Result<Self, SprayError> {
        let sprayers = [
            Sprayer::new(pins[0])?,
            Sprayer::new(pins[1])?,
            Sprayer::new(pins[2])?,
            Sprayer::new(pins[3])?,
        ];

        Ok(SprayController { sprayers })
    }

    /// Activate a specific sprayer by index
    ///
    /// # Arguments
    /// * `index` - Sprayer index (0-3)
    ///
    /// # Returns
    /// * `Result<(), SprayError>` - Success or error if index is invalid
    pub fn activate_sprayer(&mut self, index: usize) -> Result<(), SprayError> {
        if index >= 4 {
            return Err(SprayError::InvalidIndex(index));
        }
        self.sprayers[index].activate();
        Ok(())
    }

    /// Deactivate a specific sprayer by index
    ///
    /// # Arguments
    /// * `index` - Sprayer index (0-3)
    ///
    /// # Returns
    /// * `Result<(), SprayError>` - Success or error if index is invalid
    pub fn deactivate_sprayer(&mut self, index: usize) -> Result<(), SprayError> {
        if index >= 4 {
            return Err(SprayError::InvalidIndex(index));
        }
        self.sprayers[index].deactivate();
        Ok(())
    }

    /// Pulse a specific sprayer for a duration
    ///
    /// # Arguments
    /// * `index` - Sprayer index (0-3)
    /// * `duration` - How long to pulse the sprayer
    ///
    /// # Returns
    /// * `Result<(), SprayError>` - Success or error if index is invalid
    pub fn pulse_sprayer(&mut self, index: usize, duration: Duration) -> Result<(), SprayError> {
        if index >= 4 {
            return Err(SprayError::InvalidIndex(index));
        }
        self.sprayers[index].pulse(duration);
        Ok(())
    }

    /// Activate all sprayers simultaneously
    pub fn activate_all(&mut self) {
        for sprayer in &mut self.sprayers {
            sprayer.activate();
        }
    }

    /// Deactivate all sprayers simultaneously
    pub fn deactivate_all(&mut self) {
        for sprayer in &mut self.sprayers {
            sprayer.deactivate();
        }
    }

    /// Pulse all sprayers for the same duration
    ///
    /// # Arguments
    /// * `duration` - How long to pulse all sprayers
    pub fn pulse_all(&mut self, duration: Duration) {
        self.activate_all();
        std::thread::sleep(duration);
        self.deactivate_all();
    }

    /// Get the number of sprayers
    pub fn sprayer_count(&self) -> usize {
        self.sprayers.len()
    }
}
