use opencv::core::Mat;

#[test]
fn link_opencv() -> opencv::Result<()> {
    let _mat = Mat::default();
    Ok(())
}
