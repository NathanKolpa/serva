#![no_std]
#![no_main]

mod writer;

use crate::writer::Writer;
use core::panic::PanicInfo;
use user::ipc::{Endpoint, Listener, Request};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn write_to_tty(_req: Request, _writer: &mut Writer) {}

#[no_mangle]
fn main(mut listener: Listener) {
    let mut writer = Writer {};

    let write_endpoint = Endpoint::try_lookup("write").unwrap();

    while let Some((req, endpoint)) = listener.accept() {
        if endpoint == write_endpoint {
            write_to_tty(req, &mut writer);
        }
    }
}
