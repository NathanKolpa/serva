//! A library wrapper for the Serva kernel.

#![no_std]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![feature(doc_cfg)]
#![cfg_attr(test, no_main)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod arch;
pub mod debug;
pub mod init;
pub mod memory;
pub mod multi_tasking;
pub mod testing;
pub mod util;
pub mod interrupts;
