//! Basic utilities that the `std` crate normally provides but can't be used due to the `#![no_std]` attribute.

#![feature(doc_cfg)]
#![no_std]

pub mod address;
pub mod collections;
pub mod display;
pub mod sync;
