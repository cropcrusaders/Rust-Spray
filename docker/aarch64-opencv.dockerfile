# Start from the official cross image so the required
# cross-compilation tooling and pkg-config wrappers are
# already available. This avoids "pkg-config has not been
# configured" errors when building OpenCV crates.
FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:edge

# Configure tzdata non-interactively so image builds do not block waiting
# for timezone selection when a package pulls it in as a dependency.
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Australia/Brisbane

RUN apt-get update && \
    apt-get install -y --no-install-recommends tzdata && \
    ln -fs /usr/share/zoneinfo/${TZ} /etc/localtime && \
    echo ${TZ} > /etc/timezone && \
    dpkg-reconfigure -f noninteractive tzdata && \
    rm -rf /var/lib/apt/lists/*

# Enable the ARM64 architecture for cross-compiling OpenCV. The default
# sources in the Ubuntu image only contain packages for the host
# architecture.  When we add a foreign architecture apt will attempt to
# fetch `binary-arm64` indexes from the `archive.ubuntu.com` mirror which
# does not host them, resulting in 404 errors.  Use the ports mirror for
# arm64 packages instead.
RUN dpkg --add-architecture arm64 \
    && dpkg --remove-architecture i386 || true \
    && sed -Ei '/^deb \[/! s/^deb /deb [arch=amd64] /' /etc/apt/sources.list \
    && printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal main restricted universe multiverse\n' > /etc/apt/sources.list.d/arm64.list \
    && printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal-updates main restricted universe multiverse\n' >> /etc/apt/sources.list.d/arm64.list \
    && printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal-security main restricted universe multiverse\n' >> /etc/apt/sources.list.d/arm64.list \
    && printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal-backports main restricted universe multiverse\n' >> /etc/apt/sources.list.d/arm64.list \
    && apt-get -o Acquire::Retries=3 update \
    && apt-get -o Acquire::Retries=3 install -y --no-install-recommends \
        build-essential \
        gcc-aarch64-linux-gnu g++-aarch64-linux-gnu \
        libc6-dev-arm64-cross linux-libc-dev-arm64-cross \
        libopencv-core-dev:arm64 \
        libopencv-imgproc-dev:arm64 \
        libopencv-highgui-dev:arm64 \
        libopencv-imgcodecs-dev:arm64 \
        libopencv-videoio-dev:arm64 \
        libopencv-objdetect-dev:arm64 \
        pkg-config \
        ninja-build \
        && rm -rf /var/lib/apt/lists/*
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
