use crate::{
    audio::{RingStats, RingStatsAudio},
    rtsp_frame::RtspResult,
};

/// Represents consumer side of audio ring, providing PCM audio samples for playback.
pub trait AudioSource: Send + 'static {
    type Sample;

    /// Retrieve number of buffered samples.
    fn stat(&mut self) -> RingStatsAudio;

    /// Retrieve number of buffered samples.
    fn stat_raw(&mut self) -> RingStats;

    /// Read as many audio samples as possible into the buffer,
    /// while internally filling underflows and overflow with silence, or using loss concealment supported by the codec.
    ///
    /// The caller should NOT attempt to duplicate internal logic by padding the buffer with silence in any way.
    ///
    /// The amount of samples written to the buffer will ALWAYS be equal to the capacity of provided slice.
    ///
    /// This, along with internal audio ring implementation, ensures a high-precision audio playback - where we play
    /// given samples at their EXACT expected timestamp, or if they are lost, we play nothing at all (silence).
    ///
    /// The player is started when `write_head >= stream_latency` with goal of always keeping `write_head - read_head = stream_latency` distance.
    ///
    /// Small deviations from that distance will be automatically adjusted internally using resampling.
    fn read(&mut self, slice: &mut [Self::Sample]) -> bool;
}

pub type AudioSourceBox<S> = Box<dyn AudioSource<Sample = S>>;

/// Represents an audio player - a sink for decoded PCM samples - like ALSA.
pub trait AudioPlayer: Send + 'static {
    type Sample;

    /// Always called(only once, at init) before call to start/stop to pass exclusive ownership(via proxy) of lock-free consumer side of audio ring.
    fn init(&mut self, source: AudioSourceBox<Self::Sample>) -> RtspResult<()>;

    /// Signals to start consuming audio samples as soon as possible.
    ///
    /// See [AudioSource::read].
    fn start(&mut self);

    /// Signals to stop consuming audio samples; preferably - closing all resources synchronously before this function returns.
    ///
    /// In the future, start may be called again to resume consumption.
    ///
    /// See [AudioSource::read].
    fn stop(&mut self, drain: bool);
}

pub type AudioPlayerBox<S> = Box<dyn AudioPlayer<Sample = S>>;
