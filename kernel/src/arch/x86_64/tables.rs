use crate::util::address::VirtualAddress;

#[derive(Clone, Copy)]
#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: VirtualAddress,
}

impl DescriptorTablePointer {
    #[inline]
    pub unsafe fn load_interrupt_table(&self) {
        core::arch::asm!("lidt [{}]", in(reg) self, options(readonly, nostack, preserves_flags));
    }

    #[inline]
    pub unsafe fn load_descriptor_table(&self) {
        core::arch::asm!("lgdt [{}]", in(reg) self, options(readonly, nostack, preserves_flags));
    }
}
