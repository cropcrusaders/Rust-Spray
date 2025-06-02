FROM ubuntu:22.04

# Enable the ARM64 architecture for cross-compiling OpenCV. The default
# sources in the Ubuntu image only contain packages for the host
# architecture.  When we add a foreign architecture apt will attempt to
# fetch `binary-arm64` indexes from the `archive.ubuntu.com` mirror which
# does not host them, resulting in 404 errors.  Use the ports mirror for
# arm64 packages instead.
RUN dpkg --add-architecture arm64 \
    && dpkg --remove-architecture i386 || true \
    && sed -Ei 's/^deb /deb [arch=amd64] /' /etc/apt/sources.list \
    && printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports jammy main restricted universe multiverse\n' > /etc/apt/sources.list.d/arm64.list \
    && printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports jammy-updates main restricted universe multiverse\n' >> /etc/apt/sources.list.d/arm64.list \
    && printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports jammy-security main restricted universe multiverse\n' >> /etc/apt/sources.list.d/arm64.list \
    && printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports jammy-backports main restricted universe multiverse\n' >> /etc/apt/sources.list.d/arm64.list \
    && apt-get -o Acquire::Retries=3 update \
    && apt-get -o Acquire::Retries=3 install -y --no-install-recommends \
        libopencv-dev:arm64 \
        pkg-config:arm64 \
        ninja-build:arm64 \
    && rm -rf /var/lib/apt/lists/*
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
