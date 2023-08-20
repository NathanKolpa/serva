use crate::arch::x86_64::paging::*;
use crate::memory::flush::{TableCacheFlush, TableListCacheFlush};
use crate::memory::frame_allocator::FrameAllocator;
use crate::util::address::*;

#[derive(Debug, Clone, Copy)]
pub enum NewMappingError {
    AlreadyMapped,
    OutOfFrames,
}

pub struct MemoryMapper {
    frame_allocator: &'static FrameAllocator,
    l4_page: PhysicalPage,
    global_offset: u64,
}

impl MemoryMapper {
    /// ## Safety
    /// The caller must guarantee that:
    /// 1. There is only one mapper at a given time.
    /// 2. The complete physical memory is mapped to virtual memory at the passed `global_offset`.
    /// 3. The passed `l4_page` points to a valid level 4 page.
    pub unsafe fn new(
        frame_allocator: &'static FrameAllocator,
        l4_page: PhysicalPage,
        global_offset: u64,
    ) -> Self {
        Self {
            frame_allocator,
            l4_page,
            global_offset,
        }
    }

    fn translate_table_frame(&self, phys: PhysicalAddress) -> VirtualAddress {
        let a = phys.as_u64() + self.global_offset;
        a.into()
    }

    /// Get the physical address from a virtual address.
    pub fn translate_virtual_to_physical(&self, addr: VirtualAddress) -> Option<PhysicalAddress> {
        let mut frame = self.l4_page;
        let mut offset = addr.page_offset() as u64;

        for (page_level, index) in addr
            .indices()
            .into_iter()
            .enumerate()
            .map(|(i, index)| (4 - i, index as usize))
        {
            let table_ptr: *const PageTable = self.translate_table_frame(frame.addr()).as_ptr();
            let table = unsafe { &*table_ptr };

            let entry = table.as_slice()[index];

            if !entry.flags().present() {
                return None;
            }

            frame = entry.as_frame(page_level);

            match (page_level, entry.flags().huge()) {
                (2, true) => {
                    offset = addr.l2_page_offset();
                    break;
                }
                (1, true) => {
                    offset = addr.l1_page_offset();
                    break;
                }
                (_, true) => panic!("Unexpected huge page in L{page_level} entry"),
                (_, false) => {}
            }
        }

        Some(frame.addr() + offset)
    }

    pub fn set_flags(
        &mut self,
        address: VirtualAddress,
        flags: PageTableEntryFlags,
    ) -> impl TableCacheFlush {
        let mut cache_flush = TableListCacheFlush::new();
        let mut frame = self.l4_page;

        for index in address.indices() {
            let table_ptr: *mut PageTable = self.translate_table_frame(frame.addr()).as_mut_ptr();
            let table = unsafe { &mut *table_ptr };
            let entry = &mut table.as_mut_slice()[index as usize];

            if !entry.flags().contains(flags) {
                entry.set_flags(entry.flags() | flags);
                cache_flush.add_table(frame);
            }

            if entry.flags().huge() {
                break;
            }

            frame = PhysicalPage::new(entry.addr(), PageSize::Size4Kib);
        }

        cache_flush
    }

    pub fn new_mapper(&self, inherit: bool) -> Result<Self, NewMappingError> {
        let new_frame = self
            .frame_allocator
            .allocate_new_page_table()
            .ok_or(NewMappingError::OutOfFrames)?;

        let table_ptr: *mut PageTable = self.translate_table_frame(new_frame.addr()).as_mut_ptr();
        let table = unsafe { &mut *table_ptr };

        if inherit {
            let clone_table_ptr: *const PageTable =
                self.translate_table_frame(self.l4_page.addr()).as_ptr();
            let clone_table = unsafe { &*clone_table_ptr };

            table.as_mut_slice().copy_from_slice(clone_table.as_slice())
        } else {
            table.zero();
        }

        Ok(Self {
            l4_page: new_frame,
            frame_allocator: self.frame_allocator,
            global_offset: self.global_offset,
        })
    }

    fn map_to_inner(
        &mut self,
        flags: PageTableEntryFlags,
        parent_flags: PageTableEntryFlags,
        new_page: VirtualPage,
        frame: PhysicalPage,
    ) -> Result<impl TableCacheFlush, NewMappingError> {
        let mut cache_flush = TableListCacheFlush::new();
        let mut table_frame = self.l4_page;

        for (page_level, index) in new_page
            .addr()
            .indices()
            .into_iter()
            .enumerate()
            .map(|(i, index)| (4 - i, index as usize))
        {
            let table_ptr: *mut PageTable =
                self.translate_table_frame(table_frame.addr()).as_mut_ptr();

            let table = unsafe { &mut *table_ptr };

            let mut entry = table.as_slice()[index];

            match (page_level, entry.flags().present()) {
                (1, true) => {
                    return Err(NewMappingError::AlreadyMapped);
                }
                (1, false) => {
                    entry.set_flags(flags);
                    entry.set_addr(frame.addr());
                    table.as_mut_slice()[index] = entry;
                    cache_flush.add_table(table_frame);
                }
                (_, false) => {
                    let allocated_page = self
                        .frame_allocator
                        .allocate_new_page_table()
                        .ok_or(NewMappingError::OutOfFrames)?;

                    let new_table_ptr: *mut PageTable = self
                        .translate_table_frame(allocated_page.addr())
                        .as_mut_ptr();

                    let new_table = unsafe { &mut *new_table_ptr };

                    new_table.zero();

                    entry.set_flags(parent_flags | entry.flags());
                    entry.set_addr(allocated_page.addr());
                    table.as_mut_slice()[index] = entry;
                    cache_flush.add_table(table_frame);
                }
                _ => {}
            }

            table_frame = entry.as_frame(page_level);
        }

        Ok(cache_flush)
    }

    /// Creates a new mapping in the page table to the specified physical memory.
    pub unsafe fn map_to(
        &mut self,
        flags: PageTableEntryFlags,
        parent_flags: PageTableEntryFlags,
        new_page: VirtualPage,
        frame: PhysicalPage,
    ) -> Result<impl TableCacheFlush, NewMappingError> {
        self.map_to_inner(flags, parent_flags, new_page, frame)
    }

    /// Creates a new mapping in the page table.
    pub fn new_map(
        &mut self,
        flags: PageTableEntryFlags,
        parent_flags: PageTableEntryFlags,
        new_page: VirtualPage,
    ) -> Result<impl TableCacheFlush, NewMappingError> {
        let frame = self
            .frame_allocator
            .allocate_new_page_table()
            .ok_or(NewMappingError::OutOfFrames)?;

        unsafe { self.map_to(flags, parent_flags, new_page, frame) }
    }

    pub fn l4_page(&self) -> PhysicalPage {
        self.l4_page
    }
}
