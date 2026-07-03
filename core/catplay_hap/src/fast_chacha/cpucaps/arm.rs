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
 * ARM CPU capabilities detection module.
 *
 * This module provides detection of ARM CPU features (such as NEON, AES, SHA, PMULL, etc.)
 * at runtime, and sets the global capability variable `OPENSSL_armcap_P` accordingly.
 * This is used to enable optimized cryptographic routines depending on hardware support.
 */

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused)]

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[cfg(any(target_os = "linux", target_os = "android"))]
use super::get_auxv;

/// Global variable storing detected ARM CPU capabilities as bitflags.
/// The value is set during initialization and used by cryptographic routines.
#[unsafe(export_name = "CATPLAY_OPENSSL_armcap_P")]
pub static mut OPENSSL_armcap_P: u32 = 0;

/// Atomic flag indicating whether ARM CPU capabilities have been initialized.
pub static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Detects ARM CPU capabilities based on hardware capability bitfields.
///
/// # Arguments
/// * `hwcap` - Primary hardware capability bitfield (AT_HWCAP).
/// * `hwcap2` - Optional secondary hardware capability bitfield (AT_HWCAP2).
///
/// # Returns
/// Bitmask of detected capabilities, with each bit representing a specific feature.
///
/// # Supported Features
/// - aarch64: ASIMD, AES, PMULL, SHA1, SHA2, CPUID, SVE
/// - arm: NEON, AES, PMULL, SHA1, SHA2
#[cfg(any(target_os = "linux", target_os = "android"))]
fn detect_arm_capabilities(hwcap: usize, hwcap2: Option<usize>) -> u32 {
    let mut caps = 0u32;

    #[cfg(target_arch = "aarch64")]
    {
        // HWCAP bit definitions for aarch64
        const HWCAP_ASIMD: usize = 1 << 1;
        //const HWCAP_AES: usize = 1 << 3;
        const HWCAP_PMULL: usize = 1 << 4;
        //const HWCAP_SHA1: usize = 1 << 5;
        //const HWCAP_SHA2: usize = 1 << 6;
        const HWCAP_CPUID: usize = 1 << 11;
        //const HWCAP_SVE: usize = 1 << 22;

        // Set corresponding bits in caps if features are present
        if hwcap & HWCAP_ASIMD != 0 {
            caps |= 1 << 0; // NEON/ASIMD
        }
        if hwcap & HWCAP_PMULL != 0 {
            caps |= 1 << 5; // PMULL
        }
        if hwcap & HWCAP_CPUID != 0 {
            caps |= 1 << 7; // CPUID
        }
        //if hwcap & HWCAP_SVE != 0 {
        //    caps |= 1 << 13; // SVE
        //}
    }

    #[cfg(target_arch = "arm")]
    {
        // HWCAP bit definitions for arm
        const HWCAP_NEON: usize = 1 << 12;

        // Set NEON if present
        if hwcap & HWCAP_NEON != 0 {
            caps |= 1 << 0; // NEON
        }

        // Check secondary capabilities if available
        if let Some(hwcap2) = hwcap2 {
            const HWCAP2_AES: usize = 1 << 0;
            const HWCAP2_PMULL: usize = 1 << 1;

            if hwcap2 & HWCAP2_PMULL != 0 {
                caps |= 1 << 5; // PMULL
            }
        }
    }

    caps
}

/// Initializes the global ARM CPU capabilities variable (`OPENSSL_armcap_P`).
///
/// This function detects hardware features at runtime and sets the global bitmask.
/// It is safe to call multiple times; initialization will only occur once.
///
/// On Linux/Android, uses `get_auxv` to read hardware capability bitfields.
/// On Windows and macOS (aarch64), sets a default set of capabilities.
pub fn init() {
    // Ensure initialization only happens once
    if INITIALIZED.swap(true, Ordering::Relaxed) {
        return;
    }
    #[cfg(all(any(target_os = "linux", target_os = "android")))]
    {
        // Read AT_HWCAP and AT_HWCAP2 from auxiliary vector
        let hwcap = get_auxv(16).unwrap_or(0); // AT_HWCAP

        #[cfg(target_arch = "arm")]
        let hwcap2 = get_auxv(26); // AT_HWCAP2 (optional)

        #[cfg(target_arch = "aarch64")]
        let hwcap2 = None;

        let caps = detect_arm_capabilities(hwcap, hwcap2);
        unsafe {
            OPENSSL_armcap_P = caps;
        }
    }

    #[cfg(all(any(target_os = "macos", target_os = "windows"), target_arch = "aarch64"))]
    {
        // Assume common features are present on Windows aarch64
        unsafe {
            OPENSSL_armcap_P = (1 << 0) | (1 << 3) | (1 << 4);
        }
    }
    INITIALIZED.store(true, Ordering::Release);
}
