pub use phys_mapper::*;

use crate::arch::x86_64::paging::*;
use crate::memory::frame_allocator::FrameAllocator;
use crate::util::address::*;

mod phys_mapper;

#[derive(Debug, Clone, Copy)]
pub enum NewMappingError {
    AlreadyMapped,
    OutOfFrames,
}

pub trait MemoryMapper {
    /// Get the physical address from a virtual address.
    fn translate_physical_to_virtual(&self, addr: VirtualAddress) -> Option<PhysicalAddress>;

    /// Creates a new mapping in the page table to the specified physical memory.
    unsafe fn map_to(
        &mut self,
        allocator: &impl FrameAllocator,
        flags: PageTableEntryFlags,
        parent_flags: PageTableEntryFlags,
        new_page: VirtualPage,
        frame: PhysicalPage,
        l4_page_table: Option<PhysicalPage>,
    ) -> Result<(), NewMappingError>;

    /// Creates a new mapping in the page table.
    fn new_map(
        &mut self,
        allocator: &impl FrameAllocator,
        flags: PageTableEntryFlags,
        parent_flags: PageTableEntryFlags,
        new_page: VirtualPage,
        l4_page_table: Option<PhysicalPage>,
    ) -> Result<(), NewMappingError>;
}
