# Kernel

## Current Status

todo:
- Document x86_64 module so that people new to os development can learn on how the archetecture works.
- A scheduler.
- Loading elf programs.
- Service abstraction.
- ipc.

### Boot process

Currently, the boot process is handled by the [`bootloader`](https://github.com/rust-osdev/bootloader) crate.
There is a work in progress version on the **custom_boot** branch
However there are several problems:

- Booting using qemu's `-kernel` flag is preferred, but long mode is not possible (#e3fa94e).
- Booting by creating a .iso file breaks the entire system (#92cb835)
- Both options completely break the testing setup.