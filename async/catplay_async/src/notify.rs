use crate::{EventSleeper, EventToken};
use futures::task::AtomicWaker;
use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};

#[derive(Clone)]
pub struct Notify {
    inner: Arc<NotifyInner>,
}

struct NotifyInner {
    notified: AtomicBool,
    waker: AtomicWaker,
}

struct NotifySleep {
    inner: Arc<NotifyInner>,
}

impl Future for NotifySleep {
    type Output = Option<EventToken>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.inner.notified.swap(false, Ordering::Acquire) {
            return Poll::Ready(Some(EventToken(1)));
        }

        self.inner.waker.register(cx.waker());

        if self.inner.notified.swap(false, Ordering::Acquire) {
            Poll::Ready(Some(EventToken(1)))
        } else {
            Poll::Pending
        }
    }
}

impl Notify {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(NotifyInner {
                notified: AtomicBool::new(false),
                waker: AtomicWaker::new(),
            }),
        }
    }

    pub fn notify(&self) {
        self.inner.notified.store(true, Ordering::Release);
        self.inner.waker.wake();
    }
}

impl Default for Notify {
    fn default() -> Self {
        Self::new()
    }
}

impl EventSleeper for Notify {
    fn sleep(&mut self) -> impl Send + Future<Output = Option<EventToken>> {
        NotifySleep {
            inner: Arc::clone(&self.inner),
        }
    }
}
