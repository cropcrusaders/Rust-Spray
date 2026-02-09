/// Interactive command shell over the UART.
///
/// Provides a simple line-editing shell with built-in commands for
/// system inspection and memory access.
use crate::{mmio, print, println, timer, uart};

/// Maximum input line length.
const LINE_BUF_SIZE: usize = 256;

/// Run the interactive shell loop. This function never returns.
pub fn run() -> ! {
    println!("Type 'help' for available commands.\n");

    let mut buf = [0u8; LINE_BUF_SIZE];

    loop {
        print!("rpi-os> ");
        let len = read_line(&mut buf);
        let line = core::str::from_utf8(&buf[..len]).unwrap_or("");
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        dispatch(line);
    }
}

/// Read a line from the UART with basic editing (backspace).
/// Returns the number of bytes in the buffer.
fn read_line(buf: &mut [u8]) -> usize {
    let mut pos = 0;
    loop {
        let c = uart::getc();
        match c {
            // Enter
            b'\r' | b'\n' => {
                uart::putc(b'\r');
                uart::putc(b'\n');
                return pos;
            }
            // Backspace / DEL
            0x7F | 0x08 => {
                if pos > 0 {
                    pos -= 1;
                    // Erase character on terminal
                    uart::puts("\x08 \x08");
                }
            }
            // Ctrl-C: cancel line
            0x03 => {
                uart::puts("^C\n");
                return 0;
            }
            // Ctrl-U: clear line
            0x15 => {
                while pos > 0 {
                    uart::puts("\x08 \x08");
                    pos -= 1;
                }
            }
            // Printable ASCII
            0x20..=0x7E => {
                if pos < buf.len() {
                    buf[pos] = c;
                    pos += 1;
                    uart::putc(c);
                }
            }
            _ => {} // Ignore control characters and non-ASCII
        }
    }
}

/// Dispatch a command line to the appropriate handler.
fn dispatch(line: &str) {
    let mut parts = line.splitn(3, ' ');
    let cmd = parts.next().unwrap_or("");
    let arg1 = parts.next().unwrap_or("");
    let arg2 = parts.next().unwrap_or("");

    match cmd {
        "help" | "?" => cmd_help(),
        "info" => cmd_info(),
        "uptime" => cmd_uptime(),
        "peek" => cmd_peek(arg1),
        "poke" => cmd_poke(arg1, arg2),
        "hexdump" | "hd" => cmd_hexdump(arg1, arg2),
        "clear" | "cls" => cmd_clear(),
        "reboot" => cmd_reboot(),
        _ => println!(
            "Unknown command: '{}'. Type 'help' for available commands.",
            cmd
        ),
    }
}

fn cmd_help() {
    println!("Available commands:");
    println!("  help              Show this help message");
    println!("  info              Display system information");
    println!("  uptime            Show time since boot");
    println!("  peek <addr>       Read 32-bit value at hex address");
    println!("  poke <addr> <val> Write 32-bit hex value to hex address");
    println!("  hexdump <addr> [len]  Dump memory (default 64 bytes)");
    println!("  clear             Clear the terminal screen");
    println!("  reboot            Reboot the system");
}

fn cmd_info() {
    // Read current exception level
    let el: u64;
    unsafe { core::arch::asm!("mrs {}, CurrentEL", out(reg) el) };
    let el = (el >> 2) & 3;

    // Read main ID register
    let midr: u64;
    unsafe { core::arch::asm!("mrs {}, midr_el1", out(reg) midr) };
    let implementer = (midr >> 24) & 0xFF;
    let variant = (midr >> 20) & 0xF;
    let part = (midr >> 4) & 0xFFF;
    let revision = midr & 0xF;

    // Timer frequency
    let freq = timer::frequency();

    println!("RPI-OS v0.1.0 — Raspberry Pi 5 bare-metal kernel");
    println!();
    println!("CPU:");
    println!("  MIDR_EL1    = {:#018x}", midr);
    println!(
        "  Implementer = {:#04x} ({})",
        implementer,
        match implementer {
            0x41 => "ARM",
            0x42 => "Broadcom",
            0x43 => "Cavium",
            0x51 => "Qualcomm",
            _ => "Unknown",
        }
    );
    println!(
        "  Part        = {:#05x} ({})",
        part,
        match part {
            0xD05 => "Cortex-A55",
            0xD08 => "Cortex-A72",
            0xD09 => "Cortex-A73",
            0xD0B => "Cortex-A76",
            0xD0D => "Cortex-A77",
            _ => "Unknown",
        }
    );
    println!("  Variant     = {}, Revision = {}", variant, revision);
    println!("  Current EL  = EL{}", el);
    println!();
    println!("Timer:");
    println!("  Frequency   = {} Hz ({} MHz)", freq, freq / 1_000_000);
    println!("  Counter     = {}", timer::counter());
    println!();
    println!("SoC: BCM2712 (Raspberry Pi 5)");
    println!("  Periph base = 0x107c000000");
}

