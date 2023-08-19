use core::{arch::asm, fmt::Debug};

use crate::util::address::*;

#[derive(Clone, Copy, Default)]
pub struct PageTableEntryFlags {
    value: u64,
}

impl PageTableEntryFlags {
    pub fn contains(&self, other: Self) -> bool {
        (self.value & other.value) == self.value
    }

    pub fn set_present(&mut self, enabled: bool) {
        self.set_flag(0, enabled)
    }

    pub fn set_huge(&mut self, enabled: bool) {
        self.set_flag(7, enabled)
    }

    pub fn set_writable(&mut self, enabled: bool) {
        self.set_flag(1, enabled)
    }

    pub fn set_user_accessible(&mut self, enabled: bool) {
        self.set_flag(2, enabled)
    }

    fn set_flag(&mut self, bit: u64, enabled: bool) {
        if enabled {
            self.value |= 1 << bit;
        } else {
            self.value &= !(1 << bit)
        }
    }

    pub fn used(&self) -> bool {
        self.value != 0
    }

    pub fn present(&self) -> bool {
        self.value & (1 << 0) != 0
    }

    pub fn writable(&self) -> bool {
        self.value & (1 << 1) != 0
    }

    pub fn dirty(&self) -> bool {
        self.value & (1 << 6) != 0
    }

    pub fn global(&self) -> bool {
        self.value & (1 << 8) != 0
    }

    pub fn noexec(&self) -> bool {
        self.value & (1 << 63) != 0
    }

    pub fn huge(&self) -> bool {
        self.value & (1 << 7) != 0
    }

    pub fn user_accessible(&self) -> bool {
        self.value & (1 << 2) != 0
    }
}

impl core::ops::BitOr for PageTableEntryFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value | rhs.value,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PageTableEntry {
    value: u64,
}

impl PageTableEntry {
    const ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;

    pub fn new(flags: PageTableEntryFlags, addr: PhysicalAddress) -> Self {
        let flags_masked = flags.value & (!Self::ADDR_MASK);
        let addr_masked = addr.as_u64() & Self::ADDR_MASK;

        Self {
            value: flags_masked | addr_masked,
        }
    }

    pub fn set_flags(&mut self, flags: PageTableEntryFlags) {
        self.value = self.value ^ ((self.value ^ flags.value) & (!Self::ADDR_MASK));
    }

    pub fn set_addr(&mut self, addr: PhysicalAddress) {
        self.value = self.value ^ ((self.value ^ addr.as_u64()) & Self::ADDR_MASK);
    }

    pub fn flags(&self) -> PageTableEntryFlags {
        PageTableEntryFlags { value: self.value }
    }

    pub fn addr(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.value & Self::ADDR_MASK)
    }

    pub fn as_frame(&self, level: usize) -> PhysicalPage {
        let size = if self.flags().huge() {
            PageSize::from_level(level)
        } else {
            PageSize::Size4Kib
        };

        PhysicalPage::new(self.addr(), size)
    }
}

impl Debug for PageTableEntryFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PageTableEntryFlags")
            .field("present", &self.present())
            .field("writable", &self.writable())
            .field("huge", &self.huge())
            .field("dirty", &self.dirty())
            .field("global", &self.global())
            .field("noexec", &self.noexec())
            .field("user_accessible", &self.user_accessible())
            .finish()
    }
}

impl Debug for PageTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PageTableEntry")
            .field("addr", &self.addr())
            .field("flags", &self.flags())
            .finish()
    }
}

#[repr(align(4096))]
#[derive(Debug)]
#[repr(C)]
pub struct PageTable<const SIZE: usize = 512> {
    entries: [PageTableEntry; SIZE],
}

impl<const SIZE: usize> PageTable<SIZE> {
    pub fn as_slice(&self) -> &[PageTableEntry] {
        &self.entries
    }

    pub fn as_mut_slice(&mut self) -> &mut [PageTableEntry] {
        &mut self.entries
    }

    pub fn flush(&self) {
        let addr = self as *const Self as u64;

        unsafe {
            asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
        }
    }

    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = PageTableEntry::default()
        }
    }
}

pub type HugeL1Table = PageTable<262144>;
pub type HugeL2Table = PageTable<134217728>;

#[derive(Clone, Copy)]
pub enum PageSize {
    Size4Kib,
    Size2Mib,
    Size1Gib,
}

impl PageSize {
    pub const fn from_level(level: usize) -> Self {
        match level {
            2 => PageSize::Size2Mib,
            3 => PageSize::Size1Gib,
            1 | 4 => PageSize::Size4Kib,
            _ => panic!("Page level must be between 1 and 4"),
        }
    }

    pub fn as_bytes(&self) -> u64 {
        match self {
            PageSize::Size4Kib => 4096,
            PageSize::Size2Mib => 4096 * 512,
            PageSize::Size1Gib => 4096 * 512 * 512,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Page<A> {
    addr: Address<A>,
    size: PageSize,
}

impl<A: Copy> Page<A> {
    pub fn addr(&self) -> Address<A> {
        self.addr
    }

    pub fn size(&self) -> PageSize {
        self.size
    }

    pub fn end_addr(&self) -> Address<A> {
        Address::new(self.addr.as_u64() + self.size.as_bytes())
    }
}

impl Page<VirtualAddressMarker> {
    pub fn new(mut addr: VirtualAddress, size: PageSize) -> Self {
        addr.align_down(size.as_bytes());
        Self { addr, size }
    }
}

impl Page<PhysicalAddressMarker> {
    pub fn new(mut addr: PhysicalAddress, size: PageSize) -> Self {
        addr.align_down(size.as_bytes());
        Self { addr, size }
    }

    pub fn active() -> (Self, u16) {
        let value: u64;

        unsafe {
            asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
        }

        let addr = PhysicalAddress::new(value & 0x_000f_ffff_ffff_f000);

        (Self::new(addr, PageSize::Size4Kib), (value & 0xFFF) as u16)
    }

    pub unsafe fn make_active(&self) {
        let addr = self.addr.as_u64(); // TODO: flags are also used idk
        asm!("mov cr3, {}", in(reg) addr, options(nostack, preserves_flags));
    }
}

pub type VirtualPage = Page<VirtualAddressMarker>;
pub type PhysicalPage = Page<PhysicalAddressMarker>;
