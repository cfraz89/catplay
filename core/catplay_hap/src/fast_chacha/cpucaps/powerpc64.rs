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
 * Module: cpucaps::powerpc64
 *
 * This module provides CPU capability detection and initialization
 * for PowerPC64 architectures. It defines global variables and
 * an initialization function to manage CPU feature flags.
 */

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Global variable holding PowerPC64 CPU capability flags.
/// This variable is used by OpenSSL and related cryptographic code
/// to determine available CPU features at runtime.
///
/// # Safety
/// This variable is mutable and not thread-safe by itself.
/// Access must be synchronized or performed during initialization.
#[no_mangle]
pub static mut OPENSSL_ppccap_P: u32 = 0;

/// Atomic flag indicating whether CPU capability detection has been performed.
/// Used to ensure that initialization is only performed once.
pub static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initializes CPU capability flags for PowerPC64.
///
/// This function ensures that initialization is performed only once,
/// even in the presence of concurrent calls. It sets the `INITIALIZED`
/// flag atomically and is intended to be called at program startup.
///
/// # Safety
/// This function is unsafe because it may modify global state and
/// is expected to be called in a single-threaded context or with
/// appropriate synchronization.
#[inline(never)]
pub fn init() {
    // If already initialized, return immediately.
    if INITIALIZED.swap(true, Ordering::Relaxed) {
        return;
    }

    // Mark as initialized with Release ordering to synchronize with other threads.
    INITIALIZED.store(true, Ordering::Release);
}
