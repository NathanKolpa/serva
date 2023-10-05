//! Low level abstraction around the x86_64 architecture.

#![feature(naked_functions)]
#![feature(doc_cfg)]
#![no_std]
#![feature(abi_x86_interrupt)]

pub mod constants;
pub mod devices;
pub mod instructions;
pub mod interrupts;
pub mod paging;
pub mod port;
mod privilege;
mod rflags;
pub mod segmentation;
pub mod syscalls;
pub mod tables;

pub use privilege::PrivilegeLevel;
pub use rflags::RFlags;

pub const ARCH_NAME: &str = "x86_64";
