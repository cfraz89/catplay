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
 * This module provides a fallback CPU capabilities initialization mechanism.
 * It is used when no specific CPU feature detection is available.
 */

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use core::sync::atomic::{AtomicBool, Ordering};

/// Global atomic flag indicating whether the fallback CPU capabilities
/// initialization has been performed.
///
/// This is used to ensure that initialization is only performed once,
/// even in the presence of concurrent calls.
pub static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initializes the fallback CPU capabilities.
///
/// This function sets the `INITIALIZED` flag to `true` if it has not
/// already been set. If initialization has already occurred, the function
/// returns immediately. The use of atomic operations ensures thread safety.
///
/// # Example
/// ```
/// use fast_chacha::cpucaps::fallback::init;
/// init();
/// ```
#[inline(never)]
pub fn init() {
    // Atomically check if initialization has already occurred.
    if INITIALIZED.swap(true, Ordering::Relaxed) {
        // If already initialized, return immediately.
        return;
    }

    // Mark initialization as complete with Release ordering to ensure
    // all previous writes are visible to other threads.
    INITIALIZED.store(true, Ordering::Release);
}
