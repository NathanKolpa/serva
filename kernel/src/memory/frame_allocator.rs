use core::sync::atomic::{AtomicUsize, Ordering};

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

use crate::arch::x86_64::paging::{PageSize, PhysicalPage};
use crate::memory::MemoryInfo;

const SIZE: PageSize = PageSize::Size4Kib;

pub unsafe trait FrameAllocator {
    fn allocate_new_page_table(&self) -> Option<PhysicalPage>;
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: AtomicUsize,
}

impl BootInfoFrameAllocator {
    pub fn new(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            next: AtomicUsize::new(0),
        }
    }

    pub fn info(&self) -> MemoryInfo {
        let current = self.next.load(Ordering::Relaxed);

        let bytes_allocated = current * SIZE.as_bytes() as usize;

        let mut total_allocatable_bytes = 0;
        let mut total_bytes = 0;
        let mut kernel = 0;

        let regions = self.memory_map.iter().map(|x| {
            (
                x.region_type,
                (x.range.end_frame_number * SIZE.as_bytes()
                    - x.range.start_frame_number * SIZE.as_bytes()) as usize,
            )
        });

        for (kind, region_size) in regions {
            match &kind {
                MemoryRegionType::Usable => total_allocatable_bytes += region_size,
                MemoryRegionType::KernelStack | MemoryRegionType::Kernel => kernel += region_size,
                _ => {}
            }

            if kind != MemoryRegionType::Reserved {
                total_bytes += region_size;
            }
        }

        MemoryInfo {
            allocated: bytes_allocated,
            usable: total_allocatable_bytes,
            total_size: total_bytes,
            kernel,
        }
    }
}

unsafe impl FrameAllocator for BootInfoFrameAllocator {
    fn allocate_new_page_table(&self) -> Option<PhysicalPage> {
        let mut usable_pages = self
            .memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .flat_map(|r| {
                (r.range.start_addr()..r.range.end_addr()).step_by(SIZE.as_bytes() as usize)
            })
            .map(|addr| PhysicalPage::new(addr.into(), SIZE));

        let index = self.next.fetch_add(1, Ordering::AcqRel);
        usable_pages.nth(index)
    }
}
