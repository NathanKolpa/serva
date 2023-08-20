use core::arch::asm;
use crate::arch::x86_64::RFlags;

#[inline]
pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nomem, nostack));
    }
}

#[inline]
pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}

/// Run a block of code (aka the `callback` argument) that is guaranteed to be executed without interrupts.
/// After completing the function, the interrupt status flag is restored to its original state.
#[inline]
pub fn atomic_block<F: FnOnce()>(callback: F) {
    let ints_enabled = RFlags::read().interrupts_enabled();

    if ints_enabled {
        disable_interrupts();
    }

    callback();

    if ints_enabled {
        enable_interrupts();
    }
}

pub fn int3() {
    unsafe {
        asm!("int3", options(nomem, nostack));
    }
}
