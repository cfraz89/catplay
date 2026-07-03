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
 * CPU Capabilities Detection for x86/x86_64 Architectures
 *
 * This module provides detection of CPU features (such as AVX, AVX2, AVX512, etc.)
 * and exposes them in a format compatible with OpenSSL's ia32cap_P array.
 * It uses CPUID and XGETBV instructions to determine hardware and OS support
 * for various SIMD instruction sets.
 */

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused)]
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[cfg(target_arch = "x86")]
use core::arch::x86::{__cpuid, __cpuid_count};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{__cpuid, __cpuid_count};

/// Exposed CPU capability flags, compatible with OpenSSL's ia32cap_P.
/// This array is filled during initialization with detected CPU features.
#[unsafe(no_mangle)]
pub static mut OPENSSL_ia32cap_P: [u32; 5] = [0; 5];

/// Atomic flag to ensure that CPU feature detection is performed only once.
pub static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Reads the contents of the XCR register (via XGETBV instruction) on x86_64.
///
/// # Safety
/// This function uses inline assembly and should only be called on CPUs that support XGETBV.
#[inline(always)]
#[cfg(target_arch = "x86_64")]
unsafe fn xgetbv(index: u32) -> u64 {
    let eax: u32;
    let edx: u32;
    unsafe {
        core::arch::asm!(
            "xgetbv",
            in("ecx") index,
            out("eax") eax,
            out("edx") edx,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((edx as u64) << 32) | (eax as u64)
}

/// Reads the contents of the XCR register (via XGETBV instruction) on x86.
///
/// # Safety
/// This function uses inline assembly and should only be called on CPUs that support XGETBV.
#[inline(always)]
#[cfg(target_arch = "x86")]
unsafe fn xgetbv(index: u32) -> u64 {
    let eax: u32;
    let edx: u32;
    core::arch::asm!(
        ".byte 0x0f, 0x01, 0xd0", // Opcode for XGETBV
        in("ecx") index,
        out("eax") eax,
        out("edx") edx,
        options(nomem, nostack, preserves_flags)
    );
    ((edx as u64) << 32) | (eax as u64)
}

/// Initializes CPU feature detection and populates `OPENSSL_ia32cap_P`.
///
/// This function queries the CPU for supported instruction sets using CPUID and XGETBV,
/// and sets the appropriate flags in the global capability array. It ensures that
/// initialization is performed only once, even in multithreaded contexts.
pub fn init() {
    // Ensure initialization is performed only once.
    if INITIALIZED.swap(true, Ordering::Relaxed) {
        return;
    }
    unsafe {
        // Query basic feature flags (CPUID leaf 1) and extended features (CPUID leaf 7, subleaf 0)
        let c1 = __cpuid(1);
        let c7 = __cpuid_count(7, 0);

        let eax1 = c1.eax;
        let ebx1 = c1.ebx;
        let mut ecx1 = c1.ecx;
        let edx1 = c1.edx;

        let mut ebx7 = c7.ebx;
        let ecx7 = c7.ecx;
        let edx7 = c7.edx;

        // Check for OSXSAVE and AVX support
        let has_osxsave = (ecx1 & (1 << 27)) != 0;
        let has_avx = (ecx1 & (1 << 28)) != 0;

        if has_osxsave && has_avx {
            // Check if OS has enabled XMM and YMM state support via XCR0
            let xcr0 = xgetbv(0);
            let xmm_enabled = (xcr0 & 0x2) != 0;
            let ymm_enabled = (xcr0 & 0x4) != 0;

            if !(xmm_enabled && ymm_enabled) {
                // If not enabled, clear AVX and AVX2 feature bits
                ecx1 &= !(1 << 28); // AVX
                ebx7 &= !(1 << 5); // AVX2
            }
        } else {
            // If OSXSAVE or AVX not supported, clear AVX and AVX2 feature bits
            ecx1 &= !(1 << 28);
            ebx7 &= !(1 << 5);
        }

        // Check for AVX-512 support (requires OPMASK, ZMM_Hi256, Hi16_ZMM in XCR0)
        let avx512_enabled = if has_osxsave {
            let xcr0 = xgetbv(0);
            (xcr0 & 0xe0) == 0xe0 // OPMASK (0x20), ZMM_Hi256 (0x40), Hi16_ZMM (0x80)
        } else {
            false
        };

        if !avx512_enabled {
            // If AVX-512 not enabled, clear all AVX-512 related feature bits
            ebx7 &= !(1 << 16); // AVX512F
            ebx7 &= !(1 << 17); // AVX512DQ
            ebx7 &= !(1 << 21); // AVX512IFMA
            ebx7 &= !(1 << 26); // AVX512PF
            ebx7 &= !(1 << 27); // AVX512ER
            ebx7 &= !(1 << 28); // AVX512CD
            ebx7 &= !(1 << 30); // AVX512BW
            ebx7 &= !(1 << 31); // AVX512VL
        }

        // Populate the capability array with detected features
        OPENSSL_ia32cap_P[0] = edx1;
        OPENSSL_ia32cap_P[1] = ecx1;
        OPENSSL_ia32cap_P[2] = ebx7;
        OPENSSL_ia32cap_P[3] = ecx7;
        OPENSSL_ia32cap_P[4] = edx7;

        // Set bit 10 in OPENSSL_ia32cap_P[0] to indicate "CPUID is present"
        OPENSSL_ia32cap_P[0] |= 1 << 10;
    }
    // Mark initialization as complete.
    INITIALIZED.store(true, Ordering::Release);
}
