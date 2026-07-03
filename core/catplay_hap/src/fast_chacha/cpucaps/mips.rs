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

//! MIPS CPU capabilities detection module.
//!
//! This module provides detection of MIPS CPU features at runtime,
//! setting capability flags based on hardware support as reported by the OS.
//! The detected capabilities are stored in a global variable for use by
//! cryptographic or performance-sensitive code.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused)]

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[cfg(any(target_os = "linux", target_os = "android"))]
use super::get_auxv;

/// Global variable storing detected MIPS CPU capabilities.
/// The value is set during initialization and used by other modules.
#[unsafe(no_mangle)]
pub static mut OPENSSL_mips_cap_P: u32 = 0;

/// Atomic flag indicating whether CPU capabilities have been initialized.
pub static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Indicates whether the target architecture is 64-bit MIPS.
#[cfg(any(target_arch = "mips", target_arch = "mips32r6"))]
const IS_64BIT: bool = false;
#[cfg(any(target_arch = "mips64", target_arch = "mips64r6"))]
const IS_64BIT: bool = true;

/// Detects MIPS CPU capabilities from the given hardware capability bitmask.
///
/// # Arguments
///
/// * `hwcap` - Bitmask of hardware capabilities, typically obtained from the OS.
///
/// # Returns
///
/// Returns a bitmask of detected MIPS CPU features.
fn detect_mips_capabilities(hwcap: usize) -> u32 {
    let mut caps = 0u32;

    // Hardware capability flags (from Linux kernel headers)
    const HWCAP_MIPS_R6: usize = 1 << 0;
    const HWCAP_MIPS_MSA: usize = 1 << 1;
    const HWCAP_MIPS_DSP: usize = 1 << 2;
    const HWCAP_MIPS_DSP2: usize = 1 << 3;

    // Internal capability flags
    const MIPS_CPU_R6: u32 = 1 << 0;
    const MIPS_CPU_MSA: u32 = 1 << 1;
    const MIPS_CPU_DSP: u32 = 1 << 2;
    const MIPS_CPU_DSP2: u32 = 1 << 3;

    // Set capability bits based on detected hardware features
    if hwcap & HWCAP_MIPS_R6 != 0 {
        caps |= MIPS_CPU_R6;
    }
    if hwcap & HWCAP_MIPS_MSA != 0 {
        caps |= MIPS_CPU_MSA;
    }
    if hwcap & HWCAP_MIPS_DSP != 0 {
        caps |= MIPS_CPU_DSP;
    }
    if hwcap & HWCAP_MIPS_DSP2 != 0 {
        caps |= MIPS_CPU_DSP2;
    }

    caps
}

/// Initializes MIPS CPU capability detection.
///
/// This function reads hardware capability flags from the OS (via `get_auxv`),
/// detects supported CPU features, and stores the result in the global
/// `OPENSSL_mips_cap_P` variable. Initialization is performed only once.
///
/// This function is safe to call multiple times; only the first call will
/// perform detection.
pub fn init() {
    // Ensure initialization is performed only once
    if INITIALIZED.swap(true, Ordering::Relaxed) {
        return;
    }
    {
        // 16 is AT_HWCAP on Linux
        let hwcap = get_auxv(16).unwrap_or(0); // AT_HWCAP
        let caps = detect_mips_capabilities(hwcap);
        unsafe {
            OPENSSL_mips_cap_P = caps;
        }
    }
    INITIALIZED.store(true, Ordering::Release);
}
