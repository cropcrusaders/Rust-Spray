FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# Install build tools
RUN apt-get update && apt-get install -y \
    curl git unzip pkg-config cmake ninja-build build-essential \
    clang libclang-dev libgtk-3-dev libv4l-dev libavcodec-dev libavformat-dev libswscale-dev \
    libtbb2 libtbb-dev libjpeg-dev libpng-dev libtiff-dev libopenexr-dev libwebp-dev \
    python3-dev python3-pip tzdata

# Set timezone non-interactively
RUN ln -fs /usr/share/zoneinfo/UTC /etc/localtime && dpkg-reconfigure -f noninteractive tzdata

# Build OpenCV for ARM64
WORKDIR /opt
RUN git clone --depth=1 -b 4.9.0 https://github.com/opencv/opencv.git && \
    git clone --depth=1 -b 4.9.0 https://github.com/opencv/opencv_contrib.git

WORKDIR /opt/opencv/build
RUN cmake -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX=/opt/opencv-aarch64 \
    -DCMAKE_TOOLCHAIN_FILE=/usr/share/cmake-*/Modules/Platform/Linux-aarch64.cmake \
    -DOPENCV_EXTRA_MODULES_PATH=/opt/opencv_contrib/modules \
    -DBUILD_SHARED_LIBS=ON \
    -DWITH_QT=OFF -DWITH_OPENGL=ON \
    -DWITH_GTK=ON -DWITH_V4L=ON \
    -DBUILD_TESTS=OFF -DBUILD_PERF_TESTS=OFF \
    -DBUILD_opencv_python_bindings_generator=OFF \
    -DBUILD_opencv_python3=OFF \
    -DCMAKE_SYSTEM_PROCESSOR=aarch64 \
    -DCMAKE_C_COMPILER=clang \
    -DCMAKE_CXX_COMPILER=clang++ \
    ..

RUN ninja install

# Install Rust + Zig + cargo-zigbuild
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"

RUN ~/.cargo/bin/rustup target add aarch64-unknown-linux-gnu

RUN curl -L https://ziglang.org/download/0.11.0/zig-linux-x86_64-0.11.0.tar.xz | tar -xJ -C /opt && \
    ln -s /opt/zig-linux-x86_64-0.11.0/zig /usr/local/bin/zig

RUN cargo install cargo-zigbuild

# Set pkg-config paths for ARM64 OpenCV
ENV PKG_CONFIG_PATH="/opt/opencv-aarch64/lib/pkgconfig"
ENV LD_LIBRARY_PATH="/opt/opencv-aarch64/lib"
