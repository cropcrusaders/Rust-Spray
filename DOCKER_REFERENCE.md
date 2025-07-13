# 🐳 Docker Quick Reference for Rust-Spray

## 📋 Available Docker Images to Build

### 🚀 **Cross-Compilation Images** (Root directory)
```bash
# ARM64 with OpenCV (full-featured)
docker build -f Dockerfile.cross-aarch64 -t rust-spray:cross-aarch64 .

# ARMv7 with OpenCV  
docker build -f Dockerfile.armv7-opencv -t rust-spray:armv7-opencv .

# Raspberry Pi optimized
docker build -f Dockerfile.pi-opencv -t rust-spray:pi-opencv .
docker build -f Dockerfile.pi-opencv-armv7 -t rust-spray:pi-opencv-armv7 .
```

### 🎯 **Native ARM Images** (docker/ directory)
```bash
# Native ARM64 build
docker build -f docker/Dockerfile.aarch64 -t rust-spray:native-aarch64 .

# Native ARMv7 build  
docker build -f docker/Dockerfile.armv7 -t rust-spray:native-armv7 .

# ARM64 with OpenCV
docker build -f docker/aarch64-opencv.dockerfile -t rust-spray:aarch64-opencv .
```

## 🎮 **Using the PowerShell Script**
```powershell
# Build specific image
.\scripts\docker-run.ps1 build-cross-aarch64

# Build all images
.\scripts\docker-run.ps1 build-all

# Run a container
.\scripts\docker-run.ps1 run-aarch64

# Development mode (mount source)
.\scripts\docker-run.ps1 dev-run

# Clean up all images
.\scripts\docker-run.ps1 clean

# List current images
.\scripts\docker-run.ps1 list
```

## 🎮 **Using the Bash Script** (Linux/macOS/WSL)
```bash
# Make executable
chmod +x scripts/docker-run.sh

# Build specific image
./scripts/docker-run.sh build-cross-aarch64

# Build all images
./scripts/docker-run.sh build-all

# Run container
./scripts/docker-run.sh run-aarch64

# Development mode
./scripts/docker-run.sh dev-run

# Clean up
./scripts/docker-run.sh clean
```

## 🔧 **Manual Docker Commands**

### Build Commands
```bash
# Cross-compilation for ARM64 (currently building)
docker build -f Dockerfile.cross-aarch64 -t rust-spray:cross-aarch64 .

# Build with build arguments
docker build -f Dockerfile.cross-aarch64 \
  --build-arg OPENCV_VERSION=4.11.0 \
  --build-arg CMAKE_BUILD_TYPE=Release \
  -t rust-spray:cross-aarch64 .
```

### Run Commands
```bash
# Run interactive container
docker run -it --rm rust-spray:cross-aarch64

# Run with volume mount (development)
docker run -it --rm -v $(pwd):/workspace rust-spray:cross-aarch64

# Run with custom command
docker run --rm rust-spray:cross-aarch64 /usr/local/bin/rustspray --help

# Run with environment variables
docker run --rm -e RUST_LOG=debug rust-spray:cross-aarch64
```

### Management Commands
```bash
# List images
docker images | grep rust-spray

# Remove specific image
docker rmi rust-spray:cross-aarch64

# Remove all rust-spray images
docker images | grep rust-spray | awk '{print $3}' | xargs docker rmi -f

# View build history
docker history rust-spray:cross-aarch64

# Inspect image
docker inspect rust-spray:cross-aarch64
```

### Debug Commands
```bash
# Run shell in container
docker run -it --rm rust-spray:cross-aarch64 /bin/bash

# Check container logs
docker logs <container_id>

# Copy file from container
docker cp <container_id>:/usr/local/bin/rustspray ./rustspray-arm64

# View layer sizes
docker history --human --format "table {{.Size}}\t{{.CreatedBy}}" rust-spray:cross-aarch64
```

## 🏗️ **Build Process Overview**

### Current Build (Dockerfile.cross-aarch64):
1. **Stage 1**: OpenCV Build for ARM64
   - ✅ Ubuntu 22.04 base image
   - ✅ Install cross-compilation toolchain  
   - 🔄 Build OpenCV from source
   
2. **Stage 2**: Rust Build with Cross
   - 🔄 Use cross-rs image for ARM64
   - 🔄 Install Rust toolchain
   - 🔄 Copy OpenCV libraries
   - 🔄 Build Rust-Spray binary

3. **Stage 3**: Runtime Image
   - 🔄 Minimal Ubuntu base
   - 🔄 Copy OpenCV runtime libraries
   - 🔄 Copy compiled binary

### Estimated Build Time:
- **First build**: 15-30 minutes (OpenCV compilation)
- **Subsequent builds**: 5-10 minutes (cached layers)

## 🐛 **Troubleshooting**

### Common Issues:
```bash
# Docker not running
docker info  # Should show docker daemon info

# Permission issues (Linux)
sudo usermod -aG docker $USER  # Add user to docker group
newgrp docker  # Refresh groups

# Out of space
docker system prune -a  # Clean up unused images/containers

# Build context too large
# Check .dockerignore file for exclusions
```

### Build Failures:
```bash
# Clean build (no cache)
docker build --no-cache -f Dockerfile.cross-aarch64 -t rust-spray:cross-aarch64 .

# Debug build
docker build --progress=plain -f Dockerfile.cross-aarch64 -t rust-spray:cross-aarch64 .

# Inspect failed layer
docker run -it <failed_image_id> /bin/bash
```

## 📊 **Expected Outputs**

### Successful Build:
- ✅ `rust-spray:cross-aarch64` image (~800MB-1.2GB)
- ✅ ARM64 binary at `/usr/local/bin/rustspray`
- ✅ OpenCV libraries at `/opt/opencv/lib`

### Test the Build:
```bash
# Verify binary architecture
docker run --rm rust-spray:cross-aarch64 file /usr/local/bin/rustspray
# Should show: ARM aarch64

# Test application help
docker run --rm rust-spray:cross-aarch64 /usr/local/bin/rustspray --help
```

## 🎯 **Next Steps**

Once the current build completes:
1. Test the ARM64 binary
2. Build other Docker variants
3. Push to container registry (optional)
4. Deploy to ARM devices (Raspberry Pi, etc.)

**Current Status**: ✅ Docker is building successfully!
