// NOT IMPLEMENTED YET!
// This file is part of the fast_chacha crate, which provides a high-performance
// implementation of the ChaCha20 stream cipher, optimized for various CPU architectures.

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
 * RISC-V CPU capabilities detection module for fast_chacha.
 *
 * This module provides basic infrastructure for detecting and storing
 * RISC-V-specific CPU features, primarily for cryptographic optimizations.
 */

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Global variable to store RISC-V CPU capabilities bitmask.
///
/// This variable is intended to be accessed from C code as well,
/// hence the `#[no_mangle]` attribute and `pub static mut`.
#[no_mangle]
pub static mut OPENSSL_riscvcap_P: u32 = 0;

/// Atomic flag indicating whether CPU capabilities have been initialized.
///
/// Used to ensure that initialization happens only once in a thread-safe manner.
pub static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initializes RISC-V CPU capabilities detection.
///
/// This function is intended to be called once, and is thread-safe.
/// It sets the `INITIALIZED` flag to prevent redundant initialization.
/// Currently, it does not perform actual feature detection, but provides
/// the necessary structure for future extensions.
///
/// # Safety
///
/// This function is `unsafe` because it may modify global state and is
/// expected to be called in low-level initialization contexts.
#[inline(never)]
pub fn init() {
    // If already initialized, return early.
    if INITIALIZED.swap(true, Ordering::Relaxed) {
        return;
    }

    // Mark as initialized with Release ordering for memory safety.
    INITIALIZED.store(true, Ordering::Release);
}
