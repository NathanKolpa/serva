#![no_std]

extern crate alloc;

#[cfg(not(test))]
use crate::ipc::Listener;
use core::alloc::{GlobalAlloc, Layout};

pub mod io;
pub mod ipc;

struct NullAlloc;

unsafe impl GlobalAlloc for NullAlloc {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        todo!()
    }
}

#[global_allocator]
static GLOBAL_ALLOC: NullAlloc = NullAlloc;

#[cfg(not(test))]
extern "C" {
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
