//! Data logging for weed detection events
//!
//! This module handles logging of weed detection events including location,
//! timestamp, and detection metadata for mapping and analysis.

use crate::gps::GpsCoordinate;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum LoggingError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid log format: {0}")]
    InvalidFormat(String),
}

/// Detection event record for a single weed detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeedDetectionEvent {
    pub id: String,                    // Unique ID for this detection
    pub timestamp: DateTime<Utc>,      // When the detection occurred
    pub location: Option<GpsCoordinate>, // GPS coordinates if available
    pub detection_info: DetectionInfo, // Details about the detection
    pub action_taken: ActionTaken,     // What action was taken (spray, log only, etc.)
}

/// Information about the weed detection itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionInfo {
    pub algorithm: String,           // Detection algorithm used
    pub center_x: i32,              // Center X coordinate in image
    pub center_y: i32,              // Center Y coordinate in image
    pub bounding_box: [i32; 4],     // [x, y, width, height]
    pub area: f64,                  // Detected area in pixels
    pub confidence: Option<f64>,    // Detection confidence if available
    pub frame_number: u64,          // Frame number in video stream
}

/// Action taken after detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionTaken {
    SprayActivated { duration_ms: u32, sprayers: Vec<u8> },
    LoggedOnly,
    Manual { description: String },
}

/// Configuration for data logging
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub output_file: String,
    pub format: LogFormat,
    pub buffer_size: usize,
    pub auto_flush: bool,
}

/// Supported output formats
#[derive(Debug, Clone)]
pub enum LogFormat {
    Json,
    Csv,
    Both,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            output_file: "weed_detections".to_string(), // Extension added based on format
            format: LogFormat::Json,
            buffer_size: 1024,
            auto_flush: true,
        }
    }
}

/// Main data logger for weed detection events
pub struct WeedDetectionLogger {
    config: LoggingConfig,
    json_writer: Option<BufWriter<File>>,
    csv_writer: Option<BufWriter<File>>,
    event_count: u64,
}

impl WeedDetectionLogger {
    /// Create a new logger with the specified configuration
    pub fn new(config: LoggingConfig) -> Result<Self, LoggingError> {
        let mut logger = Self {
            config,
            json_writer: None,
            csv_writer: None,
            event_count: 0,
        };

        if logger.config.enabled {
            logger.setup_writers()?;
        }

        Ok(logger)
    }

    /// Setup file writers based on configuration
    fn setup_writers(&mut self) -> Result<(), LoggingError> {
        match self.config.format {
            LogFormat::Json | LogFormat::Both => {
                let json_path = format!("{}.json", self.config.output_file);
                let json_file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&json_path)?;
                self.json_writer = Some(BufWriter::with_capacity(
                    self.config.buffer_size,
                    json_file,
                ));
                log::info!("JSON logging enabled: {}", json_path);
            }
            _ => {}
        }

        match self.config.format {
            LogFormat::Csv | LogFormat::Both => {
                let csv_path = format!("{}.csv", self.config.output_file);
                let should_write_header = !Path::new(&csv_path).exists();
                let csv_file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&csv_path)?;
                let mut csv_writer = BufWriter::with_capacity(self.config.buffer_size, csv_file);

                // Write CSV header if this is a new file
                if should_write_header {
                    self.write_csv_header(&mut csv_writer)?;
                }

                self.csv_writer = Some(csv_writer);
                log::info!("CSV logging enabled: {}", csv_path);
            }
            _ => {}
        }

        Ok(())
    }

    /// Write CSV header row
    fn write_csv_header(&self, writer: &mut BufWriter<File>) -> Result<(), LoggingError> {
        writeln!(
            writer,
            "id,timestamp,latitude,longitude,altitude,gps_accuracy,is_mock_gps,algorithm,center_x,center_y,bbox_x,bbox_y,bbox_width,bbox_height,area,confidence,frame_number,action_type,spray_duration_ms,sprayers"
        )?;
        Ok(())
    }

    /// Log a weed detection event
    pub fn log_detection(
        &mut self,
        location: Option<GpsCoordinate>,
        detection_info: DetectionInfo,
        action_taken: ActionTaken,
    ) -> Result<(), LoggingError> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = WeedDetectionEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            location,
            detection_info,
            action_taken,
        };

        self.event_count += 1;

        // Write to JSON if enabled
        if let Some(ref mut writer) = self.json_writer {
            writeln!(writer, "{}", serde_json::to_string(&event)?)?;
            if self.config.auto_flush {
                writer.flush()?;
            }
        }

        // Write to CSV if enabled
        if self.csv_writer.is_some() {
            let csv_record = self.format_csv_record(&event)?;
            if let Some(ref mut writer) = self.csv_writer {
                writeln!(writer, "{}", csv_record)?;
                if self.config.auto_flush {
                    writer.flush()?;
                }
            }
        }

        if self.event_count % 100 == 0 {
            log::info!("Logged {} weed detection events", self.event_count);
        }

        Ok(())
    }

    /// Format a CSV record as a string
    fn format_csv_record(&self, event: &WeedDetectionEvent) -> Result<String, LoggingError> {
        let (lat, lon, alt, accuracy, is_mock) = if let Some(ref loc) = event.location {
            (
                loc.latitude.to_string(),
                loc.longitude.to_string(),
                loc.altitude.map_or("".to_string(), |a| a.to_string()),
                loc.accuracy.map_or("".to_string(), |a| a.to_string()),
                loc.is_mock.to_string(),
            )
        } else {
            ("".to_string(), "".to_string(), "".to_string(), "".to_string(), "true".to_string())
        };

        let (action_type, spray_duration, sprayers) = match &event.action_taken {
            ActionTaken::SprayActivated { duration_ms, sprayers } => (
                "spray".to_string(),
                duration_ms.to_string(),
                sprayers.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(";"),
            ),
            ActionTaken::LoggedOnly => ("log_only".to_string(), "".to_string(), "".to_string()),
            ActionTaken::Manual { description } => (format!("manual:{}", description), "".to_string(), "".to_string()),
        };

        Ok(format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            event.id,
            event.timestamp.to_rfc3339(),
            lat,
            lon,
            alt,
            accuracy,
            is_mock,
            event.detection_info.algorithm,
            event.detection_info.center_x,
            event.detection_info.center_y,
            event.detection_info.bounding_box[0],
            event.detection_info.bounding_box[1],
            event.detection_info.bounding_box[2],
            event.detection_info.bounding_box[3],
            event.detection_info.area,
            event.detection_info.confidence.map_or("".to_string(), |c| c.to_string()),
            event.detection_info.frame_number,
            action_type,
            spray_duration,
            sprayers
        ))
    }

    /// Flush all buffers
    pub fn flush(&mut self) -> Result<(), LoggingError> {
        if let Some(ref mut writer) = self.json_writer {
            writer.flush()?;
        }
        if let Some(ref mut writer) = self.csv_writer {
            writer.flush()?;
        }
        Ok(())
    }

    /// Get the number of events logged so far
    pub fn event_count(&self) -> u64 {
        self.event_count
    }

    /// Check if logging is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

impl Drop for WeedDetectionLogger {
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            log::error!("Failed to flush logger on drop: {}", e);
        }
    }
}