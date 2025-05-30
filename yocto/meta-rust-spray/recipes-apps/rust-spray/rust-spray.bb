DESCRIPTION = "Rust-Spray weed detection application"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://../../../../LICENSE;md5=9d852dbaca7af3fd17c0249a3f04e40d"

SRC_URI = "git://github.com/cropcrusaders/Rust-Spray.git;branch=main"
SRCREV = "${AUTOREV}"

S = "${WORKDIR}/git"
inherit cargo

RDEPENDS:${PN} += "opencv"

do_install:append() {
    install -d ${D}${bindir}
    install -m 0755 target/release/rustspray ${D}${bindir}/rustspray
}
