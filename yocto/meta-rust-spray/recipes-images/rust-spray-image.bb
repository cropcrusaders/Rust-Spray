DESCRIPTION = "Rust-Spray graphical demo image"
LICENSE = "MIT"
inherit core-image

IMAGE_INSTALL += "rust-spray"

# Use the sato GUI for a lightweight interface
require recipes-sato/images/core-image-sato.bb
