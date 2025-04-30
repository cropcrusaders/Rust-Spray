# 🌾 Rust-Spray

**Rust‑Spray** is an open‑source, real‑time section‑spray controller written in **Rust** for row‑crop, pasture and spot‑spray applications.  
It targets SBCs (Raspberry Pi 4/5, Jetson Nano/Xavier NX) running Linux and integrates with ISOBUS or stand‑alone PWM/boom‑valve hardware.

| CI | Crate | Docs | License |
|----|-------|------|---------|
| ![CI](https://github.com/your-org/Rust-Spray/actions/workflows/ci.yml/badge.svg) | coming&nbsp;soon | coming&nbsp;soon | MIT OR Apache‑2.0 + **Safety Rider** |

> **Safety Rider** – Users are **solely responsible** for safe operation; see `LICENSE-SAFETY.md`.

---

## ✨ Features
- **High‑speed CAN / ISOBUS I/O** (SocketCAN ≤ 1 kHz loop)
- **Section control**: up to 16 unique widths or 64 uniform widths
- **GPS‑driven rate & coverage logging** (NMEA 0183, u‑blox UBX, RTK corrections)
- **OpenCV hooks** for camera‑based weed detection & feed‑forward rate control
- **Modular HAL** – swap CAN backend, valve drivers or GPS sources without touching core logic
- **Async Tokio runtime** with watchdog & back‑pressure handling
- **Prometheus metrics endpoint** + optional Grafana dashboards
- **Pluggable UI**
  - CLI (`clap`)
  - gRPC service for remote control
  - Web dashboard starter (Tauri/Leptos – WiP)

---

## 🛠️ Quick Build

> Tested on **Ubuntu 22.04** and **Raspberry Pi OS Bookworm 64‑bit**.

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add aarch64-unknown-linux-gnu   # when cross‑compiling

# 2. Clone & build
git clone https://github.com/your-org/Rust-Spray.git
cd Rust-Spray
cargo build --release

# 3. Cross‑build for Raspberry Pi
sudo apt-get install gcc-aarch64-linux-gnu pkg-config libopencv-dev
cargo build --release --target aarch64-unknown-linux-gnu
```

### OpenCV note
`opencv = "0.89"` expects **system OpenCV ≥ 4.8.0**. If you hit build errors:

```bash
sudo apt-get install libopencv-core-dev libopencv-imgproc-dev                      libopencv-highgui-dev libopencv-videoio-dev
```

---

## 🔌 Hardware Matrix

| Component             | Example           | Notes                           |
|-----------------------|-------------------|---------------------------------|
| SBC                  | Raspberry Pi 5    | 64‑bit Bookworm                 |
| CAN HAT              | PiCAN 3           | SocketCAN                       |
| Boom valves          | TeeJet 344E       | PWM capable, up to 16           |
| Flow sensor          | DigiFlow 200      | 4 kHz sampling                  |
| GPS Receiver         | u‑blox F9P RTK    | NMEA or UBX serial              |
| Camera (optional)    | Pi HQ v2          | Vision rate control             |

Full pinouts & wiring live in the **Wiki → Hardware** page.

---

## 🚀 Running

```bash
sudo ./target/release/rust-spray   --sections 16   --gps /dev/ttyACM0   --can if=can0,bitrate=250000   --config config/default.toml
```

Logs go to `logs/` by default. Add `--json` for JSON lines.

---

## 🧩 Configuration

All runtime settings live in `config/*.toml`.

```toml
[gps]
source = "nmea"
baud   = 460800

[boom]
sections      = 16
valve_driver  = "teejet"
pwm_frequency = 50

[logging]
level = "info"
file  = "logs/rust_spray.log"
```

---

## 🤝 Contributing

1. **Fork** → feature branch (`git checkout -b feat/my-feature`)  
2. Commit with conventional commits (`feat:`, `fix:`)  
3. `cargo fmt && cargo clippy -- -D warnings`  
4. PR against `main`

We follow the [CropCrusaders RFC workflow](https://github.com/cropcrusaders/.github/tree/main/rfcs).

---

## 📜 License

Dual‑licensed under **MIT OR Apache‑2.0**.  
Commercial operators **must** comply with *Safety Rider* in `LICENSE-SAFETY.md`, which places liability on the end‑user and mandates preservation of safety features in derivative works.

---

## 🙋 Support / Contact

- Issues → GitHub **Issues** tab  
- Chat   → Discord `#rust-spray` (invite link in Wiki)  
- Commercial integration → <support@cropcrusaders.com.au>

Happy Spraying! 🌱
