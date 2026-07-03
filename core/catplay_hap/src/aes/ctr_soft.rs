use aes::Aes128;
use aes::cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr128BE;

pub struct Aes128CtrSoft {
    inner: Ctr128BE<Aes128>,
}

impl Aes128CtrSoft {
    pub fn new(key: &[u8; 16], iv: &[u8; 16]) -> Self {
        Self {
            inner: Ctr128BE::<Aes128>::new_from_slices(key, iv).expect("valid AES-128-CTR key/iv"),
        }
    }

    pub fn apply_keystream(&mut self, data: &mut [u8]) {
        self.inner.apply_keystream(data);
    }
}

#[cfg(test)]
mod tests {
    use super::Aes128CtrSoft;

    #[test]
    fn ctr_matches_nist_vector() {
        let key = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
        ];
        let iv = [
            0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
        ];
        let mut data = [
            0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
        ];
        let expected = [
            0x87, 0x4d, 0x61, 0x91, 0xb6, 0x20, 0xe3, 0x26, 0x1b, 0xef, 0x68, 0x64, 0x99, 0x0d, 0xb6, 0xce,
        ];

        let mut cipher = Aes128CtrSoft::new(&key, &iv);
        cipher.apply_keystream(&mut data);

        assert_eq!(data, expected);
    }

    #[test]
    fn ctr_encrypt_decrypt_roundtrip_changes_intermediate() {
        let key = [0x11u8; 16];
        let iv = [0x22u8; 16];
        let plaintext = *b"catplay-ctr-roundtrip-demo";

        let mut encrypted = plaintext;
        let mut enc = Aes128CtrSoft::new(&key, &iv);
        enc.apply_keystream(&mut encrypted);

        assert_ne!(encrypted, plaintext);

        let mut decrypted = encrypted;
        let mut dec = Aes128CtrSoft::new(&key, &iv);
        dec.apply_keystream(&mut decrypted);

        assert_eq!(decrypted, plaintext);
    }
}
