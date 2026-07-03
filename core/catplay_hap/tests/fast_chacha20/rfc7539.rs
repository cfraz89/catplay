/*
 * Copyright 2025 sh0rch <sh0rch@iwl.dev>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/*!
 * # RFC 7539 Test Vectors for fast_chacha
 *
 * This module contains tests for the `fast_chacha` crate, verifying
 * compatibility with the RFC 7539 ChaCha20 test vectors.
 *
 * The test checks both the fast (assembly-optimized) and fallback (pure Rust)
 * implementations of ChaCha20 for correctness.
 */

use catplay_hap::fast_chacha::FastChaCha20;

/// Tests the first ChaCha20 keystream block against the RFC 7539 test vector.
///
/// This test verifies that both the fast (assembly) and fallback (pure Rust)
/// implementations of `FastChaCha20` produce the correct keystream output
/// for the given key, nonce, and counter values.
///
/// The test is skipped if the fast (assembly) implementation is not available
/// for the current target.
///
/// # Panics
/// Panics if the produced keystream does not match the expected RFC 7539 output.
// #[test]
// fn rfc7539_keystream_block1() {
fn main() {
    // RFC 7539 test vector key (32 bytes)
    const KEY: [u8; 32] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
        0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    ];
    // RFC 7539 test vector nonce (12 bytes)
    const NONCE: [u8; 12] = [0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00];
    // Expected output for the first 64 bytes of the keystream (RFC 7539)
    const EXP: [u8; 64] = [
        0x10, 0xf1, 0xe7, 0xe4, 0xd1, 0x3b, 0x59, 0x15, 0x50, 0x0f, 0xdd, 0x1f, 0xa3, 0x20, 0x71, 0xc4, 0xc7, 0xd1, 0xf4, 0xc7, 0x33, 0xc0,
        0x68, 0x03, 0x04, 0x22, 0xaa, 0x9a, 0xc3, 0xd4, 0x6c, 0x4e, 0xd2, 0x82, 0x64, 0x46, 0x07, 0x9f, 0xaa, 0x09, 0x14, 0xc2, 0xd7, 0x05,
        0xd9, 0x8b, 0x02, 0xa2, 0xb5, 0x12, 0x9c, 0xd1, 0xde, 0x16, 0x4e, 0xb9, 0xcb, 0xd0, 0x83, 0xe8, 0xa2, 0x50, 0x3c, 0x4e,
    ];

    // Input buffer of 64 zero bytes (plaintext)
    let inp = [0u8; 64];

    // Check if the fast (assembly) implementation is available
    let use_asm = catplay_hap::fast_chacha::is_asm_available_chacha20();

    if use_asm {
        // Test the fast (assembly) implementation, if available
        let mut cipher = FastChaCha20::new(&KEY, &NONCE);

        // Set the stream position to block 1 (counter = 1, offset = 64 bytes)
        cipher.seek(64);
        let mut buf_fast = inp;

        // Apply the keystream using the fast (assembly) implementation
        cipher.apply_keystream(&mut buf_fast[..]);

        // Compare the output with the expected RFC 7539 keystream
        assert!(buf_fast == EXP, "FastChaCha20 is not compatible with RFC 7539");
    }
    // Test the fallback (pure Rust) implementation
    let mut buf_fallback = inp;
    let mut fb_cipher = FastChaCha20::new(&KEY, &NONCE);
    fb_cipher.seek(64);
    // Apply the keystream using the pure Rust implementation
    fb_cipher.apply_keystream_pure(&mut buf_fallback[..], 10);

    // Compare the output with the expected RFC 7539 keystream
    assert!(buf_fallback == EXP, "FastChaCha20(Fallback) is not compatible with RFC 7539");
}
