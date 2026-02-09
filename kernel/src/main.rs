//! # RPI-OS: Bare-metal operating system for Raspberry Pi 5
//!
//! A minimal kernel targeting the BCM2712 SoC (Quad Cortex-A76, ARMv8.2-A).
//! Boots from the SD card as `kernel8.img`, provides a serial console shell
//! over the mini UART at 115200 baud.
//!
//! ## Boot Sequence
//!
//! 1. RPi firmware loads `kernel8.img` at physical address `0x80000`
//! 2. All four cores start executing `_start` in `boot.rs`
//! 3. Cores 1-3 are parked (WFE loop)
//! 4. Core 0 transitions from EL2 to EL1
//! 5. BSS is cleared, stack is set up
//! 6. `kernel_main` is called

#![no_std]
#![no_main]

mod boot;
mod console;
mod exceptions;
mod mmio;
mod panic;
mod shell;
mod timer;
mod uart;

/// Kernel entry point, called from the boot assembly after hardware init.
///
/// At this point we are running on core 0 at EL1 with:
/// - MMU and caches disabled
/// - BSS zeroed
/// - Stack set up
/// - Exception vectors installed
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // Initialize the mini UART for serial I/O
    uart::init();

    // Print boot banner
    println!();
    println!("==================================================");
    println!("  RPI-OS v0.1.0");
    println!("  Bare-metal kernel for Raspberry Pi 5");
    println!("  BCM2712 / Cortex-A76 / AArch64");
    println!("==================================================");
    println!();

    // Display current exception level
    let el: u64;
    unsafe { core::arch::asm!("mrs {}, CurrentEL", out(reg) el) };
    let el = (el >> 2) & 3;
    println!("[boot] Running at EL{}", el);

    // Display timer info
    let freq = timer::frequency();
    println!("[boot] Timer frequency: {} Hz", freq);
    println!("[boot] Boot complete.\n");

    // Enter interactive shell (never returns)
    shell::run();
}
