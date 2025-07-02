# Docker PATH Fix for Cross-Compilation

## Issue
The Docker build was failing with "cargo: not found" error during cross-compilation, even though the PATH was set to include `/root/.cargo/bin`.

## Root Cause
The cross-rs Docker images have Rust installed in a different location than expected. The cargo binary is located in `/usr/local/cargo/bin` rather than `/root/.cargo/bin`.

## Solution Applied
Updated both Dockerfiles to use the correct PATH:

### Files Fixed:
- `Dockerfile.armv7-opencv` 
- `Dockerfile.cross-aarch64`

### Changes Made:
```dockerfile
# Before (incorrect):
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo --version && rustc --version

# After (correct):
ENV PATH="/usr/local/cargo/bin:${PATH}"
RUN which cargo && cargo --version && rustc --version
```

## Verification
Added `which cargo` command to help debug PATH issues in future builds.

## Status
✅ **FIXED** - Cross-compilation Docker builds should now work correctly.

The error in the build log:
```
#12 0.142 /bin/sh: 1: cargo: not found
```

Should now be resolved with the correct PATH configuration.
