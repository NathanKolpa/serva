use core::mem::size_of;

use crate::util::address::VirtualAddress;

/// An abstraction around stacks for interrupts.
///
/// # Safety
/// Note how its safe to reference the same stack multiple times, this is due to the way interrupts treat stacks.
/// Hence, clone is also implemented around this "mutable reference wrapper".
///
/// The user can also reference null stacks, this would crash the kernel but is still considered safe since no ub is caused.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptStackRef {
    addr: VirtualAddress,
}

impl InterruptStackRef {
    pub const fn null() -> Self {
        Self {
            addr: VirtualAddress::new(0),
        }
    }

    pub fn from_slice(stack: &'static mut [u8]) -> Self {
        let start = VirtualAddress::from(stack.as_ptr());
        let end = start + stack.len();

        Self { addr: end }
    }

    pub fn stack_end(&self) -> VirtualAddress {
        self.addr
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    _reserved_1: u32,
    pub privilege_stack_table: [InterruptStackRef; 3],
    _reserved_2: u64,
    pub interrupt_stack_table: [InterruptStackRef; 7],
    _reserved_3: u64,
    _reserved_4: u16,
    io_map_base: u16,
}

impl TaskStateSegment {
    pub const fn new() -> Self {
        TaskStateSegment {
            _reserved_1: 0,
            privilege_stack_table: [InterruptStackRef::null(); 3],
            _reserved_2: 0,
            interrupt_stack_table: [InterruptStackRef::null(); 7],
            _reserved_3: 0,
            _reserved_4: 0,
            io_map_base: size_of::<Self>() as u16,
        }
    }
}
