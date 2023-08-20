use crate::arch::x86_64::interrupts::InterruptDescriptorTable;

const START: usize = InterruptDescriptorTable::STANDARD_INTERRUPTS_COUNT;

pub const TICK_INTERRUPT_INDEX: usize = START;
