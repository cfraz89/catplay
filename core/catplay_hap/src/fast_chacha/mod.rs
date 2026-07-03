pub mod fallback_chacha20;
mod fast_chacha20;
pub use fast_chacha20::*;

// pub mod fast_chacha20poly1305;
mod fast_poly1305;
pub use fast_poly1305::*;

#[cfg(fast_chacha_asm)]
mod cpucaps;
#[cfg(fast_chacha_asm)]
pub use cpucaps::init as init_cpu_caps;

#[cfg(not(fast_chacha_asm))]
/// No-op CPU capabilities initialization when assembly optimizations are not enabled.
fn init_cpu_caps() {
    // No-op when assembly optimizations are not enabled
}
