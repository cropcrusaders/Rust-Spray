use rustspray::config::{Config, ConfigError};

#[test]
fn test_config_loading() {
    let config = Config::load("config/Config.toml").expect("Failed to load config");

    // Verify basic structure
    assert!(!config.camera.device.is_empty());
    assert!(config.camera.resolution_width > 0);
    assert!(config.camera.resolution_height > 0);
    assert!(config.spray.pins.len() == 4);
    assert!(config.spray.activation_duration_ms > 0);
}

#[test]
fn test_config_validation() {
    // Test invalid config with zero resolution
    let toml_content = r#"
[camera]
device = "/dev/video0"
resolution_width = 0
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
    let validation_result = config.validate();
    assert!(
        validation_result.is_err(),
        "Expected validation to fail for zero resolution"
    );
}

#[test]
fn test_unsupported_algorithm() {
    let toml_content = r#"
[camera]
device = "/dev/video0"
resolution_width = 640
resolution_height = 480

[detection]
algorithm = "invalid_algorithm"
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
    let validation_result = config.validate();
    assert!(
        validation_result.is_err(),
        "Expected validation to fail for unsupported algorithm"
    );

    if let Err(ConfigError::Validation(msg)) = validation_result {
        assert!(
            msg.contains("Unsupported algorithm"),
            "Error message should mention unsupported algorithm"
        );
    } else {
        panic!("Expected ConfigError::Validation");
    }
}

#[test]
fn test_valid_config() {
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
    let validation_result = config.validate();
    assert!(
        validation_result.is_ok(),
        "Expected validation to pass for valid config"
    );
}
