/// Console output via the mini UART.
///
/// Provides `print!` and `println!` macros that work in a `no_std`
/// bare-metal environment by writing to the UART.
use crate::uart;
use core::fmt::{self, Write};

/// Zero-sized writer that forwards to the UART.
struct UartWriter;

impl Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        uart::puts(s);
        Ok(())
    }
}

/// Internal function used by the `print!` macro.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    UartWriter.write_fmt(args).unwrap();
}

/// Print formatted text to the UART (no newline).
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::console::_print(format_args!($($arg)*))
    }};
}

/// Print formatted text to the UART followed by a newline.
#[macro_export]
macro_rules! println {
    () => {{ $crate::print!("\n") }};
    ($($arg:tt)*) => {{
        $crate::console::_print(format_args!($($arg)*));
        $crate::print!("\n")
    }};
}
