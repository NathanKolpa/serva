//! Implementation of x86_64 concepts
pub use init::init_x86_64;
pub use instructions::*;
pub use privilege::PrivilegeLevel;

pub mod init;
mod instructions;
pub mod interrupts;
pub mod paging;
pub mod port;
mod privilege;
pub mod segmentation;
pub mod tables;
pub mod trampoline;

pub const ARCH_NAME: &str = "x86_64";
