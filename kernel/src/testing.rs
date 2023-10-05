use crate::arch::x86_64::devices::QEMU_DEVICE;
use core::panic::PanicInfo;
use x86_64::devices::qemu::ExitCode;
use x86_64::instructions::halt_loop;

pub trait TestCase {
    fn run(&self);
}

impl<T> TestCase for T
where
    T: Fn(),
{
    fn run(&self) {
        debug_print!("{}...\t", core::any::type_name::<T>());
        self();
        debug_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn TestCase]) {
    debug_println!("Running {} tests", tests.len());

    for test in tests {
        test.run();
    }

    QEMU_DEVICE.lock().exit(ExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    debug_println!("[failed]\n");
    debug_println!("Error: {}\n", info);
    QEMU_DEVICE.lock().exit(ExitCode::Failed);
    halt_loop()
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    crate::test_main();
    halt_loop()
}

#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
