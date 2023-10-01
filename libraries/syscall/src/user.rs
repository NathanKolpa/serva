//! Typed wrappers around syscalls.

use core::arch::asm;

use crate::{decode_syscall_result, SyscallResult};

type KernelSyscall = extern "C" fn(syscall: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64;

/// # Safety
///
/// This function is unsafe because arguments can be interpreted as pointers.
/// The caller must ensure that the rust borrow checker rule's are respected on order to guarantee safety.
pub unsafe fn syscall(syscall: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> SyscallResult {
    let segment: u16;
    asm!("mov {0:x}, cs", out(reg) segment, options(nomem, nostack, preserves_flags));

    let raw_result: u64;

    // Because the limitations of the x86_64 we can't use the `syscall` instruction while in the kernel privilege level.
    // Therefore we check if the last 2 bits of CS indicate a user privilege level.
    if (segment & 0b11) != 0 {
        asm!(
            "mov rax, 1",
            "mov rdi, 2",
            "mov rsi, 3",
            "mov rdx, 4",
            "mov r10, 5",
            "syscall",
            in("rax") syscall,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            in("r10") arg3,
        );

        asm!("", out("rax") raw_result, options(nomem, nostack, preserves_flags));
    } else {
        let kernel_syscall_location = 0x3fffffff000 as *const KernelSyscall;
        raw_result = (*kernel_syscall_location)(syscall, arg0, arg1, arg2, arg3);
    }

    decode_syscall_result(raw_result)
}

pub fn thread_exit() -> ! {
    todo!()
}

pub fn hello() {
    unsafe {
        let _ = syscall(0, 0, 0, 0, 0);
    }
}
