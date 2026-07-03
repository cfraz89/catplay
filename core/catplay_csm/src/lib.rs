#![cfg_attr(not(test), no_std)]

pub mod decoder;
pub mod files;
#[cfg(feature = "alloc")]
pub mod msg;
