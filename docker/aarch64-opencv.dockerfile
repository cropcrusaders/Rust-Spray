FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:0.2.7
RUN apt-get update && \
    apt-get install -y --no-install-recommends libopencv-dev:arm64 pkg-config ninja-build && \
    rm -rf /var/lib/apt/lists/*
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
