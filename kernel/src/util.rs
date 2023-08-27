//! Basic utilities that the `std` crate normally provides but can't be used due to the `#![no_std]` attribute.

pub use expected::Expected;
pub use singleton::Singleton;
pub use init_guard::InitializeGuard;

pub mod address;
pub mod collections;
pub mod display;
mod expected;
mod singleton;
pub mod sync;
mod init_guard;
