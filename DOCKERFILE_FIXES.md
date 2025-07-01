# Dockerfile Cross-Compilation Fixes

## Root Cause Analysis

The CI/CD pipeline was failing with the error:
```
/bin/sh: 1: rustup: not found
```

This occurred because the Dockerfiles were incorrectly trying to use `rustup` commands inside cross-rs Docker images. Cross-rs images (like `ghcr.io/cross-rs/armv7-unknown-linux-gnueabihf:edge`) come pre-configured with the appropriate Rust toolchain for the target architecture and **do not include rustup**.

## Issues Fixed

### 1. `Dockerfile.armv7-opencv`
**Before:**
```dockerfile
ARG RUST_TOOLCHAIN=stable
FROM ghcr.io/cross-rs/armv7-unknown-linux-gnueabihf:edge AS rust-build
ARG RUST_TOOLCHAIN

RUN rustup default ${RUST_TOOLCHAIN}  # ❌ rustup not available
RUN cargo install --git https://github.com/cross-rs/cross cross --locked
RUN cross build --release --target armv7-unknown-linux-gnueabihf  # ❌ wrong approach
COPY --from=rust-build /workspace/target/armv7-unknown-linux-gnueabihf/release/rustspray  # ❌ wrong path
```

**After:**
```dockerfile
FROM ghcr.io/cross-rs/armv7-unknown-linux-gnueabihf:edge AS rust-build

# Cross-rs images come with pre-configured Rust toolchain for the target
# No need to install rustup or change toolchain
RUN cargo --version && rustc --version  # ✅ verify toolchain
RUN cargo build --release  # ✅ use cargo directly
COPY --from=rust-build /workspace/target/release/rustspray  # ✅ correct path
```

### 2. `Dockerfile.cross-aarch64`
**Before:**
```dockerfile
ARG RUST_TOOLCHAIN=stable
FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main AS rust-build

RUN rustup default ${RUST_TOOLCHAIN}  # ❌ rustup not available
RUN cargo install --git https://github.com/cross-rs/cross cross --locked  # ❌ unnecessary
RUN cross build --release --target aarch64-unknown-linux-gnu  # ❌ wrong approach
COPY --from=rust-build /workspace/target/aarch64-unknown-linux-gnu/release/rustspray  # ❌ wrong path
```

**After:**
```dockerfile
FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main AS rust-build

# Cross-rs images come with pre-configured Rust toolchain for the target
# No need to install rustup or change toolchain
RUN cargo --version && rustc --version  # ✅ verify toolchain
RUN cargo build --release  # ✅ use cargo directly
COPY --from=rust-build /workspace/target/release/rustspray  # ✅ correct path
```

## Key Changes Summary

1. **Removed rustup dependencies**: Cross-rs images don't have rustup and don't need it
2. **Removed cross installation**: Cross should be used from the host, not inside the container
3. **Use cargo directly**: Inside cross-rs containers, use `cargo build` instead of `cross build`
4. **Fixed binary paths**: Cross-rs containers output to `target/release/` not target-specific subdirectories
5. **Added verification**: Simple `cargo --version && rustc --version` to verify the toolchain

## How Cross-rs Images Work

Cross-rs Docker images are specifically designed for cross-compilation:

1. **Pre-configured toolchain**: They come with the appropriate Rust toolchain already installed for the target architecture.
2. **No rustup needed**: Since the toolchain is pre-configured, there's no need to install or use rustup.
3. **Direct cargo usage**: You can use `cargo build` directly instead of `cross build` when inside the container.
4. **Standard target directory**: The compiled binaries go to `target/release/` rather than the target-specific subdirectory.
5. **Sysroot configured**: The system libraries and headers for the target architecture are pre-installed.

## CI/CD Integration

The GitHub Actions workflow correctly builds these Docker images:
- `docker/aarch64-opencv.dockerfile` → `ghcr.io/.../aarch64-opencv:latest`
- `Dockerfile.armv7-opencv` → `ghcr.io/.../armv7-opencv:latest`

These images are then referenced in `Cross.toml` for cross-compilation:
```toml
[target.aarch64-unknown-linux-gnu]
image = "ghcr.io/${GHCR_USER}/aarch64-opencv:latest"

[target.armv7-unknown-linux-gnueabihf]
image = "ghcr.io/${GHCR_USER}/armv7-opencv:latest"
```

## Expected Results

These fixes should resolve the CI/CD pipeline failures related to cross-compilation for ARM targets. The Docker builds should now:

1. ✅ Successfully use the pre-configured Rust toolchain in the cross-rs images
2. ✅ Build the project without attempting to install or configure rustup
3. ✅ Find the compiled binaries in the correct location for the runtime stage
4. ✅ Complete the full cross-compilation pipeline for ARM64 and ARMv7 targets

## Testing

To test these fixes locally (requires Docker):

```bash
# For ARMv7 (builds both OpenCV and Rust components)
docker build -f Dockerfile.armv7-opencv -t rust-spray:armv7 .

# For AArch64 (using the aarch64-opencv.dockerfile for OpenCV)
docker build -f docker/aarch64-opencv.dockerfile -t aarch64-opencv .
docker build -f Dockerfile.cross-aarch64 -t rust-spray:aarch64 .
```

The CI/CD pipeline should now pass for these cross-compilation targets.

## Files Modified

- `Dockerfile.armv7-opencv` - Fixed rustup issues and build commands
- `Dockerfile.cross-aarch64` - Fixed rustup issues and build commands  
- `build_check.ps1` - Added CI/CD status information
- `DOCKERFILE_FIXES.md` - This documentation file
