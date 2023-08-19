use core::sync::atomic::{AtomicUsize, Ordering};

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

use crate::arch::x86_64::paging::{PageSize, PhysicalPage};
use crate::memory::MemoryInfo;
use crate::util::sync::SpinRwLock;
use crate::util::Expected;

const SIZE: PageSize = PageSize::Size4Kib;

pub struct FrameAllocator {
    memory_map: SpinRwLock<Expected<&'static MemoryMap>>,
    next: AtomicUsize,
}

impl FrameAllocator {
    pub const fn new() -> Self {
        Self {
            memory_map: SpinRwLock::new(Expected::new()),
            next: AtomicUsize::new(0),
        }
    }

    pub unsafe fn init(&self, memory_map: &'static MemoryMap) {
        let mut lock = self.memory_map.write();
        lock.set(memory_map);
    }

    pub fn info(&self) -> MemoryInfo {
        let memory_map = self.memory_map.read().clone();

        let current = self.next.load(Ordering::Relaxed);

        let bytes_allocated = current * SIZE.as_bytes() as usize;

        let mut total_allocatable_bytes = 0;
        let mut total_bytes = 0;
        let mut kernel = 0;

        let regions = memory_map.iter().map(|x| {
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

    pub fn allocate_new_page_table(&self) -> Option<PhysicalPage> {
        let memory_map = self.memory_map.read().clone();

        let mut usable_pages = memory_map
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

pub static FRAME_ALLOCATOR: FrameAllocator = FrameAllocator::new();
