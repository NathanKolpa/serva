# Kernel

## Current Status

todo:

- Document x86_64 module so that people new to os development can learn on how the archetecture works.
- A scheduler.
- Loading elf programs.
- Service abstraction.
- ipc.

### Quirks

- Using opt level `0` (for debugging) breaks multithreading. I don't know if this is my fault because im implicitly
  relying on some compiler optimization. Or because the compiler is being dishonest about optimization. I'm fairly
  certain this is a compiler issue, because I have tried to run [EuraliOS](https://github.com/bendudson/EuraliOS) in
  debug mode, which crashes.
  And [this is what redox does](https://gitlab.redox-os.org/redox-os/kernel/-/blob/master/Cargo.toml#L71).

### Boot process

Currently, the boot process is handled by the [`bootloader`](https://github.com/rust-osdev/bootloader) crate.
There is a work in progress version on the **custom_boot** branch
However there are several problems:

- Booting using qemu's `-kernel` flag is preferred, but long mode is not possible (#e3fa94e).
- Booting by creating a .iso file breaks the entire system (#92cb835)
- Both options completely break the testing setup.