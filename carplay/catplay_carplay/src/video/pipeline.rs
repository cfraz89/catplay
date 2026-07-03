use std::time::{Duration, Instant};

use async_trait::async_trait;

use crate::video::{EncodedVideoFrame, Pts, YuvDecoded};

pub trait VideoDecoder: Sized {
    type Error;
    type Output: YuvDecoded;

    fn new() -> Result<Self, Self::Error>;

    fn push(&mut self, data: &EncodedVideoFrame) -> Result<(), Self::Error>;

    fn pop(&mut self) -> Result<Option<Self::Output>, Self::Error>;

    fn last_decode_error(&self) -> Option<Instant>;

    fn last_decode_success(&self) -> Option<Instant>;
}

pub trait VideoScheduler<F>: Sized {
    type Error;
    type Receiver: VideoReceiver<F>;

    fn new() -> Result<(Self, Self::Receiver), Self::Error>;

    fn push(&mut self, data: EncodedVideoFrame) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait VideoReceiver<F>: Sized {
    async fn poll_frame(&mut self, display_latency: Duration, timeout: Duration) -> Option<F>;

    fn peek_next(&self) -> Option<Pts>;

    fn pop(&mut self, display_latency: Duration) -> Option<F>;
}
