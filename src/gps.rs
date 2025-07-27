//! GPS location tracking for weed detection
//!
//! This module provides GPS coordinate tracking functionality.
//! For systems without GPS hardware, it provides mock coordinates.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GpsError {
    #[error("GPS hardware not available")]
    NotAvailable,
    #[error("GPS fix not available")]
    NoFix,
    #[error("GPS read error: {0}")]
    ReadError(String),
}

/// GPS coordinates with accuracy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsCoordinate {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: Option<f64>, // Horizontal accuracy in meters
    pub timestamp: u64,        // Unix timestamp
    pub is_mock: bool,         // True if this is a simulated coordinate
}

impl Default for GpsCoordinate {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            altitude: None,
            accuracy: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_mock: true,
        }
    }
}

/// GPS provider trait for different GPS backends
pub trait GpsProvider {
    fn get_location(&mut self) -> Result<GpsCoordinate, GpsError>;
    fn is_available(&self) -> bool;
}

/// Mock GPS provider for testing and systems without GPS hardware
pub struct MockGpsProvider {
    base_lat: f64,
    base_lon: f64,
    drift_amount: f64,
    reading_count: u64,
}

impl MockGpsProvider {
    pub fn new(base_lat: f64, base_lon: f64) -> Self {
        Self {
            base_lat,
            base_lon,
            drift_amount: 0.0001, // Small drift to simulate movement
            reading_count: 0,
        }
    }

    /// Create a mock GPS provider with default coordinates (somewhere in a field)
    pub fn default_field() -> Self {
        // Default to coordinates in agricultural area (example: Iowa, USA)
        Self::new(42.0, -93.5)
    }
}

impl GpsProvider for MockGpsProvider {
    fn get_location(&mut self) -> Result<GpsCoordinate, GpsError> {
        self.reading_count += 1;
        
        // Simulate small movements in a field pattern
        let drift_x = (self.reading_count as f64 * 0.1).sin() * self.drift_amount;
        let drift_y = (self.reading_count as f64 * 0.05).cos() * self.drift_amount * 0.5;
        
        Ok(GpsCoordinate {
            latitude: self.base_lat + drift_y,
            longitude: self.base_lon + drift_x,
            altitude: Some(100.0 + (self.reading_count as f64 * 0.1).sin() * 2.0),
            accuracy: Some(2.5), // 2.5 meter accuracy
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_mock: true,
        })
    }

    fn is_available(&self) -> bool {
        true
    }
}

/// Main GPS controller that abstracts different GPS providers
pub struct GpsController {
    provider: Box<dyn GpsProvider>,
}

impl GpsController {
    /// Create a new GPS controller with mock provider
    pub fn new_mock(base_lat: f64, base_lon: f64) -> Self {
        Self {
            provider: Box::new(MockGpsProvider::new(base_lat, base_lon)),
        }
    }

    /// Create a GPS controller with default field coordinates
    pub fn new_default() -> Self {
        Self {
            provider: Box::new(MockGpsProvider::default_field()),
        }
    }

    /// Get current GPS location
    pub fn get_location(&mut self) -> Result<GpsCoordinate, GpsError> {
        self.provider.get_location()
    }

    /// Check if GPS is available
    pub fn is_available(&self) -> bool {
        self.provider.is_available()
    }
}