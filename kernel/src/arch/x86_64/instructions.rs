use core::arch::asm;

#[inline]
pub fn halt() -> ! {
    unsafe { asm!("hlt", options(noreturn)) }
}
