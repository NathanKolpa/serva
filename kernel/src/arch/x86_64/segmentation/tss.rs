use core::mem::size_of;

use crate::util::address::VirtualAddress;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptStackRef {
    addr: VirtualAddress,
}

impl InterruptStackRef {
    pub const fn new() -> Self {
        Self {
            addr: VirtualAddress::new(0),
        }
    }

    pub fn from_stack(stack: &'static mut [u8]) -> Self {
        let start = stack.as_ptr() as u64;
        let end = start + stack.len() as u64;

        Self {
            addr: VirtualAddress::new(end),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    _reserved_1: u32,
    privilege_stack_table: [VirtualAddress; 3],
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
            privilege_stack_table: [VirtualAddress::new(0); 3],
            _reserved_2: 0,
            interrupt_stack_table: [InterruptStackRef::new(); 7],
            _reserved_3: 0,
            _reserved_4: 0,
            io_map_base: size_of::<Self>() as u16,
        }
    }
}