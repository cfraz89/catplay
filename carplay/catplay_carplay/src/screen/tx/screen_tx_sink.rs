use std::time::Duration;

use crate::{
    rtsp_frame::RtspError,
    video::{AvccConfigExtended, EncodedVideoFrame, Pts},
};

#[derive(Clone, Debug, thiserror::Error)]
pub enum ScreenTransmitError {
    #[error("The frame was not preceded by configuration data")]
    NotConfigured,
    #[error("The frame queue is full")]
    QueueOverflow,
    #[error("Screen socket was already closed")]
    Closed,
    #[error("The first transmitted frame should be a keyframe")]
    NeedsKeyframe,
    #[error("Failed to lazy-connect screen socket: {0}")]
    FailedToConnect(RtspError),
}

pub trait ScreenTransmitSink: Send + 'static {
    // Get stream latency that was pre-declared during stream session setup.
    fn stream_latency(&self) -> Duration;

    // Get how many frames are currently sitting in the transmit buffer.
    fn queue_len(&self) -> usize;

    // Get max size of internal transmit buffer.
    fn queue_max(&self) -> usize;

    /// Checks if the transmit buffer is full at the time of call (`self.queue_len() >= self.queue_max()`).
    fn is_full(&self) -> bool;

    // Checks if the connection was closed by remote.
    fn is_closed(&self) -> bool;

    /// Checks if the stream requires a keyframe (usually true only before first frame is pushed).
    fn needs_keyframe(&self) -> bool;

    /// Set first or updated AVCC stream config.
    ///
    /// This will always need to happen before first frame.
    fn push_avcc_config(&mut self, _config: AvccConfigExtended, _pts: Pts) -> Result<(), ScreenTransmitError>;

    /// Adds a frame to transmit buffer and wakes up TCP transmit task.
    ///
    /// If the transmit queue is overflowing, because remote failed to keep up(Wi-Fi disruptions; lagging HU), this `Future` will pause until a slot
    /// in the queue is available.
    ///
    /// If you don't want to wait, you can poll the returned future only once, any non-ready status signifies queue overflow.
    ///
    /// If the channel is closed while waiting for a slot, this method will unpause with [ScreenTransmitError]=Closed.
    ///
    /// In case you don't want to wait for a slot and the queue has overflown, it's recommended to re-try with a small delay(<`self.stream_latency()`)
    /// or switch to generating keyframes with write attempts every, for example, 500ms or so, and back to generating
    /// delta frames after a successful write.
    ///
    /// **For convenience, this method will push [AvccConfigExtended] referenced in the frame if one is provided and it differs from the cached one.**
    fn push_frame(&mut self, frame: EncodedVideoFrame) -> impl Future<Output = Result<(), (ScreenTransmitError, EncodedVideoFrame)>>;
}

pub type ScreenTransmitSinkBox = Box<dyn ScreenTransmitSink>;
