# Start from the official cross image so the required
# cross-compilation tooling and pkg-config wrappers are
# already available. This avoids "pkg-config has not been" errors
# when building OpenCV crates.
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

# Reset package sources to avoid duplicate entries from the base image
RUN rm -rf /etc/apt/sources.list.d/* && \
    printf 'deb http://ports.ubuntu.com/ubuntu-ports focal main universe\n' \
           'deb http://ports.ubuntu.com/ubuntu-ports focal-updates main universe\n' \
           'deb http://ports.ubuntu.com/ubuntu-ports focal-security main universe\n' \
           'deb http://ports.ubuntu.com/ubuntu-ports focal-backports main universe\n' \
           > /etc/apt/sources.list && \
    dpkg --add-architecture arm64 && \
    dpkg --remove-architecture i386 || true && \
    apt-get -o Acquire::Retries=3 update && \
    apt-get -o Acquire::Retries=3 install -y --no-install-recommends \
        build-essential \
        gcc-aarch64-linux-gnu g++-aarch64-linux-gnu \
        libc6-dev-arm64-cross linux-libc-dev-arm64-cross \
        libopencv-dev:arm64 \
        libopencv-core-dev:arm64 \
        libopencv-imgproc-dev:arm64 \
        libopencv-highgui-dev:arm64 \
        libopencv-imgcodecs-dev:arm64 \
        libopencv-videoio-dev:arm64 \
        libopencv-objdetect-dev:arm64 \
        pkg-config \
        ninja-build && \
    rm -rf /var/lib/apt/lists/*

ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
