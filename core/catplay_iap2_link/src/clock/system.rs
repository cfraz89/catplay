use crate::clock::{Clock, ClockInstant};

extern crate std;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> ClockInstant {
        std::time::Instant::now()
    }
}
