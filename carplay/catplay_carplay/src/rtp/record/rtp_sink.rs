use std::num::NonZeroUsize;

use log::{debug, trace, warn};

use crate::audio::{AudioSink, RingProducer, WriteReport, Zeroable};

pub struct RtpSink<S: Zeroable> {
    ring: RingProducer<S>,
    samples_per_packet: NonZeroUsize,
    written_samples: usize,
    notify: Box<dyn FnMut() + Send>,
}

impl<S: Zeroable> RtpSink<S> {
    pub fn new(ring: RingProducer<S>, samples_per_packet: NonZeroUsize, notify: impl FnMut() + Send + 'static) -> Self {
        Self {
            ring,
            samples_per_packet,
            notify: Box::new(notify),
            written_samples: 0,
        }
    }
}

impl<S: Zeroable> AudioSink for RtpSink<S> {
    type Sample = S;

    fn write(&mut self, slice: &[Self::Sample]) {
        let written = match self.ring.write(slice) {
            WriteReport::Ok => slice.len(),
            WriteReport::Partial { written } => written,
        };

        if written < slice.len() {
            warn!("RTP recorder: partial write {written}/{}", slice.len());
        } else {
            debug!("RTP recorder: {written} samples");
        }

        let before = self.written_samples;
        self.written_samples += written;

        if before / self.samples_per_packet < self.written_samples / self.samples_per_packet {
            trace!("RTP recorder: Waking up encoder");
            (self.notify)();
        }
    }

    fn writable(&mut self) -> usize {
        self.ring.writable_slice().len()
    }
}
