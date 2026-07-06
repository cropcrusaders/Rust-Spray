#![feature(portable_simd)]

//! Core lane spraying pipeline for agricultural robotics.
//!
//! Processes camera frames through three stages:
//! 1. **Vision** — multi-cue vegetation detection ([`vision::PlantVision`])
//! 2. **Lane reduction** — per-lane coverage with hysteresis ([`lanes::LaneReducer`])
//! 3. **Actuation** — GPIO relay control ([`io_gpio::NozzleControl`])
//!
//! Host integrations (see `INTEGRATION.md`):
//! - [`ipc`] — subprocess protocol (frames on stdin, JSON on stdout)
//! - [`ffi`] — C ABI for direct linking (`librustspray_core.so`)

pub mod config;
pub mod exg;
pub mod ffi;
pub mod io_gpio;
pub mod ipc;
pub mod lanes;
pub mod pipeline;
pub mod vision;
