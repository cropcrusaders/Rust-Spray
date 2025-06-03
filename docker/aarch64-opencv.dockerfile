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
    dpkg --add-architecture arm64 && \
    dpkg --remove-architecture i386 || true && \
    apt-get -o Acquire::Retries=3 update && \
    apt-get -o Acquire::Retries=3 install -y --no-install-recommends \
        build-essential \
        gcc-aarch64-linux-gnu g++-aarch64-linux-gnu \
        cmake ninja-build git pkg-config \
        libgtk-3-dev libjpeg-dev libpng-dev libtiff-dev \
        libavcodec-dev libavformat-dev libswscale-dev libv4l-dev \
        libxvidcore-dev libx264-dev gfortran libtbb2 libtbb-dev \
        libatlas-base-dev libdc1394-22-dev && \
    rm -rf /var/lib/apt/lists/*

ENV CC=aarch64-linux-gnu-gcc
ENV CXX=aarch64-linux-gnu-g++

# Build OpenCV for the aarch64 sysroot
WORKDIR /opt
RUN git clone --depth 1 -b 4.8.1 https://github.com/opencv/opencv.git && \
    mkdir build && cd build && \
    cmake -G Ninja ../opencv \
        -DCMAKE_INSTALL_PREFIX=/usr/local \
        -DBUILD_LIST=core,imgproc,highgui,imgcodecs,videoio,objdetect \
        -DBUILD_SHARED_LIBS=ON \
        -DCMAKE_BUILD_TYPE=Release && \
    ninja -j$(nproc) && ninja install && \
    rm -rf /opt/opencv

# Copy OpenCV to the location expected by cross
RUN mkdir -p /usr/aarch64-linux-gnu && \
    cp -r /usr/local/* /usr/aarch64-linux-gnu/

ENV PKG_CONFIG_PATH=/usr/aarch64-linux-gnu/lib/pkgconfig
