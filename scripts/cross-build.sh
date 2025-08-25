#!/usr/bin/env bash
set -euo pipefail

cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target armv7-unknown-linux-gnueabihf
