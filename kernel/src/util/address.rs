use core::{
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
    ops::Add,
};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtualAddressMarker {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysicalAddressMarker {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(C)]
pub struct Address<L> {
    addr: u64,
    _phantom: PhantomData<L>,
}

pub type VirtualAddress = Address<VirtualAddressMarker>;
pub type PhysicalAddress = Address<PhysicalAddressMarker>;

impl<L> Address<L> {
    pub const fn new(addr: u64) -> Self {
        Self {
            addr,
            _phantom: PhantomData,
        }
    }

    pub fn align_ptr_up(addr: u64, align: u64) -> u64 {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        (addr + align - 1) & !(align - 1)
    }

    pub fn align_ptr_down(addr: u64, align: u64) -> u64 {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        addr & !(align - 1)
    }

    pub fn as_u64(&self) -> u64 {
        self.addr
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.as_u64() as *const T
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.as_u64() as *mut T
    }
}

impl Address<PhysicalAddressMarker> {
    pub fn align_down(&mut self, align: u64) {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        self.addr = Self::align_ptr_down(self.addr, align)
    }
}

impl Address<VirtualAddressMarker> {
    pub fn align_down(&mut self, align: u64) {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        self.addr = Self::align_ptr_down(self.addr, align);
        self.addr = ((self.addr << 16) as i64 >> 16) as u64
    }

    fn truncate_index(value: u64) -> u64 {
        value % 512
    }

    pub fn indices(&self) -> [u16; 4] {
        [
            Self::truncate_index(self.addr >> 12 >> 9 >> 9 >> 9) as u16,
            Self::truncate_index(self.addr >> 12 >> 9 >> 9) as u16,
            Self::truncate_index(self.addr >> 12 >> 9) as u16,
            Self::truncate_index(self.addr >> 12) as u16,
        ]
    }

    pub fn page_offset(&self) -> u16 {
        (self.addr as u16) % (1 << 12)
    }

    pub fn l2_page_offset(&self) -> u64 {
        self.addr & 0o_777_777_7777
    }

    pub fn l1_page_offset(&self) -> u64 {
        self.addr & 0o_777_7777
    }
}

impl<T> Add<u64> for Address<T> {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self::new(self.addr + rhs)
    }
}

impl<L> From<u64> for Address<L> {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl<T> Display for Address<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.addr)
    }
}

impl Debug for Address<PhysicalAddressMarker> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "p{}", self)
    }
}

impl Debug for Address<VirtualAddressMarker> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "v{}", self)
    }
}
