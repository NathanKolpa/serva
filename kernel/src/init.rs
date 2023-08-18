use core::arch::asm;
use core::mem::transmute;
use core::ops::{Add, Deref};

use bootloader::BootInfo;

use crate::arch::x86_64::init::GDT;
use crate::arch::x86_64::paging::{Page, PageSize, PageTableEntryFlags, PhysicalPage, VirtualPage};
use crate::arch::x86_64::segmentation::{InterruptStackRef, SegmentDescriptor, NormalSegment};
use crate::arch::x86_64::trampoline::enter_ring3;
use crate::arch::x86_64::{halt, init_x86_64, ARCH_NAME};
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
    debug_println!("User data {:?}", NormalSegment::USER_DATA.as_u64());

    let mut user_table_flags = PageTableEntryFlags::default();
    user_table_flags.set_present(true);
    user_table_flags.set_writable(true);
    user_table_flags.set_user_accessible(true);

    let mut memory_mapper = MEMORY_MAPPER.write();

    let user_page_table = memory_mapper
        .new_l4_page_table(Some(PhysicalPage::active().0))
        .unwrap();

    let user_fn_virt = VirtualAddress::new(user_mode_function as *const () as u64);
    memory_mapper.update_flags(user_table_flags, user_fn_virt, Some(user_page_table));
    debug_println!("user_fn_virt: {user_fn_virt:?}");

    let stack_page = VirtualPage::new(VirtualAddress::new(0x800000), PageSize::Size4Kib);
    let stack_addr = stack_page.addr().add(100);

    memory_mapper
        .new_map(
            user_table_flags,
            user_table_flags,
            stack_page,
            Some(user_page_table),
        )
        .unwrap();


    debug_println!("stack_page: {:?}", stack_page.addr());

    debug_println!("Setting user page active");

    debug_println!("User DS: {:?}", GDT.user_data);
    debug_println!("User CS: {:?}", GDT.user_code);

    unsafe {
        user_page_table.make_active();

        enter_ring3(
            user_fn_virt,
            stack_addr,
            GDT.user_code,
            GDT.user_data,
        );
    }
}

fn user_mode_function() {
    unsafe {
        asm!("nop");
        asm!("nop");
        asm!("nop");
    }
}
