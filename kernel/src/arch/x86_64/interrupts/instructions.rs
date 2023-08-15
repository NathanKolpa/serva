use core::arch::asm;

#[repr(transparent)]
pub struct RFlags {
    value: u64,
}

impl RFlags {
    const INTERRUPTS_ENABLED: u64 = 1 << 9;

    #[inline]
    pub fn read() -> Self {
        let value: u64;

        unsafe {
            asm!("pushfq; pop {}", out(reg) value, options(nomem, preserves_flags));
        }

        Self { value }
    }

    pub fn interrupts_enabled(&self) -> bool {
        self.value & Self::INTERRUPTS_ENABLED != 0
    }
}

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
