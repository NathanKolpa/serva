#![no_std]
#![no_main]

mod writer;

use crate::writer::Writer;
use core::panic::PanicInfo;
use user::ipc::router::{RouterMut, StackRouter};
use user::ipc::{Listener, Request};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn write_to_tty(_req: Request, _writer: &mut Writer) {

}

#[no_mangle]
fn main(mut listener: Listener) {
    let mut writer = Writer {};

    let mut router = StackRouter::new().route("write", |r| write_to_tty(r, &mut writer));

    while let Some((req, endpoint)) = listener.accept() {
        (router.forward_to_mut(&endpoint))(req)
    }
}
