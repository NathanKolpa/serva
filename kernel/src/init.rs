use core::arch::asm;
use core::mem::transmute;
use core::ops::{Add, Deref};

use bootloader::BootInfo;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

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

    user_mode_test();

    halt()
}

fn user_mode_test() {
    let mut user_table_flags = PageTableEntryFlags::default();
    user_table_flags.set_present(true);
    user_table_flags.set_writable(true);
    user_table_flags.set_user_accessible(true);

    let mut memory_mapper = MEMORY_MAPPER.write();

    let user_page_table = memory_mapper
        .new_l4_page_table(Some(PhysicalPage::active().0))
        .unwrap();


    let user_fn_virt = VirtualAddress::new(user_mode_function as *const () as u64);
    let user_fn_virt_page = VirtualPage::new(user_fn_virt, PageSize::Size4Kib);
    memory_mapper.update_flags(user_table_flags, user_fn_virt_page.addr(), Some(user_page_table));
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

    debug_println!("User DS: {:?}", GDT.user_data);
    debug_println!("User CS: {:?}", GDT.user_code);


    unsafe {
        user_page_table.make_active();

        disable_interrupts();

        return_from_interrupt(
            user_fn_virt,
            stack_addr,
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
