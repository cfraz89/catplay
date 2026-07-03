use crate::{
    AudioStreamBasicDescription,
    codec::{
        AudioEncoder, AudioEncoderFactory,
        pcm::{PcmDecoder, PcmError},
    },
};

pub struct PcmEncoder {
    input: AudioStreamBasicDescription,
    output: AudioStreamBasicDescription,
}

impl AudioEncoderFactory for PcmEncoder {
    type Error = PcmError;

    fn new(input: AudioStreamBasicDescription, output: AudioStreamBasicDescription) -> Result<Self, Self::Error> {
        Ok(Self { input, output })
    }
}

impl AudioEncoder for PcmEncoder {
    type Error = PcmError;
    type Sample = i16;

    fn output_type(&self) -> AudioStreamBasicDescription {
        self.output
    }

    fn input_type(&self) -> AudioStreamBasicDescription {
        self.input
    }

    fn encode(&mut self, samples: &[Self::Sample], output: &mut [u8]) -> Result<(usize, usize), Self::Error> {
        let taken_samples = (output.len() / 2).min(samples.len());
        if taken_samples == 0 {
            return Ok((0, 0));
        }

        PcmDecoder::copy_samples_to_pcm16_be(&samples[..taken_samples], &mut output[..taken_samples * 2]);

        Ok((taken_samples * 2, taken_samples))
    }
}
