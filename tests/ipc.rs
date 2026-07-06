//! End-to-end tests of `rustspray --ipc-mode` over real pipes.
//!
//! These spawn the actual production binary (via `CARGO_BIN_EXE_rustspray`)
//! and speak IPC protocol v1 to it, the same way a host process such as
//! OpenWeedLocator would.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 16;

fn spawn_ipc() -> (Child, ChildStdin, BufReader<std::process::ChildStdout>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustspray"))
        .args([
            "--ipc-mode",
            "--mock-gpio",
            // Missing config file -> compiled-in defaults (4 lanes).
            "--config",
            "/nonexistent/rustspray-ipc-test.toml",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn rustspray");
    let stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    (child, stdin, stdout)
}

/// Header + payload for one frame; `green` paints the left half green.
fn encode_frame(width: u32, height: u32, green: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + (width * height * 3) as usize);
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    for _y in 0..height {
        for x in 0..width {
            if green && x < width / 2 {
                out.extend_from_slice(&[20, 200, 20]);
            } else {
                out.extend_from_slice(&[120, 90, 70]);
            }
        }
    }
    out
}

fn read_response(stdout: &mut BufReader<std::process::ChildStdout>) -> serde_json::Value {
    let mut line = String::new();
    stdout.read_line(&mut line).expect("read response line");
    serde_json::from_str(&line).expect("valid JSON response")
}

#[test]
fn frames_produce_lane_state_json() {
    let (mut child, mut stdin, mut stdout) = spawn_ipc();

    // Frame 1: left half green -> lanes 0+1 of 4 on.
    stdin.write_all(&encode_frame(WIDTH, HEIGHT, true)).unwrap();
    stdin.flush().unwrap();
    let resp = read_response(&mut stdout);
    assert_eq!(resp["v"], 1);
    assert_eq!(resp["frame"], 1);
    assert_eq!(resp["lanes"], serde_json::json!([true, true, false, false]));
    let ts_us = resp["ts_us"].as_u64().unwrap();
    assert!(ts_us > 1_600_000_000_000_000, "ts_us must be unix micros");
    assert!(resp["latency_us"].as_u64().unwrap() < 5_000_000);

    // Frame 2: all soil -> everything off, counter increments.
    stdin
        .write_all(&encode_frame(WIDTH, HEIGHT, false))
        .unwrap();
    stdin.flush().unwrap();
    let resp = read_response(&mut stdout);
    assert_eq!(resp["frame"], 2);
    assert_eq!(
        resp["lanes"],
        serde_json::json!([false, false, false, false])
    );

    // Closing stdin is a clean end-of-input.
    drop(stdin);
    let status = child.wait().unwrap();
    assert!(status.success(), "clean EOF must exit 0, got {status:?}");
}

#[test]
fn dimension_change_is_a_fatal_protocol_error() {
    let (mut child, mut stdin, mut stdout) = spawn_ipc();

    stdin.write_all(&encode_frame(WIDTH, HEIGHT, true)).unwrap();
    stdin.flush().unwrap();
    assert_eq!(read_response(&mut stdout)["frame"], 1);

    // Different dimensions mid-stream must kill the process (exit 1).
    stdin
        .write_all(&encode_frame(WIDTH * 2, HEIGHT, true))
        .unwrap();
    stdin.flush().unwrap();
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(1));
}

#[test]
fn truncated_stream_is_a_fatal_protocol_error() {
    let (mut child, mut stdin, _stdout) = spawn_ipc();

    let mut frame = encode_frame(WIDTH, HEIGHT, true);
    frame.truncate(frame.len() / 2);
    stdin.write_all(&frame).unwrap();
    stdin.flush().unwrap();
    drop(stdin); // EOF mid-payload
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(1));
}

#[test]
fn output_version_reports_ipc_protocol() {
    let out = Command::new(env!("CARGO_BIN_EXE_rustspray"))
        .arg("--output-version")
        .output()
        .expect("run --output-version");
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    assert_eq!(v["ipc_protocol"], 1);
    assert_eq!(v["rustspray_version"], env!("CARGO_PKG_VERSION"));
}
