use core::ffi::c_void;

#[cfg(all(fast_chacha_asm, target_os = "linux", target_arch = "aarch64"))]
use crate::fast_chacha::cpucaps::OPENSSL_armcap_P;
#[cfg(all(fast_chacha_asm, target_os = "linux", target_arch = "x86_64"))]
use crate::fast_chacha::cpucaps::OPENSSL_ia32cap_P;
use crate::fast_chacha::init_cpu_caps;

const CHACHA20_KEY_WORDS: usize = 8;
const CHACHA20_NONCE_LEN: usize = 12;
const POLY1305_TAG_LEN: usize = 16;

#[derive(Clone)]
pub struct FastChaCha20Poly1305 {
    key_words: [u32; CHACHA20_KEY_WORDS],
}

impl FastChaCha20Poly1305 {
    pub fn new(key: &[u8; 32]) -> Self {
        init_cpu_caps();
        Self {
            key_words: [
                u32::from_le_bytes([key[0], key[1], key[2], key[3]]),
                u32::from_le_bytes([key[4], key[5], key[6], key[7]]),
                u32::from_le_bytes([key[8], key[9], key[10], key[11]]),
                u32::from_le_bytes([key[12], key[13], key[14], key[15]]),
                u32::from_le_bytes([key[16], key[17], key[18], key[19]]),
                u32::from_le_bytes([key[20], key[21], key[22], key[23]]),
                u32::from_le_bytes([key[24], key[25], key[26], key[27]]),
                u32::from_le_bytes([key[28], key[29], key[30], key[31]]),
            ],
        }
    }

    pub fn is_available() -> bool {
        init_cpu_caps();
        asm_integrated_available()
    }

    pub fn seal_in_place(&self, nonce: &[u8; 12], aad: &[u8], in_out: &mut [u8]) -> Option<[u8; 16]> {
        if !asm_integrated_available() {
            return None;
        }

        let mut data = InOut {
            input: SealDataIn {
                key: self.key_words,
                counter: 0,
                nonce: *nonce,
                extra_ciphertext: core::ptr::null(),
                extra_ciphertext_len: 0,
            },
        };

        unsafe {
            call_chacha20_poly1305_seal(in_out.as_mut_ptr(), in_out.as_ptr(), in_out.len(), aad.as_ptr(), aad.len(), &mut data);
            Some(data.out.tag)
        }
    }

    pub fn open_in_place(&self, nonce: &[u8; 12], aad: &[u8], in_out: &mut [u8]) -> Option<[u8; 16]> {
        if !asm_integrated_available() {
            return None;
        }

        let mut data = InOut {
            input: OpenDataIn {
                key: self.key_words,
                counter: 0,
                nonce: *nonce,
            },
        };

        unsafe {
            call_chacha20_poly1305_open(in_out.as_mut_ptr(), in_out.as_ptr(), in_out.len(), aad.as_ptr(), aad.len(), &mut data);
            Some(data.out.tag)
        }
    }
}

#[repr(align(16), C)]
#[derive(Clone, Copy)]
struct SealDataIn {
    key: [u32; CHACHA20_KEY_WORDS],
    counter: u32,
    nonce: [u8; CHACHA20_NONCE_LEN],
    extra_ciphertext: *const u8,
    extra_ciphertext_len: usize,
}

#[repr(align(16), C)]
#[derive(Clone, Copy)]
struct OpenDataIn {
    key: [u32; CHACHA20_KEY_WORDS],
    counter: u32,
    nonce: [u8; CHACHA20_NONCE_LEN],
}

#[repr(C)]
union InOut<T>
where
    T: Copy,
{
    input: T,
    out: Out,
}

#[derive(Clone, Copy)]
#[repr(align(16), C)]
struct Out {
    tag: [u8; POLY1305_TAG_LEN],
}

#[cfg(all(fast_chacha_asm, target_os = "linux", target_arch = "aarch64"))]
fn asm_integrated_available() -> bool {
    unsafe { (OPENSSL_armcap_P & (1 << 0)) != 0 }
}

#[cfg(all(fast_chacha_asm, target_os = "linux", target_arch = "x86_64"))]
fn asm_integrated_available() -> bool {
    unsafe { (OPENSSL_ia32cap_P[1] & (1 << 19)) != 0 }
}

