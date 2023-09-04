use linked_list_allocator::LockedHeap;
use crate::arch::x86_64::paging::{PageSize, PageTableEntryFlags, VirtualPage};
use crate::memory::{MemoryMapper, NewMappingError};
use crate::util::address::VirtualAddress;
use crate::memory::flush::TableCacheFlush;

const HEAP_START: VirtualAddress = VirtualAddress::new(0x_4444_4444_0000);
pub const HEAP_SIZE: usize = 100 * 1024;

pub fn map_heap(mapper: &mut MemoryMapper) -> Result<(), NewMappingError> {
    const PAGE_SIZE: PageSize = PageSize::Size4Kib;

    let mut flags = PageTableEntryFlags::default();
    flags.set_writable(true);
    flags.set_present(true);

    for page_addr in (HEAP_START.as_usize()..HEAP_START.as_usize() + HEAP_SIZE)
        .step_by(PAGE_SIZE.as_usize())
        .map(VirtualAddress::new)
    {
        let new_page = VirtualPage::new(page_addr, PAGE_SIZE);

        mapper.new_map(flags, flags, new_page)?.flush();
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START.as_mut_ptr(), HEAP_SIZE);
    }

    Ok(())
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
