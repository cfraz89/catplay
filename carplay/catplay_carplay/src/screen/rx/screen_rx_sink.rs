use async_trait::async_trait;
use catplay_util::AsyncShutdownDyn;

use crate::{
    rtsp_frame::RtspResult,
    video::{AvccConfigExtended, EncodedVideoFrame},
};

/// Receiver sink for screen frames.
#[async_trait]
pub trait ScreenReceiverSink: Send + AsyncShutdownDyn + 'static {
    /// Initialize the sink by opening resources like the screen or GPU decoder.
    ///
    /// Any errors returned here are critical and will terminate the session.
    async fn init(&mut self) -> RtspResult<()>;

    /// Set first or updated AVCC stream config.
    ///
    /// This will always happen before first frame.
    async fn set_avcc_config(&mut self, _config: AvccConfigExtended) -> RtspResult<()> {
        Ok(())
    }

    /// A received frame is provided for consumption(to be scheduled by the user for presentation as close to PTS as possible).
    ///
    /// Any errors returned here are critical and will result in session termination.
    async fn process_frame(&mut self, frame: EncodedVideoFrame) -> RtspResult<()>;
}

pub type ScreenReceiverSinkBox = Box<dyn ScreenReceiverSink>;
