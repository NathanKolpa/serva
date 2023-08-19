//! Basic utilities that the `std` crate normally provides but can't be used because the `#![no_std]` attribute.

pub use expected::Expected;

pub mod address;
pub mod display;
mod expected;
mod fixed_vec;
pub mod sync;
mod singleton;

pub use fixed_vec::FixedVec;
