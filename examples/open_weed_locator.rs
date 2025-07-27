//! Example demonstrating Open Weed Locator functionality
//! 
//! This example shows how to use the system in logging mode without spraying,
//! simulating what an open weed locator would do.

use rustspray::{
    Config, GpsController, WeedDetectionLogger, 
    logging::{DetectionInfo, ActionTaken}
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // Load configuration
    let config = Config::load("config/Config.toml")?;
    
    // Initialize GPS controller (mock)
    let mut gps = GpsController::new_mock(42.0314, -93.5854);
    
    // Initialize logger for weed locator mode
    let mut logger = WeedDetectionLogger::new(config.logging.to_logging_config())?;
    
    println!("=== Open Weed Locator Example ===");
    println!("Simulating weed detection and location logging...");
    
    // Simulate detecting several weeds with different locations
    for i in 1..=5 {
        // Get current location
        let location = gps.get_location()?;
        
        // Simulate a weed detection
        let detection_info = DetectionInfo {
            algorithm: "hsv".to_string(),
            center_x: 100 + i * 50,
            center_y: 150 + i * 30,
            bounding_box: [90 + i * 50, 140 + i * 30, 20, 20],
            area: 300.0 + i as f64 * 50.0,
            confidence: Some(0.8 + i as f64 * 0.02),
            frame_number: i as u64,
        };
        
        // In open weed locator mode, we log but don't spray
        let action = ActionTaken::LoggedOnly;
        
        // Log the weed detection with location
        logger.log_detection(Some(location.clone()), detection_info, action)?;
        
        println!(
            "Weed {} logged at ({:.6}, {:.6}) - Area: {:.1} px²",
            i, location.latitude, location.longitude, 300.0 + i as f64 * 50.0
        );
        
        // Small delay to simulate time between detections
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    // Flush the logger to ensure all data is written
    logger.flush()?;
    
    println!("\n=== Results ===");
    println!("Total weeds detected and logged: {}", logger.event_count());
    println!("Data saved to: {}.json", config.logging.output_file);
    
    if config.logging.format == "both" || config.logging.format == "csv" {
        println!("CSV data saved to: {}.csv", config.logging.output_file);
    }
    
    println!("\nThis data can be imported into GIS software for mapping!");
    
    Ok(())
}