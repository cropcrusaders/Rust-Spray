/// Panic handler for the bare-metal kernel.
///
/// Prints the panic message to the UART and halts the CPU.
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::println!();
    crate::println!("!!! KERNEL PANIC !!!");
    crate::println!("{}", info);
    crate::println!();
    crate::println!("System halted.");

    loop {
        core::hint::spin_loop();
    }
}
