# syntax=docker/dockerfile:1
FROM rust:1-bookworm as builder
WORKDIR /usr/src/rustspray
# Copy manifest separately for caching
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config ./config
COPY xtask ./xtask
# Build release binary
RUN cargo build --release --locked --bin rustspray

FROM debian:bookworm-slim
# Install minimal runtime dependencies if any
RUN apt-get update && apt-get install -y --no-install-recommends \
    libgpiod2 \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rustspray/target/release/rustspray /usr/local/bin/rustspray
ENTRYPOINT ["/usr/local/bin/rustspray"]
