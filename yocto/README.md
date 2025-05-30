# Yocto Build Setup

This directory contains a minimal Yocto build configuration for Rust-Spray.
It provides a small example layer and image that includes a basic graphical
interface based on the Poky `core-image-sato` image.

Steps to build locally:

```bash
cd yocto
git clone --depth 1 https://git.yoctoproject.org/git/poky poky   # fetch Poky if not using submodules
source poky/oe-init-build-env build
bitbake rust-spray-image
```

The resulting image will be in `build/tmp/deploy/images/` and can be
written to an SD card or run under QEMU.
