use std::time::Duration;

use catplay_tokio::TcpHelper;
use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper, EventToken};
use log::{debug, trace};

use crate::{
    carplay_tx::{AirPlayRtspClient, StreamStateError, TeardownTask},
    msg::StreamType,
    rtp::RtpReceiver,
    rtsp_frame::RtspResult,
    screen::tx::ScreenTransmitSession,
};

pub(super) enum StreamState<T> {
    Unconnected {
        stream_type: StreamType,
    },
    Stream {
        stream: T,
        latency: Duration,
        stream_type: StreamType,
        uuid: Option<String>,
    },
    Teardown {
        stream_type: StreamType,
        task: TeardownTask,
    },
}

impl<T> StreamState<T> {
    pub fn new(stream_type: StreamType) -> Self {
        Self::Unconnected { stream_type }
    }

    pub fn connect(&mut self, stream: T, latency: Duration) {
        *self = Self::Stream {
            stream,
            latency,
            stream_type: self.stream_type(),
            uuid: None,
        }
    }

    pub fn connect_with_uuid(&mut self, stream: T, latency: Duration, uuid: String) {
        *self = Self::Stream {
            stream,
            latency,
            stream_type: self.stream_type(),
            uuid: Some(uuid),
        }
    }

    pub async fn drain_teardown(&mut self) -> Option<RtspResult<()>> {
        let stream_type = self.stream_type();

        match self {
            StreamState::Teardown { task, .. } => {
                trace!("drain_teardown: {:?}", stream_type);
                let ret = task.wait().await;
                trace!("drain_teardown post-await: {:?}", stream_type);

                self.reset();
                Some(ret)
            }
            _ => None,
        }
    }

    pub fn is_connected(&self) -> bool {
        !matches!(self, StreamState::Unconnected { .. })
    }

    pub fn stream_type(&self) -> StreamType {
        match self {
            StreamState::Unconnected { stream_type } => *stream_type,
            StreamState::Stream { stream_type, .. } => *stream_type,
            StreamState::Teardown { stream_type, .. } => *stream_type,
        }
    }

    pub fn reset(&mut self) {
        *self = StreamState::Unconnected {
            stream_type: self.stream_type(),
        };
    }

    pub fn take(&mut self) -> Option<(T, Duration, Option<String>)> {
        let old = std::mem::replace(
            self,
            StreamState::Unconnected {
                stream_type: self.stream_type(),
            },
        );

        match old {
            StreamState::Stream { stream, latency, uuid, .. } => Some((stream, latency, uuid)),
            _ => {
                *self = old;
                None
            }
        }
    }
}

impl StreamState<RtpReceiver> {
    pub fn teardown_audio(&mut self, client: AirPlayRtspClient, immediate: bool) {
        debug!("Creating TEARDOWN task for {:?}", self.stream_type());
        if let Some((stream, latency, _)) = self.take() {
            *self = StreamState::Teardown {
                stream_type: self.stream_type(),
                task: TeardownTask::audio(client, latency, stream, self.stream_type(), immediate),
            }
        }
    }
}

impl StreamState<TcpHelper<ScreenTransmitSession>> {
    pub fn teardown_screen(&mut self, client: AirPlayRtspClient, immediate: bool) {
        debug!("Creating TEARDOWN task for {:?}", self.stream_type());
        if let Some((stream, latency, Some(uuid))) = self.take() {
            *self = StreamState::Teardown {
                stream_type: self.stream_type(),
                task: TeardownTask::screen(client, latency, stream, uuid, self.stream_type(), immediate),
            }
        }
    }
}

impl<T: EventSleeper> EventSleeper for StreamState<T> {
    async fn sleep(&mut self) -> Option<EventToken> {
        match self {
            StreamState::Stream { stream, .. } => stream.sleep().await,
            StreamState::Teardown { task, .. } => task.sleep().await,
            _ => None,
        }
    }
}

impl<T: EventReconciler<Error: Into<StreamStateError>>> EventReconciler for StreamState<T> {
    type Error = StreamStateError;

    async fn reconcile(&mut self) -> Result<(), StreamStateError> {
        match self {
            StreamState::Stream { stream, .. } => {
                if let Err(err) = stream.reconcile().await {
                    return Err(err.into());
                }
            }
            StreamState::Teardown { task, .. } => {
                if let Err(err) = task.reconcile().await {
                    return Err(StreamStateError::Teardown(err));
                }

                if task.completed.is_some() {
                    self.reset();
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl<T: AsyncShutdown> AsyncShutdown for StreamState<T> {
    async fn shutdown(&mut self) {
        // Shutdown without TEARDOWN

        if let StreamState::Stream { stream, .. } = self {
            stream.shutdown().await;
        }
    }
}
