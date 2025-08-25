use rustspray::{
    exg::exg_mask,
    lanes::{reduce_lanes, LaneController},
    SprayCfg,
};

#[test]
fn exg_unit() {
    let rgb = [10u8, 50, 10, 50, 10, 10];
    let mut mask = [0u8; 2];
    exg_mask(&rgb, &mut mask, 16);
    assert_eq!(mask, [1, 0]);
}

#[test]
fn lane_reducer() {
    let mask = [1, 0, 0, 0, 0, 0, 0, 0]; // 4x2 image
    let ratios = reduce_lanes(&mask, 4, 2, 1.0);
    assert_eq!(ratios, [0.5, 0.0, 0.0, 0.0]);
}

#[test]
fn timing() {
    let cfg = SprayCfg {
        fire_ms: 40,
        holdoff_ms: 100,
        min_ratio: 0.01,
        hysteresis: 0.5,
        ..SprayCfg::default()
    };
    let mut ctrl = LaneController::new(cfg);
    let mut state = ctrl.update([0.02, 0.0, 0.0, 0.0], 0);
    assert_eq!(state[0], true);
    // within fire window
    state = ctrl.update([0.0, 0.0, 0.0, 0.0], 20);
    assert_eq!(state[0], true);
    // fire window expired -> off
    state = ctrl.update([0.0, 0.0, 0.0, 0.0], 20);
    assert_eq!(state[0], false);
    // cooldown prevents retrigger
    state = ctrl.update([0.02, 0.0, 0.0, 0.0], 20);
    assert_eq!(state[0], false);
    // after cooldown
    state = ctrl.update([0.02, 0.0, 0.0, 0.0], 80);
    assert_eq!(state[0], true);
}
