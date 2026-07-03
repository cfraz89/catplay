use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::clock::{MediaClockSession, NtpU64};

pub trait Clock: Send {
    fn elapsed_ns(&self) -> i128;

    fn encode_ns(&self, pts: Instant) -> i128;

    fn decode_ns(&self, ns: i128) -> Instant;
}

#[derive(Debug, Clone)]
pub struct ClockMonotonic {
    pub start: Instant,
    base_ns: Duration,
}

impl Default for ClockMonotonic {
    fn default() -> Self {
        Self::new()
    }
}

impl ClockMonotonic {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            base_ns: Duration::from_nanos(0),
        }
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
    }
}

impl Clock for ClockMonotonic {
    fn elapsed_ns(&self) -> i128 {
        self.encode_ns(Instant::now())
    }

    fn encode_ns(&self, pts: Instant) -> i128 {
        self.base_ns.as_nanos() as i128 + pts.saturating_duration_since(self.start).as_nanos() as i128
    }

    fn decode_ns(&self, ns: i128) -> Instant {
        let delta = ns - self.base_ns.as_nanos() as i128;
        if delta >= 0 {
            self.start + Duration::from_nanos(delta as u64)
        } else {
            self.start - Duration::from_nanos((-delta) as u64)
        }
    }
}

#[test]
fn test_clock() {
    let clock = ClockMonotonic::new();
    let test = clock.start + Duration::from_secs(5);

    assert_eq!(clock.encode_ns(test), 5000000000);
    assert_eq!(test, clock.decode_ns(clock.encode_ns(test)));
}

pub trait MediaClock: Send {
    /// Checks whether the clock was already synchronized with remote and can perform pts decoding.
    fn is_synchronized(&self) -> bool;

    /// Decode remote timestamp into local PLL-corrected `Instant`.
    ///
    /// `None` means that clock is not synchronized yet; a fallback of `pts = Instant::now()` may be used as alternative.
    fn decode_remote(&self, pts: NtpU64) -> Option<Instant>;

    /// Decode timestamp in local time domain.
    fn decode_local(&self, pts: NtpU64) -> Instant;

    /// Encode timestamp in local time domain.
    fn encode_local(&self, pts: Instant) -> NtpU64;

    /// Encode timestamp in remote time domain.
    ///
    /// `None` means that clock is not synchronized yet.
    fn encode_remote(&self, pts: Instant) -> Option<NtpU64>;
}

pub type MediaClockBox = Box<dyn MediaClock>;

#[derive(Clone)]
pub struct MediaClockProxy {
    clock: Arc<Mutex<MediaClockSession<ClockMonotonic>>>,
}

impl MediaClockProxy {
    pub fn new(clock: Arc<Mutex<MediaClockSession<ClockMonotonic>>>) -> Self {
        Self { clock }
    }

    pub fn boxed(&self) -> MediaClockBox {
        Box::new(self.clone())
    }
}

impl MediaClock for MediaClockProxy {
    fn is_synchronized(&self) -> bool {
        self.clock.lock().unwrap().is_synchronized()
    }

    fn decode_remote(&self, pts: NtpU64) -> Option<Instant> {
        self.clock.lock().unwrap().decode_remote(pts)
    }

    fn decode_local(&self, pts: NtpU64) -> Instant {
        self.clock.lock().unwrap().decode_local(pts)
    }

    fn encode_local(&self, pts: Instant) -> NtpU64 {
        self.clock.lock().unwrap().encode_local(pts)
    }

    fn encode_remote(&self, pts: Instant) -> Option<NtpU64> {
        self.clock.lock().unwrap().encode_remote(pts)
    }
}
