# Rust-Spray

Rust-Spray is a small example project that uses a camera to detect weeds and pulse up to four sprayer outputs via GPIO pins. It targets Linux boards such as the Raspberry Pi. The detection pipeline is implemented with OpenCV in Rust.

## Beginner Quickstart

1. **Install Rust, Cargo and OpenCV development libraries**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   # Add cargo to PATH for the current shell
   source "$HOME/.cargo/env"
   sudo apt-get update
   sudo apt-get install libopencv-dev pkg-config build-essential

   cargo install cargo-opencv --git https://github.com/twistedfall/opencv-rust
   ```
2. **Clone this repository**
   ```bash
   git clone https://github.com/cropcrusaders/Rust-Spray.git
   cd Rust-Spray
   ```
3. **Build the project**
   ```bash
   cargo build --release
   ```
   For Raspberry Pi targets install the appropriate cross toolchain and add the target.
   For 64-bit OS:
   ```bash
   sudo apt-get install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
   rustup target add aarch64-unknown-linux-gnu
   ```
   For 32-bit OS:
   ```bash
   sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf
   rustup target add armv7-unknown-linux-gnueabihf
   ```
   Then build with:
   ```bash
   cargo build --release --target aarch64-unknown-linux-gnu
   # or for 32-bit
   cargo build --release --target armv7-unknown-linux-gnueabihf
   ```
   Add `--features picam` if the Raspberry Pi camera module is required.

4. **Copy the example configuration**
   ```bash
   cp config/Config.toml config/config.toml
   ```
5. **Run the binary** (use sudo for GPIO access):
   ```bash
   sudo ./target/release/rust-spray --config config/config.toml --show-display
   ```

## Features

- Capture frames from a USB or Raspberry Pi camera.
- Weed detection using ExG or HSV colour thresholds.
- Control four GPIO-driven sprayers via rppal.
- Configuration via `config/config.toml`.
- Optional display window for debugging.

## Building

1. Install Rust (via [rustup](https://rustup.rs)) and OpenCV development libraries.
2. Clone the repository:
   ```bash
   git clone https://github.com/cropcrusaders/Rust-Spray.git
   cd Rust-Spray
   ```
3. Build for the host platform:
   ```bash
   cargo build --release
   ```

To cross compile for Raspberry Pi choose the appropriate target.
For 64-bit OS:
   ```bash
   sudo apt-get install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
   rustup target add aarch64-unknown-linux-gnu
   cargo build --release --target aarch64-unknown-linux-gnu
   ```
For 32-bit OS:
   ```bash
   sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf
   rustup target add armv7-unknown-linux-gnueabihf
   cargo build --release --target armv7-unknown-linux-gnueabihf
   ```
Add `--features picam` when the Raspberry Pi camera module is required.
The repository's `.cargo/config.toml` configures the linker as `aarch64-linux-gnu-gcc` for this target.
It also defines a linker for the 32-bit `armv7-unknown-linux-gnueabihf` target so you can cross compile for older Raspberry Pi models.
Alternatively, uncomment the `target` line in `.cargo/config.toml` to make cross compilation the default.
When using `cross`, ensure Docker is available. If it prints `Falling back to cargo on the host`, the container could not start and the build may fail.
Install the `aarch64-linux-gnu` or `arm-linux-gnueabihf` toolchain and run `cargo build --target <target>` as a fallback.

## Building with Docker

If you prefer to build inside a container you can create the images used by
`cross` from the provided Dockerfiles.

```bash
# For 64-bit ARM targets
docker build -f Dockerfile.pi-opencv -t ghcr.io/<your-namespace>/aarch64-opencv:latest .

# For 32-bit ARM targets
docker build -f Dockerfile.pi-opencv-armv7 -t ghcr.io/<your-namespace>/armv7-opencv:latest .
```

These Dockerfiles install common build tools and now include
`libunwind-dev` for the target architecture. This resolves missing
dependencies when building ROS 2 packages such as `nav2` inside the
container.

When publishing these images to GitHub Container Registry (GHCR) you must
provide a `GHCR_TOKEN` secret with `write:packages` permission. The
workflows use `${{ github.repository_owner }}` as the namespace for the
image tags, so the token needs permission to push to that account.

To **pull** these images from GHCR you must also authenticate. Use a
Personal Access Token (PAT) with at least `read:packages` permission and
log in before running `docker pull`:

```bash
echo <your_token> | docker login ghcr.io -u <your_username> --password-stdin
```

In GitHub Actions set the workflow permissions so the automatically
generated `GITHUB_TOKEN` can read packages, then authenticate using the
Docker login action before pulling:

```yaml
permissions:
  packages: read
  contents: read

