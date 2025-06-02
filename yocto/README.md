# Yocto Build Setup

This directory contains a minimal Yocto build configuration for Rust-Spray.
It provides a small example layer and image that includes a basic graphical
interface based on the Poky `core-image-sato` image.

Steps to build locally:

```bash
cd yocto
git clone --depth 1 https://git.yoctoproject.org/git/poky poky   # fetch Poky if not using submodules
git clone --depth 1 https://github.com/openembedded/meta-openembedded.git meta-openembedded
source poky/oe-init-build-env build
# Edit `conf/local.conf` and replace the deprecated
# `EXTRA_IMAGE_FEATURES ?= "debug-tweaks"` line with:
# `EXTRA_IMAGE_FEATURES ?= "allow-root-login allow-empty-password"`
# If the build fails with a Meson "clock skew" error, clean
# the pixman recipe first:
bitbake -c clean pixman
# If glib fails during configuration, clean its native recipe:
bitbake -c clean glib-2.0-native
# If it still fails during the initial build, clean that recipe as well:
bitbake -c clean glib-2.0-initial
bitbake rust-spray-image
```

The resulting image will be in `build/tmp/deploy/images/` and can be
written to an SD card or run under QEMU.
