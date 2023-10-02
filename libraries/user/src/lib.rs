#![no_std]

extern crate alloc;

use crate::ipc::Listener;

pub mod io;
pub mod ipc;

#[cfg(not(test))]
extern {
    fn main(listener: Listener);
}

#[no_mangle]
extern "C" fn _start() -> ! {
    #[cfg(not(test))]
    unsafe {
        main(Listener::new());
    }

    syscall::thread_exit()
}
