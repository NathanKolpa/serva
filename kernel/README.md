# Kernel

## Current Status

Must have:

- Loading elf programs.
- Reimplement the frame allocator with the buddy allocator algorithm.
- Implement drop for MemoryMapper, which deallocates all owned tables.

Should have:

- Document x86_64 module so that people new to os development can learn on how the architecture works.
- Refactor the scheduler and service table memory:
  - Implement a slab, and eternal allocator.
  - Handle low memory situations.
  - Optimize the scheduler for heap usage.
  - Optimize the service table for heap usage.
  - Move the buffer for a pipe to a shared map.
- After implementing multiprocessing:
  - Optimize service table for concurrency.
  - Optimize the scheduler for concurrency.

### Boot process

Currently, the boot process is handled by the [`bootloader`](https://github.com/rust-osdev/bootloader) crate.
There is a work in progress version on the **custom_boot** branch.
However, there are several problems:

- Booting using qemu's `-kernel` flag is preferred, but long mode is not possible (#e3fa94e).
- Booting by creating a .iso file breaks the entire system (#92cb835)
- Both options completely break the testing setup.