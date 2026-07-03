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

//! CPU Capabilities Detection Module
//!
//! This module provides architecture-specific detection and utilization of CPU features
//! for optimized cryptographic operations. It conditionally compiles and re-exports
//! submodules based on the target architecture, allowing the rest of the crate to use
//! the most efficient implementation available for the current platform.
//!
//! # Supported Architectures
//! - x86, x86_64: Uses the `x86` submodule for Intel/AMD CPUs.
//! - arm, aarch64: Uses the `arm` submodule for ARM CPUs.
//! - mips, mips32r6, mips64, mips64r6: Uses the `mips` submodule for MIPS CPUs.
//! - Others: Uses the `fallback` submodule as a generic implementation.
#[cfg(all(target_os = "linux", not(target_arch = "x86"), not(target_arch = "x86_64")))]
mod hwcap;
/// Re-export all public items from the `hwcap` module for Linux.
#[cfg(all(target_os = "linux", not(target_arch = "x86"), not(target_arch = "x86_64")))]
pub use hwcap::get_auxv;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;
/// Re-export all public items from the `x86` module for x86/x86_64 architectures.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use x86::*;

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
mod arm;
/// Re-export all public items from the `arm` module for ARM architectures.
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub use arm::*;

#[cfg(any(target_arch = "mips", target_arch = "mips32r6", target_arch = "mips64", target_arch = "mips64r6"))]
mod mips;
/// Re-export all public items from the `mips` module for MIPS architectures.
#[cfg(any(target_arch = "mips", target_arch = "mips32r6", target_arch = "mips64", target_arch = "mips64r6"))]
pub use mips::*;
/// Fallback for other platforms or when no ASM/cap detection is needed.
/// This module provides a generic implementation for unsupported architectures.
#[cfg(not(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "arm",
    target_arch = "aarch64",
    target_arch = "mips",
    target_arch = "mips64",
    target_arch = "mips32r6",
    target_arch = "mips64r6"
)))]
mod fallback;
/// Re-export all public items from the `fallback` module for unsupported architectures.
#[cfg(not(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "arm",
    target_arch = "aarch64",
    target_arch = "mips",
    target_arch = "mips64",
    target_arch = "mips32r6",
    target_arch = "mips64r6"
)))]
pub use fallback::*;
