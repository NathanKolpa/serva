#![no_std]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod arch;
pub mod debug;
pub mod devices;
pub mod init;
pub mod memory;
pub mod testing;
pub mod util;
