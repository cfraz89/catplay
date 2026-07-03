use crate::{
    AudioCodec, AudioStreamBasicDescription,
    codec::{
        AudioDecoder, AudioDecoderFactory,
        aac::{AacDecoder, AacError},
        alac::{AlacDecoder, AlacError},
        opus::{OpusDecoder, OpusError},
        pcm::{PcmDecoder, PcmError},
    },
};

pub struct AudioDecoderDispatch {
    codec: Dispatch,
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum CodecDispatchError {
    #[error("{0}")]
    Aac(#[from] AacError),
    #[error("{0}")]
    Alac(#[from] AlacError),
    #[error("{0}")]
    Opus(#[from] OpusError),
    #[error("{0}")]
    Pcm(#[from] PcmError),

    #[error("Unsupported codec for {0:?} -> {1:?}")]
    UnsupportedCodec(AudioCodec, AudioCodec),
}
enum Dispatch {
    Aac(AacDecoder),
    Alac(AlacDecoder),
    Opus(OpusDecoder),
    Pcm(PcmDecoder),
}

impl AudioDecoder for Dispatch {
    type Error = CodecDispatchError;
    type Sample = i16;

    fn output_type(&self) -> AudioStreamBasicDescription {
        match self {
            Dispatch::Aac(dec) => dec.output_type(),
            Dispatch::Alac(dec) => dec.output_type(),
            Dispatch::Opus(dec) => dec.output_type(),
            Dispatch::Pcm(dec) => dec.output_type(),
        }
    }

    fn decode(&mut self, data: &[u8], output: &mut [Self::Sample]) -> Result<(usize, usize), Self::Error> {
        match self {
            Dispatch::Aac(dec) => Ok(dec.decode(data, output)?),
            Dispatch::Alac(dec) => Ok(dec.decode(data, output)?),
            Dispatch::Opus(dec) => Ok(dec.decode(data, output)?),
            Dispatch::Pcm(dec) => Ok(dec.decode(data, output)?),
        }
    }

    fn conceal_lost_packet(&mut self, output: &mut [Self::Sample]) -> Result<usize, Self::Error> {
        match self {
            Dispatch::Aac(dec) => Ok(dec.conceal_lost_packet(output)?),
            Dispatch::Alac(dec) => Ok(dec.conceal_lost_packet(output)?),
            Dispatch::Opus(dec) => Ok(dec.conceal_lost_packet(output)?),
            Dispatch::Pcm(dec) => Ok(dec.conceal_lost_packet(output)?),
        }
    }
}

impl AudioDecoderDispatch {
    pub fn has_runtime_support(codec: AudioCodec) -> bool {
        matches!(
            codec,
            AudioCodec::LinearPcm | AudioCodec::AppleLossless | AudioCodec::Mpeg4Aac | AudioCodec::Mpeg4AacEld | AudioCodec::Opus
        )
    }
}

impl AudioDecoderFactory for AudioDecoderDispatch {
    type Error = CodecDispatchError;

    fn new(asbd: AudioStreamBasicDescription) -> Result<Self, Self::Error> {
        let codec = match asbd.format {
            AudioCodec::LinearPcm => Dispatch::Pcm(PcmDecoder::new(asbd)?),
            AudioCodec::AppleLossless => Dispatch::Alac(AlacDecoder::new(asbd)?),
            AudioCodec::Mpeg4Aac | AudioCodec::Mpeg4AacEld => Dispatch::Aac(AacDecoder::new(asbd)?),
            AudioCodec::Opus => Dispatch::Opus(OpusDecoder::new(asbd)?),
        };
        Ok(Self { codec })
    }
}

impl AudioDecoder for AudioDecoderDispatch {
    type Error = CodecDispatchError;
    type Sample = i16;

    fn output_type(&self) -> AudioStreamBasicDescription {
        self.codec.output_type()
    }

    fn conceal_lost_packet(&mut self, output: &mut [Self::Sample]) -> Result<usize, Self::Error> {
        self.codec.conceal_lost_packet(output)
    }

    fn decode(&mut self, data: &[u8], output: &mut [Self::Sample]) -> Result<(usize, usize), Self::Error> {
        self.codec.decode(data, output)
    }
}
