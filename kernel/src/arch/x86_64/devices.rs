use essentials::sync::{Singleton, SpinMutex};
use x86_64::devices::pic_8259::ChainedPic8259;
use x86_64::devices::qemu::Qemu;
use x86_64::devices::uart_16550::Uart16550;
use x86_64::interrupts::InterruptDescriptorTable;

const PIC_CHAIN_INTS_START: usize = InterruptDescriptorTable::STANDARD_INTERRUPTS_COUNT;
pub const PIC_CHAIN_TICK_INT_INDEX: usize = PIC_CHAIN_INTS_START;

pub static PIC_CHAIN: Singleton<SpinMutex<ChainedPic8259>> =
    Singleton::new(|| SpinMutex::new(unsafe { ChainedPic8259::new(PIC_CHAIN_INTS_START as u8) }));

pub static QEMU_DEVICE: SpinMutex<Qemu> = SpinMutex::new(unsafe { Qemu::new() });

pub static SERIAL: Singleton<SpinMutex<Uart16550>> =
    Singleton::new(|| SpinMutex::new(unsafe { Uart16550::new(0x3F8) }));
