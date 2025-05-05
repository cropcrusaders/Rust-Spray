FROM ubuntu:22.04

# Prevent tzdata from prompting during install
ENV DEBIAN_FRONTEND=noninteractive

# Install Zig, Rust, OpenCV, and build tools
RUN apt-get update && apt-get install -y \
    curl git build-essential cmake pkg-config \
    libopencv-dev clang libclang-dev unzip tzdata

# Set timezone to UTC to avoid interactive tzdata config
RUN ln -fs /usr/share/zoneinfo/UTC /etc/localtime && dpkg-reconfigure -f noninteractive tzdata

# Install Rust
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Zig
RUN curl -L https://ziglang.org/download/0.11.0/zig-linux-x86_64-0.11.0.tar.xz | tar -xJ -C /opt && \
    ln -s /opt/zig-linux-x86_64-0.11.0/zig /usr/local/bin/zig

# Install cargo-zigbuild
RUN cargo install cargo-zigbuild
