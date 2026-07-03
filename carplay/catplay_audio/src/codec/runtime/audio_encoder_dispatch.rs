use crate::{
    AudioCodec, AudioStreamBasicDescription,
    codec::{AudioEncoder, AudioEncoderFactory, opus::OpusEncoder, pcm::PcmEncoder, runtime::CodecDispatchError},
};

pub struct AudioEncoderDispatch {
    codec: Dispatch,
}

enum Dispatch {
    // Aac(AacDecoder),
    Opus(OpusEncoder),
    Pcm(PcmEncoder),
}

impl AudioEncoder for Dispatch {
    type Error = CodecDispatchError;
    type Sample = i16;

    fn output_type(&self) -> AudioStreamBasicDescription {
        match self {
            Dispatch::Opus(dec) => dec.output_type(),
            Dispatch::Pcm(dec) => dec.output_type(),
        }
    }

    fn input_type(&self) -> AudioStreamBasicDescription {
        match self {
            Dispatch::Opus(dec) => dec.input_type(),
            Dispatch::Pcm(dec) => dec.input_type(),
        }
    }

    fn encode(&mut self, samples: &[Self::Sample], output: &mut [u8]) -> Result<(usize, usize), Self::Error> {
        match self {
            Dispatch::Opus(dec) => Ok(dec.encode(samples, output)?),
            Dispatch::Pcm(dec) => Ok(dec.encode(samples, output)?),
        }
    }
}

impl AudioEncoderDispatch {
    pub fn has_runtime_support(codec: AudioCodec) -> bool {
        matches!(
            codec,
            AudioCodec::LinearPcm | AudioCodec::Mpeg4Aac | AudioCodec::Mpeg4AacEld | AudioCodec::Opus
        )
    }
}

impl AudioEncoderFactory for AudioEncoderDispatch {
    type Error = CodecDispatchError;

    fn new(input: AudioStreamBasicDescription, output: AudioStreamBasicDescription) -> Result<Self, Self::Error> {
        if input.format != AudioCodec::LinearPcm {
            return Err(CodecDispatchError::UnsupportedCodec(input.format, output.format));
        }

        let codec = match output.format {
            AudioCodec::LinearPcm => Dispatch::Pcm(PcmEncoder::new(input, output)?),
            // AudioCodec::Mpeg4Aac | AudioCodec::Mpeg4AacEld => Dispatch::Aac?),
            AudioCodec::Opus => Dispatch::Opus(OpusEncoder::new(input, output)?),
            _ => return Err(CodecDispatchError::UnsupportedCodec(input.format, output.format)),
        };
        Ok(Self { codec })
    }
}

impl AudioEncoder for AudioEncoderDispatch {
    type Error = CodecDispatchError;
    type Sample = i16;

    fn input_type(&self) -> AudioStreamBasicDescription {
        self.codec.input_type()
    }

    fn output_type(&self) -> AudioStreamBasicDescription {
        self.codec.output_type()
    }

    fn encode(&mut self, samples: &[Self::Sample], output: &mut [u8]) -> Result<(usize, usize), Self::Error> {
        self.codec.encode(samples, output)
    }
}
