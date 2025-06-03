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

RUN rm -rf /etc/apt/sources.list.d/* && \
    # cross-rs base images restrict apt to amd64 packages only. Retain the
    # host repositories and add arm64 sources from the Ubuntu ports archive so
    # we can fetch dependencies for the sysroot.
    sed -Ei '/^deb \[/! s/^deb /deb [arch=amd64] /' /etc/apt/sources.list && \
    printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal main universe restricted\n' > /etc/apt/sources.list.d/arm64.list && \
    printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal-updates main universe restricted\n' >> /etc/apt/sources.list.d/arm64.list && \
    printf 'deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal-security main universe restricted\n' >> /etc/apt/sources.list.d/arm64.list && \
    dpkg --add-architecture arm64 && \
    dpkg --remove-architecture i386 || true && \
    apt-get -o Acquire::Retries=3 update && \
    apt-get -o Acquire::Retries=3 install -y --no-install-recommends \
        build-essential \
        gcc-aarch64-linux-gnu g++-aarch64-linux-gnu \
        cmake ninja-build git pkg-config \
        libgtk-3-dev:arm64 libjpeg-dev:arm64 libpng-dev:arm64 libtiff-dev:arm64 \
        libavcodec-dev:arm64 libavformat-dev:arm64 libswscale-dev:arm64 libv4l-dev:arm64 \
        libxvidcore-dev:arm64 libx264-dev:arm64 libtbb2:arm64 libtbb-dev:arm64 \
        libatlas-base-dev:arm64 libdc1394-22-dev:arm64 && \
    rm -rf /var/lib/apt/lists/*

ENV CC=aarch64-linux-gnu-gcc
ENV CXX=aarch64-linux-gnu-g++

# Build OpenCV for the aarch64 sysroot
WORKDIR /opt
RUN git clone --depth 1 -b 4.11.0 https://github.com/opencv/opencv.git && \
    mkdir build && cd build && \
    cmake -G Ninja ../opencv \
        -DCMAKE_INSTALL_PREFIX=/usr/local \
        -DBUILD_LIST=core,imgproc,highgui,imgcodecs,videoio,objdetect \
        -DBUILD_SHARED_LIBS=ON \
        -DWITH_IPP=OFF \
        -DCMAKE_BUILD_TYPE=Release && \
    ninja -j$(nproc) && ninja install && \
    rm -rf /opt/opencv

# Copy OpenCV to the location expected by cross
RUN mkdir -p /usr/aarch64-linux-gnu && \
    cp -r /usr/local/* /usr/aarch64-linux-gnu/

ENV PKG_CONFIG_PATH=/usr/aarch64-linux-gnu/lib/pkgconfig
