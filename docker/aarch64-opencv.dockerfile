FROM ubuntu:22.04

# Enable the ARM64 architecture. The default Ubuntu sources only host
# amd64 packages, so switch to the ports repository which provides
# binaries for additional architectures before attempting to install
# ARM64 packages.
RUN dpkg --add-architecture arm64 \
    && dpkg --remove-architecture i386 || true \
    && sed -i 's|http://archive.ubuntu.com/ubuntu|http://ports.ubuntu.com/ubuntu-ports|g' /etc/apt/sources.list \
    && apt-get -o Acquire::Retries=3 update \
    && apt-get -o Acquire::Retries=3 install -y --no-install-recommends \
        libopencv-dev:arm64 \
        pkg-config:arm64 \
        ninja-build:arm64 \
    && rm -rf /var/lib/apt/lists/*
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
