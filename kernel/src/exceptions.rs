/// Exception handlers for AArch64 EL1.
///
/// These functions are called from the assembly stubs in `boot.rs`.
/// Each handler prints diagnostic information about the exception
/// and then the assembly stub halts the core.

/// Decode the Exception Class from ESR_EL1 bits [31:26].
fn exception_class_str(esr: u64) -> &'static str {
    let ec = (esr >> 26) & 0x3F;
    match ec {
        0x00 => "Unknown reason",
        0x01 => "Trapped WF*",
        0x0E => "Illegal Execution state",
        0x15 => "SVC instruction (AArch64)",
        0x18 => "MSR/MRS trap (AArch64)",
        0x20 => "Instruction Abort (lower EL)",
        0x21 => "Instruction Abort (same EL)",
        0x22 => "PC alignment fault",
        0x24 => "Data Abort (lower EL)",
        0x25 => "Data Abort (same EL)",
        0x26 => "SP alignment fault",
        0x2C => "Floating-point exception",
        0x30 => "Breakpoint (lower EL)",
        0x31 => "Breakpoint (same EL)",
        0x32 => "Software Step (lower EL)",
        0x33 => "Software Step (same EL)",
        0x34 => "Watchpoint (lower EL)",
        0x35 => "Watchpoint (same EL)",
        0x38 => "BKPT instruction (AArch32)",
        0x3C => "BRK instruction (AArch64)",
        _ => "Other",
    }
}

/// Print common exception context.
fn print_exception(kind: &str, esr: u64, elr: u64, far: u64) {
    crate::println!();
    crate::println!("*** EXCEPTION: {} ***", kind);
    crate::println!("  ESR_EL1  = {:#018x} ({})", esr, exception_class_str(esr));
    crate::println!("  ELR_EL1  = {:#018x}", elr);
    crate::println!("  FAR_EL1  = {:#018x}", far);
    crate::println!(
        "  EC       = {:#04x}, ISS = {:#010x}",
        (esr >> 26) & 0x3F,
        esr & 0x1FFFFFF
    );
    crate::println!("CPU halted.");
}

// -- Current EL, SP_EL0 (shouldn't happen in kernel) --

#[no_mangle]
extern "C" fn handle_sync_sp0(esr: u64, elr: u64, far: u64) {
    print_exception("Synchronous (SP_EL0)", esr, elr, far);
}

#[no_mangle]
extern "C" fn handle_irq_sp0(esr: u64, elr: u64, far: u64) {
    print_exception("IRQ (SP_EL0)", esr, elr, far);
}

#[no_mangle]
extern "C" fn handle_fiq_sp0(esr: u64, elr: u64, far: u64) {
    print_exception("FIQ (SP_EL0)", esr, elr, far);
}

#[no_mangle]
extern "C" fn handle_serror_sp0(esr: u64, elr: u64, far: u64) {
    print_exception("SError (SP_EL0)", esr, elr, far);
}

// -- Current EL, SP_ELx (kernel exceptions) --

#[no_mangle]
extern "C" fn handle_sync_spx(esr: u64, elr: u64, far: u64) {
    print_exception("Synchronous (SP_ELx)", esr, elr, far);
}

#[no_mangle]
extern "C" fn handle_irq_spx(esr: u64, elr: u64, far: u64) {
    print_exception("IRQ (SP_ELx)", esr, elr, far);
}

#[no_mangle]
extern "C" fn handle_fiq_spx(esr: u64, elr: u64, far: u64) {
    print_exception("FIQ (SP_ELx)", esr, elr, far);
}

#[no_mangle]
extern "C" fn handle_serror_spx(esr: u64, elr: u64, far: u64) {
    print_exception("SError (SP_ELx)", esr, elr, far);
}

// -- Lower EL, AArch64 --

#[no_mangle]
extern "C" fn handle_sync_lower64(esr: u64, elr: u64, far: u64) {
    print_exception("Synchronous (Lower EL, AArch64)", esr, elr, far);
}

#[no_mangle]
extern "C" fn handle_irq_lower64(esr: u64, elr: u64, far: u64) {
    print_exception("IRQ (Lower EL, AArch64)", esr, elr, far);
}

#[no_mangle]
extern "C" fn handle_fiq_lower64(esr: u64, elr: u64, far: u64) {
    print_exception("FIQ (Lower EL, AArch64)", esr, elr, far);
}

#[no_mangle]
extern "C" fn handle_serror_lower64(esr: u64, elr: u64, far: u64) {
    print_exception("SError (Lower EL, AArch64)", esr, elr, far);
}

// -- Lower EL, AArch32 --

#[no_mangle]
extern "C" fn handle_sync_lower32(esr: u64, elr: u64, far: u64) {
    print_exception("Synchronous (Lower EL, AArch32)", esr, elr, far);
}

#[no_mangle]
extern "C" fn handle_irq_lower32(esr: u64, elr: u64, far: u64) {
    print_exception("IRQ (Lower EL, AArch32)", esr, elr, far);
}

#[no_mangle]
extern "C" fn handle_fiq_lower32(esr: u64, elr: u64, far: u64) {
    print_exception("FIQ (Lower EL, AArch32)", esr, elr, far);
}

#[no_mangle]
extern "C" fn handle_serror_lower32(esr: u64, elr: u64, far: u64) {
    print_exception("SError (Lower EL, AArch32)", esr, elr, far);
}
