use crate::arch::x86_64::paging::*;
use crate::memory::frame_allocator::FrameAllocator;
use crate::memory::mapper::{MemoryMapper, NewMappingError};
use crate::util::address::*;

pub struct PhysicalMemoryMapper {
    global_offset: u64,
}

impl PhysicalMemoryMapper {
    /// ## Safety
    /// The caller must guarantee that:
    /// 1. There is only one mapper at a given time.
    /// 2. The complete physical memory is mapped to virtual memory at the passed `global_offset`.
    pub unsafe fn new(global_offset: u64) -> Self {
        PhysicalMemoryMapper { global_offset }
    }

    fn translate_table_frame(&self, phys: PhysicalAddress) -> VirtualAddress {
        let a = phys.as_u64() + self.global_offset;
        a.into()
    }

    fn map_to_inner(
        &mut self,
        allocator: &impl FrameAllocator,
        flags: PageTableEntryFlags,
        parent_flags: PageTableEntryFlags,
        new_page: VirtualPage,
        frame: PhysicalPage,
        l4_page_table: Option<PhysicalPage>,
    ) -> Result<(), NewMappingError> {
        let mut table_frame = l4_page_table.unwrap_or_else(|| PhysicalPage::active().0);

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
                    table.flush();
                }
                (_, false) => {
                    let allocated_page = allocator
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
                    table.flush();
                }
                _ => {}
            }

            table_frame = entry.as_frame(page_level);
        }

        Ok(())
    }
}

impl MemoryMapper for PhysicalMemoryMapper {
    fn translate_physical_to_virtual(&self, addr: VirtualAddress) -> Option<PhysicalAddress> {
        let (mut frame, _) = PhysicalPage::active();
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

    unsafe fn map_to(
        &mut self,
        allocator: &impl FrameAllocator,
        flags: PageTableEntryFlags,
        parent_flags: PageTableEntryFlags,
        new_page: VirtualPage,
        frame: PhysicalPage,
        l4_page_table: Option<PhysicalPage>,
    ) -> Result<(), NewMappingError> {
        self.map_to_inner(
            allocator,
            flags,
            parent_flags,
            new_page,
            frame,
            l4_page_table,
        )
    }

    fn new_map(
        &mut self,
        allocator: &impl FrameAllocator,
        flags: PageTableEntryFlags,
        parent_flags: PageTableEntryFlags,
        new_page: VirtualPage,
        l4_page_table: Option<PhysicalPage>,
    ) -> Result<(), NewMappingError> {
        let frame = allocator
            .allocate_new_page_table()
            .ok_or(NewMappingError::OutOfFrames)?;

        unsafe {
            self.map_to(
                allocator,
                flags,
                parent_flags,
                new_page,
                frame,
                l4_page_table,
            )
        }
    }
}
