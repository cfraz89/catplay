use std::ops::{Deref, DerefMut};

use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper, EventToken};

pub struct TeardownGuard<T> {
    notify: Option<Box<dyn Send + FnOnce()>>,
    stream: T,
}

impl<T> TeardownGuard<T> {
    pub fn new(stream: T, notify: impl Send + FnOnce() + 'static) -> Self {
        Self {
            stream,
            notify: Some(Box::new(notify)),
        }
    }

    pub fn noop(stream: T) -> Self {
        Self::new(stream, || {})
    }
}

impl<T> Drop for TeardownGuard<T> {
    fn drop(&mut self) {
        if let Some(notify) = self.notify.take() {
            (notify)();
        }
    }
}

impl<T> AsRef<T> for TeardownGuard<T> {
    fn as_ref(&self) -> &T {
        &self.stream
    }
}

impl<T> AsMut<T> for TeardownGuard<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.stream
    }
}

impl<T> Deref for TeardownGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl<T> DerefMut for TeardownGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

impl<T: EventSleeper> EventSleeper for TeardownGuard<T> {
    async fn sleep(&mut self) -> Option<EventToken> {
        self.stream.sleep().await
    }
}

impl<T: AsyncShutdown> AsyncShutdown for TeardownGuard<T> {
    async fn shutdown(&mut self) {
        self.stream.shutdown().await
    }
}

impl<T: EventReconciler> EventReconciler for TeardownGuard<T> {
    type Error = T::Error;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        self.stream.reconcile().await
    }
}
