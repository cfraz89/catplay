use std::{
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use crate::{EventSleeper, EventToken};

pub struct Sleep(tokio::time::Sleep);
impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: `self` is pinned, this is just projection
        unsafe { self.map_unchecked_mut(|s| &mut s.0) }.poll(cx)
    }
}

#[must_use = "aborts on drop"]
pub fn sleep(duration: Duration) -> Sleep {
    Sleep(tokio::time::sleep(duration))
}

#[must_use = "aborts on drop"]
pub fn sleep_until(deadline: Instant) -> Sleep {
    Sleep(tokio::time::sleep_until(deadline.into()))
}

pub enum Deadline {
    SleepUntil(Instant),
    Now,
    Never,
}

impl Deadline {
    pub fn new(deadline: Instant) -> Self {
        Self::SleepUntil(deadline)
    }
}

impl EventSleeper for Deadline {
    async fn sleep(&mut self) -> Option<EventToken> {
        match self {
            Deadline::SleepUntil(instant) => {
                let _ = sleep_until(*instant).await;
                Some(EventToken(1))
            }
            Deadline::Now => Some(EventToken(1)),
            Deadline::Never => None,
        }
    }
}

pub fn deadline(deadline: Instant) -> Deadline {
    Deadline::new(deadline)
}

pub fn deadline_after(time: Duration) -> Deadline {
    if time == Duration::MAX {
        return Deadline::Never;
    }
    if time == Duration::ZERO {
        return Deadline::Now;
    }

    match Instant::now().checked_add(time) {
        Some(v) => Deadline::SleepUntil(v),
        // Deal with Duration::MAX
        None => Deadline::Never,
    }
}

pub fn deadline_maybe(time: Option<Duration>) -> Deadline {
    deadline_after(time.unwrap_or(Duration::MAX))
}
