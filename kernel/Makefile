TARGET     := aarch64-unknown-none
KERNEL_ELF := target/$(TARGET)/release/rpi-os
KERNEL_IMG := kernel8.img

# Use rust-objcopy from cargo-binutils, fall back to llvm-objcopy
OBJCOPY := $(shell command -v rust-objcopy 2>/dev/null || echo llvm-objcopy)

.PHONY: all build clean fmt check

all: $(KERNEL_IMG)

build:
	cargo build --release

$(KERNEL_IMG): build
	$(OBJCOPY) --strip-all -O binary $(KERNEL_ELF) $(KERNEL_IMG)
	@echo "Built $(KERNEL_IMG) ($$(stat -c %s $(KERNEL_IMG) 2>/dev/null || stat -f %z $(KERNEL_IMG)) bytes)"

clean:
	cargo clean
	rm -f $(KERNEL_IMG)

fmt:
	cargo fmt

check:
	cargo fmt -- --check
	cargo clippy --release