steps:
  - uses: actions/checkout@v4
  - name: Log in to GHCR
    uses: docker/login-action@v3
    with:
      registry: ghcr.io
      username: ${{ github.repository_owner }}
      password: ${{ secrets.GITHUB_TOKEN }}
```

Install [`cross`](https://github.com/cross-rs/cross) from the GitHub repository
and build using the image. The crate is no longer published on crates.io, so the
`--git` option must be used. You may lock to a tag such as `v0.2.6` or
`v0.2.7`, but it is also fine to install from the default branch. The Docker
image tag does not have to match the commit used for the CLI:

```bash
cargo install cross --git https://github.com/cross-rs/cross --locked
cross build --release --target aarch64-unknown-linux-gnu
```

Replace the target as needed. You can also run arbitrary commands inside the
image, for example:

```bash
docker run --rm -it -v $(pwd):/project -w /project \
  ghcr.io/<your-namespace>/aarch64-opencv:latest cargo test
```

An all-in-one Dockerfile named `Dockerfile.cross-aarch64` is provided for
convenience. It builds OpenCV from source and then cross-compiles the project
for `aarch64-unknown-linux-gnu` in a single multi-stage image. Build it with:

```bash
docker buildx build --platform linux/arm64 -t ghcr.io/<your-namespace>/rustspray:latest \
  -f Dockerfile.cross-aarch64 .
```
The resulting image contains `/usr/local/bin/rustspray` together with the
required OpenCV runtime libraries.

### Windows

Rust-Spray targets Linux boards, but you can cross compile from Windows using
the Docker setup above. Install [Docker Desktop](https://www.docker.com/products/docker-desktop)
with the WSL2 backend and ensure `cargo` is available (either through WSL or
native rustup). Clone the repository and run:

```bash
git clone https://github.com/cropcrusaders/Rust-Spray.git
cd Rust-Spray
cargo install cross --git https://github.com/cross-rs/cross --locked
cross build --release --target aarch64-unknown-linux-gnu
```

Make sure Docker Desktop is running so that `cross` can launch its container.
You may also run other commands with `docker run` as shown above.

### Pre-built Raspberry Pi Package

GitHub Actions builds a Debian package for Raspberry Pi (ARM64) on every
release. Download the latest `rustspray_*.deb` from the
[releases page](https://github.com/cropcrusaders/Rust-Spray/releases) and
install it on the Pi with:

```bash
sudo dpkg -i rustspray_*_arm64.deb
```


## Configuration

Copy `config/Config.toml` to `config/config.toml` and edit as needed:

```toml
[camera]
device = "/dev/video2"
resolution_width = 1280
resolution_height = 720
use_rpi_cam = false

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
```

## Hardware Setup

- Connect a USB webcam or the Raspberry Pi camera module.
- Wire solenoid valves (or other actuators) to GPIO pins 23–26 as shown above.
- Provide external drivers and power for the valves and verify fail‑safe behaviour.
- Optional peripherals such as CAN bus adapters, flow sensors and GPS receivers can also be attached.
- See [wiki/Wiring.md](wiki/Wiring.md) for diagrams and more details.

## Running

```bash
sudo ./target/release/rust-spray --config config/config.toml --show-display
```

The program opens the camera, runs the detection algorithm and pulses the sprayers whenever weeds are detected. Use `--show-display` to view the annotated video stream.

## License

Rust-Spray is released under the MIT license.

## Yocto Build

A minimal Yocto configuration is provided in the `yocto/` directory. It builds
a small graphical demo image containing Rust-Spray using Poky. To build on a
machine with the Yocto build dependencies installed:

```bash
cd yocto
git clone --depth 1 https://git.yoctoproject.org/git/poky poky
git clone --depth 1 https://github.com/openembedded/meta-openembedded.git meta-openembedded
source poky/oe-init-build-env build
bitbake rust-spray-image
```

See `yocto/README.md` for more details.
