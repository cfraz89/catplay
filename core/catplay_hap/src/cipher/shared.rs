#[derive(Default, Clone, Copy)]
pub struct HomeKitChaChaNonce(pub u64);

#[allow(unused)]
pub type HomeKitChaChaKey = [u8; 32];

impl HomeKitChaChaNonce {
    pub fn advance(&mut self) -> Result<Self, HomeKitCipherError> {
        let prev = self.0;
        self.0 += 1;
        Ok(Self(prev))
    }
}

impl From<HomeKitChaChaNonce> for u64 {
    fn from(value: HomeKitChaChaNonce) -> Self {
        value.0
    }
}

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub enum HomeKitCipherError {
    #[error("unspecified")]
    Unspecified,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("crypto payload too small")]
    PayloadTooSmall,
    #[error("unexpected decrypted length")]
    UnexpectedDecryptedLength,
    #[error("AES-128-CBC requires data length divisible by 16")]
    AesCbcBadBlock,
}
