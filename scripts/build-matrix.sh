#!/usr/bin/env bash
set -euo pipefail

measure() {
  local name=$1
  shift
  local start=$(date +%s)
  "$@"
  local end=$(date +%s)
  echo $((end-start))
}

cross_time=$(measure cross cross build --release --target aarch64-unknown-linux-gnu)

tool_time=$(measure cargo cargo build --release --target aarch64-unknown-linux-gnu)

echo "cross aarch64 build: ${cross_time}s"
echo "toolchain aarch64 build: ${tool_time}s"

if [ "$tool_time" -lt $((cross_time*80/100)) ]; then
  echo "USE_TOOLCHAIN=true" > build-lane.env
else
  echo "USE_TOOLCHAIN=false" > build-lane.env
fi
