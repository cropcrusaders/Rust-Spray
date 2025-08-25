//! Rust-Spray: fast excess-green based weed detection.
//!
//! This crate implements a minimal pipeline for real-time weed spraying
//! without any OpenCV dependency.  The processing stages are:
//! 1. `exg` – compute a binary mask using SIMD.
//! 2. `lanes` – reduce the bottom band into four lane ratios and apply
//!    timing logic.
//! 3. `io_gpio` – platform specific GPIO control.
//! 4. `pipeline` – thread orchestration using crossbeam channels.

pub mod config;
pub mod exg;
pub mod io_gpio;
pub mod lanes;
pub mod pipeline;

pub use config::SprayCfg;
