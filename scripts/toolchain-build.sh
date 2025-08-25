#!/usr/bin/env bash
set -euo pipefail

rustup target add aarch64-unknown-linux-gnu aarch64-unknown-linux-musl armv7-unknown-linux-gnueabihf
rustup component add llvm-tools-preview
cargo install cargo-binutils -q || true

if ! command -v zig >/dev/null; then
  echo "zig not found in PATH" >&2
  exit 1
fi

for target in aarch64-unknown-linux-gnu aarch64-unknown-linux-musl armv7-unknown-linux-gnueabihf; do
  cargo build --release --target "$target"
done
