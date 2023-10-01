use core::mem::MaybeUninit;

use crate::arch::x86_64::paging::{PageSize, PageTableEntryFlags, VirtualPage};
use crate::arch::x86_64::syscalls::SyscallArgs;
use crate::memory::{MemoryMapper, NewMappingError, TableCacheFlush};
use crate::util::address::VirtualAddress;

// TODO: is there a better way of putting a function at a known location?

static mut SYSCALL_HANDLER: MaybeUninit<fn(SyscallArgs) -> u64> = MaybeUninit::uninit();

extern "C" fn syscall_handler(syscall: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe {
        (SYSCALL_HANDLER.assume_init_ref())(SyscallArgs {
            syscall,
            arg0,
            arg1,
            arg2,
            arg3,
        })
    }
}

/// Setup a vector table at address 0x3fffffff000, for ABI safe calls.
///
/// 0x3fffffff000 is chosen because its the last page before userspace starts.
pub unsafe fn setup_abi_page(
    memory_mapper: &mut MemoryMapper,
    kernel_syscall_handler: fn(SyscallArgs) -> u64,
) -> Result<(), NewMappingError> {
    SYSCALL_HANDLER.write(kernel_syscall_handler);

    let abi_page = VirtualPage::new(VirtualAddress::from_l4_index(8), PageSize::Size4Kib).prev();
    let mut creation_flags = PageTableEntryFlags::default();
    creation_flags.set_writable(true);
    creation_flags.set_present(true);

    memory_mapper
        .new_map(creation_flags, creation_flags, abi_page)?
        .flush();

    let fn_ptr = abi_page.addr().as_mut_ptr::<extern "C" fn(
        syscall: u64,
        arg0: u64,
        arg1: u64,
        arg2: u64,
        arg3: u64,
    ) -> u64>();

    *fn_ptr = syscall_handler;

    let mut final_flags = PageTableEntryFlags::default();
    final_flags.set_writable(false);
    final_flags.set_present(true);

    memory_mapper
        .set_flags(abi_page.addr(), final_flags)
        .expect("the newly mapped page should be mapped")
        .flush();

    Ok(())
}
