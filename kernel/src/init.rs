use bootloader::BootInfo;

use crate::arch::x86_64::paging::PhysicalPage;
use crate::arch::x86_64::{halt_loop, init_x86_64, ARCH_NAME};
use crate::debug::DEBUG_CHANNEL;
use crate::arch::x86_64::devices::pic_8259::PIC_CHAIN;
use crate::arch::x86_64::interrupts::enable_interrupts;
use crate::memory::{MemoryMapper, FRAME_ALLOCATOR};

pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    debug_println!("Starting the Serva Operating System...");
    debug_println!("Architecture: {ARCH_NAME}");
    debug_println!("Debug channel: {DEBUG_CHANNEL}");

    init_x86_64();

    PIC_CHAIN.lock(); // init pic chain

    let memory_mapper = unsafe {
        FRAME_ALLOCATOR.init(&boot_info.memory_map);
        MemoryMapper::new(
            &FRAME_ALLOCATOR,
            PhysicalPage::active().0,
            boot_info.physical_memory_offset,
        )
    };

    debug_println!("{:#?}", FRAME_ALLOCATOR.info());

    test_syscall(memory_mapper.new_mapper(true).unwrap());

    halt_loop()
}


fn test_syscall(mut memory_map: MemoryMapper) {
    use crate::memory::TableCacheFlush;
    use crate::arch::x86_64::paging::*;
    use crate::util::address::*;
    use crate::arch::x86_64::trampoline::*;
    use crate::arch::x86_64::init::GDT;

    unsafe {
        init_syscall();
    }

    let mut user_flags = PageTableEntryFlags::default();
    user_flags.set_present(true);
    user_flags.set_writable(true);
    user_flags.set_user_accessible(true);

    let user_fn_virt = VirtualAddress::new(user_mode_function as *const () as u64);
    memory_map.set_flags(user_fn_virt, user_flags).discard();

    let stack_page = VirtualPage::new(VirtualAddress::new(0x800000), PageSize::Size4Kib);
    memory_map.new_map(user_flags, user_flags, stack_page).unwrap().discard();

    unsafe {
        memory_map.l4_page().make_active();

        return_from_interrupt(
            user_fn_virt,
            stack_page.end_addr(),
            GDT.user_code,
            GDT.user_data
        )
    }
}

extern "C" fn user_mode_function() {
    loop {
        unsafe {
            core::arch::asm!(
                "mov rax, 0xCA11",
                "mov rdi, 10",
                "mov rsi, 20",
                "mov rdx, 30",
                "mov r10, 40",
                "syscall"
            )
        }
    }
}
