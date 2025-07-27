#[cfg(feature = "opencv")]
use opencv::core::Mat;

#[cfg(feature = "opencv")]
#[test]
fn link_opencv() -> opencv::Result<()> {
    let _mat = Mat::default();
    Ok(())
}

#[cfg(not(feature = "opencv"))]
#[test]
fn opencv_not_available() {
    // Test that passes when OpenCV is not available
    assert!(true, "OpenCV feature not enabled");
}
