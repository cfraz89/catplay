use crate::rtsp_frame::RtspResult;

/// Represents producer side of audio ring, accepting microphone input.
pub trait AudioSink: Send + 'static {
    type Sample;

    /// Returns a number of samples that can be accepted right now without overflow.
    fn writable(&mut self) -> usize;

    /// Write all provided samples into the buffer; in case of overflow, only the newest samples will be accepted.
    ///
    /// Returns number of accepted samples.
    fn write(&mut self, slice: &[Self::Sample]);
}

pub type AudioSinkBox<S> = Box<dyn AudioSink<Sample = S>>;

/// Represents an audio recorder - a producer for PCM samples - like ALSA with a microphone.
pub trait AudioRecorder: Send + 'static {
    type Sample;

    /// Always called(only once, at init) before call to start/stop to pass exclusive ownership(via proxy) of lock-free producer side of audio ring.
    fn init(&mut self, source: AudioSinkBox<Self::Sample>) -> RtspResult<()>;

    /// Signals to start consuming audio samples as soon as possible.
    ///
    /// See [AudioSource::read].
    fn start(&mut self);

    /// Signals to stop consuming audio samples; preferably - closing all resources synchronously before this function returns.
    ///
    /// In the future, start may be called again to resume consumption.
    ///
    /// See [AudioRecorder::read].
    fn stop(&mut self, drain: bool);
}

pub type AudioRecorderBox<S> = Box<dyn AudioRecorder<Sample = S>>;
