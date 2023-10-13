#![no_std]
#![no_main]

use core::panic::PanicInfo;
use user::ipc::Listener;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
fn main(_listener: Listener) {
}
