#![no_std]
#![no_main]

use core::panic::PanicInfo;
use user::ipc::router::StackRouter;
use user::ipc::Listener;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
fn main(_listener: Listener) {
    let _router = StackRouter::new()
        .route("global_read", |_| {})
        .route("global_write", |_| {});
}
