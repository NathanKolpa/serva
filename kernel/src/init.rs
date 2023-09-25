use core::panic::PanicInfo;

use bootloader::BootInfo;

use crate::arch::x86_64::constants::MIN_STACK_SIZE;
use crate::arch::x86_64::init::GDT;
use crate::arch::x86_64::interrupts::atomic_block;
use crate::arch::x86_64::paging::PhysicalPage;
use crate::arch::x86_64::syscalls::init_syscalls;
use crate::arch::x86_64::ARCH_NAME;
use crate::arch::x86_64::{halt_loop, init_x86_64};
use crate::debug::DEBUG_CHANNEL;
use crate::interface::interrupts::INTERRUPT_HANDLERS;
use crate::interface::syscalls::handle_user_syscall_raw;
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
    use alloc::borrow::Cow;
    use alloc::boxed::Box;
    use alloc::ffi::CString;
    use alloc::vec::Vec;

    use crate::arch::x86_64::halt_loop;
    use crate::arch::x86_64::syscalls::SyscallArgs;
    use crate::interface::syscalls::{handle_kernel_syscall, SyscallResult};
    use crate::service::{
        EndpointParameter, NewEndpoint, NewIntent, Privilege, ServiceEntrypoint, SERVICE_TABLE,
    };
    use crate::util::address::VirtualAddress;
    use crate::util::collections::FixedVec;

    pub fn setup_test_service() {
        let entry = ServiceEntrypoint::MappedFunction(VirtualAddress::from(
            test_service_start as *const (),
        ));
        let dep_entry = ServiceEntrypoint::MappedFunction(VirtualAddress::from(
            test_dep_service_start as *const (),
        ));

        let _test = Box::new(1);

        let _dep_spec = unsafe {
            let intents = [];

            let mut request = FixedVec::new();
            request.push(EndpointParameter::SizedBuffer(50));
            request.push(EndpointParameter::SizedBuffer(50));

            let mut endpoints = Vec::new();
            endpoints.push(NewEndpoint {
                name: Cow::Borrowed("echo"),
                request,
                response: FixedVec::new(),
                min_privilege: Privilege::User,
            });

            SERVICE_TABLE
                .register_spec(
                    Cow::Borrowed("Test Dependency"),
                    Privilege::Kernel,
                    false,
                    dep_entry,
                    intents,
                    endpoints,
                )
                .unwrap()
        };

        let spec = unsafe {
            let mut intents = Vec::new();
            intents.push(NewIntent {
                spec_name: Cow::Borrowed("Test Dependency"),
                endpoint_name: Cow::Borrowed("echo"),
                required: true,
            });

            let endpoints = [];

            SERVICE_TABLE
                .register_spec(
                    Cow::Borrowed("test"),
                    Privilege::Kernel,
                    false,
                    entry,
                    intents,
                    endpoints,
                )
                .unwrap()
        };

        // een endpoint request schrijft gewoon een ruwe blokken data.
        // de kernel validate deze data losjes.
        // de request handler, leest deze parameter voor parameter uit.

        // een request forwad ziet er zo uit:
        // - een endpoint met een unsized buffer.
        // client schrijft de data zoals hij normaal naar de target schrijft.
        // de proxy dumpt deze data naar de target.
        // de target kan gewoon als normaal uitlezen.

        debug_println!("First!");
        SERVICE_TABLE.start_service(spec.id()).unwrap();
    }

    fn syscall(args: SyscallArgs) -> SyscallResult {
        handle_kernel_syscall(&args)
    }

    fn test_service_start() -> ! {
        debug_println!("Connecting");

        let service_name = CString::new("Test Dependency").unwrap();

        let connection = syscall(SyscallArgs {
            syscall: 1,
            arg0: service_name.as_ptr() as u64,
            arg1: 0,
            arg2: 0,
            arg3: 0,
        })
        .unwrap();

        debug_println!("Connection Handle {}", connection);

        let endpoint_name = CString::new("echo").unwrap();

        debug_println!("Requesting to {endpoint_name:?}");

        syscall(SyscallArgs {
            syscall: 2,
            arg0: connection,
            arg1: endpoint_name.as_ptr() as u64,
            arg2: 0,
            arg3: 0,
        })
        .unwrap();

        debug_println!("Request open");

        let buffer = [1u8; 10];

        for _ in 0..10 {
            debug_println!("Writing {} bytes", buffer.len());

            syscall(SyscallArgs {
                syscall: 3,
                arg0: connection,
                arg1: buffer.as_ptr() as u64,
                arg2: buffer.len() as u64,
                arg3: 0,
            })
            .unwrap();
        }

        debug_println!("Finishing request");

        syscall(SyscallArgs {
            syscall: 3,
            arg0: connection,
            arg1: buffer.as_ptr() as u64,
            arg2: 0,
            arg3: 1, // end flag
        })
        .unwrap();

        debug_println!("Request finished");

        halt_loop()
    }

    fn test_dep_service_start() -> ! {
        loop {}
    }
}
