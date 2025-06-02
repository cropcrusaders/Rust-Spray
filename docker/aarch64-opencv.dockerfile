FROM ubuntu:22.04

# Enable the ARM64 architecture for cross-compiling OpenCV. The default
# Ubuntu mirrors already provide arm64 packages, so we simply add the
# architecture and keep the existing sources list intact. This avoids
# 404 errors when the amd64 repositories are queried.
RUN dpkg --add-architecture arm64 \
    && dpkg --remove-architecture i386 || true \
    && apt-get -o Acquire::Retries=3 update \
    && apt-get -o Acquire::Retries=3 install -y --no-install-recommends \
        libopencv-dev:arm64 \
        pkg-config:arm64 \
        ninja-build:arm64 \
    && rm -rf /var/lib/apt/lists/*
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
