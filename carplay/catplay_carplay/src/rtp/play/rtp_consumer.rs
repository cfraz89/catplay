use std::{num::NonZeroUsize, time::Instant};

use log::{debug, warn};

use crate::audio::{AudioSource, RingConsumer, RingStats, RingStatsAudio, Zeroable};

pub struct RtpConsumer<S: Zeroable> {
    ring: RingConsumer<S>,
    sample_rate: NonZeroUsize,
    frame_size_in_samples: NonZeroUsize,
    start: Instant,
}

impl<S: Zeroable> RtpConsumer<S> {
    pub fn new(ring: RingConsumer<S>, sample_rate: NonZeroUsize, frame_size_in_samples: NonZeroUsize, start: Instant) -> Self {
        Self {
            ring,
            sample_rate,
            frame_size_in_samples,
            start,
        }
    }

    pub fn stat(&mut self) -> RingStatsAudio {
        self.ring
            .stat()
            .as_audio_stats(self.sample_rate.get(), self.frame_size_in_samples.get(), self.start)
    }

    pub fn stat_raw(&mut self) -> RingStats {
        self.ring.stat()
    }

    pub fn read(&mut self, out: &mut [S]) -> bool {
        match self.ring.read(out) {
            Ok(_) => {
                debug!("RtpConsumer stat={}", self.stat());
                true
            }
            Err(err) => {
                warn!(
                    "Audio player XRUN, filling gaps with silence: over={} copied={} under={} | {}",
                    err.overflow,
                    err.copied,
                    err.underflow,
                    self.stat()
                );
                err.zero_fill_overflow_and_underflow(out);
                false
            }
        }
    }
}

impl<S: Zeroable> AudioSource for RtpConsumer<S> {
    type Sample = S;

    fn stat(&mut self) -> RingStatsAudio {
        self.stat()
    }

    fn stat_raw(&mut self) -> RingStats {
        self.stat_raw()
    }

    fn read(&mut self, slice: &mut [Self::Sample]) -> bool {
        self.read(slice)
    }
}
