#[cfg(feature = "alloc")]
mod alloc_bytes;
#[cfg(feature = "alloc")]
mod alloc_string;
#[cfg(feature = "alloc")]
mod alloc_vec;

mod data;
mod packet;
mod prim_decode;
mod prim_encode;
mod serialize;
mod structs;

#[cfg(feature = "alloc")]
pub use alloc_bytes::*;
#[cfg(feature = "alloc")]
pub use alloc_string::*;
#[cfg(feature = "alloc")]
pub use alloc_vec::*;
pub use data::*;
pub use packet::*;
pub use serialize::*;
pub use structs::*;
