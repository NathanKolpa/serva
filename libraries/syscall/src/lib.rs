#![no_std]
#![feature(doc_cfg)]

mod error;
mod result;

#[doc(cfg(feature = "user"))]
#[cfg(feature = "user")]
mod user;

#[doc(cfg(feature = "user"))]
#[cfg(feature = "user")]
pub use user::*;

pub use error::*;
pub use result::*;
