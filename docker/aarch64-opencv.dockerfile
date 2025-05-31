FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main
RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        libopencv-dev:arm64 \
        pkg-config:arm64 \
        ninja-build:arm64 && \
    rm -rf /var/lib/apt/lists/*
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
