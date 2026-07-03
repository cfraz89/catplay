mod chacha;
pub use chacha::*;
mod aes;
pub use aes::*;

#[derive(Debug, Clone, Copy, Default)]
pub enum AirPlayStreamEncryption {
    #[default]
    Unconfigured,
    Aes {
        key: [u8; 16],
        iv: [u8; 16],
    },
    ChaCha {
        shared_secret: [u8; 32],
    },
    None,
}
