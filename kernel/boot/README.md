# SD Card Setup for RPI-OS

## Prerequisites

- Raspberry Pi 5
- microSD card (any size, FAT32 formatted)
- USB-to-UART adapter (connects to GPIO 14/15 on the Pi header)
- Terminal program (e.g., `minicom`, `picocom`, `screen`)

## Preparing the SD Card

### 1. Get Raspberry Pi firmware files

Download the latest boot firmware from the Raspberry Pi firmware repository.
You need these files on the FAT32 boot partition:

```
bootcode.bin        (not needed on Pi 5 — firmware is in EEPROM)
start4.elf          (GPU firmware for Pi 4/5)
fixup4.dat          (GPU firmware fixup)
bcm2712-rpi-5-b.dtb (Device tree for Pi 5)
```

The easiest way is to flash Raspberry Pi OS to the SD card, then replace
`kernel8.img` with the one built from this project. The firmware files
are already present on a standard Raspberry Pi OS image.

### 2. Build the kernel

```bash
cd kernel/
make
```

This produces `kernel8.img` in the kernel directory.

### 3. Copy files to the SD card

Mount the boot partition and copy:

```bash
cp kernel8.img /path/to/sdcard/boot/
cp boot/config.txt /path/to/sdcard/boot/
```

### 4. Connect the UART

Connect a USB-to-UART adapter to the Raspberry Pi 5 header:

| UART Adapter | RPi 5 Header |
|-------------|-------------|
| TX          | GPIO 15 (RXD) — Pin 10 |
| RX          | GPIO 14 (TXD) — Pin 8  |
| GND         | GND — Pin 6            |

**Do NOT connect VCC/3.3V from the adapter to the Pi.**

### 5. Open a terminal

```bash
# Linux
picocom -b 115200 /dev/ttyUSB0

# macOS
screen /dev/tty.usbserial-* 115200

# Windows (use PuTTY or similar, 115200 8N1)
```

### 6. Boot

Insert the SD card into the Pi 5 and apply power. You should see the
RPI-OS banner and an interactive shell prompt on the serial console.

## Troubleshooting

- **No output**: Check UART wiring (TX/RX crossed correctly). Verify
  `enable_uart=1` is in `config.txt`. Try a different baud rate.
- **Garbled output**: Baud rate mismatch. Ensure both sides use 115200.
- **Pi doesn't boot**: Make sure the firmware files (`start4.elf`, etc.)
  are present on the SD card. The green LED should blink during boot.
