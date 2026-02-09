/// BCM2712 Mini UART (AUX UART) driver for Raspberry Pi 5.
///
/// The mini UART is part of the AUX peripheral block on the BCM2712 SoC.
/// When `enable_uart=1` is set in config.txt, the firmware initializes
/// the mini UART at 115200 baud before handing control to the kernel.
///
/// # Peripheral Addresses
///
/// BCM2712 ARM peripheral base: `0x107c000000`
/// AUX block offset: `0x215000`
///
/// The mini UART is simple: 8-bit data, no parity, 1 stop bit, no flow control.
/// Baud rate is derived from the system clock:
///   `baud_reg = system_clock / (8 * baud_rate) - 1`
use crate::mmio;

/// BCM2712 peripheral base address in the ARM physical address space.
const PERIPHERAL_BASE: usize = 0x107c000000;

/// AUX peripheral block base.
const AUX_BASE: usize = PERIPHERAL_BASE + 0x215000;

// AUX registers
const AUX_ENABLES: usize = AUX_BASE + 0x04;

// Mini UART registers (offsets from AUX_BASE)
const AUX_MU_IO: usize = AUX_BASE + 0x40;
const AUX_MU_IER: usize = AUX_BASE + 0x44;
const AUX_MU_IIR: usize = AUX_BASE + 0x48;
const AUX_MU_LCR: usize = AUX_BASE + 0x4C;
const AUX_MU_MCR: usize = AUX_BASE + 0x50;
const AUX_MU_LSR: usize = AUX_BASE + 0x54;
const AUX_MU_CNTL: usize = AUX_BASE + 0x60;
const AUX_MU_BAUD: usize = AUX_BASE + 0x68;

/// Initialize the mini UART.
///
/// This puts the UART into a known good state. If `enable_uart=1` is set
/// in config.txt (recommended), the firmware has already configured GPIO
/// pins and baud rate, so this just ensures the UART is ready.
pub fn init() {
    // Enable the mini UART (bit 0 of AUX_ENABLES)
    mmio::write(AUX_ENABLES, mmio::read(AUX_ENABLES) | 1);

    // Disable TX/RX while configuring
    mmio::write(AUX_MU_CNTL, 0);

    // Disable interrupts
    mmio::write(AUX_MU_IER, 0);

    // 8-bit mode
    mmio::write(AUX_MU_LCR, 3);

    // RTS line high
    mmio::write(AUX_MU_MCR, 0);

    // Clear TX and RX FIFOs
    mmio::write(AUX_MU_IIR, 0xC6);

    // Baud rate: 115200 at 500 MHz system clock => register = 541
    // Note: RPi 5 VPU clock may differ; firmware setup via config.txt
    // is more reliable. This value works if core_freq=500.
    mmio::write(AUX_MU_BAUD, 541);

    // Enable TX and RX
    mmio::write(AUX_MU_CNTL, 3);
}

/// Send a single byte. Blocks until the transmit FIFO has space.
pub fn putc(c: u8) {
    // Wait for TX FIFO to have space (bit 5 of LSR = transmitter empty)
    while mmio::read(AUX_MU_LSR) & (1 << 5) == 0 {
        core::hint::spin_loop();
    }
    mmio::write(AUX_MU_IO, c as u32);
}

/// Receive a single byte. Blocks until data is available.
pub fn getc() -> u8 {
    // Wait for RX FIFO to have data (bit 0 of LSR = data ready)
    while mmio::read(AUX_MU_LSR) & 1 == 0 {
        core::hint::spin_loop();
    }
    (mmio::read(AUX_MU_IO) & 0xFF) as u8
}

/// Try to receive a byte without blocking. Returns `None` if no data available.
pub fn try_getc() -> Option<u8> {
    if mmio::read(AUX_MU_LSR) & 1 != 0 {
        Some((mmio::read(AUX_MU_IO) & 0xFF) as u8)
    } else {
        None
    }
}

/// Send a string, converting `\n` to `\r\n` for terminal compatibility.
pub fn puts(s: &str) {
    for b in s.bytes() {
        if b == b'\n' {
            putc(b'\r');
        }
        putc(b);
    }
}
