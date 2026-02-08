//! ONNX model inference for vegetation detection.
//!
//! Enable with `cargo build --features model`.
//!
//! Train a model with `scripts/train_model.py`, which exports a small
//! vegetation classifier to ONNX format.  Load the resulting
//! `vegetation.onnx` file at runtime with [`ModelDetector::load`].

#[cfg(feature = "model")]
use tract_onnx::prelude::*;

#[cfg(feature = "model")]
use crate::vision::Detector;

/// Vegetation detector backed by an ONNX model.
///
/// The model should accept an `[N, 3]` float32 input (batched RGB pixels
/// normalised to 0–1) and produce an `[N, 1]` float32 output where values
/// above `threshold` are classified as vegetation.
///
/// # Example
///
/// ```ignore
/// use rustspray::model::ModelDetector;
///
/// let detector = ModelDetector::load("vegetation.onnx", 640, 480, 0.5);
/// let mask = detector.detect(&frame_rgb);
/// ```
#[cfg(feature = "model")]
pub struct ModelDetector {
    model: TypedRunnableModel<TypedModel>,
    threshold: f32,
}

#[cfg(feature = "model")]
impl ModelDetector {
    /// Load an ONNX model from `path`.
    ///
    /// * `path` - filesystem path to the `.onnx` file.
    /// * `width` / `height` - expected frame dimensions (used to fix the
    ///   batch axis so tract can optimise the graph).
    /// * `threshold` - output probability above which a pixel counts as
    ///   vegetation.
    pub fn load(path: &str, width: usize, height: usize, threshold: f32) -> Self {
        let n_pixels = width * height;
        let model = tract_onnx::onnx()
            .model_for_path(path)
            .expect("failed to load ONNX model")
            .with_input_fact(
                0,
                InferenceFact::dt_shape(f32::datum_type(), &[n_pixels.to_dim(), 3.to_dim()]),
            )
            .expect("failed to set input shape")
            .into_optimized()
            .expect("failed to optimise model")
            .into_runnable()
            .expect("failed to create runnable model");
        Self { model, threshold }
    }
}

#[cfg(feature = "model")]
impl Detector for ModelDetector {
    fn detect(&self, rgb: &[u8]) -> Vec<bool> {
        assert!(rgb.len() % 3 == 0, "RGB slice must be multiple of 3",);
        let n_pixels = rgb.len() / 3;

        // Normalise u8 RGB to f32 [0, 1].
        let data: Vec<f32> = rgb.iter().map(|&b| b as f32 / 255.0).collect();

        let input = tract_ndarray::Array2::from_shape_vec((n_pixels, 3), data)
            .expect("failed to reshape input");
        let result = self
            .model
            .run(tvec!(input.into_tvalue()))
            .expect("model inference failed");
        let output = result[0]
            .to_array_view::<f32>()
            .expect("failed to read model output");

        output.iter().map(|&v| v > self.threshold).collect()
    }
}
