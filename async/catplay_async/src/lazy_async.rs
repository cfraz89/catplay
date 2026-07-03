use std::pin::Pin;

use crate::{EventSleeper, EventToken, event_select, filling_slot_value};

/// Reduces boilerplate for a common flow that involves storing a [Future], polling it regularly
/// and replacing it with it's returned value after completion.
pub struct LazyAsync<T: Send + 'static> {
    fut: Option<Pin<Box<dyn 'static + Send + Future<Output = T>>>>,
    val: Option<T>,
}

impl<T: Send + 'static> Default for LazyAsync<T> {
    /// Default value that never results in any completion; removes need for `Option<LazyAsync<T>>` boilerplate
    fn default() -> Self {
        Self { fut: None, val: None }
    }
}

impl<T: Send + 'static> LazyAsync<T> {
    pub fn new<F: 'static + Sized + Send + Future<Output = T>>(callback: impl 'static + Send + FnOnce() -> F) -> Self {
        Self {
            fut: Some(Box::pin((callback)())),
            val: None,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.val.is_some()
    }

    pub fn take(&mut self) -> Option<T> {
        self.val.take()
    }

    pub fn reset<F: 'static + Sized + Send + Future<Output = T>>(&mut self, callback: impl 'static + Send + FnOnce() -> F) {
        *self = Self::new(callback);
    }
}

impl<T: Send + 'static> AsRef<Option<T>> for LazyAsync<T> {
    fn as_ref(&self) -> &Option<T> {
        &self.val
    }
}

impl<T: Send + 'static> AsMut<Option<T>> for LazyAsync<T> {
    fn as_mut(&mut self) -> &mut Option<T> {
        &mut self.val
    }
}

impl<T: Send + 'static> EventSleeper for LazyAsync<T> {
    #[inline(always)]
    async fn sleep(&mut self) -> Option<EventToken> {
        let fut = self.fut.as_mut()?;
        event_select!(filling_slot_value(&mut self.val, fut));
        self.fut.take();
        Some(EventToken(1))
    }
}
