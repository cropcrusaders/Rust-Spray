use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustSprayError {
    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),
    #[error("Camera error: {0}")]
    Camera(String),
    #[error("Detection error: {0}")]
    Detection(String),
    #[error("Spray error: {0}")]
    Spray(#[from] crate::spray::SprayError),
    #[error("OpenCV error: {0}")]
    OpenCV(#[from] opencv::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
