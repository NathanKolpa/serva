//! Basic utilities that the `std` crate normally provides but can't be used due to the `#![no_std]` attribute.

pub use expected::Expected;
pub use init_guard::InitializeGuard;
pub use singleton::Singleton;

pub mod address;
pub mod collections;
pub mod display;
mod expected;
mod init_guard;
mod singleton;
pub mod sync;
