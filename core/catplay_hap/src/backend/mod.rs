pub mod auth_setup;
pub mod pair_setup;
pub mod pair_verify;

mod hkdf;
pub use hkdf::*;
#[cfg(feature = "openssl")]
mod srp_openssl;

mod error;
pub mod tlv;

pub use error::*;

/// `Result` type redefinition.
pub type Result<T> = core::result::Result<T, Error>;
