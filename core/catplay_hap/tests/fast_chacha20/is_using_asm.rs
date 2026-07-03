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

//! Test module for verifying whether the FastChaCha20 cipher uses the assembly backend.
//!
//! This test ensures that the `fast_chacha` crate is correctly detecting and using
//! the optimized assembly implementation when available on the current platform.

use catplay_hap::fast_chacha::is_asm_available_chacha20;

/// Test to check if the assembly backend is being used by FastChaCha20.
///
/// This test asserts that the assembly implementation is available and being used.
/// If the assertion fails, it means the ASM implementation is not active,
/// and the test will output an error message in English.
// #[test]
// fn is_using_asm() {
fn main() {
    // Ensure the cipher is actually using the assembly backend.
    assert!(is_asm_available_chacha20(), "FastChaCha20 is not using the ASM implementation");
}
