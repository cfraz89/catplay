pub trait Clock: Send + Sync {
    fn now(&self) -> ClockInstant;
}

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub type ClockInstant = std::time::Instant;
#[cfg(not(feature = "std"))]
pub type ClockInstant = i128;
