#![macro_use]

use crate::arch::x86_64::interrupts::atomic_block;
use core::fmt::Write;

use crate::devices::uart_16550::SERIAL;

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::debug::_serial_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debug_println {
    () => ($crate::debug_print!("\n"));
    ($($arg:tt)*) => ($crate::debug_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _serial_print(args: core::fmt::Arguments) {
    atomic_block(|| {
        SERIAL.lock().write_fmt(args).unwrap();
    });
}

pub const DEBUG_CHANNEL: &str = "16550 UART (Serial)";
