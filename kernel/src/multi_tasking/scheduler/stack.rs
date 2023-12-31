use essentials::address::VirtualAddress;
use x86_64::paging::VirtualPage;

pub struct ThreadStack {
    top: VirtualAddress,
}

impl ThreadStack {
    pub fn from_slice(slice: &'static mut [u8]) -> Self {
        Self {
            top: VirtualAddress::from(slice.as_ptr()) + slice.len(),
        }
    }

    pub fn from_page(page: VirtualPage) -> Self {
        Self {
            top: page.end_addr(),
        }
    }

    pub fn top(&self) -> VirtualAddress {
        self.top
    }
}
