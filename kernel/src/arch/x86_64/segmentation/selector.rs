use core::arch::asm;

use crate::arch::x86_64::privilege::PrivilegeLevel;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct SegmentSelector {
    value: u16,
}

impl SegmentSelector {
    pub const fn empty() -> Self {
        Self { value: 0 }
    }

    pub const fn new(index: u16, privilege: PrivilegeLevel) -> Self {
        Self {
            value: index << 3 | privilege as u16,
        }
    }

    pub unsafe fn load_into_tss(&self) {
        unsafe {
            asm!("ltr {0:x}", in(reg) self.value, options(nostack, preserves_flags));
        }
    }

    pub unsafe fn load_into_cs(&self) {
        let value = self.value;
        asm!(
        "push {value}",
        "lea {tmp}, [1f + rip]",
        "push {tmp}",
        "retfq",
        "1:",
        value = in(reg) u64::from(value),
        tmp = lateout(reg) _,
        options(preserves_flags),
        );
    }
}
