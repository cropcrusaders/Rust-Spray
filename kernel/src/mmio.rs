/// Volatile memory-mapped I/O helpers for bare-metal register access.
use core::ptr;

/// Read a 32-bit value from a memory-mapped register.
#[inline(always)]
pub fn read(addr: usize) -> u32 {
    unsafe { ptr::read_volatile(addr as *const u32) }
}

/// Write a 32-bit value to a memory-mapped register.
#[inline(always)]
pub fn write(addr: usize, val: u32) {
    unsafe { ptr::write_volatile(addr as *mut u32, val) }
}
