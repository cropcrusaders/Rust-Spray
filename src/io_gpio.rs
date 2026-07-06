//! GPIO abstraction for nozzle control.

/// Trait for controlling spray nozzles.
pub trait NozzleControl {
    /// Apply lane activations.
    fn apply(&mut self, lanes: &[bool]);
}

/// Mock implementation that logs lane state **changes** to stderr as
/// `[MOCK GPIO] lane=N state=ON/OFF`.
///
/// Output goes to stderr, never stdout: in `--ipc-mode` stdout carries the
/// newline-delimited JSON protocol and must not be polluted.
#[derive(Default)]
pub struct MockGpio {
    last: Option<Vec<bool>>,
}

impl NozzleControl for MockGpio {
    fn apply(&mut self, lanes: &[bool]) {
        for (lane, &state) in lanes.iter().enumerate() {
            let changed = match &self.last {
                Some(prev) => prev.get(lane) != Some(&state),
                None => true, // first application: report every lane
            };
            if changed {
                eprintln!(
                    "[MOCK GPIO] lane={} state={}",
                    lane,
                    if state { "ON" } else { "OFF" },
                );
            }
        }
        self.last = Some(lanes.to_vec());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_gpio_tracks_state_changes() {
        let mut gpio = MockGpio::default();
        gpio.apply(&[true, false]);
        assert_eq!(gpio.last.as_deref(), Some(&[true, false][..]));
        gpio.apply(&[false, false]);
        assert_eq!(gpio.last.as_deref(), Some(&[false, false][..]));
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
