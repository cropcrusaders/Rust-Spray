/// AArch64 boot code and exception vector table for Raspberry Pi 5.
///
/// The RPi firmware loads kernel8.img at 0x80000 and starts all four
/// Cortex-A76 cores. Core 0 runs the boot sequence; cores 1-3 are
/// parked in a WFE loop. The firmware starts the CPU at EL2; we
/// transition to EL1 before entering Rust code.
use core::arch::global_asm;

// ---------------------------------------------------------------------------
// Boot entry point — placed in .text.boot so the linker script puts it first
// ---------------------------------------------------------------------------
global_asm!(
    r#"
.section .text.boot, "ax"
.global _start

_start:
    // ---------------------------------------------------------------
    // 1. Park secondary cores (cores 1-3 spin in WFE loop)
    // ---------------------------------------------------------------
    mrs     x0, mpidr_el1
    and     x0, x0, #0xFF          // Core ID in Aff0
    cbnz    x0, .L_park            // Non-zero => not core 0

    // ---------------------------------------------------------------
    // 2. Transition from EL2 to EL1 (if we are at EL2)
    // ---------------------------------------------------------------
    mrs     x0, CurrentEL
    and     x0, x0, #0x0C          // Extract EL field (bits [3:2])
    cmp     x0, #0x08              // EL2 = 0b1000
    b.ne    .L_at_el1              // Already at EL1 (or EL3), skip

    // -- Configure EL2 for EL1 entry --
    // HCR_EL2: set RW bit so EL1 executes in AArch64
    mov     x0, #(1 << 31)
    msr     hcr_el2, x0

    // Disable EL2 coprocessor traps
    msr     cptr_el2, xzr

    // SPSR_EL2: return to EL1h (SPSel=1), all DAIF masked
    mov     x0, #0x3C5
    msr     spsr_el2, x0

    // ELR_EL2: return address is .L_at_el1
    adr     x0, .L_at_el1
    msr     elr_el2, x0

    isb
    eret

.L_at_el1:
    // ---------------------------------------------------------------
    // 3. Configure EL1
    // ---------------------------------------------------------------
    // Disable MMU and caches (start clean)
    mrs     x0, sctlr_el1
    bic     x0, x0, #(1 << 0)     // M  = 0 (MMU off)
    bic     x0, x0, #(1 << 2)     // C  = 0 (data cache off)
    bic     x0, x0, #(1 << 12)    // I  = 0 (instruction cache off)
    msr     sctlr_el1, x0
    isb

    // Set up VBAR_EL1 (exception vector base)
    ldr     x0, =exception_vectors
    msr     vbar_el1, x0
    isb

    // ---------------------------------------------------------------
    // 4. Clear BSS
    // ---------------------------------------------------------------
    ldr     x0, =__bss_start
    ldr     x1, =__bss_end
.L_bss_loop:
    cmp     x0, x1
    b.ge    .L_bss_done
    str     xzr, [x0], #8
    b       .L_bss_loop
.L_bss_done:

    // ---------------------------------------------------------------
    // 5. Set up stack and jump to Rust
    // ---------------------------------------------------------------
    ldr     x0, =__stack_top
    mov     sp, x0

    bl      kernel_main

    // kernel_main should never return, but if it does, park.
.L_park:
    wfe
    b       .L_park
"#
);

// ---------------------------------------------------------------------------
// Exception vector table
//
// AArch64 requires a 2048-byte aligned table with 16 entries of 128 bytes
// each. Four exception types x four execution contexts.
// ---------------------------------------------------------------------------
global_asm!(
    r#"
.balign 0x800
.global exception_vectors
exception_vectors:

    // ----- Current EL, SP_EL0 (unused in kernel mode) -----
    .balign 0x80
    b       exc_sync_sp0
    .balign 0x80
    b       exc_irq_sp0
    .balign 0x80
    b       exc_fiq_sp0
    .balign 0x80
    b       exc_serror_sp0

    // ----- Current EL, SP_ELx (kernel exceptions) -----
    .balign 0x80
    b       exc_sync_spx
    .balign 0x80
    b       exc_irq_spx
    .balign 0x80
    b       exc_fiq_spx
    .balign 0x80
    b       exc_serror_spx

    // ----- Lower EL, AArch64 -----
    .balign 0x80
    b       exc_sync_lower64
    .balign 0x80
    b       exc_irq_lower64
    .balign 0x80
    b       exc_fiq_lower64
    .balign 0x80
    b       exc_serror_lower64

    // ----- Lower EL, AArch32 -----
    .balign 0x80
    b       exc_sync_lower32
    .balign 0x80
    b       exc_irq_lower32
    .balign 0x80
    b       exc_fiq_lower32
    .balign 0x80
    b       exc_serror_lower32

// ---------------------------------------------------------------------------
// Exception handler stubs — save caller-saved regs, call Rust, then eret/hang
// ---------------------------------------------------------------------------

.macro EXCEPTION_STUB name, handler
\name:
    // Save x0-x7 and x29-x30 (enough for Rust handler to run)
    stp     x0,  x1,  [sp, #-16]!
    stp     x2,  x3,  [sp, #-16]!
    stp     x4,  x5,  [sp, #-16]!
    stp     x6,  x7,  [sp, #-16]!
    stp     x29, x30, [sp, #-16]!

    // Read exception context into arguments
    mrs     x0, esr_el1            // x0 = syndrome
    mrs     x1, elr_el1            // x1 = return address
    mrs     x2, far_el1            // x2 = fault address

    bl      \handler

    // Restore and hang (we don't resume from unexpected exceptions)
    ldp     x29, x30, [sp], #16
    ldp     x6,  x7,  [sp], #16
    ldp     x4,  x5,  [sp], #16
    ldp     x2,  x3,  [sp], #16
    ldp     x0,  x1,  [sp], #16
1:
    wfe
    b       1b
.endm

EXCEPTION_STUB exc_sync_sp0,      handle_sync_sp0
EXCEPTION_STUB exc_irq_sp0,       handle_irq_sp0
EXCEPTION_STUB exc_fiq_sp0,       handle_fiq_sp0
EXCEPTION_STUB exc_serror_sp0,    handle_serror_sp0

EXCEPTION_STUB exc_sync_spx,      handle_sync_spx
EXCEPTION_STUB exc_irq_spx,       handle_irq_spx
EXCEPTION_STUB exc_fiq_spx,       handle_fiq_spx
EXCEPTION_STUB exc_serror_spx,    handle_serror_spx

EXCEPTION_STUB exc_sync_lower64,  handle_sync_lower64
EXCEPTION_STUB exc_irq_lower64,   handle_irq_lower64
EXCEPTION_STUB exc_fiq_lower64,   handle_fiq_lower64
EXCEPTION_STUB exc_serror_lower64,handle_serror_lower64

EXCEPTION_STUB exc_sync_lower32,  handle_sync_lower32
EXCEPTION_STUB exc_irq_lower32,   handle_irq_lower32
EXCEPTION_STUB exc_fiq_lower32,   handle_fiq_lower32
EXCEPTION_STUB exc_serror_lower32,handle_serror_lower32
"#
);
