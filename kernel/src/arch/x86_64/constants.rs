use crate::arch::x86_64::interrupts::InterruptDescriptorTable;

const START: usize = InterruptDescriptorTable::STANDARD_INTERRUPTS_COUNT;

pub const TICK_INTERRUPT_INDEX: usize = START;
pub const MIN_STACK_SIZE: usize = if cfg!(debug_assertions) {
    1024 * 16
} else {
    1024 * 4
};
