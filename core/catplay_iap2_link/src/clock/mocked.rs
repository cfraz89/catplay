// Mock clock
extern crate std;

use core::time::Duration;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::clock::Clock;

#[derive(Clone)]
pub struct MockClock {
    time: Arc<Mutex<Instant>>,
}

#[allow(unused)]
impl MockClock {
    pub fn new(start: Instant) -> Self {
        Self {
            time: Arc::new(Mutex::new(start)),
        }
    }

    pub fn advance(&self, dur: Duration) {
        let mut guard = self.time.lock().unwrap();
        *guard += dur;
    }

    pub fn set(&self, instant: Instant) {
        let mut guard = self.time.lock().unwrap();
        *guard = instant;
    }
}

impl Clock for MockClock {
    fn now(&self) -> Instant {
        let guard = self.time.lock().unwrap();
        *guard
    }
}
