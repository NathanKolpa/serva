use core::arch::asm;
use core::mem::transmute;
use core::ops::{Add, Deref};

use bootloader::BootInfo;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, PageTableFlags, PhysFrame, Size4KiB, Translate};

use crate::arch::x86_64::init::GDT;
use crate::arch::x86_64::paging::{Page, PageSize, PageTable, PageTableEntryFlags, PhysicalPage, VirtualPage};
use crate::arch::x86_64::segmentation::{InterruptStackRef, SegmentDescriptor, NormalSegment};
use crate::arch::x86_64::trampoline::return_from_interrupt;
use crate::arch::x86_64::{halt, init_x86_64, ARCH_NAME};
use crate::arch::x86_64::interrupts::disable_interrupts;
use crate::debug::DEBUG_CHANNEL;
use crate::memory::{init_memory_mapper, MEMORY_MAPPER};
use crate::util::address::{PhysicalAddress, VirtualAddress};

pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    debug_println!("Starting the Serva Operating System...");
    debug_println!("Architecture: {ARCH_NAME}");
    debug_println!("Debug channel: {DEBUG_CHANNEL}");

    init_x86_64();

    unsafe {
        init_memory_mapper(boot_info);
    }

    debug_println!("{:#?}", MEMORY_MAPPER.read().info());

    user_mode_test(boot_info);

    halt()
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item=PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

fn user_mode_test(boot_info: &'static BootInfo) {
    debug_println!("User data {:?}", NormalSegment::USER_DATA.as_u64());


    let l4_table_addr = Cr3::read().0.start_address() + boot_info.physical_memory_offset;
    let l4_table = unsafe { &mut *(l4_table_addr.as_u64() as *mut x86_64::structures::paging::PageTable) };


    let mut mapper =
        unsafe { OffsetPageTable::new(l4_table, VirtAddr::new(boot_info.physical_memory_offset)) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    let flags =
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    let user_fn_addr = VirtAddr::from_ptr(user_mode_function as *const ());
    let user_fn_page: x86_64::structures::paging::Page<Size4KiB> = x86_64::structures::paging::Page::containing_address(user_fn_addr);

    unsafe {
        mapper
            .set_flags_p4_entry(user_fn_page, flags)
            .unwrap()
            .flush_all();
    }
    unsafe {
        mapper
            .set_flags_p3_entry(user_fn_page, flags)
            .unwrap()
            .flush_all();
    }
    unsafe {
        mapper
            .set_flags_p2_entry(user_fn_page, flags)
            .unwrap()
            .flush_all();
    }
    unsafe {
        mapper.update_flags(user_fn_page, flags).unwrap().flush();
    }

    static USER_STACK: [u8; 1000] = [0; 1000];

    let stack_phys = mapper.translate_addr(VirtAddr::from_ptr(&USER_STACK as *const u8)).unwrap();
    let stack_page: x86_64::structures::paging::Page<Size4KiB> = x86_64::structures::paging::Page::containing_address(VirtAddr::new(0x800000));

    unsafe {
        mapper.map_to(
            stack_page,
            PhysFrame::containing_address(stack_phys),
            flags,
            &mut frame_allocator,
        ).unwrap()
            .flush();
    }

    /*let mut user_table_flags = PageTableEntryFlags::default();
    user_table_flags.set_present(true);
    user_table_flags.set_writable(true);
    user_table_flags.set_user_accessible(true);

    let mut memory_mapper = MEMORY_MAPPER.write();

    // let user_page_table = memory_mapper
    //     .new_l4_page_table(Some(PhysicalPage::active().0))
    //     .unwrap();

    let user_fn_virt = VirtualAddress::new(user_mode_function as *const () as u64);
    let user_fn_virt_page = VirtualPage::new(user_fn_virt, PageSize::Size4Kib);
    memory_mapper.update_flags(user_table_flags, user_fn_virt_page.addr(), None);
    debug_println!("user_fn_virt: {user_fn_virt:?}");

    let stack_page = VirtualPage::new(VirtualAddress::new(0x800000), PageSize::Size4Kib);
    let stack_addr = stack_page.addr().add(100);

    memory_mapper
        .new_map(
            user_table_flags,
            user_table_flags,
            stack_page,
            None,
        )
        .unwrap();

*/
    debug_println!("User DS: {:?}", GDT.user_data);
    debug_println!("User CS: {:?}", GDT.user_code);

    unsafe {
        disable_interrupts();

        return_from_interrupt(
            VirtualAddress::new(user_fn_addr.as_u64()),
            VirtualAddress::new(0x800100),
            GDT.user_code,
            GDT.user_data,
        );
    }
}

extern "C" fn user_mode_function() {
    loop {
        unsafe {
            asm!("nop");
        }
    }
}
