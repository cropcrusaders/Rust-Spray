use rustspray::config::Config;

#[test]
fn load_config_values() {
    let cfg = Config::load("config/Config.toml").expect("load config");
    assert_eq!(cfg.camera.device, "/dev/video2");
    assert!(!cfg.camera.use_rpi_cam);
    assert_eq!(cfg.spray.pins[0], 23);
}