#[cfg(not(all(fast_chacha_asm, target_os = "linux", any(target_arch = "aarch64", target_arch = "x86_64"))))]
fn asm_integrated_available() -> bool {
    false
}

#[cfg(all(fast_chacha_asm, target_os = "linux", target_arch = "aarch64"))]
unsafe fn call_chacha20_poly1305_seal(
    output: *mut u8,
    input: *const u8,
    len: usize,
    ad: *const u8,
    ad_len: usize,
    data: *mut InOut<SealDataIn>,
) {
    unsafe { chacha20_poly1305_seal(input, output, len, ad, ad_len, data.cast::<c_void>()) }
}

#[cfg(all(fast_chacha_asm, target_os = "linux", target_arch = "aarch64"))]
unsafe fn call_chacha20_poly1305_open(
    output: *mut u8,
    input: *const u8,
    len: usize,
    ad: *const u8,
    ad_len: usize,
    data: *mut InOut<OpenDataIn>,
) {
    unsafe { chacha20_poly1305_open(output, input, len, ad, ad_len, data.cast::<c_void>()) }
}

#[cfg(all(fast_chacha_asm, target_os = "linux", target_arch = "x86_64"))]
unsafe fn call_chacha20_poly1305_seal(
    output: *mut u8,
    input: *const u8,
    len: usize,
    ad: *const u8,
    ad_len: usize,
    data: *mut InOut<SealDataIn>,
) {
    let avx2 = unsafe { (OPENSSL_ia32cap_P[2] & (1 << 5)) != 0 };
    let bmi2 = unsafe { (OPENSSL_ia32cap_P[2] & (1 << 8)) != 0 };

    unsafe {
        if avx2 && bmi2 {
            chacha20_poly1305_seal_avx2(input, output, len, ad, ad_len, data.cast::<c_void>());
        } else {
            chacha20_poly1305_seal_sse41(input, output, len, ad, ad_len, data.cast::<c_void>());
        }
    }
}

#[cfg(all(fast_chacha_asm, target_os = "linux", target_arch = "x86_64"))]
unsafe fn call_chacha20_poly1305_open(
    output: *mut u8,
    input: *const u8,
    len: usize,
    ad: *const u8,
    ad_len: usize,
    data: *mut InOut<OpenDataIn>,
) {
    let avx2 = unsafe { (OPENSSL_ia32cap_P[2] & (1 << 5)) != 0 };
    let bmi2 = unsafe { (OPENSSL_ia32cap_P[2] & (1 << 8)) != 0 };

    unsafe {
        if avx2 && bmi2 {
            chacha20_poly1305_open_avx2(output, input, len, ad, ad_len, data.cast::<c_void>());
        } else {
            chacha20_poly1305_open_sse41(output, input, len, ad, ad_len, data.cast::<c_void>());
        }
    }
}

#[cfg(all(fast_chacha_asm, target_os = "linux", target_arch = "aarch64"))]
unsafe extern "C" {
    fn chacha20_poly1305_seal(
        pt: *const u8,
        ct: *mut u8,
        len_in: usize,
        ad: *const u8,
        len_ad: usize,
        seal_data: *mut c_void,
    );

    fn chacha20_poly1305_open(
        pt: *mut u8,
        ct: *const u8,
        len_in: usize,
        ad: *const u8,
        len_ad: usize,
        aead_data: *mut c_void,
    );
}

#[cfg(all(fast_chacha_asm, target_os = "linux", target_arch = "x86_64"))]
unsafe extern "C" {
    fn chacha20_poly1305_seal_sse41(
        pt: *const u8,
        ct: *mut u8,
        len_in: usize,
        ad: *const u8,
        len_ad: usize,
        seal_data: *mut c_void,
    );

    fn chacha20_poly1305_open_sse41(
        pt: *mut u8,
        ct: *const u8,
        len_in: usize,
        ad: *const u8,
        len_ad: usize,
        aead_data: *mut c_void,
    );

    fn chacha20_poly1305_seal_avx2(
        pt: *const u8,
        ct: *mut u8,
        len_in: usize,
        ad: *const u8,
        len_ad: usize,
        seal_data: *mut c_void,
    );

    fn chacha20_poly1305_open_avx2(
        pt: *mut u8,
        ct: *const u8,
        len_in: usize,
        ad: *const u8,
        len_ad: usize,
        aead_data: *mut c_void,
    );
}
