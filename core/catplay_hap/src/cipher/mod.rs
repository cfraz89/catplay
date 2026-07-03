mod shared;
pub use shared::*;

mod cipher_ring;
pub use cipher_ring::*;
mod cipher_fastchacha;
pub use cipher_fastchacha::*;

#[cfg(feature = "fast_chacha")]
pub type HomeKitCipher = HomeKitCipherFast;
#[cfg(not(feature = "fast_chacha"))]
pub type HomeKitCipher = HomeKitCipherRing;
