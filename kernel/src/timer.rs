/// ARM Generic Timer access for AArch64.
///
/// The Cortex-A76 cores on the RPi 5 include the ARM generic timer,
/// which provides a high-resolution monotonic counter independent of
/// the SoC peripheral clocks. The counter typically runs at 54 MHz
/// on Raspberry Pi hardware.
use core::arch::asm;

/// Read the counter frequency (CNTFRQ_EL0) in Hz.
pub fn frequency() -> u64 {
    let freq: u64;
    unsafe { asm!("mrs {}, cntfrq_el0", out(reg) freq) };
    freq
}

/// Read the current counter value (CNTPCT_EL0).
pub fn counter() -> u64 {
    let cnt: u64;
    unsafe { asm!("mrs {}, cntpct_el0", out(reg) cnt) };
    cnt
}

/// Return the elapsed time since boot in microseconds (approximate).
pub fn uptime_us() -> u64 {
    let freq = frequency();
    if freq == 0 {
        return 0;
    }
    // Compute counter * 1_000_000 / freq avoiding overflow by dividing first
    let cnt = counter();
    let secs = cnt / freq;
    let remainder = cnt % freq;
    secs * 1_000_000 + remainder * 1_000_000 / freq
}

/// Busy-wait for approximately `us` microseconds.
pub fn delay_us(us: u64) {
    let freq = frequency();
    if freq == 0 {
        return;
    }
    let target = counter() + (freq * us / 1_000_000);
    while counter() < target {
        core::hint::spin_loop();
    }
}

/// Busy-wait for approximately `ms` milliseconds.
pub fn delay_ms(ms: u64) {
    delay_us(ms * 1000);
}
