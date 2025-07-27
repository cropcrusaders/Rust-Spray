use rustspray::gps::{GpsController, MockGpsProvider, GpsProvider};
use rustspray::logging::{WeedDetectionLogger, LoggingConfig, LogFormat, DetectionInfo, ActionTaken};
use rustspray::config::Config;
use std::fs;
use std::path::Path;

#[test]
fn test_mock_gps_provider() {
    let mut provider = MockGpsProvider::new(42.0, -93.5);
    
    // Test multiple readings
    let loc1 = provider.get_location().expect("Failed to get location");
    assert!(provider.is_available());
    assert!(loc1.is_mock);
    assert!((loc1.latitude - 42.0).abs() < 0.01);
    assert!((loc1.longitude - (-93.5)).abs() < 0.01);
    
    // Test drift over time
    let loc2 = provider.get_location().expect("Failed to get location");
    assert!(loc1.timestamp <= loc2.timestamp);
}

#[test]
fn test_gps_controller() {
    let mut controller = GpsController::new_mock(40.0, -95.0);
    
    let location = controller.get_location().expect("Failed to get location");
    assert!(location.is_mock);
    assert!((location.latitude - 40.0).abs() < 0.01);
    assert!((location.longitude - (-95.0)).abs() < 0.01);
}

#[test]
fn test_logging_json() {
    let temp_file = "test_weed_detections_json";
    let json_path = format!("{}.json", temp_file);
    
    // Clean up any existing test file
    let _ = fs::remove_file(&json_path);
    
    let config = LoggingConfig {
        enabled: true,
        output_file: temp_file.to_string(),
        format: LogFormat::Json,
        buffer_size: 1024,
        auto_flush: true,
    };
    
    let mut logger = WeedDetectionLogger::new(config).expect("Failed to create logger");
    
    let detection_info = DetectionInfo {
        algorithm: "hsv".to_string(),
        center_x: 100,
        center_y: 150,
        bounding_box: [90, 140, 20, 20],
        area: 400.0,
        confidence: Some(0.85),
        frame_number: 42,
    };
    
    let action = ActionTaken::LoggedOnly;
    
    logger.log_detection(None, detection_info, action).expect("Failed to log detection");
    logger.flush().expect("Failed to flush logger");
    
    // Verify file was created and contains data
    assert!(Path::new(&json_path).exists());
    let contents = fs::read_to_string(&json_path).expect("Failed to read log file");
    assert!(contents.contains("hsv"));
    assert!(contents.contains("\"center_x\":100"));
    
    // Clean up
    let _ = fs::remove_file(&json_path);
}

#[test]
fn test_logging_csv() {
    let temp_file = "test_weed_detections_csv";
    let csv_path = format!("{}.csv", temp_file);
    
    // Clean up any existing test file
    let _ = fs::remove_file(&csv_path);
    
    let config = LoggingConfig {
        enabled: true,
        output_file: temp_file.to_string(),
        format: LogFormat::Csv,
        buffer_size: 1024,
        auto_flush: true,
    };
    
    let mut logger = WeedDetectionLogger::new(config).expect("Failed to create logger");
    
    let detection_info = DetectionInfo {
        algorithm: "exg".to_string(),
        center_x: 200,
        center_y: 250,
        bounding_box: [190, 240, 20, 20],
        area: 500.0,
        confidence: None,
        frame_number: 100,
    };
    
    let action = ActionTaken::SprayActivated {
        duration_ms: 200,
        sprayers: vec![23, 24, 25, 26],
    };
    
    logger.log_detection(None, detection_info, action).expect("Failed to log detection");
    logger.flush().expect("Failed to flush logger");
    
    // Verify file was created and contains data
    assert!(Path::new(&csv_path).exists());
    let contents = fs::read_to_string(&csv_path).expect("Failed to read log file");
    assert!(contents.contains("id,timestamp,latitude")); // Header
    assert!(contents.contains("exg"));
    assert!(contents.contains("200,250"));
    assert!(contents.contains("spray"));
    
    // Clean up
    let _ = fs::remove_file(&csv_path);
}

#[test]
fn test_config_with_new_sections() {
    let toml_content = r#"
[camera]
device = "/dev/video0"
resolution_width = 640
resolution_height = 480

[detection]
algorithm = "hsv"
exg_min = 20
exg_max = 200
hue_min = 25
hue_max = 100
brightness_min = 10
brightness_max = 220
saturation_min = 40
saturation_max = 250
min_area = 15.0
invert_hue = true

[spray]
pins = [23, 24, 25, 26]
activation_duration_ms = 200
enabled = false

[gps]
enabled = true
mock_latitude = 42.123
mock_longitude = -93.456

[logging]
enabled = true
output_file = "my_detections"
format = "both"
auto_flush = false
"#;

    let config: Config = toml::from_str(toml_content).expect("Failed to parse TOML");
    let validation_result = config.validate();
    assert!(validation_result.is_ok(), "Expected validation to pass");
    
    // Test new fields
    assert!(!config.spray.enabled); // Spray disabled
    assert!(config.gps.enabled);
    assert!((config.gps.mock_latitude - 42.123).abs() < 0.001);
    assert!(config.logging.enabled);
    assert_eq!(config.logging.format, "both");
    
    // Test logging config conversion
    let logging_config = config.logging.to_logging_config();
    assert!(logging_config.enabled);
    assert_eq!(logging_config.output_file, "my_detections");
    assert!(!logging_config.auto_flush);
}

#[test]
fn test_default_config_sections() {
    let toml_content = r#"
[camera]
device = "/dev/video0"
resolution_width = 640
resolution_height = 480

[detection]
algorithm = "hsv"
exg_min = 20
exg_max = 200
hue_min = 25
hue_max = 100
brightness_min = 10
brightness_max = 220
saturation_min = 40
saturation_max = 250
min_area = 15.0
invert_hue = true

[spray]
pins = [23, 24, 25, 26]
activation_duration_ms = 200
"#;

    let config: Config = toml::from_str(toml_content).expect("Failed to parse TOML");
    
    // Test defaults for new sections
    assert!(config.spray.enabled); // Default is true
    assert!(!config.gps.enabled);  // Default is false
    assert_eq!(config.gps.mock_latitude, 42.0);
    assert!(config.logging.enabled); // Default is true
    assert_eq!(config.logging.format, "json");
}