//! Low level abstraction around the x86_64 architecture.
pub use init::init_x86_64;
pub use instructions::*;
pub use privilege::PrivilegeLevel;
pub use rflags::*;

pub mod constants;
pub mod devices;
pub mod init;
mod instructions;
pub mod interrupts;
pub mod paging;
pub mod port;
mod privilege;
mod rflags;
pub mod segmentation;
pub mod syscalls;
pub mod tables;

pub const ARCH_NAME: &str = "x86_64";