fn cmd_uptime() {
    let us = timer::uptime_us();
    let secs = us / 1_000_000;
    let ms = (us % 1_000_000) / 1_000;
    println!("Uptime: {}.{:03} seconds", secs, ms);
}

fn cmd_peek(addr_str: &str) {
    match parse_hex(addr_str) {
        Some(addr) => {
            let val = mmio::read(addr as usize);
            println!("[{:#010x}] = {:#010x}", addr, val);
        }
        None => println!("Usage: peek <hex_address>  (e.g. peek 0x107c215054)"),
    }
}

fn cmd_poke(addr_str: &str, val_str: &str) {
    let addr = match parse_hex(addr_str) {
        Some(a) => a,
        None => {
            println!("Usage: poke <hex_address> <hex_value>");
            return;
        }
    };
    let val = match parse_hex(val_str) {
        Some(v) => v,
        None => {
            println!("Usage: poke <hex_address> <hex_value>");
            return;
        }
    };
    mmio::write(addr as usize, val as u32);
    println!("[{:#010x}] <- {:#010x}", addr, val as u32);
}

fn cmd_hexdump(addr_str: &str, len_str: &str) {
    let addr = match parse_hex(addr_str) {
        Some(a) => a as usize,
        None => {
            println!("Usage: hexdump <hex_address> [length]");
            return;
        }
    };
    let len = if len_str.is_empty() {
        64
    } else {
        match parse_hex(len_str) {
            Some(l) => l as usize,
            None => 64,
        }
    };
    // Cap at 256 bytes
    let len = if len > 256 { 256 } else { len };
    let start = addr & !0xF; // Align to 16-byte boundary

    for offset in (0..len).step_by(16) {
        let line_addr = start + offset;
        print!("{:08x}: ", line_addr);

        // Hex bytes
        for i in 0..16 {
            if offset + i < len {
                let byte = unsafe { core::ptr::read_volatile((line_addr + i) as *const u8) };
                print!("{:02x} ", byte);
            } else {
                print!("   ");
            }
            if i == 7 {
                print!(" ");
            }
        }

        // ASCII
        print!(" |");
        for i in 0..16 {
            if offset + i < len {
                let byte = unsafe { core::ptr::read_volatile((line_addr + i) as *const u8) };
                if (0x20..=0x7E).contains(&byte) {
                    print!("{}", byte as char);
                } else {
                    print!(".");
                }
            }
        }
        println!("|");
    }
}

fn cmd_clear() {
    // ANSI escape: clear screen and move cursor to top-left
    uart::puts("\x1B[2J\x1B[H");
}

fn cmd_reboot() {
    println!("Rebooting...");
    timer::delay_ms(100);

    // BCM2712 PM watchdog reboot (same mechanism as BCM2711)
    const PERIPHERAL_BASE: usize = 0x107c000000;
    const PM_WDOG: usize = PERIPHERAL_BASE + 0x100024;
    const PM_RSTC: usize = PERIPHERAL_BASE + 0x10001c;
    const PM_PASSWORD: u32 = 0x5a000000;

    // Set watchdog to trigger in ~10 ticks
    mmio::write(PM_WDOG, PM_PASSWORD | 10);
    // Full reset
    mmio::write(PM_RSTC, PM_PASSWORD | 0x20);

    loop {
        core::hint::spin_loop();
    }
}

/// Parse a hexadecimal string (with optional `0x` prefix) into a `u64`.
fn parse_hex(s: &str) -> Option<u64> {
    if s.is_empty() {
        return None;
    }
    let s = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    if s.is_empty() {
        return None;
    }
    let mut result: u64 = 0;
    for &b in s.as_bytes() {
        let digit = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => return None,
        };
        result = result.checked_shl(4)?;
        result = result.checked_add(digit as u64)?;
    }
    Some(result)
}
