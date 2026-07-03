use log::trace;

use crate::{
    AudioStreamBasicDescription,
    codec::{AudioEncoder, AudioEncoderFactory, opus::OpusError},
};

pub struct OpusEncoder {
    opus: opus2::Encoder,
    buffer: Vec<i16>,

    output: AudioStreamBasicDescription,
    input: AudioStreamBasicDescription,
    samples_per_packet: usize,
}

impl AudioEncoderFactory for OpusEncoder {
    type Error = OpusError;

    fn new(input: AudioStreamBasicDescription, output: AudioStreamBasicDescription) -> Result<Self, Self::Error> {
        let channels = output.channels_per_frame.clamp(1, 2) as usize;
        let samples_per_packet = output.frames_per_packet as usize * channels;

        let opus = opus2::Encoder::new(
            output.sample_rate,
            if channels == 2 {
                opus2::Channels::Stereo
            } else {
                opus2::Channels::Mono
            },
            opus2::Application::Voip,
        )?;

        Ok(Self {
            opus,
            buffer: Vec::with_capacity(samples_per_packet),
            output,
            input,
            samples_per_packet,
        })
    }
}

impl AudioEncoder for OpusEncoder {
    type Error = OpusError;
    type Sample = i16;

    fn output_type(&self) -> AudioStreamBasicDescription {
        self.output
    }

    fn input_type(&self) -> AudioStreamBasicDescription {
        self.input
    }

    fn encode(&mut self, samples: &[Self::Sample], output: &mut [u8]) -> Result<(usize, usize), Self::Error> {
        let samples_per_packet = self.samples_per_packet;
        let mut consumed = 0;
        let mut written_total = 0;

        // Opus expects exact number of samples for a single frame, and we might receive less, so we allow internal buffering here
        // As a fast-path, recorder will usually call with exact number of expected samples, so we skip buffering then
        if self.buffer.is_empty() && samples.len().is_multiple_of(samples_per_packet) {
            trace!("Opus encoder: multi-frame fast path");

            let mut out = output;

            for frame in samples.chunks_exact(samples_per_packet) {
                if out.is_empty() {
                    break;
                }

                let written = self.opus.encode(frame, out)?;
                written_total += written;
                consumed += samples_per_packet;

                if written > out.len() {
                    break;
                }
                out = &mut out[written..];
            }

            return Ok((written_total, consumed));
        }

        let missing = samples_per_packet.saturating_sub(self.buffer.len());
        if missing > 0 {
            let to_copy = missing.min(samples.len());

            self.buffer.extend_from_slice(&samples[..to_copy]);
            consumed += to_copy;
        }

        if self.buffer.len() < samples_per_packet {
            return Ok((0, consumed));
        }

        let written = self.opus.encode(&self.buffer, output)?;

        self.buffer.clear();
        Ok((written, consumed))
    }
}
