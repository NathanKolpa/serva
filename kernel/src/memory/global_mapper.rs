use bootloader::BootInfo;

use crate::arch::x86_64::paging::*;
use crate::memory::frame_allocator::{BootInfoFrameAllocator, FrameAllocator};
use crate::memory::mapper::*;
use crate::memory::MemoryInfo;
use crate::util::sync::{Expected, SpinRwLock};

pub struct GlobalMemoryMapper<A, M> {
    frame_allocator: A,
    memory_mapper: M,
}

impl<M> GlobalMemoryMapper<BootInfoFrameAllocator, M> {
    pub fn info(&self) -> MemoryInfo {
        self.frame_allocator.info()
    }
}

impl<A, M> GlobalMemoryMapper<A, M>
where
    A: FrameAllocator,
    M: MemoryMapper,
{
    fn new_map(
        &mut self,
        flags: PageTableEntryFlags,
        parent_flags: PageTableEntryFlags,
        new_page: VirtualPage,
        l4_page_table: Option<PhysicalPage>,
    ) -> Result<(), NewMappingError> {
        self.memory_mapper.new_map(
            &self.frame_allocator,
            flags,
            parent_flags,
            new_page,
            l4_page_table,
        )
    }
}

pub type GlobalMapperImpl = GlobalMemoryMapper<BootInfoFrameAllocator, PhysicalMemoryMapper>;
pub static MEMORY_MAPPER: SpinRwLock<Expected<GlobalMapperImpl>> = SpinRwLock::new(Expected::new());

pub unsafe fn init_memory_mapper(boot_info: &'static BootInfo) {
    let mut lock = MEMORY_MAPPER.write();
    lock.set(GlobalMemoryMapper {
        memory_mapper: PhysicalMemoryMapper::new(boot_info.physical_memory_offset),
        frame_allocator: BootInfoFrameAllocator::new(&boot_info.memory_map),
    });
}
