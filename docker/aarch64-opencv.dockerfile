FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main
RUN find /etc/apt -name '*.list' -print0 \
        | xargs -0 sed -i \
            -e 's|archive.ubuntu.com/ubuntu|ports.ubuntu.com/ubuntu-ports|g' \
            -e 's|security.ubuntu.com/ubuntu|ports.ubuntu.com/ubuntu-ports|g' && \
    apt-get -o Acquire::Retries=3 update && \
    apt-get -o Acquire::Retries=3 --fix-missing install -y --no-install-recommends \
        libopencv-dev \
        pkg-config \
        ninja-build && \
    rm -rf /var/lib/apt/lists/*
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
