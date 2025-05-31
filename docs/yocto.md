# Building with Yocto

The project can also be built within the Yocto Project. The following steps outline
one possible approach. Replace paths and layers to suit your Yocto setup.

1. **Clone Poky and required layers**
   ```bash
   git clone https://git.yoctoproject.org/poky.git -b kirkstone
   cd poky
   git clone https://github.com/meta-rust/meta-rust.git
   git clone https://github.com/hamcrest/meta-opencv.git
   ```
   Add any Raspberry Pi BSP layer such as `meta-raspberrypi` if targeting the Pi.

2. **Create a custom layer**
   ```bash
   source oe-init-build-env
   bitbake-layers create-layer ../meta-rustspray
   bitbake-layers add-layer ../meta-rustspray
   ```
   Inside `meta-rustspray/recipes-rust/rustspray`, create a recipe `rustspray_%.bb`:
   ```bitbake
   SUMMARY = "Rust-Spray weed detection controller"
   LICENSE = "MIT"
   LIC_FILES_CHKSUM = "file://LICENSE;md5=<md5sum>"

   inherit cargo

   SRC_URI = "git://path/to/Rust-Spray.git;protocol=https"
   S = "${WORKDIR}/git"
   ```
   Ensure the recipe depends on `opencv` and `rust` from `meta-rust`.

3. **Configure the build**
   Edit `conf/local.conf` and set the desired machine, for example:
   ```
   MACHINE = "raspberrypi4"
   ```
   Add the required layers in `bblayers.conf`:
   ```
   BBLAYERS += "\
     ${TOPDIR}/../meta-rust \ 
     ${TOPDIR}/../meta-opencv \ 
     ${TOPDIR}/../meta-rustspray"
   ```

4. **Build the image or package**
   To build only the Rust-Spray binary run:
   ```bash
   bitbake rustspray
   ```
   Or add `rustspray` to your image recipe and build the full image with `bitbake core-image-minimal` (or another image).

