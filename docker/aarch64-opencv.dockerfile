FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main
RUN rm -f /etc/apt/sources.list.d/ports.list \
    && printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal main universe\n' \
           'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal-updates main universe\n' \
           'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal-security main universe\n' \
           'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal-backports main universe\n' \
           > /etc/apt/sources.list \
    && dpkg --add-architecture arm64 || true \
    && dpkg --remove-architecture amd64 || true \
    && dpkg --remove-architecture i386 || true \
    && test -z "$(dpkg --print-foreign-architectures)" \
    && apt-get -o Acquire::Retries=3 update \
    && apt-get -o Acquire::Retries=3 install -y --no-install-recommends \
        libopencv-dev \
        pkg-config \
        ninja-build \
    && rm -rf /var/lib/apt/lists/*
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
