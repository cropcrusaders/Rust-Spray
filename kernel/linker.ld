/*
 * Linker script for Raspberry Pi 5 bare-metal kernel.
 *
 * The RPi firmware loads kernel8.img at physical address 0x80000
 * and jumps to it. The .text.boot section must be placed first
 * at that address to contain the entry point (_start).
 */

ENTRY(_start)

SECTIONS
{
    . = 0x80000;

    /* Boot code must come first — contains _start entry point */
    .text.boot : {
        KEEP(*(.text.boot))
    }

    .text : {
        *(.text .text.*)
    }

    /* Read-only data */
    .rodata : ALIGN(4096) {
        *(.rodata .rodata.*)
    }

    /* Initialized data */
    .data : ALIGN(4096) {
        *(.data .data.*)
    }

    /* Uninitialized data — cleared to zero by boot code */
    .bss : ALIGN(4096) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /* Kernel stack: 512 KB, grows downward from __stack_top */
    . = ALIGN(16);
    __stack_bottom = .;
    . += 0x80000;
    __stack_top = .;

    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame*)
    }
}
