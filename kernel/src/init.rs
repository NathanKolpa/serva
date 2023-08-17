use core::mem::transmute;
use core::ops::Deref;

use bootloader::BootInfo;

use crate::arch::x86_64::init::GDT;
use crate::arch::x86_64::paging::{Page, PageSize, PageTableEntryFlags, PhysicalPage, VirtualPage};
use crate::arch::x86_64::segmentation::InterruptStackRef;
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
    let mut user_table_parent_flags = PageTableEntryFlags::default();
    user_table_parent_flags.set_present(true);
    user_table_parent_flags.set_writable(true);
    user_table_parent_flags.set_user_accessible(true);

    let mut user_table_flags = PageTableEntryFlags::default();
    user_table_flags.set_present(true);
    user_table_flags.set_writable(true);
    user_table_flags.set_user_accessible(true);

    let mut memory_mapper = MEMORY_MAPPER.write();

    let user_page_table = memory_mapper
        .new_l4_page_table(Some(PhysicalPage::active().0))
        .unwrap();

    let user_fn_in_kernel = VirtualAddress::new(user_mode_function as *const fn() as u64);
    let user_fn_in_phys = memory_mapper
        .translate_virtual_to_physical(user_fn_in_kernel, None)
        .unwrap();

    debug_println!("user_fn_in_kernel: {user_fn_in_kernel:?}");
    debug_println!("user_fn_in_phys: {user_fn_in_phys:?}");

    let user_fn_virt_page = VirtualPage::new(VirtualAddress::new(0x1600000), PageSize::Size4Kib);
    let user_fn_phys_page = PhysicalPage::new(user_fn_in_phys, PageSize::Size4Kib);
    let user_fn_ptr_offset_from_page = user_fn_in_phys.as_u64() - user_fn_phys_page.addr().as_u64();
    let user_fn_in_user_virt =
        VirtualAddress::new(user_fn_virt_page.addr().as_u64() + user_fn_ptr_offset_from_page);

    debug_println!(
        "Mapping {:?} to {:?}",
        user_fn_virt_page.addr(),
        user_fn_phys_page.addr()
    );
    debug_println!("user_fn_in_phys offset from page: {user_fn_ptr_offset_from_page}");
    debug_println!("user_fn_in_user_virt: {user_fn_in_user_virt:?}");

    unsafe {
        memory_mapper
            .map_to(
                user_table_flags,
                user_table_parent_flags,
                user_fn_virt_page,
                user_fn_phys_page,
                Some(user_page_table),
            )
            .unwrap();
    }

    let user_fn_in_phys_from_user_table = memory_mapper
        .translate_virtual_to_physical(user_fn_in_user_virt, Some(user_page_table))
        .unwrap();
    debug_println!("user_fn_in_phys_from_user_table: {user_fn_in_phys_from_user_table:?}");

    debug_println!("Setting user page active");

    unsafe {
        user_page_table.make_active();
    }

    debug_println!("Still works!");

    let user_fn: fn() = unsafe { transmute(user_fn_in_user_virt.as_u64()) };
    debug_println!(
        "user_fn_in_user: {:?}",
        VirtualAddress::new(user_fn as *const fn() as u64)
    );

    user_fn();
}

fn user_mode_function() -> ! {
    loop {}
}
