use futures::{
    Future,
    channel::oneshot::{self, Canceled},
    future::FusedFuture,
};

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::{EventSleeper, EventToken, filling_slot};

pub type RecvResult<T> = Result<T, Canceled>;

pub struct PeekReceiver<T> {
    inner: Option<oneshot::Receiver<T>>,
    cached: Option<RecvResult<T>>,
}

impl<T> PeekReceiver<T> {
    pub fn new(inner: oneshot::Receiver<T>) -> Self {
        Self {
            inner: Some(inner),
            cached: None,
        }
    }

    pub fn peek(&self) -> Option<&RecvResult<T>> {
        self.cached.as_ref()
    }

    pub fn take(&mut self) -> Option<RecvResult<T>> {
        self.cached.take()
    }

    pub fn inner(&self) -> Option<&oneshot::Receiver<T>> {
        self.inner.as_ref()
    }

    pub fn inner_mut(&mut self) -> Option<&mut oneshot::Receiver<T>> {
        self.inner.as_mut()
    }
}

impl<T> Default for PeekReceiver<T> {
    fn default() -> Self {
        Self { inner: None, cached: None }
    }
}

impl<T: Send> EventSleeper for PeekReceiver<T> {
    async fn sleep(&mut self) -> Option<EventToken> {
        let inner = &mut self.inner;

        filling_slot(&mut self.cached, async {
            let result = match inner.as_mut() {
                None => return None,
                Some(inner) => inner.await,
            };

            *inner = None;
            Some(result)
        })
        .sleep()
        .await
    }
}

impl<T: Unpin> Future for PeekReceiver<T> {
    type Output = RecvResult<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();

        if let Some(cached) = this.cached.take() {
            return Poll::Ready(cached);
        }

        let Some(inner) = this.inner.as_mut() else {
            panic!("PeekReceiver polled after completion")
        };

        match Pin::new(inner).poll(cx) {
            Poll::Ready(result) => {
                this.inner = None;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T: Unpin> FusedFuture for PeekReceiver<T> {
    fn is_terminated(&self) -> bool {
        self.cached.is_none() && self.inner.is_none()
    }
}

impl<T> Unpin for PeekReceiver<T> {}
unsafe impl<T: Send> Send for PeekReceiver<T> {}

pub type Receiver<T> = PeekReceiver<T>;
pub type Sender<T> = oneshot::Sender<T>;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let chan = oneshot::channel();
    (chan.0, PeekReceiver::new(chan.1))
}

#[cfg(test)]
mod tests {
    use crate::{EventSleeper, EventToken};

    #[tokio::test]
    async fn sleep_peek_take_received_value() {
        let (sender, mut receiver) = super::channel();

        sender.send(7).unwrap();

        assert_eq!(receiver.sleep().await, Some(EventToken(1)));
        assert_eq!(receiver.peek(), Some(&Ok(7)));
        assert_eq!(receiver.take(), Some(Ok(7)));
        assert_eq!(receiver.peek(), None);
        assert_eq!(receiver.sleep().await, None);
    }

    #[tokio::test]
    async fn sleep_peek_take_canceled_sender() {
        let (sender, mut receiver) = super::channel::<u8>();

        drop(sender);

        assert_eq!(receiver.sleep().await, Some(EventToken(1)));
        assert!(receiver.peek().is_some_and(Result::is_err));
        assert!(receiver.take().is_some_and(|result| result.is_err()));
        assert_eq!(receiver.sleep().await, None);
    }
}
