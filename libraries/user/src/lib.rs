#![no_std]

#[cfg(not(test))]
extern "C" {
    fn main() -> ();
}

extern "C" fn _start() -> ! {
    syscall::thread_exit()
}
