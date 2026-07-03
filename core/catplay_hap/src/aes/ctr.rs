use super::Aes128CtrBackend;

/// Thin crypto wrapper so we can swap implementations per-arch later.
pub struct Aes128Ctr {
    inner: Aes128CtrBackend,
}

impl Aes128Ctr {
    pub fn new(key: &[u8; 16], iv: &[u8; 16]) -> Self {
        Self {
            inner: Aes128CtrBackend::new(key, iv),
        }
    }

    pub fn apply_keystream(&mut self, data: &mut [u8]) {
        self.inner.apply_keystream(data);
    }
}
