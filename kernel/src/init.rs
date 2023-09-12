use core::panic::PanicInfo;

use bootloader::BootInfo;

use crate::arch::x86_64::constants::MIN_STACK_SIZE;
use crate::arch::x86_64::init::GDT;
use crate::arch::x86_64::interrupts::{atomic_block, enable_interrupts};
use crate::arch::x86_64::paging::PhysicalPage;
use crate::arch::x86_64::syscalls::init_syscalls;
use crate::arch::x86_64::ARCH_NAME;
use crate::arch::x86_64::{halt_loop, init_x86_64};
use crate::debug::DEBUG_CHANNEL;
use crate::interface::interrupts::INTERRUPT_HANDLERS;
use crate::interface::syscalls::{handle_user_syscall, handle_user_syscall_raw};
use crate::memory::heap::{map_heap, HEAP_SIZE};
use crate::memory::{MemoryMapper, FRAME_ALLOCATOR};
use crate::multi_tasking::scheduler::{Thread, ThreadStack, SCHEDULER};
use crate::service::SERVICE_TABLE;
use crate::util::address::VirtualAddress;
use crate::util::display::ReadableSize;
use crate::util::sync::{PanicOnce, SpinMutex};

/// The kernel panic handler.
pub fn handle_panic(info: &PanicInfo) -> ! {
    debug_println!("{info}");
    halt_loop()
}

static ROOT_MAPPER: SpinMutex<PanicOnce<MemoryMapper>> = SpinMutex::new(PanicOnce::new());

/// The kernel entry point.
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    init_x86_64(INTERRUPT_HANDLERS);

    let memory_mapper = unsafe {
        FRAME_ALLOCATOR.init(&boot_info.memory_map);
        MemoryMapper::new(
            &FRAME_ALLOCATOR,
            PhysicalPage::active().0,
            boot_info.physical_memory_offset,
        )
    };

    ROOT_MAPPER.lock().initialize_with(memory_mapper);

    SCHEDULER.add_thread(unsafe {
        Thread::start_new(
            Some("Kernel Main/Idle Thread"),
            ThreadStack::from_slice(&mut KERNEL_MAIN_STACK),
            VirtualAddress::from(main_kernel_thread as *const fn()),
            None,
        )
    });

    SCHEDULER.yield_current();
    unreachable!()
}

static mut KERNEL_MAIN_STACK: [u8; MIN_STACK_SIZE] = [0; MIN_STACK_SIZE];

/// The main kernel thread.
///
/// Execution is given to this code asap in the init process.
/// This is preferred because `multi_tasking::sync` primitives are allowed only from a scheduled thread.
/// This thread also acts as the idle task.
fn main_kernel_thread() -> ! {
    let mut mapper = ROOT_MAPPER.lock();

    atomic_block(|| {
        debug_println!("Starting the Serva Operating System...");
        debug_println!("Architecture: {}", ARCH_NAME);
        debug_println!("Debug channel: {}", DEBUG_CHANNEL);

        debug_println!(
            "Initializing kernel heap with {} of memory",
            ReadableSize::new(HEAP_SIZE)
        );

        map_heap(&mut mapper).expect("Failed to map the kernel heap");

        debug_println!("{:#?}", FRAME_ALLOCATOR.info());

        // from this point on the kernel map is shared and cannot be changed.
        SERVICE_TABLE.set_root_memory_map(
            mapper
                .borrow_to_new_mapper(true)
                .expect("Failed to inherit root memory map"),
        );

        unsafe {
            init_syscalls(handle_user_syscall_raw, GDT.syscall, GDT.sysret);
        }

        test_service::setup_test_service();
    });

    SCHEDULER.yield_current();

    // make sure there is always nothing to do.
    halt_loop()
}

mod test_service {
    use crate::arch::x86_64::constants::MIN_STACK_SIZE;
    use crate::arch::x86_64::syscalls::SyscallArgs;
    use crate::interface::syscalls::{handle_kernel_syscall, SyscallResult};
    use crate::multi_tasking::scheduler::{Thread, ThreadStack, SCHEDULER};
    use crate::service::{Privilege, ServiceEntrypoint, SERVICE_TABLE};
    use crate::util::address::VirtualAddress;
    use alloc::borrow::Cow;
    use crate::arch::x86_64::halt;

    pub fn setup_test_service() {
        let entry = ServiceEntrypoint::MappedFunction(VirtualAddress::from(
            test_service_start as *const (),
        ));
        let dep_entry = ServiceEntrypoint::MappedFunction(VirtualAddress::from(
            test_dep_service_start as *const (),
        ));

        let dep_spec = unsafe {
            let intents = [];
            let endpoints = [];

            SERVICE_TABLE.register_spec(
                Cow::Borrowed("Test Dependency"),
                Privilege::Kernel,
                dep_entry,
                intents,
                endpoints,
            )
        };

        let spec = unsafe {
            let intents = [];
            let endpoints = [];

            SERVICE_TABLE.register_spec(
                Cow::Borrowed("test"),
                Privilege::Kernel,
                entry,
                intents,
                endpoints,
            )
        };

        debug_println!("First!");
        SERVICE_TABLE.start_service(spec.id()).unwrap();
        debug_println!("Second!");
        SERVICE_TABLE.start_service(dep_spec.id()).unwrap();
    }

    fn syscall(args: SyscallArgs) -> SyscallResult {
        handle_kernel_syscall(&args)
    }

    fn test_service_start() -> ! {
        let mut nonce = 0;
        loop {
            nonce += 1;
            syscall(SyscallArgs {
                syscall: 0,
                arg0: nonce,
                arg1: 1,
                arg2: 2,
                arg3: 3,
            })
            .unwrap();
            halt();
        }
    }

    fn test_dep_service_start() -> ! {
        let mut nonce = 0;
        loop {
            nonce += 10;
            debug_println!("Hello I am a dependency! {}", nonce);
            halt();
            // syscall(SyscallArgs {
            //     syscall: 0,
            //     arg0: 0,
            //     arg1: 1,
            //     arg2: 2,
            //     arg3: 3,
            // })
            // .unwrap();
        }
    }
}
