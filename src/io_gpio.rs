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

// Real GPIO is only available when the `rpi` feature is enabled AND we are
// compiling for ARM — `rppal` is a target-specific dependency, so gating on
// the feature alone would break `--features rpi` builds on desktop hosts.
#[cfg(all(feature = "rpi", any(target_arch = "arm", target_arch = "aarch64")))]
use rppal::gpio::{Gpio, OutputPin};

#[cfg(all(feature = "rpi", any(target_arch = "arm", target_arch = "aarch64")))]
/// GPIO implementation using `rppal`.
///
/// All pins are initialised **low** (nozzles off) and are driven low again
/// when the struct is dropped, so a graceful shutdown never leaves a valve
/// open.
pub struct RppalGpio {
    pins: Vec<OutputPin>,
}

#[cfg(all(feature = "rpi", any(target_arch = "arm", target_arch = "aarch64")))]
impl RppalGpio {
    /// Create from BCM pin numbers.
    ///
    /// # Panics
    /// Panics with a descriptive message if the GPIO peripheral or any
    /// requested pin cannot be acquired.
    pub fn new(pins: &[u8]) -> Self {
        let gpio = Gpio::new().expect(
            "failed to access the GPIO peripheral — is this a Raspberry Pi \
             and is /dev/gpiomem accessible (root or `gpio` group)?",
        );
        let pins = pins
            .iter()
            .map(|&p| {
                let mut pin = gpio
                    .get(p)
                    .unwrap_or_else(|e| panic!("failed to acquire GPIO pin {p}: {e}"))
                    .into_output_low();
                // Keep the pin as a low output on drop instead of rppal's
                // default reset-to-floating-input, which could energise
                // active-low relay boards.
                pin.set_reset_on_drop(false);
                pin
            })
            .collect();
        Self { pins }
    }
}

#[cfg(all(feature = "rpi", any(target_arch = "arm", target_arch = "aarch64")))]
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

#[cfg(all(feature = "rpi", any(target_arch = "arm", target_arch = "aarch64")))]
impl Drop for RppalGpio {
    fn drop(&mut self) {
        for pin in &mut self.pins {
            pin.set_low();
        }
    }
}
